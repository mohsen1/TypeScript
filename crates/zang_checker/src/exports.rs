//! Export Declaration Checking
//!
//! This module handles type checking of export declarations, including:
//! - Named exports (export { x, y })
//! - Re-exports (export { x } from "mod", export * from "mod")
//! - Default exports (export default expr)
//! - Export assignments (export = expr)
//! - Type-only exports (export type { x })
//! - Declaration exports (export const/function/class)

use std::collections::HashMap;
use zang_core::{InternedString, Span, StringInterner};
use zang_parser::{
    ExportAssignment, ExportDeclaration, ExportSpecifier, ExportableDeclaration,
    NamedExportBindings, NamedExports, NamespaceExport,
};

use crate::diagnostics::{Diagnostic, DiagnosticKind, DiagnosticSeverity};
use crate::imports::{ExportedSymbol, ImportResolution};
use crate::types::ResolvedType;

/// Export information for a module
#[derive(Debug, Clone)]
pub struct ModuleExports {
    /// Named exports
    pub exports: HashMap<InternedString, ExportedSymbol>,
    /// Default export (if any)
    pub default_export: Option<ExportedSymbol>,
    /// Whether the module uses CommonJS-style export =
    pub has_export_equals: bool,
    /// Re-exported modules (export * from "mod")
    pub re_exports: Vec<ReExport>,
}

impl Default for ModuleExports {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleExports {
    /// Creates a new empty module exports
    pub fn new() -> Self {
        Self {
            exports: HashMap::new(),
            default_export: None,
            has_export_equals: false,
            re_exports: Vec::new(),
        }
    }

    /// Adds a named export
    pub fn add_export(&mut self, name: InternedString, symbol: ExportedSymbol) {
        self.exports.insert(name, symbol);
    }

    /// Sets the default export
    pub fn set_default_export(&mut self, symbol: ExportedSymbol) {
        self.default_export = Some(symbol);
    }

    /// Adds a re-export
    pub fn add_re_export(&mut self, re_export: ReExport) {
        self.re_exports.push(re_export);
    }

    /// Gets an export by name
    pub fn get_export(&self, name: InternedString) -> Option<&ExportedSymbol> {
        self.exports.get(&name)
    }

    /// Returns true if the module has a named export with the given name
    pub fn has_export(&self, name: InternedString) -> bool {
        self.exports.contains_key(&name)
    }
}

/// Re-export information
#[derive(Debug, Clone)]
pub struct ReExport {
    /// The module being re-exported from
    pub module_specifier: String,
    /// The export name (for `export * as ns from`)
    pub namespace_name: Option<InternedString>,
    /// Whether this is a full re-export (export * from)
    pub is_star_export: bool,
    /// The span of the re-export
    pub span: Span,
}

/// Export checker
pub struct ExportChecker<'a> {
    /// String interner
    interner: &'a StringInterner,
    /// Current module exports being built
    current_exports: ModuleExports,
    /// Symbol types from the current module
    symbol_types: &'a HashMap<InternedString, ResolvedType>,
    /// Diagnostics
    diagnostics: Vec<Diagnostic>,
    /// Module resolver callback
    module_resolver: Option<Box<dyn Fn(&str) -> Option<ImportResolution> + 'a>>,
}

impl<'a> ExportChecker<'a> {
    /// Creates a new export checker
    pub fn new(
        interner: &'a StringInterner,
        symbol_types: &'a HashMap<InternedString, ResolvedType>,
    ) -> Self {
        Self {
            interner,
            current_exports: ModuleExports::new(),
            symbol_types,
            diagnostics: Vec::new(),
            module_resolver: None,
        }
    }

    /// Sets a module resolver callback
    pub fn with_module_resolver<F>(mut self, resolver: F) -> Self
    where
        F: Fn(&str) -> Option<ImportResolution> + 'a,
    {
        self.module_resolver = Some(Box::new(resolver));
        self
    }

