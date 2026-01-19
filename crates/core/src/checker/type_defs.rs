//! Core type definitions for the TypeScript type checker.
//!
//! This module defines the Type enum and related structures used
//! throughout the type checker.

use std::collections::HashMap;
use std::sync::Arc;

/// Unique identifier for a type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

/// Unique identifier for a symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

/// Type flags indicating the kind of type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeFlags(pub u32);

impl TypeFlags {
    pub const ANY: TypeFlags = TypeFlags(1 << 0);
    pub const UNKNOWN: TypeFlags = TypeFlags(1 << 1);
    pub const STRING: TypeFlags = TypeFlags(1 << 2);
    pub const NUMBER: TypeFlags = TypeFlags(1 << 3);
    pub const BOOLEAN: TypeFlags = TypeFlags(1 << 4);
    pub const ENUM: TypeFlags = TypeFlags(1 << 5);
    pub const BIGINT: TypeFlags = TypeFlags(1 << 6);
    pub const STRING_LITERAL: TypeFlags = TypeFlags(1 << 7);
    pub const NUMBER_LITERAL: TypeFlags = TypeFlags(1 << 8);
    pub const BOOLEAN_LITERAL: TypeFlags = TypeFlags(1 << 9);
    pub const ENUM_LITERAL: TypeFlags = TypeFlags(1 << 10);
    pub const BIGINT_LITERAL: TypeFlags = TypeFlags(1 << 11);
    pub const SYMBOL: TypeFlags = TypeFlags(1 << 12);
    pub const VOID: TypeFlags = TypeFlags(1 << 13);
    pub const UNDEFINED: TypeFlags = TypeFlags(1 << 14);
    pub const NULL: TypeFlags = TypeFlags(1 << 15);
    pub const NEVER: TypeFlags = TypeFlags(1 << 16);
    pub const OBJECT: TypeFlags = TypeFlags(1 << 17);
    pub const UNION: TypeFlags = TypeFlags(1 << 18);
    pub const INTERSECTION: TypeFlags = TypeFlags(1 << 19);
    pub const INDEX: TypeFlags = TypeFlags(1 << 20);
    pub const INDEXED_ACCESS: TypeFlags = TypeFlags(1 << 21);
    pub const CONDITIONAL: TypeFlags = TypeFlags(1 << 22);
    pub const SUBSTITUTION: TypeFlags = TypeFlags(1 << 23);
    pub const TYPE_PARAMETER: TypeFlags = TypeFlags(1 << 24);

    pub fn contains(&self, other: TypeFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn intersects(&self, other: TypeFlags) -> bool {
        (self.0 & other.0) != 0
    }
}

/// Represents a TypeScript type
#[derive(Debug, Clone)]
pub enum Type {
    /// Primitive types
    Any,
    Unknown,
    String,
    Number,
    Boolean,
    BigInt,
    Symbol,
    Void,
    Undefined,
    Null,
    Never,

    /// Literal types
    StringLiteral(String),
    NumberLiteral(f64),
    BooleanLiteral(bool),
    BigIntLiteral(i64),

    /// Object types
    Object(ObjectType),

    /// Union type (T | U)
    Union(Vec<Arc<Type>>),

    /// Intersection type (T & U)
    Intersection(Vec<Arc<Type>>),

    /// Type parameter
    TypeParameter(TypeParameter),

    /// Conditional type (T extends U ? X : Y)
    Conditional(ConditionalType),

    /// Indexed access type (T[K])
    IndexedAccess(IndexedAccessType),

    /// Index type (keyof T)
    Index(Arc<Type>),

    /// Tuple type
    Tuple(TupleType),

    /// Array type
    Array(Arc<Type>),

    /// Function type
    Function(FunctionType),

    /// Class type
    Class(ClassType),

    /// Enum type
    Enum(EnumType),

    /// Unique symbol type
    UniqueSymbol(SymbolId),
}

impl Type {
    /// Get the type flags for this type
    pub fn flags(&self) -> TypeFlags {
        match self {
            Type::Any => TypeFlags::ANY,
            Type::Unknown => TypeFlags::UNKNOWN,
            Type::String => TypeFlags::STRING,
            Type::Number => TypeFlags::NUMBER,
            Type::Boolean => TypeFlags::BOOLEAN,
            Type::BigInt => TypeFlags::BIGINT,
            Type::Symbol => TypeFlags::SYMBOL,
            Type::Void => TypeFlags::VOID,
            Type::Undefined => TypeFlags::UNDEFINED,
            Type::Null => TypeFlags::NULL,
            Type::Never => TypeFlags::NEVER,
            Type::StringLiteral(_) => TypeFlags::STRING_LITERAL,
            Type::NumberLiteral(_) => TypeFlags::NUMBER_LITERAL,
            Type::BooleanLiteral(_) => TypeFlags::BOOLEAN_LITERAL,
            Type::BigIntLiteral(_) => TypeFlags::BIGINT_LITERAL,
            Type::Object(_) | Type::Array(_) | Type::Tuple(_) | Type::Function(_) | Type::Class(_) => {
                TypeFlags::OBJECT
            }
            Type::Union(_) => TypeFlags::UNION,
            Type::Intersection(_) => TypeFlags::INTERSECTION,
            Type::TypeParameter(_) => TypeFlags::TYPE_PARAMETER,
            Type::Conditional(_) => TypeFlags::CONDITIONAL,
            Type::IndexedAccess(_) => TypeFlags::INDEXED_ACCESS,
            Type::Index(_) => TypeFlags::INDEX,
            Type::Enum(_) => TypeFlags::ENUM,
            Type::UniqueSymbol(_) => TypeFlags::SYMBOL,
        }
    }

