//! Zang Type Checker
//!
//! Lock-free type checking for TypeScript programs.
//! Uses atomic operations and careful cache design to avoid deadlocks.

pub mod binder;
pub mod checker;
pub mod context;
pub mod diagnostics;
pub mod types;

pub use binder::Binder;
pub use checker::TypeChecker;
pub use context::CheckContext;
pub use diagnostics::{Diagnostic, DiagnosticKind, DiagnosticSeverity};