    /// Takes the collected diagnostics
    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    /// Returns the collected module exports
    pub fn get_exports(&self) -> &ModuleExports {
        &self.current_exports
    }

    /// Consumes the checker and returns the module exports
    pub fn into_exports(self) -> ModuleExports {
        self.current_exports
    }

    /// Checks an export declaration
    pub fn check_export(&mut self, export: &ExportDeclaration) {
        // Handle declaration exports (export const x = 1)
        if let Some(ref decl) = export.declaration {
            self.check_declaration_export(decl, export.is_type_only);
            return;
        }

        // Handle re-exports (export { x } from "mod" or export * from "mod")
        if let Some(ref module_specifier) = export.module_specifier {
            let specifier = self
                .interner
                .lookup(module_specifier.value)
                .unwrap_or_default();

            if let Some(ref clause) = export.export_clause {
                self.check_re_export(clause, &specifier, export.is_type_only, export.span);
            } else {
                // export * from "mod"
                self.current_exports.add_re_export(ReExport {
                    module_specifier: specifier,
                    namespace_name: None,
                    is_star_export: true,
                    span: export.span,
                });
            }
            return;
        }

        // Handle named exports (export { x, y })
        if let Some(ref clause) = export.export_clause {
            self.check_named_exports(clause, export.is_type_only);
        }
    }

    /// Checks a declaration export
    fn check_declaration_export(&mut self, decl: &ExportableDeclaration, is_type_only: bool) {
        match decl {
            ExportableDeclaration::Variable(var) => {
                self.export_variable_declaration(var, is_type_only);
            }
            ExportableDeclaration::Function(func) => {
                self.export_function_declaration(func, is_type_only);
            }
            ExportableDeclaration::Class(class) => {
                self.export_class_declaration(class, is_type_only);
            }
            ExportableDeclaration::Interface(iface) => {
                self.export_interface_declaration(iface);
            }
            ExportableDeclaration::TypeAlias(alias) => {
                self.export_type_alias_declaration(alias);
            }
            ExportableDeclaration::Enum(enum_decl) => {
                self.export_enum_declaration(enum_decl, is_type_only);
            }
        }
    }

    /// Exports a variable declaration
    fn export_variable_declaration(
        &mut self,
        var: &zang_parser::VariableStatement,
        is_type_only: bool,
    ) {
        for decl in &var.declaration_list.declarations {
            if let zang_parser::BindingName::Identifier(ident) = &decl.name {
                let ty = self
                    .symbol_types
                    .get(&ident.name)
                    .cloned()
                    .unwrap_or(ResolvedType::Any);

                self.current_exports.add_export(
                    ident.name,
                    ExportedSymbol {
                        name: ident.name,
                        ty,
                        is_type_only,
                        declaration_span: Some(decl.span),
                    },
                );
            }
            // TODO: Handle destructuring patterns in exports
        }
    }

    /// Exports a function declaration
    fn export_function_declaration(
        &mut self,
        func: &zang_parser::FunctionDeclaration,
        is_type_only: bool,
    ) {
        if let Some(ref name) = func.name {
            let ty = self
                .symbol_types
                .get(&name.name)
                .cloned()
                .unwrap_or(ResolvedType::Any);

            self.current_exports.add_export(
                name.name,
                ExportedSymbol {
                    name: name.name,
                    ty,
                    is_type_only,
                    declaration_span: Some(func.span),
                },
            );
        }
    }

    /// Exports a class declaration
    fn export_class_declaration(
        &mut self,
        class: &zang_parser::ClassDeclaration,
        is_type_only: bool,
    ) {
        if let Some(ref name) = class.name {
            let ty = self
                .symbol_types
                .get(&name.name)
                .cloned()
                .unwrap_or(ResolvedType::Any);

            self.current_exports.add_export(
                name.name,
                ExportedSymbol {
                    name: name.name,
                    ty,
                    is_type_only,
                    declaration_span: Some(class.span),
                },
            );
        }
    }

