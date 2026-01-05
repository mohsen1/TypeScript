//! Type representation for the structural solver.
//!
//! Types are represented as lightweight `TypeId` handles that point into
//! an interning table. The actual structure is stored in `TypeKey`.

use serde::Serialize;
use std::sync::Arc;

/// A lightweight handle to an interned type.
/// Equality check is O(1) - just compare the u32 values.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct TypeId(pub u32);

impl TypeId {
    pub const NONE: TypeId = TypeId(0);
    pub const ERROR: TypeId = TypeId(1);
    pub const NEVER: TypeId = TypeId(2);
    pub const UNKNOWN: TypeId = TypeId(3);
    pub const ANY: TypeId = TypeId(4);
    pub const VOID: TypeId = TypeId(5);
    pub const UNDEFINED: TypeId = TypeId(6);
    pub const NULL: TypeId = TypeId(7);
    pub const BOOLEAN: TypeId = TypeId(8);
    pub const NUMBER: TypeId = TypeId(9);
    pub const STRING: TypeId = TypeId(10);
    pub const BIGINT: TypeId = TypeId(11);
    pub const SYMBOL: TypeId = TypeId(12);
    pub const OBJECT: TypeId = TypeId(13);

    /// First user-defined type ID (after built-in intrinsics)
    pub const FIRST_USER: u32 = 100;

    pub fn is_intrinsic(self) -> bool {
        self.0 < Self::FIRST_USER
    }

    pub fn is_error(self) -> bool {
        self == Self::ERROR
    }

    pub fn is_any(self) -> bool {
        self == Self::ANY
    }

    pub fn is_unknown(self) -> bool {
        self == Self::UNKNOWN
    }

    pub fn is_never(self) -> bool {
        self == Self::NEVER
    }
}

/// The structural "shape" of a type.
/// This is the key used for interning - structurally identical types
/// will have the same TypeKey and therefore the same TypeId.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeKey {
    /// Intrinsic types (any, unknown, never, void, null, undefined, boolean, number, string, bigint, symbol, object)
    Intrinsic(IntrinsicKind),

    /// Literal types ("hello", 42, true, 123n)
    Literal(LiteralValue),

    /// Object type with sorted property list for structural identity
    /// Vec is sorted by property name for consistent hashing
    Object(Vec<PropertyInfo>),

    /// Union type (A | B | C)
    /// Vec is sorted by TypeId for consistent hashing
    Union(Vec<TypeId>),

    /// Intersection type (A & B & C)
    /// Vec is sorted by TypeId for consistent hashing
    Intersection(Vec<TypeId>),

    /// Array type
    Array(TypeId),

    /// Tuple type
    Tuple(Vec<TupleElement>),

    /// Function type
    Function(FunctionShape),

    /// Type parameter (generic)
    TypeParameter(TypeParamInfo),

    /// Reference to a named type (interface, class, type alias)
    /// Uses SymbolId to break infinite recursion
    Ref(SymbolRef),

    /// Conditional type (T extends U ? X : Y)
    Conditional(Box<ConditionalType>),

    /// Mapped type ({ [K in Keys]: ValueType })
    Mapped(Box<MappedType>),

    /// Index access type (T[K])
    IndexAccess(TypeId, TypeId),

    /// Template literal type (`hello${string}world`)
    TemplateLiteral(Vec<TemplateSpan>),

    /// Error type for recovery
    Error,
}

/// Intrinsic type kinds
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IntrinsicKind {
    Any,
    Unknown,
    Never,
    Void,
    Null,
    Undefined,
    Boolean,
    Number,
    String,
    Bigint,
    Symbol,
    Object,
}

impl IntrinsicKind {
    pub fn to_type_id(self) -> TypeId {
        match self {
            IntrinsicKind::Any => TypeId::ANY,
            IntrinsicKind::Unknown => TypeId::UNKNOWN,
            IntrinsicKind::Never => TypeId::NEVER,
            IntrinsicKind::Void => TypeId::VOID,
            IntrinsicKind::Null => TypeId::NULL,
            IntrinsicKind::Undefined => TypeId::UNDEFINED,
            IntrinsicKind::Boolean => TypeId::BOOLEAN,
            IntrinsicKind::Number => TypeId::NUMBER,
            IntrinsicKind::String => TypeId::STRING,
            IntrinsicKind::Bigint => TypeId::BIGINT,
            IntrinsicKind::Symbol => TypeId::SYMBOL,
            IntrinsicKind::Object => TypeId::OBJECT,
        }
    }
}

/// Literal values (for literal types)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LiteralValue {
    String(Arc<str>),
    Number(OrderedFloat),
    BigInt(Arc<str>),
    Boolean(bool),
}

/// Wrapper for f64 that implements Eq and Hash for use in TypeKey
#[derive(Clone, Copy, Debug)]
pub struct OrderedFloat(pub f64);

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for OrderedFloat {}

impl std::hash::Hash for OrderedFloat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

/// Property information for object types
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PropertyInfo {
    pub name: Arc<str>,
    pub type_id: TypeId,
    pub optional: bool,
    pub readonly: bool,
}

/// Tuple element information
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TupleElement {
    pub type_id: TypeId,
    pub name: Option<Arc<str>>,
    pub optional: bool,
    pub rest: bool,
}

/// Function shape for function types
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionShape {
    pub type_params: Vec<TypeParamInfo>,
    pub params: Vec<ParamInfo>,
    pub return_type: TypeId,
    pub is_constructor: bool,
}

/// Parameter information
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParamInfo {
    pub name: Option<Arc<str>>,
    pub type_id: TypeId,
    pub optional: bool,
    pub rest: bool,
}

/// Type parameter information
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeParamInfo {
    pub name: Arc<str>,
    pub constraint: Option<TypeId>,
    pub default: Option<TypeId>,
}

/// Reference to a symbol (for named types)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SymbolRef(pub u32);

/// Conditional type structure
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConditionalType {
    pub check_type: TypeId,
    pub extends_type: TypeId,
    pub true_type: TypeId,
    pub false_type: TypeId,
}

/// Mapped type structure
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MappedType {
    pub type_param: TypeParamInfo,
    pub constraint: TypeId,
    pub template: TypeId,
    pub readonly_modifier: Option<MappedModifier>,
    pub optional_modifier: Option<MappedModifier>,
}

/// Mapped type modifier (+/-)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MappedModifier {
    Add,
    Remove,
}

/// Template literal span
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TemplateSpan {
    Text(Arc<str>),
    Type(TypeId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_id_intrinsics() {
        assert!(TypeId::ANY.is_intrinsic());
        assert!(TypeId::STRING.is_intrinsic());
        assert!(!TypeId(100).is_intrinsic());
        assert!(!TypeId(1000).is_intrinsic());
    }

    #[test]
    fn test_type_id_equality() {
        // O(1) equality check
        let a = TypeId(42);
        let b = TypeId(42);
        let c = TypeId(43);

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_ordered_float_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(OrderedFloat(1.5));
        set.insert(OrderedFloat(2.5));
        set.insert(OrderedFloat(1.5)); // duplicate

        assert_eq!(set.len(), 2);
    }
}
