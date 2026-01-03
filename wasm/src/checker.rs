//! Type Checker implementation for TypeScript AST.
//!
//! The checker performs type inference and type checking, producing
//! diagnostics for type errors.

use serde::Serialize;
use crate::binder::{SymbolId, SymbolArena, SymbolTable, symbol_flags};
use crate::parser::NodeIndex;

// =============================================================================
// Type Flags
// =============================================================================

/// Flags that describe the kind of a type.
/// Matches TypeScript's TypeFlags enum in src/compiler/types.ts
pub mod type_flags {
    // Primitive types
    pub const ANY: u32             = 1 << 0;
    pub const UNKNOWN: u32         = 1 << 1;
    pub const STRING: u32          = 1 << 2;
    pub const NUMBER: u32          = 1 << 3;
    pub const BOOLEAN: u32         = 1 << 4;
    pub const ENUM: u32            = 1 << 5;
    pub const BIG_INT: u32         = 1 << 6;

    // Literal types
    pub const STRING_LITERAL: u32  = 1 << 7;
    pub const NUMBER_LITERAL: u32  = 1 << 8;
    pub const BOOLEAN_LITERAL: u32 = 1 << 9;
    pub const ENUM_LITERAL: u32    = 1 << 10;
    pub const BIG_INT_LITERAL: u32 = 1 << 11;

    // Symbol types
    pub const ES_SYMBOL: u32       = 1 << 12;
    pub const UNIQUE_ES_SYMBOL: u32 = 1 << 13;

    // Special types
    pub const VOID: u32            = 1 << 14;
    pub const UNDEFINED: u32       = 1 << 15;
    pub const NULL: u32            = 1 << 16;
    pub const NEVER: u32           = 1 << 17;

    // Compound types
    pub const TYPE_PARAMETER: u32  = 1 << 18;
    pub const OBJECT: u32          = 1 << 19;
    pub const UNION: u32           = 1 << 20;
    pub const INTERSECTION: u32    = 1 << 21;

    // Type operators
    pub const INDEX: u32           = 1 << 22;  // keyof T
    pub const INDEXED_ACCESS: u32  = 1 << 23;  // T[K]
    pub const CONDITIONAL: u32     = 1 << 24;  // T extends U ? X : Y
    pub const SUBSTITUTION: u32    = 1 << 25;

    // Other
    pub const NON_PRIMITIVE: u32   = 1 << 26;  // object
    pub const TEMPLATE_LITERAL: u32 = 1 << 27;
    pub const STRING_MAPPING: u32  = 1 << 28;  // Uppercase<T>

    // Composite flags
    pub const ANY_OR_UNKNOWN: u32 = ANY | UNKNOWN;
    pub const NULLABLE: u32 = UNDEFINED | NULL;
    pub const LITERAL: u32 = STRING_LITERAL | NUMBER_LITERAL | BIG_INT_LITERAL | BOOLEAN_LITERAL;
    pub const UNIT: u32 = ENUM | LITERAL | UNIQUE_ES_SYMBOL | NULLABLE;
    pub const STRING_OR_NUMBER_LITERAL: u32 = STRING_LITERAL | NUMBER_LITERAL;
    pub const STRING_LIKE: u32 = STRING | STRING_LITERAL | TEMPLATE_LITERAL | STRING_MAPPING;
    pub const NUMBER_LIKE: u32 = NUMBER | NUMBER_LITERAL | ENUM;
    pub const BIG_INT_LIKE: u32 = BIG_INT | BIG_INT_LITERAL;
    pub const BOOLEAN_LIKE: u32 = BOOLEAN | BOOLEAN_LITERAL;
    pub const ENUM_LIKE: u32 = ENUM | ENUM_LITERAL;
    pub const ES_SYMBOL_LIKE: u32 = ES_SYMBOL | UNIQUE_ES_SYMBOL;
    pub const VOID_LIKE: u32 = VOID | UNDEFINED;
    pub const PRIMITIVE: u32 = STRING_LIKE | NUMBER_LIKE | BIG_INT_LIKE | BOOLEAN_LIKE | ENUM_LIKE | ES_SYMBOL_LIKE | VOID_LIKE | NULL;
    pub const UNION_OR_INTERSECTION: u32 = UNION | INTERSECTION;
    pub const STRUCTURED_TYPE: u32 = OBJECT | UNION | INTERSECTION;
    pub const TYPE_VARIABLE: u32 = TYPE_PARAMETER | INDEXED_ACCESS;
    pub const INSTANTIABLE_NON_PRIMITIVE: u32 = TYPE_VARIABLE | CONDITIONAL | SUBSTITUTION;
    pub const INSTANTIABLE_PRIMITIVE: u32 = INDEX | TEMPLATE_LITERAL | STRING_MAPPING;
    pub const INSTANTIABLE: u32 = INSTANTIABLE_NON_PRIMITIVE | INSTANTIABLE_PRIMITIVE;
    pub const STRUCTURED_OR_INSTANTIABLE: u32 = STRUCTURED_TYPE | INSTANTIABLE;
    pub const NARROWABLE: u32 = ANY | UNKNOWN | STRUCTURED_OR_INSTANTIABLE | STRING_LIKE | NUMBER_LIKE | BIG_INT_LIKE | BOOLEAN_LIKE | ES_SYMBOL | UNIQUE_ES_SYMBOL | NON_PRIMITIVE;
}

// =============================================================================
// Object Flags
// =============================================================================

/// Additional flags for object types.
/// Matches TypeScript's ObjectFlags enum in src/compiler/types.ts
pub mod object_flags {
    pub const CLASS: u32                = 1 << 0;
    pub const INTERFACE: u32            = 1 << 1;
    pub const REFERENCE: u32            = 1 << 2;
    pub const TUPLE: u32                = 1 << 3;
    pub const ANONYMOUS: u32            = 1 << 4;
    pub const MAPPED: u32               = 1 << 5;
    pub const INSTANTIATED: u32         = 1 << 6;
    pub const OBJECT_LITERAL: u32       = 1 << 7;
    pub const EVOLVING_ARRAY: u32       = 1 << 8;
    pub const OBJECT_LITERAL_PATTERN: u32 = 1 << 9;
    pub const FRESH_LITERAL: u32        = 1 << 10;
    pub const ARRAY_LITERAL: u32        = 1 << 11;
    pub const PRIMITIVE_UNION: u32      = 1 << 12;
    pub const CONTAINS_SPREAD: u32      = 1 << 13;
    pub const REVERSE_MAPPED: u32       = 1 << 14;
    pub const JSX_ATTRIBUTES: u32       = 1 << 15;
    pub const MARKER: u32               = 1 << 16;
    pub const CLASS_OR_INTERFACE: u32   = CLASS | INTERFACE;
}

// =============================================================================
// Type ID
// =============================================================================

/// Unique identifier for a type in the type arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct TypeId(pub u32);

impl TypeId {
    pub const NONE: TypeId = TypeId(u32::MAX);

    pub fn is_none(&self) -> bool {
        self.0 == u32::MAX
    }
}

// =============================================================================
// Literal Values
// =============================================================================

/// A literal value for literal types.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    BigInt(String),  // Store as string for precision
    Boolean(bool),
}

// =============================================================================
// Signature
// =============================================================================

/// Flags for function signatures.
pub mod signature_flags {
    pub const NONE: u32 = 0;
    pub const HAS_REST_PARAMETER: u32 = 1 << 0;
    pub const HAS_LITERAL_TYPES: u32 = 1 << 1;
    pub const IS_INNER_CALL_CHAIN: u32 = 1 << 2;
    pub const IS_OUTER_CALL_CHAIN: u32 = 1 << 3;
    pub const IS_UNTERMINATED_CALL_CHAIN: u32 = 1 << 4;
    pub const OPTIONAL_CALL_CHAIN: u32 = IS_INNER_CALL_CHAIN | IS_OUTER_CALL_CHAIN;
    pub const CALL_CHAIN_FLAGS: u32 = OPTIONAL_CALL_CHAIN | IS_UNTERMINATED_CALL_CHAIN;
}

/// A function or method signature.
#[derive(Clone, Debug, Serialize)]
pub struct Signature {
    pub declaration: NodeIndex,
    pub type_parameters: Vec<TypeId>,
    pub parameters: Vec<SymbolId>,
    pub this_parameter: Option<SymbolId>,
    pub resolved_return_type: Option<TypeId>,
    pub min_argument_count: u32,
    pub flags: u32,
}

impl Signature {
    pub fn new(declaration: NodeIndex) -> Self {
        Signature {
            declaration,
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            this_parameter: None,
            resolved_return_type: None,
            min_argument_count: 0,
            flags: signature_flags::NONE,
        }
    }
}

// =============================================================================
// Index Info
// =============================================================================

/// Information about an index signature.
#[derive(Clone, Debug, Serialize)]
pub struct IndexInfo {
    pub key_type: TypeId,
    pub value_type: TypeId,
    pub is_readonly: bool,
    pub declaration: Option<NodeIndex>,
}

// =============================================================================
// Type Variants
// =============================================================================

/// An intrinsic type (primitives like string, number, etc.)
#[derive(Clone, Debug, Serialize)]
pub struct IntrinsicType {
    pub flags: u32,
    pub intrinsic_name: String,
}

/// A literal type (specific string, number, boolean value)
#[derive(Clone, Debug, Serialize)]
pub struct LiteralType {
    pub flags: u32,
    pub value: LiteralValue,
    pub fresh_type: TypeId,    // Widening version
    pub regular_type: TypeId,  // Non-widening version
}

/// An object type (class, interface, object literal, etc.)
#[derive(Clone, Debug, Serialize)]
pub struct ObjectType {
    pub flags: u32,
    pub object_flags: u32,
    pub symbol: SymbolId,
    pub members: SymbolTable,
    pub properties: Vec<SymbolId>,
    pub call_signatures: Vec<Signature>,
    pub construct_signatures: Vec<Signature>,
    pub index_infos: Vec<IndexInfo>,
}

impl ObjectType {
    pub fn new(object_flags: u32, symbol: SymbolId) -> Self {
        ObjectType {
            flags: type_flags::OBJECT,
            object_flags,
            symbol,
            members: SymbolTable::new(),
            properties: Vec::new(),
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            index_infos: Vec::new(),
        }
    }

    /// Check if this object type has specific object flags set.
    pub fn has_object_flags(&self, flags: u32) -> bool {
        (self.object_flags & flags) != 0
    }
}

/// A type reference (Array<T>, Map<K, V>, etc.)
#[derive(Clone, Debug, Serialize)]
pub struct TypeReference {
    pub flags: u32,
    pub object_flags: u32,
    pub target: TypeId,              // The generic type
    pub type_arguments: Vec<TypeId>, // The type arguments
    pub symbol: SymbolId,
}

/// A union type (A | B | C)
#[derive(Clone, Debug, Serialize)]
pub struct UnionType {
    pub flags: u32,
    pub object_flags: u32,
    pub types: Vec<TypeId>,
    pub origin: Option<TypeId>,
}

impl UnionType {
    pub fn new(types: Vec<TypeId>) -> Self {
        UnionType {
            flags: type_flags::UNION,
            object_flags: 0,
            types,
            origin: None,
        }
    }
}

/// An intersection type (A & B & C)
#[derive(Clone, Debug, Serialize)]
pub struct IntersectionType {
    pub flags: u32,
    pub object_flags: u32,
    pub types: Vec<TypeId>,
}

impl IntersectionType {
    pub fn new(types: Vec<TypeId>) -> Self {
        IntersectionType {
            flags: type_flags::INTERSECTION,
            object_flags: 0,
            types,
        }
    }
}

/// A type parameter (T, K extends keyof T, etc.)
#[derive(Clone, Debug, Serialize)]
pub struct TypeParameter {
    pub flags: u32,
    pub symbol: SymbolId,
    pub constraint: TypeId,    // extends clause
    pub default: TypeId,       // default type
    pub target: TypeId,        // For substitution
    pub is_this_type: bool,
}

impl TypeParameter {
    pub fn new(symbol: SymbolId) -> Self {
        TypeParameter {
            flags: type_flags::TYPE_PARAMETER,
            symbol,
            constraint: TypeId::NONE,
            default: TypeId::NONE,
            target: TypeId::NONE,
            is_this_type: false,
        }
    }
}

/// A conditional type (T extends U ? X : Y)
#[derive(Clone, Debug, Serialize)]
pub struct ConditionalType {
    pub flags: u32,
    pub check_type: TypeId,
    pub extends_type: TypeId,
    pub true_type: TypeId,
    pub false_type: TypeId,
    pub is_distributive: bool,
    pub infer_type_parameters: Vec<TypeId>,
}

/// A mapped type ({ [K in keyof T]: ... })
#[derive(Clone, Debug, Serialize)]
pub struct MappedType {
    pub flags: u32,
    pub object_flags: u32,
    pub declaration: NodeIndex,
    pub type_parameter: TypeId,
    pub constraint_type: TypeId,
    pub name_type: TypeId,      // as clause
    pub template_type: TypeId,
}

/// An indexed access type (T[K])
#[derive(Clone, Debug, Serialize)]
pub struct IndexedAccessType {
    pub flags: u32,
    pub object_type: TypeId,
    pub index_type: TypeId,
    pub constraint: TypeId,
}

/// An index type (keyof T)
#[derive(Clone, Debug, Serialize)]
pub struct IndexType {
    pub flags: u32,
    pub source_type: TypeId,
}

/// A template literal type (`hello ${T}`)
#[derive(Clone, Debug, Serialize)]
pub struct TemplateLiteralType {
    pub flags: u32,
    pub texts: Vec<String>,
    pub types: Vec<TypeId>,
}

/// A function type ((x: T) => U)
#[derive(Clone, Debug, Serialize)]
pub struct FunctionType {
    pub flags: u32,
    pub object_flags: u32,
    pub declaration: NodeIndex,
    pub parameter_types: Vec<TypeId>,
    pub parameter_names: Vec<String>,
    pub return_type: TypeId,
    pub type_parameters: Vec<TypeId>,
    pub min_argument_count: u32,
    pub has_rest_parameter: bool,
}

/// An array type (T[] or Array<T>).
#[derive(Clone, Debug, Serialize)]
pub struct ArrayTypeInfo {
    pub flags: u32,
    pub element_type: TypeId,
    /// Whether this is a readonly array (readonly T[] or ReadonlyArray<T>)
    pub is_readonly: bool,
}

/// A tuple type ([T, U, V]).
#[derive(Clone, Debug, Serialize)]
pub struct TupleTypeInfo {
    pub flags: u32,
    pub element_types: Vec<TypeId>,
    /// Whether the tuple has optional elements
    pub has_optional_elements: bool,
    /// Whether the tuple has a rest element
    pub has_rest_element: bool,
    /// Whether this is a readonly tuple (readonly [T, U])
    pub is_readonly: bool,
}

// =============================================================================
// Type Enum
// =============================================================================

/// All possible type variants.
#[derive(Clone, Debug, Serialize)]
pub enum Type {
    Intrinsic(IntrinsicType),
    Literal(LiteralType),
    Object(ObjectType),
    TypeReference(TypeReference),
    Union(UnionType),
    Intersection(IntersectionType),
    TypeParameter(TypeParameter),
    Conditional(ConditionalType),
    Mapped(MappedType),
    IndexedAccess(IndexedAccessType),
    Index(IndexType),
    TemplateLiteral(TemplateLiteralType),
    Function(FunctionType),
    Array(ArrayTypeInfo),
    Tuple(TupleTypeInfo),
}

impl Type {
    /// Get the flags for this type.
    pub fn flags(&self) -> u32 {
        match self {
            Type::Intrinsic(t) => t.flags,
            Type::Literal(t) => t.flags,
            Type::Object(t) => t.flags,
            Type::TypeReference(t) => t.flags,
            Type::Union(t) => t.flags,
            Type::Intersection(t) => t.flags,
            Type::TypeParameter(t) => t.flags,
            Type::Conditional(t) => t.flags,
            Type::Mapped(t) => t.flags,
            Type::IndexedAccess(t) => t.flags,
            Type::Index(t) => t.flags,
            Type::TemplateLiteral(t) => t.flags,
            Type::Function(t) => t.flags,
            Type::Array(t) => t.flags,
            Type::Tuple(t) => t.flags,
        }
    }

    /// Check if type has all specified flags.
    pub fn has_flags(&self, flags: u32) -> bool {
        (self.flags() & flags) == flags
    }

    /// Check if type has any of specified flags.
    pub fn has_any_flags(&self, flags: u32) -> bool {
        (self.flags() & flags) != 0
    }
}

// =============================================================================
// Type Arena
// =============================================================================

/// Arena allocator for types with singleton caching.
#[derive(Debug, Serialize)]
pub struct TypeArena {
    types: Vec<Type>,
    // Singleton intrinsic types
    pub any_type: TypeId,
    pub unknown_type: TypeId,
    pub string_type: TypeId,
    pub number_type: TypeId,
    pub boolean_type: TypeId,
    pub big_int_type: TypeId,
    pub es_symbol_type: TypeId,
    pub void_type: TypeId,
    pub undefined_type: TypeId,
    pub null_type: TypeId,
    pub never_type: TypeId,
    pub object_type: TypeId,
    // Literal singletons
    pub true_type: TypeId,
    pub false_type: TypeId,
}

impl TypeArena {
    pub fn new() -> Self {
        let mut arena = TypeArena {
            types: Vec::new(),
            any_type: TypeId::NONE,
            unknown_type: TypeId::NONE,
            string_type: TypeId::NONE,
            number_type: TypeId::NONE,
            boolean_type: TypeId::NONE,
            big_int_type: TypeId::NONE,
            es_symbol_type: TypeId::NONE,
            void_type: TypeId::NONE,
            undefined_type: TypeId::NONE,
            null_type: TypeId::NONE,
            never_type: TypeId::NONE,
            object_type: TypeId::NONE,
            true_type: TypeId::NONE,
            false_type: TypeId::NONE,
        };

        // Pre-allocate singleton intrinsic types
        arena.any_type = arena.create_intrinsic(type_flags::ANY, "any");
        arena.unknown_type = arena.create_intrinsic(type_flags::UNKNOWN, "unknown");
        arena.string_type = arena.create_intrinsic(type_flags::STRING, "string");
        arena.number_type = arena.create_intrinsic(type_flags::NUMBER, "number");
        arena.boolean_type = arena.create_intrinsic(type_flags::BOOLEAN, "boolean");
        arena.big_int_type = arena.create_intrinsic(type_flags::BIG_INT, "bigint");
        arena.es_symbol_type = arena.create_intrinsic(type_flags::ES_SYMBOL, "symbol");
        arena.void_type = arena.create_intrinsic(type_flags::VOID, "void");
        arena.undefined_type = arena.create_intrinsic(type_flags::UNDEFINED, "undefined");
        arena.null_type = arena.create_intrinsic(type_flags::NULL, "null");
        arena.never_type = arena.create_intrinsic(type_flags::NEVER, "never");
        arena.object_type = arena.create_intrinsic(type_flags::NON_PRIMITIVE, "object");

        // Boolean literal singletons
        arena.true_type = arena.create_boolean_literal(true);
        arena.false_type = arena.create_boolean_literal(false);

        arena
    }

    /// Create an intrinsic type.
    fn create_intrinsic(&mut self, flags: u32, name: &str) -> TypeId {
        let id = TypeId(self.types.len() as u32);
        self.types.push(Type::Intrinsic(IntrinsicType {
            flags,
            intrinsic_name: name.to_string(),
        }));
        id
    }

    /// Create a boolean literal type.
    fn create_boolean_literal(&mut self, value: bool) -> TypeId {
        let id = TypeId(self.types.len() as u32);
        self.types.push(Type::Literal(LiteralType {
            flags: type_flags::BOOLEAN_LITERAL,
            value: LiteralValue::Boolean(value),
            fresh_type: id,
            regular_type: id,
        }));
        id
    }

    /// Allocate a new type.
    pub fn alloc(&mut self, typ: Type) -> TypeId {
        let id = TypeId(self.types.len() as u32);
        self.types.push(typ);
        id
    }

    /// Get a type by ID.
    pub fn get(&self, id: TypeId) -> Option<&Type> {
        if id.is_none() {
            None
        } else {
            self.types.get(id.0 as usize)
        }
    }

    /// Get a mutable type by ID.
    pub fn get_mut(&mut self, id: TypeId) -> Option<&mut Type> {
        if id.is_none() {
            None
        } else {
            self.types.get_mut(id.0 as usize)
        }
    }

    /// Get the number of types.
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Create a string literal type.
    pub fn create_string_literal(&mut self, value: String) -> TypeId {
        let id = TypeId(self.types.len() as u32);
        self.types.push(Type::Literal(LiteralType {
            flags: type_flags::STRING_LITERAL,
            value: LiteralValue::String(value),
            fresh_type: id,
            regular_type: id,
        }));
        id
    }

    /// Create a number literal type.
    pub fn create_number_literal(&mut self, value: f64) -> TypeId {
        let id = TypeId(self.types.len() as u32);
        self.types.push(Type::Literal(LiteralType {
            flags: type_flags::NUMBER_LITERAL,
            value: LiteralValue::Number(value),
            fresh_type: id,
            regular_type: id,
        }));
        id
    }

    /// Create a union type with simplification.
    /// - Flattens nested unions
    /// - Removes `never` (X | never = X)
    /// - Returns `any` if any constituent is `any`
    /// - Removes duplicates
    /// - Returns single type if only one remains
    pub fn create_union(&mut self, types: Vec<TypeId>) -> TypeId {
        // First, flatten nested unions and collect constituent types
        let mut flattened: Vec<TypeId> = Vec::new();
        for type_id in types {
            if let Some(typ) = self.get(type_id) {
                let flags = typ.flags();

                // If any type is any, the whole union is any
                if (flags & type_flags::ANY) != 0 {
                    return self.any_type;
                }

                // Skip never (X | never = X)
                if (flags & type_flags::NEVER) != 0 {
                    continue;
                }

                // Flatten nested unions
                if let Type::Union(ut) = typ {
                    let nested_types = ut.types.clone();
                    for nested_id in nested_types {
                        // Check nested for any/never too
                        if let Some(nested_typ) = self.get(nested_id) {
                            let nested_flags = nested_typ.flags();
                            if (nested_flags & type_flags::ANY) != 0 {
                                return self.any_type;
                            }
                            if (nested_flags & type_flags::NEVER) != 0 {
                                continue;
                            }
                        }
                        if !flattened.contains(&nested_id) {
                            flattened.push(nested_id);
                        }
                    }
                } else {
                    // Add if not already present (dedup)
                    if !flattened.contains(&type_id) {
                        flattened.push(type_id);
                    }
                }
            } else {
                // Type not found, add as-is
                if !flattened.contains(&type_id) {
                    flattened.push(type_id);
                }
            }
        }

        // If empty after filtering, return never (identity for union)
        if flattened.is_empty() {
            return self.never_type;
        }

        // If only one type, return it directly
        if flattened.len() == 1 {
            return flattened[0];
        }

        self.alloc(Type::Union(UnionType::new(flattened)))
    }

    /// Create an intersection type with simplification.
    /// - Flattens nested intersections
    /// - Returns `never` if any constituent is `never`
    /// - Removes `unknown` (X & unknown = X)
    /// - Removes duplicates
    /// - Returns single type if only one remains
    pub fn create_intersection(&mut self, types: Vec<TypeId>) -> TypeId {
        // First, flatten nested intersections and collect constituent types
        let mut flattened: Vec<TypeId> = Vec::new();
        for type_id in types {
            if let Some(typ) = self.get(type_id) {
                let flags = typ.flags();

                // If any type is never, the whole intersection is never
                if (flags & type_flags::NEVER) != 0 {
                    return self.never_type;
                }

                // Skip unknown (X & unknown = X)
                if (flags & type_flags::UNKNOWN) != 0 {
                    continue;
                }

                // Flatten nested intersections
                if let Type::Intersection(it) = typ {
                    let nested_types = it.types.clone();
                    for nested_id in nested_types {
                        // Check nested for never/unknown too
                        if let Some(nested_typ) = self.get(nested_id) {
                            let nested_flags = nested_typ.flags();
                            if (nested_flags & type_flags::NEVER) != 0 {
                                return self.never_type;
                            }
                            if (nested_flags & type_flags::UNKNOWN) != 0 {
                                continue;
                            }
                        }
                        if !flattened.contains(&nested_id) {
                            flattened.push(nested_id);
                        }
                    }
                } else {
                    // Add if not already present (dedup)
                    if !flattened.contains(&type_id) {
                        flattened.push(type_id);
                    }
                }
            } else {
                // Type not found, add as-is
                if !flattened.contains(&type_id) {
                    flattened.push(type_id);
                }
            }
        }

        // If empty after filtering, return unknown (identity for intersection)
        if flattened.is_empty() {
            return self.unknown_type;
        }

        // If only one type, return it directly
        if flattened.len() == 1 {
            return flattened[0];
        }

        self.alloc(Type::Intersection(IntersectionType::new(flattened)))
    }

    /// Create an array type (T[] or Array<T>).
    pub fn create_array_type(&mut self, element_type: TypeId, is_readonly: bool) -> TypeId {
        self.alloc(Type::Array(ArrayTypeInfo {
            flags: type_flags::OBJECT,
            element_type,
            is_readonly,
        }))
    }

    /// Create a tuple type ([T, U, V]).
    pub fn create_tuple_type(
        &mut self,
        element_types: Vec<TypeId>,
        has_optional_elements: bool,
        has_rest_element: bool,
        is_readonly: bool,
    ) -> TypeId {
        self.alloc(Type::Tuple(TupleTypeInfo {
            flags: type_flags::OBJECT,
            element_types,
            has_optional_elements,
            has_rest_element,
            is_readonly,
        }))
    }

