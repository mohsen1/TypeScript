//! Type signatures for functions, methods, and call expressions
//!
//! This module defines the signature representation used throughout the type checker
//! for function declarations, call expressions, and overload resolution.

use crate::ast::{NodeId, StringId, Span};

/// Unique identifier for a type in the type arena
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct TypeId(pub u32);

impl TypeId {
    pub const NONE: TypeId = TypeId(u32::MAX);

    // Built-in type IDs
    pub const ANY: TypeId = TypeId(0);
    pub const UNKNOWN: TypeId = TypeId(1);
    pub const STRING: TypeId = TypeId(2);
    pub const NUMBER: TypeId = TypeId(3);
    pub const BOOLEAN: TypeId = TypeId(4);
    pub const VOID: TypeId = TypeId(5);
    pub const UNDEFINED: TypeId = TypeId(6);
    pub const NULL: TypeId = TypeId(7);
    pub const NEVER: TypeId = TypeId(8);
    pub const OBJECT: TypeId = TypeId(9);
    pub const SYMBOL: TypeId = TypeId(10);
    pub const BIGINT: TypeId = TypeId(11);

    /// First user-defined type ID
    pub const FIRST_USER_TYPE: u32 = 100;

    #[inline]
    pub const fn new(index: u32) -> Self {
        TypeId(index)
    }

    #[inline]
    pub const fn is_none(self) -> bool {
        self.0 == u32::MAX
    }

    #[inline]
    pub const fn is_some(self) -> bool {
        self.0 != u32::MAX
    }

    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub const fn is_primitive(self) -> bool {
        self.0 <= TypeId::BIGINT.0
    }
}

/// Flags for type properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct TypeFlags(pub u32);

impl TypeFlags {
    pub const NONE: TypeFlags = TypeFlags(0);
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
    pub const ES_SYMBOL: TypeFlags = TypeFlags(1 << 12);
    pub const UNIQUE_ES_SYMBOL: TypeFlags = TypeFlags(1 << 13);
    pub const VOID: TypeFlags = TypeFlags(1 << 14);
    pub const UNDEFINED: TypeFlags = TypeFlags(1 << 15);
    pub const NULL: TypeFlags = TypeFlags(1 << 16);
    pub const NEVER: TypeFlags = TypeFlags(1 << 17);
    pub const TYPE_PARAMETER: TypeFlags = TypeFlags(1 << 18);
    pub const OBJECT: TypeFlags = TypeFlags(1 << 19);
    pub const UNION: TypeFlags = TypeFlags(1 << 20);
    pub const INTERSECTION: TypeFlags = TypeFlags(1 << 21);
    pub const INDEX: TypeFlags = TypeFlags(1 << 22);
    pub const INDEXED_ACCESS: TypeFlags = TypeFlags(1 << 23);
    pub const CONDITIONAL: TypeFlags = TypeFlags(1 << 24);
    pub const SUBSTITUTION: TypeFlags = TypeFlags(1 << 25);
    pub const NON_PRIMITIVE: TypeFlags = TypeFlags(1 << 26);
    pub const TEMPLATE_LITERAL: TypeFlags = TypeFlags(1 << 27);
    pub const STRING_MAPPING: TypeFlags = TypeFlags(1 << 28);

    // Composite flags
    pub const LITERAL: TypeFlags = TypeFlags(
        Self::STRING_LITERAL.0 | Self::NUMBER_LITERAL.0 | Self::BOOLEAN_LITERAL.0 |
        Self::ENUM_LITERAL.0 | Self::BIGINT_LITERAL.0
    );

    pub const UNIT: TypeFlags = TypeFlags(
        Self::LITERAL.0 | Self::UNIQUE_ES_SYMBOL.0 | Self::NULL.0 | Self::UNDEFINED.0
    );

    pub const STRING_LIKE: TypeFlags = TypeFlags(
        Self::STRING.0 | Self::STRING_LITERAL.0 | Self::TEMPLATE_LITERAL.0 | Self::STRING_MAPPING.0
    );

    pub const NUMBER_LIKE: TypeFlags = TypeFlags(Self::NUMBER.0 | Self::NUMBER_LITERAL.0 | Self::ENUM.0);

    pub const BOOLEAN_LIKE: TypeFlags = TypeFlags(Self::BOOLEAN.0 | Self::BOOLEAN_LITERAL.0);

    pub const BIGINT_LIKE: TypeFlags = TypeFlags(Self::BIGINT.0 | Self::BIGINT_LITERAL.0);

