//! Zang Parser - TypeScript parser
//!
//! This crate provides a fast, efficient TypeScript parser that produces
//! an AST compatible with the TypeScript compiler.

#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

pub mod ast;
pub mod lexer;
pub mod parser;

pub use ast::*;
pub use lexer::Lexer;
pub use parser::Parser;