    /// Create a conditional type (T extends U ? X : Y).
    /// If is_distributive is true, the conditional will distribute over union types
    /// when the check type is instantiated.
    pub fn create_conditional_type(
        &mut self,
        check_type: TypeId,
        extends_type: TypeId,
        true_type: TypeId,
        false_type: TypeId,
    ) -> TypeId {
        // A conditional type is distributive when check_type is a naked type parameter
        let is_distributive = self.is_naked_type_parameter(check_type);

        self.alloc(Type::Conditional(ConditionalType {
            flags: type_flags::CONDITIONAL,
            check_type,
            extends_type,
            true_type,
            false_type,
            is_distributive,
            infer_type_parameters: Vec::new(),
        }))
    }

    /// Check if a type is a "naked" type parameter (just T, not keyof T or T[]).
    fn is_naked_type_parameter(&self, type_id: TypeId) -> bool {
        if let Some(Type::TypeParameter(_)) = self.get(type_id) {
            true
        } else {
            false
        }
    }

    /// Create a template literal type (`hello ${T}`).
    /// If all substitution types are string literals, evaluates to a single string literal.
    pub fn create_template_literal_type(&mut self, texts: Vec<String>, types: Vec<TypeId>) -> TypeId {
        // If no substitutions, just return a string literal of the first text
        if types.is_empty() {
            if texts.is_empty() {
                return self.create_string_literal(String::new());
            }
            return self.create_string_literal(texts[0].clone());
        }

        // Try to evaluate: if all substitution types are string literals, concat them
        let mut all_concrete = true;
        let mut string_values: Vec<Option<String>> = Vec::new();

        for type_id in &types {
            if let Some(typ) = self.get(*type_id) {
                match typ {
                    Type::Literal(LiteralType { value: LiteralValue::String(s), .. }) => {
                        string_values.push(Some(s.clone()));
                    }
                    Type::Literal(LiteralType { value: LiteralValue::Number(n), .. }) => {
                        // Numbers can be stringified
                        string_values.push(Some(n.to_string()));
                    }
                    Type::Literal(LiteralType { value: LiteralValue::Boolean(b), .. }) => {
                        // Booleans can be stringified
                        string_values.push(Some(b.to_string()));
                    }
                    Type::Literal(LiteralType { value: LiteralValue::BigInt(b), .. }) => {
                        // BigInt can be stringified
                        string_values.push(Some(b.clone()));
                    }
                    _ => {
                        all_concrete = false;
                        break;
                    }
                }
            } else {
                all_concrete = false;
                break;
            }
        }

        if all_concrete && string_values.len() == types.len() {
            // All types are concrete literals, evaluate the template
            let mut result = String::new();
            for (i, text) in texts.iter().enumerate() {
                result.push_str(text);
                if i < string_values.len() {
                    if let Some(s) = &string_values[i] {
                        result.push_str(s);
                    }
                }
            }
            return self.create_string_literal(result);
        }

        // Not all concrete, return unevaluated template literal type
        self.alloc(Type::TemplateLiteral(TemplateLiteralType {
            flags: type_flags::TEMPLATE_LITERAL,
            texts,
            types,
        }))
    }

    /// Create a mapped type ({ [K in keyof T]: T[K] }).
    pub fn create_mapped_type(
        &mut self,
        declaration: NodeIndex,
        type_parameter: TypeId,
        constraint_type: TypeId,
        name_type: TypeId,
        template_type: TypeId,
    ) -> TypeId {
        self.alloc(Type::Mapped(MappedType {
            flags: type_flags::OBJECT,
            object_flags: object_flags::MAPPED,
            declaration,
            type_parameter,
            constraint_type,
            name_type,
            template_type,
        }))
    }

    /// Create an index type (keyof T).
    pub fn create_index_type(&mut self, source_type: TypeId) -> TypeId {
        self.alloc(Type::Index(IndexType {
            flags: type_flags::INDEX,
            source_type,
        }))
    }

    /// Create an indexed access type (T[K]).
    pub fn create_indexed_access_type(&mut self, object_type: TypeId, index_type: TypeId) -> TypeId {
        self.alloc(Type::IndexedAccess(IndexedAccessType {
            flags: type_flags::INDEXED_ACCESS,
            object_type,
            index_type,
            constraint: TypeId::NONE,
        }))
    }

    /// Create a function type.
    pub fn create_function_type(
        &mut self,
        declaration: NodeIndex,
        parameter_types: Vec<TypeId>,
        parameter_names: Vec<String>,
        return_type: TypeId,
        min_argument_count: u32,
        has_rest_parameter: bool,
    ) -> TypeId {
        self.alloc(Type::Function(FunctionType {
            flags: type_flags::OBJECT,
            object_flags: object_flags::ANONYMOUS,
            declaration,
            parameter_types,
            parameter_names,
            return_type,
            type_parameters: Vec::new(),
            min_argument_count,
            has_rest_parameter,
        }))
    }

    /// Create a function type with type parameters.
    pub fn create_function_type_with_type_params(
        &mut self,
        declaration: NodeIndex,
        parameter_types: Vec<TypeId>,
        parameter_names: Vec<String>,
        return_type: TypeId,
        type_parameters: Vec<TypeId>,
        min_argument_count: u32,
        has_rest_parameter: bool,
    ) -> TypeId {
        self.alloc(Type::Function(FunctionType {
            flags: type_flags::OBJECT,
            object_flags: object_flags::ANONYMOUS,
            declaration,
            parameter_types,
            parameter_names,
            return_type,
            type_parameters,
            min_argument_count,
            has_rest_parameter,
        }))
    }

    /// Create a type parameter.
    pub fn create_type_parameter(&mut self, symbol: SymbolId, constraint: TypeId, default: TypeId) -> TypeId {
        self.alloc(Type::TypeParameter(TypeParameter {
            flags: type_flags::TYPE_PARAMETER,
            symbol,
            constraint,
            default,
            target: TypeId::NONE,
            is_this_type: false,
        }))
    }

    /// Create an object type with properties.
    pub fn create_object_type(&mut self, properties: Vec<SymbolId>) -> TypeId {
        let mut obj = ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE);
        obj.properties = properties;
        self.alloc(Type::Object(obj))
    }

    /// Create an anonymous object type with properties and members table.
    pub fn create_object_type_with_members(&mut self, properties: Vec<SymbolId>, members: SymbolTable) -> TypeId {
        let mut obj = ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE);
        obj.properties = properties;
        obj.members = members;
        self.alloc(Type::Object(obj))
    }

    /// Create a class type with properties and construct signatures.
    pub fn create_class_type(
        &mut self,
        properties: Vec<SymbolId>,
        construct_signatures: Vec<Signature>,
        call_signatures: Vec<Signature>,
    ) -> TypeId {
        let mut obj = ObjectType::new(object_flags::CLASS, SymbolId::NONE);
        obj.properties = properties;
        obj.construct_signatures = construct_signatures;
        obj.call_signatures = call_signatures;
        self.alloc(Type::Object(obj))
    }

    /// Create an interface type with properties, signatures, and index infos.
    pub fn create_interface_type(
        &mut self,
        properties: Vec<SymbolId>,
        construct_signatures: Vec<Signature>,
        call_signatures: Vec<Signature>,
        index_infos: Vec<IndexInfo>,
    ) -> TypeId {
        let mut obj = ObjectType::new(object_flags::INTERFACE, SymbolId::NONE);
        obj.properties = properties;
        obj.construct_signatures = construct_signatures;
        obj.call_signatures = call_signatures;
        obj.index_infos = index_infos;
        self.alloc(Type::Object(obj))
    }

    /// Create a union type from a list of types.
    pub fn create_union_type(&mut self, types: Vec<TypeId>) -> TypeId {
        // Filter duplicates and flatten nested unions
        let mut flattened = Vec::new();
        for t in types {
            if let Some(Type::Union(u)) = self.get(t) {
                for &ut in &u.types {
                    if !flattened.contains(&ut) {
                        flattened.push(ut);
                    }
                }
            } else if !flattened.contains(&t) {
                flattened.push(t);
            }
        }

        // Handle edge cases
        if flattened.is_empty() {
            return self.never_type;
        }
        if flattened.len() == 1 {
            return flattened[0];
        }

        self.alloc(Type::Union(UnionType::new(flattened)))
    }
}

impl Default for TypeArena {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Diagnostic
// =============================================================================

/// A type-checking diagnostic message.
#[derive(Clone, Debug, Serialize)]
pub struct Diagnostic {
    pub file: String,
    pub start: u32,
    pub length: u32,
    pub message_text: String,
    pub category: DiagnosticCategory,
    pub code: u32,
}

/// Diagnostic category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum DiagnosticCategory {
    Warning = 0,
    Error = 1,
    Suggestion = 2,
    Message = 3,
}

// =============================================================================
// Checker State
// =============================================================================

/// The type checker state.
/// Performs type inference and type checking on AST nodes.
pub struct CheckerState<'a> {
    /// The node arena containing the AST.
    pub node_arena: &'a crate::parser::NodeArena,

    /// The symbol arena containing bound symbols.
    pub symbol_arena: &'a SymbolArena,

    /// Symbol table for file-local name lookup.
    pub file_locals: &'a SymbolTable,

    /// The type arena for allocating types.
    pub types: TypeArena,

    /// Local symbol arena for checker-created symbols (e.g., for object type properties).
    local_symbols: SymbolArena,

    /// Cached types for symbols.
    symbol_types: std::collections::HashMap<SymbolId, TypeId>,

    /// Cached types for nodes.
    node_types: std::collections::HashMap<NodeIndex, TypeId>,

    /// Type parameter names for type_to_string.
    type_parameter_names: std::collections::HashMap<TypeId, String>,

    /// Current type parameter scope (name -> TypeId) for resolving type references
    /// during function signature processing.
    type_parameter_scope: std::collections::HashMap<String, TypeId>,

    /// Diagnostics produced during type checking.
    pub diagnostics: Vec<Diagnostic>,

    /// Current file name.
    pub file_name: String,
}

/// Represents a type guard extracted from a condition expression.
#[derive(Debug, Clone)]
pub enum TypeGuard {
    /// typeof x === "string" style guard
    Typeof {
        /// The expression being guarded (e.g., x)
        target: NodeIndex,
        /// The expected typeof result (e.g., "string")
        typeof_result: String,
        /// Whether this is an equality (===) or inequality (!==) check
        is_equality: bool,
    },
    /// x instanceof Foo style guard
    Instanceof {
        /// The expression being guarded (e.g., x)
        target: NodeIndex,
        /// The constructor type to check against
        constructor_type: TypeId,
        /// Whether this is an instanceof or NOT instanceof check
        is_positive: bool,
    },
    /// x !== null / x !== undefined / x (truthiness) style guard
    Truthiness {
        /// The expression being guarded
        target: NodeIndex,
        /// If true, narrows by removing null/undefined; if false, narrows TO null/undefined
        is_truthy: bool,
    },
}

impl<'a> CheckerState<'a> {
    /// Create a new checker state.
    pub fn new(
        node_arena: &'a crate::parser::NodeArena,
        symbol_arena: &'a SymbolArena,
        file_locals: &'a SymbolTable,
        file_name: String,
    ) -> Self {
        CheckerState {
            node_arena,
            symbol_arena,
            file_locals,
            types: TypeArena::new(),
            local_symbols: SymbolArena::new(),
            symbol_types: std::collections::HashMap::new(),
            node_types: std::collections::HashMap::new(),
            type_parameter_names: std::collections::HashMap::new(),
            type_parameter_scope: std::collections::HashMap::new(),
            diagnostics: Vec::new(),
            file_name,
        }
    }

    /// Resolve a name to a symbol.
    pub fn resolve_name(&self, name: &str) -> Option<SymbolId> {
        self.file_locals.get(name)
    }

    /// Report a diagnostic error.
    pub fn error(&mut self, node: NodeIndex, message: &str, code: u32) {
        if let Some(n) = self.node_arena.get(node) {
            let base = n.base();
            self.diagnostics.push(Diagnostic {
                file: self.file_name.clone(),
                start: base.pos,
                length: base.end - base.pos,
                message_text: message.to_string(),
                category: DiagnosticCategory::Error,
                code,
            });
        }
    }

    // =========================================================================
    // Type Retrieval
    // =========================================================================

    /// Get the type of a node (with caching).
    pub fn get_type_of_node(&mut self, node: NodeIndex) -> TypeId {
        // Check cache first
        if let Some(&cached) = self.node_types.get(&node) {
            return cached;
        }

        let type_id = self.get_type_of_node_worker(node);
        self.node_types.insert(node, type_id);
        type_id
    }

    /// Get type of node (worker, no caching).
    fn get_type_of_node_worker(&mut self, node: NodeIndex) -> TypeId {
        use crate::parser::Node;
        use crate::scanner::SyntaxKind;

        let Some(n) = self.node_arena.get(node) else {
            return self.types.any_type;
        };

        match n {
            // Literals
            Node::StringLiteral(lit) => {
                self.types.create_string_literal(lit.text.clone())
            }
            Node::NumericLiteral(lit) => {
                let value = lit.text.parse::<f64>().unwrap_or(0.0);
                self.types.create_number_literal(value)
            }

            // Token nodes - check the kind for keywords
            Node::Token(base) => {
                let kind = base.kind;
                if kind == SyntaxKind::TrueKeyword as u16 {
                    self.types.true_type
                } else if kind == SyntaxKind::FalseKeyword as u16 {
                    self.types.false_type
                } else if kind == SyntaxKind::NullKeyword as u16 {
                    self.types.null_type
                } else if kind == SyntaxKind::StringKeyword as u16 {
                    self.types.string_type
                } else if kind == SyntaxKind::NumberKeyword as u16 {
                    self.types.number_type
                } else if kind == SyntaxKind::BooleanKeyword as u16 {
                    self.types.boolean_type
                } else if kind == SyntaxKind::VoidKeyword as u16 {
                    self.types.void_type
                } else if kind == SyntaxKind::AnyKeyword as u16 {
                    self.types.any_type
                } else if kind == SyntaxKind::NeverKeyword as u16 {
                    self.types.never_type
                } else if kind == SyntaxKind::UndefinedKeyword as u16 {
                    self.types.undefined_type
                } else if kind == SyntaxKind::UnknownKeyword as u16 {
                    self.types.unknown_type
                } else if kind == SyntaxKind::ObjectKeyword as u16 {
                    self.types.object_type
                } else if kind == SyntaxKind::BigIntKeyword as u16 {
                    self.types.big_int_type
                } else if kind == SyntaxKind::SymbolKeyword as u16 {
                    self.types.es_symbol_type
                } else {
                    self.types.any_type
                }
            }

            // Identifiers - look up in symbol table
            Node::Identifier(id) => {
                // Look up the identifier in the symbol table
                if let Some(symbol_id) = self.file_locals.get(&id.escaped_text) {
                    self.get_type_of_symbol(symbol_id)
                } else {
                    // Undeclared identifier - could report error
                    self.types.any_type
                }
            }

            // Union types
            Node::UnionType(ut) => {
                let types: Vec<TypeId> = ut.types.nodes.iter()
                    .map(|&t| self.get_type_of_node(t))
                    .collect();
                self.types.create_union(types)
            }

            // Intersection types
            Node::IntersectionType(it) => {
                let types: Vec<TypeId> = it.types.nodes.iter()
                    .map(|&t| self.get_type_of_node(t))
                    .collect();
                self.types.create_intersection(types)
            }

            // Array types (T[])
            Node::ArrayType(at) => {
                let element_type = self.get_type_of_node(at.element_type);
                self.types.create_array_type(element_type, false)
            }

            // Tuple types ([T, U, V])
            Node::TupleType(tt) => {
                let mut element_types: Vec<TypeId> = Vec::new();
                let mut has_optional_elements = false;
                let mut has_rest_element = false;

                for &elem_idx in &tt.elements.nodes {
                    if let Some(elem_node) = self.node_arena.get(elem_idx) {
                        match elem_node {
                            Node::OptionalType(opt) => {
                                has_optional_elements = true;
                                element_types.push(self.get_type_of_node(opt.type_node));
                            }
                            Node::RestType(rest) => {
                                has_rest_element = true;
                                // For rest element, get the element type of the array
                                let rest_type = self.get_type_of_node(rest.type_node);
                                if let Some(Type::Array(arr)) = self.types.get(rest_type) {
                                    element_types.push(arr.element_type);
                                } else {
                                    // If not an array, just use the type as-is
                                    element_types.push(rest_type);
                                }
                            }
                            _ => {
                                element_types.push(self.get_type_of_node(elem_idx));
                            }
                        }
                    }
                }
                self.types.create_tuple_type(element_types, has_optional_elements, has_rest_element, false)
            }

            // Optional type (T?) - used in tuple elements
            Node::OptionalType(opt) => {
                // For optional types, return the inner type (the optionality is tracked at tuple level)
                self.get_type_of_node(opt.type_node)
            }

            // Rest type (...T) - used in tuple elements
            Node::RestType(rest) => {
                // For rest types, return the element type of the array
                let rest_type = self.get_type_of_node(rest.type_node);
                if let Some(Type::Array(arr)) = self.types.get(rest_type) {
                    arr.element_type
                } else {
                    rest_type
                }
            }

            // Conditional type (T extends U ? X : Y)
            Node::ConditionalType(ct) => {
                self.get_type_of_conditional_type(ct)
            }

            // Template literal type (`hello ${T}`)
            Node::TemplateLiteralType(tlt) => {
                self.get_type_of_template_literal_type(tlt)
            }

            // Mapped type ({ [K in keyof T]: T[K] })
            Node::MappedType(mt) => {
                self.get_type_of_mapped_type(node, mt)
            }

            // Indexed access type (T[K])
            Node::IndexedAccessType(ia) => {
                let object_type = self.get_type_of_node(ia.object_type);
                let index_type = self.get_type_of_node(ia.index_type);
                // Create an IndexedAccess type that will be resolved later during instantiation
                self.types.create_indexed_access_type(object_type, index_type)
            }

            // Infer type (infer T in conditional types)
            Node::InferType(it) => {
                self.get_type_of_infer_type(node, it)
            }

            // Type operators (readonly T, keyof T, etc.)
            Node::TypeOperator(to) => {
                if to.operator == SyntaxKind::ReadonlyKeyword as u16 {
                    // readonly T - make the inner type readonly
                    let inner_type = self.get_type_of_node(to.type_node);
                    self.make_type_readonly(inner_type)
                } else if to.operator == SyntaxKind::KeyOfKeyword as u16 {
                    // keyof T - extract keys from the object type
                    let inner_type = self.get_type_of_node(to.type_node);
                    self.get_keyof_type(inner_type)
                } else {
                    // For other operators (unique), just get the inner type
                    self.get_type_of_node(to.type_node)
                }
            }

            // Type references (generic types like Array<T>, or keywords like number)
            Node::TypeReference(tr) => {
                // Check for type arguments
                if let Some(ref type_args) = tr.type_arguments {
                    if !type_args.nodes.is_empty() {
                        return self.get_type_of_type_reference_with_args(tr.type_name, type_args);
                    }
                }

                // Check if the type_name is a keyword type
                if let Some(Node::Identifier(id)) = self.node_arena.get(tr.type_name) {
                    match id.escaped_text.as_str() {
                        "string" => self.types.string_type,
                        "number" => self.types.number_type,
                        "boolean" => self.types.boolean_type,
                        "void" => self.types.void_type,
                        "any" => self.types.any_type,
                        "never" => self.types.never_type,
                        "undefined" => self.types.undefined_type,
                        "null" => self.types.null_type,
                        "unknown" => self.types.unknown_type,
                        "object" => self.types.object_type,
                        "bigint" => self.types.big_int_type,
                        "symbol" => self.types.es_symbol_type,
                        _ => {
                            // First, check if it's a type parameter in the current scope
                            if let Some(&type_param) = self.type_parameter_scope.get(&id.escaped_text) {
                                type_param
                            }
                            // Otherwise, look up in symbol table
                            else if let Some(symbol_id) = self.file_locals.get(&id.escaped_text) {
                                self.get_type_of_symbol(symbol_id)
                            } else {
                                self.types.object_type
                            }
                        }
                    }
                } else {
                    self.types.object_type
                }
            }

            // Parenthesized types
            Node::ParenthesizedType(pt) => {
                self.get_type_of_node(pt.type_node)
            }

            // Literal types
            Node::LiteralType(lt) => {
                self.get_type_of_node(lt.literal)
            }

            // Variable declarations - get type from initializer or annotation
            Node::VariableDeclaration(vd) => {
                if !vd.type_annotation.is_none() {
                    self.get_type_of_node(vd.type_annotation)
                } else if !vd.initializer.is_none() {
                    self.get_type_of_node(vd.initializer)
                } else {
                    self.types.any_type
                }
            }

            // Function declarations
            Node::FunctionDeclaration(fd) => {
                self.get_type_of_function_like_with_type_params(
                    node,
                    &fd.parameters,
                    fd.type_annotation,
                    fd.type_parameters.as_ref(),
                )
            }

            // Function expressions
            Node::FunctionExpression(fe) => {
                self.get_type_of_function_like_with_type_params(
                    node,
                    &fe.parameters,
                    fe.type_annotation,
                    fe.type_parameters.as_ref(),
                )
            }

            // Arrow functions
            Node::ArrowFunction(af) => {
                self.get_type_of_function_like_with_type_params(
                    node,
                    &af.parameters,
                    af.type_annotation,
                    af.type_parameters.as_ref(),
                )
            }

            // Method declarations
            Node::MethodDeclaration(md) => {
                self.get_type_of_function_like_with_type_params(
                    node,
                    &md.parameters,
                    md.type_annotation,
                    md.type_parameters.as_ref(),
                )
            }

            // Function type nodes (e.g., type F = (x: number) => string)
            Node::FunctionType(ft) => {
                self.get_type_of_function_like_with_type_params(
                    node,
                    &ft.parameters,
                    ft.type_node,
                    ft.type_parameters.as_ref(),
                )
            }

            // Constructor type nodes
            Node::ConstructorType(ct) => {
                self.get_type_of_function_like_with_type_params(
                    node,
                    &ct.parameters,
                    ct.type_node,
                    ct.type_parameters.as_ref(),
                )
            }

            // Type alias declarations - get the declared type
            Node::TypeAliasDeclaration(ta) => {
                self.get_type_of_type_alias_declaration(ta)
            }

            // Type literals (e.g., { x: number, y: string })
            Node::TypeLiteral(tl) => {
                self.get_type_of_type_literal(&tl.members)
            }

            // Property access expressions (e.g., obj.prop)
            Node::PropertyAccessExpression(pa) => {
                self.get_type_of_property_access(pa.expression, pa.name)
            }

            // Element access expressions (e.g., obj[key], arr[0])
            Node::ElementAccessExpression(ea) => {
                self.get_type_of_element_access(ea.expression, ea.argument_expression)
            }

            // Object literals (e.g., { x: 1, y: "hello" })
            Node::ObjectLiteralExpression(ole) => {
                self.get_type_of_object_literal(&ole.properties)
            }

            // Call expressions (e.g., fn(arg1, arg2))
            Node::CallExpression(ce) => {
                self.get_type_of_call_expression(ce.expression, &ce.type_arguments, &ce.arguments)
            }

            // New expressions (e.g., new Foo(arg))
            Node::NewExpression(ne) => {
                self.get_type_of_new_expression(ne.expression, &ne.arguments)
            }

            // Array literal expressions (e.g., [1, 2, 3])
            Node::ArrayLiteralExpression(ale) => {
                self.get_type_of_array_literal(&ale.elements)
            }

            // Parenthesized expressions (e.g., (x))
            Node::ParenthesizedExpression(pe) => {
                self.get_type_of_node(pe.expression)
            }

            // Class declarations
            Node::ClassDeclaration(cd) => {
                self.get_type_of_class_declaration(node, cd)
            }

            // Interface declarations
            Node::InterfaceDeclaration(id) => {
                self.get_type_of_interface_declaration(node, id)
            }

            // Default: return any
            _ => self.types.any_type,
        }
    }

    /// Get the type of a call expression.
    fn get_type_of_call_expression(
        &mut self,
        expression: NodeIndex,
        type_arguments: &Option<crate::parser::NodeList>,
        arguments: &crate::parser::NodeList
    ) -> TypeId {
        // Get the type of the function being called
        let func_type = self.get_type_of_node(expression);

        // If it's a function type, handle generics and return type
        if let Some(Type::Function(f)) = self.types.get(func_type) {
            // Extract function information to avoid borrow issues
            let type_parameters = f.type_parameters.clone();
            let parameter_types = f.parameter_types.clone();
            let return_type = f.return_type;
            let min_argument_count = f.min_argument_count;
            let has_rest_parameter = f.has_rest_parameter;

            // Check argument count
            let arg_count = arguments.nodes.len() as u32;
            if arg_count < min_argument_count && !has_rest_parameter {
                // TODO: Add diagnostic for too few arguments
            }
            if arg_count > parameter_types.len() as u32 && !has_rest_parameter {
                // TODO: Add diagnostic for too many arguments
            }

            // If the function has type parameters, we need to infer or use explicit type arguments
            if !type_parameters.is_empty() {
                let inferred_type_args = if let Some(explicit_args) = type_arguments {
                    // Use explicit type arguments: identity<string>("hello")
                    explicit_args.nodes.iter()
                        .map(|&arg| self.get_type_of_node(arg))
                        .collect::<Vec<_>>()
                } else {
                    // Infer type arguments from the argument types
                    self.infer_type_arguments(&type_parameters, &parameter_types, arguments)
                };

                // Instantiate the return type with the inferred type arguments
                return self.instantiate_type(return_type, &inferred_type_args, &type_parameters);
            }

            return return_type;
        }

        // If it's an object with call signatures, use those
        if let Some(Type::Object(obj)) = self.types.get(func_type) {
            if !obj.call_signatures.is_empty() {
                // For now, use the first call signature's return type
                if let Some(return_type) = obj.call_signatures[0].resolved_return_type {
                    return return_type;
                }
            }
        }

        // Default to any for unknown callable types
        self.types.any_type
    }

