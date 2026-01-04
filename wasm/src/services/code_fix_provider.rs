//! Code Fix Provider infrastructure.
//!
//! This module provides the core infrastructure for code fixes (quick fixes),
//! including the registration system and base types.

use std::collections::HashMap;
use crate::binder::{Symbol, SymbolFlags};
use crate::checker::CheckerState;
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::text_span::TextSpan;
use super::text_changes::TextChange;

// =============================================================================
// Code Fix Types
// =============================================================================

/// A code fix action.
#[derive(Debug, Clone)]
pub struct CodeFixAction {
    /// The unique identifier for this fix.
    pub fix_name: String,
    /// A human-readable description of the fix.
    pub description: String,
    /// The changes to apply.
    pub changes: Vec<FileTextChanges>,
    /// Commands to execute after applying changes.
    pub commands: Option<Vec<CodeActionCommand>>,
    /// The fix ID (for fix all).
    pub fix_id: Option<String>,
    /// All available fix IDs (for fix all menu).
    pub fix_all_description: Option<String>,
}

impl CodeFixAction {
    pub fn new(fix_name: String, description: String, changes: Vec<FileTextChanges>) -> Self {
        Self {
            fix_name,
            description,
            changes,
            commands: None,
            fix_id: None,
            fix_all_description: None,
        }
    }

    pub fn with_fix_all(mut self, fix_id: String, description: String) -> Self {
        self.fix_id = Some(fix_id);
        self.fix_all_description = Some(description);
        self
    }
}

/// Text changes for a single file.
#[derive(Debug, Clone)]
pub struct FileTextChanges {
    /// The file to modify.
    pub file_name: String,
    /// The changes to apply.
    pub text_changes: Vec<TextChange>,
    /// Whether this creates a new file.
    pub is_new_file: Option<bool>,
}

impl FileTextChanges {
    pub fn new(file_name: String, changes: Vec<TextChange>) -> Self {
        Self {
            file_name,
            text_changes: changes,
            is_new_file: None,
        }
    }

    pub fn new_file(file_name: String, content: String) -> Self {
        Self {
            file_name,
            text_changes: vec![TextChange::new(TextSpan::new(0, 0), content)],
            is_new_file: Some(true),
        }
    }
}

/// A command to execute after applying changes.
#[derive(Debug, Clone)]
pub struct CodeActionCommand {
    /// The command identifier.
    pub command: String,
    /// Human-readable title.
    pub title: String,
    /// Arguments to the command.
    pub arguments: Option<Vec<String>>,
}

// =============================================================================
// Diagnostic Information
// =============================================================================

/// A diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticCode(pub u32);

impl DiagnosticCode {
    // Common TypeScript error codes
    pub const CANNOT_FIND_NAME: Self = Self(2304);
    pub const PROPERTY_DOES_NOT_EXIST: Self = Self(2339);
    pub const ARGUMENT_TYPE_NOT_ASSIGNABLE: Self = Self(2345);
    pub const TYPE_NOT_ASSIGNABLE: Self = Self(2322);
    pub const MISSING_RETURN_TYPE: Self = Self(7030);
    pub const UNUSED_VARIABLE: Self = Self(6133);
    pub const UNUSED_PARAMETER: Self = Self(6138);
    pub const MISSING_PROPERTY: Self = Self(2741);
    pub const CANNOT_FIND_MODULE: Self = Self(2307);
    pub const IMPLICIT_ANY: Self = Self(7006);
    pub const NO_OVERLOAD_MATCHES: Self = Self(2769);
}

/// Information about a diagnostic.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The file containing the diagnostic.
    pub file_name: String,
    /// The span of the diagnostic.
    pub span: TextSpan,
    /// The error code.
    pub code: DiagnosticCode,
    /// The message.
    pub message: String,
    /// Related information.
    pub related_info: Vec<DiagnosticRelatedInfo>,
}

