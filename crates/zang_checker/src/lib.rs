//! Zang Type Checker
//!
//! Lock-free type checking for TypeScript programs.
//! Uses atomic operations and careful cache design to avoid deadlocks.
//!
//! # Modules
//!
//! - `checker`: Core type checker implementation
//! - `inference`: Type inference for expressions
//! - `assignability`: Type compatibility checking
//! - `narrowing`: Control flow type narrowing
//! - `binder`: Symbol table creation
//! - `context`: Check context with cycle detection
//! - `diagnostics`: Error and warning messages
//! - `types`: Type representations

pub mod assignability;
pub mod binder;
pub mod checker;
pub mod context;
pub mod diagnostics;
pub mod inference;
pub mod narrowing;
pub mod types;

pub use assignability::{AssignabilityChecker, AssignabilityResult, RelationshipCache};
pub use binder::Binder;
pub use checker::{TypeChecker, CheckResult};
pub use context::CheckContext;
pub use diagnostics::{Diagnostic, DiagnosticKind, DiagnosticSeverity};
pub use inference::{InferenceContext, TypeInferrer};
pub use narrowing::{NarrowingContext, TypeNarrower};
pub use types::ResolvedType;