    pub const ES_SYMBOL_LIKE: TypeFlags = TypeFlags(Self::ES_SYMBOL.0 | Self::UNIQUE_ES_SYMBOL.0);

    pub const VOID_LIKE: TypeFlags = TypeFlags(Self::VOID.0 | Self::UNDEFINED.0);

    pub const PRIMITIVE: TypeFlags = TypeFlags(
        Self::STRING.0 | Self::NUMBER.0 | Self::BIGINT.0 | Self::BOOLEAN.0 |
        Self::ENUM.0 | Self::ENUM_LITERAL.0 | Self::ES_SYMBOL.0 |
        Self::VOID.0 | Self::UNDEFINED.0 | Self::NULL.0 | Self::LITERAL.0 | Self::UNIQUE_ES_SYMBOL.0
    );

    pub const UNION_OR_INTERSECTION: TypeFlags = TypeFlags(Self::UNION.0 | Self::INTERSECTION.0);

    pub const STRUCTURED_TYPE: TypeFlags = TypeFlags(
        Self::OBJECT.0 | Self::UNION.0 | Self::INTERSECTION.0
    );

    pub const INSTANTIABLE_NON_PRIMITIVE: TypeFlags = TypeFlags(
        Self::TYPE_PARAMETER.0 | Self::CONDITIONAL.0 | Self::SUBSTITUTION.0
    );

    pub const INSTANTIABLE_PRIMITIVE: TypeFlags = TypeFlags(
        Self::INDEX.0 | Self::TEMPLATE_LITERAL.0 | Self::STRING_MAPPING.0
    );

    pub const INSTANTIABLE: TypeFlags = TypeFlags(
        Self::INSTANTIABLE_NON_PRIMITIVE.0 | Self::INSTANTIABLE_PRIMITIVE.0
    );

    pub const STRUCTURED_OR_INSTANTIABLE: TypeFlags = TypeFlags(
        Self::STRUCTURED_TYPE.0 | Self::INSTANTIABLE.0
    );

    pub const NARROWABLE: TypeFlags = TypeFlags(
        Self::ANY.0 | Self::UNKNOWN.0 | Self::STRUCTURED_OR_INSTANTIABLE.0 |
        Self::STRING_LIKE.0 | Self::NUMBER_LIKE.0 | Self::BIGINT_LIKE.0 |
        Self::BOOLEAN_LIKE.0 | Self::ES_SYMBOL_LIKE.0 | Self::NON_PRIMITIVE.0 | Self::UNIQUE_ES_SYMBOL.0
    );

    #[inline]
    pub const fn contains(self, other: TypeFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn intersects(self, other: TypeFlags) -> bool {
        (self.0 & other.0) != 0
    }

    #[inline]
    pub const fn union(self, other: TypeFlags) -> TypeFlags {
        TypeFlags(self.0 | other.0)
    }
}

impl std::ops::BitOr for TypeFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self { self.union(rhs) }
}

impl std::ops::BitOrAssign for TypeFlags {
    fn bitor_assign(&mut self, rhs: Self) { self.0 |= rhs.0; }
}

impl std::ops::BitAnd for TypeFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self { TypeFlags(self.0 & rhs.0) }
}

/// Signature flags for function-like signatures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct SignatureFlags(pub u16);

impl SignatureFlags {
    pub const NONE: SignatureFlags = SignatureFlags(0);
    pub const YIELD: SignatureFlags = SignatureFlags(1 << 0);
    pub const AWAIT: SignatureFlags = SignatureFlags(1 << 1);
    pub const VARIADIC_TUPLE: SignatureFlags = SignatureFlags(1 << 2);
    pub const HAS_REST_PARAMETER: SignatureFlags = SignatureFlags(1 << 3);
    pub const HAS_LITERAL_TYPES: SignatureFlags = SignatureFlags(1 << 4);
    pub const ABSTRACT: SignatureFlags = SignatureFlags(1 << 5);
    pub const IS_SPREAD: SignatureFlags = SignatureFlags(1 << 6);
    pub const PROPAGATING_FLAGS: SignatureFlags = SignatureFlags(
        Self::YIELD.0 | Self::AWAIT.0 | Self::VARIADIC_TUPLE.0
    );
    pub const CALL_SIGNATURE: SignatureFlags = SignatureFlags(1 << 7);
    pub const CONSTRUCT_SIGNATURE: SignatureFlags = SignatureFlags(1 << 8);
    pub const CALL_CHAIN: SignatureFlags = SignatureFlags(1 << 9);
    pub const NON_NULLABLE: SignatureFlags = SignatureFlags(1 << 10);

