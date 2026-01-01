//! Binder implementation for TypeScript AST.
//!
//! The binder walks the AST and creates symbols, establishing
//! scope and name resolution.

use serde::Serialize;
use crate::parser::NodeIndex;

// =============================================================================
// Symbol Flags
// =============================================================================

/// Flags that describe the kind and properties of a symbol.
/// Matches TypeScript's SymbolFlags enum in src/compiler/types.ts
pub mod symbol_flags {
    pub const NONE: u32 = 0;
    pub const FUNCTION_SCOPED_VARIABLE: u32 = 1 << 0;   // Variable (var) or parameter
    pub const BLOCK_SCOPED_VARIABLE: u32 = 1 << 1;      // Block-scoped variable (let or const)
    pub const PROPERTY: u32 = 1 << 2;                   // Property or enum member
    pub const ENUM_MEMBER: u32 = 1 << 3;                // Enum member
    pub const FUNCTION: u32 = 1 << 4;                   // Function
    pub const CLASS: u32 = 1 << 5;                      // Class
    pub const INTERFACE: u32 = 1 << 6;                  // Interface
    pub const CONST_ENUM: u32 = 1 << 7;                 // Const enum
    pub const REGULAR_ENUM: u32 = 1 << 8;               // Enum
    pub const VALUE_MODULE: u32 = 1 << 9;               // Instantiated module
    pub const NAMESPACE_MODULE: u32 = 1 << 10;          // Uninstantiated module
    pub const TYPE_LITERAL: u32 = 1 << 11;              // Type Literal or mapped type
    pub const OBJECT_LITERAL: u32 = 1 << 12;            // Object Literal
    pub const METHOD: u32 = 1 << 13;                    // Method
    pub const CONSTRUCTOR: u32 = 1 << 14;               // Constructor
    pub const GET_ACCESSOR: u32 = 1 << 15;              // Get accessor
    pub const SET_ACCESSOR: u32 = 1 << 16;              // Set accessor
    pub const SIGNATURE: u32 = 1 << 17;                 // Call, construct, or index signature
    pub const TYPE_PARAMETER: u32 = 1 << 18;            // Type parameter
    pub const TYPE_ALIAS: u32 = 1 << 19;                // Type alias
    pub const EXPORT_VALUE: u32 = 1 << 20;              // Exported value marker
    pub const ALIAS: u32 = 1 << 21;                     // Alias for another symbol
    pub const PROTOTYPE: u32 = 1 << 22;                 // Prototype property
    pub const EXPORT_STAR: u32 = 1 << 23;               // Export * declaration
    pub const OPTIONAL: u32 = 1 << 24;                  // Optional property
    pub const TRANSIENT: u32 = 1 << 25;                 // Transient symbol
    pub const ASSIGNMENT: u32 = 1 << 26;                // Assignment treated as declaration
    pub const MODULE_EXPORTS: u32 = 1 << 27;            // CommonJS module.exports

    // Composite flags
    pub const ENUM: u32 = REGULAR_ENUM | CONST_ENUM;
    pub const VARIABLE: u32 = FUNCTION_SCOPED_VARIABLE | BLOCK_SCOPED_VARIABLE;
    pub const VALUE: u32 = VARIABLE | PROPERTY | ENUM_MEMBER | OBJECT_LITERAL |
                           FUNCTION | CLASS | ENUM | VALUE_MODULE | METHOD |
                           GET_ACCESSOR | SET_ACCESSOR;
    pub const TYPE: u32 = CLASS | INTERFACE | ENUM | ENUM_MEMBER | TYPE_LITERAL |
                          TYPE_PARAMETER | TYPE_ALIAS;
    pub const NAMESPACE: u32 = VALUE_MODULE | NAMESPACE_MODULE | ENUM;
    pub const MODULE: u32 = VALUE_MODULE | NAMESPACE_MODULE;
    pub const ACCESSOR: u32 = GET_ACCESSOR | SET_ACCESSOR;

    // Exclusion flags for redeclaration checks
    pub const FUNCTION_SCOPED_VARIABLE_EXCLUDES: u32 = VALUE & !FUNCTION_SCOPED_VARIABLE;
    pub const BLOCK_SCOPED_VARIABLE_EXCLUDES: u32 = VALUE;
    pub const PARAMETER_EXCLUDES: u32 = VALUE;
    pub const PROPERTY_EXCLUDES: u32 = NONE;
    pub const ENUM_MEMBER_EXCLUDES: u32 = VALUE | TYPE;
    pub const FUNCTION_EXCLUDES: u32 = VALUE & !FUNCTION;
    pub const CLASS_EXCLUDES: u32 = VALUE | TYPE & !VALUE_MODULE & !INTERFACE & !FUNCTION;
    pub const INTERFACE_EXCLUDES: u32 = TYPE & !INTERFACE & !CLASS;
    pub const REGULAR_ENUM_EXCLUDES: u32 = VALUE | TYPE & !REGULAR_ENUM;
    pub const CONST_ENUM_EXCLUDES: u32 = VALUE | TYPE & !CONST_ENUM;
    pub const VALUE_MODULE_EXCLUDES: u32 = VALUE & !FUNCTION & !CLASS & !REGULAR_ENUM & !VALUE_MODULE;
    pub const NAMESPACE_MODULE_EXCLUDES: u32 = NONE;
    pub const METHOD_EXCLUDES: u32 = VALUE & !METHOD;
    pub const GET_ACCESSOR_EXCLUDES: u32 = VALUE & !SET_ACCESSOR;
    pub const SET_ACCESSOR_EXCLUDES: u32 = VALUE & !GET_ACCESSOR;
    pub const TYPE_PARAMETER_EXCLUDES: u32 = TYPE & !TYPE_PARAMETER;
    pub const TYPE_ALIAS_EXCLUDES: u32 = TYPE;
    pub const ALIAS_EXCLUDES: u32 = ALIAS;
}

