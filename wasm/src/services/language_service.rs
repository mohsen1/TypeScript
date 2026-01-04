//! Main Language Service orchestration.
//!
//! This module provides the main entry point for all language service
//! operations, orchestrating the various submodules.

use std::collections::HashMap;
use std::sync::Arc;

use crate::binder::{Symbol, SymbolTable};
use crate::checker::CheckerState;
use crate::parser::arena::NodeArena;
use crate::parser::Parser;
use crate::scanner::Scanner;

use super::code_fix_provider::{CodeFixRegistry, CodeFixAction, DiagnosticCode, FileTextChanges};
use super::codefixes::register_all_fixes;
use super::completions::{CompletionInfo, CompletionsContext, GetCompletionsAtPositionOptions, CompletionEntryDetails};
use super::document_registry::{DocumentRegistry, ScriptSnapshot};
use super::export_info_map::ExportInfoMap;
use super::find_all_references::{ReferencedSymbol, ReferenceEntry};
use super::formatting::{FormatResult, FormatOptions};
use super::go_to_definition::{DefinitionInfo, DefinitionInfoAndBoundSpan};
use super::inlay_hints::{InlayHint, InlayHintsPreferences};
use super::navigation_bar::{NavigationBarItem, NavigationTree};
use super::quick_info::QuickInfo;
use super::refactor_provider::{RefactorRegistry, ApplicableRefactorInfo, RefactorEditInfo};
use super::refactors::register_all_refactors;
use super::rename::{RenameInfo, RenameLocation};
use super::signature_help::SignatureHelpItems;
use super::text_span::TextSpan;

// =============================================================================
// Language Service Host
// =============================================================================

/// The host interface that provides the language service with project information.
pub trait LanguageServiceHost: Send + Sync {
    /// Get the compilation settings.
    fn get_compilation_settings(&self) -> CompilerOptions;

    /// Get all script file names in the project.
    fn get_script_file_names(&self) -> Vec<String>;

    /// Get the version of a script (for caching).
    fn get_script_version(&self, file_name: &str) -> String;

    /// Get the content of a script.
    fn get_script_snapshot(&self, file_name: &str) -> Option<Box<dyn ScriptSnapshot>>;

    /// Get the current directory.
    fn get_current_directory(&self) -> String;

    /// Get the default lib file name.
    fn get_default_lib_file_name(&self, options: &CompilerOptions) -> String;

    /// Read a file.
    fn read_file(&self, file_name: &str) -> Option<String>;

    /// Check if a file exists.
    fn file_exists(&self, file_name: &str) -> bool;

    /// Get directories.
    fn get_directories(&self, path: &str) -> Vec<String>;

    /// Resolve module names.
    fn resolve_module_names(
        &self,
        module_names: &[String],
        containing_file: &str,
    ) -> Vec<Option<ResolvedModule>>;
}

/// A resolved module.
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub resolved_file_name: String,
    pub is_external_library_import: bool,
}

/// Compiler options.
#[derive(Debug, Clone, Default)]
pub struct CompilerOptions {
    pub target: ScriptTarget,
    pub module: ModuleKind,
    pub strict: bool,
    pub no_implicit_any: bool,
    pub strict_null_checks: bool,
    pub strict_function_types: bool,
    pub strict_property_initialization: bool,
    pub no_unused_locals: bool,
    pub no_unused_parameters: bool,
    pub declaration: bool,
    pub source_map: bool,
    pub out_dir: Option<String>,
    pub root_dir: Option<String>,
    pub base_url: Option<String>,
    pub paths: Option<HashMap<String, Vec<String>>>,
}

/// Script target (ES version).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScriptTarget {
    ES3,
    ES5,
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    #[default]
    ESNext,
}

/// Module kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleKind {
    None,
    CommonJS,
    AMD,
    UMD,
    System,
    ES2015,
    ES2020,
    ES2022,
    #[default]
    ESNext,
    Node16,
    NodeNext,
}

