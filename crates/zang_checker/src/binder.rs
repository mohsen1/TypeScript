//! Symbol Binder
//!
//! Creates symbol tables by walking the AST and recording declarations.

use zang_core::{Symbol, StringInterner};
use zang_parser::SourceFile;

/// Binder for creating symbol tables from AST
pub struct Binder<'a> {
    /// String interner for symbol names
    interner: &'a StringInterner,
}

impl<'a> Binder<'a> {
    /// Creates a new binder
    pub fn new(interner: &'a StringInterner) -> Self {
        Self { interner }
    }

    /// Binds a source file and returns the symbol table
    pub fn bind(&mut self, _source_file: &SourceFile) -> BindResult {
        // TODO: Implement binding
        BindResult {
            symbols: Vec::new(),
            errors: Vec::new(),
        }
    }
}

/// Result of binding a source file
pub struct BindResult {
    /// Symbols found in the source file
    pub symbols: Vec<Symbol>,
    /// Binding errors
    pub errors: Vec<BindError>,
}

/// A binding error
#[derive(Debug, Clone)]
pub struct BindError {
    /// Error message
    pub message: String,
    /// Location in source
    pub start: u32,
    /// End location
    pub end: u32,
}
