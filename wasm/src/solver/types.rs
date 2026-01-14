//! Type representation for the structural solver.
//!
//! Types are represented as lightweight `TypeId` handles that point into
//! an interning table. The actual structure is stored in `TypeKey`.

use crate::interner::Atom;
use serde::Serialize;
use bitflags::bitflags;

/// Fast-rejection flags for types.
///
/// These flags are computed during type interning and allow O(1) checks
/// for common type properties without traversing the full type structure.
///
/// This enables optimizations like:
/// - Quick truthiness checks for control flow analysis
/// - Fast object structure checks for property access
/// - Early rejection in subtype checking
bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct TypeFlags: u16 {
        /// No flags set (used for intrinsic types that don't match any special categories)
        const NONE = 0;

        /// Type is always truthy in boolean context (objects, arrays, functions)
        const IS_TRUTHY = 1 << 0;

        /// Type is always falsy in boolean context (null, undefined, false, 0, "", NaN)
        const IS_FALSY = 1 << 1;

        /// Type contains object structure (has properties, may be accessed with . operator)
        const HAS_OBJECT_STRUCTURE = 1 << 2;

        /// Type is callable (function type with call signatures)
        const IS_CALLABLE = 1 << 3;

        /// Type contains a union (may need distributive operations)
        const IS_UNION = 1 << 4;

        /// Type contains an intersection (may need distributive operations)
        const IS_INTERSECTION = 1 << 5;

        /// Type is or contains generics (type parameters, conditionals, etc.)
        const IS_GENERIC = 1 << 6;

        /// Type is an error type (poison pill for error recovery)
        const IS_ERROR = 1 << 7;
    }
}

impl TypeFlags {
    /// Compute flags for a TypeKey.
    /// This is called during type interning to populate the flags lookup table.
    pub fn from_type_key(key: &TypeKey) -> Self {
        let mut flags = TypeFlags::NONE;

        match key {
            // Intrinsic types
            TypeKey::Intrinsic(intrinsic) => {
                match intrinsic {
                    IntrinsicKind::Any | IntrinsicKind::Unknown => {
                        // any/unknown are neither truthy nor falsy
                        flags |= TypeFlags::IS_GENERIC;
                    }
                    IntrinsicKind::Never => {
                        // never is falsy (unreachable)
                        flags |= TypeFlags::IS_FALSY;
                    }
                    IntrinsicKind::Void | IntrinsicKind::Null | IntrinsicKind::Undefined => {
                        flags |= TypeFlags::IS_FALSY;
                    }
                    IntrinsicKind::Boolean | IntrinsicKind::Number | IntrinsicKind::String
                    | IntrinsicKind::Bigint | IntrinsicKind::Symbol => {
                        // Primitives without literal values - could be either
                    }
                    IntrinsicKind::Object => {
                        flags |= TypeFlags::IS_TRUTHY | TypeFlags::HAS_OBJECT_STRUCTURE;
                    }
                }
            }

            // Literal types
            TypeKey::Literal(literal) => {
                match literal {
                    LiteralValue::Boolean(false) => {
                        flags |= TypeFlags::IS_FALSY;
                    }
                    LiteralValue::Boolean(true) => {
                        flags |= TypeFlags::IS_TRUTHY;
                    }
                    LiteralValue::Number(n) => {
                        if n.0 == 0.0 || n.0.is_nan() {
                            flags |= TypeFlags::IS_FALSY;
                        } else {
                            flags |= TypeFlags::IS_TRUTHY;
                        }
                    }
                    LiteralValue::String(_s) => {
                        // Need to check if empty string - we don't have Atom access here
                        // For now, assume non-empty (will be corrected if needed)
                        flags |= TypeFlags::IS_TRUTHY;
                    }
                    LiteralValue::BigInt(_) => {
                        // BigInts are always truthy except 0n
                        // We'd need to check the string value
                        flags |= TypeFlags::IS_TRUTHY;
                    }
                }
            }

            // Object types
            TypeKey::Object(_) | TypeKey::ObjectWithIndex(_) => {
                flags |= TypeFlags::IS_TRUTHY | TypeFlags::HAS_OBJECT_STRUCTURE;
            }

            // Array type
            TypeKey::Array(_) => {
                flags |= TypeFlags::IS_TRUTHY | TypeFlags::HAS_OBJECT_STRUCTURE;
            }

            // Tuple type
            TypeKey::Tuple(_) => {
                flags |= TypeFlags::IS_TRUTHY | TypeFlags::HAS_OBJECT_STRUCTURE;
            }

            // Function type
            TypeKey::Function(_) => {
                flags |= TypeFlags::IS_TRUTHY | TypeFlags::IS_CALLABLE | TypeFlags::HAS_OBJECT_STRUCTURE;
            }

            // Callable type
            TypeKey::Callable(_) => {
                flags |= TypeFlags::IS_TRUTHY | TypeFlags::IS_CALLABLE | TypeFlags::HAS_OBJECT_STRUCTURE;
            }

            // Union type
            TypeKey::Union(_) => {
                flags |= TypeFlags::IS_UNION | TypeFlags::IS_GENERIC;
            }

            // Intersection type
            TypeKey::Intersection(_) => {
                flags |= TypeFlags::IS_INTERSECTION | TypeFlags::IS_GENERIC;
            }

            // Generic types
            TypeKey::TypeParameter(_) | TypeKey::Infer(_) => {
                flags |= TypeFlags::IS_GENERIC;
            }

            // Reference types (could be anything - need to resolve to know)
            TypeKey::Ref(_) | TypeKey::UniqueSymbol(_) => {
                flags |= TypeFlags::IS_GENERIC;
            }

            // Generic application
            TypeKey::Application(_) => {
                flags |= TypeFlags::IS_GENERIC;
            }

            // Conditional type
            TypeKey::Conditional(_) => {
                flags |= TypeFlags::IS_GENERIC;
            }

            // Mapped type
            TypeKey::Mapped(_) => {
                flags |= TypeFlags::IS_GENERIC | TypeFlags::HAS_OBJECT_STRUCTURE;
            }

            // Index access
            TypeKey::IndexAccess(_, _) => {
                flags |= TypeFlags::IS_GENERIC;
            }

            // Template literal
            TypeKey::TemplateLiteral(_) => {
                flags |= TypeFlags::IS_TRUTHY;
            }

            // Type query
            TypeKey::TypeQuery(_) => {
                flags |= TypeFlags::IS_GENERIC;
            }

            // KeyOf
            TypeKey::KeyOf(_) => {
                flags |= TypeFlags::IS_GENERIC;
            }

            // Readonly type modifier
            TypeKey::ReadonlyType(_) => {
                // Inherits flags from inner type
            }

            // This type
            TypeKey::ThisType => {
                flags |= TypeFlags::IS_GENERIC | TypeFlags::HAS_OBJECT_STRUCTURE;
            }

            // Error type
            TypeKey::Error => {
                flags |= TypeFlags::IS_ERROR;
            }
        }

        flags
    }

