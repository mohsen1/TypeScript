//! Module Binding and Symbol Resolution
//!
//! This module handles:
//! - Binding import declarations to the symbol table
//! - Binding export declarations to the symbol table
//! - Module augmentation
//! - Ambient module declarations

use std::collections::HashMap;
use zang_core::{InternedString, Span, StringInterner, Symbol};
use zang_core::symbol::{SymbolFlags, SymbolId};
use zang_parser::{
    ExportAssignment, ExportDeclaration, ExportableDeclaration, ImportDeclaration,
    ModuleDeclaration, NamedExportBindings, NamedImportBindings, Statement,
};

/// A scope in the module
#[derive(Debug, Clone)]
pub struct ModuleScope {
    /// Symbols in this scope
    pub symbols: HashMap<InternedString, SymbolId>,
    /// Parent scope (if any)
    pub parent: Option<Box<ModuleScope>>,
    /// Whether this is a module scope
    pub is_module: bool,
}

impl Default for ModuleScope {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleScope {
    /// Creates a new module scope
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            parent: None,
            is_module: true,
        }
    }

    /// Creates a child scope
    pub fn child(&self) -> Self {
        Self {
            symbols: HashMap::new(),
            parent: Some(Box::new(self.clone())),
            is_module: false,
        }
    }

    /// Looks up a symbol in this scope or parent scopes
    pub fn lookup(&self, name: InternedString) -> Option<SymbolId> {
        self.symbols
            .get(&name)
            .copied()
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }

    /// Declares a symbol in this scope
    pub fn declare(&mut self, name: InternedString, id: SymbolId) {
        self.symbols.insert(name, id);
    }
}

/// Module symbol table entry
#[derive(Debug, Clone)]
pub struct ModuleSymbol {
    /// The symbol
    pub symbol: Symbol,
    /// Import binding (if this symbol was imported)
    pub import_binding: Option<ImportBindingInfo>,
    /// Export binding (if this symbol is exported)
    pub export_binding: Option<ExportBindingInfo>,
}

/// Information about an import binding
#[derive(Debug, Clone)]
pub struct ImportBindingInfo {
    /// The module specifier
    pub module_specifier: String,
    /// The imported name (might differ from local name)
    pub imported_name: InternedString,
    /// Whether this is a type-only import
    pub is_type_only: bool,
    /// Whether this is a namespace import
    pub is_namespace: bool,
    /// Whether this is a default import
    pub is_default: bool,
}

/// Information about an export binding
#[derive(Debug, Clone)]
pub struct ExportBindingInfo {
    /// The exported name (might differ from local name)
    pub exported_name: InternedString,
    /// Whether this is a type-only export
    pub is_type_only: bool,
    /// Whether this is a default export
    pub is_default: bool,
    /// Re-export module (if this is a re-export)
    pub re_export_module: Option<String>,
}

/// Module binder
pub struct ModuleBinder<'a> {
    /// String interner
    interner: &'a StringInterner,
    /// Current scope
    current_scope: ModuleScope,
    /// All symbols
    symbols: Vec<ModuleSymbol>,
    /// Next symbol ID
    next_symbol_id: u32,
    /// Errors
    errors: Vec<ModuleBindError>,
    /// Ambient modules (declare module "name")
    ambient_modules: HashMap<String, ModuleScope>,
    /// Module augmentations
    augmentations: Vec<ModuleAugmentation>,
}

/// A module augmentation
#[derive(Debug, Clone)]
pub struct ModuleAugmentation {
    /// The module being augmented
    pub module_name: String,
    /// Added symbols
    pub added_symbols: Vec<SymbolId>,
    /// The span of the augmentation
    pub span: Span,
}

/// Module bind error
#[derive(Debug, Clone)]
pub struct ModuleBindError {
    /// Error message
    pub message: String,
    /// Error span
    pub span: Span,
    /// Error kind
    pub kind: ModuleBindErrorKind,
}

