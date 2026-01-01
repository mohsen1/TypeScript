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

    /// Create an interface type with properties and signatures.
    pub fn create_interface_type(
        &mut self,
        properties: Vec<SymbolId>,
        construct_signatures: Vec<Signature>,
        call_signatures: Vec<Signature>,
    ) -> TypeId {
        let mut obj = ObjectType::new(object_flags::INTERFACE, SymbolId::NONE);
        obj.properties = properties;
        obj.construct_signatures = construct_signatures;
        obj.call_signatures = call_signatures;
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

    /// Diagnostics produced during type checking.
    pub diagnostics: Vec<Diagnostic>,

    /// Current file name.
    pub file_name: String,
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

            // Array types
            Node::ArrayType(at) => {
                let element_type = self.get_type_of_node(at.element_type);
                // For now, return object type. Proper array handling needs Array<T> type.
                let _ = element_type;
                self.types.object_type
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
                            // For other type references, look up in symbol table
                            if let Some(symbol_id) = self.file_locals.get(&id.escaped_text) {
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
                self.get_type_of_node(ta.type_node)
            }

            // Type literals (e.g., { x: number, y: string })
            Node::TypeLiteral(tl) => {
                self.get_type_of_type_literal(&tl.members)
            }

            // Property access expressions (e.g., obj.prop)
            Node::PropertyAccessExpression(pa) => {
                self.get_type_of_property_access(pa.expression, pa.name)
            }

            // Object literals (e.g., { x: 1, y: "hello" })
            Node::ObjectLiteralExpression(ole) => {
                self.get_type_of_object_literal(&ole.properties)
            }

            // Call expressions (e.g., fn(arg1, arg2))
            Node::CallExpression(ce) => {
                self.get_type_of_call_expression(ce.expression, &ce.arguments)
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
    fn get_type_of_call_expression(&mut self, expression: NodeIndex, arguments: &crate::parser::NodeList) -> TypeId {
        // Get the type of the function being called
        let func_type = self.get_type_of_node(expression);

        // If it's a function type, return the return type
        if let Some(Type::Function(f)) = self.types.get(func_type) {
            // Check argument count
            let arg_count = arguments.nodes.len() as u32;
            if arg_count < f.min_argument_count && !f.has_rest_parameter {
                // TODO: Add diagnostic for too few arguments
            }
            if arg_count > f.parameter_types.len() as u32 && !f.has_rest_parameter {
                // TODO: Add diagnostic for too many arguments
            }

            // TODO: Check argument types match parameter types

            return f.return_type;
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
        // Collect element types
        let mut element_types = Vec::new();
        for &elem_idx in &elements.nodes {
            let elem_type = self.get_type_of_node(elem_idx);
            if !element_types.contains(&elem_type) {
                element_types.push(elem_type);
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

    /// Get the type of a class declaration.
    /// Creates an ObjectType with CLASS object flags, containing all class members.
    fn get_type_of_class_declaration(
        &mut self,
        _node: NodeIndex,
        class: &crate::parser::ClassDeclaration,
    ) -> TypeId {
        use crate::parser::Node;

        let mut properties = Vec::new();
        let mut call_signatures = Vec::new();
        let mut construct_signatures = Vec::new();

        // Process class members
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

                    // Constructor declaration
                    Node::ConstructorDeclaration(cd) => {
                        // Create a construct signature
                        let mut signature = Signature::new(member_idx);

                        // Add parameters
                        for &param_idx in &cd.parameters.nodes {
                            if let Some(Node::ParameterDeclaration(param)) = self.node_arena.get(param_idx) {
                                if let Some(Node::Identifier(id)) = self.node_arena.get(param.name) {
                                    let param_symbol = self.local_symbols_mut().alloc(
                                        symbol_flags::FUNCTION_SCOPED_VARIABLE,
                                        id.escaped_text.clone(),
                                    );
                                    signature.parameters.push(param_symbol);
                                }
                            }
                        }

                        signature.min_argument_count = signature.parameters.len() as u32;
                        construct_signatures.push(signature);
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

        // Create the class type as an ObjectType with CLASS object flags
        let class_type = self.types.create_class_type(properties, construct_signatures, call_signatures);
        class_type
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

                    // TODO: Add CallSignature and ConstructSignature when parser supports them
                    _ => {}
                }
            }
        }

        // Create the interface type as an ObjectType with INTERFACE object flags
        self.types.create_interface_type(properties, construct_signatures, call_signatures)
    }

    /// Get the type of a type literal ({ x: number, y: string }).
    fn get_type_of_type_literal(&mut self, members: &crate::parser::NodeList) -> TypeId {
        use crate::parser::Node;

        let mut properties = Vec::new();

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
                    }
                    _ => {}
                }
            }
        }

        // Create object type
        self.types.create_object_type(properties)
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
    fn get_property_type(&self, object_type: TypeId, prop_name: &str) -> TypeId {
        let Some(typ) = self.types.get(object_type) else {
            return self.types.any_type;
        };

        match typ {
            Type::Object(obj) => {
                // Look up property in object's properties
                for &prop_id in &obj.properties {
                    if let Some(sym) = self.get_symbol(prop_id) {
                        if sym.escaped_name == prop_name {
                            return self.symbol_types.get(&prop_id).copied().unwrap_or(self.types.any_type);
                        }
                    }
                }
                self.types.any_type
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
        self.symbol_arena.get(id).or_else(|| self.local_symbols.get(id))
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

        // Create type parameters
        let type_param_ids: Vec<TypeId> = if let Some(type_params) = type_parameters {
            type_params.nodes.iter()
                .filter_map(|&tp_idx| self.create_type_parameter(tp_idx))
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

            // Other types - return as-is for now
            TypeInfo::Other => type_id,
        }
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
            let base_type = self.get_type_of_symbol(symbol_id);

            // Check if it's a generic type that needs instantiation
            if let Some(Type::Function(f)) = self.types.get(base_type) {
                if !f.type_parameters.is_empty() {
                    return self.instantiate_type(base_type, &type_args, &f.type_parameters.clone());
                }
            }

            return base_type;
        }

        // Built-in generic types
        match name {
            "Array" | "ReadonlyArray" => {
                // For now, return object type. Full Array<T> support needs special handling.
                self.types.object_type
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
            Type::Object(_) => "object".to_string(),
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
            Type::Conditional(_) => "ConditionalType".to_string(),
            Type::Mapped(_) => "MappedType".to_string(),
            Type::IndexedAccess(_) => "IndexedAccessType".to_string(),
            Type::Index(_) => "IndexType".to_string(),
            Type::TemplateLiteral(_) => "TemplateLiteralType".to_string(),
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
}
