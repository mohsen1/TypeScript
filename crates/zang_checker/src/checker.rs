//! Type Checker
//!
//! Core type checking logic with lock-free caching.

use zang_core::{Arena, StringInterner};
use zang_parser::SourceFile;
use crate::context::CheckContext;
use crate::diagnostics::Diagnostic;

/// Type checker for TypeScript programs
pub struct TypeChecker<'a> {
    /// Arena for type allocation
    arena: &'a Arena,
    /// String interner
    interner: &'a StringInterner,
    /// Check context
    context: CheckContext,
}

impl<'a> TypeChecker<'a> {
    /// Creates a new type checker
    pub fn new(arena: &'a Arena, interner: &'a StringInterner) -> Self {
        Self {
            arena,
            interner,
            context: CheckContext::new(),
        }
    }

    /// Type checks a source file
    pub fn check(&mut self, _source_file: &SourceFile) -> CheckResult {
        // TODO: Implement type checking
        CheckResult {
            diagnostics: Vec::new(),
        }
    }

    /// Gets the arena
    pub fn arena(&self) -> &'a Arena {
        self.arena
    }

    /// Gets the string interner
    pub fn interner(&self) -> &'a StringInterner {
        self.interner
    }
}

/// Result of type checking
pub struct CheckResult {
    /// Diagnostics produced during type checking
    pub diagnostics: Vec<Diagnostic>,
}
