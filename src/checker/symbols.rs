//! Symbol type checking and analysis
//!
//! This module implements TypeScript's symbol type system, including:
//! - Unique symbol types for const symbol declarations
//! - Well-known symbols (Symbol.iterator, Symbol.asyncIterator, etc.)
//! - Symbol index signatures
//! - Symbol property access and narrowing
//! - Symbol assignability rules

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique ID generator for unique symbols
static UNIQUE_SYMBOL_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Represents a TypeScript symbol type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolType {
    /// The generic `symbol` type
    Symbol,

    /// A unique symbol type (created by `const sym = Symbol()`)
    /// Each unique symbol has a unique ID and optional name
    UniqueSymbol(UniqueSymbolId),

    /// A well-known symbol (Symbol.iterator, etc.)
    WellKnownSymbol(WellKnownSymbol),
}

/// Identifier for a unique symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UniqueSymbolId {
    /// Unique numeric identifier
    pub id: u64,
    /// Declaration name (for display purposes)
    pub name: Option<&'static str>,
}

impl UniqueSymbolId {
    /// Create a new unique symbol ID
    pub fn new() -> Self {
        Self {
            id: UNIQUE_SYMBOL_COUNTER.fetch_add(1, Ordering::SeqCst),
            name: None,
        }
    }

    /// Create a new unique symbol ID with a name
    pub fn with_name(name: &'static str) -> Self {
        Self {
            id: UNIQUE_SYMBOL_COUNTER.fetch_add(1, Ordering::SeqCst),
            name: Some(name),
        }
    }

    /// Create from an existing ID (for testing)
    pub fn from_id(id: u64) -> Self {
        Self { id, name: None }
    }
}

impl Default for UniqueSymbolId {
    fn default() -> Self {
        Self::new()
    }
}

/// Well-known symbols defined in ECMAScript
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WellKnownSymbol {
    /// Symbol.asyncIterator
    AsyncIterator,
    /// Symbol.hasInstance
    HasInstance,
    /// Symbol.isConcatSpreadable
    IsConcatSpreadable,
    /// Symbol.iterator
    Iterator,
    /// Symbol.match
    Match,
    /// Symbol.matchAll
    MatchAll,
    /// Symbol.replace
    Replace,
    /// Symbol.search
    Search,
    /// Symbol.species
    Species,
    /// Symbol.split
    Split,
    /// Symbol.toPrimitive
    ToPrimitive,
    /// Symbol.toStringTag
    ToStringTag,
    /// Symbol.unscopables
    Unscopables,
    /// Symbol.dispose (ES2024)
    Dispose,
    /// Symbol.asyncDispose (ES2024)
    AsyncDispose,
}

