//! Type system module
//!
//! This module provides the core type system functionality for TypeScript,
//! including unique symbol types and other specialized type representations.

pub mod unique_symbol;

pub use unique_symbol::{
    UniqueSymbolType,
    UniqueSymbolFactory,
    UniqueSymbolPredicate,
    ConstSymbolDeclaration,
    TypeofUniqueSymbol,
    TypeofResultType,
};
