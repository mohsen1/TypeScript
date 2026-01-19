//! TypeScript Type Solver Library
//!
//! This library provides type resolution and inference for TypeScript:
//!
//! - **This Type Resolution**: Resolve 'this' in various contexts
//! - **Fluent API Support**: Handle method chaining with polymorphic this
//! - **This Parameter Checking**: Validate this parameter compatibility
//! - **Scope Tracking**: Track this binding across scopes
//!
//! # Quick Start
//!
//! ```rust
//! use ts_solver::{ThisTypeSolver, ClassInfo, FunctionKind};
//!
//! let mut solver = ThisTypeSolver::new();
//!
//! // Register a class
//! solver.register_class(ClassInfo::new(1, "MyClass"));
//!
//! // Enter class scope
//! solver.enter_class(1);
//! solver.enter_instance_member(1);
//!
//! // Resolve 'this' - gets ClassThis(1)
//! let this_type = solver.resolve_this(None);
//!
//! // Enter arrow function - captures parent's this
//! solver.enter_function(FunctionKind::Arrow, None);
//! let arrow_this = solver.current_this(); // Still ClassThis(1)
//! ```

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