/// Kind of module bind error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleBindErrorKind {
    DuplicateIdentifier,
    InvalidImport,
    InvalidExport,
    ModuleNotFound,
    AmbientModuleConflict,
}

impl<'a> ModuleBinder<'a> {
    /// Creates a new module binder
    pub fn new(interner: &'a StringInterner) -> Self {
        Self {
            interner,
            current_scope: ModuleScope::new(),
            symbols: Vec::new(),
            next_symbol_id: 0,
            errors: Vec::new(),
            ambient_modules: HashMap::new(),
            augmentations: Vec::new(),
        }
    }

    /// Takes the collected errors
    pub fn take_errors(&mut self) -> Vec<ModuleBindError> {
        std::mem::take(&mut self.errors)
    }

    /// Returns the symbols
    pub fn symbols(&self) -> &[ModuleSymbol] {
        &self.symbols
    }

    /// Returns the current scope
    pub fn current_scope(&self) -> &ModuleScope {
        &self.current_scope
    }

    /// Allocates a new symbol ID
    fn alloc_symbol_id(&mut self) -> SymbolId {
        let id = SymbolId::new(self.next_symbol_id);
        self.next_symbol_id += 1;
        id
    }

    /// Creates a new symbol
    fn create_symbol(
        &mut self,
        name: InternedString,
        flags: SymbolFlags,
        span: Span,
    ) -> SymbolId {
        let id = self.alloc_symbol_id();
        let mut symbol = Symbol::new(id, name, flags);
        symbol.add_declaration(span);

        self.symbols.push(ModuleSymbol {
            symbol,
            import_binding: None,
            export_binding: None,
        });

        id
    }

    /// Binds an import declaration
    pub fn bind_import(&mut self, import: &ImportDeclaration) {
        let module_specifier = self
            .interner
            .lookup(import.module_specifier.value)
            .unwrap_or_default();

        // If no import clause, this is a side-effect import
        let Some(ref clause) = import.import_clause else {
            return;
        };

        let is_type_only = import.is_type_only || clause.is_type_only;

        // Handle default import
        if let Some(ref name) = clause.name {
            self.bind_default_import(name, &module_specifier, is_type_only);
        }

        // Handle named bindings
        if let Some(ref bindings) = clause.named_bindings {
            self.bind_named_import_bindings(bindings, &module_specifier, is_type_only);
        }
    }

    /// Binds a default import
    fn bind_default_import(
        &mut self,
        name: &zang_parser::Identifier,
        module_specifier: &str,
        is_type_only: bool,
    ) {
        // Check for duplicate
        if self.current_scope.lookup(name.name).is_some() {
            let name_str = self.interner.lookup(name.name).unwrap_or_else(|| "<unknown>".to_string());
            self.errors.push(ModuleBindError {
                message: format!("Duplicate identifier '{}'", name_str),
                span: name.span,
                kind: ModuleBindErrorKind::DuplicateIdentifier,
            });
            return;
        }

        let default_name = self.interner.intern("default");
        let flags = if is_type_only {
            SymbolFlags::ALIAS
        } else {
            SymbolFlags::ALIAS | SymbolFlags::BLOCK_SCOPED_VARIABLE
        };

        let id = self.create_symbol(name.name, flags, name.span);
        self.current_scope.declare(name.name, id);

        // Set import binding info
        if let Some(sym) = self.symbols.iter_mut().find(|s| s.symbol.id == id) {
            sym.import_binding = Some(ImportBindingInfo {
                module_specifier: module_specifier.to_string(),
                imported_name: default_name,
                is_type_only,
                is_namespace: false,
                is_default: true,
            });
        }
    }

    /// Binds named import bindings
    fn bind_named_import_bindings(
        &mut self,
        bindings: &NamedImportBindings,
        module_specifier: &str,
        is_type_only: bool,
    ) {
        match bindings {
            NamedImportBindings::Namespace(ns) => {
                self.bind_namespace_import(ns, module_specifier, is_type_only);
            }
            NamedImportBindings::Named(named) => {
                self.bind_named_imports(named, module_specifier, is_type_only);
            }
        }
    }

