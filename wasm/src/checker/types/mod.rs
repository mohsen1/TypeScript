//! Type definitions module.
//!
//! This module contains all type-related definitions for the type checker.

pub mod flags;
pub mod type_def;
pub mod diagnostics;

// Re-export commonly used items
pub use flags::{type_flags, object_flags, signature_flags};
pub use type_def::*;
pub use diagnostics::diagnostic_codes;
