//! Symbol and Symbol Table Implementation
//!
//! This module provides the core symbol representation for the TypeScript binder.

use std::collections::HashMap;

/// Unique identifier for symbols
pub type SymbolId = u32;

/// Symbol flags that describe the kind of declaration
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

    // Compound flags
    pub const ENUM: SymbolFlags = SymbolFlags(Self::REGULAR_ENUM.0 | Self::CONST_ENUM.0);
    pub const VARIABLE: SymbolFlags = SymbolFlags(Self::FUNCTION_SCOPED_VARIABLE.0 | Self::BLOCK_SCOPED_VARIABLE.0);
    pub const VALUE: SymbolFlags = SymbolFlags(
        Self::VARIABLE.0 | Self::PROPERTY.0 | Self::ENUM_MEMBER.0 |
        Self::OBJECT_LITERAL.0 | Self::FUNCTION.0 | Self::CLASS.0 |
        Self::ENUM.0 | Self::VALUE_MODULE.0 | Self::METHOD.0 |
        Self::GET_ACCESSOR.0 | Self::SET_ACCESSOR.0
    );
    pub const TYPE: SymbolFlags = SymbolFlags(
        Self::CLASS.0 | Self::INTERFACE.0 | Self::ENUM.0 |
        Self::TYPE_LITERAL.0 | Self::TYPE_PARAMETER.0 | Self::TYPE_ALIAS.0
    );
    pub const NAMESPACE: SymbolFlags = SymbolFlags(
        Self::VALUE_MODULE.0 | Self::NAMESPACE_MODULE.0 | Self::ENUM.0
    );
    pub const MODULE: SymbolFlags = SymbolFlags(
        Self::VALUE_MODULE.0 | Self::NAMESPACE_MODULE.0
    );
    pub const ACCESSOR: SymbolFlags = SymbolFlags(Self::GET_ACCESSOR.0 | Self::SET_ACCESSOR.0);

    // Exclusion flags - what flags conflict with each other
    pub const FUNCTION_SCOPED_VARIABLE_EXCLUDES: SymbolFlags = SymbolFlags(
        Self::VALUE.0 & !(Self::FUNCTION_SCOPED_VARIABLE.0)
    );
    pub const BLOCK_SCOPED_VARIABLE_EXCLUDES: SymbolFlags = Self::VALUE;
    pub const PARAMETER_EXCLUDES: SymbolFlags = Self::VALUE;
    pub const PROPERTY_EXCLUDES: SymbolFlags = SymbolFlags(0);
    pub const ENUM_MEMBER_EXCLUDES: SymbolFlags = SymbolFlags(Self::VALUE.0 | Self::TYPE.0);
    pub const FUNCTION_EXCLUDES: SymbolFlags = SymbolFlags(Self::VALUE.0 & !(Self::FUNCTION.0));
    pub const CLASS_EXCLUDES: SymbolFlags = SymbolFlags(Self::VALUE.0 | Self::TYPE.0);
    pub const INTERFACE_EXCLUDES: SymbolFlags = SymbolFlags(Self::TYPE.0 & !(Self::INTERFACE.0));
    pub const REGULAR_ENUM_EXCLUDES: SymbolFlags = SymbolFlags(
        (Self::VALUE.0 | Self::TYPE.0) & !(Self::REGULAR_ENUM.0 | Self::VALUE_MODULE.0)
    );
    pub const CONST_ENUM_EXCLUDES: SymbolFlags = SymbolFlags(Self::VALUE.0 | Self::TYPE.0);
    pub const TYPE_ALIAS_EXCLUDES: SymbolFlags = Self::TYPE;
    pub const ALIAS_EXCLUDES: SymbolFlags = Self::ALIAS;

    pub fn contains(&self, other: SymbolFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn intersects(&self, other: SymbolFlags) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
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

/// A symbol represents a named declaration in the program
#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub flags: SymbolFlags,
    /// Declarations that contribute to this symbol
    pub declarations: Vec<DeclarationId>,
    /// The value declaration (for values)
    pub value_declaration: Option<DeclarationId>,
    /// Members of this symbol (for classes, interfaces, enums, etc.)
    pub members: Option<SymbolTable>,
    /// Exports of this symbol (for modules)
    pub exports: Option<SymbolTable>,
    /// The parent symbol
    pub parent: Option<SymbolId>,
}

