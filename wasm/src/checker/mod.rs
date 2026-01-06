//! Type checker module for TypeScript AST.
//!
//! This module is organized into several submodules:
//! - `types` - Type definitions (Type enum, flags, diagnostics)
//! - `arena` - TypeArena for type allocation
//! - `context` - CheckerContext for shared state (Phase 8 refactor)
//!
//! Note: CheckerState has been replaced by ThinCheckerState in thin_checker.rs
//! The types module is still used by both ThinChecker and Solver.

pub mod types;
pub mod arena;
pub mod context;

// Re-export key types
pub use types::{
    type_flags, object_flags, signature_flags, diagnostic_codes,
    Type, TypeId, LiteralValue, LiteralType, IntrinsicType,
    ObjectType, UnionType, IntersectionType, TypeParameter,
    ConditionalType, MappedType, IndexType, IndexedAccessType,
    TemplateLiteralType, FunctionType, ArrayTypeInfo, TupleTypeInfo,
    EnumTypeInfo, TypeReference, Signature, IndexInfo,
};
pub use arena::TypeArena;
pub use context::{CheckerContext, EnclosingClassInfo};
