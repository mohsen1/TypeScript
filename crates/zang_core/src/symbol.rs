//! Symbol table and symbol types
//!
//! Provides types for representing symbols in the TypeScript program.

use crate::interner::InternedString;
use crate::span::Span;

/// A unique identifier for a symbol
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SymbolId(u32);

impl SymbolId {
    /// Creates a new symbol ID
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw ID value
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Flags describing symbol properties
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SymbolFlags(u32);

impl SymbolFlags {
    pub const NONE: Self = Self(0);
    pub const FUNCTION_SCOPED_VARIABLE: Self = Self(1 << 0);
    pub const BLOCK_SCOPED_VARIABLE: Self = Self(1 << 1);
    pub const PROPERTY: Self = Self(1 << 2);
    pub const ENUM_MEMBER: Self = Self(1 << 3);
    pub const FUNCTION: Self = Self(1 << 4);
    pub const CLASS: Self = Self(1 << 5);
    pub const INTERFACE: Self = Self(1 << 6);
    pub const CONST_ENUM: Self = Self(1 << 7);
    pub const REGULAR_ENUM: Self = Self(1 << 8);
    pub const VALUE_MODULE: Self = Self(1 << 9);
    pub const NAMESPACE_MODULE: Self = Self(1 << 10);
    pub const TYPE_LITERAL: Self = Self(1 << 11);
    pub const OBJECT_LITERAL: Self = Self(1 << 12);
    pub const METHOD: Self = Self(1 << 13);
    pub const CONSTRUCTOR: Self = Self(1 << 14);
    pub const GET_ACCESSOR: Self = Self(1 << 15);
    pub const SET_ACCESSOR: Self = Self(1 << 16);
    pub const SIGNATURE: Self = Self(1 << 17);
    pub const TYPE_PARAMETER: Self = Self(1 << 18);
    pub const TYPE_ALIAS: Self = Self(1 << 19);
    pub const EXPORT_VALUE: Self = Self(1 << 20);
    pub const ALIAS: Self = Self(1 << 21);
    pub const PROTOTYPE: Self = Self(1 << 22);
    pub const EXPORT_STAR: Self = Self(1 << 23);
    pub const OPTIONAL: Self = Self(1 << 24);
    pub const TRANSIENT: Self = Self(1 << 25);

    /// Checks if this contains the given flags
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Returns the union of two flag sets
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns the intersection of two flag sets
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

impl std::ops::BitOr for SymbolFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl std::ops::BitAnd for SymbolFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.intersection(rhs)
    }
}

/// A symbol in the program
#[derive(Clone, Debug)]
pub struct Symbol {
    /// Unique identifier
    pub id: SymbolId,
    /// Symbol name
    pub name: InternedString,
    /// Symbol flags
    pub flags: SymbolFlags,
    /// Declaration spans
    pub declarations: Vec<Span>,
    /// Parent symbol (for nested declarations)
    pub parent: Option<SymbolId>,
    /// Exported symbol (for internal/external pairs)
    pub export_symbol: Option<SymbolId>,
    /// Value declaration span
    pub value_declaration: Option<Span>,
}

impl Symbol {
    /// Creates a new symbol
    pub fn new(id: SymbolId, name: InternedString, flags: SymbolFlags) -> Self {
        Self {
            id,
            name,
            flags,
            declarations: Vec::new(),
            parent: None,
            export_symbol: None,
            value_declaration: None,
        }
    }

    /// Adds a declaration to this symbol
    pub fn add_declaration(&mut self, span: Span) {
        self.declarations.push(span);
        if self.value_declaration.is_none() {
            self.value_declaration = Some(span);
        }
    }

    /// Returns true if this symbol is a variable
    pub fn is_variable(&self) -> bool {
        self.flags.contains(SymbolFlags::FUNCTION_SCOPED_VARIABLE)
            || self.flags.contains(SymbolFlags::BLOCK_SCOPED_VARIABLE)
    }

    /// Returns true if this symbol is a type
    pub fn is_type(&self) -> bool {
        self.flags.contains(SymbolFlags::CLASS)
            || self.flags.contains(SymbolFlags::INTERFACE)
            || self.flags.contains(SymbolFlags::TYPE_ALIAS)
            || self.flags.contains(SymbolFlags::TYPE_PARAMETER)
            || self.flags.contains(SymbolFlags::CONST_ENUM)
            || self.flags.contains(SymbolFlags::REGULAR_ENUM)
    }

    /// Returns true if this symbol is a namespace
    pub fn is_namespace(&self) -> bool {
        self.flags.contains(SymbolFlags::VALUE_MODULE)
            || self.flags.contains(SymbolFlags::NAMESPACE_MODULE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interner::StringInterner;

    #[test]
    fn test_symbol_flags() {
        let flags = SymbolFlags::FUNCTION | SymbolFlags::EXPORT_VALUE;
        assert!(flags.contains(SymbolFlags::FUNCTION));
        assert!(flags.contains(SymbolFlags::EXPORT_VALUE));
        assert!(!flags.contains(SymbolFlags::CLASS));
    }

    #[test]
    fn test_symbol_creation() {
        let interner = StringInterner::new();
        let name = interner.intern("myFunc");
        let symbol = Symbol::new(
            SymbolId::new(0),
            name,
            SymbolFlags::FUNCTION,
        );

        assert_eq!(symbol.id.as_u32(), 0);
        assert!(symbol.flags.contains(SymbolFlags::FUNCTION));
    }

    #[test]
    fn test_symbol_is_type() {
        let interner = StringInterner::new();
        let name = interner.intern("MyClass");

        let class_symbol = Symbol::new(
            SymbolId::new(0),
            name,
            SymbolFlags::CLASS,
        );
        assert!(class_symbol.is_type());

        let func_symbol = Symbol::new(
            SymbolId::new(1),
            name,
            SymbolFlags::FUNCTION,
        );
        assert!(!func_symbol.is_type());
    }
}
