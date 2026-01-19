//! Symbol definitions and symbol table implementation
//!
//! This module defines the Symbol struct and SymbolFlags, as well as the
//! SymbolTable type used for storing symbols in a scope.

use std::collections::HashMap;

use crate::arena::ArenaId;

/// Unique identifier for a symbol within the binder
pub type SymbolId = ArenaId;

/// Unique identifier for a node in the AST
pub type NodeId = u32;

/// Symbol flags indicating the nature and properties of a symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SymbolFlags(u32);

impl SymbolFlags {
    pub const NONE: SymbolFlags = SymbolFlags(0);
    pub const FUNCTION_SCOPED_VARIABLE: SymbolFlags = SymbolFlags(1 << 0);
    pub const BLOCK_SCOPED_VARIABLE: SymbolFlags = SymbolFlags(1 << 1);
    pub const PROPERTY: SymbolFlags = SymbolFlags(1 << 2);
    pub const ENUM_MEMBER: SymbolFlags = SymbolFlags(1 << 3);
    pub const FUNCTION: SymbolFlags = SymbolFlags(1 << 4);
    pub const CLASS: SymbolFlags = SymbolFlags(1 << 5);
    pub const INTERFACE: SymbolFlags = SymbolFlags(1 << 6);
    pub const CONST_ENUM: SymbolFlags = SymbolFlags(1 << 7);
    pub const REGULAR_ENUM: SymbolFlags = SymbolFlags(1 << 8);
    pub const VALUE_MODULE: SymbolFlags = SymbolFlags(1 << 9);
    pub const NAMESPACE_MODULE: SymbolFlags = SymbolFlags(1 << 10);
    pub const TYPE_LITERAL: SymbolFlags = SymbolFlags(1 << 11);
    pub const OBJECT_LITERAL: SymbolFlags = SymbolFlags(1 << 12);
    pub const METHOD: SymbolFlags = SymbolFlags(1 << 13);
    pub const CONSTRUCTOR: SymbolFlags = SymbolFlags(1 << 14);
    pub const GET_ACCESSOR: SymbolFlags = SymbolFlags(1 << 15);
    pub const SET_ACCESSOR: SymbolFlags = SymbolFlags(1 << 16);
    pub const SIGNATURE: SymbolFlags = SymbolFlags(1 << 17);
    pub const TYPE_PARAMETER: SymbolFlags = SymbolFlags(1 << 18);
    pub const TYPE_ALIAS: SymbolFlags = SymbolFlags(1 << 19);
    pub const EXPORT_VALUE: SymbolFlags = SymbolFlags(1 << 20);
    pub const ALIAS: SymbolFlags = SymbolFlags(1 << 21);
    pub const PROTOTYPE: SymbolFlags = SymbolFlags(1 << 22);
    pub const EXPORT_STAR: SymbolFlags = SymbolFlags(1 << 23);
    pub const OPTIONAL: SymbolFlags = SymbolFlags(1 << 24);
    pub const TRANSIENT: SymbolFlags = SymbolFlags(1 << 25);
    pub const ASSIGNMENT: SymbolFlags = SymbolFlags(1 << 26);
    pub const MODULE_EXPORTS: SymbolFlags = SymbolFlags(1 << 27);

    // Combined flags
    pub const ENUM: SymbolFlags = SymbolFlags(Self::REGULAR_ENUM.0 | Self::CONST_ENUM.0);
    pub const VARIABLE: SymbolFlags =
        SymbolFlags(Self::FUNCTION_SCOPED_VARIABLE.0 | Self::BLOCK_SCOPED_VARIABLE.0);
    pub const VALUE: SymbolFlags = SymbolFlags(
        Self::VARIABLE.0
            | Self::PROPERTY.0
            | Self::ENUM_MEMBER.0
            | Self::OBJECT_LITERAL.0
            | Self::FUNCTION.0
            | Self::CLASS.0
            | Self::ENUM.0
            | Self::VALUE_MODULE.0
            | Self::METHOD.0
            | Self::GET_ACCESSOR.0
            | Self::SET_ACCESSOR.0,
    );
    pub const TYPE: SymbolFlags = SymbolFlags(
        Self::CLASS.0
            | Self::INTERFACE.0
            | Self::ENUM.0
            | Self::ENUM_MEMBER.0
            | Self::TYPE_LITERAL.0
            | Self::TYPE_PARAMETER.0
            | Self::TYPE_ALIAS.0,
    );
    pub const NAMESPACE: SymbolFlags =
        SymbolFlags(Self::VALUE_MODULE.0 | Self::NAMESPACE_MODULE.0 | Self::ENUM.0);
    pub const MODULE: SymbolFlags = SymbolFlags(Self::VALUE_MODULE.0 | Self::NAMESPACE_MODULE.0);
    pub const ACCESSOR: SymbolFlags = SymbolFlags(Self::GET_ACCESSOR.0 | Self::SET_ACCESSOR.0);

