//! Type solver module for TypeScript type checking.
//!
//! This module provides:
//! - Generic instantiation and inference
//! - Conditional type resolution
//! - Mapped type resolution
//! - Built-in utility types

pub mod generics;
pub mod conditional;
pub mod mapped;
pub mod utility;

pub use generics::*;
pub use conditional::*;
pub use mapped::*;
pub use utility::*;

use std::collections::HashMap;

/// A unique identifier for a type.
pub type TypeId = u64;

/// Represents primitive type kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    Number,
    String,
    Boolean,
    BigInt,
    Symbol,
}

/// Represents a type in the type system.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// The any type
    Any,
    /// The unknown type
    Unknown,
    /// The never type
    Never,
    /// The void type
    Void,
    /// The null type
    Null,
    /// The undefined type
    Undefined,
    /// Primitive types
    Primitive(PrimitiveKind),
    /// Literal types
    Literal(LiteralType),
    /// Object types
    Object(ObjectType),
    /// Array types
    Array(Box<Type>),
    /// Tuple types
    Tuple(Vec<Type>),
    /// Function types
    Function(FunctionType),
    /// Union types
    Union(Vec<Type>),
    /// Intersection types
    Intersection(Vec<Type>),
    /// Type parameter (generic)
    TypeParameter(TypeParameter),
    /// Conditional type
    Conditional(Box<ConditionalType>),
    /// Mapped type
    Mapped(Box<MappedType>),
    /// Indexed access type (T[K])
    IndexedAccess(Box<Type>, Box<Type>),
    /// Index type (keyof T)
    Keyof(Box<Type>),
    /// Type reference (named type with optional type arguments)
    TypeRef(String, Vec<Type>),
    /// Template literal type
    TemplateLiteral(Vec<TemplateLiteralSpan>),
    /// Infer type (used in conditional types)
    Infer(String),
}

/// Literal type variants.
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType {
    Number(f64),
    String(String),
    Boolean(bool),
    BigInt(i128),
}

/// Object type representation.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObjectType {
    pub properties: HashMap<String, Property>,
    pub call_signatures: Vec<FunctionType>,
    pub construct_signatures: Vec<FunctionType>,
    pub index_signatures: Vec<IndexSignature>,
}

/// Property on an object type.
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: String,
    pub ty: Type,
    pub optional: bool,
    pub readonly: bool,
}

/// Index signature.
#[derive(Debug, Clone, PartialEq)]
pub struct IndexSignature {
    pub key_type: IndexKeyType,
    pub value_type: Type,
    pub readonly: bool,
}

/// Index key type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexKeyType {
    String,
    Number,
    Symbol,
}

/// Function type representation.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<Parameter>,
    pub return_type: Box<Type>,
    pub rest_parameter: Option<Box<Parameter>>,
}

/// Function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub optional: bool,
}

/// Type parameter (generic).
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParameter {
    pub name: String,
    pub constraint: Option<Box<Type>>,
    pub default: Option<Box<Type>>,
}

/// Template literal span.
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateLiteralSpan {
    Text(String),
    Type(Type),
}

impl Type {
    /// Check if type is any.
    pub fn is_any(&self) -> bool {
        matches!(self, Type::Any)
    }

    /// Check if type is unknown.
    pub fn is_unknown(&self) -> bool {
        matches!(self, Type::Unknown)
    }

    /// Check if type is never.
    pub fn is_never(&self) -> bool {
        matches!(self, Type::Never)
    }

    /// Check if type is void.
    pub fn is_void(&self) -> bool {
        matches!(self, Type::Void)
    }

    /// Check if type is null or undefined.
    pub fn is_nullable(&self) -> bool {
        matches!(self, Type::Null | Type::Undefined)
    }

    /// Check if type is a union.
    pub fn is_union(&self) -> bool {
        matches!(self, Type::Union(_))
    }

    /// Check if type is an intersection.
    pub fn is_intersection(&self) -> bool {
        matches!(self, Type::Intersection(_))
    }

    /// Check if type is an object.
    pub fn is_object(&self) -> bool {
        matches!(self, Type::Object(_))
    }

    /// Check if type is a function.
    pub fn is_function(&self) -> bool {
        matches!(self, Type::Function(_))
    }

    /// Check if type is a type parameter.
    pub fn is_type_parameter(&self) -> bool {
        matches!(self, Type::TypeParameter(_))
    }

    /// Check if type is a conditional type.
    pub fn is_conditional(&self) -> bool {
        matches!(self, Type::Conditional(_))
    }

    /// Check if type is a mapped type.
    pub fn is_mapped(&self) -> bool {
        matches!(self, Type::Mapped(_))
    }

    /// Get the properties of an object type.
    pub fn get_properties(&self) -> Option<&HashMap<String, Property>> {
        match self {
            Type::Object(obj) => Some(&obj.properties),
            _ => None,
        }
    }

    /// Get union members if this is a union type.
    pub fn get_union_members(&self) -> Option<&Vec<Type>> {
        match self {
            Type::Union(members) => Some(members),
            _ => None,
        }
    }

    /// Get intersection members if this is an intersection type.
    pub fn get_intersection_members(&self) -> Option<&Vec<Type>> {
        match self {
            Type::Intersection(members) => Some(members),
            _ => None,
        }
    }

    /// Create a simple number type.
    pub fn number() -> Self {
        Type::Primitive(PrimitiveKind::Number)
    }

    /// Create a simple string type.
    pub fn string() -> Self {
        Type::Primitive(PrimitiveKind::String)
    }

    /// Create a simple boolean type.
    pub fn boolean() -> Self {
        Type::Primitive(PrimitiveKind::Boolean)
    }

    /// Create a union of types.
    pub fn union(types: Vec<Type>) -> Self {
        if types.len() == 1 {
            types.into_iter().next().unwrap()
        } else {
            Type::Union(types)
        }
    }

    /// Create an intersection of types.
    pub fn intersection(types: Vec<Type>) -> Self {
        if types.len() == 1 {
            types.into_iter().next().unwrap()
        } else {
            Type::Intersection(types)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_predicates() {
        assert!(Type::Any.is_any());
        assert!(Type::Unknown.is_unknown());
        assert!(Type::Never.is_never());
        assert!(Type::Void.is_void());
        assert!(Type::Null.is_nullable());
        assert!(Type::Undefined.is_nullable());
    }

    #[test]
    fn test_type_constructors() {
        assert!(matches!(Type::number(), Type::Primitive(PrimitiveKind::Number)));
        assert!(matches!(Type::string(), Type::Primitive(PrimitiveKind::String)));
        assert!(matches!(Type::boolean(), Type::Primitive(PrimitiveKind::Boolean)));
    }

    #[test]
    fn test_union_simplification() {
        // Single type should not be wrapped
        let single = Type::union(vec![Type::number()]);
        assert!(matches!(single, Type::Primitive(PrimitiveKind::Number)));

        // Multiple types should be a union
        let multi = Type::union(vec![Type::number(), Type::string()]);
        assert!(multi.is_union());
    }
}