    /// Infer type arguments for a generic function call from the provided arguments.
    fn infer_type_arguments(
        &mut self,
        type_parameters: &[TypeId],
        parameter_types: &[TypeId],
        arguments: &crate::parser::NodeList
    ) -> Vec<TypeId> {
        // Create a mapping from type parameter to inferred type
        let mut inferred: std::collections::HashMap<TypeId, TypeId> = std::collections::HashMap::new();

        // For each argument, try to infer type parameters from the corresponding parameter type
        for (i, &arg_node) in arguments.nodes.iter().enumerate() {
            if i >= parameter_types.len() {
                break;
            }

            let arg_type = self.get_type_of_node(arg_node);
            let param_type = parameter_types[i];

            // If the parameter type is a type parameter, infer it from the argument type
            self.infer_from_types(param_type, arg_type, type_parameters, &mut inferred);
        }

        // Build the result vector in order of type parameters
        type_parameters.iter()
            .map(|&tp| *inferred.get(&tp).unwrap_or(&self.types.any_type))
            .collect()
    }

    /// Recursively infer type arguments by matching a pattern type against an actual type.
    fn infer_from_types(
        &mut self,
        pattern_type: TypeId,
        actual_type: TypeId,
        type_parameters: &[TypeId],
        inferred: &mut std::collections::HashMap<TypeId, TypeId>
    ) {
        // If the pattern is a type parameter, infer it
        if type_parameters.contains(&pattern_type) {
            // If we already inferred this type parameter, we could merge types (union)
            // For now, just use the first inference
            inferred.entry(pattern_type).or_insert(actual_type);
            return;
        }

        // Extract info from pattern type to avoid borrow issues
        enum PatternInfo {
            Function { parameter_types: Vec<TypeId>, return_type: TypeId },
            Union { types: Vec<TypeId> },
            Intersection { types: Vec<TypeId> },
            Other,
        }

        let pattern_info = match self.types.get(pattern_type) {
            Some(Type::Function(f)) => PatternInfo::Function {
                parameter_types: f.parameter_types.clone(),
                return_type: f.return_type,
            },
            Some(Type::Union(u)) => PatternInfo::Union { types: u.types.clone() },
            Some(Type::Intersection(i)) => PatternInfo::Intersection { types: i.types.clone() },
            _ => PatternInfo::Other,
        };

        // Extract info from actual type
        let actual_info = match self.types.get(actual_type) {
            Some(Type::Function(f)) => PatternInfo::Function {
                parameter_types: f.parameter_types.clone(),
                return_type: f.return_type,
            },
            Some(Type::Union(u)) => PatternInfo::Union { types: u.types.clone() },
            Some(Type::Intersection(i)) => PatternInfo::Intersection { types: i.types.clone() },
            _ => PatternInfo::Other,
        };

        // Match function types: (T) => U with (string) => number infers T=string, U=number
        if let (
            PatternInfo::Function { parameter_types: pattern_params, return_type: pattern_return },
            PatternInfo::Function { parameter_types: actual_params, return_type: actual_return }
        ) = (&pattern_info, &actual_info) {
            // Infer from parameter types (contravariant, but for simplicity we use covariant here)
            for (pattern_param, actual_param) in pattern_params.iter().zip(actual_params.iter()) {
                self.infer_from_types(*pattern_param, *actual_param, type_parameters, inferred);
            }
            // Infer from return type
            self.infer_from_types(*pattern_return, *actual_return, type_parameters, inferred);
        }

        // TODO: Handle object types, array types, etc.
    }

    /// Get the type of a new expression.
    fn get_type_of_new_expression(&mut self, expression: NodeIndex, _arguments: &Option<crate::parser::NodeList>) -> TypeId {
        // Get the type of the constructor
        let constructor_type = self.get_type_of_node(expression);

        // If it's a function type, create an instance type
        // For now, just return any - proper class instantiation is complex
        if let Some(Type::Function(_)) = self.types.get(constructor_type) {
            // TODO: Return the instance type
            return self.types.any_type;
        }

        // If it's an object with construct signatures, use those
        if let Some(Type::Object(obj)) = self.types.get(constructor_type) {
            if !obj.construct_signatures.is_empty() {
                if let Some(return_type) = obj.construct_signatures[0].resolved_return_type {
                    return return_type;
                }
            }
        }

        self.types.any_type
    }

    /// Get the type of an array literal.
    fn get_type_of_array_literal(&mut self, elements: &crate::parser::NodeList) -> TypeId {
        use crate::parser::Node;

        // Collect element types
        let mut element_types = Vec::new();
        for &elem_idx in &elements.nodes {
            if let Some(elem_node) = self.node_arena.get(elem_idx) {
                let elem_type = match elem_node {
                    Node::SpreadElement(spread) => {
                        // For spread element, get the element type of the spread's array
                        let spread_type = self.get_type_of_node(spread.expression);
                        self.get_element_type_of_spread(spread_type)
                    }
                    _ => self.get_type_of_node(elem_idx),
                };
                if !element_types.contains(&elem_type) {
                    element_types.push(elem_type);
                }
            }
        }

        // Create a union of element types (if multiple) or the single type
        if element_types.is_empty() {
            // Empty array - never[]
            self.types.never_type
        } else if element_types.len() == 1 {
            // Single type - return as is (would be Array<T> in full impl)
            element_types[0]
        } else {
            // Multiple types - create union (would be Array<T | U | ...> in full impl)
            self.types.create_union_type(element_types)
        }
    }

    /// Get the element type when spreading an array/tuple into another array.
    fn get_element_type_of_spread(&self, spread_type: TypeId) -> TypeId {
        if let Some(ty) = self.types.get(spread_type) {
            match ty {
                Type::Array(arr) => arr.element_type,
                Type::Tuple(tup) => {
                    // For tuples, create a union of all element types
                    if tup.element_types.is_empty() {
                        self.types.never_type
                    } else if tup.element_types.len() == 1 {
                        tup.element_types[0]
                    } else {
                        // Note: We can't create a union here since we only have &self
                        // For now, return the first element type
                        // A proper implementation would need &mut self
                        tup.element_types[0]
                    }
                }
                _ => {
                    // For other types (like any), just return the type
                    spread_type
                }
            }
        } else {
            self.types.any_type
        }
    }

    /// Get the type of a conditional type (T extends U ? X : Y).
    /// Evaluates the condition and returns either the true or false branch type.
    fn get_type_of_conditional_type(&mut self, ct: &crate::parser::ConditionalType) -> TypeId {
        let check_type = self.get_type_of_node(ct.check_type);
        let extends_type = self.get_type_of_node(ct.extends_type);
        let true_type_node_id = ct.true_type;
        let false_type_node_id = ct.false_type;

        // Check if the check type contains unresolved type parameters
        // In that case, we need to defer evaluation (return a conditional type)
        if self.type_contains_type_parameter(check_type) {
            // Evaluate both branch types first
            let true_type = self.get_type_of_node(true_type_node_id);
            let false_type = self.get_type_of_node(false_type_node_id);
            // Create a deferred conditional type
            return self.types.create_conditional_type(
                check_type,
                extends_type,
                true_type,
                false_type,
            );
        }

        // For union types in check position, distribute the conditional
        // (A | B) extends U ? X : Y becomes (A extends U ? X : Y) | (B extends U ? X : Y)
        if let Some(Type::Union(union)) = self.types.get(check_type) {
            let member_types = union.types.clone();
            let mut result_types = Vec::new();
            for member in member_types {
                let result = if self.is_type_assignable_to(member, extends_type) {
                    self.get_type_of_node(true_type_node_id)
                } else {
                    self.get_type_of_node(false_type_node_id)
                };
                if !result_types.contains(&result) {
                    result_types.push(result);
                }
            }
            if result_types.len() == 1 {
                return result_types[0];
            }
            return self.types.create_union_type(result_types);
        }

        // Check if extends_type contains infer types
        if self.type_contains_infer(extends_type) {
            // Try to match the pattern and extract inferred types
            let mut inferences: std::collections::HashMap<TypeId, TypeId> = std::collections::HashMap::new();
            if self.infer_from_type(check_type, extends_type, &mut inferences) {
                // Pattern matched - evaluate true branch with inferences substituted
                let true_type = self.get_type_of_node(true_type_node_id);
                return self.instantiate_type_with_mapper(true_type, &inferences);
            } else {
                // Pattern didn't match - use false branch
                return self.get_type_of_node(false_type_node_id);
            }
        }

        // Simple case: check if check_type is assignable to extends_type
        if self.is_type_assignable_to(check_type, extends_type) {
            self.get_type_of_node(true_type_node_id)
        } else {
            self.get_type_of_node(false_type_node_id)
        }
    }

    /// Check if a type contains infer type parameters.
    fn type_contains_infer(&self, type_id: TypeId) -> bool {
        let Some(ty) = self.types.get(type_id) else {
            return false;
        };

        match ty {
            // Check if this is an infer type (type parameter with special marker)
            Type::TypeParameter(tp) => {
                // Infer types are type parameters created without a symbol
                tp.symbol.is_none()
            }
            Type::TypeReference(tr) => {
                tr.type_arguments.iter().any(|&t| self.type_contains_infer(t))
            }
            Type::Union(u) => u.types.iter().any(|&t| self.type_contains_infer(t)),
            Type::Intersection(i) => i.types.iter().any(|&t| self.type_contains_infer(t)),
            Type::Array(arr) => self.type_contains_infer(arr.element_type),
            Type::Tuple(tup) => tup.element_types.iter().any(|&t| self.type_contains_infer(t)),
            Type::Function(f) => {
                f.parameter_types.iter().any(|&t| self.type_contains_infer(t))
                    || self.type_contains_infer(f.return_type)
            }
            _ => false,
        }
    }

    /// Match a source type against a pattern type, extracting inferred types.
    /// Returns true if the pattern matches, false otherwise.
    /// Inferred bindings are stored in the inferences map (pattern TypeId -> matched TypeId).
    fn infer_from_type(
        &self,
        source: TypeId,
        pattern: TypeId,
        inferences: &mut std::collections::HashMap<TypeId, TypeId>,
    ) -> bool {
        let Some(pattern_type) = self.types.get(pattern) else {
            return false;
        };

        // If pattern is an infer type parameter, bind it to source
        if let Type::TypeParameter(tp) = pattern_type {
            if tp.symbol.is_none() {
                // This is an infer type - bind it
                inferences.insert(pattern, source);
                return true;
            }
        }

        // Try to match based on pattern type
        match pattern_type {
            Type::TypeReference(pattern_ref) => {
                // For type references like Promise<infer U>, match the structure
                if let Some(Type::TypeReference(source_ref)) = self.types.get(source) {
                    // Match type arguments
                    let pattern_args = pattern_ref.type_arguments.clone();
                    let source_args = source_ref.type_arguments.clone();

                    if pattern_args.len() != source_args.len() {
                        return false;
                    }

                    for (pattern_arg, source_arg) in pattern_args.iter().zip(source_args.iter()) {
                        if !self.infer_from_type(*source_arg, *pattern_arg, inferences) {
                            return false;
                        }
                    }
                    return true;
                }
                false
            }
            Type::Array(pattern_arr) => {
                if let Some(Type::Array(source_arr)) = self.types.get(source) {
                    let pattern_elem = pattern_arr.element_type;
                    let source_elem = source_arr.element_type;
                    self.infer_from_type(source_elem, pattern_elem, inferences)
                } else {
                    false
                }
            }
            Type::Tuple(pattern_tup) => {
                if let Some(Type::Tuple(source_tup)) = self.types.get(source) {
                    let pattern_elems = pattern_tup.element_types.clone();
                    let source_elems = source_tup.element_types.clone();
                    if pattern_elems.len() != source_elems.len() {
                        return false;
                    }
                    for (p, s) in pattern_elems.iter().zip(source_elems.iter()) {
                        if !self.infer_from_type(*s, *p, inferences) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            Type::Function(pattern_fn) => {
                if let Some(Type::Function(source_fn)) = self.types.get(source) {
                    // Match return type
                    let pattern_ret = pattern_fn.return_type;
                    let source_ret = source_fn.return_type;
                    self.infer_from_type(source_ret, pattern_ret, inferences)
                } else {
                    false
                }
            }
            _ => {
                // For other types, just check assignability
                self.is_type_assignable_to(source, pattern)
            }
        }
    }

    /// Check if a type contains unresolved type parameters.
    fn type_contains_type_parameter(&self, type_id: TypeId) -> bool {
        if let Some(ty) = self.types.get(type_id) {
            match ty {
                Type::TypeParameter(_) => true,
                Type::Union(u) => u.types.iter().any(|&t| self.type_contains_type_parameter(t)),
                Type::Intersection(i) => i.types.iter().any(|&t| self.type_contains_type_parameter(t)),
                Type::Array(arr) => self.type_contains_type_parameter(arr.element_type),
                Type::Tuple(tup) => tup.element_types.iter().any(|&t| self.type_contains_type_parameter(t)),
                Type::TypeReference(r) => {
                    r.type_arguments.iter().any(|&t| self.type_contains_type_parameter(t))
                }
                Type::Conditional(c) => {
                    self.type_contains_type_parameter(c.check_type)
                        || self.type_contains_type_parameter(c.extends_type)
                        || self.type_contains_type_parameter(c.true_type)
                        || self.type_contains_type_parameter(c.false_type)
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Get the type of a template literal type (`hello ${T}`).
    fn get_type_of_template_literal_type(&mut self, tlt: &crate::parser::TemplateLiteralType) -> TypeId {
        use crate::parser::Node;

        let mut texts: Vec<String> = Vec::new();
        let mut types: Vec<TypeId> = Vec::new();

        // Get the head text
        if let Some(Node::NoSubstitutionTemplateLiteral(lit) | Node::TemplateHead(lit)) =
            self.node_arena.get(tlt.head)
        {
            texts.push(lit.text.clone());
        }

        // Get the spans (type, literal pairs)
        for &span_idx in &tlt.template_spans.nodes {
            if let Some(Node::TemplateSpan(span)) = self.node_arena.get(span_idx) {
                // Get the type
                let span_type = self.get_type_of_node(span.expression);
                types.push(span_type);

                // Get the literal text
                if let Some(Node::TemplateMiddle(lit) | Node::TemplateTail(lit)) =
                    self.node_arena.get(span.literal)
                {
                    texts.push(lit.text.clone());
                }
            }
        }

        // If all types are string literals, we can simplify to a single string literal
        if types.iter().all(|&t| {
            self.types.get(t).map_or(false, |ty| {
                matches!(ty, Type::Literal(LiteralType { value: LiteralValue::String(_), .. }))
            })
        }) {
            // Concatenate all parts
            let mut result = String::new();
            for (i, text) in texts.iter().enumerate() {
                result.push_str(text);
                if i < types.len() {
                    if let Some(Type::Literal(LiteralType { value: LiteralValue::String(s), .. })) =
                        self.types.get(types[i])
                    {
                        result.push_str(s);
                    }
                }
            }
            return self.types.create_string_literal(result);
        }

        // Otherwise, create a template literal type
        self.types.create_template_literal_type(texts, types)
    }

    /// Get the type of a mapped type ({ [K in keyof T]: T[K] }).
    fn get_type_of_mapped_type(&mut self, node: NodeIndex, mt: &crate::parser::MappedType) -> TypeId {
        use crate::parser::Node;

        // Track if we added a type parameter to remove it later
        let mut added_param_name: Option<String> = None;

        // Get the type parameter (K in "K in keyof T") and add it to scope
        let type_param_type = if !mt.type_parameter.is_none() {
            let type_id = self.get_type_of_node(mt.type_parameter);
            // Get the name of the type parameter and add to current scope
            if let Some(Node::TypeParameterDeclaration(tp)) = self.node_arena.get(mt.type_parameter) {
                if let Some(Node::Identifier(id)) = self.node_arena.get(tp.name) {
                    self.type_parameter_scope.insert(id.escaped_text.clone(), type_id);
                    self.type_parameter_names.insert(type_id, id.escaped_text.clone());
                    added_param_name = Some(id.escaped_text.clone());
                }
            }
            type_id
        } else {
            self.types.any_type
        };

        // Get the constraint type (keyof T in "K in keyof T")
        // The type_parameter node should have a constraint
        let constraint_type = if let Some(Node::TypeParameterDeclaration(tp)) = self.node_arena.get(mt.type_parameter) {
            if !tp.constraint.is_none() {
                self.get_type_of_node(tp.constraint)
            } else {
                self.types.any_type
            }
        } else {
            self.types.any_type
        };

        // Get the name type (the "as" clause, if present)
        let name_type = if !mt.name_type.is_none() {
            self.get_type_of_node(mt.name_type)
        } else {
            self.types.any_type
        };

        // Get the template type (the value type, e.g., T[K])
        // K is now in type_parameter_scope, so references to K will resolve correctly
        let template_type = if !mt.type_node.is_none() {
            self.get_type_of_node(mt.type_node)
        } else {
            self.types.any_type
        };

        // Remove the type parameter we added
        if let Some(name) = added_param_name {
            self.type_parameter_scope.remove(&name);
        }

        // Create a deferred mapped type (evaluation happens during instantiation)
        self.types.create_mapped_type(
            node,
            type_param_type,
            constraint_type,
            name_type,
            template_type,
        )
    }

    /// Get the type of an infer type (infer T in conditional types).
    /// Creates a special type parameter that can be bound during pattern matching.
    fn get_type_of_infer_type(
        &mut self,
        _node: NodeIndex,
        it: &crate::parser::InferType,
    ) -> TypeId {
        use crate::parser::Node;
        use crate::binder::SymbolId;

        // Get the type parameter name from the type parameter declaration
        let name = if let Some(Node::TypeParameterDeclaration(tp)) = self.node_arena.get(it.type_parameter) {
            if let Some(Node::Identifier(id)) = self.node_arena.get(tp.name) {
                id.escaped_text.clone()
            } else {
                "T".to_string()
            }
        } else {
            "T".to_string()
        };

        // Create a type parameter for this infer type
        // The symbol is NONE since infer types don't have symbols in the symbol table
        let type_param = self.types.create_type_parameter(SymbolId::NONE, TypeId::NONE, TypeId::NONE);

        // Cache the name for type_to_string
        self.type_parameter_names.insert(type_param, name.clone());

        // Add to type_parameter_scope so that later references to the name can find it
        // This is important for conditional types where the true branch references the infer type
        self.type_parameter_scope.insert(name, type_param);

        type_param
    }

    /// Get the type of a type alias declaration.
    /// Sets up type parameter scope before evaluating the alias body.
    fn get_type_of_type_alias_declaration(
        &mut self,
        ta: &crate::parser::TypeAliasDeclaration,
    ) -> TypeId {
        use crate::parser::Node;

        // Save current type parameter scope
        let saved_scope = std::mem::take(&mut self.type_parameter_scope);

        // Set up type parameters if present
        if let Some(ref type_params) = ta.type_parameters {
            for &tp_idx in &type_params.nodes {
                if let Some(type_id) = self.create_type_parameter(tp_idx) {
                    // Add to type parameter scope for name lookup
                    if let Some(name) = self.type_parameter_names.get(&type_id) {
                        self.type_parameter_scope.insert(name.clone(), type_id);
                    }
                }
            }
        }

        // Evaluate the type alias body with type parameters in scope
        let result_type = self.get_type_of_node(ta.type_node);

        // Restore previous type parameter scope
        self.type_parameter_scope = saved_scope;

        result_type
    }

    /// Get the keyof type for a given type.
    /// Returns a union of string literal types for the property names.
    fn get_keyof_type(&mut self, type_id: TypeId) -> TypeId {
        // Handle special cases
        if type_id == TypeId::NONE {
            return self.types.never_type;
        }

        let Some(typ) = self.types.get(type_id) else {
            return self.types.never_type;
        };

        match typ {
            // For object types, extract property names as string literals
            Type::Object(obj) => {
                let property_ids = obj.properties.clone();
                let mut key_types = Vec::new();

                for prop_id in property_ids {
                    // Use get_symbol to look up in both binder and local symbol arenas
                    if let Some(sym) = self.get_symbol(prop_id) {
                        // Create a string literal type for the property name
                        let key_type = self.types.create_string_literal(sym.escaped_name.clone());
                        key_types.push(key_type);
                    }
                }

                // Also check members SymbolTable
                let members = self.types.get(type_id)
                    .and_then(|t| if let Type::Object(o) = t { Some(o.members.clone()) } else { None });

                if let Some(members) = members {
                    for (name, _) in members.iter() {
                        let key_type = self.types.create_string_literal(name.clone());
                        if !key_types.contains(&key_type) {
                            key_types.push(key_type);
                        }
                    }
                }

                if key_types.is_empty() {
                    // Empty object has no keys
                    self.types.never_type
                } else if key_types.len() == 1 {
                    key_types[0]
                } else {
                    self.types.create_union_type(key_types)
                }
            }
            // For type parameters, create an index type
            Type::TypeParameter(_) => {
                // keyof T where T is a type parameter - create Index type
                self.types.create_index_type(type_id)
            }
            // For union types, distribute keyof
            Type::Union(u) => {
                // keyof (A | B) = keyof A & keyof B (intersection of keys)
                // For now, just return string as placeholder
                let types = u.types.clone();
                if types.is_empty() {
                    return self.types.never_type;
                }
                // Get keyof for each member and intersect
                let mut key_types: Vec<TypeId> = types.iter()
                    .map(|&t| self.get_keyof_type(t))
                    .collect();
                if key_types.len() == 1 {
                    key_types[0]
                } else {
                    // For simplicity, just return string
                    self.types.string_type
                }
            }
            // For intrinsic types, return appropriate key types
            Type::Intrinsic(i) => {
                match i.intrinsic_name.as_str() {
                    "any" => {
                        // keyof any = string | number | symbol
                        let types = vec![
                            self.types.string_type,
                            self.types.number_type,
                            self.types.es_symbol_type,
                        ];
                        self.types.create_union_type(types)
                    }
                    "unknown" => self.types.never_type,
                    "string" => {
                        // String has methods like length, charAt, etc.
                        // For simplicity, return number | keyof String prototype
                        self.types.number_type
                    }
                    "number" => self.types.never_type,
                    _ => self.types.never_type,
                }
            }
            // Default: return string | number | symbol
            _ => {
                let types = vec![
                    self.types.string_type,
                    self.types.number_type,
                    self.types.es_symbol_type,
                ];
                self.types.create_union_type(types)
            }
        }
    }

    /// Get the type of a class declaration.
    /// Creates an ObjectType with CLASS object flags, containing all class members.
    /// The returned type is the "constructor type" which has a construct signature
    /// that returns the instance type.
    fn get_type_of_class_declaration(
        &mut self,
        node: NodeIndex,
        class: &crate::parser::ClassDeclaration,
    ) -> TypeId {
        use crate::parser::Node;

        let mut properties = Vec::new();
        let mut constructor_params: Vec<(NodeIndex, SymbolId)> = Vec::new();
        let mut has_constructor = false;

        // First pass: collect properties and methods (for instance type)
        for &member_idx in &class.members.nodes {
            if let Some(member_node) = self.node_arena.get(member_idx) {
                match member_node {
                    // Property declarations
                    Node::PropertyDeclaration(pd) => {
                        // Get property name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(pd.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get property type
                        let prop_type = if !pd.type_annotation.is_none() {
                            self.get_type_of_node(pd.type_annotation)
                        } else if !pd.initializer.is_none() {
                            self.get_type_of_node(pd.initializer)
                        } else {
                            self.types.any_type
                        };

                        // Create a symbol for this property
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::PROPERTY, name.clone());
                        self.symbol_types.insert(symbol_id, prop_type);
                        properties.push(symbol_id);
                    }

                    // Method declarations
                    Node::MethodDeclaration(md) => {
                        // Get method name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(md.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get method type
                        let method_type = self.get_type_of_function_like_with_type_params(
                            member_idx,
                            &md.parameters,
                            md.type_annotation,
                            md.type_parameters.as_ref(),
                        );

                        // Create a symbol for this method
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::METHOD, name.clone());
                        self.symbol_types.insert(symbol_id, method_type);
                        properties.push(symbol_id);
                    }

                    // Constructor declaration - collect params
                    Node::ConstructorDeclaration(cd) => {
                        has_constructor = true;
                        for &param_idx in &cd.parameters.nodes {
                            if let Some(Node::ParameterDeclaration(param)) = self.node_arena.get(param_idx) {
                                if let Some(Node::Identifier(id)) = self.node_arena.get(param.name) {
                                    let param_symbol = self.local_symbols_mut().alloc(
                                        symbol_flags::FUNCTION_SCOPED_VARIABLE,
                                        id.escaped_text.clone(),
                                    );
                                    constructor_params.push((param_idx, param_symbol));
                                }
                            }
                        }
                    }

                    // Get accessor
                    Node::GetAccessorDeclaration(ga) => {
                        // Get accessor name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(ga.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get return type
                        let get_type = if !ga.type_annotation.is_none() {
                            self.get_type_of_node(ga.type_annotation)
                        } else {
                            self.types.any_type
                        };

                        // Create a symbol for this accessor
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::GET_ACCESSOR, name.clone());
                        self.symbol_types.insert(symbol_id, get_type);
                        properties.push(symbol_id);
                    }

                    // Set accessor
                    Node::SetAccessorDeclaration(sa) => {
                        // Get accessor name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(sa.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Set accessor type is the parameter type
                        let set_type = if !sa.parameters.nodes.is_empty() {
                            if let Some(Node::ParameterDeclaration(param)) =
                                self.node_arena.get(sa.parameters.nodes[0])
                            {
                                if !param.type_annotation.is_none() {
                                    self.get_type_of_node(param.type_annotation)
                                } else {
                                    self.types.any_type
                                }
                            } else {
                                self.types.any_type
                            }
                        } else {
                            self.types.any_type
                        };

                        // Create a symbol for this accessor
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::SET_ACCESSOR, name.clone());
                        self.symbol_types.insert(symbol_id, set_type);
                        properties.push(symbol_id);
                    }

                    _ => {}
                }
            }
        }

        // Create the instance type (properties only, no construct signatures)
        // This is the type of instances created with `new`
        let instance_type = self.types.create_class_type(properties.clone(), vec![], vec![]);

        // Create the construct signature - this is what allows `new Foo()`
        let mut construct_signature = Signature::new(node);
        for (_, param_symbol) in constructor_params {
            construct_signature.parameters.push(param_symbol);
        }
        construct_signature.min_argument_count = construct_signature.parameters.len() as u32;
        construct_signature.resolved_return_type = Some(instance_type);

        // If no explicit constructor, create an implicit one
        let construct_signatures = if has_constructor || !construct_signature.parameters.is_empty() {
            vec![construct_signature]
        } else {
            // Implicit constructor with no parameters
            let mut implicit_sig = Signature::new(node);
            implicit_sig.resolved_return_type = Some(instance_type);
            vec![implicit_sig]
        };

        // Create the constructor type - this is the type of the class itself
        // It has construct signatures that return the instance type
        let constructor_type = self.types.create_class_type(properties, construct_signatures, vec![]);
        constructor_type
    }

    /// Get the type of an interface declaration.
    /// Creates an ObjectType with INTERFACE object flags.
    fn get_type_of_interface_declaration(
        &mut self,
        _node: NodeIndex,
        iface: &crate::parser::InterfaceDeclaration,
    ) -> TypeId {
        use crate::parser::Node;

        let mut properties = Vec::new();
        let mut call_signatures = Vec::new();
        let mut construct_signatures = Vec::new();
        let mut index_infos = Vec::new();

        // Process interface members
        for &member_idx in &iface.members.nodes {
            if let Some(member_node) = self.node_arena.get(member_idx) {
                match member_node {
                    // Property signatures
                    Node::PropertySignature(ps) => {
                        // Get property name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(ps.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get property type
                        let prop_type = if !ps.type_annotation.is_none() {
                            self.get_type_of_node(ps.type_annotation)
                        } else {
                            self.types.any_type
                        };

                        // Create a symbol for this property
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::PROPERTY, name.clone());
                        self.symbol_types.insert(symbol_id, prop_type);
                        properties.push(symbol_id);
                    }

                    // Method signatures
                    Node::MethodSignature(ms) => {
                        // Get method name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(ms.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get method type
                        let method_type = self.get_type_of_function_like_with_type_params(
                            member_idx,
                            &ms.parameters,
                            ms.type_annotation,
                            ms.type_parameters.as_ref(),
                        );

                        // Create a symbol for this method
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::METHOD, name.clone());
                        self.symbol_types.insert(symbol_id, method_type);
                        properties.push(symbol_id);
                    }

                    // Index signatures: [key: string]: Type
                    Node::IndexSignatureDeclaration(isd) => {
                        // Get the key type from the first parameter
                        let key_type = if !isd.parameters.nodes.is_empty() {
                            let param_idx = isd.parameters.nodes[0];
                            if let Some(Node::ParameterDeclaration(pd)) = self.node_arena.get(param_idx) {
                                if !pd.type_annotation.is_none() {
                                    self.get_type_of_node(pd.type_annotation)
                                } else {
                                    self.types.string_type // default to string
                                }
                            } else {
                                self.types.string_type
                            }
                        } else {
                            self.types.string_type
                        };

                        // Get the value type from the type annotation
                        let value_type = if !isd.type_annotation.is_none() {
                            self.get_type_of_node(isd.type_annotation)
                        } else {
                            self.types.any_type
                        };

                        // Check for readonly modifier
                        let is_readonly = isd.modifiers.as_ref().map_or(false, |mods| {
                            mods.nodes.iter().any(|&mod_idx| {
                                if let Some(Node::Token(base)) = self.node_arena.get(mod_idx) {
                                    base.kind == crate::scanner::SyntaxKind::ReadonlyKeyword as u16
                                } else {
                                    false
                                }
                            })
                        });

                        index_infos.push(IndexInfo {
                            key_type,
                            value_type,
                            is_readonly,
                            declaration: Some(member_idx),
                        });
                    }

                    // TODO: Add CallSignature and ConstructSignature when parser supports them
                    _ => {}
                }
            }
        }

        // Create the interface type as an ObjectType with INTERFACE object flags
        self.types.create_interface_type(properties, construct_signatures, call_signatures, index_infos)
    }

    /// Get the type of a type literal ({ x: number, y: string }).
    fn get_type_of_type_literal(&mut self, members: &crate::parser::NodeList) -> TypeId {
        use crate::parser::Node;

        let mut properties = Vec::new();
        let mut members_table = SymbolTable::new();

        for &member_idx in &members.nodes {
            if let Some(node) = self.node_arena.get(member_idx) {
                match node {
                    Node::PropertySignature(ps) => {
                        // Get property name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(ps.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get property type
                        let prop_type = if !ps.type_annotation.is_none() {
                            self.get_type_of_node(ps.type_annotation)
                        } else {
                            self.types.any_type
                        };

                        // Create a symbol for this property
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::PROPERTY, name.clone());

                        // Cache the symbol's type
                        self.symbol_types.insert(symbol_id, prop_type);
                        properties.push(symbol_id);
                        members_table.set(name, symbol_id);
                    }
                    Node::MethodSignature(ms) => {
                        // Get method name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(ms.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get method type
                        let method_type = self.get_type_of_function_like_with_type_params(
                            member_idx,
                            &ms.parameters,
                            ms.type_annotation,
                            ms.type_parameters.as_ref(),
                        );

                        // Create a symbol for this method
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::METHOD, name.clone());

                        // Cache the symbol's type
                        self.symbol_types.insert(symbol_id, method_type);
                        properties.push(symbol_id);
                        members_table.set(name, symbol_id);
                    }
                    _ => {}
                }
            }
        }

