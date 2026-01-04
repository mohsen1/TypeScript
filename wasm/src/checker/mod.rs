//! Type checker module for TypeScript AST.
//!
//! This module is organized into several submodules:
//! - `types` - Type definitions (Type enum, flags, etc.)
//! - `arena` - TypeArena for type allocation
//! - `state` - CheckerState struct and diagnostics
//! - `type_retrieval` - Core type inference (get_type_of_node)
//! - `relations` - Type relationship checking
//! - `narrowing` - Type narrowing and guards

pub mod types;
pub mod arena;
pub mod state;
mod type_retrieval;
mod relations;
mod narrowing;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod baseline_tests;

// Re-export key types for backwards compatibility
pub use types::{
    type_flags, object_flags, signature_flags, diagnostic_codes,
    Type, TypeId, LiteralValue, LiteralType, IntrinsicType,
    ObjectType, UnionType, IntersectionType, TypeParameter,
    ConditionalType, MappedType, IndexType, IndexedAccessType,
    TemplateLiteralType, FunctionType, ArrayTypeInfo, TupleTypeInfo,
    EnumTypeInfo, TypeReference, Signature, IndexInfo,
};
pub use arena::TypeArena;
pub use state::{CheckerState, Diagnostic, DiagnosticCategory, TypeGuard, TypeRelation};
