//! Type checker module for TypeScript AST.
//!
//! This module is organized into several submodules:
//! - `types` - Type definitions (Type enum, flags, diagnostics)
//! - `arena` - TypeArena for type allocation
//! - `context` - CheckerContext for shared state
//! - `expr` - Expression type checking
//! - `statements` - Statement type checking
//! - `declarations` - Declaration type checking
//!
//! Note: CheckerState has been replaced by ThinCheckerState in thin_checker.rs
//! The types module is still used by both ThinChecker and Solver.

pub mod types;
pub mod arena;
pub mod context;
pub mod expr;
pub mod statements;
pub mod declarations;
pub mod control_flow;

#[cfg(test)]
mod control_flow_tests;

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
pub use context::{CheckerContext, EnclosingClassInfo, TypeCache};
pub use expr::ExpressionChecker;
pub use statements::StatementChecker;
pub use declarations::DeclarationChecker;
pub use control_flow::FlowAnalyzer;
