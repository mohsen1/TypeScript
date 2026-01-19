//! TypeScript Binder and Config Implementation
//!
//! This module implements core TypeScript compiler functionality in Rust:
//!
//! ## Binder
//! - Creating symbols for declarations
//! - Building symbol tables (locals and exports)
//! - Managing scope chains
//! - Handling TypeScript-specific binding (modules, namespaces, classes)
//!
//! ## Config
//! - Parsing tsconfig.json files
//! - Handling extends directive for config inheritance
//! - Include/exclude glob pattern matching
//! - Project references support

pub mod symbols;
pub mod scope;
pub mod binder;
pub mod arena;
pub mod syntax_kind;
pub mod config;

pub use symbols::{Symbol, SymbolFlags, SymbolId, SymbolTable};
pub use scope::{Scope, ScopeChain, ScopeKind};
pub use binder::Binder;
pub use arena::Arena;
pub use config::{CompilerOptions, TsConfig, TsConfigParser, parse_tsconfig, parse_tsconfig_string};