// =============================================================================
// Language Service Mode
// =============================================================================

/// The mode of the language service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LanguageServiceMode {
    /// Full semantic support.
    #[default]
    Semantic,
    /// Syntactic only (faster, no type checking).
    Syntactic,
    /// Partial semantic (some type inference).
    PartialSemantic,
}

// =============================================================================
// Language Service
// =============================================================================

/// The main language service implementation.
pub struct LanguageService {
    /// The host providing project information.
    host: Arc<dyn LanguageServiceHost>,
    /// Document registry for caching parsed files.
    document_registry: DocumentRegistry,
    /// Export info map for auto-imports.
    export_map: ExportInfoMap,
    /// Code fix registry.
    code_fix_registry: CodeFixRegistry,
    /// Refactor registry.
    refactor_registry: RefactorRegistry,
    /// The language service mode.
    mode: LanguageServiceMode,
}

impl LanguageService {
    /// Create a new language service.
    pub fn new(
        host: Arc<dyn LanguageServiceHost>,
        mode: LanguageServiceMode,
    ) -> Self {
        let document_registry = DocumentRegistry::new();
        let export_map = ExportInfoMap::new();

        let mut code_fix_registry = CodeFixRegistry::new();
        register_all_fixes(&mut code_fix_registry);

        let mut refactor_registry = RefactorRegistry::new();
        register_all_refactors(&mut refactor_registry);

        Self {
            host,
            document_registry,
            export_map,
            code_fix_registry,
            refactor_registry,
            mode,
        }
    }

    /// Create a language service with syntactic-only mode.
    pub fn create_syntactic_only(host: Arc<dyn LanguageServiceHost>) -> Self {
        Self::new(host, LanguageServiceMode::Syntactic)
    }

    // =========================================================================
    // Synchronization
    // =========================================================================

    /// Synchronize the project with the host.
    pub fn synchronize(&mut self) {
        let file_names = self.host.get_script_file_names();

        for file_name in file_names {
            self.ensure_file_up_to_date(&file_name);
        }
    }

    fn ensure_file_up_to_date(&mut self, file_name: &str) {
        let version = self.host.get_script_version(file_name);

        // Check if we need to update the cached file
        if !self.document_registry.is_up_to_date(file_name, &version) {
            if let Some(snapshot) = self.host.get_script_snapshot(file_name) {
                self.document_registry.update_document(
                    file_name,
                    &version,
                    snapshot,
                );
            }
        }
    }

    fn get_source_file(&self, file_name: &str) -> Option<&NodeArena> {
        self.document_registry.get_source_file(file_name)
    }

    // =========================================================================
    // Go To Definition
    // =========================================================================

    /// Get definition at a position.
    pub fn get_definition_at_position(
        &mut self,
        file_name: &str,
        position: u32,
    ) -> Option<Vec<DefinitionInfo>> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let checker = self.get_checker(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let ctx = super::go_to_definition::GoToDefinitionContext {
            arena,
            checker: &checker,
            source_text: &source_text,
            file_name,
        };

        super::go_to_definition::get_definition_at_position(&ctx, position)
    }

    /// Get definition and bound span.
    pub fn get_definition_and_bound_span(
        &mut self,
        file_name: &str,
        position: u32,
    ) -> DefinitionInfoAndBoundSpan {
        self.ensure_file_up_to_date(file_name);

        if let (Some(arena), Some(checker), Some(source_text)) = (
            self.get_source_file(file_name),
            self.get_checker(file_name),
            self.get_source_text(file_name),
        ) {
            let ctx = super::go_to_definition::GoToDefinitionContext {
                arena,
                checker: &checker,
                source_text: &source_text,
                file_name,
            };

            super::go_to_definition::get_definition_and_bound_span(&ctx, position)
        } else {
            DefinitionInfoAndBoundSpan::empty()
        }
    }