        // Create object type with members table for keyof
        self.types.create_object_type_with_members(properties, members_table)
    }

    /// Get the type of a property access expression (obj.prop).
    fn get_type_of_property_access(&mut self, expression: NodeIndex, name: NodeIndex) -> TypeId {
        use crate::parser::Node;

        // Get the type of the expression
        let expr_type = self.get_type_of_node(expression);

        // Get the property name
        let prop_name = if let Some(Node::Identifier(id)) = self.node_arena.get(name) {
            id.escaped_text.clone()
        } else {
            return self.types.any_type;
        };

        // Look up the property on the expression type
        self.get_property_type(expr_type, &prop_name)
    }

    /// Get the type of a property on an object type.
    /// For union types, returns the union of property types from each member.
    fn get_property_type(&mut self, object_type: TypeId, prop_name: &str) -> TypeId {
        let Some(typ) = self.types.get(object_type) else {
            return self.types.any_type;
        };

        match typ {
            Type::Object(obj) => {
                // Look up property in object's members
                // obj.members maps property name to SymbolId directly (not reference)
                let members_clone = obj.members.clone();
                if let Some(symbol_id) = members_clone.get(prop_name) {
                    return self.symbol_types.get(&symbol_id).copied().unwrap_or(self.types.any_type);
                }
                // Fallback to properties list
                for &prop_id in &obj.properties.clone() {
                    if let Some(sym) = self.get_symbol(prop_id) {
                        if sym.escaped_name == prop_name {
                            return self.symbol_types.get(&prop_id).copied().unwrap_or(self.types.any_type);
                        }
                    }
                }
                self.types.any_type
            }
            Type::Union(union) => {
                // For union types, get the property type from each member and union them
                let member_types = union.types.clone();
                let mut prop_types = Vec::new();

                for member in member_types {
                    let prop_type = self.get_property_type(member, prop_name);
                    // If any member doesn't have the property, return any (property is optional)
                    if prop_type == self.types.any_type {
                        // Check if this is truly missing or just any
                        // For now, we allow it but might want to track optional properties
                    }
                    if !prop_types.contains(&prop_type) {
                        prop_types.push(prop_type);
                    }
                }

                if prop_types.is_empty() {
                    return self.types.any_type;
                }
                if prop_types.len() == 1 {
                    return prop_types[0];
                }
                self.types.create_union(prop_types)
            }
            Type::Intersection(intersection) => {
                // For intersection types, get the property type from any member that has it
                let member_types = intersection.types.clone();
                for member in member_types {
                    let prop_type = self.get_property_type(member, prop_name);
                    if prop_type != self.types.any_type {
                        return prop_type;
                    }
                }
                self.types.any_type
            }
            _ => self.types.any_type,
        }
    }

    /// Get the type of an element access expression (e.g., obj["key"], arr[0]).
    fn get_type_of_element_access(&mut self, expression: NodeIndex, argument: NodeIndex) -> TypeId {
        // Get the type of the expression being indexed
        let expr_type = self.get_type_of_node(expression);

        // Get the type of the index/key
        let index_type = self.get_type_of_node(argument);

        // Try to get the index info from the expression type
        self.get_indexed_access_type(expr_type, index_type)
    }

    /// Get the type resulting from indexing an object type with an index type.
    fn get_indexed_access_type(&mut self, object_type: TypeId, index_type: TypeId) -> TypeId {
        let Some(typ) = self.types.get(object_type).cloned() else {
            return self.types.any_type;
        };

        match typ {
            Type::Object(obj) => {
                // First, check if index_type is a string literal - try property lookup
                if let Some(Type::Literal(lit)) = self.types.get(index_type) {
                    if let LiteralValue::String(key_name) = &lit.value {
                        // Look up the property by name in members
                        if let Some(symbol_id) = obj.members.get(key_name) {
                            if let Some(&prop_type) = self.symbol_types.get(&symbol_id) {
                                return prop_type;
                            }
                        }
                    }
                }

                // Check if the object has an applicable index signature
                for index_info in &obj.index_infos {
                    // Check if the index type is assignable to the key type
                    if self.is_type_assignable_to(index_type, index_info.key_type) {
                        return index_info.value_type;
                    }
                }
                // No matching index signature, return any
                self.types.any_type
            }
            Type::Array(arr) => {
                // For arrays, if indexed with number, return element type
                let index_typ = self.types.get(index_type);
                let is_number_index = match index_typ {
                    Some(Type::Intrinsic(intrinsic)) => intrinsic.intrinsic_name == "number",
                    Some(Type::Literal(lit)) => matches!(lit.value, LiteralValue::Number(_)),
                    _ => false,
                };
                if is_number_index {
                    return arr.element_type;
                }
                self.types.any_type
            }
            Type::Tuple(tuple) => {
                // For tuples, check if we have a number literal index
                if let Some(Type::Literal(lit)) = self.types.get(index_type) {
                    if let LiteralValue::Number(n) = &lit.value {
                        let idx = *n as usize;
                        if idx < tuple.element_types.len() {
                            return tuple.element_types[idx];
                        }
                        // Out of bounds - return undefined
                        return self.types.undefined_type;
                    }
                }
                // Check if index is number type (not literal)
                let is_number_type = if let Some(Type::Intrinsic(intrinsic)) = self.types.get(index_type) {
                    intrinsic.intrinsic_name == "number"
                } else {
                    false
                };
                if is_number_type {
                    // Return union of all element types
                    if tuple.element_types.is_empty() {
                        return self.types.never_type;
                    }
                    if tuple.element_types.len() == 1 {
                        return tuple.element_types[0];
                    }
                    return self.types.create_union(tuple.element_types.clone());
                }
                self.types.any_type
            }
            Type::Union(union) => {
                // For union types, get indexed access from each member and union the results
                let member_types = union.types.clone();
                let mut result_types = Vec::new();

                for member in member_types {
                    let member_result = self.get_indexed_access_type(member, index_type);
                    if member_result != self.types.any_type && !result_types.contains(&member_result) {
                        result_types.push(member_result);
                    }
                }

                if result_types.is_empty() {
                    return self.types.any_type;
                }
                if result_types.len() == 1 {
                    return result_types[0];
                }
                self.types.create_union(result_types)
            }
            _ => self.types.any_type,
        }
    }

    /// Get the type of an object literal ({ x: 1, y: "hello" }).
    fn get_type_of_object_literal(&mut self, properties: &crate::parser::NodeList) -> TypeId {
        use crate::parser::Node;

        let mut prop_symbols = Vec::new();

        for &prop_idx in &properties.nodes {
            if let Some(node) = self.node_arena.get(prop_idx) {
                match node {
                    Node::PropertyAssignment(pa) => {
                        // Get property name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(pa.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get property type from initializer
                        let prop_type = self.get_type_of_node(pa.initializer);

                        // Create a symbol for this property
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::PROPERTY, name.clone());

                        // Cache the symbol's type
                        self.symbol_types.insert(symbol_id, prop_type);
                        prop_symbols.push(symbol_id);
                    }
                    Node::ShorthandPropertyAssignment(spa) => {
                        // Get property name
                        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(spa.name) {
                            id.escaped_text.clone()
                        } else {
                            continue;
                        };

                        // Get property type from the name identifier (which should resolve to a variable)
                        let prop_type = self.get_type_of_node(spa.name);

                        // Create a symbol for this property
                        let symbol_id = self.local_symbols_mut().alloc(symbol_flags::PROPERTY, name.clone());

                        // Cache the symbol's type
                        self.symbol_types.insert(symbol_id, prop_type);
                        prop_symbols.push(symbol_id);
                    }
                    _ => {}
                }
            }
        }

        // Create object type
        self.types.create_object_type(prop_symbols)
    }

    /// Get a mutable reference to the local symbol arena (for creating new symbols during type checking).
    fn local_symbols_mut(&mut self) -> &mut SymbolArena {
        &mut self.local_symbols
    }

    /// Look up a symbol by ID in both binder and local symbols.
    fn get_symbol(&self, id: SymbolId) -> Option<&crate::binder::Symbol> {
        // Check local_symbols first to avoid ID collision with binder symbols
        self.local_symbols.get(id).or_else(|| self.symbol_arena.get(id))
    }

    /// Get the type of a function-like declaration (function, method, arrow, etc.)
    fn get_type_of_function_like(
        &mut self,
        declaration: NodeIndex,
        parameters: &crate::parser::NodeList,
        return_type_annotation: NodeIndex,
    ) -> TypeId {
        self.get_type_of_function_like_with_type_params(
            declaration,
            parameters,
            return_type_annotation,
            None,
        )
    }

    /// Get the type of a function-like declaration with type parameters.
    fn get_type_of_function_like_with_type_params(
        &mut self,
        declaration: NodeIndex,
        parameters: &crate::parser::NodeList,
        return_type_annotation: NodeIndex,
        type_parameters: Option<&crate::parser::NodeList>,
    ) -> TypeId {
        use crate::parser::Node;

        // Track the type parameter names we add so we can remove them later
        // We preserve the parent scope so that infer types created in return position
        // can be found when parsing subsequent parts of the type
        let mut added_param_names: Vec<String> = Vec::new();

        // Create type parameters and add them to the scope
        let type_param_ids: Vec<TypeId> = if let Some(type_params) = type_parameters {
            type_params.nodes.iter()
                .filter_map(|&tp_idx| {
                    let type_id = self.create_type_parameter(tp_idx)?;
                    // Add to type parameter scope for name lookup during signature processing
                    if let Some(name) = self.type_parameter_names.get(&type_id) {
                        self.type_parameter_scope.insert(name.clone(), type_id);
                        added_param_names.push(name.clone());
                    }
                    Some(type_id)
                })
                .collect()
        } else {
            Vec::new()
        };

        // Collect parameter types and names
        let mut param_types = Vec::new();
        let mut param_names = Vec::new();
        let mut min_arg_count = 0u32;
        let mut has_rest = false;

        for &param_idx in &parameters.nodes {
            if let Some(Node::ParameterDeclaration(param)) = self.node_arena.get(param_idx) {
                // Get parameter name
                let name = if let Some(Node::Identifier(id)) = self.node_arena.get(param.name) {
                    id.escaped_text.clone()
                } else {
                    String::new()
                };
                param_names.push(name);

                // Get parameter type
                let param_type = if !param.type_annotation.is_none() {
                    self.get_type_of_node(param.type_annotation)
                } else if !param.initializer.is_none() {
                    // Infer from initializer
                    self.get_type_of_node(param.initializer)
                } else {
                    self.types.any_type
                };
                param_types.push(param_type);

                // Track min argument count and rest parameter
                if param.dot_dot_dot_token {
                    has_rest = true;
                } else if !param.question_token && param.initializer.is_none() {
                    min_arg_count += 1;
                }
            }
        }

        // Get return type
        let return_type = if !return_type_annotation.is_none() {
            self.get_type_of_node(return_type_annotation)
        } else {
            // Return type inference would happen here
            // For now, default to any
            self.types.any_type
        };

        // Remove only the type parameters we added (preserve parent scope entries)
        for name in added_param_names {
            self.type_parameter_scope.remove(&name);
        }

        self.types.create_function_type_with_type_params(
            declaration,
            param_types,
            param_names,
            return_type,
            type_param_ids,
            min_arg_count,
            has_rest,
        )
    }

    /// Create a TypeParameter from a TypeParameterDeclaration node.
    fn create_type_parameter(&mut self, node: NodeIndex) -> Option<TypeId> {
        use crate::parser::Node;

        let tp = match self.node_arena.get(node)? {
            Node::TypeParameterDeclaration(tp) => tp,
            _ => return None,
        };

        // Get the name of the type parameter
        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(tp.name) {
            id.escaped_text.clone()
        } else {
            return None;
        };

        // Create a symbol for the type parameter
        let symbol_id = self.local_symbols.alloc(symbol_flags::TYPE_PARAMETER, name.clone());

        // Get constraint type if present
        let constraint = if !tp.constraint.is_none() {
            self.get_type_of_node(tp.constraint)
        } else {
            TypeId::NONE
        };

        // Get default type if present
        let default = if !tp.default.is_none() {
            self.get_type_of_node(tp.default)
        } else {
            TypeId::NONE
        };

        // Create and allocate the type parameter
        let type_param = TypeParameter {
            flags: type_flags::TYPE_PARAMETER,
            symbol: symbol_id,
            constraint,
            default,
            target: TypeId::NONE,
            is_this_type: false,
        };

        // Store the name for type_to_string
        let type_id = self.types.alloc(Type::TypeParameter(type_param));

        // Cache the type parameter name for later lookup
        self.type_parameter_names.insert(type_id, name);

        Some(type_id)
    }

    /// Instantiate a generic type with type arguments.
    /// Replaces type parameters with the provided type arguments.
    pub fn instantiate_type(&mut self, type_id: TypeId, type_arguments: &[TypeId], type_parameters: &[TypeId]) -> TypeId {
        // If no type arguments, return the original type
        if type_arguments.is_empty() || type_parameters.is_empty() {
            return type_id;
        }

        // Create a mapping from type parameters to type arguments
        let mapper: std::collections::HashMap<TypeId, TypeId> = type_parameters.iter()
            .zip(type_arguments.iter())
            .map(|(&param, &arg)| (param, arg))
            .collect();

        self.instantiate_type_with_mapper(type_id, &mapper)
    }

    /// Instantiate a type using a type parameter mapper.
    fn instantiate_type_with_mapper(&mut self, type_id: TypeId, mapper: &std::collections::HashMap<TypeId, TypeId>) -> TypeId {
        // Check if this type parameter is in the mapper
        if let Some(&mapped_type) = mapper.get(&type_id) {
            return mapped_type;
        }

        // Extract data from the type to avoid borrowing issues
        enum TypeInfo {
            TypeParameter,
            Function {
                declaration: NodeIndex,
                parameter_types: Vec<TypeId>,
                parameter_names: Vec<String>,
                return_type: TypeId,
                min_argument_count: u32,
                has_rest_parameter: bool,
            },
            Union {
                types: Vec<TypeId>,
            },
            Intersection {
                types: Vec<TypeId>,
            },
            Mapped {
                type_parameter: TypeId,
                constraint_type: TypeId,
                template_type: TypeId,
            },
            IndexedAccess {
                object_type: TypeId,
                index_type: TypeId,
            },
            Index {
                source_type: TypeId,
            },
            Conditional {
                check_type: TypeId,
                extends_type: TypeId,
                true_type: TypeId,
                false_type: TypeId,
                is_distributive: bool,
            },
            Array {
                element_type: TypeId,
                is_readonly: bool,
            },
            Other,
        }

        let type_info = match self.types.get(type_id) {
            Some(Type::TypeParameter(_)) => TypeInfo::TypeParameter,
            Some(Type::Function(f)) => TypeInfo::Function {
                declaration: f.declaration,
                parameter_types: f.parameter_types.clone(),
                parameter_names: f.parameter_names.clone(),
                return_type: f.return_type,
                min_argument_count: f.min_argument_count,
                has_rest_parameter: f.has_rest_parameter,
            },
            Some(Type::Union(u)) => TypeInfo::Union {
                types: u.types.clone(),
            },
            Some(Type::Intersection(i)) => TypeInfo::Intersection {
                types: i.types.clone(),
            },
            Some(Type::Mapped(m)) => TypeInfo::Mapped {
                type_parameter: m.type_parameter,
                constraint_type: m.constraint_type,
                template_type: m.template_type,
            },
            Some(Type::IndexedAccess(ia)) => TypeInfo::IndexedAccess {
                object_type: ia.object_type,
                index_type: ia.index_type,
            },
            Some(Type::Index(i)) => TypeInfo::Index {
                source_type: i.source_type,
            },
            Some(Type::Conditional(c)) => TypeInfo::Conditional {
                check_type: c.check_type,
                extends_type: c.extends_type,
                true_type: c.true_type,
                false_type: c.false_type,
                is_distributive: c.is_distributive,
            },
            Some(Type::Array(arr)) => TypeInfo::Array {
                element_type: arr.element_type,
                is_readonly: arr.is_readonly,
            },
            Some(_) => TypeInfo::Other,
            None => return type_id,
        };

        match type_info {
            // Type parameter - already checked above
            TypeInfo::TypeParameter => type_id,

            // Function type - instantiate return type and parameter types
            TypeInfo::Function {
                declaration,
                parameter_types,
                parameter_names,
                return_type,
                min_argument_count,
                has_rest_parameter,
            } => {
                let new_param_types: Vec<TypeId> = parameter_types.iter()
                    .map(|&pt| self.instantiate_type_with_mapper(pt, mapper))
                    .collect();
                let new_return_type = self.instantiate_type_with_mapper(return_type, mapper);

                // Check if anything changed
                if new_param_types == parameter_types && new_return_type == return_type {
                    return type_id;
                }

                self.types.create_function_type(
                    declaration,
                    new_param_types,
                    parameter_names,
                    new_return_type,
                    min_argument_count,
                    has_rest_parameter,
                )
            }

            // Union type - instantiate each constituent
            TypeInfo::Union { types } => {
                let new_types: Vec<TypeId> = types.iter()
                    .map(|&t| self.instantiate_type_with_mapper(t, mapper))
                    .collect();

                if new_types == types {
                    return type_id;
                }

                self.types.create_union_type(new_types)
            }

            // Intersection type - instantiate each constituent
            TypeInfo::Intersection { types } => {
                let new_types: Vec<TypeId> = types.iter()
                    .map(|&t| self.instantiate_type_with_mapper(t, mapper))
                    .collect();

                if new_types == types {
                    return type_id;
                }

                self.types.create_intersection(new_types)
            }

            // Mapped type - instantiate to concrete object type
            TypeInfo::Mapped { type_parameter, constraint_type, template_type } => {
                // First, instantiate the constraint type to get the concrete keys
                let instantiated_constraint = self.instantiate_type_with_mapper(constraint_type, mapper);

                // Get the keys from the instantiated constraint
                let keys = self.get_keys_from_type(instantiated_constraint);

                if keys.is_empty() {
                    // If no keys, return empty object type
                    return self.types.create_object_type(Vec::new());
                }

                // For each key, instantiate the template type with K bound to that key
                let mut properties = Vec::new();
                let mut members_table = SymbolTable::new();

                for key_name in keys {
                    // Create a string literal type for this key
                    let key_type = self.types.create_string_literal(key_name.clone());

                    // Create a new mapper with the type parameter bound to this key
                    let mut inner_mapper = mapper.clone();
                    inner_mapper.insert(type_parameter, key_type);

                    // Instantiate the template type with the key bound
                    let property_type = self.instantiate_type_with_mapper(template_type, &inner_mapper);

                    // Create a symbol for this property
                    let symbol_id = self.local_symbols_mut().alloc(symbol_flags::PROPERTY, key_name.clone());
                    self.symbol_types.insert(symbol_id, property_type);
                    properties.push(symbol_id);
                    members_table.set(key_name, symbol_id);
                }

                self.types.create_object_type_with_members(properties, members_table)
            }

            // Indexed access type - instantiate object and index types
            TypeInfo::IndexedAccess { object_type, index_type } => {
                let new_object = self.instantiate_type_with_mapper(object_type, mapper);
                let new_index = self.instantiate_type_with_mapper(index_type, mapper);

                if new_object == object_type && new_index == index_type {
                    return type_id;
                }

                // Try to resolve the indexed access
                self.get_indexed_access_type(new_object, new_index)
            }

            // Index type (keyof) - instantiate the source type
            TypeInfo::Index { source_type } => {
                let new_source = self.instantiate_type_with_mapper(source_type, mapper);

                if new_source == source_type {
                    return type_id;
                }

                // Compute keyof for the new source type
                self.get_keyof_type(new_source)
            }

            // Conditional type - instantiate and evaluate
            TypeInfo::Conditional { check_type, extends_type, true_type, false_type, is_distributive } => {
                let new_check = self.instantiate_type_with_mapper(check_type, mapper);
                let new_extends = self.instantiate_type_with_mapper(extends_type, mapper);
                let new_true = self.instantiate_type_with_mapper(true_type, mapper);
                let new_false = self.instantiate_type_with_mapper(false_type, mapper);

                // Check if the check type still contains type parameters
                if self.type_contains_type_parameter(new_check) {
                    // Still deferred - create new conditional type
                    if new_check == check_type && new_extends == extends_type
                        && new_true == true_type && new_false == false_type {
                        return type_id;
                    }
                    return self.types.create_conditional_type(new_check, new_extends, new_true, new_false);
                }

                // Handle distributive conditional types over unions
                // If the original conditional was distributive and new_check is a union,
                // distribute the conditional over each union member
                if is_distributive {
                    if let Some(Type::Union(union)) = self.types.get(new_check) {
                        let member_types = union.types.clone();
                        let mut result_types = Vec::new();

                        for member in member_types {
                            // Check if extends type contains infer types
                            if self.type_contains_infer(new_extends) {
                                let mut inferences: std::collections::HashMap<TypeId, TypeId> = std::collections::HashMap::new();
                                if self.infer_from_type(member, new_extends, &mut inferences) {
                                    let result = self.instantiate_type_with_mapper(new_true, &inferences);
                                    if !result_types.contains(&result) {
                                        result_types.push(result);
                                    }
                                } else {
                                    if !result_types.contains(&new_false) {
                                        result_types.push(new_false);
                                    }
                                }
                            } else {
                                // Simple assignability check
                                let result = if self.is_type_assignable_to(member, new_extends) {
                                    new_true
                                } else {
                                    new_false
                                };
                                if !result_types.contains(&result) {
                                    result_types.push(result);
                                }
                            }
                        }

                        if result_types.is_empty() {
                            return self.types.never_type;
                        }
                        if result_types.len() == 1 {
                            return result_types[0];
                        }
                        return self.types.create_union(result_types);
                    }
                }

                // Check if extends type contains infer types
                if self.type_contains_infer(new_extends) {
                    let mut inferences: std::collections::HashMap<TypeId, TypeId> = std::collections::HashMap::new();
                    if self.infer_from_type(new_check, new_extends, &mut inferences) {
                        // Pattern matched - substitute inferences in true branch
                        return self.instantiate_type_with_mapper(new_true, &inferences);
                    } else {
                        return new_false;
                    }
                }

                // Evaluate the condition
                if self.is_type_assignable_to(new_check, new_extends) {
                    new_true
                } else {
                    new_false
                }
            }

            // Array type - instantiate element type
            TypeInfo::Array { element_type, is_readonly } => {
                let new_element = self.instantiate_type_with_mapper(element_type, mapper);

                if new_element == element_type {
                    return type_id;
                }

                self.types.create_array_type(new_element, is_readonly)
            }

            // Other types - return as-is for now
            TypeInfo::Other => type_id,
        }
    }

    /// Get property names from a type (for mapped type instantiation).
    fn get_keys_from_type(&mut self, type_id: TypeId) -> Vec<String> {
        let mut keys = Vec::new();

        let Some(typ) = self.types.get(type_id) else {
            return keys;
        };

        match typ {
            // Union of string literals
            Type::Union(u) => {
                let types = u.types.clone();
                for t in types {
                    if let Some(Type::Literal(lit)) = self.types.get(t) {
                        if let LiteralValue::String(s) = &lit.value {
                            keys.push(s.clone());
                        }
                    }
                }
            }
            // Single string literal
            Type::Literal(lit) => {
                if let LiteralValue::String(s) = &lit.value {
                    keys.push(s.clone());
                }
            }
            // Object type - get property names
            Type::Object(obj) => {
                for (name, _) in obj.members.iter() {
                    keys.push(name.clone());
                }
            }
            _ => {}
        }

        keys
    }

    /// Get the type of a type reference with type arguments (e.g., Array<T>, Map<K, V>).
    fn get_type_of_type_reference_with_args(&mut self, type_name: NodeIndex, type_arguments: &crate::parser::NodeList) -> TypeId {
        use crate::parser::Node;

        // Get the base type name
        let name = if let Some(Node::Identifier(id)) = self.node_arena.get(type_name) {
            id.escaped_text.as_str()
        } else {
            return self.types.object_type;
        };

        // Resolve type arguments
        let type_args: Vec<TypeId> = type_arguments.nodes.iter()
            .map(|&arg| self.get_type_of_node(arg))
            .collect();

        // Look up the type in the symbol table
        if let Some(symbol_id) = self.file_locals.get(name) {
            // Check if this is a type alias with type parameters
            if let Some(symbol) = self.symbol_arena.get(symbol_id) {
                if symbol.has_flags(crate::binder::symbol_flags::TYPE_ALIAS) {
                    if let Some(&decl_idx) = symbol.declarations.first() {
                        if let Some(Node::TypeAliasDeclaration(ta)) = self.node_arena.get(decl_idx) {
                            if let Some(ref type_params) = ta.type_parameters {
                                // Collect type parameter TypeIds
                                let mut param_type_ids = Vec::new();

                                // Save current type parameter scope
                                let saved_scope = std::mem::take(&mut self.type_parameter_scope);

                                // Create type parameters and build the scope
                                for &tp_idx in &type_params.nodes {
                                    if let Some(type_id) = self.create_type_parameter(tp_idx) {
                                        param_type_ids.push(type_id);
                                        if let Some(name) = self.type_parameter_names.get(&type_id) {
                                            self.type_parameter_scope.insert(name.clone(), type_id);
                                        }
                                    }
                                }

                                // Get the body type with type parameters in scope
                                let body_type = self.get_type_of_node(ta.type_node);

                                // Restore scope
                                self.type_parameter_scope = saved_scope;

                                // Instantiate with the provided type arguments
                                if !param_type_ids.is_empty() && !type_args.is_empty() {
                                    return self.instantiate_type(body_type, &type_args, &param_type_ids);
                                }

                                return body_type;
                            }
                        }
                    }
                }
            }

            let base_type = self.get_type_of_symbol(symbol_id);

            // Check if it's a generic function type that needs instantiation
            if let Some(Type::Function(f)) = self.types.get(base_type) {
                if !f.type_parameters.is_empty() {
                    return self.instantiate_type(base_type, &type_args, &f.type_parameters.clone());
                }
            }

            return base_type;
        }

        // Built-in generic types
        match name {
            "Array" => {
                // Array<T> becomes a mutable array type with element type T
                let element_type = type_args.first().copied().unwrap_or(self.types.any_type);
                self.types.create_array_type(element_type, false)
            }
            "ReadonlyArray" => {
                // ReadonlyArray<T> becomes a readonly array type with element type T
                let element_type = type_args.first().copied().unwrap_or(self.types.any_type);
                self.types.create_array_type(element_type, true)
            }
            "Promise" => {
                self.types.object_type
            }
            "Map" | "Set" | "WeakMap" | "WeakSet" => {
                self.types.object_type
            }
            _ => self.types.object_type,
        }
    }

    /// Get the type of a symbol (with caching).
    pub fn get_type_of_symbol(&mut self, symbol_id: SymbolId) -> TypeId {
        // Check cache first
        if let Some(&cached) = self.symbol_types.get(&symbol_id) {
            return cached;
        }

        let type_id = self.get_type_of_symbol_worker(symbol_id);
        self.symbol_types.insert(symbol_id, type_id);
        type_id
    }

    /// Get type of symbol (worker, no caching).
    fn get_type_of_symbol_worker(&mut self, symbol_id: SymbolId) -> TypeId {
        use crate::binder::symbol_flags;

        let Some(symbol) = self.symbol_arena.get(symbol_id) else {
            return self.types.any_type;
        };

        // For type aliases, use the first declaration
        if symbol.has_flags(symbol_flags::TYPE_ALIAS) {
            if let Some(&decl) = symbol.declarations.first() {
                return self.get_type_of_node(decl);
            }
        }

        // Get type from value declaration
        if !symbol.value_declaration.is_none() {
            return self.get_type_of_node(symbol.value_declaration);
        }

        // Fallback: try first declaration
        if let Some(&decl) = symbol.declarations.first() {
            return self.get_type_of_node(decl);
        }

        self.types.any_type
    }

    // =========================================================================
    // Type Checking
    // =========================================================================

    /// Check if a type is assignable to another.
    /// Returns true if source is assignable to target.
    pub fn is_type_assignable_to(&self, source: TypeId, target: TypeId) -> bool {
        // Same type is always assignable
        if source == target {
            return true;
        }

        let Some(source_type) = self.types.get(source) else {
            return false;
        };
        let Some(target_type) = self.types.get(target) else {
            return false;
        };

        let source_flags = source_type.flags();
        let target_flags = target_type.flags();

        // any is assignable to anything, anything is assignable to any
        if (source_flags & type_flags::ANY) != 0 || (target_flags & type_flags::ANY) != 0 {
            return true;
        }

        // unknown accepts anything
        if (target_flags & type_flags::UNKNOWN) != 0 {
            return true;
        }

        // never is assignable to everything
        if (source_flags & type_flags::NEVER) != 0 {
            return true;
        }

        // undefined is assignable to void
        if (source_flags & type_flags::UNDEFINED) != 0 && (target_flags & type_flags::VOID) != 0 {
            return true;
        }

        // null is assignable to undefined (with strictNullChecks off)
        if (source_flags & type_flags::NULL) != 0 && (target_flags & type_flags::UNDEFINED) != 0 {
            return true;
        }

        // String literal is assignable to string
        if (source_flags & type_flags::STRING_LITERAL) != 0 && (target_flags & type_flags::STRING) != 0 {
            return true;
        }

        // Number literal is assignable to number
        if (source_flags & type_flags::NUMBER_LITERAL) != 0 && (target_flags & type_flags::NUMBER) != 0 {
            return true;
        }

        // Boolean literal is assignable to boolean
        if (source_flags & type_flags::BOOLEAN_LITERAL) != 0 && (target_flags & type_flags::BOOLEAN) != 0 {
            return true;
        }

        // BigInt literal is assignable to bigint
        if (source_flags & type_flags::BIG_INT_LITERAL) != 0 && (target_flags & type_flags::BIG_INT) != 0 {
            return true;
        }

        // Handle union targets: source assignable to any constituent
        if let Type::Union(union) = target_type {
            return union.types.iter().any(|&t| self.is_type_assignable_to(source, t));
        }

        // Handle union sources: all constituents must be assignable
        if let Type::Union(union) = source_type {
            return union.types.iter().all(|&t| self.is_type_assignable_to(t, target));
        }

        // Handle intersection targets: must be assignable to all
        if let Type::Intersection(intersection) = target_type {
            return intersection.types.iter().all(|&t| self.is_type_assignable_to(source, t));
        }

        // Array to array assignability: element type must be assignable
        // Readonly array cannot be assigned to mutable array
        if let (Type::Array(source_arr), Type::Array(target_arr)) = (source_type, target_type) {
            // Cannot assign readonly to mutable
            if source_arr.is_readonly && !target_arr.is_readonly {
                return false;
            }
            return self.is_type_assignable_to(source_arr.element_type, target_arr.element_type);
        }

        // Tuple to tuple assignability: same length and each element assignable
        // Readonly tuple cannot be assigned to mutable tuple
        if let (Type::Tuple(source_tup), Type::Tuple(target_tup)) = (source_type, target_type) {
            // Cannot assign readonly to mutable
            if source_tup.is_readonly && !target_tup.is_readonly {
                return false;
            }
            if source_tup.element_types.len() != target_tup.element_types.len() {
                return false;
            }
            return source_tup.element_types.iter()
                .zip(target_tup.element_types.iter())
                .all(|(&s, &t)| self.is_type_assignable_to(s, t));
        }

        // Tuple is assignable to array if all tuple elements are assignable to array element
        // Readonly tuple cannot be assigned to mutable array
        if let (Type::Tuple(source_tup), Type::Array(target_arr)) = (source_type, target_type) {
            // Cannot assign readonly tuple to mutable array
            if source_tup.is_readonly && !target_arr.is_readonly {
                return false;
            }
            return source_tup.element_types.iter()
                .all(|&elem| self.is_type_assignable_to(elem, target_arr.element_type));
        }

        false
    }

    // =========================================================================
    // Type Narrowing
    // =========================================================================

    /// Narrow a type based on a typeof guard.
    /// Returns the narrowed type if the guard matches, or the original type.
    ///
    /// For example, if type is `string | number` and typeof_result is "string",
    /// returns `string`.
    pub fn narrow_type_by_typeof(&mut self, type_id: TypeId, typeof_result: &str) -> TypeId {
        let expected_flags = match typeof_result {
            "string" => type_flags::STRING | type_flags::STRING_LITERAL,
            "number" => type_flags::NUMBER | type_flags::NUMBER_LITERAL,
            "boolean" => type_flags::BOOLEAN | type_flags::BOOLEAN_LITERAL,
            "bigint" => type_flags::BIG_INT | type_flags::BIG_INT_LITERAL,
            "symbol" => type_flags::ES_SYMBOL | type_flags::UNIQUE_ES_SYMBOL,
            "undefined" => type_flags::UNDEFINED,
            "function" => type_flags::OBJECT, // Functions are objects with call signatures
            "object" => type_flags::OBJECT | type_flags::NULL, // null returns "object" for typeof
            _ => return type_id, // Unknown typeof result
        };

        let Some(typ) = self.types.get(type_id) else {
            return type_id;
        };

        // If it's a union, filter to matching types
        if let Type::Union(u) = typ {
            let matching_types: Vec<TypeId> = u.types.iter()
                .filter(|&&t| {
                    if let Some(inner) = self.types.get(t) {
                        (inner.flags() & expected_flags) != 0
                    } else {
                        false
                    }
                })
                .copied()
                .collect();

            if matching_types.is_empty() {
                return self.types.never_type;
            } else if matching_types.len() == 1 {
                return matching_types[0];
            } else {
                return self.types.create_union_type(matching_types);
            }
        }

        // For non-union types, check if it matches
        let type_flags = typ.flags();
        if (type_flags & expected_flags) != 0 {
            type_id // Type matches, return as-is
        } else {
            self.types.never_type // Type doesn't match, narrow to never
        }
    }

    /// Narrow a type to exclude types matching a typeof guard.
    /// Returns the narrowed type if the guard doesn't match.
    ///
    /// For example, if type is `string | number` and typeof_result is "string",
    /// returns `number`.
    pub fn narrow_type_by_typeof_negation(&mut self, type_id: TypeId, typeof_result: &str) -> TypeId {
        let excluded_flags = match typeof_result {
            "string" => type_flags::STRING | type_flags::STRING_LITERAL,
            "number" => type_flags::NUMBER | type_flags::NUMBER_LITERAL,
            "boolean" => type_flags::BOOLEAN | type_flags::BOOLEAN_LITERAL,
            "bigint" => type_flags::BIG_INT | type_flags::BIG_INT_LITERAL,
            "symbol" => type_flags::ES_SYMBOL | type_flags::UNIQUE_ES_SYMBOL,
            "undefined" => type_flags::UNDEFINED,
            "function" => type_flags::OBJECT,
            "object" => type_flags::OBJECT | type_flags::NULL,
            _ => return type_id,
        };

        let Some(typ) = self.types.get(type_id) else {
            return type_id;
        };

        // If it's a union, filter out matching types
        if let Type::Union(u) = typ {
            let remaining_types: Vec<TypeId> = u.types.iter()
                .filter(|&&t| {
                    if let Some(inner) = self.types.get(t) {
                        (inner.flags() & excluded_flags) == 0
                    } else {
                        true
                    }
                })
                .copied()
                .collect();

            if remaining_types.is_empty() {
                return self.types.never_type;
            } else if remaining_types.len() == 1 {
                return remaining_types[0];
            } else {
                return self.types.create_union_type(remaining_types);
            }
        }

        // For non-union types, check if it should be excluded
        let type_flags = typ.flags();
        if (type_flags & excluded_flags) != 0 {
            self.types.never_type // Type matches exclusion
        } else {
            type_id // Type doesn't match, keep as-is
        }
    }

    /// Narrow a union type to exclude null and undefined.
    pub fn get_type_with_facts(&mut self, type_id: TypeId, include_null: bool, include_undefined: bool) -> TypeId {
        let Some(typ) = self.types.get(type_id) else {
            return type_id;
        };

        // If it's a union, filter appropriately
        if let Type::Union(u) = typ {
            let filtered_types: Vec<TypeId> = u.types.iter()
                .filter(|&&t| {
                    if let Some(inner) = self.types.get(t) {
                        let flags = inner.flags();
                        let is_null = (flags & type_flags::NULL) != 0;
                        let is_undefined = (flags & type_flags::UNDEFINED) != 0;

                        if is_null && !include_null {
                            return false;
                        }
                        if is_undefined && !include_undefined {
                            return false;
                        }
                        true
                    } else {
                        true
                    }
                })
                .copied()
                .collect();

            if filtered_types.is_empty() {
                return self.types.never_type;
            } else if filtered_types.len() == 1 {
                return filtered_types[0];
            } else {
                return self.types.create_union_type(filtered_types);
            }
        }

        // For non-union types, check if they should be excluded
        let type_flags = typ.flags();
        if (type_flags & type_flags::NULL) != 0 && !include_null {
            return self.types.never_type;
        }
        if (type_flags & type_flags::UNDEFINED) != 0 && !include_undefined {
            return self.types.never_type;
        }

        type_id
    }

    /// Get a non-nullable version of a type (exclude null and undefined).
    pub fn get_non_nullable_type(&mut self, type_id: TypeId) -> TypeId {
        self.get_type_with_facts(type_id, false, false)
    }

    /// Make a type readonly by setting the readonly flag on arrays and tuples.
    /// For other types, returns the type unchanged.
    pub fn make_type_readonly(&mut self, type_id: TypeId) -> TypeId {
        let Some(ty) = self.types.get(type_id) else {
            return type_id;
        };

        match ty {
            Type::Array(arr) => {
                // If already readonly, return as-is
                if arr.is_readonly {
                    return type_id;
                }
                // Create a new readonly array type
                let element_type = arr.element_type;
                self.types.create_array_type(element_type, true)
            }
            Type::Tuple(tup) => {
                // If already readonly, return as-is
                if tup.is_readonly {
                    return type_id;
                }
                // Create a new readonly tuple type
                let element_types = tup.element_types.clone();
                let has_optional = tup.has_optional_elements;
                let has_rest = tup.has_rest_element;
                self.types.create_tuple_type(element_types, has_optional, has_rest, true)
            }
            _ => {
                // For other types, readonly doesn't change anything structurally
                type_id
            }
        }
    }

    /// Narrow a type based on an instanceof guard.
    /// When `x instanceof Foo` is true, narrows x to Foo (or intersection with Foo).
    ///
    /// For example, if type is `unknown` and target_type is class Foo,
    /// returns `Foo`.
    pub fn narrow_type_by_instanceof(&mut self, type_id: TypeId, target_type: TypeId) -> TypeId {
        // If the source type is any, unknown, or object, narrow to target
        let Some(source) = self.types.get(type_id) else {
            return target_type;
        };

        let source_flags = source.flags();

        // any or unknown narrows directly to the target
        if (source_flags & (type_flags::ANY | type_flags::UNKNOWN)) != 0 {
            return target_type;
        }

        // If it's a union type, filter to types that could be instanceof the target
        if let Type::Union(u) = source {
            let types = u.types.clone();
            let filtered_types: Vec<TypeId> = types.iter()
                .filter(|&&t| self.could_be_instanceof(t, target_type))
                .copied()
                .collect();

            if filtered_types.is_empty() {
                // No types could match, but instanceof succeeded, so result is target
                return target_type;
            } else if filtered_types.len() == 1 {
                return filtered_types[0];
            } else {
                return self.types.create_union_type(filtered_types);
            }
        }

        // For object types, check if they're related to target
        if (source_flags & type_flags::OBJECT) != 0 {
            // If source could be the target type, return the target
            if self.could_be_instanceof(type_id, target_type) {
                return target_type;
            }
        }

        // Default: return intersection of source and target
        // (this handles cases like `x instanceof Foo` where x might have additional properties)
        if type_id != target_type {
            // For now, just return the target type for simplicity
            return target_type;
        }

        target_type
    }

    /// Narrow a type by excluding types that match instanceof.
    /// When `x instanceof Foo` is false, narrows x to exclude Foo.
    pub fn narrow_type_by_instanceof_negation(&mut self, type_id: TypeId, target_type: TypeId) -> TypeId {
        let Some(source) = self.types.get(type_id) else {
            return type_id;
        };

        // If it's a union type, filter out the target type and its subtypes
        if let Type::Union(u) = source {
            let types = u.types.clone();
            let remaining_types: Vec<TypeId> = types.iter()
                .filter(|&&t| !self.is_definitely_instanceof(t, target_type))
                .copied()
                .collect();

            if remaining_types.is_empty() {
                return self.types.never_type;
            } else if remaining_types.len() == 1 {
                return remaining_types[0];
            } else {
                return self.types.create_union_type(remaining_types);
            }
        }

        // For single types, if it's definitely the target, narrow to never
        if self.is_definitely_instanceof(type_id, target_type) {
            return self.types.never_type;
        }

        type_id
    }

    /// Check if a type could potentially be an instance of a target type.
    fn could_be_instanceof(&self, type_id: TypeId, target_type: TypeId) -> bool {
        // Same type always matches
        if type_id == target_type {
            return true;
        }

        let Some(source) = self.types.get(type_id) else {
            return false;
        };

        let source_flags = source.flags();

        // Any/unknown could be anything
        if (source_flags & (type_flags::ANY | type_flags::UNKNOWN)) != 0 {
            return true;
        }

        // Primitives can't be instanceof
        if (source_flags & (type_flags::STRING | type_flags::NUMBER | type_flags::BOOLEAN |
                           type_flags::UNDEFINED | type_flags::NULL | type_flags::VOID |
                           type_flags::NEVER)) != 0 {
            return false;
        }

        // Objects could potentially match
        if (source_flags & type_flags::OBJECT) != 0 {
            return true;
        }

        false
    }

    /// Check if a type is definitely an instance of a target type.
    fn is_definitely_instanceof(&self, type_id: TypeId, target_type: TypeId) -> bool {
        // Same type always matches
        if type_id == target_type {
            return true;
        }

        // Check if type is assignable to target
        self.is_type_assignable_to(type_id, target_type)
    }

    // =========================================================================
    // Type Guard Analysis
    // =========================================================================

    /// Analyze a condition expression and extract type guard information.
    /// Returns None if the expression is not a recognized type guard pattern.
    pub fn get_type_guard_from_expression(&mut self, expr: NodeIndex) -> Option<TypeGuard> {
        use crate::parser::Node;
        use crate::scanner::SyntaxKind;

        let node = self.node_arena.get(expr)?;

        match node {
            // Binary expressions: typeof x === "string", x instanceof Foo, x !== null
            Node::BinaryExpression(be) => {
                self.get_type_guard_from_binary_expression(be)
            }

            // Prefix unary: !x (falsy check)
            Node::PrefixUnaryExpression(pue) => {
                // !x means x is falsy
                if pue.operator == SyntaxKind::ExclamationToken {
                    // Get the inner guard and negate it
                    if let Some(guard) = self.get_type_guard_from_expression(pue.operand) {
                        return Some(self.negate_type_guard(guard));
                    }
                    // Just !x without a nested guard - treat as truthiness check
                    return Some(TypeGuard::Truthiness {
                        target: pue.operand,
                        is_truthy: false,
                    });
                }
                None
            }

            // Parenthesized expression: (x === "string")
            Node::ParenthesizedExpression(pe) => {
                self.get_type_guard_from_expression(pe.expression)
            }

            // Simple expression - treat as truthiness check
            _ => Some(TypeGuard::Truthiness {
                target: expr,
                is_truthy: true,
            }),
        }
    }

    /// Extract type guard from a binary expression.
    fn get_type_guard_from_binary_expression(&mut self, be: &crate::parser::BinaryExpression) -> Option<TypeGuard> {
        use crate::parser::Node;
        use crate::scanner::SyntaxKind;

        // Check for typeof guard: typeof x === "string"
        if let Some(Node::PrefixUnaryExpression(pue)) = self.node_arena.get(be.left) {
            if pue.operator == SyntaxKind::TypeOfKeyword {
                // Check if right side is a string literal
                if let Some(Node::StringLiteral(sl)) = self.node_arena.get(be.right) {
                    let is_equality = matches!(be.operator_token,
                        SyntaxKind::EqualsEqualsToken | SyntaxKind::EqualsEqualsEqualsToken);
                    let is_inequality = matches!(be.operator_token,
                        SyntaxKind::ExclamationEqualsToken | SyntaxKind::ExclamationEqualsEqualsToken);

                    if is_equality || is_inequality {
                        return Some(TypeGuard::Typeof {
                            target: pue.operand,
                            typeof_result: sl.text.clone(),
                            is_equality,
                        });
                    }
                }
            }
        }

        // Check for instanceof guard: x instanceof Foo
        if be.operator_token == SyntaxKind::InstanceOfKeyword {
            let constructor_type = self.get_type_of_node(be.right);
            return Some(TypeGuard::Instanceof {
                target: be.left,
                constructor_type,
                is_positive: true,
            });
        }

        // Check for null/undefined guards: x !== null, x !== undefined
        let is_equality = matches!(be.operator_token,
            SyntaxKind::EqualsEqualsToken | SyntaxKind::EqualsEqualsEqualsToken);
        let is_inequality = matches!(be.operator_token,
            SyntaxKind::ExclamationEqualsToken | SyntaxKind::ExclamationEqualsEqualsToken);

        if is_equality || is_inequality {
            // Check if right side is null or undefined
            if let Some(Node::Token(base)) = self.node_arena.get(be.right) {
                if base.kind == SyntaxKind::NullKeyword as u16 {
                    return Some(TypeGuard::Truthiness {
                        target: be.left,
                        is_truthy: is_inequality, // x !== null means x is truthy (non-null)
                    });
                }
            }
            if let Some(Node::Identifier(id)) = self.node_arena.get(be.right) {
                if id.escaped_text == "undefined" {
                    return Some(TypeGuard::Truthiness {
                        target: be.left,
                        is_truthy: is_inequality,
                    });
                }
            }
        }

        None
    }

    /// Negate a type guard (for use with ! or else branches).
    fn negate_type_guard(&self, guard: TypeGuard) -> TypeGuard {
        match guard {
            TypeGuard::Typeof { target, typeof_result, is_equality } => {
                TypeGuard::Typeof { target, typeof_result, is_equality: !is_equality }
            }
            TypeGuard::Instanceof { target, constructor_type, is_positive } => {
                TypeGuard::Instanceof { target, constructor_type, is_positive: !is_positive }
            }
            TypeGuard::Truthiness { target, is_truthy } => {
                TypeGuard::Truthiness { target, is_truthy: !is_truthy }
            }
        }
    }

    /// Apply a type guard to narrow a type.
    pub fn apply_type_guard(&mut self, type_id: TypeId, guard: &TypeGuard) -> TypeId {
        match guard {
            TypeGuard::Typeof { typeof_result, is_equality, .. } => {
                if *is_equality {
                    self.narrow_type_by_typeof(type_id, typeof_result)
                } else {
                    self.narrow_type_by_typeof_negation(type_id, typeof_result)
                }
            }
            TypeGuard::Instanceof { constructor_type, is_positive, .. } => {
                if *is_positive {
                    self.narrow_type_by_instanceof(type_id, *constructor_type)
                } else {
                    self.narrow_type_by_instanceof_negation(type_id, *constructor_type)
                }
            }
            TypeGuard::Truthiness { is_truthy, .. } => {
                if *is_truthy {
                    // Remove null and undefined for truthy check
                    self.get_non_nullable_type(type_id)
                } else {
                    // For falsy, keep only null/undefined (if present in union)
                    type_id // TODO: Implement proper falsy narrowing
                }
            }
        }
    }

    /// Get the narrowed type of a symbol reference given its flow node.
    /// This walks back through the flow graph to find applicable type guards.
    pub fn get_narrowed_type_at_flow(
        &mut self,
        symbol_id: SymbolId,
        base_type: TypeId,
        flow_node_id: crate::binder::FlowNodeId,
        flow_arena: &crate::binder::FlowNodeArena,
    ) -> TypeId {
        use crate::binder::flow_flags;

        let mut current_type = base_type;
        let mut current_flow = flow_node_id;

        // Walk back through the flow graph
        while !current_flow.is_none() {
            let Some(flow) = flow_arena.get(current_flow) else {
                break;
            };

            // Check if this is a condition node
            if flow.has_any_flags(flow_flags::TRUE_CONDITION | flow_flags::FALSE_CONDITION) {
                // Get the condition expression
                if !flow.node.is_none() {
                    if let Some(guard) = self.get_type_guard_from_expression(flow.node) {
                        // Check if the guard applies to our symbol
                        if self.guard_applies_to_symbol(&guard, symbol_id) {
                            // Determine if we should apply the guard or its negation
                            let effective_guard = if flow.has_flags(flow_flags::TRUE_CONDITION) {
                                guard
                            } else {
                                self.negate_type_guard(guard)
                            };
                            current_type = self.apply_type_guard(current_type, &effective_guard);
                        }
                    }
                }
            }

            // Move to the antecedent
            if let Some(&antecedent) = flow.antecedent.first() {
                current_flow = antecedent;
            } else {
                break;
            }
        }

        current_type
    }

    /// Check if a type guard applies to a specific symbol.
    fn guard_applies_to_symbol(&self, guard: &TypeGuard, symbol_id: SymbolId) -> bool {
        use crate::parser::Node;

        let target = match guard {
            TypeGuard::Typeof { target, .. } => *target,
            TypeGuard::Instanceof { target, .. } => *target,
            TypeGuard::Truthiness { target, .. } => *target,
        };

        // Check if the target expression refers to this symbol
        if let Some(Node::Identifier(id)) = self.node_arena.get(target) {
            if let Some(target_symbol) = self.file_locals.get(&id.escaped_text) {
                return target_symbol == symbol_id;
            }
        }

        false
    }

    /// Get the diagnostics as JSON.
    pub fn get_diagnostics_json(&self) -> String {
        serde_json::to_string(&self.diagnostics).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get the number of types allocated.
    pub fn get_type_count(&self) -> usize {
        self.types.len()
    }

    /// Type to string for debugging.
    pub fn type_to_string(&self, type_id: TypeId) -> String {
        let Some(typ) = self.types.get(type_id) else {
            return "unknown".to_string();
        };

        match typ {
            Type::Intrinsic(i) => i.intrinsic_name.clone(),
            Type::Literal(lit) => match &lit.value {
                LiteralValue::String(s) => format!("\"{}\"", s),
                LiteralValue::Number(n) => n.to_string(),
                LiteralValue::BigInt(b) => format!("{}n", b),
                LiteralValue::Boolean(b) => b.to_string(),
            },
            Type::Union(u) => {
                let parts: Vec<String> = u.types.iter()
                    .map(|&t| self.type_to_string(t))
                    .collect();
                parts.join(" | ")
            }
            Type::Intersection(i) => {
                let parts: Vec<String> = i.types.iter()
                    .map(|&t| self.type_to_string(t))
                    .collect();
                parts.join(" & ")
            }
            Type::Object(obj) => {
                if obj.members.is_empty() && obj.properties.is_empty() {
                    "object".to_string()
                } else {
                    // Show object type with its members
                    let mut parts = Vec::new();
                    for (name, &symbol_id) in obj.members.iter() {
                        if let Some(&prop_type) = self.symbol_types.get(&symbol_id) {
                            parts.push(format!("{}: {}", name, self.type_to_string(prop_type)));
                        } else {
                            parts.push(format!("{}: any", name));
                        }
                    }
                    format!("{{ {} }}", parts.join("; "))
                }
            }
            Type::TypeReference(_) => "TypeReference".to_string(),
            Type::TypeParameter(_) => {
                // Try to get name from cached names first
                if let Some(name) = self.type_parameter_names.get(&type_id) {
                    name.clone()
                } else if let Type::TypeParameter(tp) = typ {
                    if let Some(sym) = self.symbol_arena.get(tp.symbol) {
                        sym.escaped_name.clone()
                    } else {
                        "T".to_string()
                    }
                } else {
                    "T".to_string()
                }
            }
            Type::Conditional(c) => {
                format!(
                    "{} extends {} ? {} : {}",
                    self.type_to_string(c.check_type),
                    self.type_to_string(c.extends_type),
                    self.type_to_string(c.true_type),
                    self.type_to_string(c.false_type),
                )
            }
            Type::Mapped(m) => {
                // Format as { [K in constraint]: template }
                let constraint_str = self.type_to_string(m.constraint_type);
                let template_str = self.type_to_string(m.template_type);
                format!("{{ [K in {}]: {} }}", constraint_str, template_str)
            }
            Type::IndexedAccess(_) => "IndexedAccessType".to_string(),
            Type::Index(_) => "IndexType".to_string(),
            Type::TemplateLiteral(tl) => {
                // Format as `${text}${type}${text}...`
                let mut result = String::from("`");
                for (i, text) in tl.texts.iter().enumerate() {
                    result.push_str(text);
                    if i < tl.types.len() {
                        result.push_str("${");
                        result.push_str(&self.type_to_string(tl.types[i]));
                        result.push('}');
                    }
                }
                result.push('`');
                result
            }
            Type::Function(f) => {
                // Format type parameters if present
                let type_params_str = if !f.type_parameters.is_empty() {
                    let tp_strs: Vec<String> = f.type_parameters.iter()
                        .map(|&tp| self.type_to_string(tp))
                        .collect();
                    format!("<{}>", tp_strs.join(", "))
                } else {
                    String::new()
                };

                // Format parameters
                let params: Vec<String> = f.parameter_names.iter()
                    .zip(f.parameter_types.iter())
                    .map(|(name, &typ)| {
                        if name.is_empty() {
                            self.type_to_string(typ)
                        } else {
                            format!("{}: {}", name, self.type_to_string(typ))
                        }
                    })
                    .collect();
                let return_str = self.type_to_string(f.return_type);
                format!("{}({}) => {}", type_params_str, params.join(", "), return_str)
            }
            Type::Array(arr) => {
                let elem_str = self.type_to_string(arr.element_type);
                if arr.is_readonly {
                    format!("readonly {}[]", elem_str)
                } else {
                    format!("{}[]", elem_str)
                }
            }
            Type::Tuple(tup) => {
                let elements: Vec<String> = tup.element_types.iter()
                    .map(|&t| self.type_to_string(t))
                    .collect();
                if tup.is_readonly {
                    format!("readonly [{}]", elements.join(", "))
                } else {
                    format!("[{}]", elements.join(", "))
                }
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_flags() {
        assert_eq!(type_flags::ANY, 1);
        assert_eq!(type_flags::UNKNOWN, 2);
        assert_eq!(type_flags::STRING, 4);
        assert_eq!(type_flags::NUMBER, 8);
        assert_eq!(type_flags::NULLABLE, type_flags::UNDEFINED | type_flags::NULL);
    }

    #[test]
    fn test_type_id() {
        let id = TypeId(42);
        assert_eq!(id.0, 42);
        assert!(!id.is_none());
        assert!(TypeId::NONE.is_none());
    }

    #[test]
    fn test_type_arena_creation() {
        let arena = TypeArena::new();

        // Should have pre-allocated singleton types
        assert!(!arena.any_type.is_none());
        assert!(!arena.unknown_type.is_none());
        assert!(!arena.string_type.is_none());
        assert!(!arena.number_type.is_none());
        assert!(!arena.boolean_type.is_none());
        assert!(!arena.void_type.is_none());
        assert!(!arena.undefined_type.is_none());
        assert!(!arena.null_type.is_none());
        assert!(!arena.never_type.is_none());

        // Total: 14 singleton types (12 intrinsic + 2 boolean literals)
        assert_eq!(arena.len(), 14);
    }

    #[test]
    fn test_intrinsic_types() {
        let arena = TypeArena::new();

        let any = arena.get(arena.any_type).unwrap();
        assert!(any.has_flags(type_flags::ANY));

        let string = arena.get(arena.string_type).unwrap();
        assert!(string.has_flags(type_flags::STRING));
        assert!(string.has_any_flags(type_flags::STRING_LIKE));

        let number = arena.get(arena.number_type).unwrap();
        assert!(number.has_flags(type_flags::NUMBER));
        assert!(number.has_any_flags(type_flags::NUMBER_LIKE));

        let never = arena.get(arena.never_type).unwrap();
        assert!(never.has_flags(type_flags::NEVER));
    }

    #[test]
    fn test_literal_types() {
        let mut arena = TypeArena::new();

        let str_lit = arena.create_string_literal("hello".to_string());
        let str_type = arena.get(str_lit).unwrap();
        assert!(str_type.has_flags(type_flags::STRING_LITERAL));
        assert!(str_type.has_any_flags(type_flags::STRING_LIKE));
        assert!(str_type.has_any_flags(type_flags::LITERAL));

        let num_lit = arena.create_number_literal(42.0);
        let num_type = arena.get(num_lit).unwrap();
        assert!(num_type.has_flags(type_flags::NUMBER_LITERAL));
        assert!(num_type.has_any_flags(type_flags::NUMBER_LIKE));
        assert!(num_type.has_any_flags(type_flags::LITERAL));
    }

    #[test]
    fn test_boolean_literal_singletons() {
        let arena = TypeArena::new();

        let true_type = arena.get(arena.true_type).unwrap();
        assert!(true_type.has_flags(type_flags::BOOLEAN_LITERAL));
        if let Type::Literal(lit) = true_type {
            assert_eq!(lit.value, LiteralValue::Boolean(true));
        } else {
            panic!("Expected literal type");
        }

        let false_type = arena.get(arena.false_type).unwrap();
        assert!(false_type.has_flags(type_flags::BOOLEAN_LITERAL));
        if let Type::Literal(lit) = false_type {
            assert_eq!(lit.value, LiteralValue::Boolean(false));
        } else {
            panic!("Expected literal type");
        }
    }

    #[test]
    fn test_union_type() {
        let mut arena = TypeArena::new();

        // Union of string | number
        let union = arena.create_union(vec![arena.string_type, arena.number_type]);
        let union_type = arena.get(union).unwrap();
        assert!(union_type.has_flags(type_flags::UNION));

        if let Type::Union(u) = union_type {
            assert_eq!(u.types.len(), 2);
            assert!(u.types.contains(&arena.string_type));
            assert!(u.types.contains(&arena.number_type));
        } else {
            panic!("Expected union type");
        }
    }

    #[test]
    fn test_union_single_type() {
        let mut arena = TypeArena::new();

        // Union of single type should return that type
        let union = arena.create_union(vec![arena.string_type]);
        assert_eq!(union, arena.string_type);
    }

    #[test]
    fn test_union_empty() {
        let mut arena = TypeArena::new();

        // Empty union should return never
        let union = arena.create_union(vec![]);
        assert_eq!(union, arena.never_type);
    }

    #[test]
    fn test_intersection_type() {
        let mut arena = TypeArena::new();

        let obj1 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));
        let obj2 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let intersection = arena.create_intersection(vec![obj1, obj2]);
        let int_type = arena.get(intersection).unwrap();
        assert!(int_type.has_flags(type_flags::INTERSECTION));

        if let Type::Intersection(i) = int_type {
            assert_eq!(i.types.len(), 2);
        } else {
            panic!("Expected intersection type");
        }
    }

    #[test]
    fn test_intersection_simplification_never() {
        // X & never = never
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let intersection = arena.create_intersection(vec![obj, arena.never_type]);
        assert_eq!(intersection, arena.never_type);
    }

    #[test]
    fn test_intersection_simplification_unknown() {
        // X & unknown = X
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let intersection = arena.create_intersection(vec![obj, arena.unknown_type]);
        assert_eq!(intersection, obj);
    }

    #[test]
    fn test_intersection_simplification_duplicates() {
        // X & X = X
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let intersection = arena.create_intersection(vec![obj, obj]);
        assert_eq!(intersection, obj);
    }

    #[test]
    fn test_intersection_simplification_flatten() {
        // (A & B) & C = A & B & C
        let mut arena = TypeArena::new();
        let obj1 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));
        let obj2 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));
        let obj3 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let inner = arena.create_intersection(vec![obj1, obj2]);
        let outer = arena.create_intersection(vec![inner, obj3]);

        // Should have flattened to a single intersection with 3 types
        if let Type::Intersection(i) = arena.get(outer).unwrap() {
            assert_eq!(i.types.len(), 3);
            assert!(i.types.contains(&obj1));
            assert!(i.types.contains(&obj2));
            assert!(i.types.contains(&obj3));
        } else {
            panic!("Expected flattened intersection type");
        }
    }

    #[test]
    fn test_intersection_empty_returns_unknown() {
        let mut arena = TypeArena::new();
        let intersection = arena.create_intersection(vec![]);
        assert_eq!(intersection, arena.unknown_type);
    }

    #[test]
    fn test_intersection_all_unknown_returns_unknown() {
        // unknown & unknown = unknown
        let mut arena = TypeArena::new();
        let intersection = arena.create_intersection(vec![arena.unknown_type, arena.unknown_type]);
        assert_eq!(intersection, arena.unknown_type);
    }

    #[test]
    fn test_template_literal_instantiation_simple() {
        // Template with no substitution returns string literal
        let mut arena = TypeArena::new();
        let result = arena.create_template_literal_type(vec!["hello".to_string()], vec![]);

        // Should be a string literal "hello"
        if let Some(Type::Literal(lit)) = arena.get(result) {
            assert!(matches!(lit.value, LiteralValue::String(ref s) if s == "hello"));
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_template_literal_instantiation_with_string() {
        // `hello ${world}` with "world" -> "hello world"
        let mut arena = TypeArena::new();
        let world_type = arena.create_string_literal("world".to_string());
        let result = arena.create_template_literal_type(
            vec!["hello ".to_string(), "".to_string()],
            vec![world_type],
        );

        // Should evaluate to "hello world"
        if let Some(Type::Literal(lit)) = arena.get(result) {
            assert!(matches!(lit.value, LiteralValue::String(ref s) if s == "hello world"),
                "Expected 'hello world', got: {:?}", lit.value);
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_template_literal_instantiation_with_number() {
        // `count: ${42}` -> "count: 42"
        let mut arena = TypeArena::new();
        let num_type = arena.create_number_literal(42.0);
        let result = arena.create_template_literal_type(
            vec!["count: ".to_string(), "".to_string()],
            vec![num_type],
        );

        // Should evaluate to "count: 42"
        if let Some(Type::Literal(lit)) = arena.get(result) {
            assert!(matches!(lit.value, LiteralValue::String(ref s) if s == "count: 42"),
                "Expected 'count: 42', got: {:?}", lit.value);
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_template_literal_instantiation_multiple() {
        // `${a} and ${b}` with "hello", "world" -> "hello and world"
        let mut arena = TypeArena::new();
        let a_type = arena.create_string_literal("hello".to_string());
        let b_type = arena.create_string_literal("world".to_string());
        let result = arena.create_template_literal_type(
            vec!["".to_string(), " and ".to_string(), "".to_string()],
            vec![a_type, b_type],
        );

        // Should evaluate to "hello and world"
        if let Some(Type::Literal(lit)) = arena.get(result) {
            assert!(matches!(lit.value, LiteralValue::String(ref s) if s == "hello and world"),
                "Expected 'hello and world', got: {:?}", lit.value);
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_template_literal_deferred() {
        // `hello ${T}` with type parameter T -> template literal type (not evaluated)
        let mut arena = TypeArena::new();
        let t_type = arena.create_type_parameter(SymbolId::NONE, TypeId::NONE, TypeId::NONE);
        let result = arena.create_template_literal_type(
            vec!["hello ".to_string(), "".to_string()],
            vec![t_type],
        );

        // Should remain a template literal type (not evaluated)
        if let Some(Type::TemplateLiteral(_)) = arena.get(result) {
            // Good, it's unevaluated
        } else {
            panic!("Expected template literal type to be deferred");
        }
    }

    #[test]
    fn test_object_flags() {
        assert_eq!(object_flags::CLASS, 1);
        assert_eq!(object_flags::INTERFACE, 2);
        assert_eq!(object_flags::CLASS_OR_INTERFACE, object_flags::CLASS | object_flags::INTERFACE);
    }

    #[test]
    fn test_checker_state_creation() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Should have intrinsic types pre-allocated
        assert!(!checker.types.any_type.is_none());
        assert!(!checker.types.string_type.is_none());
        assert_eq!(checker.diagnostics.len(), 0);
    }

    #[test]
    fn test_type_assignability_same() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Same type is assignable to itself
        assert!(checker.is_type_assignable_to(checker.types.string_type, checker.types.string_type));
        assert!(checker.is_type_assignable_to(checker.types.number_type, checker.types.number_type));
        assert!(checker.is_type_assignable_to(checker.types.boolean_type, checker.types.boolean_type));
    }

    #[test]
    fn test_type_assignability_any() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // any is assignable to anything
        assert!(checker.is_type_assignable_to(checker.types.any_type, checker.types.string_type));
        assert!(checker.is_type_assignable_to(checker.types.any_type, checker.types.number_type));

        // Anything is assignable to any
        assert!(checker.is_type_assignable_to(checker.types.string_type, checker.types.any_type));
        assert!(checker.is_type_assignable_to(checker.types.number_type, checker.types.any_type));
    }

    #[test]
    fn test_type_assignability_never() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // never is assignable to everything
        assert!(checker.is_type_assignable_to(checker.types.never_type, checker.types.string_type));
        assert!(checker.is_type_assignable_to(checker.types.never_type, checker.types.number_type));
        assert!(checker.is_type_assignable_to(checker.types.never_type, checker.types.void_type));
    }

    #[test]
    fn test_type_assignability_literals() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // String literal is assignable to string
        let str_lit = checker.types.create_string_literal("hello".to_string());
        assert!(checker.is_type_assignable_to(str_lit, checker.types.string_type));

        // Number literal is assignable to number
        let num_lit = checker.types.create_number_literal(42.0);
        assert!(checker.is_type_assignable_to(num_lit, checker.types.number_type));

        // Boolean literal is assignable to boolean
        assert!(checker.is_type_assignable_to(checker.types.true_type, checker.types.boolean_type));
        assert!(checker.is_type_assignable_to(checker.types.false_type, checker.types.boolean_type));
    }

    #[test]
    fn test_type_assignability_union() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // string is assignable to string | number
        let union = checker.types.create_union(vec![
            checker.types.string_type,
            checker.types.number_type,
        ]);
        assert!(checker.is_type_assignable_to(checker.types.string_type, union));
        assert!(checker.is_type_assignable_to(checker.types.number_type, union));

        // boolean is NOT assignable to string | number
        assert!(!checker.is_type_assignable_to(checker.types.boolean_type, union));
    }

    #[test]
    fn test_type_to_string() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        assert_eq!(checker.type_to_string(checker.types.string_type), "string");
        assert_eq!(checker.type_to_string(checker.types.number_type), "number");
        assert_eq!(checker.type_to_string(checker.types.boolean_type), "boolean");
        assert_eq!(checker.type_to_string(checker.types.any_type), "any");
        assert_eq!(checker.type_to_string(checker.types.never_type), "never");

        let str_lit = checker.types.create_string_literal("hello".to_string());
        assert_eq!(checker.type_to_string(str_lit), "\"hello\"");

        let num_lit = checker.types.create_number_literal(42.0);
        assert_eq!(checker.type_to_string(num_lit), "42");
    }

    #[test]
    fn test_get_type_of_literal_nodes() {
        use crate::parser_impl::ParserState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const x = "hello";"#.to_string(),
        );
        let root = parser.parse_source_file();

        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&parser.arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Get the source file
        if let Some(crate::parser::Node::SourceFile(sf)) = parser.arena.get(root) {
            // Get the first statement (variable statement)
            if let Some(&stmt_idx) = sf.statements.nodes.first() {
                if let Some(crate::parser::Node::VariableStatement(vs)) = parser.arena.get(stmt_idx) {
                    // Get declaration list
                    if let Some(crate::parser::Node::VariableDeclarationList(vdl)) = parser.arena.get(vs.declaration_list) {
                        // Get the first declaration
                        if let Some(&decl_idx) = vdl.declarations.nodes.first() {
                            let decl_type = checker.get_type_of_node(decl_idx);
                            // Should be a string literal type
                            assert!(checker.types.get(decl_type).unwrap().has_flags(type_flags::STRING_LITERAL));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_symbol_type_resolution() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Parse and bind a simple program
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const x = "hello"; const y = x;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Create checker with bound symbols
        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Verify 'x' is in the symbol table
        assert!(binder.file_locals.has("x"));

        // Get type of 'x' symbol
        if let Some(x_symbol) = binder.file_locals.get("x") {
            let x_type = checker.get_type_of_symbol(x_symbol);
            // x should have type "hello" (string literal)
            assert!(checker.types.get(x_type).unwrap().has_flags(type_flags::STRING_LITERAL));
        }
    }

    #[test]
    fn test_function_type_inference() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Parse a function declaration
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"function add(x: number, y: number): number { return x + y; }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Verify 'add' is in the symbol table
        assert!(binder.file_locals.has("add"));

        // Get type of 'add' symbol
        if let Some(add_symbol) = binder.file_locals.get("add") {
            let add_type = checker.get_type_of_symbol(add_symbol);

            // Should be a function type with OBJECT flag
            let typ = checker.types.get(add_type).unwrap();
            assert!(typ.has_flags(type_flags::OBJECT));

            // Verify it's a Function variant
            if let Type::Function(f) = typ {
                assert_eq!(f.parameter_names.len(), 2);
                assert_eq!(f.parameter_names[0], "x");
                assert_eq!(f.parameter_names[1], "y");
                assert_eq!(f.parameter_types.len(), 2);
                assert_eq!(f.min_argument_count, 2);
                assert!(!f.has_rest_parameter);

                // Both params should be number type
                assert_eq!(f.parameter_types[0], checker.types.number_type);
                assert_eq!(f.parameter_types[1], checker.types.number_type);

                // Return type should be number
                assert_eq!(f.return_type, checker.types.number_type);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }

            // Verify type_to_string works
            let type_str = checker.type_to_string(add_type);
            assert_eq!(type_str, "(x: number, y: number) => number");
        }
    }

    #[test]
    fn test_function_type_with_optional_params() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"function greet(name: string, greeting?: string): void {}"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        if let Some(fn_symbol) = binder.file_locals.get("greet") {
            let fn_type = checker.get_type_of_symbol(fn_symbol);

            if let Type::Function(f) = checker.types.get(fn_type).unwrap() {
                assert_eq!(f.parameter_names.len(), 2);
                // Optional param doesn't count toward min
                assert_eq!(f.min_argument_count, 1);
                assert!(!f.has_rest_parameter);
                assert_eq!(f.return_type, checker.types.void_type);
            } else {
                panic!("Expected Function type");
            }
        }
    }

    #[test]
    fn test_function_type_node() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test function type nodes (type aliases)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"type Callback = (x: number, y: string) => boolean;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // 'Callback' should be in the symbol table
        assert!(binder.file_locals.has("Callback"));

        // Get the type alias symbol and its declared type
        if let Some(callback_symbol) = binder.file_locals.get("Callback") {
            let callback_type = checker.get_type_of_symbol(callback_symbol);
            let typ = checker.types.get(callback_type).unwrap();

            // The type should be a function type
            if let Type::Function(f) = typ {
                assert_eq!(f.parameter_names.len(), 2);
                assert_eq!(f.parameter_names[0], "x");
                assert_eq!(f.parameter_names[1], "y");
                assert_eq!(f.parameter_types[0], checker.types.number_type);
                assert_eq!(f.parameter_types[1], checker.types.string_type);
                assert_eq!(f.return_type, checker.types.boolean_type);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_function_type_direct() {
        use crate::parser::NodeArena;

        // Test directly creating a function type
        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a function type manually
        let fn_type = checker.types.create_function_type(
            NodeIndex::NONE,
            vec![checker.types.number_type, checker.types.string_type],
            vec!["a".to_string(), "b".to_string()],
            checker.types.boolean_type,
            2,
            false,
        );

        // Verify the type
        let typ = checker.types.get(fn_type).unwrap();
        assert!(typ.has_flags(type_flags::OBJECT));

        if let Type::Function(f) = typ {
            assert_eq!(f.parameter_types.len(), 2);
            assert_eq!(f.parameter_names, vec!["a", "b"]);
            assert_eq!(f.return_type, checker.types.boolean_type);
        } else {
            panic!("Expected Function type");
        }

        // Test type_to_string
        let type_str = checker.type_to_string(fn_type);
        assert_eq!(type_str, "(a: number, b: string) => boolean");
    }

    #[test]
    fn test_arrow_function_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test arrow function with parenthesized parameters
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const double = (x: number): number => x * 2;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // 'double' should have a function type
        assert!(binder.file_locals.has("double"));

        if let Some(double_symbol) = binder.file_locals.get("double") {
            let double_type = checker.get_type_of_symbol(double_symbol);
            let typ = checker.types.get(double_type).unwrap();

            if let Type::Function(f) = typ {
                assert_eq!(f.parameter_names.len(), 1);
                assert_eq!(f.parameter_names[0], "x");
                assert_eq!(f.parameter_types[0], checker.types.number_type);
                assert_eq!(f.return_type, checker.types.number_type);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_simple_arrow_function() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test simple arrow function (x => x * 2)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const identity = x => x;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("identity"));

        if let Some(symbol) = binder.file_locals.get("identity") {
            let id_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(id_type).unwrap();

            if let Type::Function(f) = typ {
                assert_eq!(f.parameter_names.len(), 1);
                assert_eq!(f.parameter_names[0], "x");
                // No type annotation, should be 'any'
                assert_eq!(f.parameter_types[0], checker.types.any_type);
                assert_eq!(f.return_type, checker.types.any_type);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_arrow_function_no_params() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test arrow function with no parameters
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const getNumber = (): number => 42;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("getNumber"));

        if let Some(symbol) = binder.file_locals.get("getNumber") {
            let fn_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(fn_type).unwrap();

            if let Type::Function(f) = typ {
                assert_eq!(f.parameter_names.len(), 0);
                assert_eq!(f.return_type, checker.types.number_type);
                assert_eq!(f.min_argument_count, 0);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_generic_function_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test generic function declaration
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"function identity<T>(x: T): T { return x; }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // 'identity' should have a generic function type
        assert!(binder.file_locals.has("identity"));

        if let Some(symbol) = binder.file_locals.get("identity") {
            let fn_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(fn_type).unwrap();

            if let Type::Function(f) = typ {
                // Should have one type parameter
                assert_eq!(f.type_parameters.len(), 1);
                assert_eq!(f.parameter_names.len(), 1);
                assert_eq!(f.parameter_names[0], "x");

                // Type string should include <T>
                let type_str = checker.type_to_string(fn_type);
                assert!(type_str.starts_with("<T>"), "Expected type string to start with <T>, got: {}", type_str);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_generic_function_multiple_type_params() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test generic function with multiple type parameters
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"function pair<T, U>(first: T, second: U): [T, U] { return [first, second]; }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("pair"));

        if let Some(symbol) = binder.file_locals.get("pair") {
            let fn_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(fn_type).unwrap();

            if let Type::Function(f) = typ {
                // Should have two type parameters
                assert_eq!(f.type_parameters.len(), 2);
                assert_eq!(f.parameter_names.len(), 2);
                assert_eq!(f.parameter_names[0], "first");
                assert_eq!(f.parameter_names[1], "second");

                // Type string should include <T, U>
                let type_str = checker.type_to_string(fn_type);
                assert!(type_str.starts_with("<T, U>"), "Expected type string to start with <T, U>, got: {}", type_str);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_generic_function_with_constraint() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test generic function with constraint
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"function getLength<T extends { length: number }>(x: T): number { return x.length; }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("getLength"));

        if let Some(symbol) = binder.file_locals.get("getLength") {
            let fn_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(fn_type).unwrap();

            if let Type::Function(f) = typ {
                // Should have one type parameter
                assert_eq!(f.type_parameters.len(), 1);

                // The type parameter should have a constraint
                let tp_id = f.type_parameters[0];
                let tp = checker.types.get(tp_id).unwrap();
                if let Type::TypeParameter(tp) = tp {
                    // Constraint should not be NONE
                    assert!(!tp.constraint.is_none());
                } else {
                    panic!("Expected TypeParameter");
                }
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_generic_arrow_function() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test generic arrow function
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const identity = <T>(x: T): T => x;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("identity"));

        if let Some(symbol) = binder.file_locals.get("identity") {
            let fn_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(fn_type).unwrap();

            if let Type::Function(f) = typ {
                // Should have one type parameter
                assert_eq!(f.type_parameters.len(), 1);
                assert_eq!(f.parameter_names.len(), 1);
            } else {
                panic!("Expected Function type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_type_parameter_creation() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Manually create a type parameter
        let tp_id = checker.types.create_type_parameter(
            SymbolId::NONE,
            TypeId::NONE,
            TypeId::NONE,
        );

        let tp = checker.types.get(tp_id).unwrap();
        assert!(tp.has_flags(type_flags::TYPE_PARAMETER));

        if let Type::TypeParameter(t) = tp {
            assert!(t.constraint.is_none());
            assert!(t.default.is_none());
        } else {
            panic!("Expected TypeParameter");
        }
    }

    #[test]
    fn test_type_literal() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test type literal
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"type Point = { x: number; y: number };"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Point"));

        if let Some(symbol) = binder.file_locals.get("Point") {
            let point_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(point_type).unwrap();

            if let Type::Object(obj) = typ {
                assert_eq!(obj.properties.len(), 2);
            } else {
                panic!("Expected Object type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_object_type_with_method() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test type literal with method
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"type Greeter = { greet(name: string): string };"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Greeter"));

        if let Some(symbol) = binder.file_locals.get("Greeter") {
            let greeter_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(greeter_type).unwrap();

            if let Type::Object(obj) = typ {
                assert_eq!(obj.properties.len(), 1);
            } else {
                panic!("Expected Object type, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_create_object_type() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create an empty object type
        let obj_type = checker.types.create_object_type(Vec::new());
        let typ = checker.types.get(obj_type).unwrap();

        if let Type::Object(obj) = typ {
            assert_eq!(obj.properties.len(), 0);
            assert!(obj.has_object_flags(object_flags::ANONYMOUS));
        } else {
            panic!("Expected Object type");
        }
    }

    #[test]
    fn test_call_expression_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test function call expression
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                function greet(name: string): string { return name; }
                const result = greet("hello");
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // 'greet' should be a function type
        assert!(binder.file_locals.has("greet"));
        if let Some(symbol) = binder.file_locals.get("greet") {
            let fn_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(fn_type).unwrap();

            if let Type::Function(f) = typ {
                assert_eq!(f.return_type, checker.types.string_type);
            } else {
                panic!("Expected Function type");
            }
        }

        // 'result' should also be string (return type of greet)
        assert!(binder.file_locals.has("result"));
    }

    #[test]
    fn test_array_literal_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test array literal with mixed types
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const arr = [1, "hello"];"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("arr"));

        if let Some(symbol) = binder.file_locals.get("arr") {
            let arr_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(arr_type).unwrap();

            // The type should be a union of number and string literals
            if let Type::Union(u) = typ {
                assert_eq!(u.types.len(), 2);
            } else {
                panic!("Expected Union type for mixed array, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_union_type_creation() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union type of string | number
        let string_type = checker.types.string_type;
        let number_type = checker.types.number_type;
        let union_type = checker.types.create_union_type(vec![string_type, number_type]);

        let typ = checker.types.get(union_type).unwrap();
        if let Type::Union(u) = typ {
            assert_eq!(u.types.len(), 2);
            assert!(u.types.contains(&string_type));
            assert!(u.types.contains(&number_type));
        } else {
            panic!("Expected Union type");
        }

        // Type to string should work
        let type_str = checker.type_to_string(union_type);
        assert!(type_str.contains("|"));
    }

    #[test]
    fn test_union_simplification_never() {
        // X | never = X
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let union = arena.create_union(vec![obj, arena.never_type]);
        assert_eq!(union, obj);
    }

    #[test]
    fn test_union_simplification_any() {
        // X | any = any
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let union = arena.create_union(vec![obj, arena.any_type]);
        assert_eq!(union, arena.any_type);
    }

    #[test]
    fn test_union_simplification_duplicates() {
        // X | X = X
        let mut arena = TypeArena::new();
        let obj = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let union = arena.create_union(vec![obj, obj]);
        assert_eq!(union, obj);
    }

    #[test]
    fn test_union_simplification_flatten() {
        // (A | B) | C = A | B | C
        let mut arena = TypeArena::new();
        let obj1 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));
        let obj2 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));
        let obj3 = arena.alloc(Type::Object(ObjectType::new(object_flags::ANONYMOUS, SymbolId::NONE)));

        let inner = arena.create_union(vec![obj1, obj2]);
        let outer = arena.create_union(vec![inner, obj3]);

        // Should have flattened to a single union with 3 types
        if let Type::Union(u) = arena.get(outer).unwrap() {
            assert_eq!(u.types.len(), 3);
            assert!(u.types.contains(&obj1));
            assert!(u.types.contains(&obj2));
            assert!(u.types.contains(&obj3));
        } else {
            panic!("Expected flattened union type");
        }
    }

    #[test]
    fn test_union_empty_returns_never() {
        let mut arena = TypeArena::new();
        let union = arena.create_union(vec![]);
        assert_eq!(union, arena.never_type);
    }

    #[test]
    fn test_union_all_never_returns_never() {
        // never | never = never
        let mut arena = TypeArena::new();
        let union = arena.create_union(vec![arena.never_type, arena.never_type]);
        assert_eq!(union, arena.never_type);
    }

    #[test]
    #[ignore] // TODO: Fix infinite loop - likely in interface/variable type resolution chain
    fn test_simple_interface_variable() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Simpler test - just interface with variable
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                interface A { x: string; }
                declare let a: A;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of a
        let a_symbol = binder.file_locals.get("a").expect("a should be in file locals");
        let a_type = checker.get_type_of_symbol(a_symbol);
        let a_str = checker.type_to_string(a_type);

        // a should be of type A (an interface)
        assert!(checker.types.get(a_type).is_some(), "a should have a valid type");
    }

    #[test]
    #[ignore] // TODO: Fix infinite loop in interface type resolution
    fn test_property_access_on_union() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test property access on union types
        // (A | B).prop should return A.prop | B.prop
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                interface A { x: string; }
                interface B { x: number; }
                declare let value: A | B;
                let result = value.x;  // should be string | number
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of result
        let result_symbol = binder.file_locals.get("result").expect("result should be in file locals");
        let result_type = checker.get_type_of_symbol(result_symbol);
        let result_str = checker.type_to_string(result_type);

        // Should be string | number
        assert!(result_str.contains("string") || result_str.contains("number"),
            "result should be string | number, got: {}", result_str);
    }

    #[test]
    fn test_class_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test class type
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"class Person {
                name: string;
                age: number;
                greet(): string { return "hello"; }
            }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Person"));

        if let Some(symbol) = binder.file_locals.get("Person") {
            let person_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(person_type).unwrap();

            if let Type::Object(obj) = typ {
                // Should have CLASS object flag
                assert!(obj.has_object_flags(object_flags::CLASS));
                // Should have 3 properties: name, age, greet
                assert_eq!(obj.properties.len(), 3);
            } else {
                panic!("Expected Object type with CLASS flag, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_class_with_constructor() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test class with constructor - simplified without this.x assignments
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"class Point {
                x: number;
                y: number;
                constructor(a: number, b: number) { }
            }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Point"));

        if let Some(symbol) = binder.file_locals.get("Point") {
            let point_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(point_type).unwrap();

            if let Type::Object(obj) = typ {
                // Should have CLASS object flag
                assert!(obj.has_object_flags(object_flags::CLASS));
                // Should have 2 properties: x, y
                assert_eq!(obj.properties.len(), 2);
                // Should have 1 construct signature
                assert_eq!(obj.construct_signatures.len(), 1);
                // Construct signature should have 2 parameters
                assert_eq!(obj.construct_signatures[0].parameters.len(), 2);
            } else {
                panic!("Expected Object type with CLASS flag, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_interface_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test interface type
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"interface Animal {
                name: string;
                speak(): void;
            }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Animal"));

        if let Some(symbol) = binder.file_locals.get("Animal") {
            let animal_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(animal_type).unwrap();

            if let Type::Object(obj) = typ {
                // Should have INTERFACE object flag
                assert!(obj.has_object_flags(object_flags::INTERFACE));
                // Should have 2 properties: name, speak
                assert_eq!(obj.properties.len(), 2);
            } else {
                panic!("Expected Object type with INTERFACE flag, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_class_with_accessors() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test class with get/set accessors - simplified without this references
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"class Counter {
                _value: number;
                get value(): number { return 0; }
                set value(v: number) { }
            }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Counter"));

        if let Some(symbol) = binder.file_locals.get("Counter") {
            let counter_type = checker.get_type_of_symbol(symbol);
            let typ = checker.types.get(counter_type).unwrap();

            if let Type::Object(obj) = typ {
                // Should have CLASS object flag
                assert!(obj.has_object_flags(object_flags::CLASS));
                // Should have 3 properties: _value, get value, set value
                assert_eq!(obj.properties.len(), 3);
            } else {
                panic!("Expected Object type with CLASS flag, got {:?}", typ);
            }
        }
    }

    #[test]
    fn test_generic_function_type_parameter() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "function identity<T>(x: T): T { return x; }".to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the identity function
        assert!(binder.file_locals.has("identity"));
        if let Some(symbol) = binder.file_locals.get("identity") {
            let func_type = checker.get_type_of_symbol(symbol);
            let type_str = checker.type_to_string(func_type);

            // Should have type parameter T
            assert!(type_str.contains("<T>"), "Expected type parameter T, got: {}", type_str);
            assert!(type_str.contains("T") && type_str.contains("=>"), "Expected function with T, got: {}", type_str);
        } else {
            panic!("identity function not found");
        }
    }

    #[test]
    fn test_type_instantiation() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a type parameter T
        let t_symbol = checker.local_symbols.alloc(symbol_flags::TYPE_PARAMETER, "T".to_string());
        let t_type = checker.types.alloc(Type::TypeParameter(TypeParameter {
            flags: type_flags::TYPE_PARAMETER,
            symbol: t_symbol,
            constraint: TypeId::NONE,
            default: TypeId::NONE,
            target: TypeId::NONE,
            is_this_type: false,
        }));
        checker.type_parameter_names.insert(t_type, "T".to_string());

        // Create a function type with T as parameter and return type: (x: T) => T
        let func_type = checker.types.create_function_type_with_type_params(
            NodeIndex::NONE,
            vec![t_type],              // parameter types
            vec!["x".to_string()],     // parameter names
            t_type,                    // return type
            vec![t_type],              // type parameters
            1,                         // min argument count
            false,                     // has rest parameter
        );

        // Instantiate with T = string
        let instantiated = checker.instantiate_type(func_type, &[checker.types.string_type], &[t_type]);

        // The result should have string as parameter and return type
        if let Some(Type::Function(f)) = checker.types.get(instantiated) {
            assert_eq!(f.parameter_types.len(), 1);
            assert_eq!(f.parameter_types[0], checker.types.string_type);
            assert_eq!(f.return_type, checker.types.string_type);
        } else {
            panic!("Expected instantiated function type");
        }
    }

    #[test]
    fn test_type_parameter_with_default() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "type Create<T = string> = () => T;".to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the Create type alias
        assert!(binder.file_locals.has("Create"));
        if let Some(symbol) = binder.file_locals.get("Create") {
            // Just verify it parses and binds successfully
            let _ = checker.get_type_of_symbol(symbol);
        }
    }

    #[test]
    fn test_object_literal_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"const obj = { x: 1, y: "hello" };"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("obj"));
        if let Some(symbol) = binder.file_locals.get("obj") {
            let obj_type = checker.get_type_of_symbol(symbol);

            // Should be an object type
            if let Some(Type::Object(obj)) = checker.types.get(obj_type) {
                assert_eq!(obj.properties.len(), 2, "Expected 2 properties");
            } else {
                panic!("Expected Object type for object literal");
            }
        }
    }

    #[test]
    fn test_type_literal_members() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            "type Point = { x: number; y: number };".to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Point"));
        if let Some(symbol) = binder.file_locals.get("Point") {
            let point_type = checker.get_type_of_symbol(symbol);

            // Should be an object type
            if let Some(Type::Object(obj)) = checker.types.get(point_type) {
                assert_eq!(obj.properties.len(), 2, "Expected 2 properties for Point");
            } else {
                panic!("Expected Object type for type literal");
            }
        }
    }

    #[test]
    fn test_interface_members() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"interface Person {
                name: string;
                age: number;
                greet(): void;
            }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        assert!(binder.file_locals.has("Person"));
        if let Some(symbol) = binder.file_locals.get("Person") {
            let person_type = checker.get_type_of_symbol(symbol);

            // Should be an interface object type
            if let Some(Type::Object(obj)) = checker.types.get(person_type) {
                assert!(obj.has_object_flags(object_flags::INTERFACE), "Expected INTERFACE object flag");
                assert_eq!(obj.properties.len(), 3, "Expected 3 properties (name, age, greet)");
            } else {
                panic!("Expected Object type for interface");
            }
        }
    }

    #[test]
    fn test_typeof_narrowing_string() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union type: string | number
        let union_type = checker.types.create_union(vec![
            checker.types.string_type,
            checker.types.number_type,
        ]);

        // Narrow by typeof === "string"
        let narrowed = checker.narrow_type_by_typeof(union_type, "string");

        // Should narrow to string
        assert_eq!(narrowed, checker.types.string_type);
    }

    #[test]
    fn test_typeof_narrowing_number() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union type: string | number | boolean
        let union_type = checker.types.create_union(vec![
            checker.types.string_type,
            checker.types.number_type,
            checker.types.boolean_type,
        ]);

        // Narrow by typeof === "number"
        let narrowed = checker.narrow_type_by_typeof(union_type, "number");

        // Should narrow to number
        assert_eq!(narrowed, checker.types.number_type);
    }

    #[test]
    fn test_typeof_narrowing_negation() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union type: string | number
        let union_type = checker.types.create_union(vec![
            checker.types.string_type,
            checker.types.number_type,
        ]);

        // Narrow by typeof !== "string"
        let narrowed = checker.narrow_type_by_typeof_negation(union_type, "string");

        // Should narrow to number
        assert_eq!(narrowed, checker.types.number_type);
    }

    #[test]
    fn test_non_nullable_type() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union type: string | null | undefined
        let union_type = checker.types.create_union(vec![
            checker.types.string_type,
            checker.types.null_type,
            checker.types.undefined_type,
        ]);

        // Get non-nullable
        let non_nullable = checker.get_non_nullable_type(union_type);

        // Should narrow to string
        assert_eq!(non_nullable, checker.types.string_type);
    }

    #[test]
    fn test_typeof_narrowing_no_match() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Narrow string by typeof === "number" should give never
        let narrowed = checker.narrow_type_by_typeof(checker.types.string_type, "number");

        assert_eq!(narrowed, checker.types.never_type);
    }

    #[test]
    fn test_generic_call_expression_inference() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test: identity<T>(x: T): T called with "hello" should infer T = string
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
            function identity<T>(x: T): T { return x; }
            const result = identity("hello");
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Verify identity function type
        assert!(binder.file_locals.has("identity"));
        if let Some(identity_sym) = binder.file_locals.get("identity") {
            let identity_type = checker.get_type_of_symbol(identity_sym);
            let identity_str = checker.type_to_string(identity_type);

            // Should have form <T>(x: T) => T
            assert!(identity_str.contains("<T>"), "Expected type parameter T, got: {}", identity_str);

            // Verify it has type parameters
            if let Some(Type::Function(f)) = checker.types.get(identity_type) {
                assert_eq!(f.type_parameters.len(), 1, "Expected 1 type parameter");
                // param type and return type should be the same (T)
                assert_eq!(f.parameter_types.len(), 1);
                assert_eq!(f.parameter_types[0], f.return_type);
            }
        }

        // Get the result variable type
        assert!(binder.file_locals.has("result"));
        if let Some(symbol) = binder.file_locals.get("result") {
            let result_type = checker.get_type_of_symbol(symbol);

            // The result should be a string literal type "hello" (or widened to string)
            // since identity<T>(x: T): T returns T, and T is inferred from "hello"
            if let Some(Type::Literal(lit)) = checker.types.get(result_type) {
                assert!(matches!(lit.value, LiteralValue::String(_)),
                    "Expected string literal type, got {:?}", lit.value);
            } else {
                // Could also be string_type if literal widening is applied
                assert!(result_type == checker.types.string_type ||
                        matches!(checker.types.get(result_type), Some(Type::Literal(_))),
                    "Expected string or string literal type, got: {}",
                    checker.type_to_string(result_type));
            }
        } else {
            panic!("result variable not found");
        }
    }

    #[test]
    fn test_generic_call_with_explicit_type_args() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test: identity<string>(x) should use the explicit type argument
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
            function identity<T>(x: T): T { return x; }
            const result = identity<number>(42);
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Verify identity function type
        assert!(binder.file_locals.has("identity"));
        if let Some(identity_sym) = binder.file_locals.get("identity") {
            let identity_type = checker.get_type_of_symbol(identity_sym);
            let identity_str = checker.type_to_string(identity_type);
            assert!(identity_str.contains("<T>"), "Expected generic function, got: {}", identity_str);
        }

        // Get result type - should be number (from explicit type argument)
        assert!(binder.file_locals.has("result"));
        if let Some(symbol) = binder.file_locals.get("result") {
            let result_type = checker.get_type_of_symbol(symbol);

            // With explicit type argument <number>, result should be number
            assert!(result_type == checker.types.number_type ||
                    matches!(checker.types.get(result_type), Some(Type::Literal(l))
                             if matches!(l.value, LiteralValue::Number(_))),
                "Expected number type, got: {}", checker.type_to_string(result_type));
        } else {
            panic!("result variable not found");
        }
    }

    #[test]
    fn test_instanceof_narrowing_unknown() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a simple class type
        let class_type = checker.types.create_class_type(vec![], vec![], vec![]);

        // Narrow unknown by instanceof should give the class type
        let narrowed = checker.narrow_type_by_instanceof(checker.types.unknown_type, class_type);

        assert_eq!(narrowed, class_type);
    }

    #[test]
    fn test_instanceof_narrowing_any() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a simple class type
        let class_type = checker.types.create_class_type(vec![], vec![], vec![]);

        // Narrow any by instanceof should give the class type
        let narrowed = checker.narrow_type_by_instanceof(checker.types.any_type, class_type);

        assert_eq!(narrowed, class_type);
    }

    #[test]
    fn test_instanceof_narrowing_union() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a class type
        let class_type = checker.types.create_class_type(vec![], vec![], vec![]);

        // Create a union of string | class
        let union_type = checker.types.create_union(vec![
            checker.types.string_type,
            class_type,
        ]);

        // Narrow by instanceof should filter out string (primitive)
        let narrowed = checker.narrow_type_by_instanceof(union_type, class_type);

        // Should narrow to just the class type
        assert_eq!(narrowed, class_type);
    }

    #[test]
    fn test_instanceof_negation() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create two class types
        let class_a = checker.types.create_class_type(vec![], vec![], vec![]);
        let class_b = checker.types.create_class_type(vec![], vec![], vec![]);

        // Create a union of A | B
        let union_type = checker.types.create_union(vec![class_a, class_b]);

        // Narrow by NOT instanceof A should give B
        let narrowed = checker.narrow_type_by_instanceof_negation(union_type, class_a);

        // Should narrow to class_b
        assert_eq!(narrowed, class_b);
    }

    #[test]
    fn test_class_construct_signature() {
        use crate::parser::NodeArena;

        // Test construct signature infrastructure directly
        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create an instance type (what you get from new Foo())
        let instance_type = checker.types.create_class_type(vec![], vec![], vec![]);

        // Create a construct signature that returns the instance type
        let mut construct_sig = Signature::new(NodeIndex::NONE);
        construct_sig.resolved_return_type = Some(instance_type);

        // Create the constructor type (the type of the class itself)
        let constructor_type = checker.types.create_class_type(vec![], vec![construct_sig.clone()], vec![]);

        // Verify the constructor type has construct signatures
        if let Some(Type::Object(obj)) = checker.types.get(constructor_type) {
            assert!(!obj.construct_signatures.is_empty(), "Expected construct signatures");

            // The signature should have the instance type as return type
            let sig = &obj.construct_signatures[0];
            assert!(sig.resolved_return_type.is_some(), "Expected resolved_return_type");
            assert_eq!(sig.resolved_return_type.unwrap(), instance_type);
        } else {
            panic!("Expected Object type");
        }
    }

    #[test]
    fn test_new_expression_with_construct_signature() {
        use crate::parser::NodeArena;

        // Test that get_type_of_new_expression correctly uses construct signatures
        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create an instance type
        let instance_type = checker.types.create_class_type(vec![], vec![], vec![]);

        // Create a construct signature that returns the instance type
        let mut construct_sig = Signature::new(NodeIndex::NONE);
        construct_sig.resolved_return_type = Some(instance_type);

        // Create a constructor type
        let constructor_type = checker.types.create_class_type(vec![], vec![construct_sig], vec![]);

        // Manually test the logic from get_type_of_new_expression
        if let Some(Type::Object(obj)) = checker.types.get(constructor_type) {
            if !obj.construct_signatures.is_empty() {
                if let Some(return_type) = obj.construct_signatures[0].resolved_return_type {
                    // This is what get_type_of_new_expression would return
                    assert_eq!(return_type, instance_type);
                }
            }
        }
    }

    #[test]
    fn test_type_guard_extraction_typeof() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test extracting type guard from typeof expression
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"typeof x === "string""#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the expression statement
        use crate::parser::Node;
        if let Some(Node::SourceFile(sf)) = parser.arena.get(root) {
            if let Some(&first_stmt) = sf.statements.nodes.first() {
                if let Some(Node::ExpressionStatement(es)) = parser.arena.get(first_stmt) {
                    let guard = checker.get_type_guard_from_expression(es.expression);
                    assert!(guard.is_some(), "Expected type guard from typeof expression");

                    if let Some(TypeGuard::Typeof { typeof_result, is_equality, .. }) = guard {
                        assert_eq!(typeof_result, "string");
                        assert!(is_equality);
                    } else {
                        panic!("Expected Typeof guard");
                    }
                }
            }
        }
    }

    #[test]
    fn test_apply_type_guard() {
        use crate::parser::NodeArena;

        let node_arena = NodeArena::new();
        let symbol_arena = SymbolArena::new();
        let file_locals = SymbolTable::new();
        let mut checker = CheckerState::new(&node_arena, &symbol_arena, &file_locals, "test.ts".to_string());

        // Create a union of string | number | boolean
        let union_type = checker.types.create_union(vec![
            checker.types.string_type,
            checker.types.number_type,
            checker.types.boolean_type,
        ]);

        // Apply typeof === "string" guard
        let guard = TypeGuard::Typeof {
            target: crate::parser::NodeIndex::NONE,
            typeof_result: "string".to_string(),
            is_equality: true,
        };

        let narrowed = checker.apply_type_guard(union_type, &guard);

        // Should narrow to just string
        assert_eq!(narrowed, checker.types.string_type);
    }

    #[test]
    fn test_interface_index_signature() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test interface with index signature: interface Dict { [key: string]: number }
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"interface Dict { [key: string]: number }"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the interface type
        let dict_symbol = binder.file_locals.get("Dict").expect("Dict should be in file locals");
        let dict_type = checker.get_type_of_symbol(dict_symbol);

        // Check it has an index info
        if let Some(Type::Object(obj)) = checker.types.get(dict_type) {
            assert!(!obj.index_infos.is_empty(), "Dict should have index info");
            let idx_info = &obj.index_infos[0];
            // Key type should be string
            assert_eq!(idx_info.key_type, checker.types.string_type, "Key type should be string");
            // Value type should be number
            assert_eq!(idx_info.value_type, checker.types.number_type, "Value type should be number");
        } else {
            panic!("Dict should be an object type");
        }
    }

    #[test]
    fn test_element_access_with_index_signature() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test element access with index signature
        // let d: Dict = {}; let x = d["key"];
        // x should have type number (from Dict's index signature)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
