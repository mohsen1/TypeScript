//! Unique symbol type representation
//!
//! This module provides the type system representation for unique symbols,
//! which are created when a symbol is declared with `const`.
//!
//! Example TypeScript:
//! ```typescript
//! const mySymbol = Symbol("description");  // type is `typeof mySymbol` (unique symbol)
//! let otherSymbol = Symbol("description");  // type is `symbol` (not unique)
//! ```

use crate::checker::symbols::{UniqueSymbolId, WellKnownSymbol, SymbolType};
use std::fmt;

/// Represents a unique symbol type in the TypeScript type system
///
/// Unique symbols are created when:
/// 1. A symbol is declared with `const` keyword
/// 2. A property is declared with `readonly` and has a Symbol() initializer
/// 3. A well-known symbol is accessed (Symbol.iterator, etc.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniqueSymbolType {
    /// The unique identifier for this symbol
    pub id: UniqueSymbolId,
    /// The declaration name (if available)
    pub declaration_name: Option<&'static str>,
    /// The symbol description (from Symbol("description"))
    pub description: Option<&'static str>,
    /// Whether this is a well-known symbol
    pub well_known: Option<WellKnownSymbol>,
}

impl UniqueSymbolType {
    /// Create a new unique symbol type
    pub fn new(id: UniqueSymbolId) -> Self {
        Self {
            id,
            declaration_name: None,
            description: None,
            well_known: None,
        }
    }

    /// Create a unique symbol type with a declaration name
    pub fn with_declaration(id: UniqueSymbolId, name: &'static str) -> Self {
        Self {
            id,
            declaration_name: Some(name),
            description: None,
            well_known: None,
        }
    }

    /// Create a unique symbol type with a description
    pub fn with_description(id: UniqueSymbolId, description: &'static str) -> Self {
        Self {
            id,
            declaration_name: None,
            description: Some(description),
            well_known: None,
        }
    }

    /// Create a well-known symbol type
    pub fn well_known(wks: WellKnownSymbol) -> Self {
        Self {
            id: UniqueSymbolId::with_name(wks.property_name()),
            declaration_name: None,
            description: None,
            well_known: Some(wks),
        }
    }

    /// Get the display name for this unique symbol type
    pub fn display_name(&self) -> String {
        if let Some(wks) = self.well_known {
            return format!("typeof {}", wks.full_name());
        }

        if let Some(decl_name) = self.declaration_name {
            return format!("typeof {}", decl_name);
        }

        if let Some(desc) = self.description {
            return format!("unique symbol ({})", desc);
        }

        format!("unique symbol")
    }

    /// Check if this is a well-known symbol
    pub fn is_well_known(&self) -> bool {
        self.well_known.is_some()
    }

    /// Get the well-known symbol, if any
    pub fn get_well_known(&self) -> Option<WellKnownSymbol> {
        self.well_known
    }

    /// Convert to the generic SymbolType enum
    pub fn to_symbol_type(&self) -> SymbolType {
        if let Some(wks) = self.well_known {
            SymbolType::WellKnownSymbol(wks)
        } else {
            SymbolType::UniqueSymbol(self.id)
        }
    }
}

impl fmt::Display for UniqueSymbolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Factory for creating unique symbol types
#[derive(Debug, Default)]
pub struct UniqueSymbolFactory {
    /// Counter for generating unique IDs
    counter: std::sync::atomic::AtomicU64,
}

impl UniqueSymbolFactory {
    pub fn new() -> Self {
        Self {
            counter: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Create a new unique symbol type
    pub fn create(&self) -> UniqueSymbolType {
        let id = UniqueSymbolId::from_id(
            self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        );
        UniqueSymbolType::new(id)
    }

    /// Create a unique symbol with a declaration name
    pub fn create_with_name(&self, name: &'static str) -> UniqueSymbolType {
        let id = UniqueSymbolId::from_id(
            self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        );
        UniqueSymbolType::with_declaration(id, name)
    }

    /// Create a well-known symbol type
    pub fn create_well_known(&self, wks: WellKnownSymbol) -> UniqueSymbolType {
        UniqueSymbolType::well_known(wks)
    }
}

/// Type predicate for checking if a type is a unique symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniqueSymbolPredicate {
    /// The type is exactly a specific unique symbol
    Exact(u64),
    /// The type is any unique symbol
    AnyUnique,
    /// The type is a well-known symbol
    WellKnown(WellKnownSymbol),
    /// The type is the generic `symbol` type
    GenericSymbol,
}

impl UniqueSymbolPredicate {
    /// Check if a symbol type matches this predicate
    pub fn matches(&self, symbol_type: &SymbolType) -> bool {
        match (self, symbol_type) {
            (UniqueSymbolPredicate::Exact(id), SymbolType::UniqueSymbol(uid)) => {
                uid.id == *id
            }
            (UniqueSymbolPredicate::AnyUnique, SymbolType::UniqueSymbol(_)) => true,
            (UniqueSymbolPredicate::WellKnown(wks), SymbolType::WellKnownSymbol(w)) => {
                wks == w
            }
            (UniqueSymbolPredicate::GenericSymbol, SymbolType::Symbol) => true,
            _ => false,
        }
    }
}

/// Const symbol declaration info
///
/// When `const sym = Symbol()` is encountered, this captures the type info
#[derive(Debug, Clone, PartialEq)]
pub struct ConstSymbolDeclaration {
    /// The variable name
    pub name: &'static str,
    /// The unique symbol type
    pub symbol_type: UniqueSymbolType,
    /// The source position (start offset)
    pub start: usize,
    /// The source position (end offset)
    pub end: usize,
}

impl ConstSymbolDeclaration {
    pub fn new(name: &'static str, symbol_type: UniqueSymbolType, start: usize, end: usize) -> Self {
        Self {
            name,
            symbol_type,
            start,
            end,
        }
    }
}

/// Typeof expression result for unique symbols
///
/// When `typeof sym` is used where `sym` is a unique symbol,
/// the result type is the literal type "symbol" for runtime,
/// but for type-level operations it preserves the unique symbol type.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeofUniqueSymbol {
    /// The unique symbol being queried
    pub symbol: UniqueSymbolType,
    /// The runtime typeof result (always "symbol")
    pub runtime_type: &'static str,
}