    /// Get type definition at position.
    pub fn get_type_definition_at_position(
        &mut self,
        file_name: &str,
        position: u32,
    ) -> Option<Vec<DefinitionInfo>> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let checker = self.get_checker(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let ctx = super::go_to_definition::GoToDefinitionContext {
            arena,
            checker: &checker,
            source_text: &source_text,
            file_name,
        };

        super::go_to_definition::get_type_definition_at_position(&ctx, position)
    }

    // =========================================================================
    // References
    // =========================================================================

    /// Find all references to a symbol.
    pub fn find_references(
        &mut self,
        file_name: &str,
        position: u32,
    ) -> Option<Vec<ReferencedSymbol>> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let checker = self.get_checker(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let files_to_search = self.host.get_script_file_names();

        let ctx = super::find_all_references::FindReferencesContext {
            arena,
            checker: &checker,
            source_text: &source_text,
            file_name,
            files_to_search: &files_to_search,
            options: Default::default(),
        };

        super::find_all_references::find_references_at_position(&ctx, position)
    }

    /// Get references (flattened list).
    pub fn get_references_at_position(
        &mut self,
        file_name: &str,
        position: u32,
    ) -> Option<Vec<ReferenceEntry>> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let checker = self.get_checker(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let files_to_search = self.host.get_script_file_names();

        let ctx = super::find_all_references::FindReferencesContext {
            arena,
            checker: &checker,
            source_text: &source_text,
            file_name,
            files_to_search: &files_to_search,
            options: Default::default(),
        };

        super::find_all_references::get_references_at_position(&ctx, position)
    }

    // =========================================================================
    // Quick Info
    // =========================================================================

    /// Get quick info (hover) at a position.
    pub fn get_quick_info_at_position(
        &mut self,
        file_name: &str,
        position: u32,
    ) -> Option<QuickInfo> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let checker = self.get_checker(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let ctx = super::quick_info::QuickInfoContext {
            arena,
            checker: &checker,
            source_text: &source_text,
            file_name,
        };

        super::quick_info::get_quick_info_at_position(&ctx, position)
    }

    // =========================================================================
    // Completions
    // =========================================================================

    /// Get completions at a position.
    pub fn get_completions_at_position(
        &mut self,
        file_name: &str,
        position: u32,
        options: GetCompletionsAtPositionOptions,
    ) -> Option<CompletionInfo> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let checker = self.get_checker(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let ctx = CompletionsContext {
            arena,
            checker: &checker,
            source_text: &source_text,
            file_name,
            export_map: Some(&self.export_map),
            options,
        };

        super::completions::get_completions_at_position(&ctx, position)
    }

    /// Get completion entry details.
    pub fn get_completion_entry_details(
        &mut self,
        file_name: &str,
        position: u32,
        entry_name: &str,
        source: Option<&str>,
    ) -> Option<CompletionEntryDetails> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let checker = self.get_checker(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let ctx = CompletionsContext {
            arena,
            checker: &checker,
            source_text: &source_text,
            file_name,
            export_map: Some(&self.export_map),
            options: Default::default(),
        };

        super::completions::get_completion_entry_details(&ctx, position, entry_name, source)
    }

    // =========================================================================
    // Signature Help
    // =========================================================================

    /// Get signature help at a position.
    pub fn get_signature_help_items(
        &mut self,
        file_name: &str,
        position: u32,
    ) -> Option<SignatureHelpItems> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let checker = self.get_checker(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let ctx = super::signature_help::SignatureHelpContext {
            arena,
            checker: &checker,
            source_text: &source_text,
            file_name,
            trigger_reason: super::signature_help::SignatureHelpTriggerReason::invoked(),
        };

        super::signature_help::get_signature_help_items(&ctx, position)
    }

    // =========================================================================
    // Rename
    // =========================================================================

