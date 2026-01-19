//! AST Transformations
//!
//! This module contains transformations that convert TypeScript AST nodes
//! to different output formats (CommonJS, AMD, UMD, etc.)

pub mod modules;

pub use modules::{
    ModuleTransformer, ModuleFormat, TransformResult, TransformOptions,
    CommonJsTransformer, AmdTransformer, UmdTransformer,
};