    // Exclusion flags (what cannot coexist)
    pub const FUNCTION_SCOPED_VARIABLE_EXCLUDES: SymbolFlags = SymbolFlags(
        Self::VALUE.0 & !Self::FUNCTION_SCOPED_VARIABLE.0,
    );
    pub const BLOCK_SCOPED_VARIABLE_EXCLUDES: SymbolFlags = Self::VALUE;
    pub const PARAMETER_EXCLUDES: SymbolFlags = Self::VALUE;
    pub const PROPERTY_EXCLUDES: SymbolFlags = Self::NONE;
    pub const ENUM_MEMBER_EXCLUDES: SymbolFlags = SymbolFlags(Self::VALUE.0 | Self::TYPE.0);
    pub const FUNCTION_EXCLUDES: SymbolFlags = SymbolFlags(
        Self::VALUE.0 & !(Self::FUNCTION.0 | Self::VALUE_MODULE.0 | Self::CLASS.0),
    );
    pub const CLASS_EXCLUDES: SymbolFlags = SymbolFlags(
        (Self::VALUE.0 | Self::TYPE.0) & !(Self::VALUE_MODULE.0 | Self::INTERFACE.0 | Self::FUNCTION.0),
    );
    pub const INTERFACE_EXCLUDES: SymbolFlags = SymbolFlags(
        Self::TYPE.0 & !(Self::INTERFACE.0 | Self::CLASS.0),
    );
    pub const REGULAR_ENUM_EXCLUDES: SymbolFlags = SymbolFlags(
        (Self::VALUE.0 | Self::TYPE.0) & !(Self::REGULAR_ENUM.0 | Self::VALUE_MODULE.0),
    );
    pub const CONST_ENUM_EXCLUDES: SymbolFlags = SymbolFlags(
        (Self::VALUE.0 | Self::TYPE.0) & !Self::CONST_ENUM.0,
    );
    pub const VALUE_MODULE_EXCLUDES: SymbolFlags = SymbolFlags(
        Self::VALUE.0 & !(Self::FUNCTION.0 | Self::CLASS.0 | Self::REGULAR_ENUM.0 | Self::VALUE_MODULE.0),
    );
    pub const NAMESPACE_MODULE_EXCLUDES: SymbolFlags = Self::NONE;
    pub const METHOD_EXCLUDES: SymbolFlags = SymbolFlags(Self::VALUE.0 & !Self::METHOD.0);
    pub const GET_ACCESSOR_EXCLUDES: SymbolFlags = SymbolFlags(Self::VALUE.0 & !Self::SET_ACCESSOR.0);
    pub const SET_ACCESSOR_EXCLUDES: SymbolFlags = SymbolFlags(Self::VALUE.0 & !Self::GET_ACCESSOR.0);
    pub const TYPE_PARAMETER_EXCLUDES: SymbolFlags = SymbolFlags(Self::TYPE.0 & !Self::TYPE_PARAMETER.0);
    pub const TYPE_ALIAS_EXCLUDES: SymbolFlags = Self::TYPE;
    pub const ALIAS_EXCLUDES: SymbolFlags = Self::ALIAS;

