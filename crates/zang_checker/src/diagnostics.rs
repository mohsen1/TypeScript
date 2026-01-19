//! Diagnostics
//!
//! Error and warning messages from type checking.

use zang_core::Span;

/// A diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The kind of diagnostic
    pub kind: DiagnosticKind,
    /// The severity
    pub severity: DiagnosticSeverity,
    /// The diagnostic code
    pub code: u32,
    /// The message
    pub message: String,
    /// The source location
    pub span: Span,
    /// Related information
    pub related: Vec<RelatedInformation>,
}

/// Diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Error - compilation will fail
    Error,
    /// Warning - compilation will succeed but there may be issues
    Warning,
    /// Suggestion - style or best practice recommendation
    Suggestion,
    /// Message - informational
    Message,
}

/// Diagnostic kind for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// Syntax error
    Syntax,
    /// Semantic error
    Semantic,
    /// Type error
    Type,
    /// Declaration error
    Declaration,
    /// Module resolution error
    Module,
    // Import/Export specific errors
    /// Module not found
    ModuleNotFound,
    /// No default export in module
    NoDefaultExport,
    /// Export not found in module
    ExportNotFound,
    /// Type-only import used as value
    TypeOnlyUsedAsValue,
    /// Duplicate identifier
    DuplicateIdentifier,
    /// Undeclared identifier
    UndeclaredIdentifier,
    /// Duplicate export
    DuplicateExport,
    /// Invalid export
    InvalidExport,
}

/// Related diagnostic information
#[derive(Debug, Clone)]
pub struct RelatedInformation {
    /// The message
    pub message: String,
    /// The source location
    pub span: Span,
}

impl Diagnostic {
    /// Creates a new error diagnostic
    pub fn error(code: u32, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind: DiagnosticKind::Semantic,
            severity: DiagnosticSeverity::Error,
            code,
            message: message.into(),
            span,
            related: Vec::new(),
        }
    }

    /// Creates a new warning diagnostic
    pub fn warning(code: u32, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind: DiagnosticKind::Semantic,
            severity: DiagnosticSeverity::Warning,
            code,
            message: message.into(),
            span,
            related: Vec::new(),
        }
    }

    /// Creates a type error diagnostic
    pub fn type_error(code: u32, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind: DiagnosticKind::Type,
            severity: DiagnosticSeverity::Error,
            code,
            message: message.into(),
            span,
            related: Vec::new(),
        }
    }

    /// Adds related information
    pub fn with_related(mut self, message: impl Into<String>, span: Span) -> Self {
        self.related.push(RelatedInformation {
            message: message.into(),
            span,
        });
        self
    }
}
