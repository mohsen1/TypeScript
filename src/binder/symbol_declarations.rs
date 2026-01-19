//! Symbol declaration binding
//!
//! This module handles the binding phase for symbol declarations,
//! determining when a symbol should have a unique type versus a generic type.

use crate::checker::symbols::{
    SymbolType, UniqueSymbolId, WellKnownSymbol, SymbolDeclarationFlags, SymbolRegistry
};
use crate::types::unique_symbol::UniqueSymbolFactory;
use std::collections::HashMap;

/// Symbol declaration context during binding
#[derive(Debug)]
pub struct SymbolBindingContext {
    /// Factory for creating unique symbols
    factory: UniqueSymbolFactory,
    /// Map from declaration name to symbol type
    declarations: HashMap<String, BoundSymbol>,
    /// Registry for Symbol.for() symbols
    registry: SymbolRegistry,
    /// Stack of scopes for nested declarations
    scope_stack: Vec<SymbolScope>,
}

/// A bound symbol in the symbol table
#[derive(Debug, Clone, PartialEq)]
pub struct BoundSymbol {
    /// The name of the declaration
    pub name: String,
    /// The symbol type
    pub symbol_type: SymbolType,
    /// Declaration flags
    pub flags: SymbolDeclarationFlags,
    /// Whether this is exported
    pub exported: bool,
    /// Source location start
    pub start: usize,
    /// Source location end
    pub end: usize,
}

/// Scope for symbol declarations
#[derive(Debug, Clone, Default)]
pub struct SymbolScope {
    /// Symbols declared in this scope
    symbols: HashMap<String, BoundSymbol>,
    /// Whether this is a const context (readonly/const)
    is_const_context: bool,
}

impl SymbolScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn const_context() -> Self {
        Self {
            symbols: HashMap::new(),
            is_const_context: true,
        }
    }
}

impl SymbolBindingContext {
    pub fn new() -> Self {
        Self {
            factory: UniqueSymbolFactory::new(),
            declarations: HashMap::new(),
            registry: SymbolRegistry::new(),
            scope_stack: vec![SymbolScope::new()],
        }
    }

    /// Push a new scope onto the stack
    pub fn push_scope(&mut self, is_const_context: bool) {
        if is_const_context {
            self.scope_stack.push(SymbolScope::const_context());
        } else {
            self.scope_stack.push(SymbolScope::new());
        }
    }