/// Related information for a diagnostic.
#[derive(Debug, Clone)]
pub struct DiagnosticRelatedInfo {
    pub file_name: String,
    pub span: TextSpan,
    pub message: String,
}

// =============================================================================
// Code Fix Context
// =============================================================================

/// Context for code fix operations.
pub struct CodeFixContext<'a> {
    pub arena: &'a NodeArena,
    pub checker: &'a CheckerState,
    pub source_text: &'a str,
    pub file_name: &'a str,
    /// The diagnostic being fixed.
    pub error_code: DiagnosticCode,
    /// The span of the error.
    pub span: TextSpan,
    /// Preferences for code fixes.
    pub preferences: CodeFixPreferences,
}

/// Preferences for code fixes.
#[derive(Debug, Clone, Default)]
pub struct CodeFixPreferences {
    /// Quote style preference.
    pub quote_style: QuoteStyle,
    /// Whether to include accessibility modifiers.
    pub include_accessibility_modifiers: bool,
    /// Whether to organize imports.
    pub organize_imports_on_fix: bool,
}

/// Quote style preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuoteStyle {
    #[default]
    Auto,
    Single,
    Double,
}

// =============================================================================
// Code Fix Registry
// =============================================================================

/// Type alias for code fix handler functions.
pub type CodeFixHandler = fn(&CodeFixContext) -> Vec<CodeFixAction>;

/// Registry of all available code fixes.
#[derive(Default)]
pub struct CodeFixRegistry {
    /// Map from error code to list of fix handlers.
    handlers: HashMap<DiagnosticCode, Vec<RegisteredFix>>,
    /// Map from fix name to handler.
    by_name: HashMap<String, RegisteredFix>,
}

#[derive(Clone)]
struct RegisteredFix {
    name: String,
    error_codes: Vec<DiagnosticCode>,
    handler: CodeFixHandler,
}

impl CodeFixRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a code fix.
    pub fn register(
        &mut self,
        name: &str,
        error_codes: &[DiagnosticCode],
        handler: CodeFixHandler,
    ) {
        let fix = RegisteredFix {
            name: name.to_string(),
            error_codes: error_codes.to_vec(),
            handler,
        };

        for code in error_codes {
            self.handlers
                .entry(*code)
                .or_default()
                .push(fix.clone());
        }

        self.by_name.insert(name.to_string(), fix);
    }

    /// Get all fixes for an error code.
    pub fn get_fixes_for_code(&self, code: DiagnosticCode) -> Option<&Vec<RegisteredFix>> {
        self.handlers.get(&code)
    }

    /// Get a specific fix by name.
    pub fn get_fix_by_name(&self, name: &str) -> Option<&RegisteredFix> {
        self.by_name.get(name)
    }

    /// Get all applicable fixes for a context.
    pub fn get_code_fixes(&self, ctx: &CodeFixContext) -> Vec<CodeFixAction> {
        let mut fixes = Vec::new();

        if let Some(handlers) = self.handlers.get(&ctx.error_code) {
            for registered in handlers {
                let handler_fixes = (registered.handler)(ctx);
                fixes.extend(handler_fixes);
            }
        }

        fixes
    }
}

// =============================================================================
// Combined Code Fixes
// =============================================================================

/// Context for combined/fix-all operations.
pub struct CombinedCodeFixContext<'a> {
    pub arena: &'a NodeArena,
    pub checker: &'a CheckerState,
    pub source_text: &'a str,
    pub file_name: &'a str,
    /// The fix ID to apply.
    pub fix_id: &'a str,
    /// All diagnostics to fix.
    pub diagnostics: &'a [Diagnostic],
    pub preferences: CodeFixPreferences,
}

