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

    /// Create a union type.
    pub fn create_union(&mut self, types: Vec<TypeId>) -> TypeId {
        // If only one type, return it directly
        if types.len() == 1 {
            return types[0];
        }
        // If empty, return never
        if types.is_empty() {
            return self.never_type;
        }
        self.alloc(Type::Union(UnionType::new(types)))
    }

    /// Create an intersection type.
    pub fn create_intersection(&mut self, types: Vec<TypeId>) -> TypeId {
        // If only one type, return it directly
        if types.len() == 1 {
            return types[0];
        }
        // If empty, return unknown
        if types.is_empty() {
            return self.unknown_type;
        }
        self.alloc(Type::Intersection(IntersectionType::new(types)))
    }
}

impl Default for TypeArena {
    fn default() -> Self {
        Self::new()
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
    fn test_object_flags() {
        assert_eq!(object_flags::CLASS, 1);
        assert_eq!(object_flags::INTERFACE, 2);
        assert_eq!(object_flags::CLASS_OR_INTERFACE, object_flags::CLASS | object_flags::INTERFACE);
    }
}
