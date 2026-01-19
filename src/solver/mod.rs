//! Type Solver Module
//!
//! Type inference and computation algorithms.

pub mod intersection;
pub mod union;

pub use intersection::{
    IntersectionSolver, intersect_types, intersect_all, is_empty_intersection,
};
pub use union::{
    UnionSolver, UnionSolverOptions, union_types, union_all, normalize_union,
};
