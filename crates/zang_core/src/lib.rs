//! Zang Core - Core types and utilities for the Zang TypeScript compiler
//!
//! This crate provides foundational types and utilities used throughout the
//! Zang TypeScript compiler, including:
//!
//! - String interning for efficient symbol handling
//! - Arena allocation for AST nodes
//! - Type representations
//! - Common error types

#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

pub mod arena;
pub mod interner;
pub mod span;
pub mod symbol;
pub mod types;

pub use arena::Arena;
pub use interner::{InternedString, StringInterner};
pub use span::Span;
pub use symbol::Symbol;
