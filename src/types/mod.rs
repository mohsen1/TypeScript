//! TypeScript Type System Implementation
//!
//! This module provides core type system components including:
//! - Polymorphic 'this' type for classes and interfaces
//! - ThisType<T> utility type
//! - this parameter handling
//! - JSDoc type representations

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
