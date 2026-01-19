//! TypeScript Type System Library
//!
//! This library provides type support for TypeScript including:
//!
//! - **Polymorphic This**: The 'this' type in class/interface method return types
//! - **ThisType<T>**: Utility type for object literals with method this binding
//! - **This Parameter**: Explicit this parameter in function signatures
//! - **This Context**: Tracking this binding across different scopes
//! - **JSDoc Types**: Type representations for JSDoc annotations
//!
//! # Quick Start
//!
//! ```rust
//! use ts_types::{ThisType, ThisContext, FluentMethod, JsDocType, PrimitiveType};
//!
//! // Create a this type for a class
//! let this_type = ThisType::new(1);
//!
//! // Create a context for class instance methods
//! let ctx = ThisContext::class_instance(1);
//!
//! // Create a fluent method that returns 'this'
//! let method = FluentMethod::new("setName").returning_this();
//!
//! // Create JSDoc types
//! let string_type = JsDocType::primitive(PrimitiveType::String);
//! ```

pub mod this;
pub mod jsdoc;

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

pub use jsdoc::{
    JsDocType,
    JsDocComment,
    JsDocParameter,
    JsDocTypedef,
    JsDocCallback,
    JsDocTypeParameter,
    JsDocModifier,
    JsDocReturns,
    JsDocFunctionType,
    JsDocPropertySignature,
    PrimitiveType,
    LiteralType,
    HeritageClauseKind,
};
