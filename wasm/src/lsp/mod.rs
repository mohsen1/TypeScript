//! LSP (Language Server Protocol) support for the WASM TypeScript compiler.
//!
//! This module provides LSP features:
//! - Go to Definition
//! - Find References
//! - Completions
//! - Hover
//! - Signature Help
//! - Document Symbols
//! - Rename
//! - (Future: Semantic Tokens, Code Actions, etc.)
//!
//! Architecture:
//! - Position utilities for line/column <-> offset conversion
//! - AST node lookup by position
//! - Symbol-based navigation using binder data

pub mod position;
pub mod utils;
pub mod resolver;
pub mod definition;
pub mod references;
pub mod completions;
pub mod hover;
pub mod signature_help;
pub mod document_symbols;
pub mod rename;

#[cfg(test)]
mod tests;

pub use definition::GoToDefinition;
pub use references::FindReferences;
pub use completions::{Completions, CompletionItem, CompletionItemKind};
pub use hover::{HoverProvider, HoverInfo};
pub use signature_help::{SignatureHelpProvider, SignatureHelp, SignatureInformation, ParameterInformation};
pub use document_symbols::{DocumentSymbolProvider, DocumentSymbol, SymbolKind};
pub use rename::{RenameProvider, WorkspaceEdit, TextEdit};
pub use position::{Position, Location, SourceLocation, Range};
