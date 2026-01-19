//! Import Declaration Checking
//!
//! This module handles type checking of import declarations, including:
//! - Named imports (import { x, y } from "mod")
//! - Default imports (import x from "mod")
//! - Namespace imports (import * as ns from "mod")
//! - Type-only imports (import type { x } from "mod")
//! - Side-effect imports (import "mod")

use std::collections::HashMap;
use zang_core::{InternedString, Span, StringInterner};
use zang_parser::{
    ImportDeclaration, ImportSpecifier, NamedImportBindings, NamedImports,
    NamespaceImport,
};

use crate::diagnostics::{Diagnostic, DiagnosticKind, DiagnosticSeverity, RelatedInformation};
use crate::types::ResolvedType;

/// Result of resolving an import
#[derive(Debug, Clone)]
pub struct ImportResolution {
    /// The resolved module path
    pub resolved_path: String,
    /// Whether the module was found
    pub is_resolved: bool,
    /// Exported symbols from the module
    pub exports: HashMap<InternedString, ExportedSymbol>,
    /// Whether the module has a default export
    pub has_default_export: bool,
    /// The type of the default export (if any)
    pub default_export_type: Option<ResolvedType>,
}

/// An exported symbol from a module
#[derive(Debug, Clone)]
pub struct ExportedSymbol {
    /// The name of the export
    pub name: InternedString,
    /// The type of the export
    pub ty: ResolvedType,
    /// Whether this is a type-only export
    pub is_type_only: bool,
    /// The span where the export was declared
    pub declaration_span: Option<Span>,
}

/// Import binding information
#[derive(Debug, Clone)]
pub struct ImportBinding {
    /// The local name of the binding
    pub local_name: InternedString,
    /// The imported name from the module
    pub imported_name: InternedString,
    /// The resolved type
    pub ty: ResolvedType,
    /// Whether this is a type-only import
    pub is_type_only: bool,
    /// The span of the import specifier
    pub span: Span,
}

/// Module resolution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleResolutionMode {
    /// Node.js style resolution (node_modules, index.js)
    Node,
    /// Classic TypeScript resolution
    Classic,
    /// Bundler-style resolution
    Bundler,
}

/// Import checker
pub struct ImportChecker<'a> {
    /// String interner
    interner: &'a StringInterner,
    /// Module resolution mode
    resolution_mode: ModuleResolutionMode,
    /// Cache of resolved modules
    module_cache: HashMap<String, ImportResolution>,
    /// Diagnostics
    diagnostics: Vec<Diagnostic>,
    /// Current file path (for relative imports)
    current_file: String,
}

impl<'a> ImportChecker<'a> {
    /// Creates a new import checker
    pub fn new(interner: &'a StringInterner, current_file: String) -> Self {
        Self {
            interner,
            resolution_mode: ModuleResolutionMode::Node,
            module_cache: HashMap::new(),
            diagnostics: Vec::new(),
            current_file,
        }
    }

    /// Sets the module resolution mode
    pub fn with_resolution_mode(mut self, mode: ModuleResolutionMode) -> Self {
        self.resolution_mode = mode;
        self
    }

    /// Takes the collected diagnostics
    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    /// Checks an import declaration and returns the bindings it creates
    pub fn check_import(
        &mut self,
        import: &ImportDeclaration,
    ) -> Result<Vec<ImportBinding>, Diagnostic> {
        let module_specifier = self.interner.lookup(import.module_specifier.value);
        let module_specifier_str = module_specifier.as_deref().unwrap_or("");

        // Resolve the module
        let resolution = self.resolve_module(module_specifier_str, import.module_specifier.span)?;

        // If no import clause, this is a side-effect import (import "mod")
        let Some(ref clause) = import.import_clause else {
            return Ok(vec![]);
        };

        let mut bindings = Vec::new();

        // Check if the entire import is type-only
        let is_type_only = import.is_type_only || clause.is_type_only;

        // Handle default import
        if let Some(ref name) = clause.name {
            let binding = self.check_default_import(name, &resolution, is_type_only)?;
            bindings.push(binding);
        }

        // Handle named bindings
        if let Some(ref named_bindings) = clause.named_bindings {
            let named = self.check_named_bindings(named_bindings, &resolution, is_type_only)?;
            bindings.extend(named);
        }

        Ok(bindings)
    }