    /// Binds a namespace import
    fn bind_namespace_import(
        &mut self,
        ns: &zang_parser::NamespaceImport,
        module_specifier: &str,
        is_type_only: bool,
    ) {
        // Check for duplicate
        if self.current_scope.lookup(ns.name.name).is_some() {
            let name_str = self.interner.lookup(ns.name.name).unwrap_or_else(|| "<unknown>".to_string());
            self.errors.push(ModuleBindError {
                message: format!("Duplicate identifier '{}'", name_str),
                span: ns.span,
                kind: ModuleBindErrorKind::DuplicateIdentifier,
            });
            return;
        }

        let star_name = self.interner.intern("*");
        let flags = SymbolFlags::ALIAS | SymbolFlags::NAMESPACE_MODULE;

        let id = self.create_symbol(ns.name.name, flags, ns.span);
        self.current_scope.declare(ns.name.name, id);

        if let Some(sym) = self.symbols.iter_mut().find(|s| s.symbol.id == id) {
            sym.import_binding = Some(ImportBindingInfo {
                module_specifier: module_specifier.to_string(),
                imported_name: star_name,
                is_type_only,
                is_namespace: true,
                is_default: false,
            });
        }
    }

    /// Binds named imports
    fn bind_named_imports(
        &mut self,
        named: &zang_parser::NamedImports,
        module_specifier: &str,
        parent_is_type_only: bool,
    ) {
        for specifier in &named.elements {
            self.bind_import_specifier(specifier, module_specifier, parent_is_type_only);
        }
    }

    /// Binds an import specifier
    fn bind_import_specifier(
        &mut self,
        specifier: &zang_parser::ImportSpecifier,
        module_specifier: &str,
        parent_is_type_only: bool,
    ) {
        let local_name = specifier.name.name;
        let imported_name = specifier
            .property_name
            .as_ref()
            .map(|p| p.name)
            .unwrap_or(local_name);

        // Check for duplicate
        if self.current_scope.lookup(local_name).is_some() {
            let name_str = self.interner.lookup(local_name).unwrap_or_else(|| "<unknown>".to_string());
            self.errors.push(ModuleBindError {
                message: format!("Duplicate identifier '{}'", name_str),
                span: specifier.span,
                kind: ModuleBindErrorKind::DuplicateIdentifier,
            });
            return;
        }

        let is_type_only = parent_is_type_only || specifier.is_type_only;
        let flags = if is_type_only {
            SymbolFlags::ALIAS
        } else {
            SymbolFlags::ALIAS | SymbolFlags::BLOCK_SCOPED_VARIABLE
        };

        let id = self.create_symbol(local_name, flags, specifier.span);
        self.current_scope.declare(local_name, id);

        if let Some(sym) = self.symbols.iter_mut().find(|s| s.symbol.id == id) {
            sym.import_binding = Some(ImportBindingInfo {
                module_specifier: module_specifier.to_string(),
                imported_name,
                is_type_only,
                is_namespace: false,
                is_default: false,
            });
        }
    }

    /// Binds an export declaration
    pub fn bind_export(&mut self, export: &ExportDeclaration) {
        // Handle declaration exports
        if let Some(ref decl) = export.declaration {
            self.bind_declaration_export(decl, export.is_type_only);
            return;
        }

        // Handle named exports and re-exports
        if let Some(ref clause) = export.export_clause {
            let module_specifier = export
                .module_specifier
                .as_ref()
                .map(|s| self.interner.lookup(s.value).unwrap_or_else(|| "".to_string()).to_string());

            self.bind_named_exports(clause, module_specifier.as_deref(), export.is_type_only);
        }
    }

