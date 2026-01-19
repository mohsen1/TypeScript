//! Built-in TypeScript/JavaScript type declarations
//!
//! This module provides Rust representations of the standard library types
//! that TypeScript uses, including ES5, ES2015+, ES2020+, and DOM types.
//!
//! The types are loaded based on the compiler's target and lib options.

pub mod types;
pub mod es5;
pub mod es2015;
pub mod es2020;
pub mod dom;
pub mod loader;

pub use types::*;
pub use loader::{LibLoader, LibLoaderConfig, ScriptTarget, LibFile};

#[cfg(test)]
mod tests;