    /// Resolves a module specifier to a module resolution
    fn resolve_module(
        &mut self,
        specifier: &str,
        span: Span,
    ) -> Result<ImportResolution, Diagnostic> {
        // Check cache first
        if let Some(resolution) = self.module_cache.get(specifier) {
            return Ok(resolution.clone());
        }

        // Determine if this is a relative or bare specifier
        let is_relative = specifier.starts_with("./") || specifier.starts_with("../");
        let is_absolute = specifier.starts_with('/');

        let resolved_path = if is_relative || is_absolute {
            self.resolve_relative_path(specifier)
        } else {
            self.resolve_bare_specifier(specifier)
        };

        // For now, create a placeholder resolution
        // In a real implementation, this would read and parse the module
        let resolution = ImportResolution {
            resolved_path: resolved_path.clone(),
            is_resolved: true, // Assume resolved for now
            exports: HashMap::new(),
            has_default_export: true, // Assume module has default export
            default_export_type: Some(ResolvedType::Any),
        };

        // Cache the resolution
        self.module_cache
            .insert(specifier.to_string(), resolution.clone());

        // If module not found, emit diagnostic but continue
        if !resolution.is_resolved {
            self.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::ModuleNotFound,
                message: format!("Cannot find module '{}'", specifier),
                span,
                severity: DiagnosticSeverity::Error,
                related: Vec::new(),
                code: 2307,
            });
        }

        Ok(resolution)
    }

    /// Resolves a relative path
    fn resolve_relative_path(&self, specifier: &str) -> String {
        // Basic path resolution - in production this would handle:
        // - Normalizing paths
        // - Checking file extensions (.ts, .tsx, .js, .jsx, .json)
        // - Checking for index files
        // - Handling package.json main/exports
        let base_dir = std::path::Path::new(&self.current_file)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        format!("{}/{}", base_dir, specifier)
    }

    /// Resolves a bare specifier (package name)
    fn resolve_bare_specifier(&self, specifier: &str) -> String {
        // In production this would:
        // - Look up node_modules
        // - Handle scoped packages (@org/pkg)
        // - Handle subpath exports
        // - Check package.json exports field
        format!("node_modules/{}", specifier)
    }

    /// Checks a default import
    fn check_default_import(
        &mut self,
        name: &zang_parser::Identifier,
        resolution: &ImportResolution,
        is_type_only: bool,
    ) -> Result<ImportBinding, Diagnostic> {
        // Check if module has a default export
        if !resolution.has_default_export {
            self.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::NoDefaultExport,
                message: format!(
                    "Module '{}' has no default export",
                    resolution.resolved_path
                ),
                span: name.span,
                severity: DiagnosticSeverity::Error,
                related: Vec::new(),
                code: 2307,
            });
        }

        let default_name = self.interner.intern("default");

        Ok(ImportBinding {
            local_name: name.name,
            imported_name: default_name,
            ty: resolution
                .default_export_type
                .clone()
                .unwrap_or(ResolvedType::Any),
            is_type_only,
            span: name.span,
        })
    }

    /// Checks named bindings (namespace or named imports)
    fn check_named_bindings(
        &mut self,
        bindings: &NamedImportBindings,
        resolution: &ImportResolution,
        is_type_only: bool,
    ) -> Result<Vec<ImportBinding>, Diagnostic> {
        match bindings {
            NamedImportBindings::Namespace(ns) => {
                let binding = self.check_namespace_import(ns, resolution, is_type_only)?;
                Ok(vec![binding])
            }
            NamedImportBindings::Named(named) => {
                self.check_named_imports(named, resolution, is_type_only)
            }
        }
    }

    /// Checks a namespace import (import * as ns from "mod")
    fn check_namespace_import(
        &mut self,
        ns_import: &NamespaceImport,
        resolution: &ImportResolution,
        is_type_only: bool,
    ) -> Result<ImportBinding, Diagnostic> {
        // Create a namespace object type with all exports
        let namespace_type = self.create_namespace_type(resolution);
        let star_name = self.interner.intern("*");

        Ok(ImportBinding {
            local_name: ns_import.name.name,
            imported_name: star_name,
            ty: namespace_type,
            is_type_only,
            span: ns_import.span,
        })
    }

    /// Creates a namespace type from module exports
    fn create_namespace_type(&self, resolution: &ImportResolution) -> ResolvedType {
        use crate::types::{ObjectType, PropertySignature};

        let properties: Vec<PropertySignature> = resolution
            .exports
            .iter()
            .map(|(name, export)| PropertySignature {
                name: *name,
                ty: Box::new(export.ty.clone()),
                optional: false,
                readonly: true,
            })
            .collect();

        ResolvedType::Object(ObjectType {
            properties,
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            index_signatures: Vec::new(),
        })
    }

    /// Checks named imports (import { a, b as c } from "mod")
    fn check_named_imports(
        &mut self,
        named: &NamedImports,
        resolution: &ImportResolution,
        parent_is_type_only: bool,
    ) -> Result<Vec<ImportBinding>, Diagnostic> {
        let mut bindings = Vec::new();

        for specifier in &named.elements {
            let binding =
                self.check_import_specifier(specifier, resolution, parent_is_type_only)?;
            bindings.push(binding);
        }

        Ok(bindings)
    }

    /// Checks an individual import specifier
    fn check_import_specifier(
        &mut self,
        specifier: &ImportSpecifier,
        resolution: &ImportResolution,
        parent_is_type_only: bool,
    ) -> Result<ImportBinding, Diagnostic> {
        // Determine the imported name (property_name if renaming, otherwise name)
        let imported_name = specifier
            .property_name
            .as_ref()
            .map(|p| p.name)
            .unwrap_or(specifier.name.name);

        let is_type_only = parent_is_type_only || specifier.is_type_only;

        // Look up the export in the resolution
        let ty = if let Some(export) = resolution.exports.get(&imported_name) {
            // Check if importing a value as type-only when it's not a type
            if is_type_only && !export.is_type_only {
                // This is allowed - you can import values type-only
            }
            export.ty.clone()
        } else {
            // Export not found - this is an error in strict mode
            let imported_str = self
                .interner
                .lookup(imported_name)
                .unwrap_or_else(|| "<unknown>".to_string());
            self.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::ExportNotFound,
                message: format!(
                    "Module '{}' has no exported member '{}'",
                    resolution.resolved_path, imported_str
                ),
                span: specifier.span,
                severity: DiagnosticSeverity::Error,
                related: Vec::new(),
                code: 2307,
            });
            ResolvedType::Any
        };

        Ok(ImportBinding {
            local_name: specifier.name.name,
            imported_name,
            ty,
            is_type_only,
            span: specifier.span,
        })
    }

    /// Validates that type-only imports are used correctly
    pub fn validate_type_only_usage(&mut self, binding: &ImportBinding, used_as_value: bool) {
        if binding.is_type_only && used_as_value {
            let name = self
                .interner
                .lookup(binding.local_name)
                .unwrap_or_else(|| "<unknown>".to_string());
            self.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::TypeOnlyUsedAsValue,
                message: format!(
                    "'{}' is a type-only import and cannot be used as a value",
                    name
                ),
                span: binding.span,
                severity: DiagnosticSeverity::Error,
                related: Vec::new(),
                code: 2307,
            });
        }
    }

    /// Checks for duplicate import bindings
    pub fn check_duplicate_bindings(&mut self, bindings: &[ImportBinding]) {
        let mut seen: HashMap<InternedString, Span> = HashMap::new();

        for binding in bindings {
            if let Some(&prev_span) = seen.get(&binding.local_name) {
                let name = self
                    .interner
                    .lookup(binding.local_name)
                    .unwrap_or_else(|| "<unknown>".to_string());
                self.diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::DuplicateIdentifier,
                    message: format!("Duplicate identifier '{}'", name),
                    span: binding.span,
                    severity: DiagnosticSeverity::Error,
                    related: vec![RelatedInformation {
                        message: "Previously declared here".to_string(),
                        span: prev_span,
                    }],
                    code: 2300,
                });
            } else {
                seen.insert(binding.local_name, binding.span);
            }
        }
    }
}

