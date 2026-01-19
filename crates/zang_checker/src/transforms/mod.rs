//! AST Transformations
//!
//! This module contains transformations that convert TypeScript AST nodes
//! to different output formats (CommonJS, AMD, UMD, etc.) and downlevel
//! async/await to ES5/ES2015.

pub mod async_generator;
pub mod modules;

pub use async_generator::{AsyncTransformer, AsyncTransformOptions, AsyncTransformTarget};
pub use modules::{
    ModuleTransformer, ModuleFormat, TransformResult, TransformOptions,
    CommonJsTransformer, AmdTransformer, UmdTransformer,
};
