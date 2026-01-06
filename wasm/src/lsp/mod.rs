//! LSP (Language Server Protocol) support for the WASM TypeScript compiler.
//!
//! This module provides LSP features:
//! - Go to Definition
//! - Find References
//! - (Future: Completions, Hover, etc.)
//!
//! Architecture:
//! - Position utilities for line/column <-> offset conversion
//! - AST node lookup by position
//! - Symbol-based navigation using binder data

pub mod position;
pub mod utils;
pub mod resolver;
pub mod definition;
pub mod references;

#[cfg(test)]
mod tests;

pub use definition::GoToDefinition;
pub use references::FindReferences;
pub use position::{Position, Location, SourceLocation, Range};