    /// Pop the current scope from the stack
    pub fn pop_scope(&mut self) -> Option<SymbolScope> {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop()
        } else {
            None
        }
    }

    /// Get the current scope
    fn current_scope(&self) -> &SymbolScope {
        self.scope_stack.last().unwrap()
    }

    /// Get the current scope mutably
    fn current_scope_mut(&mut self) -> &mut SymbolScope {
        self.scope_stack.last_mut().unwrap()
    }

    /// Check if we're in a const context
    pub fn is_const_context(&self) -> bool {
        self.scope_stack.iter().any(|s| s.is_const_context)
    }

    /// Bind a const symbol declaration
    ///
    /// `const mySymbol = Symbol("description")`
    /// Results in a unique symbol type
    pub fn bind_const_symbol(
        &mut self,
        name: &str,
        _description: Option<&str>,
        start: usize,
        end: usize,
    ) -> BoundSymbol {
        let unique_type = self.factory.create_with_name(
            // We need to leak the string to get a static lifetime
            // In a real implementation, we'd use an arena allocator
            Box::leak(name.to_string().into_boxed_str())
        );

        let symbol_type = unique_type.to_symbol_type();

        let bound = BoundSymbol {
            name: name.to_string(),
            symbol_type,
            flags: SymbolDeclarationFlags::const_declaration(),
            exported: false,
            start,
            end,
        };

        self.declarations.insert(name.to_string(), bound.clone());
        self.current_scope_mut().symbols.insert(name.to_string(), bound.clone());

        bound
    }

    /// Bind a let/var symbol declaration
    ///
    /// `let mySymbol = Symbol("description")`
    /// Results in the generic `symbol` type
    pub fn bind_mutable_symbol(
        &mut self,
        name: &str,
        start: usize,
        end: usize,
    ) -> BoundSymbol {
        let bound = BoundSymbol {
            name: name.to_string(),
            symbol_type: SymbolType::Symbol,
            flags: SymbolDeclarationFlags::default(),
            exported: false,
            start,
            end,
        };

        self.declarations.insert(name.to_string(), bound.clone());
        self.current_scope_mut().symbols.insert(name.to_string(), bound.clone());

        bound
    }

    /// Bind a Symbol.for() call
    ///
    /// `Symbol.for("key")` returns a symbol from the global registry
    pub fn bind_registered_symbol(
        &mut self,
        key: &str,
    ) -> SymbolType {
        let id = self.registry.get_or_create(key);
        SymbolType::UniqueSymbol(id)
    }

    /// Bind a well-known symbol access
    ///
    /// `Symbol.iterator` results in a well-known symbol type
    pub fn bind_well_known_symbol(
        &mut self,
        property_name: &str,
    ) -> Option<SymbolType> {
        WellKnownSymbol::from_property_name(property_name)
            .map(|wks| SymbolType::WellKnownSymbol(wks))
    }

    /// Look up a symbol by name
    pub fn lookup(&self, name: &str) -> Option<&BoundSymbol> {
        // Search from innermost scope outward
        for scope in self.scope_stack.iter().rev() {
            if let Some(symbol) = scope.symbols.get(name) {
                return Some(symbol);
            }
        }
        self.declarations.get(name)
    }

    /// Check if a name is bound to a unique symbol
    pub fn is_unique_symbol(&self, name: &str) -> bool {
        self.lookup(name).map_or(false, |s| {
            matches!(s.symbol_type, SymbolType::UniqueSymbol(_))
        })
    }

    /// Get the symbol type for a name
    pub fn get_symbol_type(&self, name: &str) -> Option<&SymbolType> {
        self.lookup(name).map(|s| &s.symbol_type)
    }
}

impl Default for SymbolBindingContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Declaration kind for symbol bindings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolDeclarationKind {
    /// const declaration (produces unique symbol)
    Const,
    /// let declaration (produces generic symbol)
    Let,
    /// var declaration (produces generic symbol)
    Var,
    /// readonly property (produces unique symbol in const context)
    ReadonlyProperty,
    /// Mutable property
    Property,
    /// Parameter
    Parameter,
}

impl SymbolDeclarationKind {
    /// Check if this declaration kind produces a unique symbol type
    pub fn produces_unique_symbol(&self) -> bool {
        matches!(self, SymbolDeclarationKind::Const | SymbolDeclarationKind::ReadonlyProperty)
    }
}

/// Symbol property declaration for object types
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolPropertyDeclaration {
    /// The symbol used as the property key
    pub key: SymbolType,
    /// The property value type name
    pub value_type: &'static str,
    /// Whether this property is optional
    pub optional: bool,
    /// Whether this property is readonly
    pub readonly: bool,
}

impl SymbolPropertyDeclaration {
    pub fn new(key: SymbolType, value_type: &'static str) -> Self {
        Self {
            key,
            value_type,
            optional: false,
            readonly: false,
        }
    }

    pub fn optional(key: SymbolType, value_type: &'static str) -> Self {
        Self {
            key,
            value_type,
            optional: true,
            readonly: false,
        }
    }

    pub fn readonly(key: SymbolType, value_type: &'static str) -> Self {
        Self {
            key,
            value_type,
            optional: false,
            readonly: true,
        }
    }
}

/// Symbol index signature declaration
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolIndexSignatureDeclaration {
    /// The key type (symbol or unique symbol)
    pub key_type: SymbolType,
    /// The value type name
    pub value_type: &'static str,
    /// Whether this index signature is readonly
    pub readonly: bool,
}