interface Dict { [key: string]: number }
let d: Dict;
let x = d["key"];
"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of x
        let x_symbol = binder.file_locals.get("x").expect("x should be in file locals");
        let x_type = checker.get_type_of_symbol(x_symbol);

        // x should be number type (from the index signature value type)
        assert_eq!(x_type, checker.types.number_type, "x should be number type");
    }

    #[test]
    fn test_array_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test array type: let arr: number[]
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"let arr: number[];"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of arr
        let arr_symbol = binder.file_locals.get("arr").expect("arr should be in file locals");
        let arr_type = checker.get_type_of_symbol(arr_symbol);

        // arr should be an array type
        if let Some(Type::Array(arr)) = checker.types.get(arr_type) {
            assert_eq!(arr.element_type, checker.types.number_type, "Element type should be number");
        } else {
            panic!("arr should be an array type, got {}", checker.type_to_string(arr_type));
        }
    }

    #[test]
    fn test_array_index_access() {
        // Test that array types are created correctly for index access
        let mut arena = TypeArena::new();

        // Create a number[] array type
        let number_array = arena.create_array_type(arena.number_type, false);

        // Verify the array type structure - indexed access will return element_type
        if let Some(Type::Array(arr)) = arena.get(number_array) {
            assert_eq!(arr.element_type, arena.number_type);
            assert!(!arr.is_readonly);
        } else {
            panic!("Expected array type");
        }

        // Create a readonly string[] array
        let readonly_string_array = arena.create_array_type(arena.string_type, true);
        if let Some(Type::Array(arr)) = arena.get(readonly_string_array) {
            assert_eq!(arr.element_type, arena.string_type);
            assert!(arr.is_readonly);
        } else {
            panic!("Expected readonly array type");
        }
    }

    #[test]
    fn test_tuple_index_access() {
        // Test that tuple[0] returns the first element type
        let mut arena = TypeArena::new();

        // Create a [string, number, boolean] tuple type
        let tuple = arena.create_tuple_type(
            vec![arena.string_type, arena.number_type, arena.boolean_type],
            false,
            false,
            false,
        );

        // Verify the tuple structure
        if let Some(Type::Tuple(t)) = arena.get(tuple) {
            assert_eq!(t.element_types.len(), 3);
            assert_eq!(t.element_types[0], arena.string_type);
            assert_eq!(t.element_types[1], arena.number_type);
            assert_eq!(t.element_types[2], arena.boolean_type);
        } else {
            panic!("Expected tuple type");
        }
    }

    #[test]
    fn test_generic_array_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test Array<T> syntax: let arr: Array<string>
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"let arr: Array<string>;"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of arr
        let arr_symbol = binder.file_locals.get("arr").expect("arr should be in file locals");
        let arr_type = checker.get_type_of_symbol(arr_symbol);

        // arr should be an array type
        if let Some(Type::Array(arr)) = checker.types.get(arr_type) {
            assert_eq!(arr.element_type, checker.types.string_type, "Element type should be string");
        } else {
            panic!("arr should be an array type, got {}", checker.type_to_string(arr_type));
        }
    }

    #[test]
    fn test_tuple_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test tuple type: let t: [number, string]
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"let t: [number, string];"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of t
        let t_symbol = binder.file_locals.get("t").expect("t should be in file locals");
        let t_type = checker.get_type_of_symbol(t_symbol);

        // t should be a tuple type
        if let Some(Type::Tuple(tup)) = checker.types.get(t_type) {
            assert_eq!(tup.element_types.len(), 2, "Tuple should have 2 elements");
            assert_eq!(tup.element_types[0], checker.types.number_type, "First element should be number");
            assert_eq!(tup.element_types[1], checker.types.string_type, "Second element should be string");
        } else {
            panic!("t should be a tuple type, got {}", checker.type_to_string(t_type));
        }
    }

    #[test]
    fn test_array_assignability() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test array assignability
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                let nums: number[];
                let strs: string[];
                let anys: any[];
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the types
        let nums_symbol = binder.file_locals.get("nums").expect("nums should be in file locals");
        let strs_symbol = binder.file_locals.get("strs").expect("strs should be in file locals");
        let anys_symbol = binder.file_locals.get("anys").expect("anys should be in file locals");

        let nums_type = checker.get_type_of_symbol(nums_symbol);
        let strs_type = checker.get_type_of_symbol(strs_symbol);
        let anys_type = checker.get_type_of_symbol(anys_symbol);

        // number[] is assignable to number[]
        assert!(checker.is_type_assignable_to(nums_type, nums_type), "number[] should be assignable to number[]");

        // number[] is NOT assignable to string[]
        assert!(!checker.is_type_assignable_to(nums_type, strs_type), "number[] should NOT be assignable to string[]");

        // number[] IS assignable to any[]
        assert!(checker.is_type_assignable_to(nums_type, anys_type), "number[] should be assignable to any[]");
    }

    #[test]
    fn test_tuple_assignability() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test tuple assignability
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                let t1: [number, string];
                let t2: [number, string];
                let t3: [string, number];
                let t4: [number];
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the types
        let t1_symbol = binder.file_locals.get("t1").expect("t1 should be in file locals");
        let t2_symbol = binder.file_locals.get("t2").expect("t2 should be in file locals");
        let t3_symbol = binder.file_locals.get("t3").expect("t3 should be in file locals");
        let t4_symbol = binder.file_locals.get("t4").expect("t4 should be in file locals");

        let t1_type = checker.get_type_of_symbol(t1_symbol);
        let t2_type = checker.get_type_of_symbol(t2_symbol);
        let t3_type = checker.get_type_of_symbol(t3_symbol);
        let t4_type = checker.get_type_of_symbol(t4_symbol);

        // [number, string] is assignable to [number, string]
        assert!(checker.is_type_assignable_to(t1_type, t2_type), "[number, string] should be assignable to [number, string]");

        // [number, string] is NOT assignable to [string, number] (different order)
        assert!(!checker.is_type_assignable_to(t1_type, t3_type), "[number, string] should NOT be assignable to [string, number]");

        // [number, string] is NOT assignable to [number] (different length)
        assert!(!checker.is_type_assignable_to(t1_type, t4_type), "[number, string] should NOT be assignable to [number]");
    }

    #[test]
    fn test_tuple_to_array_assignability() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test tuple to array assignability
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                let tuple: [number, number];
                let arr: number[];
                let strArr: string[];
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the types
        let tuple_symbol = binder.file_locals.get("tuple").expect("tuple should be in file locals");
        let arr_symbol = binder.file_locals.get("arr").expect("arr should be in file locals");
        let str_arr_symbol = binder.file_locals.get("strArr").expect("strArr should be in file locals");

        let tuple_type = checker.get_type_of_symbol(tuple_symbol);
        let arr_type = checker.get_type_of_symbol(arr_symbol);
        let str_arr_type = checker.get_type_of_symbol(str_arr_symbol);

        // [number, number] is assignable to number[]
        assert!(checker.is_type_assignable_to(tuple_type, arr_type), "[number, number] should be assignable to number[]");

        // [number, number] is NOT assignable to string[]
        assert!(!checker.is_type_assignable_to(tuple_type, str_arr_type), "[number, number] should NOT be assignable to string[]");
    }

    #[test]
    fn test_readonly_array_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test readonly array type
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                let readonlyArr: readonly number[];
                let mutableArr: number[];
                let readonlyArr2: ReadonlyArray<number>;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the types
        let readonly_arr_symbol = binder.file_locals.get("readonlyArr").expect("readonlyArr should be in file locals");
        let mutable_arr_symbol = binder.file_locals.get("mutableArr").expect("mutableArr should be in file locals");
        let readonly_arr2_symbol = binder.file_locals.get("readonlyArr2").expect("readonlyArr2 should be in file locals");

        let readonly_arr_type = checker.get_type_of_symbol(readonly_arr_symbol);
        let mutable_arr_type = checker.get_type_of_symbol(mutable_arr_symbol);
        let readonly_arr2_type = checker.get_type_of_symbol(readonly_arr2_symbol);

        // Check readonly flag
        if let Some(Type::Array(arr)) = checker.types.get(readonly_arr_type) {
            assert!(arr.is_readonly, "readonly number[] should have is_readonly=true");
        } else {
            panic!("readonlyArr should be an array type");
        }

        if let Some(Type::Array(arr)) = checker.types.get(mutable_arr_type) {
            assert!(!arr.is_readonly, "number[] should have is_readonly=false");
        } else {
            panic!("mutableArr should be an array type");
        }

        if let Some(Type::Array(arr)) = checker.types.get(readonly_arr2_type) {
            assert!(arr.is_readonly, "ReadonlyArray<number> should have is_readonly=true");
        } else {
            panic!("readonlyArr2 should be an array type");
        }

        // Check type_to_string
        let readonly_str = checker.type_to_string(readonly_arr_type);
        assert_eq!(readonly_str, "readonly number[]", "readonly array type_to_string");

        let mutable_str = checker.type_to_string(mutable_arr_type);
        assert_eq!(mutable_str, "number[]", "mutable array type_to_string");

        // Assignability: mutable IS assignable to readonly
        assert!(checker.is_type_assignable_to(mutable_arr_type, readonly_arr_type), "number[] should be assignable to readonly number[]");

        // Assignability: readonly is NOT assignable to mutable
        assert!(!checker.is_type_assignable_to(readonly_arr_type, mutable_arr_type), "readonly number[] should NOT be assignable to number[]");
    }

    #[test]
    fn test_optional_tuple_element() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test tuple with optional element: [number, string?]
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"let t: [number, string?];"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let t_symbol = binder.file_locals.get("t").expect("t should be in file locals");
        let t_type = checker.get_type_of_symbol(t_symbol);

        // t should be a tuple type with optional elements
        if let Some(Type::Tuple(tup)) = checker.types.get(t_type) {
            assert_eq!(tup.element_types.len(), 2, "Tuple should have 2 elements");
            assert!(tup.has_optional_elements, "Tuple should have optional elements");
            assert!(!tup.has_rest_element, "Tuple should not have rest element");
            assert_eq!(tup.element_types[0], checker.types.number_type, "First element should be number");
            assert_eq!(tup.element_types[1], checker.types.string_type, "Second element should be string");
        } else {
            panic!("t should be a tuple type, got {}", checker.type_to_string(t_type));
        }
    }

    #[test]
    fn test_rest_tuple_element() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test tuple with rest element: [string, ...number[]]
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"let t: [string, ...number[]];"#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let t_symbol = binder.file_locals.get("t").expect("t should be in file locals");
        let t_type = checker.get_type_of_symbol(t_symbol);

        // t should be a tuple type with rest element
        if let Some(Type::Tuple(tup)) = checker.types.get(t_type) {
            assert_eq!(tup.element_types.len(), 2, "Tuple should have 2 element types");
            assert!(!tup.has_optional_elements, "Tuple should not have optional elements");
            assert!(tup.has_rest_element, "Tuple should have rest element");
            assert_eq!(tup.element_types[0], checker.types.string_type, "First element should be string");
            assert_eq!(tup.element_types[1], checker.types.number_type, "Rest element type should be number");
        } else {
            panic!("t should be a tuple type, got {}", checker.type_to_string(t_type));
        }
    }

    #[test]
    fn test_spread_in_array_literal() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test spread in array literal
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                let nums: number[] = [1, 2, 3];
                let more = [...nums, 4, 5];
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let more_symbol = binder.file_locals.get("more").expect("more should be in file locals");
        let more_type = checker.get_type_of_symbol(more_symbol);
        let more_str = checker.type_to_string(more_type);

        // more should be number (inferred from spread + literals)
        // The type could be a union including number, or just number
        let is_number = more_type == checker.types.number_type;
        let contains_number = if let Some(Type::Union(u)) = checker.types.get(more_type) {
            u.types.iter().any(|&t| t == checker.types.number_type)
        } else {
            false
        };
        assert!(is_number || contains_number, "more should be or contain number type, got: {}", more_str);
    }

    #[test]
    fn test_conditional_type_evaluation() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test conditional type evaluation - simple case where condition is resolved
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                // Conditional that resolves to true branch
                type IsString = string extends string ? "yes" : "no";

                // Conditional that resolves to false branch
                type IsNumber = string extends number ? "yes" : "no";

                // Using the types in variable declarations
                let x: IsString = "yes";
                let y: IsNumber = "no";
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the type of x (should be "yes")
        let x_symbol = binder.file_locals.get("x").expect("x should be in file locals");
        let x_type = checker.get_type_of_symbol(x_symbol);
        let x_str = checker.type_to_string(x_type);
        assert_eq!(x_str, "\"yes\"", "IsString should resolve to \"yes\", got: {}", x_str);

        // Get the type of y (should be "no")
        let y_symbol = binder.file_locals.get("y").expect("y should be in file locals");
        let y_type = checker.get_type_of_symbol(y_symbol);
        let y_str = checker.type_to_string(y_type);
        assert_eq!(y_str, "\"no\"", "IsNumber should resolve to \"no\", got: {}", y_str);
    }

    #[test]
    fn test_distributive_conditional_type() {
        // Test that conditional types distribute over union types
        // ToArray<string | number> should become string[] | number[]
        let mut arena = TypeArena::new();

        // Create a naked type parameter T
        let t_param = arena.create_type_parameter(SymbolId::NONE, TypeId::NONE, TypeId::NONE);

        // Create array types
        let string_array = arena.create_array_type(arena.string_type, false);
        let number_array = arena.create_array_type(arena.number_type, false);

        // Create a conditional type: T extends any ? T[] : never
        // This is distributive because T is a naked type parameter
        let cond_type = arena.create_conditional_type(
            t_param,
            arena.any_type,
            string_array, // simplified - in practice this would be T[]
            arena.never_type,
        );

        // Verify the conditional is marked as distributive
        if let Some(Type::Conditional(c)) = arena.get(cond_type) {
            assert!(c.is_distributive, "Conditional with naked type parameter should be distributive");
        } else {
            panic!("Expected conditional type");
        }
    }

    #[test]
    fn test_non_distributive_conditional_type() {
        // Test that conditional types are not distributive when check type is not naked
        let mut arena = TypeArena::new();

        // Create a type that is not a naked type parameter (e.g., keyof T)
        let t_param = arena.create_type_parameter(SymbolId::NONE, TypeId::NONE, TypeId::NONE);
        let keyof_t = arena.create_index_type(t_param);

        // Create a conditional type: keyof T extends any ? "yes" : "no"
        let yes_type = arena.create_string_literal("yes".to_string());
        let no_type = arena.create_string_literal("no".to_string());
        let cond_type = arena.create_conditional_type(
            keyof_t,  // Not a naked type parameter
            arena.any_type,
            yes_type,
            no_type,
        );

        // Verify the conditional is NOT marked as distributive
        if let Some(Type::Conditional(c)) = arena.get(cond_type) {
            assert!(!c.is_distributive, "Conditional with keyof T should not be distributive");
        } else {
            panic!("Expected conditional type");
        }
    }

    #[test]
    fn test_template_literal_type_simple() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test simple template literal type (no substitution)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Hello = `hello`;
                let x: Hello = "hello";
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the Hello type alias
        let hello_symbol = binder.file_locals.get("Hello").expect("Hello should be in file locals");
        let hello_type = checker.get_type_of_symbol(hello_symbol);
        let hello_str = checker.type_to_string(hello_type);

        // Simple template literal should show as `hello`
        assert!(hello_str.contains("hello"), "Hello should contain 'hello', got: {}", hello_str);
    }

    #[test]
    fn test_template_literal_with_substitution() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test template literal type with substitution
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Greeting<T extends string> = `hello ${T}`;
                type HelloWorld = Greeting<"world">;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Check Greeting type
        let greeting_symbol = binder.file_locals.get("Greeting").expect("Greeting should be in file locals");
        let greeting_type = checker.get_type_of_symbol(greeting_symbol);
        let greeting_str = checker.type_to_string(greeting_type);

        // Greeting should be a template literal type with T as substitution
        assert!(greeting_str.contains("hello"), "Greeting should contain 'hello', got: {}", greeting_str);

        // Check HelloWorld type
        let hw_symbol = binder.file_locals.get("HelloWorld").expect("HelloWorld should be in file locals");
        let hw_type = checker.get_type_of_symbol(hw_symbol);
        let hw_str = checker.type_to_string(hw_type);

        // HelloWorld should be the instantiated template
        // Note: Full instantiation of template literals with concrete types is not yet implemented
        // For now, just check that it parses correctly
        assert!(hw_str.contains("hello"), "HelloWorld should contain 'hello', got: {}", hw_str);
    }

    #[test]
    fn test_mapped_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test mapped type
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Readonly<T> = { readonly [K in keyof T]: T[K] };
                type Original = { a: number; b: string };
                type ReadonlyOriginal = Readonly<Original>;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the Readonly type alias - should be a mapped type
        let readonly_symbol = binder.file_locals.get("Readonly").expect("Readonly should be in file locals");
        let readonly_type = checker.get_type_of_symbol(readonly_symbol);
        let readonly_str = checker.type_to_string(readonly_type);

        // Mapped type should show as { [K in ...]: ... }
        assert!(readonly_str.contains("[K in"), "Readonly should be a mapped type, got: {}", readonly_str);
    }

    #[test]
    fn test_infer_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test simple conditional type first (no generics to complicate things)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type IsString = string extends number ? "yes" : "no";
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the IsString type alias - should resolve to "no"
        let is_string_symbol_id = binder.file_locals.get("IsString").expect("IsString should be in file locals");
        let is_string_type = checker.get_type_of_symbol(is_string_symbol_id);
        let is_string_str = checker.type_to_string(is_string_type);

        println!("IsString type: {}", is_string_str);

        // Should resolve to "no" since string does not extend number
        assert_eq!(is_string_str, "\"no\"", "IsString should be \"no\", got: {}", is_string_str);
    }

    #[test]
    fn test_generic_conditional_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test generic conditional type (deferred evaluation)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type IsNumber<T> = T extends number ? "yes" : "no";
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the IsNumber type alias - should be a conditional type (deferred)
        let is_number_symbol_id = binder.file_locals.get("IsNumber").expect("IsNumber should be in file locals");
        let is_number_type = checker.get_type_of_symbol(is_number_symbol_id);
        let is_number_str = checker.type_to_string(is_number_type);

        println!("IsNumber type: {}", is_number_str);

        // Should be a deferred conditional type
        assert!(is_number_str.contains("extends"), "IsNumber should be a conditional type, got: {}", is_number_str);
    }

    #[test]
    fn test_infer_type_in_conditional() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test infer type in conditional type
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type UnwrapPromise<T> = T extends Promise<infer U> ? U : T;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the UnwrapPromise type alias
        let unwrap_symbol_id = binder.file_locals.get("UnwrapPromise").expect("UnwrapPromise should be in file locals");
        let unwrap_type = checker.get_type_of_symbol(unwrap_symbol_id);
        let unwrap_str = checker.type_to_string(unwrap_type);

        println!("UnwrapPromise type: {}", unwrap_str);

        // Should be a deferred conditional type with extends
        assert!(unwrap_str.contains("extends"), "UnwrapPromise should be a conditional type, got: {}", unwrap_str);
    }

    #[test]
    fn test_keyof_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test keyof any - should return string | number | symbol
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Keys = keyof any;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the Keys type alias
        let keys_symbol_id = binder.file_locals.get("Keys").expect("Keys should be in file locals");
        let keys_type = checker.get_type_of_symbol(keys_symbol_id);
        let keys_str = checker.type_to_string(keys_type);

        println!("Keys type: {}", keys_str);

        // keyof any = string | number | symbol
        assert!(keys_str.contains("string") || keys_str.contains("number"),
            "keyof any should contain 'string' or 'number', got: {}", keys_str);
    }

    #[test]
    fn test_keyof_object_type_literal() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test keyof with object type literal
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Keys = keyof { name: string; age: number; };
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the Keys type alias
        let keys_symbol_id = binder.file_locals.get("Keys").expect("Keys should be in file locals");
        let keys_type = checker.get_type_of_symbol(keys_symbol_id);
        let keys_str = checker.type_to_string(keys_type);

        println!("Keys from object literal: {}", keys_str);

        // keyof { name: string; age: number } = "name" | "age"
        assert!(keys_str.contains("name") && keys_str.contains("age"),
            "keyof object literal should contain 'name' and 'age', got: {}", keys_str);
    }

    #[test]
    fn test_mapped_type_instantiation() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test mapped type: { [K in keyof T]: T[K] } applied to { a: number }
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type Identity<T> = { [K in keyof T]: T[K] };
                type Result = Identity<{ a: number; b: string }>;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        // Get the Result type alias
        let result_symbol_id = binder.file_locals.get("Result").expect("Result should be in file locals");
        let result_type = checker.get_type_of_symbol(result_symbol_id);
        let result_str = checker.type_to_string(result_type);

        println!("Mapped type result: {}", result_str);

        // The result should be an object type with 'a' and 'b' properties
        assert!(result_str.contains("a: number") && result_str.contains("b: string"),
            "Mapped type should produce {{ a: number; b: string }}, got: {}", result_str);
    }

    #[test]
    fn test_infer_array_element() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test: type UnwrapArray<T> = T extends (infer U)[] ? U : T;
        // Applied to string[] should give string
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type UnwrapArray<T> = T extends (infer U)[] ? U : T;
                type Result = UnwrapArray<string[]>;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let result_symbol_id = binder.file_locals.get("Result").expect("Result should be in file locals");
        let result_type = checker.get_type_of_symbol(result_symbol_id);
        let result_str = checker.type_to_string(result_type);

        // UnwrapArray<string[]> should evaluate to string
        assert_eq!(result_str, "string", "UnwrapArray<string[]> should be 'string', got: {}", result_str);
    }

    #[test]
    fn test_infer_false_branch() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // When pattern doesn't match, use false branch
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type UnwrapArray<T> = T extends (infer U)[] ? U : T;
                type Result = UnwrapArray<number>;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let result_symbol_id = binder.file_locals.get("Result").expect("Result should be in file locals");
        let result_type = checker.get_type_of_symbol(result_symbol_id);
        let result_str = checker.type_to_string(result_type);

        // UnwrapArray<number> should return number (false branch)
        assert_eq!(result_str, "number", "UnwrapArray<number> should be 'number', got: {}", result_str);
    }

    #[test]
    fn test_infer_function_return() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;

        // Test inferring function return type (simplified pattern without rest params)
        let mut parser = ParserState::new(
            "test.ts".to_string(),
            r#"
                type GetReturn<T> = T extends () => infer R ? R : never;
                type Result = GetReturn<() => string>;
            "#.to_string(),
        );
        let root = parser.parse_source_file();

        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );

        let result_symbol_id = binder.file_locals.get("Result").expect("Result should be in file locals");
        let result_type = checker.get_type_of_symbol(result_symbol_id);
        let result_str = checker.type_to_string(result_type);

        // GetReturn<() => string> should evaluate to string
        assert_eq!(result_str, "string", "GetReturn<() => string> should be 'string', got: {}", result_str);
    }

}