/// Helper function to check if an import is for a built-in module
pub fn is_builtin_module(specifier: &str) -> bool {
    matches!(
        specifier,
        "fs"
            | "path"
            | "os"
            | "crypto"
            | "http"
            | "https"
            | "url"
            | "util"
            | "stream"
            | "events"
            | "buffer"
            | "querystring"
            | "assert"
            | "child_process"
            | "cluster"
            | "dgram"
            | "dns"
            | "net"
            | "readline"
            | "repl"
            | "tls"
            | "tty"
            | "v8"
            | "vm"
            | "zlib"
    )
}

/// Helper function to check if a specifier is a relative path
pub fn is_relative_specifier(specifier: &str) -> bool {
    specifier.starts_with("./") || specifier.starts_with("../")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_builtin_module() {
        assert!(is_builtin_module("fs"));
        assert!(is_builtin_module("path"));
        assert!(is_builtin_module("crypto"));
        assert!(!is_builtin_module("lodash"));
        assert!(!is_builtin_module("./local"));
    }

    #[test]
    fn test_is_relative_specifier() {
        assert!(is_relative_specifier("./foo"));
        assert!(is_relative_specifier("../bar"));
        assert!(!is_relative_specifier("lodash"));
        assert!(!is_relative_specifier("/absolute"));
    }

    #[test]
    fn test_import_checker_creation() {
        let interner = StringInterner::new();
        let checker = ImportChecker::new(&interner, "/src/index.ts".to_string());
        assert_eq!(checker.current_file, "/src/index.ts");
    }

    #[test]
    fn test_module_resolution_mode() {
        let interner = StringInterner::new();
        let checker = ImportChecker::new(&interner, "/src/index.ts".to_string())
            .with_resolution_mode(ModuleResolutionMode::Bundler);
        assert_eq!(checker.resolution_mode, ModuleResolutionMode::Bundler);
    }

    #[test]
    fn test_create_namespace_type() {
        let interner = StringInterner::new();
        let checker = ImportChecker::new(&interner, "/src/index.ts".to_string());

        let mut exports = HashMap::new();
        let foo_name = interner.intern("foo");
        exports.insert(
            foo_name,
            ExportedSymbol {
                name: foo_name,
                ty: ResolvedType::Number,
                is_type_only: false,
                declaration_span: None,
            },
        );

        let resolution = ImportResolution {
            resolved_path: "./module".to_string(),
            is_resolved: true,
            exports,
            has_default_export: false,
            default_export_type: None,
        };

        let ns_type = checker.create_namespace_type(&resolution);
        assert!(matches!(ns_type, ResolvedType::Object(_)));
    }
}
