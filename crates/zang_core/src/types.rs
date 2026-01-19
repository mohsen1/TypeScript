//! Type representations for TypeScript
//!
//! Provides types for representing TypeScript's type system.

use crate::interner::InternedString;
use crate::symbol::SymbolId;

/// A unique identifier for a type
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TypeId(u32);

impl TypeId {
    /// Creates a new type ID
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw ID value
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Flags describing type properties
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TypeFlags(u32);

impl TypeFlags {
    pub const NONE: Self = Self(0);
    pub const ANY: Self = Self(1 << 0);
    pub const UNKNOWN: Self = Self(1 << 1);
    pub const STRING: Self = Self(1 << 2);
    pub const NUMBER: Self = Self(1 << 3);
    pub const BOOLEAN: Self = Self(1 << 4);
    pub const ENUM: Self = Self(1 << 5);
    pub const BIGINT: Self = Self(1 << 6);
    pub const STRING_LITERAL: Self = Self(1 << 7);
    pub const NUMBER_LITERAL: Self = Self(1 << 8);
    pub const BOOLEAN_LITERAL: Self = Self(1 << 9);
    pub const ENUM_LITERAL: Self = Self(1 << 10);
    pub const BIGINT_LITERAL: Self = Self(1 << 11);
    pub const ES_SYMBOL: Self = Self(1 << 12);
    pub const UNIQUE_ES_SYMBOL: Self = Self(1 << 13);
    pub const VOID: Self = Self(1 << 14);
    pub const UNDEFINED: Self = Self(1 << 15);
    pub const NULL: Self = Self(1 << 16);
    pub const NEVER: Self = Self(1 << 17);
    pub const TYPE_PARAMETER: Self = Self(1 << 18);
    pub const OBJECT: Self = Self(1 << 19);
    pub const UNION: Self = Self(1 << 20);
    pub const INTERSECTION: Self = Self(1 << 21);
    pub const INDEX: Self = Self(1 << 22);
    pub const INDEXED_ACCESS: Self = Self(1 << 23);
    pub const CONDITIONAL: Self = Self(1 << 24);
    pub const SUBSTITUTION: Self = Self(1 << 25);
    pub const NON_PRIMITIVE: Self = Self(1 << 26);
    pub const TEMPLATE_LITERAL: Self = Self(1 << 27);
    pub const STRING_MAPPING: Self = Self(1 << 28);

    /// Checks if this contains the given flags
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Returns the union of two flag sets
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for TypeFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

/// Object type flags
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ObjectFlags(u32);

impl ObjectFlags {
    pub const NONE: Self = Self(0);
    pub const CLASS: Self = Self(1 << 0);
    pub const INTERFACE: Self = Self(1 << 1);
    pub const REFERENCE: Self = Self(1 << 2);
    pub const TUPLE: Self = Self(1 << 3);
    pub const ANONYMOUS: Self = Self(1 << 4);
    pub const MAPPED: Self = Self(1 << 5);
    pub const INSTANTIATED: Self = Self(1 << 6);
    pub const OBJECT_LITERAL: Self = Self(1 << 7);
    pub const EVOLVING_ARRAY: Self = Self(1 << 8);
    pub const OBJECT_LITERAL_PATTERN: Self = Self(1 << 9);
    pub const REVERSE_MAPPED: Self = Self(1 << 10);
    pub const JSX_ATTRIBUTES: Self = Self(1 << 11);
    pub const MARKER: Self = Self(1 << 12);

    /// Checks if this contains the given flags
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// Base type structure
#[derive(Clone, Debug)]
pub struct Type {
    /// Unique identifier
    pub id: TypeId,
    /// Type flags
    pub flags: TypeFlags,
    /// Associated symbol
    pub symbol: Option<SymbolId>,
    /// Alias symbol
    pub alias_symbol: Option<SymbolId>,
    /// Alias type arguments
    pub alias_type_arguments: Option<Vec<TypeId>>,
}

impl Type {
    /// Creates a new type
    pub fn new(id: TypeId, flags: TypeFlags) -> Self {
        Self {
            id,
            flags,
            symbol: None,
            alias_symbol: None,
            alias_type_arguments: None,
        }
    }

    /// Returns true if this is an any type
    pub fn is_any(&self) -> bool {
        self.flags.contains(TypeFlags::ANY)
    }

    /// Returns true if this is an unknown type
    pub fn is_unknown(&self) -> bool {
        self.flags.contains(TypeFlags::UNKNOWN)
    }

    /// Returns true if this is a never type
    pub fn is_never(&self) -> bool {
        self.flags.contains(TypeFlags::NEVER)
    }

    /// Returns true if this is a primitive type
    pub fn is_primitive(&self) -> bool {
        self.flags.contains(TypeFlags::STRING)
            || self.flags.contains(TypeFlags::NUMBER)
            || self.flags.contains(TypeFlags::BOOLEAN)
            || self.flags.contains(TypeFlags::BIGINT)
            || self.flags.contains(TypeFlags::ES_SYMBOL)
            || self.flags.contains(TypeFlags::VOID)
            || self.flags.contains(TypeFlags::UNDEFINED)
            || self.flags.contains(TypeFlags::NULL)
    }

