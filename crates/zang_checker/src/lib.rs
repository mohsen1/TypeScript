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
pub mod async_checker;
pub mod binder;
pub mod checker;
pub mod context;
pub mod diagnostics;
pub mod exports;
pub mod imports;
pub mod inference;
pub mod modules;
pub mod narrowing;
pub mod promise;
pub mod transforms;
pub mod types;

pub use assignability::{AssignabilityChecker, AssignabilityResult, RelationshipCache};
pub use async_checker::AsyncChecker;
pub use binder::Binder;
pub use checker::{TypeChecker, CheckResult};
pub use context::CheckContext;
pub use diagnostics::{Diagnostic, DiagnosticKind, DiagnosticSeverity};
pub use exports::{ExportChecker, ModuleExports, ReExport};
pub use imports::{ImportChecker, ImportBinding, ImportResolution, ModuleResolutionMode};
pub use inference::{InferenceContext, TypeInferrer};
pub use modules::{ModuleBinder, ModuleScope, ModuleSymbol};
pub use narrowing::{NarrowingContext, TypeNarrower};
pub use promise::PromiseChecker;
pub use transforms::{ModuleTransformer, ModuleFormat, TransformOptions, AsyncTransformer, AsyncTransformOptions};
pub use types::ResolvedType;
