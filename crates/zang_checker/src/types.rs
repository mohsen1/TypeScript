//! Checker Types
//!
//! Type representations used during type checking.

use zang_core::InternedString;

/// Type ID for caching
pub type TypeId = u64;

/// A resolved type
#[derive(Debug, Clone)]
pub enum ResolvedType {
    /// Any type
    Any,
    /// Unknown type
    Unknown,
    /// Never type
    Never,
    /// Void type
    Void,
    /// Undefined type
    Undefined,
    /// Null type
    Null,
    /// String type
    String,
    /// Number type
    Number,
    /// Boolean type
    Boolean,
    /// BigInt type
    BigInt,
    /// Symbol type
    Symbol,
    /// Object type
    Object(ObjectType),
    /// Array type
    Array(Box<ResolvedType>),
    /// Tuple type
    Tuple(Vec<ResolvedType>),
    /// Union type
    Union(Vec<ResolvedType>),
    /// Intersection type
    Intersection(Vec<ResolvedType>),
    /// Function type
    Function(FunctionType),
    /// Type parameter
    TypeParameter(TypeParameterType),
    /// Type reference (e.g., to a class or interface)
    Reference(TypeReference),
    /// Literal type (string, number, boolean literal)
    Literal(LiteralType),
    /// Conditional type
    Conditional(Box<ConditionalType>),
    /// Mapped type
    Mapped(Box<MappedType>),
    /// Index type (keyof T)
    Index(Box<ResolvedType>),
    /// Indexed access type (T[K])
    IndexedAccess(Box<IndexedAccessType>),
}

/// Object type
#[derive(Debug, Clone)]
pub struct ObjectType {
    /// Properties of the object
    pub properties: Vec<PropertySignature>,
    /// Call signatures
    pub call_signatures: Vec<CallSignature>,
    /// Construct signatures
    pub construct_signatures: Vec<ConstructSignature>,
    /// Index signatures
    pub index_signatures: Vec<IndexSignature>,
}

/// Property signature
#[derive(Debug, Clone)]
pub struct PropertySignature {
    /// Property name
    pub name: InternedString,
    /// Property type
    pub ty: Box<ResolvedType>,
    /// Whether the property is optional
    pub optional: bool,
    /// Whether the property is readonly
    pub readonly: bool,
}

/// Call signature
#[derive(Debug, Clone)]
pub struct CallSignature {
    /// Type parameters
    pub type_parameters: Vec<TypeParameterType>,
    /// Parameters
    pub parameters: Vec<ParameterType>,
    /// Return type
    pub return_type: Box<ResolvedType>,
}

/// Construct signature
#[derive(Debug, Clone)]
pub struct ConstructSignature {
    /// Type parameters
    pub type_parameters: Vec<TypeParameterType>,
    /// Parameters
    pub parameters: Vec<ParameterType>,
    /// Return type
    pub return_type: Box<ResolvedType>,
}

/// Index signature
#[derive(Debug, Clone)]
pub struct IndexSignature {
    /// Key type (string or number)
    pub key_type: Box<ResolvedType>,
    /// Value type
    pub value_type: Box<ResolvedType>,
    /// Whether the signature is readonly
    pub readonly: bool,
}

/// Function type
#[derive(Debug, Clone)]
pub struct FunctionType {
    /// Type parameters
    pub type_parameters: Vec<TypeParameterType>,
    /// Parameters
    pub parameters: Vec<ParameterType>,
    /// Return type
    pub return_type: Box<ResolvedType>,
}

/// Parameter type
#[derive(Debug, Clone)]
pub struct ParameterType {
    /// Parameter name
    pub name: InternedString,
    /// Parameter type
    pub ty: Box<ResolvedType>,
    /// Whether the parameter is optional
    pub optional: bool,
    /// Whether this is a rest parameter
    pub rest: bool,
}

/// Type parameter
#[derive(Debug, Clone)]
pub struct TypeParameterType {
    /// Parameter name
    pub name: InternedString,
    /// Constraint (extends clause)
    pub constraint: Option<Box<ResolvedType>>,
    /// Default type
    pub default: Option<Box<ResolvedType>>,
}

/// Type reference
#[derive(Debug, Clone)]
pub struct TypeReference {
    /// Referenced type name
    pub name: InternedString,
    /// Type arguments
    pub type_arguments: Vec<ResolvedType>,
    /// Resolved target (filled in during type checking)
    pub target: Option<TypeId>,
}

/// Literal type
#[derive(Debug, Clone)]
pub enum LiteralType {
    /// String literal
    String(String),
    /// Number literal
    Number(f64),
    /// Boolean literal
    Boolean(bool),
    /// BigInt literal
    BigInt(String),
}

/// Conditional type (T extends U ? X : Y)
#[derive(Debug, Clone)]
pub struct ConditionalType {
    /// Check type (T)
    pub check_type: ResolvedType,
    /// Extends type (U)
    pub extends_type: ResolvedType,
    /// True branch type (X)
    pub true_type: ResolvedType,
    /// False branch type (Y)
    pub false_type: ResolvedType,
}

/// Mapped type ({ [K in keyof T]: ... })
#[derive(Debug, Clone)]
pub struct MappedType {
    /// Type parameter name
    pub type_parameter: InternedString,
    /// Constraint type
    pub constraint: ResolvedType,
    /// Template type
    pub template: ResolvedType,
    /// Optional modifier (+?, -?, ?)
    pub optional_modifier: Option<MappedTypeModifier>,
    /// Readonly modifier (+readonly, -readonly, readonly)
    pub readonly_modifier: Option<MappedTypeModifier>,
}

/// Mapped type modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedTypeModifier {
    /// Add the modifier
    Add,
    /// Remove the modifier
    Remove,
    /// Keep existing
    Keep,
}

/// Indexed access type (T[K])
#[derive(Debug, Clone)]
pub struct IndexedAccessType {
    /// Object type
    pub object_type: ResolvedType,
    /// Index type
    pub index_type: ResolvedType,
}

impl ResolvedType {
    /// Creates a new any type
    pub fn any() -> Self {
        ResolvedType::Any
    }

    /// Creates a new unknown type
    pub fn unknown() -> Self {
        ResolvedType::Unknown
    }

    /// Creates a new never type
    pub fn never() -> Self {
        ResolvedType::Never
    }

    /// Checks if this is the any type
    pub fn is_any(&self) -> bool {
        matches!(self, ResolvedType::Any)
    }

    /// Checks if this is the unknown type
    pub fn is_unknown(&self) -> bool {
        matches!(self, ResolvedType::Unknown)
    }

    /// Checks if this is the never type
    pub fn is_never(&self) -> bool {
        matches!(self, ResolvedType::Never)
    }
}