    /// Exports an interface declaration
    fn export_interface_declaration(&mut self, iface: &zang_parser::InterfaceDeclaration) {
        let ty = self
            .symbol_types
            .get(&iface.name.name)
            .cloned()
            .unwrap_or(ResolvedType::Any);

        self.current_exports.add_export(
            iface.name.name,
            ExportedSymbol {
                name: iface.name.name,
                ty,
                is_type_only: true, // Interfaces are always type-only
                declaration_span: Some(iface.span),
            },
        );
    }

    /// Exports a type alias declaration
    fn export_type_alias_declaration(&mut self, alias: &zang_parser::TypeAliasDeclaration) {
        let ty = self
            .symbol_types
            .get(&alias.name.name)
            .cloned()
            .unwrap_or(ResolvedType::Any);

        self.current_exports.add_export(
            alias.name.name,
            ExportedSymbol {
                name: alias.name.name,
                ty,
                is_type_only: true, // Type aliases are always type-only
                declaration_span: Some(alias.span),
            },
        );
    }

    /// Exports an enum declaration
    fn export_enum_declaration(
        &mut self,
        enum_decl: &zang_parser::EnumDeclaration,
        is_type_only: bool,
    ) {
        let ty = self
            .symbol_types
            .get(&enum_decl.name.name)
            .cloned()
            .unwrap_or(ResolvedType::Any);

        self.current_exports.add_export(
            enum_decl.name.name,
            ExportedSymbol {
                name: enum_decl.name.name,
                ty,
                is_type_only,
                declaration_span: Some(enum_decl.span),
            },
        );
    }

    /// Checks named exports
    fn check_named_exports(&mut self, clause: &NamedExportBindings, is_type_only: bool) {
        match clause {
            NamedExportBindings::Namespace(ns) => {
                self.check_namespace_export(ns);
            }
            NamedExportBindings::Named(named) => {
                self.check_named_export_specifiers(named, is_type_only);
            }
        }
    }