    // Other combined flags
    pub const MODULE_MEMBER: SymbolFlags = SymbolFlags(
        Self::VARIABLE.0
            | Self::FUNCTION.0
            | Self::CLASS.0
            | Self::INTERFACE.0
            | Self::ENUM.0
            | Self::MODULE.0
            | Self::TYPE_ALIAS.0
            | Self::ALIAS.0,
    );
    pub const EXPORT_HAS_LOCAL: SymbolFlags = SymbolFlags(
        Self::FUNCTION.0 | Self::CLASS.0 | Self::ENUM.0 | Self::VALUE_MODULE.0,
    );
    pub const BLOCK_SCOPED: SymbolFlags = SymbolFlags(
        Self::BLOCK_SCOPED_VARIABLE.0 | Self::CLASS.0 | Self::ENUM.0,
    );
    pub const PROPERTY_OR_ACCESSOR: SymbolFlags = SymbolFlags(Self::PROPERTY.0 | Self::ACCESSOR.0);
    pub const CLASS_MEMBER: SymbolFlags = SymbolFlags(
        Self::METHOD.0 | Self::ACCESSOR.0 | Self::PROPERTY.0,
    );
    pub const CLASSIFIABLE: SymbolFlags = SymbolFlags(
        Self::CLASS.0
            | Self::ENUM.0
            | Self::TYPE_ALIAS.0
            | Self::INTERFACE.0
            | Self::TYPE_PARAMETER.0
            | Self::MODULE.0
            | Self::ALIAS.0,
    );

    /// Check if the flags contain a specific flag
    pub fn contains(&self, other: SymbolFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Check if any of the flags in other are set
    pub fn intersects(&self, other: SymbolFlags) -> bool {
        (self.0 & other.0) != 0
    }

    /// Combine two flag sets
    pub fn union(self, other: SymbolFlags) -> SymbolFlags {
        SymbolFlags(self.0 | other.0)
    }

    /// Get the intersection of two flag sets
    pub fn intersection(self, other: SymbolFlags) -> SymbolFlags {
        SymbolFlags(self.0 & other.0)
    }

    /// Remove flags
    pub fn difference(self, other: SymbolFlags) -> SymbolFlags {
        SymbolFlags(self.0 & !other.0)
    }

    /// Get the raw value
    pub fn bits(&self) -> u32 {
        self.0
    }
}

impl std::ops::BitOr for SymbolFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        SymbolFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SymbolFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for SymbolFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        SymbolFlags(self.0 & rhs.0)
    }
}

impl std::ops::Not for SymbolFlags {
    type Output = Self;
    fn not(self) -> Self::Output {
        SymbolFlags(!self.0)
    }
}

/// Represents a symbol in the TypeScript program
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol flags indicating the kind and properties
    pub flags: SymbolFlags,
    /// The escaped name of the symbol
    pub escaped_name: String,
    /// AST node IDs for declarations associated with this symbol
    pub declarations: Vec<NodeId>,
    /// The first value declaration
    pub value_declaration: Option<NodeId>,
    /// Class, interface, or object literal members
    pub members: Option<SymbolTable>,
    /// Module exports
    pub exports: Option<SymbolTable>,
    /// Parent symbol
    pub parent: Option<SymbolId>,
    /// Export symbol associated with this symbol
    pub export_symbol: Option<SymbolId>,
    /// Whether the module contains only const enums
    pub const_enum_only_module: bool,
    /// Whether the symbol has been referenced
    pub is_referenced: SymbolFlags,
    /// Whether this can be replaced by a method
    pub is_replaceable_by_method: bool,
    /// Whether the symbol is a parameter with assignments
    pub is_assigned: bool,
}

impl Symbol {
    /// Create a new symbol with the given flags and name
    pub fn new(flags: SymbolFlags, name: impl Into<String>) -> Self {
        Symbol {
            flags,
            escaped_name: name.into(),
            declarations: Vec::new(),
            value_declaration: None,
            members: None,
            exports: None,
            parent: None,
            export_symbol: None,
            const_enum_only_module: false,
            is_referenced: SymbolFlags::NONE,
            is_replaceable_by_method: false,
            is_assigned: false,
        }
    }

    /// Add a declaration to this symbol
    pub fn add_declaration(&mut self, node_id: NodeId) {
        if !self.declarations.contains(&node_id) {
            self.declarations.push(node_id);
        }
    }

    /// Set the value declaration if not already set
    pub fn set_value_declaration(&mut self, node_id: NodeId) {
        if self.value_declaration.is_none() {
            self.value_declaration = Some(node_id);
        }
    }

    /// Ensure the members table exists
    pub fn ensure_members(&mut self) -> &mut SymbolTable {
        self.members.get_or_insert_with(SymbolTable::new)
    }

