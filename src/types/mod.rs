//! Type System Module
//!
//! Core type representations and operations for TypeScript types.

pub mod intersection;

pub use intersection::{
    Type, ObjectType, Property, FunctionType, TypeParameter, Parameter,
    create_intersection, flatten_intersection, merge_object_types,
    distribute_intersection_over_union, get_intersection_properties,
};