    /// Checks a namespace export (export * as ns from "mod")
    fn check_namespace_export(&mut self, ns: &NamespaceExport) {
        // This form requires a module specifier, which is handled in check_re_export
        if ns.name.is_some() {
            self.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::InvalidExport,
                message: "Namespace export requires a module specifier".to_string(),
                span: ns.span,
                severity: DiagnosticSeverity::Error,
                related: Vec::new(),
                code: 1236,
            });
        }
    }

    /// Checks named export specifiers
    fn check_named_export_specifiers(&mut self, named: &NamedExports, parent_is_type_only: bool) {
        for specifier in &named.elements {
            self.check_export_specifier(specifier, parent_is_type_only);
        }
    }

    /// Checks an individual export specifier
    fn check_export_specifier(&mut self, specifier: &ExportSpecifier, parent_is_type_only: bool) {
        let local_name = specifier
            .property_name
            .as_ref()
            .map(|p| p.name)
            .unwrap_or(specifier.name.name);

        let export_name = specifier.name.name;
        let is_type_only = parent_is_type_only || specifier.is_type_only;

        // Look up the local symbol
        let ty = if let Some(ty) = self.symbol_types.get(&local_name) {
            ty.clone()
        } else {
            let local_str = self.interner.lookup(local_name).unwrap_or_else(|| "<unknown>".to_string());
            self.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::UndeclaredIdentifier,
                message: format!("Cannot find name '{}'", local_str),
                span: specifier.span,
                severity: DiagnosticSeverity::Error,
                related: Vec::new(),
                code: 2304,
            });
            ResolvedType::Any
        };

        // Check for duplicate exports
        if self.current_exports.has_export(export_name) {
            let export_str = self.interner.lookup(export_name).unwrap_or_else(|| "<unknown>".to_string());
            self.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::DuplicateExport,
                message: format!("Module already has an export named '{}'", export_str),
                span: specifier.span,
                severity: DiagnosticSeverity::Error,
                related: Vec::new(),
                code: 2484,
            });
        }

        self.current_exports.add_export(
            export_name,
            ExportedSymbol {
                name: export_name,
                ty,
                is_type_only,
                declaration_span: Some(specifier.span),
            },
        );
    }

    /// Checks a re-export
    fn check_re_export(
        &mut self,
        clause: &NamedExportBindings,
        module_specifier: &str,
        is_type_only: bool,
        span: Span,
    ) {
        match clause {
            NamedExportBindings::Namespace(ns) => {
                // export * as ns from "mod"
                self.current_exports.add_re_export(ReExport {
                    module_specifier: module_specifier.to_string(),
                    namespace_name: ns.name.as_ref().map(|n| n.name),
                    is_star_export: true,
                    span,
                });

                // If there's a name, also add it as a named export
                if let Some(ref name) = ns.name {
                    // Resolve the module to get its type
                    let namespace_type = self
                        .resolve_namespace_type(module_specifier)
                        .unwrap_or(ResolvedType::Any);

                    self.current_exports.add_export(
                        name.name,
                        ExportedSymbol {
                            name: name.name,
                            ty: namespace_type,
                            is_type_only,
                            declaration_span: Some(span),
                        },
                    );
                }
            }
            NamedExportBindings::Named(named) => {
                // export { a, b } from "mod"
                self.check_re_export_specifiers(named, module_specifier, is_type_only);
            }
        }
    }

    /// Checks re-export specifiers
    fn check_re_export_specifiers(
        &mut self,
        named: &NamedExports,
        module_specifier: &str,
        parent_is_type_only: bool,
    ) {
        let resolution = self.resolve_module(module_specifier);

        for specifier in &named.elements {
            let imported_name = specifier
                .property_name
                .as_ref()
                .map(|p| p.name)
                .unwrap_or(specifier.name.name);

            let export_name = specifier.name.name;
            let is_type_only = parent_is_type_only || specifier.is_type_only;

            // Look up the export from the resolved module
            let ty = if let Some(ref res) = resolution {
                if let Some(export) = res.exports.get(&imported_name) {
                    export.ty.clone()
                } else {
                    let imported_str = self.interner.lookup(imported_name).unwrap_or_else(|| "<unknown>".to_string());
                    self.diagnostics.push(Diagnostic {
                        kind: DiagnosticKind::ExportNotFound,
                        message: format!(
                            "Module '{}' has no exported member '{}'",
                            module_specifier, imported_str
                        ),
                        span: specifier.span,
                        severity: DiagnosticSeverity::Error,
                        related: Vec::new(),
                        code: 2305,
                    });
                    ResolvedType::Any
                }
            } else {
                ResolvedType::Any
            };

            self.current_exports.add_export(
                export_name,
                ExportedSymbol {
                    name: export_name,
                    ty,
                    is_type_only,
                    declaration_span: Some(specifier.span),
                },
            );
        }
    }

    /// Resolves a module using the module resolver
    fn resolve_module(&self, specifier: &str) -> Option<ImportResolution> {
        self.module_resolver
            .as_ref()
            .and_then(|resolver| resolver(specifier))
    }

    /// Resolves a namespace type for a module
    fn resolve_namespace_type(&self, specifier: &str) -> Option<ResolvedType> {
        let resolution = self.resolve_module(specifier)?;

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

        Some(ResolvedType::Object(ObjectType {
            properties,
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            index_signatures: Vec::new(),
        }))
    }

    /// Checks an export assignment (export = expr or export default expr)
    pub fn check_export_assignment(&mut self, assignment: &ExportAssignment) {
        // Determine the type of the expression
        // For now, use Any - in production this would infer the type
        let ty = ResolvedType::Any;

        if assignment.is_export_equals {
            // export = expr (CommonJS style)
            self.current_exports.has_export_equals = true;

            // Check for conflicts with ES module exports
            if !self.current_exports.exports.is_empty()
                || self.current_exports.default_export.is_some()
            {
                self.diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::InvalidExport,
                    message:
                        "An export assignment cannot be used with other exported elements".to_string(),
                    span: assignment.span,
                    severity: DiagnosticSeverity::Error,
                    related: Vec::new(),
                    code: 2309,
                });
            }
        } else {
            // export default expr
            if self.current_exports.default_export.is_some() {
                self.diagnostics.push(Diagnostic {
                    kind: DiagnosticKind::DuplicateExport,
                    message: "A module cannot have multiple default exports".to_string(),
                    span: assignment.span,
                    severity: DiagnosticSeverity::Error,
                    related: Vec::new(),
                    code: 2528,
                });
            }

            let default_name = self.interner.intern("default");
            self.current_exports.set_default_export(ExportedSymbol {
                name: default_name,
                ty,
                is_type_only: false,
                declaration_span: Some(assignment.span),
            });
        }
    }
}