    /// Check if this type is falsy (can be false in a boolean context)
    pub fn is_falsy(&self) -> bool {
        match self {
            Type::Undefined | Type::Null | Type::Void => true,
            Type::BooleanLiteral(false) => true,
            Type::NumberLiteral(n) => *n == 0.0,
            Type::StringLiteral(s) => s.is_empty(),
            Type::BigIntLiteral(0) => true,
            _ => false,
        }
    }

    /// Check if this type is definitely truthy
    pub fn is_definitely_truthy(&self) -> bool {
        match self {
            Type::BooleanLiteral(true) => true,
            Type::NumberLiteral(n) => *n != 0.0 && !n.is_nan(),
            Type::StringLiteral(s) => !s.is_empty(),
            Type::BigIntLiteral(n) => *n != 0,
            Type::Object(_) | Type::Function(_) | Type::Class(_) | Type::Array(_) | Type::Tuple(_) => true,
            _ => false,
        }
    }
}

/// Object type with properties and methods
#[derive(Debug, Clone)]
pub struct ObjectType {
    pub properties: HashMap<String, PropertySignature>,
    pub call_signatures: Vec<CallSignature>,
    pub construct_signatures: Vec<CallSignature>,
    pub index_signatures: Vec<IndexSignature>,
}

impl Default for ObjectType {
    fn default() -> Self {
        Self {
            properties: HashMap::new(),
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            index_signatures: Vec::new(),
        }
    }
}

/// Property signature in an object type
#[derive(Debug, Clone)]
pub struct PropertySignature {
    pub name: String,
    pub type_: Arc<Type>,
    pub optional: bool,
    pub readonly: bool,
}

/// Call signature for functions
#[derive(Debug, Clone)]
pub struct CallSignature {
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<ParameterSignature>,
    pub return_type: Arc<Type>,
}

/// Parameter signature
#[derive(Debug, Clone)]
pub struct ParameterSignature {
    pub name: String,
    pub type_: Arc<Type>,
    pub optional: bool,
    pub rest: bool,
}

/// Index signature (e.g., [key: string]: T)
#[derive(Debug, Clone)]
pub struct IndexSignature {
    pub key_type: Arc<Type>,
    pub value_type: Arc<Type>,
    pub readonly: bool,
}

/// Type parameter (generic)
#[derive(Debug, Clone)]
pub struct TypeParameter {
    pub name: String,
    pub constraint: Option<Arc<Type>>,
    pub default: Option<Arc<Type>>,
}

/// Conditional type (T extends U ? X : Y)
#[derive(Debug, Clone)]
pub struct ConditionalType {
    pub check_type: Arc<Type>,
    pub extends_type: Arc<Type>,
    pub true_type: Arc<Type>,
    pub false_type: Arc<Type>,
}

/// Indexed access type (T[K])
#[derive(Debug, Clone)]
pub struct IndexedAccessType {
    pub object_type: Arc<Type>,
    pub index_type: Arc<Type>,
}

/// Tuple type with element types
#[derive(Debug, Clone)]
pub struct TupleType {
    pub element_types: Vec<TupleElement>,
    pub min_length: usize,
    pub has_rest: bool,
}

/// Tuple element
#[derive(Debug, Clone)]
pub struct TupleElement {
    pub type_: Arc<Type>,
    pub optional: bool,
    pub label: Option<String>,
}

/// Function type
#[derive(Debug, Clone)]
pub struct FunctionType {
    pub signatures: Vec<CallSignature>,
}

/// Class type
#[derive(Debug, Clone)]
pub struct ClassType {
    pub name: String,
    pub type_parameters: Vec<TypeParameter>,
    pub extends: Option<Arc<Type>>,
    pub implements: Vec<Arc<Type>>,
    pub members: HashMap<String, PropertySignature>,
    pub static_members: HashMap<String, PropertySignature>,
}

/// Enum type
#[derive(Debug, Clone)]
pub struct EnumType {
    pub name: String,
    pub members: Vec<EnumMember>,
}

/// Enum member
#[derive(Debug, Clone)]
pub struct EnumMember {
    pub name: String,
    pub value: EnumValue,
}

/// Enum value
#[derive(Debug, Clone)]
pub enum EnumValue {
    Number(f64),
    String(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_flags() {
        let t = Type::String;
        assert!(t.flags().contains(TypeFlags::STRING));
        assert!(!t.flags().contains(TypeFlags::NUMBER));
    }

    #[test]
    fn test_type_falsy() {
        assert!(Type::Undefined.is_falsy());
        assert!(Type::Null.is_falsy());
        assert!(Type::BooleanLiteral(false).is_falsy());
        assert!(Type::NumberLiteral(0.0).is_falsy());
        assert!(Type::StringLiteral("".to_string()).is_falsy());
        assert!(!Type::BooleanLiteral(true).is_falsy());
    }

    #[test]
    fn test_type_truthy() {
        assert!(Type::BooleanLiteral(true).is_definitely_truthy());
        assert!(Type::NumberLiteral(1.0).is_definitely_truthy());
        assert!(Type::StringLiteral("hello".to_string()).is_definitely_truthy());
        assert!(!Type::Undefined.is_definitely_truthy());
    }
}