    /// Get rename info at a position.
    pub fn get_rename_info(
        &mut self,
        file_name: &str,
        position: u32,
    ) -> RenameInfo {
        self.ensure_file_up_to_date(file_name);

        if let (Some(arena), Some(checker), Some(source_text)) = (
            self.get_source_file(file_name),
            self.get_checker(file_name),
            self.get_source_text(file_name),
        ) {
            let files_to_search = self.host.get_script_file_names();

            let ctx = super::rename::RenameContext {
                arena,
                checker: &checker,
                source_text: &source_text,
                file_name,
                files_to_search: &files_to_search,
                options: Default::default(),
            };

            super::rename::get_rename_info(&ctx, position)
        } else {
            RenameInfo::cannot_rename("Cannot find file")
        }
    }

    /// Find rename locations.
    pub fn find_rename_locations(
        &mut self,
        file_name: &str,
        position: u32,
        find_in_strings: bool,
        find_in_comments: bool,
    ) -> Option<Vec<RenameLocation>> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let checker = self.get_checker(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let files_to_search = self.host.get_script_file_names();

        let ctx = super::rename::RenameContext {
            arena,
            checker: &checker,
            source_text: &source_text,
            file_name,
            files_to_search: &files_to_search,
            options: super::rename::RenameOptions {
                find_in_strings,
                find_in_comments,
                ..Default::default()
            },
        };

        super::rename::get_rename_locations(&ctx, position)
    }

    // =========================================================================
    // Document Highlights
    // =========================================================================

    /// Get document highlights at a position.
    pub fn get_document_highlights(
        &mut self,
        file_name: &str,
        position: u32,
        files_to_search: &[String],
    ) -> Option<Vec<super::document_highlights::DocumentHighlights>> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let checker = self.get_checker(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let ctx = super::document_highlights::DocumentHighlightsContext {
            arena,
            checker: &checker,
            source_text: &source_text,
            file_name,
            files_to_search,
        };

        super::document_highlights::get_document_highlights(&ctx, position)
    }

    // =========================================================================
    // Navigation
    // =========================================================================

    /// Get navigation bar items.
    pub fn get_navigation_bar_items(
        &mut self,
        file_name: &str,
    ) -> Vec<NavigationBarItem> {
        self.ensure_file_up_to_date(file_name);

        if let (Some(arena), Some(source_text)) = (
            self.get_source_file(file_name),
            self.get_source_text(file_name),
        ) {
            let ctx = super::navigation_bar::NavigationBarContext {
                arena,
                source_text: &source_text,
                file_name,
            };

            super::navigation_bar::get_navigation_bar_items(&ctx)
        } else {
            Vec::new()
        }
    }

    /// Get navigation tree.
    pub fn get_navigation_tree(
        &mut self,
        file_name: &str,
    ) -> Option<NavigationTree> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let ctx = super::navigation_bar::NavigationBarContext {
            arena,
            source_text: &source_text,
            file_name,
        };