impl Symbol {
    pub fn new(id: SymbolId, name: impl Into<String>, flags: SymbolFlags) -> Self {
        Symbol {
            id,
            name: name.into(),
            flags,
            declarations: Vec::new(),
            value_declaration: None,
            members: None,
            exports: None,
            parent: None,
        }
    }

    pub fn add_declaration(&mut self, decl: DeclarationId) {
        if self.value_declaration.is_none() && self.flags.intersects(SymbolFlags::VALUE) {
            self.value_declaration = Some(decl);
        }
        self.declarations.push(decl);
    }

    pub fn get_members(&mut self) -> &mut SymbolTable {
        self.members.get_or_insert_with(SymbolTable::new)
    }

    pub fn get_exports(&mut self) -> &mut SymbolTable {
        self.exports.get_or_insert_with(SymbolTable::new)
    }
}

/// Reference to a declaration node
pub type DeclarationId = u32;

/// A symbol table maps names to symbols
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    symbols: HashMap<String, SymbolId>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).copied()
    }

    pub fn set(&mut self, name: impl Into<String>, symbol_id: SymbolId) {
        self.symbols.insert(name.into(), symbol_id);
    }

    pub fn has(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &SymbolId)> {
        self.symbols.iter()
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}

/// Arena-based storage for symbols
#[derive(Debug, Default)]
pub struct SymbolArena {
    symbols: Vec<Symbol>,
    next_id: SymbolId,
}

impl SymbolArena {
    pub fn new() -> Self {
        SymbolArena {
            symbols: Vec::new(),
            next_id: 1, // 0 is reserved for "no symbol"
        }
    }

    pub fn create_symbol(&mut self, name: impl Into<String>, flags: SymbolFlags) -> SymbolId {
        let id = self.next_id;
        self.next_id += 1;
        let symbol = Symbol::new(id, name, flags);
        self.symbols.push(symbol);
        id
    }

    pub fn get(&self, id: SymbolId) -> Option<&Symbol> {
        if id == 0 || id as usize > self.symbols.len() {
            None
        } else {
            Some(&self.symbols[(id - 1) as usize])
        }
    }

    pub fn get_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        if id == 0 || id as usize > self.symbols.len() {
            None
        } else {
            Some(&mut self.symbols[(id - 1) as usize])
        }
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
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
    }

    #[test]
    fn test_symbol_flags_compound() {
        assert!(SymbolFlags::VALUE.contains(SymbolFlags::FUNCTION));
        assert!(SymbolFlags::VALUE.contains(SymbolFlags::CLASS));
        assert!(SymbolFlags::TYPE.contains(SymbolFlags::INTERFACE));
    }

    #[test]
    fn test_symbol_table() {
        let mut table = SymbolTable::new();
        table.set("foo", 1);
        table.set("bar", 2);

        assert_eq!(table.get("foo"), Some(1));
        assert_eq!(table.get("bar"), Some(2));
        assert_eq!(table.get("baz"), None);
        assert!(table.has("foo"));
        assert!(!table.has("baz"));
    }

    #[test]
    fn test_symbol_arena() {
        let mut arena = SymbolArena::new();
        let id1 = arena.create_symbol("foo", SymbolFlags::FUNCTION);
        let id2 = arena.create_symbol("bar", SymbolFlags::CLASS);

        assert_eq!(arena.get(id1).unwrap().name, "foo");
        assert_eq!(arena.get(id2).unwrap().name, "bar");
        assert!(arena.get(id1).unwrap().flags.contains(SymbolFlags::FUNCTION));
    }

    #[test]
    fn test_symbol_declarations() {
        let mut arena = SymbolArena::new();
        let id = arena.create_symbol("test", SymbolFlags::FUNCTION);

        let symbol = arena.get_mut(id).unwrap();
        symbol.add_declaration(100);
        symbol.add_declaration(101);

        let symbol = arena.get(id).unwrap();
        assert_eq!(symbol.declarations.len(), 2);
        assert_eq!(symbol.value_declaration, Some(100));
    }
}