    /// Binds a declaration export
    fn bind_declaration_export(&mut self, decl: &ExportableDeclaration, is_type_only: bool) {
        match decl {
            ExportableDeclaration::Variable(var) => {
                for var_decl in &var.declaration_list.declarations {
                    if let zang_parser::BindingName::Identifier(ident) = &var_decl.name {
                        self.mark_as_exported(ident.name, is_type_only, false);
                    }
                }
            }
            ExportableDeclaration::Function(func) => {
                if let Some(ref name) = func.name {
                    self.mark_as_exported(name.name, is_type_only, false);
                }
            }
            ExportableDeclaration::Class(class) => {
                if let Some(ref name) = class.name {
                    self.mark_as_exported(name.name, is_type_only, false);
                }
            }
            ExportableDeclaration::Interface(iface) => {
                self.mark_as_exported(iface.name.name, true, false);
            }
            ExportableDeclaration::TypeAlias(alias) => {
                self.mark_as_exported(alias.name.name, true, false);
            }
            ExportableDeclaration::Enum(enum_decl) => {
                self.mark_as_exported(enum_decl.name.name, is_type_only, false);
            }
        }
    }

    /// Marks a symbol as exported
    fn mark_as_exported(&mut self, name: InternedString, is_type_only: bool, is_default: bool) {
        if let Some(id) = self.current_scope.lookup(name) {
            if let Some(sym) = self.symbols.iter_mut().find(|s| s.symbol.id == id) {
                sym.symbol.flags = sym.symbol.flags.union(SymbolFlags::EXPORT_VALUE);
                sym.export_binding = Some(ExportBindingInfo {
                    exported_name: name,
                    is_type_only,
                    is_default,
                    re_export_module: None,
                });
            }
        }
    }

    /// Binds named exports
    fn bind_named_exports(
        &mut self,
        clause: &NamedExportBindings,
        module_specifier: Option<&str>,
        is_type_only: bool,
    ) {
        match clause {
            NamedExportBindings::Namespace(ns) => {
                if let Some(ref name) = ns.name {
                    // export * as ns from "mod"
                    let flags = SymbolFlags::ALIAS | SymbolFlags::EXPORT_VALUE;
                    let id = self.create_symbol(name.name, flags, ns.span);

                    if let Some(sym) = self.symbols.iter_mut().find(|s| s.symbol.id == id) {
                        sym.export_binding = Some(ExportBindingInfo {
                            exported_name: name.name,
                            is_type_only,
                            is_default: false,
                            re_export_module: module_specifier.map(String::from),
                        });
                    }
                }
            }
            NamedExportBindings::Named(named) => {
                for specifier in &named.elements {
                    self.bind_export_specifier(specifier, module_specifier, is_type_only);
                }
            }
        }
    }

    /// Binds an export specifier
    fn bind_export_specifier(
        &mut self,
        specifier: &zang_parser::ExportSpecifier,
        module_specifier: Option<&str>,
        parent_is_type_only: bool,
    ) {
        let local_name = specifier
            .property_name
            .as_ref()
            .map(|p| p.name)
            .unwrap_or(specifier.name.name);
        let exported_name = specifier.name.name;
        let is_type_only = parent_is_type_only || specifier.is_type_only;

        if module_specifier.is_some() {
            // Re-export: export { x } from "mod"
            let flags = SymbolFlags::ALIAS | SymbolFlags::EXPORT_VALUE;
            let id = self.create_symbol(exported_name, flags, specifier.span);

            if let Some(sym) = self.symbols.iter_mut().find(|s| s.symbol.id == id) {
                sym.export_binding = Some(ExportBindingInfo {
                    exported_name,
                    is_type_only,
                    is_default: false,
                    re_export_module: module_specifier.map(String::from),
                });
                sym.import_binding = Some(ImportBindingInfo {
                    module_specifier: module_specifier.unwrap_or("").to_string(),
                    imported_name: local_name,
                    is_type_only,
                    is_namespace: false,
                    is_default: false,
                });
            }
        } else {
            // Named export: export { x }
            if let Some(id) = self.current_scope.lookup(local_name) {
                if let Some(sym) = self.symbols.iter_mut().find(|s| s.symbol.id == id) {
                    sym.symbol.flags = sym.symbol.flags.union(SymbolFlags::EXPORT_VALUE);
                    sym.export_binding = Some(ExportBindingInfo {
                        exported_name,
                        is_type_only,
                        is_default: false,
                        re_export_module: None,
                    });
                }
            } else {
                let name_str = self.interner.lookup(local_name).unwrap_or_else(|| "<unknown>".to_string());
                self.errors.push(ModuleBindError {
                    message: format!("Cannot find name '{}'", name_str),
                    span: specifier.span,
                    kind: ModuleBindErrorKind::InvalidExport,
                });
            }
        }
    }