    /// Check if type is definitely truthy in boolean context.
    /// Returns None if it depends on runtime (e.g., union of truthy and falsy).
    pub fn is_definitely_truthy(self) -> Option<bool> {
        if self.contains(TypeFlags::IS_TRUTHY) && !self.contains(TypeFlags::IS_FALSY) {
            Some(true)
        } else if self.contains(TypeFlags::IS_FALSY) && !self.contains(TypeFlags::IS_TRUTHY) {
            Some(false)
        } else {
            None // Could be either (e.g., any, unknown, unions)
        }
    }
}

/// A lightweight handle to an interned type.
/// Equality check is O(1) - just compare the u32 values.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Default)]
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
    pub const BOOLEAN_TRUE: TypeId = TypeId(14);
    pub const BOOLEAN_FALSE: TypeId = TypeId(15);
    pub const FUNCTION: TypeId = TypeId(16);

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

/// Interned list of TypeId values (e.g., unions/intersections).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeListId(pub u32);

/// Interned object shape (properties + index signatures).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ObjectShapeId(pub u32);

/// Interned tuple element list.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TupleListId(pub u32);

/// Interned function shape.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionShapeId(pub u32);

/// Interned callable shape.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CallableShapeId(pub u32);

/// Interned type application (Base<Args>).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeApplicationId(pub u32);

/// Interned template literal span list.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TemplateLiteralId(pub u32);

/// Interned conditional type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConditionalTypeId(pub u32);