    #[inline]
    pub const fn contains(self, other: SignatureFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn union(self, other: SignatureFlags) -> SignatureFlags {
        SignatureFlags(self.0 | other.0)
    }
}

impl std::ops::BitOr for SignatureFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self { self.union(rhs) }
}

/// Parameter flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct ParameterFlags(pub u8);

impl ParameterFlags {
    pub const NONE: ParameterFlags = ParameterFlags(0);
    pub const OPTIONAL: ParameterFlags = ParameterFlags(1 << 0);
    pub const REST: ParameterFlags = ParameterFlags(1 << 1);
    pub const HAS_DEFAULT: ParameterFlags = ParameterFlags(1 << 2);
    pub const READONLY: ParameterFlags = ParameterFlags(1 << 3);
    pub const PUBLIC: ParameterFlags = ParameterFlags(1 << 4);
    pub const PRIVATE: ParameterFlags = ParameterFlags(1 << 5);
    pub const PROTECTED: ParameterFlags = ParameterFlags(1 << 6);

    pub const PARAMETER_PROPERTY: ParameterFlags = ParameterFlags(
        Self::PUBLIC.0 | Self::PRIVATE.0 | Self::PROTECTED.0 | Self::READONLY.0
    );

    #[inline]
    pub const fn contains(self, other: ParameterFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn is_optional(self) -> bool {
        self.contains(Self::OPTIONAL) || self.contains(Self::HAS_DEFAULT)
    }

    #[inline]
    pub const fn is_rest(self) -> bool {
        self.contains(Self::REST)
    }
}

/// A function parameter in a signature
#[derive(Debug, Clone, Copy)]
pub struct Parameter {
    /// Name of the parameter (interned string)
    pub name: StringId,
    /// Type of the parameter
    pub type_id: TypeId,
    /// Parameter flags (optional, rest, etc.)
    pub flags: ParameterFlags,
    /// Source location of the parameter
    pub span: Span,
    /// AST node for the parameter declaration
    pub declaration: NodeId,
}

impl Parameter {
    pub fn new(name: StringId, type_id: TypeId) -> Self {
        Parameter {
            name,
            type_id,
            flags: ParameterFlags::NONE,
            span: Span::new(0, 0),
            declaration: NodeId::NONE,
        }
    }

    pub fn optional(mut self) -> Self {
        self.flags = ParameterFlags(self.flags.0 | ParameterFlags::OPTIONAL.0);
        self
    }

    pub fn rest(mut self) -> Self {
        self.flags = ParameterFlags(self.flags.0 | ParameterFlags::REST.0);
        self
    }

    pub fn with_default(mut self) -> Self {
        self.flags = ParameterFlags(self.flags.0 | ParameterFlags::HAS_DEFAULT.0);
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    pub fn with_declaration(mut self, node: NodeId) -> Self {
        self.declaration = node;
        self
    }

    #[inline]
    pub fn is_optional(&self) -> bool {
        self.flags.is_optional()
    }

    #[inline]
    pub fn is_rest(&self) -> bool {
        self.flags.is_rest()
    }
}

impl Default for Parameter {
    fn default() -> Self {
        Parameter::new(StringId::EMPTY, TypeId::ANY)
    }
}

/// A type parameter for generic functions/classes
#[derive(Debug, Clone, Copy)]
pub struct TypeParameter {
    /// Name of the type parameter (e.g., "T")
    pub name: StringId,
    /// Type ID representing this type parameter
    pub type_id: TypeId,
    /// Constraint type (extends clause)
    pub constraint: TypeId,
    /// Default type
    pub default: TypeId,
    /// Source location
    pub span: Span,
    /// AST node declaration
    pub declaration: NodeId,
}

impl TypeParameter {
    pub fn new(name: StringId, type_id: TypeId) -> Self {
        TypeParameter {
            name,
            type_id,
            constraint: TypeId::NONE,
            default: TypeId::NONE,
            span: Span::new(0, 0),
            declaration: NodeId::NONE,
        }
    }

    pub fn with_constraint(mut self, constraint: TypeId) -> Self {
        self.constraint = constraint;
        self
    }