    /// Binds an export assignment
    pub fn bind_export_assignment(&mut self, assignment: &ExportAssignment) {
        let default_name = self.interner.intern("default");
        let flags = SymbolFlags::EXPORT_VALUE | SymbolFlags::ALIAS;

        let id = self.create_symbol(default_name, flags, assignment.span);

        if let Some(sym) = self.symbols.iter_mut().find(|s| s.symbol.id == id) {
            sym.export_binding = Some(ExportBindingInfo {
                exported_name: default_name,
                is_type_only: false,
                is_default: !assignment.is_export_equals,
                re_export_module: None,
            });
        }
    }

    /// Binds a module declaration (ambient or augmentation)
    pub fn bind_module_declaration(&mut self, decl: &ModuleDeclaration) {
        let module_name = match &decl.name {
            zang_parser::ModuleName::Identifier(ident) => {
                self.interner.lookup(ident.name).unwrap_or_else(|| "".to_string()).to_string()
            }
            zang_parser::ModuleName::StringLiteral(lit) => {
                self.interner.lookup(lit.value).unwrap_or_else(|| "".to_string()).to_string()
            }
        };

        let is_augmentation = !decl.modifiers.contains(zang_parser::Modifiers::AMBIENT);

        if is_augmentation {
            // Module augmentation
            self.bind_module_augmentation(&module_name, decl);
        } else {
            // Ambient module declaration
            self.bind_ambient_module(&module_name, decl);
        }
    }

    /// Binds a module augmentation
    fn bind_module_augmentation(&mut self, module_name: &str, decl: &ModuleDeclaration) {
        let mut added_symbols = Vec::new();

        if let Some(ref body) = decl.body {
            if let zang_parser::ModuleBody::Block(block) = body {
                for statement in &block.statements {
                    if let Some(id) = self.bind_module_statement(statement) {
                        added_symbols.push(id);
                    }
                }
            }
        }

        self.augmentations.push(ModuleAugmentation {
            module_name: module_name.to_string(),
            added_symbols,
            span: decl.span,
        });
    }

    /// Binds an ambient module declaration
    fn bind_ambient_module(&mut self, module_name: &str, decl: &ModuleDeclaration) {
        let mut scope = ModuleScope::new();

        if let Some(ref body) = decl.body {
            if let zang_parser::ModuleBody::Block(block) = body {
                let old_scope = std::mem::replace(&mut self.current_scope, scope);
                for statement in &block.statements {
                    self.bind_module_statement(statement);
                }
                scope = std::mem::replace(&mut self.current_scope, old_scope);
            }
        }

        if self.ambient_modules.contains_key(module_name) {
            self.errors.push(ModuleBindError {
                message: format!("Duplicate ambient module declaration '{}'", module_name),
                span: decl.span,
                kind: ModuleBindErrorKind::AmbientModuleConflict,
            });
        } else {
            self.ambient_modules.insert(module_name.to_string(), scope);
        }
    }