    /// Ensure the exports table exists
    pub fn ensure_exports(&mut self) -> &mut SymbolTable {
        self.exports.get_or_insert_with(SymbolTable::new)
    }
}

/// Internal symbol names used by the binder
pub mod internal_symbol_names {
    pub const CALL: &str = "__call";
    pub const CONSTRUCTOR: &str = "__constructor";
    pub const NEW: &str = "__new";
    pub const INDEX: &str = "__index";
    pub const EXPORT_STAR: &str = "__export";
    pub const GLOBAL: &str = "__global";
    pub const MISSING: &str = "__missing";
    pub const TYPE: &str = "__type";
    pub const OBJECT: &str = "__object";
    pub const JSX_ATTRIBUTES: &str = "__jsxAttributes";
    pub const CLASS: &str = "__class";
    pub const FUNCTION: &str = "__function";
    pub const COMPUTED: &str = "__computed";
    pub const RESOLVING: &str = "__resolving__";
    pub const EXPORT_EQUALS: &str = "export=";
    pub const DEFAULT: &str = "default";
    pub const THIS: &str = "this";
}

/// A symbol table mapping names to symbol IDs
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    symbols: HashMap<String, SymbolId>,
}

impl SymbolTable {
    /// Create a new empty symbol table
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
        }
    }

    /// Get a symbol by name
    pub fn get(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).copied()
    }

    /// Insert a symbol into the table
    pub fn set(&mut self, name: impl Into<String>, symbol_id: SymbolId) {
        self.symbols.insert(name.into(), symbol_id);
    }

    /// Check if the table contains a symbol with the given name
    pub fn has(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    /// Remove a symbol from the table
    pub fn delete(&mut self, name: &str) -> Option<SymbolId> {
        self.symbols.remove(name)
    }

    /// Get the number of symbols in the table
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Iterate over all symbols in the table
    pub fn iter(&self) -> impl Iterator<Item = (&String, &SymbolId)> {
        self.symbols.iter()
    }

    /// Get all symbol names
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.symbols.keys()
    }

    /// Get all symbol IDs
    pub fn values(&self) -> impl Iterator<Item = &SymbolId> {
        self.symbols.values()
    }

    /// Merge another symbol table into this one
    /// Note: This does not handle symbol merging, just overwrites
    pub fn merge(&mut self, other: &SymbolTable) {
        for (name, id) in other.iter() {
            self.set(name.clone(), *id);
        }
    }

    /// Clear all symbols from the table
    pub fn clear(&mut self) {
        self.symbols.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_flags() {
        let flags = SymbolFlags::FUNCTION | SymbolFlags::EXPORT_VALUE;
        assert!(flags.contains(SymbolFlags::FUNCTION));
        assert!(flags.contains(SymbolFlags::EXPORT_VALUE));
        assert!(!flags.contains(SymbolFlags::CLASS));
        assert!(flags.intersects(SymbolFlags::VALUE));
    }

    #[test]
    fn test_symbol_creation() {
        let sym = Symbol::new(SymbolFlags::FUNCTION, "myFunction");
        assert_eq!(sym.escaped_name, "myFunction");
        assert!(sym.flags.contains(SymbolFlags::FUNCTION));
        assert!(sym.declarations.is_empty());
    }

    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();
        let id = ArenaId::null();

        table.set("foo", id);
        assert!(table.has("foo"));
        assert!(!table.has("bar"));
        assert_eq!(table.get("foo"), Some(id));
        assert_eq!(table.len(), 1);

        table.delete("foo");
        assert!(!table.has("foo"));
        assert!(table.is_empty());
    }

    #[test]
    fn test_exclusion_flags() {
        // Block-scoped variables cannot coexist with any value
        let _block_scoped = SymbolFlags::BLOCK_SCOPED_VARIABLE;
        let excludes = SymbolFlags::BLOCK_SCOPED_VARIABLE_EXCLUDES;

        // Should exclude function
        assert!(excludes.intersects(SymbolFlags::FUNCTION));

        // Functions can coexist with other functions (through merging)
        let fn_excludes = SymbolFlags::FUNCTION_EXCLUDES;
        assert!(!fn_excludes.intersects(SymbolFlags::FUNCTION));
    }
}