/// Interned mapped type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MappedTypeId(pub u32);

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
    Object(ObjectShapeId),

    /// Object type with index signatures
    /// For objects like { [key: string]: number, foo: string }
    ObjectWithIndex(ObjectShapeId),

    /// Union type (A | B | C)
    Union(TypeListId),

    /// Intersection type (A & B & C)
    Intersection(TypeListId),

    /// Array type
    Array(TypeId),

    /// Tuple type
    Tuple(TupleListId),

    /// Function type
    Function(FunctionShapeId),

    /// Callable type with overloaded signatures
    /// For interfaces with call/construct signatures
    Callable(CallableShapeId),

    /// Type parameter (generic)
    TypeParameter(TypeParamInfo),

    /// Reference to a named type (interface, class, type alias)
    /// Uses SymbolId to break infinite recursion
    Ref(SymbolRef),

    /// Generic type application (Base<Args>)
    Application(TypeApplicationId),

    /// Conditional type (T extends U ? X : Y)
    Conditional(ConditionalTypeId),

    /// Mapped type ({ [K in Keys]: ValueType })
    Mapped(MappedTypeId),

    /// Index access type (T[K])
    IndexAccess(TypeId, TypeId),

    /// Template literal type (`hello${string}world`)
    TemplateLiteral(TemplateLiteralId),

    /// Type query (typeof expression in type position)
    TypeQuery(SymbolRef),

    /// KeyOf type operator (keyof T)
    KeyOf(TypeId),

    /// Readonly type modifier (readonly T[])
    ReadonlyType(TypeId),

    /// Unique symbol type
    UniqueSymbol(SymbolRef),

    /// Infer type (infer R in conditional types)
    Infer(TypeParamInfo),

    /// This type (polymorphic this)
    ThisType,

    /// Error type for recovery
    Error,
}

/// Generic type application (Base<Args>)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeApplication {
    pub base: TypeId,
    pub args: Vec<TypeId>,
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
    String(Atom),
    Number(OrderedFloat),
    BigInt(Atom),
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
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct PropertyInfo {
    pub name: Atom,
    /// Read type (getter/lookup).
    pub type_id: TypeId,
    /// Write type (setter/assignment).
    pub write_type: TypeId,
    pub optional: bool,
    pub readonly: bool,
    pub is_method: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PropertyLookup {
    Found(usize),
    NotFound,
    Uncached,
}

/// Index signature information for object types
/// Represents `{ [key: string]: ValueType }` or `{ [key: number]: ValueType }`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IndexSignature {
    /// The key type (usually string or number)
    pub key_type: TypeId,
    /// The value type for all indexed properties
    pub value_type: TypeId,
    /// Whether the index signature is readonly
    pub readonly: bool,
}

/// Object type with properties and optional index signatures
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ObjectShape {
    /// Named properties (sorted by name for consistent hashing)
    pub properties: Vec<PropertyInfo>,
    /// String index signature: { [key: string]: T }
    pub string_index: Option<IndexSignature>,
    /// Number index signature: { [key: number]: T }
    pub number_index: Option<IndexSignature>,
}

/// Tuple element information
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TupleElement {
    pub type_id: TypeId,
    pub name: Option<Atom>,
    pub optional: bool,
    pub rest: bool,
}

/// Type predicate information (x is T / asserts x is T).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypePredicate {
    pub asserts: bool,
    pub target: TypePredicateTarget,
    pub type_id: Option<TypeId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypePredicateTarget {
    This,
    Identifier(Atom),
}

/// Function shape for function types
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionShape {
    pub type_params: Vec<TypeParamInfo>,
    pub params: Vec<ParamInfo>,
    pub this_type: Option<TypeId>,
    pub return_type: TypeId,
    pub type_predicate: Option<TypePredicate>,
    pub is_constructor: bool,
    /// Whether this function is a method (bivariant parameters) vs a standalone function (contravariant when strictFunctionTypes)
    pub is_method: bool,
}

/// Call signature for overloaded functions
/// Represents a single call signature in an overloaded type
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CallSignature {
    pub type_params: Vec<TypeParamInfo>,
    pub params: Vec<ParamInfo>,
    pub this_type: Option<TypeId>,
    pub return_type: TypeId,
    pub type_predicate: Option<TypePredicate>,
}

/// Callable type with multiple overloaded call signatures
/// Represents types like:
/// ```typescript
/// interface Overloaded {
///   (x: string): number;
///   (x: number): string;
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct CallableShape {
    /// Call signatures (order matters for overload resolution)
    pub call_signatures: Vec<CallSignature>,
    /// Constructor signatures
    pub construct_signatures: Vec<CallSignature>,
    /// Optional properties on the callable (e.g., Function.prototype)
    pub properties: Vec<PropertyInfo>,
    /// String index signature (for static index signatures on classes)
    pub string_index: Option<IndexSignature>,
    /// Number index signature (for static index signatures on classes)
    pub number_index: Option<IndexSignature>,
}

/// Parameter information
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct ParamInfo {
    pub name: Option<Atom>,
    pub type_id: TypeId,
    pub optional: bool,
    pub rest: bool,
}

/// Type parameter information
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeParamInfo {
    pub name: Atom,
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
    pub is_distributive: bool,
}

/// Mapped type structure
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MappedType {
    pub type_param: TypeParamInfo,
    pub constraint: TypeId,
    pub name_type: Option<TypeId>,
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
    Text(Atom),
    Type(TypeId),
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