/// Get combined code fixes for all diagnostics with the same fix ID.
pub fn get_combined_code_fix(
    registry: &CodeFixRegistry,
    ctx: &CombinedCodeFixContext,
) -> Option<CodeFixAction> {
    // Find all applicable fixes
    let mut all_changes: Vec<FileTextChanges> = Vec::new();

    for diagnostic in ctx.diagnostics {
        let fix_ctx = CodeFixContext {
            arena: ctx.arena,
            checker: ctx.checker,
            source_text: ctx.source_text,
            file_name: ctx.file_name,
            error_code: diagnostic.code,
            span: diagnostic.span.clone(),
            preferences: ctx.preferences.clone(),
        };

        let fixes = registry.get_code_fixes(&fix_ctx);

        for fix in fixes {
            if fix.fix_id.as_deref() == Some(ctx.fix_id) {
                all_changes.extend(fix.changes);
            }
        }
    }

    if all_changes.is_empty() {
        return None;
    }

    // Merge changes for the same file
    let merged = merge_file_changes(all_changes);

    Some(CodeFixAction::new(
        ctx.fix_id.to_string(),
        format!("Fix all: {}", ctx.fix_id),
        merged,
    ))
}

fn merge_file_changes(changes: Vec<FileTextChanges>) -> Vec<FileTextChanges> {
    let mut by_file: HashMap<String, Vec<TextChange>> = HashMap::new();

    for file_change in changes {
        by_file
            .entry(file_change.file_name)
            .or_default()
            .extend(file_change.text_changes);
    }

    by_file
        .into_iter()
        .map(|(file_name, mut changes)| {
            // Sort changes by position (descending to apply from end)
            changes.sort_by(|a, b| b.span.start.cmp(&a.span.start));
            FileTextChanges::new(file_name, changes)
        })
        .collect()
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Create a text change to insert text at a position.
pub fn create_insertion(position: u32, text: String) -> TextChange {
    TextChange::new(TextSpan::new(position, 0), text)
}

/// Create a text change to delete a span.
pub fn create_deletion(span: TextSpan) -> TextChange {
    TextChange::new(span, String::new())
}

/// Create a text change to replace a span.
pub fn create_replacement(span: TextSpan, text: String) -> TextChange {
    TextChange::new(span, text)
}

/// Get the text at a span.
pub fn get_text_at_span(source: &str, span: &TextSpan) -> &str {
    let start = span.start as usize;
    let end = start + span.length as usize;

    if end <= source.len() {
        &source[start..end]
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_fix_action() {
        let changes = vec![FileTextChanges::new(
            "test.ts".to_string(),
            vec![TextChange::new(TextSpan::new(0, 0), "import { foo } from 'bar';\n".to_string())],
        )];

        let action = CodeFixAction::new(
            "addImport".to_string(),
            "Add import from 'bar'".to_string(),
            changes,
        );

        assert_eq!(action.fix_name, "addImport");
        assert_eq!(action.changes.len(), 1);
    }

    #[test]
    fn test_code_fix_registry() {
        fn dummy_handler(_ctx: &CodeFixContext) -> Vec<CodeFixAction> {
            Vec::new()
        }

        let mut registry = CodeFixRegistry::new();
        registry.register(
            "testFix",
            &[DiagnosticCode::CANNOT_FIND_NAME],
            dummy_handler,
        );

        assert!(registry.get_fix_by_name("testFix").is_some());
        assert!(registry.get_fixes_for_code(DiagnosticCode::CANNOT_FIND_NAME).is_some());
    }

    #[test]
    fn test_create_changes() {
        let insert = create_insertion(0, "hello".to_string());
        assert_eq!(insert.span.start, 0);
        assert_eq!(insert.span.length, 0);
        assert_eq!(insert.new_text, "hello");

        let delete = create_deletion(TextSpan::new(5, 3));
        assert_eq!(delete.span.start, 5);
        assert_eq!(delete.span.length, 3);
        assert_eq!(delete.new_text, "");
    }

    #[test]
    fn test_diagnostic_codes() {
        assert_eq!(DiagnosticCode::CANNOT_FIND_NAME.0, 2304);
        assert_eq!(DiagnosticCode::PROPERTY_DOES_NOT_EXIST.0, 2339);
    }
}
