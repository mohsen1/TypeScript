//! Language Service module for TypeScript IDE features.
//!
//! This module provides IDE-like functionality including:
//! - Code completions
//! - Go to definition
//! - Find all references
//! - Rename
//! - Signature help
//! - Quick info (hover)
//! - Code fixes and refactorings
//! - Formatting
//!
//! The language service depends on the type checker (Phase 5) and provides
//! the `LanguageService` trait that can be implemented for different hosts.

// Foundation modules
pub mod utilities;
pub mod text_span;
pub mod text_changes;
pub mod document_registry;
pub mod export_info_map;

// Symbol display
pub mod symbol_display;

// Navigation features
pub mod go_to_definition;
pub mod document_highlights;
pub mod navigation_bar;
pub mod find_all_references;
pub mod rename;

// Semantic features
pub mod signature_help;
pub mod quick_info;
pub mod completions;
pub mod string_completions;

// Modern features
pub mod inlay_hints;
pub mod call_hierarchy;

// Code actions
pub mod code_fix_provider;
pub mod codefixes;
pub mod refactor_provider;
pub mod refactors;

// Formatting
pub mod formatting;

// Additional services
pub mod breakpoints;
pub mod outlining;
pub mod organize_imports;

// Main service orchestration
mod language_service;

// Re-export key types
pub use language_service::{LanguageService, LanguageServiceHost, LanguageServiceMode};
pub use text_span::{TextSpan, TextRange, TextChange};
pub use utilities::*;

#[cfg(test)]
mod tests;
