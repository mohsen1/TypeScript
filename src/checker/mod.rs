//! Type checker module
//!
//! This module provides type checking functionality for TypeScript,
//! including symbol type analysis and assignability checking.

pub mod symbols;

pub use symbols::{
    SymbolType,
    UniqueSymbolId,
    WellKnownSymbol,
    SymbolDeclarationFlags,
    SymbolAssignability,
    SymbolIndexSignature,
    SymbolIndexValueType,
    SymbolProperty,
    SymbolNarrowingContext,
    SymbolTypeofResult,
    SymbolComputedProperty,
    SymbolInTemplateLiteral,
    SymbolRegistry,
};
