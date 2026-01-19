//! Binder module
//!
//! This module handles the binding phase of compilation, which creates
//! the symbol table and establishes relationships between declarations.

pub mod symbol_declarations;

pub use symbol_declarations::{
    SymbolBindingContext,
    BoundSymbol,
    SymbolScope,
    SymbolDeclarationKind,
    SymbolPropertyDeclaration,
    SymbolIndexSignatureDeclaration,
    ComputedPropertyAnalysis,
};