impl WellKnownSymbol {
    /// Get the symbol property name (e.g., "iterator" for Symbol.iterator)
    pub fn property_name(&self) -> &'static str {
        match self {
            WellKnownSymbol::AsyncIterator => "asyncIterator",
            WellKnownSymbol::HasInstance => "hasInstance",
            WellKnownSymbol::IsConcatSpreadable => "isConcatSpreadable",
            WellKnownSymbol::Iterator => "iterator",
            WellKnownSymbol::Match => "match",
            WellKnownSymbol::MatchAll => "matchAll",
            WellKnownSymbol::Replace => "replace",
            WellKnownSymbol::Search => "search",
            WellKnownSymbol::Species => "species",
            WellKnownSymbol::Split => "split",
            WellKnownSymbol::ToPrimitive => "toPrimitive",
            WellKnownSymbol::ToStringTag => "toStringTag",
            WellKnownSymbol::Unscopables => "unscopables",
            WellKnownSymbol::Dispose => "dispose",
            WellKnownSymbol::AsyncDispose => "asyncDispose",
        }
    }

    /// Get the full Symbol property access (e.g., "Symbol.iterator")
    pub fn full_name(&self) -> &'static str {
        match self {
            WellKnownSymbol::AsyncIterator => "Symbol.asyncIterator",
            WellKnownSymbol::HasInstance => "Symbol.hasInstance",
            WellKnownSymbol::IsConcatSpreadable => "Symbol.isConcatSpreadable",
            WellKnownSymbol::Iterator => "Symbol.iterator",
            WellKnownSymbol::Match => "Symbol.match",
            WellKnownSymbol::MatchAll => "Symbol.matchAll",
            WellKnownSymbol::Replace => "Symbol.replace",
            WellKnownSymbol::Search => "Symbol.search",
            WellKnownSymbol::Species => "Symbol.species",
            WellKnownSymbol::Split => "Symbol.split",
            WellKnownSymbol::ToPrimitive => "Symbol.toPrimitive",
            WellKnownSymbol::ToStringTag => "Symbol.toStringTag",
            WellKnownSymbol::Unscopables => "Symbol.unscopables",
            WellKnownSymbol::Dispose => "Symbol.dispose",
            WellKnownSymbol::AsyncDispose => "Symbol.asyncDispose",
        }
    }

    /// Parse a property name into a well-known symbol
    pub fn from_property_name(name: &str) -> Option<Self> {
        match name {
            "asyncIterator" => Some(WellKnownSymbol::AsyncIterator),
            "hasInstance" => Some(WellKnownSymbol::HasInstance),
            "isConcatSpreadable" => Some(WellKnownSymbol::IsConcatSpreadable),
            "iterator" => Some(WellKnownSymbol::Iterator),
            "match" => Some(WellKnownSymbol::Match),
            "matchAll" => Some(WellKnownSymbol::MatchAll),
            "replace" => Some(WellKnownSymbol::Replace),
            "search" => Some(WellKnownSymbol::Search),
            "species" => Some(WellKnownSymbol::Species),
            "split" => Some(WellKnownSymbol::Split),
            "toPrimitive" => Some(WellKnownSymbol::ToPrimitive),
            "toStringTag" => Some(WellKnownSymbol::ToStringTag),
            "unscopables" => Some(WellKnownSymbol::Unscopables),
            "dispose" => Some(WellKnownSymbol::Dispose),
            "asyncDispose" => Some(WellKnownSymbol::AsyncDispose),
            _ => None,
        }
    }

    /// Get all well-known symbols
    pub fn all() -> &'static [WellKnownSymbol] {
        &[
            WellKnownSymbol::AsyncIterator,
            WellKnownSymbol::HasInstance,
            WellKnownSymbol::IsConcatSpreadable,
            WellKnownSymbol::Iterator,
            WellKnownSymbol::Match,
            WellKnownSymbol::MatchAll,
            WellKnownSymbol::Replace,
            WellKnownSymbol::Search,
            WellKnownSymbol::Species,
            WellKnownSymbol::Split,
            WellKnownSymbol::ToPrimitive,
            WellKnownSymbol::ToStringTag,
            WellKnownSymbol::Unscopables,
            WellKnownSymbol::Dispose,
            WellKnownSymbol::AsyncDispose,
        ]
    }
}

/// Symbol declaration flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolDeclarationFlags {
    /// Whether this is a const declaration (produces unique symbol type)
    pub is_const: bool,
    /// Whether this symbol was created via Symbol.for() (shared registry)
    pub is_registered: bool,
    /// Whether this is a well-known symbol
    pub is_well_known: bool,
}

impl Default for SymbolDeclarationFlags {
    fn default() -> Self {
        Self {
            is_const: false,
            is_registered: false,
            is_well_known: false,
        }
    }
}

impl SymbolDeclarationFlags {
    pub fn const_declaration() -> Self {
        Self {
            is_const: true,
            is_registered: false,
            is_well_known: false,
        }
    }

    pub fn registered() -> Self {
        Self {
            is_const: false,
            is_registered: true,
            is_well_known: false,
        }
    }

    pub fn well_known() -> Self {
        Self {
            is_const: false,
            is_registered: false,
            is_well_known: true,
        }
    }
}

/// Symbol assignability checker
pub struct SymbolAssignability;

