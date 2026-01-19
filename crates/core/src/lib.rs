//! Core types and utilities for the TypeScript Rust port.
//!
//! This crate provides fundamental data structures and utilities
//! used throughout the TypeScript implementation.

pub mod checker;
pub mod config;
pub mod types;

pub use checker::*;
pub use config::*;
pub use types::*;
