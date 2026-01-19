//! TypeScript Type Solver
//!
//! This module provides type resolution and inference including:
//! - 'this' type resolution and checking
//! - Fluent API / method chaining support

pub mod this;

pub use this::{
    ThisTypeSolver,
    ThisTypeDiagnostic,
    TextSpan,
    FunctionKind,
    ClassInfo,
    InterfaceInfo,
    ResolvedThis,
    ThisScope,
    ThisScopeKind,
    FluentApiBuilder,
    FluentMethodInfo,
    error_codes,
    TypeId,
};