    pub fn with_default(mut self, default: TypeId) -> Self {
        self.default = default;
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    #[inline]
    pub fn has_constraint(&self) -> bool {
        self.constraint.is_some()
    }

    #[inline]
    pub fn has_default(&self) -> bool {
        self.default.is_some()
    }
}

impl Default for TypeParameter {
    fn default() -> Self {
        TypeParameter::new(StringId::EMPTY, TypeId::NONE)
    }
}

/// Unique signature ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(transparent)]
pub struct SignatureId(pub u32);

impl SignatureId {
    pub const NONE: SignatureId = SignatureId(u32::MAX);

    #[inline]
    pub const fn new(index: u32) -> Self {
        SignatureId(index)
    }

    #[inline]
    pub const fn is_none(self) -> bool {
        self.0 == u32::MAX
    }

    #[inline]
    pub const fn is_some(self) -> bool {
        self.0 != u32::MAX
    }

    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A function signature representing callable/constructable types
#[derive(Debug, Clone)]
pub struct Signature {
    /// Unique identifier
    pub id: SignatureId,
    /// Signature flags
    pub flags: SignatureFlags,
    /// Type parameters (for generic functions)
    pub type_parameters: Vec<TypeParameter>,
    /// Required and optional parameters
    pub parameters: Vec<Parameter>,
    /// Minimum number of required arguments
    pub min_argument_count: u32,
    /// Return type
    pub return_type: TypeId,
    /// Resolved return type (after type inference)
    pub resolved_return_type: TypeId,
    /// The AST node that declares this signature
    pub declaration: NodeId,
    /// For overloaded functions, link to next overload
    pub next_overload: SignatureId,
    /// Target signature for call chains
    pub target: SignatureId,
}

impl Signature {
    pub fn new(id: SignatureId) -> Self {
        Signature {
            id,
            flags: SignatureFlags::NONE,
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            min_argument_count: 0,
            return_type: TypeId::VOID,
            resolved_return_type: TypeId::NONE,
            declaration: NodeId::NONE,
            next_overload: SignatureId::NONE,
            target: SignatureId::NONE,
        }
    }

    pub fn with_parameters(mut self, params: Vec<Parameter>) -> Self {
        self.min_argument_count = params.iter()
            .take_while(|p| !p.is_optional() && !p.is_rest())
            .count() as u32;
        self.parameters = params;
        self
    }

    pub fn with_type_parameters(mut self, type_params: Vec<TypeParameter>) -> Self {
        self.type_parameters = type_params;
        self
    }

    pub fn with_return_type(mut self, return_type: TypeId) -> Self {
        self.return_type = return_type;
        self
    }

    pub fn with_flags(mut self, flags: SignatureFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn with_declaration(mut self, decl: NodeId) -> Self {
        self.declaration = decl;
        self
    }

    /// Get the total number of parameters (excluding rest)
    pub fn parameter_count(&self) -> usize {
        self.parameters.iter().filter(|p| !p.is_rest()).count()
    }

    /// Get the maximum number of arguments this signature can accept
    pub fn max_argument_count(&self) -> Option<usize> {
        if self.has_rest_parameter() {
            None // Unlimited
        } else {
            Some(self.parameter_count())
        }
    }

    /// Check if this signature has a rest parameter
    pub fn has_rest_parameter(&self) -> bool {
        self.flags.contains(SignatureFlags::HAS_REST_PARAMETER) ||
            self.parameters.last().map_or(false, |p| p.is_rest())
    }

    /// Get the rest parameter if present
    pub fn rest_parameter(&self) -> Option<&Parameter> {
        self.parameters.last().filter(|p| p.is_rest())
    }

    /// Check if this is a generic signature
    pub fn is_generic(&self) -> bool {
        !self.type_parameters.is_empty()
    }

    /// Check if this is a call signature
    pub fn is_call_signature(&self) -> bool {
        self.flags.contains(SignatureFlags::CALL_SIGNATURE)
    }

    /// Check if this is a construct signature
    pub fn is_construct_signature(&self) -> bool {
        self.flags.contains(SignatureFlags::CONSTRUCT_SIGNATURE)
    }

    /// Get the effective return type (resolved if available, otherwise declared)
    pub fn get_return_type(&self) -> TypeId {
        if self.resolved_return_type.is_some() {
            self.resolved_return_type
        } else {
            self.return_type
        }
    }

    /// Check if the given argument count is valid for this signature
    pub fn accepts_argument_count(&self, arg_count: usize) -> bool {
        let min = self.min_argument_count as usize;
        match self.max_argument_count() {
            Some(max) => arg_count >= min && arg_count <= max,
            None => arg_count >= min,
        }
    }
}

impl Default for Signature {
    fn default() -> Self {
        Signature::new(SignatureId::NONE)
    }
}

/// Storage for signatures - uses SoA pattern for cache efficiency
pub struct SignatureStore {
    signatures: Vec<Signature>,
}

impl SignatureStore {
    pub fn new() -> Self {
        SignatureStore {
            signatures: Vec::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        SignatureStore {
            signatures: Vec::with_capacity(cap),
        }
    }

    /// Allocate a new signature
    pub fn alloc(&mut self) -> SignatureId {
        let id = SignatureId::new(self.signatures.len() as u32);
        self.signatures.push(Signature::new(id));
        id
    }

    /// Allocate a signature with the given configuration
    pub fn alloc_with(&mut self, sig: Signature) -> SignatureId {
        let id = SignatureId::new(self.signatures.len() as u32);
        let mut sig = sig;
        sig.id = id;
        self.signatures.push(sig);
        id
    }

    /// Get a signature by ID
    pub fn get(&self, id: SignatureId) -> Option<&Signature> {
        if id.is_none() {
            return None;
        }
        self.signatures.get(id.index())
    }

    /// Get a mutable signature by ID
    pub fn get_mut(&mut self, id: SignatureId) -> Option<&mut Signature> {
        if id.is_none() {
            return None;
        }
        self.signatures.get_mut(id.index())
    }

    /// Get the number of stored signatures
    pub fn len(&self) -> usize {
        self.signatures.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.signatures.is_empty()
    }

    /// Iterate over all signatures
    pub fn iter(&self) -> impl Iterator<Item = &Signature> {
        self.signatures.iter()
    }
}

impl Default for SignatureStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_id_primitives() {
        assert!(TypeId::ANY.is_primitive());
        assert!(TypeId::STRING.is_primitive());
        assert!(TypeId::NUMBER.is_primitive());
        assert!(TypeId::BOOLEAN.is_primitive());
        assert!(!TypeId::new(100).is_primitive());
    }