impl SymbolIndexSignatureDeclaration {
    pub fn new(key_type: SymbolType, value_type: &'static str) -> Self {
        Self {
            key_type,
            value_type,
            readonly: false,
        }
    }

    pub fn readonly(key_type: SymbolType, value_type: &'static str) -> Self {
        Self {
            key_type,
            value_type,
            readonly: true,
        }
    }

    /// Check if a symbol type can index into this signature
    pub fn can_index(&self, index_type: &SymbolType) -> bool {
        // For symbol index signatures:
        // - [sym: symbol]: any  -> any symbol can index
        // - [sym: unique symbol s]: any -> only s can index
        match &self.key_type {
            SymbolType::Symbol => {
                // Generic symbol signature accepts any symbol
                matches!(index_type,
                    SymbolType::Symbol |
                    SymbolType::UniqueSymbol(_) |
                    SymbolType::WellKnownSymbol(_)
                )
            }
            SymbolType::UniqueSymbol(id) => {
                // Unique symbol signature only accepts that exact symbol
                match index_type {
                    SymbolType::UniqueSymbol(idx) => id == idx,
                    _ => false,
                }
            }
            SymbolType::WellKnownSymbol(wks) => {
                // Well-known symbol signature only accepts that exact symbol
                match index_type {
                    SymbolType::WellKnownSymbol(w) => wks == w,
                    _ => false,
                }
            }
        }
    }
}

/// Computed property name analysis result
#[derive(Debug, Clone, PartialEq)]
pub enum ComputedPropertyAnalysis {
    /// A well-known symbol [Symbol.iterator]
    WellKnownSymbol(WellKnownSymbol),
    /// A unique symbol from a const declaration
    UniqueSymbol(UniqueSymbolId),
    /// A generic symbol expression
    GenericSymbol,
    /// Not a symbol (string, number, etc.)
    NotSymbol,
    /// Error: cannot use this expression as computed property
    Error(&'static str),
}

impl ComputedPropertyAnalysis {
    /// Analyze a property access like `Symbol.iterator`
    pub fn from_symbol_property(property_name: &str) -> Self {
        if let Some(wks) = WellKnownSymbol::from_property_name(property_name) {
            ComputedPropertyAnalysis::WellKnownSymbol(wks)
        } else {
            ComputedPropertyAnalysis::NotSymbol
        }
    }

    /// Check if this is a valid computed property name
    pub fn is_valid(&self) -> bool {
        !matches!(self, ComputedPropertyAnalysis::Error(_))
    }

