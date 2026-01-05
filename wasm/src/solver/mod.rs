//! Phase 7.5: Query-Based Structural Solver
//!
//! This module implements a declarative, query-based type solver architecture
//! that replaces the legacy imperative checker. It uses:
//!
//! - **Ena**: For unification (Union-Find) in generic type inference
//! - **Custom TypeKey**: Structural type representation with interning
//! - **Cycle Detection**: Coinductive semantics for recursive types
//!
//! Key benefits:
//! - O(1) type equality via interning (TypeId comparison)
//! - Automatic cycle handling via coinductive semantics
//! - Lazy evaluation - only compute types that are queried
//!
//! Note: Salsa integration is planned but requires nightly Rust features.
//! For now, we use manual query caching.

mod types;
mod intern;
mod lower;
mod subtype;
mod infer;

pub use types::*;
pub use intern::*;
pub use lower::*;
pub use subtype::*;
pub use infer::*;