/// Validates that exports don't conflict
pub fn validate_export_conflicts(exports: &ModuleExports, diagnostics: &mut Vec<Diagnostic>) {
    // Check if using export = with other exports
    if exports.has_export_equals && (!exports.exports.is_empty() || exports.default_export.is_some())
    {
        diagnostics.push(Diagnostic {
            kind: DiagnosticKind::InvalidExport,
            message: "An export assignment cannot be used with other exported elements".to_string(),
            span: Span::new(0, 0),
            severity: DiagnosticSeverity::Error,
            related: Vec::new(),
            code: 2309,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports_creation() {
        let exports = ModuleExports::new();
        assert!(exports.exports.is_empty());
        assert!(exports.default_export.is_none());
        assert!(!exports.has_export_equals);
    }

    #[test]
    fn test_add_export() {
        let interner = StringInterner::new();
        let foo_name = interner.intern("foo");

        let mut exports = ModuleExports::new();
        exports.add_export(
            foo_name,
            ExportedSymbol {
                name: foo_name,
                ty: ResolvedType::Number,
                is_type_only: false,
                declaration_span: None,
            },
        );

        assert!(exports.has_export(foo_name));
        assert!(exports.get_export(foo_name).is_some());
    }

    #[test]
    fn test_set_default_export() {
        let interner = StringInterner::new();
        let default_name = interner.intern("default");

        let mut exports = ModuleExports::new();
        exports.set_default_export(ExportedSymbol {
            name: default_name,
            ty: ResolvedType::String,
            is_type_only: false,
            declaration_span: None,
        });

        assert!(exports.default_export.is_some());
    }

    #[test]
    fn test_add_re_export() {
        let mut exports = ModuleExports::new();
        exports.add_re_export(ReExport {
            module_specifier: "./utils".to_string(),
            namespace_name: None,
            is_star_export: true,
            span: Span::new(0, 20),
        });

        assert_eq!(exports.re_exports.len(), 1);
        assert!(exports.re_exports[0].is_star_export);
    }

    #[test]
    fn test_export_checker_creation() {
        let interner = StringInterner::new();
        let symbol_types = HashMap::new();
        let checker = ExportChecker::new(&interner, &symbol_types);

        assert!(checker.get_exports().exports.is_empty());
    }

    #[test]
    fn test_validate_export_conflicts() {
        let interner = StringInterner::new();
        let foo_name = interner.intern("foo");

        let mut exports = ModuleExports::new();
        exports.has_export_equals = true;
        exports.add_export(
            foo_name,
            ExportedSymbol {
                name: foo_name,
                ty: ResolvedType::Number,
                is_type_only: false,
                declaration_span: None,
            },
        );

        let mut diagnostics = Vec::new();
        validate_export_conflicts(&exports, &mut diagnostics);

        assert_eq!(diagnostics.len(), 1);
        assert!(matches!(
            diagnostics[0].kind,
            DiagnosticKind::InvalidExport
        ));
    }
}