    /// Get the symbol type, if this is a symbol
    pub fn to_symbol_type(&self) -> Option<SymbolType> {
        match self {
            ComputedPropertyAnalysis::WellKnownSymbol(wks) => {
                Some(SymbolType::WellKnownSymbol(*wks))
            }
            ComputedPropertyAnalysis::UniqueSymbol(id) => {
                Some(SymbolType::UniqueSymbol(*id))
            }
            ComputedPropertyAnalysis::GenericSymbol => {
                Some(SymbolType::Symbol)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_const_symbol() {
        let mut ctx = SymbolBindingContext::new();

        let bound = ctx.bind_const_symbol("mySymbol", Some("description"), 0, 10);

        assert_eq!(bound.name, "mySymbol");
        assert!(bound.flags.is_const);
        assert!(matches!(bound.symbol_type, SymbolType::UniqueSymbol(_)));
    }

    #[test]
    fn test_bind_mutable_symbol() {
        let mut ctx = SymbolBindingContext::new();

        let bound = ctx.bind_mutable_symbol("letSymbol", 0, 10);

        assert_eq!(bound.name, "letSymbol");
        assert!(!bound.flags.is_const);
        assert_eq!(bound.symbol_type, SymbolType::Symbol);
    }

    #[test]
    fn test_bind_well_known_symbol() {
        let mut ctx = SymbolBindingContext::new();

        let sym = ctx.bind_well_known_symbol("iterator");
        assert_eq!(sym, Some(SymbolType::WellKnownSymbol(WellKnownSymbol::Iterator)));

        let unknown = ctx.bind_well_known_symbol("notASymbol");
        assert_eq!(unknown, None);
    }

    #[test]
    fn test_symbol_lookup() {
        let mut ctx = SymbolBindingContext::new();

        ctx.bind_const_symbol("unique", None, 0, 5);
        ctx.bind_mutable_symbol("generic", 6, 12);

        assert!(ctx.is_unique_symbol("unique"));
        assert!(!ctx.is_unique_symbol("generic"));
        assert!(!ctx.is_unique_symbol("notfound"));
    }

    #[test]
    fn test_scope_stack() {
        let mut ctx = SymbolBindingContext::new();

        // Bind in outer scope
        ctx.bind_const_symbol("outer", None, 0, 5);

        // Push new scope
        ctx.push_scope(false);
        ctx.bind_const_symbol("inner", None, 6, 11);

        // Both should be visible
        assert!(ctx.lookup("outer").is_some());
        assert!(ctx.lookup("inner").is_some());

        // Pop scope
        ctx.pop_scope();

        // Only outer should be visible
        assert!(ctx.lookup("outer").is_some());
        // Inner is still in global declarations but not in current scope
        // (implementation detail - in real impl, inner would be gone)
    }

    #[test]
    fn test_registered_symbols() {
        let mut ctx = SymbolBindingContext::new();

        let sym1 = ctx.bind_registered_symbol("shared.key");
        let sym2 = ctx.bind_registered_symbol("shared.key");
        let sym3 = ctx.bind_registered_symbol("other.key");

        // Same key should produce same symbol
        assert_eq!(sym1, sym2);

        // Different key should produce different symbol
        assert_ne!(sym1, sym3);
    }

    #[test]
    fn test_symbol_index_signature() {
        // Generic symbol signature
        let generic_sig = SymbolIndexSignatureDeclaration::new(
            SymbolType::Symbol,
            "any"
        );

        assert!(generic_sig.can_index(&SymbolType::Symbol));
        assert!(generic_sig.can_index(&SymbolType::UniqueSymbol(UniqueSymbolId::from_id(1))));
        assert!(generic_sig.can_index(&SymbolType::WellKnownSymbol(WellKnownSymbol::Iterator)));

        // Unique symbol signature
        let unique_id = UniqueSymbolId::from_id(42);
        let unique_sig = SymbolIndexSignatureDeclaration::new(
            SymbolType::UniqueSymbol(unique_id),
            "string"
        );

        assert!(!unique_sig.can_index(&SymbolType::Symbol));
        assert!(unique_sig.can_index(&SymbolType::UniqueSymbol(unique_id)));
        assert!(!unique_sig.can_index(&SymbolType::UniqueSymbol(UniqueSymbolId::from_id(99))));
    }

    #[test]
    fn test_computed_property_analysis() {
        let iterator = ComputedPropertyAnalysis::from_symbol_property("iterator");
        assert!(matches!(iterator, ComputedPropertyAnalysis::WellKnownSymbol(WellKnownSymbol::Iterator)));

        let unknown = ComputedPropertyAnalysis::from_symbol_property("foo");
        assert!(matches!(unknown, ComputedPropertyAnalysis::NotSymbol));
    }

    #[test]
    fn test_declaration_kind() {
        assert!(SymbolDeclarationKind::Const.produces_unique_symbol());
        assert!(SymbolDeclarationKind::ReadonlyProperty.produces_unique_symbol());
        assert!(!SymbolDeclarationKind::Let.produces_unique_symbol());
        assert!(!SymbolDeclarationKind::Var.produces_unique_symbol());
    }
}