        super::navigation_bar::get_navigation_tree(&ctx)
    }

    // =========================================================================
    // Inlay Hints
    // =========================================================================

    /// Get inlay hints for a range.
    pub fn provide_inlay_hints(
        &mut self,
        file_name: &str,
        span: TextSpan,
        preferences: InlayHintsPreferences,
    ) -> Vec<InlayHint> {
        self.ensure_file_up_to_date(file_name);

        if let (Some(arena), Some(checker), Some(source_text)) = (
            self.get_source_file(file_name),
            self.get_checker(file_name),
            self.get_source_text(file_name),
        ) {
            let ctx = super::inlay_hints::InlayHintsContext {
                arena,
                checker: &checker,
                source_text: &source_text,
                file_name,
                preferences,
                start: span.start,
                end: span.start + span.length,
            };

            super::inlay_hints::provide_inlay_hints(&ctx)
        } else {
            Vec::new()
        }
    }

    // =========================================================================
    // Call Hierarchy
    // =========================================================================

    /// Prepare call hierarchy at a position.
    pub fn prepare_call_hierarchy(
        &mut self,
        file_name: &str,
        position: u32,
    ) -> Option<Vec<super::call_hierarchy::CallHierarchyItem>> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let checker = self.get_checker(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let files_to_search = self.host.get_script_file_names();

        let ctx = super::call_hierarchy::CallHierarchyContext {
            arena,
            checker: &checker,
            source_text: &source_text,
            file_name,
            files_to_search: &files_to_search,
        };

        super::call_hierarchy::prepare_call_hierarchy(&ctx, position)
    }

    // =========================================================================
    // Code Fixes
    // =========================================================================

    /// Get code fixes at a position.
    pub fn get_code_fixes_at_position(
        &mut self,
        file_name: &str,
        start: u32,
        end: u32,
        error_codes: &[u32],
    ) -> Vec<CodeFixAction> {
        self.ensure_file_up_to_date(file_name);

        let mut all_fixes = Vec::new();

        if let (Some(arena), Some(checker), Some(source_text)) = (
            self.get_source_file(file_name),
            self.get_checker(file_name),
            self.get_source_text(file_name),
        ) {
            for &code in error_codes {
                let ctx = super::code_fix_provider::CodeFixContext {
                    arena,
                    checker: &checker,
                    source_text: &source_text,
                    file_name,
                    error_code: DiagnosticCode(code),
                    span: TextSpan::from_bounds(start, end),
                    preferences: Default::default(),
                };

                let fixes = self.code_fix_registry.get_code_fixes(&ctx);
                all_fixes.extend(fixes);
            }
        }

        all_fixes
    }

    // =========================================================================
    // Refactorings
    // =========================================================================

    /// Get applicable refactorings.
    pub fn get_applicable_refactors(
        &mut self,
        file_name: &str,
        position_or_range: TextSpan,
    ) -> Vec<ApplicableRefactorInfo> {
        self.ensure_file_up_to_date(file_name);

        if let (Some(arena), Some(checker), Some(source_text)) = (
            self.get_source_file(file_name),
            self.get_checker(file_name),
            self.get_source_text(file_name),
        ) {
            let ctx = super::refactor_provider::RefactorContext {
                arena,
                checker: &checker,
                source_text: &source_text,
                file_name,
                start_position: position_or_range.start,
                end_position: if position_or_range.length > 0 {
                    Some(position_or_range.start + position_or_range.length)
                } else {
                    None
                },
                preferences: Default::default(),
                trigger_kind: Default::default(),
            };

            self.refactor_registry.get_applicable_refactors(&ctx, None)
        } else {
            Vec::new()
        }
    }

    /// Get edits for a refactoring.
    pub fn get_edits_for_refactor(
        &mut self,
        file_name: &str,
        position_or_range: TextSpan,
        refactor_name: &str,
        action_name: &str,
    ) -> Option<RefactorEditInfo> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let checker = self.get_checker(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let ctx = super::refactor_provider::RefactorContext {
            arena,
            checker: &checker,
            source_text: &source_text,
            file_name,
            start_position: position_or_range.start,
            end_position: if position_or_range.length > 0 {
                Some(position_or_range.start + position_or_range.length)
            } else {
                None
            },
            preferences: Default::default(),
            trigger_kind: Default::default(),
        };

        self.refactor_registry.get_edits_for_refactor(&ctx, refactor_name, action_name)
    }

    // =========================================================================
    // Formatting
    // =========================================================================

    /// Get formatting edits for a document.
    pub fn get_formatting_edits_for_document(
        &mut self,
        file_name: &str,
        options: FormatOptions,
    ) -> Vec<super::text_changes::TextChange> {
        self.ensure_file_up_to_date(file_name);

        if let (Some(arena), Some(source_text)) = (
            self.get_source_file(file_name),
            self.get_source_text(file_name),
        ) {
            let ctx = super::formatting::FormattingContext {
                arena,
                source_text: &source_text,
                options,
            };

            super::formatting::get_formatting_edits_for_document(&ctx)
        } else {
            Vec::new()
        }
    }

    /// Get formatting edits for a range.
    pub fn get_formatting_edits_for_range(
        &mut self,
        file_name: &str,
        start: u32,
        end: u32,
        options: FormatOptions,
    ) -> Vec<super::text_changes::TextChange> {
        self.ensure_file_up_to_date(file_name);

        if let (Some(arena), Some(source_text)) = (
            self.get_source_file(file_name),
            self.get_source_text(file_name),
        ) {
            let ctx = super::formatting::FormattingContext {
                arena,
                source_text: &source_text,
                options,
            };

            super::formatting::get_formatting_edits_for_range(&ctx, start, end)
        } else {
            Vec::new()
        }
    }

    // =========================================================================
    // Organize Imports
    // =========================================================================

    /// Organize imports in a file.
    pub fn organize_imports(
        &mut self,
        file_name: &str,
    ) -> Vec<FileTextChanges> {
        self.ensure_file_up_to_date(file_name);

        if let (Some(arena), Some(checker), Some(source_text)) = (
            self.get_source_file(file_name),
            self.get_checker(file_name),
            self.get_source_text(file_name),
        ) {
            let ctx = super::organize_imports::OrganizeImportsContext {
                arena,
                checker: &checker,
                source_text: &source_text,
                file_name,
                options: Default::default(),
                preferences: Default::default(),
            };

            super::organize_imports::organize_imports(&ctx)
        } else {
            Vec::new()
        }
    }

    // =========================================================================
    // Outlining
    // =========================================================================

    /// Get outlining spans.
    pub fn get_outlining_spans(
        &mut self,
        file_name: &str,
    ) -> Vec<super::outlining::OutliningSpan> {
        self.ensure_file_up_to_date(file_name);

        if let (Some(arena), Some(source_text)) = (
            self.get_source_file(file_name),
            self.get_source_text(file_name),
        ) {
            let ctx = super::outlining::OutliningContext {
                arena,
                source_text: &source_text,
            };

            super::outlining::get_outlining_spans(&ctx)
        } else {
            Vec::new()
        }
    }

    // =========================================================================
    // Breakpoints
    // =========================================================================

    /// Get breakpoint span at a position.
    pub fn get_breakpoint_span_at_position(
        &mut self,
        file_name: &str,
        position: u32,
    ) -> Option<super::breakpoints::BreakpointSpan> {
        self.ensure_file_up_to_date(file_name);

        let arena = self.get_source_file(file_name)?;
        let source_text = self.get_source_text(file_name)?;

        let ctx = super::breakpoints::BreakpointContext {
            arena,
            source_text: &source_text,
        };

        super::breakpoints::get_breakpoint_span_at_position(&ctx, position)
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    fn get_checker(&self, file_name: &str) -> Option<CheckerState> {
        // In a full implementation, this would return the type checker
        // for the file. For now, we return None and let calling code handle it.
        None
    }

    fn get_source_text(&self, file_name: &str) -> Option<String> {
        self.host.get_script_snapshot(file_name)
            .map(|s| s.get_text())
    }

    /// Dispose of the language service.
    pub fn dispose(&mut self) {
        self.document_registry.clear();
        self.export_map.clear();
    }
}

/// Create a language service.
pub fn create_language_service(
    host: Arc<dyn LanguageServiceHost>,
) -> LanguageService {
    LanguageService::new(host, LanguageServiceMode::Semantic)
}

/// Create a language service in syntactic-only mode.
pub fn create_language_service_syntactic(
    host: Arc<dyn LanguageServiceHost>,
) -> LanguageService {
    LanguageService::create_syntactic_only(host)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_options_default() {
        let options = CompilerOptions::default();
        assert_eq!(options.target, ScriptTarget::ESNext);
        assert_eq!(options.module, ModuleKind::ESNext);
    }

    #[test]
    fn test_language_service_mode() {
        assert_eq!(
            LanguageServiceMode::default(),
            LanguageServiceMode::Semantic
        );
    }
}