impl SymbolAssignability {
    /// Check if source symbol type is assignable to target symbol type
    ///
    /// Rules:
    /// - `symbol` is assignable to `symbol`
    /// - `unique symbol` is assignable to `symbol`
    /// - `unique symbol` is NOT assignable to a different `unique symbol`
    /// - `unique symbol` IS assignable to itself
    /// - Well-known symbols are assignable to `symbol`
    /// - Well-known symbols are only assignable to their exact type
    pub fn is_assignable(source: &SymbolType, target: &SymbolType) -> bool {
        match (source, target) {
            // Symbol is assignable to symbol
            (SymbolType::Symbol, SymbolType::Symbol) => true,

            // Unique symbol is assignable to symbol (widening)
            (SymbolType::UniqueSymbol(_), SymbolType::Symbol) => true,

            // Unique symbol is only assignable to itself
            (SymbolType::UniqueSymbol(s), SymbolType::UniqueSymbol(t)) => s == t,

            // Well-known symbol is assignable to symbol (widening)
            (SymbolType::WellKnownSymbol(_), SymbolType::Symbol) => true,

            // Well-known symbol is only assignable to exact same well-known symbol
            (SymbolType::WellKnownSymbol(s), SymbolType::WellKnownSymbol(t)) => s == t,

            // Generic symbol is NOT assignable to unique/well-known symbol
            (SymbolType::Symbol, SymbolType::UniqueSymbol(_)) => false,
            (SymbolType::Symbol, SymbolType::WellKnownSymbol(_)) => false,

            // Cross-category assignments are not allowed
            (SymbolType::UniqueSymbol(_), SymbolType::WellKnownSymbol(_)) => false,
            (SymbolType::WellKnownSymbol(_), SymbolType::UniqueSymbol(_)) => false,
        }
    }

    /// Check if a type can be used as a symbol index key
    ///
    /// Only `symbol`, `unique symbol`, and well-known symbols can be index keys
    pub fn is_valid_index_key(symbol_type: &SymbolType) -> bool {
        // All symbol types can be used as index keys
        true
    }

    /// Check if two symbol types are identical (for strict equality)
    pub fn is_identical(a: &SymbolType, b: &SymbolType) -> bool {
        a == b
    }
}

/// Symbol index signature representation
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolIndexSignature {
    /// The symbol key type (symbol, unique symbol, or well-known symbol)
    pub key_type: SymbolType,
    /// The value type for this index signature
    pub value_type: SymbolIndexValueType,
    /// Whether this index signature is readonly
    pub readonly: bool,
}

