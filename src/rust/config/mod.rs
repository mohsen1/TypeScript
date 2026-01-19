//! TSConfig parser and compiler options module
//!
//! This module provides:
//! - CompilerOptions struct matching TypeScript's compiler options
//! - TSConfig JSON parser with inheritance support
//! - Include/exclude glob pattern matching
//! - Project references support
//! - Strict mode flag combinations

pub mod compiler_options;
pub mod tsconfig;
pub mod parser;

pub use compiler_options::*;
pub use tsconfig::*;
pub use parser::*;
