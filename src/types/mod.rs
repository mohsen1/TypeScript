//! TypeScript Type System Implementation
//!
//! This module provides core type system components including:
//! - Polymorphic 'this' type for classes and interfaces
//! - ThisType<T> utility type
//! - this parameter handling

pub mod this;

pub use this::{
    ThisType,
    ThisTypeUtility,
    ThisParameter,
    ThisTypeValue,
    ThisContext,
    ThisContextKind,
    FluentMethod,
    ThisTypeRegistry,
    TypeId,
};
