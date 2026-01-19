//! Type Solver Module
//!
//! Type inference and computation algorithms.

pub mod intersection;

pub use intersection::{
    IntersectionSolver, intersect_types, intersect_all, is_empty_intersection,
};