impl TypeofUniqueSymbol {
    pub fn new(symbol: UniqueSymbolType) -> Self {
        Self {
            symbol,
            runtime_type: "symbol",
        }
    }

    /// Get the type representation for this typeof expression
    pub fn as_type(&self) -> TypeofResultType {
        TypeofResultType::UniqueSymbol(self.symbol.clone())
    }
}

/// Typeof result type variants
#[derive(Debug, Clone, PartialEq)]
pub enum TypeofResultType {
    /// A unique symbol type (preserves uniqueness)
    UniqueSymbol(UniqueSymbolType),
    /// A generic symbol type
    Symbol,
    /// A well-known symbol type
    WellKnownSymbol(WellKnownSymbol),
}

impl TypeofResultType {
    /// Get the runtime string representation
    pub fn runtime_string(&self) -> &'static str {
        "symbol"
    }

    /// Convert to SymbolType
    pub fn to_symbol_type(&self) -> SymbolType {
        match self {
            TypeofResultType::UniqueSymbol(u) => u.to_symbol_type(),
            TypeofResultType::Symbol => SymbolType::Symbol,
            TypeofResultType::WellKnownSymbol(wks) => SymbolType::WellKnownSymbol(*wks),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_symbol_type_creation() {
        let id = UniqueSymbolId::from_id(42);
        let sym = UniqueSymbolType::new(id);

        assert_eq!(sym.id.id, 42);
        assert!(sym.declaration_name.is_none());
        assert!(sym.well_known.is_none());
    }

    #[test]
    fn test_unique_symbol_with_declaration() {
        let id = UniqueSymbolId::from_id(1);
        let sym = UniqueSymbolType::with_declaration(id, "mySymbol");

        assert_eq!(sym.declaration_name, Some("mySymbol"));
        assert_eq!(sym.display_name(), "typeof mySymbol");
    }

    #[test]
    fn test_well_known_symbol_type() {
        let sym = UniqueSymbolType::well_known(WellKnownSymbol::Iterator);

        assert!(sym.is_well_known());
        assert_eq!(sym.get_well_known(), Some(WellKnownSymbol::Iterator));
        assert_eq!(sym.display_name(), "typeof Symbol.iterator");
    }

    #[test]
    fn test_unique_symbol_factory() {
        let factory = UniqueSymbolFactory::new();

        let sym1 = factory.create();
        let sym2 = factory.create();

        // Should have different IDs
        assert_ne!(sym1.id.id, sym2.id.id);

        // Named symbols
        let named = factory.create_with_name("test");
        assert_eq!(named.declaration_name, Some("test"));
    }

    #[test]
    fn test_unique_symbol_predicate() {
        let unique = SymbolType::UniqueSymbol(UniqueSymbolId::from_id(42));
        let generic = SymbolType::Symbol;
        let iterator = SymbolType::WellKnownSymbol(WellKnownSymbol::Iterator);

        assert!(UniqueSymbolPredicate::Exact(42).matches(&unique));
        assert!(!UniqueSymbolPredicate::Exact(99).matches(&unique));
        assert!(UniqueSymbolPredicate::AnyUnique.matches(&unique));
        assert!(!UniqueSymbolPredicate::AnyUnique.matches(&generic));
        assert!(UniqueSymbolPredicate::GenericSymbol.matches(&generic));
        assert!(UniqueSymbolPredicate::WellKnown(WellKnownSymbol::Iterator).matches(&iterator));
    }

    #[test]
    fn test_typeof_unique_symbol() {
        let id = UniqueSymbolId::from_id(1);
        let sym = UniqueSymbolType::with_declaration(id, "mySym");
        let typeof_result = TypeofUniqueSymbol::new(sym.clone());

        assert_eq!(typeof_result.runtime_type, "symbol");

        match typeof_result.as_type() {
            TypeofResultType::UniqueSymbol(u) => {
                assert_eq!(u.declaration_name, Some("mySym"));
            }
            _ => panic!("Expected UniqueSymbol"),
        }
    }
}