/// Placeholder for index signature value type
/// (In a full implementation, this would reference the main Type enum)
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolIndexValueType {
    Any,
    Unknown,
    String,
    Number,
    Boolean,
    TypeReference(&'static str),
}

impl SymbolIndexSignature {
    pub fn new(key_type: SymbolType, value_type: SymbolIndexValueType) -> Self {
        Self {
            key_type,
            value_type,
            readonly: false,
        }
    }

    pub fn readonly(key_type: SymbolType, value_type: SymbolIndexValueType) -> Self {
        Self {
            key_type,
            value_type,
            readonly: true,
        }
    }
}

/// Symbol property in an object type
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolProperty {
    /// The symbol used as the property key
    pub symbol: SymbolType,
    /// The property value type
    pub value_type: SymbolIndexValueType,
    /// Whether this property is optional
    pub optional: bool,
    /// Whether this property is readonly
    pub readonly: bool,
}

impl SymbolProperty {
    pub fn new(symbol: SymbolType, value_type: SymbolIndexValueType) -> Self {
        Self {
            symbol,
            value_type,
            optional: false,
            readonly: false,
        }
    }

    pub fn optional(symbol: SymbolType, value_type: SymbolIndexValueType) -> Self {
        Self {
            symbol,
            value_type,
            optional: true,
            readonly: false,
        }
    }
}

/// Symbol narrowing context for type guards
#[derive(Debug, Clone)]
pub struct SymbolNarrowingContext {
    /// Map from symbol ID to narrowed type
    narrowed_symbols: HashMap<u64, SymbolType>,
}

impl SymbolNarrowingContext {
    pub fn new() -> Self {
        Self {
            narrowed_symbols: HashMap::new(),
        }
    }

    /// Record a narrowed symbol type
    pub fn narrow(&mut self, symbol_id: u64, narrowed_type: SymbolType) {
        self.narrowed_symbols.insert(symbol_id, narrowed_type);
    }

    /// Get the narrowed type for a symbol, if any
    pub fn get_narrowed(&self, symbol_id: u64) -> Option<&SymbolType> {
        self.narrowed_symbols.get(&symbol_id)
    }

    /// Check if a symbol has been narrowed
    pub fn is_narrowed(&self, symbol_id: u64) -> bool {
        self.narrowed_symbols.contains_key(&symbol_id)
    }

    /// Merge with another narrowing context (for branches)
    pub fn merge(&mut self, other: &SymbolNarrowingContext) {
        for (id, typ) in &other.narrowed_symbols {
            // Only keep narrowings that exist in both branches
            if self.narrowed_symbols.contains_key(id) {
                // If types differ, widen to generic symbol
                if self.narrowed_symbols.get(id) != Some(typ) {
                    self.narrowed_symbols.insert(*id, SymbolType::Symbol);
                }
            }
        }
    }

    /// Create a snapshot for branching
    pub fn snapshot(&self) -> Self {
        self.clone()
    }
}

impl Default for SymbolNarrowingContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Typeof operator result for symbols
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolTypeofResult {
    /// The string literal type "symbol"
    pub type_string: &'static str,
    /// The actual symbol type for further narrowing
    pub symbol_type: SymbolType,
}

impl SymbolTypeofResult {
    /// Create a typeof result for a symbol
    pub fn new(symbol_type: SymbolType) -> Self {
        Self {
            type_string: "symbol",
            symbol_type,
        }
    }

    /// Check if typeof x === "symbol"
    pub fn is_symbol_typeof(&self) -> bool {
        self.type_string == "symbol"
    }
}

/// Computed property name with symbol
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolComputedProperty {
    /// The symbol expression
    pub symbol: SymbolType,
    /// Whether this is a bracketed access [Symbol.iterator]
    pub is_bracketed: bool,
}

impl SymbolComputedProperty {
    pub fn new(symbol: SymbolType) -> Self {
        Self {
            symbol,
            is_bracketed: true,
        }
    }

    /// Create a well-known symbol computed property
    pub fn well_known(wks: WellKnownSymbol) -> Self {
        Self {
            symbol: SymbolType::WellKnownSymbol(wks),
            is_bracketed: true,
        }
    }
}

/// Template literal type with symbol
///
/// Note: Symbols cannot be directly used in template literal types,
/// but we need to handle errors appropriately when they appear
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolInTemplateLiteral {
    /// Error: Symbol cannot be used in template literal type
    Error(&'static str),
    /// Symbol.toStringTag might produce a string
    ToStringTagResult,
}

impl SymbolInTemplateLiteral {
    /// Check if a symbol type can be used in a template literal
    pub fn check(symbol_type: &SymbolType) -> Self {
        match symbol_type {
            SymbolType::WellKnownSymbol(WellKnownSymbol::ToStringTag) => {
                // Symbol.toStringTag is used for customizing Object.prototype.toString
                SymbolInTemplateLiteral::ToStringTagResult
            }
            _ => {
                SymbolInTemplateLiteral::Error(
                    "Type 'symbol' cannot be used as an index type"
                )
            }
        }
    }
}

/// Symbol registry for Symbol.for() calls
#[derive(Debug, Clone, Default)]
pub struct SymbolRegistry {
    /// Map from symbol key to unique symbol ID
    registry: HashMap<String, UniqueSymbolId>,
}

impl SymbolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create a symbol from the registry (Symbol.for)
    pub fn get_or_create(&mut self, key: &str) -> UniqueSymbolId {
        if let Some(existing) = self.registry.get(key) {
            *existing
        } else {
            let id = UniqueSymbolId::new();
            self.registry.insert(key.to_string(), id);
            id
        }
    }

    /// Get a symbol from the registry if it exists (Symbol.keyFor)
    pub fn get(&self, key: &str) -> Option<UniqueSymbolId> {
        self.registry.get(key).copied()
    }

    /// Check if a symbol is in the registry
    pub fn contains(&self, key: &str) -> bool {
        self.registry.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_symbol_creation() {
        let sym1 = UniqueSymbolId::new();
        let sym2 = UniqueSymbolId::new();

        // Each unique symbol should have a different ID
        assert_ne!(sym1.id, sym2.id);

        // Named symbols should work
        let named = UniqueSymbolId::with_name("mySymbol");
        assert_eq!(named.name, Some("mySymbol"));
    }

    #[test]
    fn test_well_known_symbols() {
        assert_eq!(WellKnownSymbol::Iterator.property_name(), "iterator");
        assert_eq!(WellKnownSymbol::Iterator.full_name(), "Symbol.iterator");

        assert_eq!(
            WellKnownSymbol::from_property_name("asyncIterator"),
            Some(WellKnownSymbol::AsyncIterator)
        );

        assert_eq!(WellKnownSymbol::from_property_name("unknown"), None);
    }

    #[test]
    fn test_symbol_assignability() {
        let generic = SymbolType::Symbol;
        let unique1 = SymbolType::UniqueSymbol(UniqueSymbolId::from_id(1));
        let unique2 = SymbolType::UniqueSymbol(UniqueSymbolId::from_id(2));
        let iterator = SymbolType::WellKnownSymbol(WellKnownSymbol::Iterator);
        let async_iter = SymbolType::WellKnownSymbol(WellKnownSymbol::AsyncIterator);

        // symbol -> symbol
        assert!(SymbolAssignability::is_assignable(&generic, &generic));

        // unique symbol -> symbol (widening)
        assert!(SymbolAssignability::is_assignable(&unique1, &generic));

        // unique symbol -> same unique symbol
        assert!(SymbolAssignability::is_assignable(&unique1, &unique1));

        // unique symbol -> different unique symbol (ERROR)
        assert!(!SymbolAssignability::is_assignable(&unique1, &unique2));

        // well-known symbol -> symbol (widening)
        assert!(SymbolAssignability::is_assignable(&iterator, &generic));

        // well-known symbol -> same well-known symbol
        assert!(SymbolAssignability::is_assignable(&iterator, &iterator));

        // well-known symbol -> different well-known symbol (ERROR)
        assert!(!SymbolAssignability::is_assignable(&iterator, &async_iter));

        // symbol -> unique symbol (ERROR)
        assert!(!SymbolAssignability::is_assignable(&generic, &unique1));

        // symbol -> well-known symbol (ERROR)
        assert!(!SymbolAssignability::is_assignable(&generic, &iterator));
    }

    #[test]
    fn test_symbol_registry() {
        let mut registry = SymbolRegistry::new();

        let sym1 = registry.get_or_create("shared");
        let sym2 = registry.get_or_create("shared");

        // Same key should return same symbol
        assert_eq!(sym1.id, sym2.id);

        let sym3 = registry.get_or_create("other");
        assert_ne!(sym1.id, sym3.id);

        // Check lookup
        assert!(registry.contains("shared"));
        assert!(!registry.contains("nonexistent"));
    }

    #[test]
    fn test_symbol_narrowing() {
        let mut ctx = SymbolNarrowingContext::new();

        let unique = SymbolType::UniqueSymbol(UniqueSymbolId::from_id(42));
        ctx.narrow(42, unique.clone());

        assert!(ctx.is_narrowed(42));
        assert_eq!(ctx.get_narrowed(42), Some(&unique));
        assert!(!ctx.is_narrowed(99));
    }

    #[test]
    fn test_symbol_typeof() {
        let result = SymbolTypeofResult::new(SymbolType::Symbol);
        assert!(result.is_symbol_typeof());
        assert_eq!(result.type_string, "symbol");

        let unique_result = SymbolTypeofResult::new(
            SymbolType::UniqueSymbol(UniqueSymbolId::from_id(1))
        );
        assert!(unique_result.is_symbol_typeof());
    }

    #[test]
    fn test_symbol_computed_property() {
        let prop = SymbolComputedProperty::well_known(WellKnownSymbol::Iterator);
        assert!(prop.is_bracketed);
        assert_eq!(
            prop.symbol,
            SymbolType::WellKnownSymbol(WellKnownSymbol::Iterator)
        );
    }

    #[test]
    fn test_symbol_index_signature() {
        let sig = SymbolIndexSignature::new(
            SymbolType::Symbol,
            SymbolIndexValueType::String
        );
        assert!(!sig.readonly);

        let readonly_sig = SymbolIndexSignature::readonly(
            SymbolType::Symbol,
            SymbolIndexValueType::Number
        );
        assert!(readonly_sig.readonly);
    }
}