    /// Binds a statement within a module body
    fn bind_module_statement(&mut self, statement: &Statement) -> Option<SymbolId> {
        match statement {
            Statement::Import(import) => {
                self.bind_import(import);
                None
            }
            Statement::Export(export) => {
                self.bind_export(export);
                None
            }
            Statement::ExportAssignment(assignment) => {
                self.bind_export_assignment(assignment);
                None
            }
            Statement::Function(func) => {
                if let Some(ref name) = func.name {
                    let id =
                        self.create_symbol(name.name, SymbolFlags::FUNCTION, func.span);
                    self.current_scope.declare(name.name, id);
                    Some(id)
                } else {
                    None
                }
            }
            Statement::Class(class) => {
                if let Some(ref name) = class.name {
                    let id = self.create_symbol(name.name, SymbolFlags::CLASS, class.span);
                    self.current_scope.declare(name.name, id);
                    Some(id)
                } else {
                    None
                }
            }
            Statement::Variable(var) => {
                let mut last_id = None;
                for decl in &var.declaration_list.declarations {
                    if let zang_parser::BindingName::Identifier(ident) = &decl.name {
                        let flags = match var.declaration_list.flags {
                            zang_parser::VariableDeclarationKind::Var => {
                                SymbolFlags::FUNCTION_SCOPED_VARIABLE
                            }
                            _ => SymbolFlags::BLOCK_SCOPED_VARIABLE,
                        };
                        let id = self.create_symbol(ident.name, flags, decl.span);
                        self.current_scope.declare(ident.name, id);
                        last_id = Some(id);
                    }
                }
                last_id
            }
            _ => None,
        }
    }

    /// Gets the ambient modules
    pub fn ambient_modules(&self) -> &HashMap<String, ModuleScope> {
        &self.ambient_modules
    }

    /// Gets the module augmentations
    pub fn augmentations(&self) -> &[ModuleAugmentation] {
        &self.augmentations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_scope_creation() {
        let scope = ModuleScope::new();
        assert!(scope.is_module);
        assert!(scope.parent.is_none());
        assert!(scope.symbols.is_empty());
    }

    #[test]
    fn test_module_scope_child() {
        let parent = ModuleScope::new();
        let child = parent.child();
        assert!(!child.is_module);
        assert!(child.parent.is_some());
    }

    #[test]
    fn test_module_scope_lookup() {
        let interner = StringInterner::new();
        let foo = interner.intern("foo");
        let id = SymbolId::new(1);

        let mut scope = ModuleScope::new();
        scope.declare(foo, id);

        assert_eq!(scope.lookup(foo), Some(id));
    }

    #[test]
    fn test_module_scope_parent_lookup() {
        let interner = StringInterner::new();
        let foo = interner.intern("foo");
        let bar = interner.intern("bar");
        let id1 = SymbolId::new(1);
        let id2 = SymbolId::new(2);

        let mut parent = ModuleScope::new();
        parent.declare(foo, id1);

        let mut child = parent.child();
        child.declare(bar, id2);

        assert_eq!(child.lookup(foo), Some(id1));
        assert_eq!(child.lookup(bar), Some(id2));
    }

    #[test]
    fn test_module_binder_creation() {
        let interner = StringInterner::new();
        let binder = ModuleBinder::new(&interner);
        assert!(binder.symbols().is_empty());
    }

    #[test]
    fn test_mark_as_exported() {
        let interner = StringInterner::new();
        let foo = interner.intern("foo");

        let mut binder = ModuleBinder::new(&interner);

        // First create a symbol
        let id = binder.create_symbol(foo, SymbolFlags::BLOCK_SCOPED_VARIABLE, Span::new(0, 3));
        binder.current_scope.declare(foo, id);

        // Then mark it as exported
        binder.mark_as_exported(foo, false, false);

        let sym = binder.symbols().iter().find(|s| s.symbol.id == id).unwrap();
        assert!(sym.symbol.flags.contains(SymbolFlags::EXPORT_VALUE));
        assert!(sym.export_binding.is_some());
    }
}
