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
mod db;
mod types;
mod intern;
mod lower;
mod compat;
mod subtype;
mod infer;
mod instantiate;
mod evaluate;
mod contextual;
mod narrowing;
mod diagnostics;
mod operations;
mod apparent;
mod lawyer;

pub use db::*;
pub use types::*;
pub use intern::*;
pub use lower::*;
pub use compat::*;
pub use subtype::*;
pub use infer::*;
pub use instantiate::*;
pub use evaluate::*;
pub use contextual::*;
pub use narrowing::*;
pub use diagnostics::*;
pub use operations::*;
pub use lawyer::*;
pub(crate) use apparent::*;

#[cfg(test)]
mod db_tests;
#[cfg(test)]
mod intern_tests;
#[cfg(test)]
mod types_tests;
#[cfg(test)]
mod lower_tests;
#[cfg(test)]
mod compat_tests;
#[cfg(test)]
mod subtype_tests;
#[cfg(test)]
mod infer_tests;
#[cfg(test)]
mod instantiate_tests;
#[cfg(test)]
mod evaluate_tests;
#[cfg(test)]
mod contextual_tests;
#[cfg(test)]
mod narrowing_tests;
#[cfg(test)]
mod diagnostics_tests;
#[cfg(test)]
mod operations_tests;
#[cfg(test)]
mod callable_tests;
#[cfg(test)]
mod index_signature_tests;
#[cfg(test)]
mod lawyer_tests;
#[cfg(test)]
mod union_tests;
#[cfg(test)]
mod integration_tests;
