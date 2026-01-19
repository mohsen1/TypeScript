//! TypeScript Binder Implementation
//!
//! This module implements the binder for TypeScript, which creates symbol tables
//! and manages scope chains. The binder is responsible for:
//! - Creating symbols for declarations
//! - Building symbol tables (locals and exports)
//! - Managing scope chains
//! - Handling TypeScript-specific binding (modules, namespaces, classes)

pub mod symbols;
pub mod scope;
pub mod binder;
pub mod arena;
pub mod syntax_kind;

pub use symbols::{Symbol, SymbolFlags, SymbolId, SymbolTable};
pub use scope::{Scope, ScopeChain, ScopeKind};
pub use binder::Binder;
pub use arena::Arena;