// =============================================================================
// Symbol
// =============================================================================

/// Unique identifier for a symbol in the symbol table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct SymbolId(pub u32);

impl SymbolId {
    pub const NONE: SymbolId = SymbolId(u32::MAX);

    pub fn is_none(&self) -> bool {
        self.0 == u32::MAX
    }
}

/// A symbol represents a named entity in the program.
/// Symbols are created during binding and used during type checking.
#[derive(Clone, Debug, Serialize)]
pub struct Symbol {
    /// Symbol flags describing kind and properties
    pub flags: u32,
    /// Escaped name of the symbol
    pub escaped_name: String,
    /// Declarations associated with this symbol
    pub declarations: Vec<NodeIndex>,
    /// First value declaration of the symbol
    pub value_declaration: NodeIndex,
    /// Parent symbol (for nested symbols)
    pub parent: SymbolId,
    /// Unique ID for this symbol
    pub id: SymbolId,
}

impl Symbol {
    /// Create a new symbol with the given flags and name.
    pub fn new(id: SymbolId, flags: u32, name: String) -> Self {
        Symbol {
            flags,
            escaped_name: name,
            declarations: Vec::new(),
            value_declaration: NodeIndex::NONE,
            parent: SymbolId::NONE,
            id,
        }
    }

    /// Check if symbol has all specified flags.
    pub fn has_flags(&self, flags: u32) -> bool {
        (self.flags & flags) == flags
    }

    /// Check if symbol has any of specified flags.
    pub fn has_any_flags(&self, flags: u32) -> bool {
        (self.flags & flags) != 0
    }
}

// =============================================================================
// Symbol Table
// =============================================================================

/// A symbol table maps names to symbols.
/// Used for scope management and name resolution.
#[derive(Clone, Debug, Default, Serialize)]
pub struct SymbolTable {
    /// Symbols indexed by their escaped name
    symbols: std::collections::HashMap<String, SymbolId>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: std::collections::HashMap::new(),
        }
    }

    /// Get a symbol by name.
    pub fn get(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).copied()
    }

    /// Set a symbol by name.
    pub fn set(&mut self, name: String, symbol: SymbolId) {
        self.symbols.insert(name, symbol);
    }

    /// Check if a name exists in the table.
    pub fn has(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Get number of symbols.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Iterate over symbols.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &SymbolId)> {
        self.symbols.iter()
    }
}

// =============================================================================
// Symbol Arena
// =============================================================================

/// Arena allocator for symbols.
#[derive(Debug, Default, Serialize)]
pub struct SymbolArena {
    symbols: Vec<Symbol>,
}

impl SymbolArena {
    pub fn new() -> Self {
        SymbolArena {
            symbols: Vec::new(),
        }
    }

    /// Allocate a new symbol and return its ID.
    pub fn alloc(&mut self, flags: u32, name: String) -> SymbolId {
        let id = SymbolId(self.symbols.len() as u32);
        self.symbols.push(Symbol::new(id, flags, name));
        id
    }

    /// Get a symbol by ID.
    pub fn get(&self, id: SymbolId) -> Option<&Symbol> {
        if id.is_none() {
            None
        } else {
            self.symbols.get(id.0 as usize)
        }
    }

    /// Get a mutable symbol by ID.
    pub fn get_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        if id.is_none() {
            None
        } else {
            self.symbols.get_mut(id.0 as usize)
        }
    }

    /// Get the number of symbols.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_flags() {
        assert_eq!(symbol_flags::NONE, 0);
        assert_eq!(symbol_flags::FUNCTION_SCOPED_VARIABLE, 1);
        assert_eq!(symbol_flags::BLOCK_SCOPED_VARIABLE, 2);
        assert_eq!(symbol_flags::VARIABLE, 3);
    }

    #[test]
    fn test_symbol_id() {
        let id = SymbolId(42);
        assert_eq!(id.0, 42);
        assert!(!id.is_none());
        assert!(SymbolId::NONE.is_none());
    }

    #[test]
    fn test_symbol() {
        let sym = Symbol::new(
            SymbolId(0),
            symbol_flags::FUNCTION,
            "myFunc".to_string(),
        );
        assert!(sym.has_flags(symbol_flags::FUNCTION));
        assert!(!sym.has_flags(symbol_flags::CLASS));
        assert_eq!(sym.escaped_name, "myFunc");
    }

    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();
        assert!(table.is_empty());

        table.set("x".to_string(), SymbolId(0));
        table.set("y".to_string(), SymbolId(1));

        assert_eq!(table.len(), 2);
        assert!(table.has("x"));
        assert!(!table.has("z"));
        assert_eq!(table.get("x"), Some(SymbolId(0)));
        assert_eq!(table.get("z"), None);
    }

    #[test]
    fn test_symbol_arena() {
        let mut arena = SymbolArena::new();
        assert!(arena.is_empty());

        let id1 = arena.alloc(symbol_flags::VARIABLE, "x".to_string());
        let id2 = arena.alloc(symbol_flags::FUNCTION, "f".to_string());

        assert_eq!(arena.len(), 2);
        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);

        let sym1 = arena.get(id1).unwrap();
        assert_eq!(sym1.escaped_name, "x");
        assert!(sym1.has_flags(symbol_flags::VARIABLE));

        let sym2 = arena.get(id2).unwrap();
        assert_eq!(sym2.escaped_name, "f");
        assert!(sym2.has_flags(symbol_flags::FUNCTION));
    }
}