    #[test]
    fn test_type_flags_composition() {
        let flags = TypeFlags::STRING | TypeFlags::NUMBER;
        assert!(flags.intersects(TypeFlags::STRING));
        assert!(flags.intersects(TypeFlags::NUMBER));
        assert!(!flags.intersects(TypeFlags::BOOLEAN));
    }

    #[test]
    fn test_parameter_creation() {
        let param = Parameter::new(StringId::new(1), TypeId::STRING)
            .optional()
            .with_span(Span::new(0, 10));

        assert!(param.is_optional());
        assert!(!param.is_rest());
        assert_eq!(param.span.start, 0);
        assert_eq!(param.span.end, 10);
    }

    #[test]
    fn test_signature_argument_count() {
        let sig = Signature::new(SignatureId::new(0))
            .with_parameters(vec![
                Parameter::new(StringId::new(1), TypeId::STRING),
                Parameter::new(StringId::new(2), TypeId::NUMBER).optional(),
            ]);

        assert_eq!(sig.min_argument_count, 1);
        assert_eq!(sig.parameter_count(), 2);
        assert_eq!(sig.max_argument_count(), Some(2));

        assert!(sig.accepts_argument_count(1));
        assert!(sig.accepts_argument_count(2));
        assert!(!sig.accepts_argument_count(0));
        assert!(!sig.accepts_argument_count(3));
    }

    #[test]
    fn test_signature_with_rest() {
        let sig = Signature::new(SignatureId::new(0))
            .with_parameters(vec![
                Parameter::new(StringId::new(1), TypeId::STRING),
                Parameter::new(StringId::new(2), TypeId::NUMBER).rest(),
            ]);

        assert!(sig.has_rest_parameter());
        assert_eq!(sig.max_argument_count(), None);

        assert!(sig.accepts_argument_count(1));
        assert!(sig.accepts_argument_count(5));
        assert!(sig.accepts_argument_count(100));
    }

    #[test]
    fn test_signature_store() {
        let mut store = SignatureStore::new();

        let id1 = store.alloc();
        let id2 = store.alloc();

        assert_ne!(id1, id2);
        assert!(store.get(id1).is_some());
        assert!(store.get(id2).is_some());
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_type_parameter() {
        let tp = TypeParameter::new(StringId::new(1), TypeId::new(100))
            .with_constraint(TypeId::OBJECT)
            .with_default(TypeId::STRING);

        assert!(tp.has_constraint());
        assert!(tp.has_default());
        assert_eq!(tp.constraint, TypeId::OBJECT);
        assert_eq!(tp.default, TypeId::STRING);
    }
}
