//! TypeScript Type System Library
//!
//! This library provides polymorphic 'this' type support for TypeScript:
//!
//! - **Polymorphic This**: The 'this' type in class/interface method return types
//! - **ThisType<T>**: Utility type for object literals with method this binding
//! - **This Parameter**: Explicit this parameter in function signatures
//! - **This Context**: Tracking this binding across different scopes
//!
//! # Quick Start
//!
//! ```rust
//! use ts_types::{ThisType, ThisContext, FluentMethod};
//!
//! // Create a this type for a class
//! let this_type = ThisType::new(1);
//!
//! // Create a context for class instance methods
//! let ctx = ThisContext::class_instance(1);
//!
//! // Create a fluent method that returns 'this'
//! let method = FluentMethod::new("setName").returning_this();
//! ```

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