    /// Returns true if this is a literal type
    pub fn is_literal(&self) -> bool {
        self.flags.contains(TypeFlags::STRING_LITERAL)
            || self.flags.contains(TypeFlags::NUMBER_LITERAL)
            || self.flags.contains(TypeFlags::BOOLEAN_LITERAL)
            || self.flags.contains(TypeFlags::BIGINT_LITERAL)
    }

    /// Returns true if this is a union type
    pub fn is_union(&self) -> bool {
        self.flags.contains(TypeFlags::UNION)
    }

    /// Returns true if this is an intersection type
    pub fn is_intersection(&self) -> bool {
        self.flags.contains(TypeFlags::INTERSECTION)
    }

    /// Returns true if this is a type parameter
    pub fn is_type_parameter(&self) -> bool {
        self.flags.contains(TypeFlags::TYPE_PARAMETER)
    }

    /// Returns true if this is an object type
    pub fn is_object(&self) -> bool {
        self.flags.contains(TypeFlags::OBJECT)
    }
}

/// A literal type value
#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
    String(InternedString),
    Number(f64),
    BigInt(String),
    Boolean(bool),
}

/// A type parameter
#[derive(Clone, Debug)]
pub struct TypeParameter {
    /// The base type
    pub base: Type,
    /// Constraint type
    pub constraint: Option<TypeId>,
    /// Default type
    pub default: Option<TypeId>,
}

/// A union type
#[derive(Clone, Debug)]
pub struct UnionType {
    /// The base type
    pub base: Type,
    /// Constituent types
    pub types: Vec<TypeId>,
}

/// An intersection type
#[derive(Clone, Debug)]
pub struct IntersectionType {
    /// The base type
    pub base: Type,
    /// Constituent types
    pub types: Vec<TypeId>,
}

/// An object type
#[derive(Clone, Debug)]
pub struct ObjectType {
    /// The base type
    pub base: Type,
    /// Object flags
    pub object_flags: ObjectFlags,
    /// Properties (symbol IDs)
    pub properties: Vec<SymbolId>,
    /// Call signatures
    pub call_signatures: Vec<SignatureId>,
    /// Construct signatures
    pub construct_signatures: Vec<SignatureId>,
}

/// A unique identifier for a signature
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SignatureId(u32);

impl SignatureId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// A type reference (instantiated generic)
#[derive(Clone, Debug)]
pub struct TypeReference {
    /// The base type
    pub base: Type,
    /// Target generic type
    pub target: TypeId,
    /// Type arguments
    pub type_arguments: Vec<TypeId>,
}

/// A conditional type (T extends U ? X : Y)
#[derive(Clone, Debug)]
pub struct ConditionalType {
    /// The base type
    pub base: Type,
    /// Check type
    pub check_type: TypeId,
    /// Extends type
    pub extends_type: TypeId,
    /// True branch type
    pub true_type: TypeId,
    /// False branch type
    pub false_type: TypeId,
}

/// A mapped type ({ [K in keyof T]: V })
#[derive(Clone, Debug)]
pub struct MappedType {
    /// The base type
    pub base: Type,
    /// Type parameter
    pub type_parameter: TypeId,
    /// Constraint type
    pub constraint_type: TypeId,
    /// Template type
    pub template_type: TypeId,
    /// Modifier (readonly, optional)
    pub modifiers: MappedTypeModifiers,
}

/// Mapped type modifiers
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MappedTypeModifiers(u8);

impl MappedTypeModifiers {
    pub const NONE: Self = Self(0);
    pub const READONLY: Self = Self(1 << 0);
    pub const OPTIONAL: Self = Self(1 << 1);
    pub const INCLUDE_READONLY: Self = Self(1 << 2);
    pub const EXCLUDE_READONLY: Self = Self(1 << 3);
    pub const INCLUDE_OPTIONAL: Self = Self(1 << 4);
    pub const EXCLUDE_OPTIONAL: Self = Self(1 << 5);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_flags() {
        let flags = TypeFlags::STRING | TypeFlags::NULL;
        assert!(flags.contains(TypeFlags::STRING));
        assert!(flags.contains(TypeFlags::NULL));
        assert!(!flags.contains(TypeFlags::NUMBER));
    }

    #[test]
    fn test_type_is_primitive() {
        let string_type = Type::new(TypeId::new(0), TypeFlags::STRING);
        assert!(string_type.is_primitive());

        let union_type = Type::new(TypeId::new(1), TypeFlags::UNION);
        assert!(!union_type.is_primitive());
    }

    #[test]
    fn test_type_is_literal() {
        let literal = Type::new(TypeId::new(0), TypeFlags::STRING_LITERAL);
        assert!(literal.is_literal());

        let primitive = Type::new(TypeId::new(1), TypeFlags::STRING);
        assert!(!primitive.is_literal());
    }
}
