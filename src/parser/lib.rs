//! TypeScript Parser Library
//!
//! A high-performance TypeScript parser using cache-efficient ThinNode AST representation.
//!
//! # Features
//!
//! - **Cache-efficient AST**: Uses Structure of Arrays (SoA) pattern for optimal memory layout
//! - **ThinNodes**: 32-byte node structs instead of Box-based tree structures
//! - **Arena allocation**: All nodes stored in a single arena for fast allocation
//! - **String interning**: Deduplicated string storage for identifiers and literals
//! - **Full TypeScript support**: All TypeScript 4.x syntax including JSX
//!
//! # Example
//!
//! ```
//! use ts_parser::{NodeArena, NodeKind, AstView};
//!
//! // Create an arena and allocate nodes
//! let mut arena = NodeArena::new();
//! let root = arena.alloc_node(NodeKind::SourceFile, 0, 100);
//!
//! // Access nodes via AstView
//! let view = AstView::new(&arena, root);
//! let node = view.get(root).unwrap();
//! assert_eq!(node.kind, NodeKind::SourceFile);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod node_kind;
pub mod thin_node;
pub mod ast;
pub mod checker;

// Re-export commonly used types at crate root
pub use node_kind::NodeKind;
pub use thin_node::{
    AstView, ChildrenRef, ModifierFlags, NodeArena, NodeFlags, NodeId, StringId, StringInterner,
    TextSpan, ThinNode,
};

// Re-export AST builders
pub use ast::{
    AstArena, AstBuilder, StatementBuilder, ExpressionBuilder, TypeBuilder,
    Span,
};

// Re-export type checker components
pub use checker::{
    TypeChecker, CheckerOptions,
    TypeId, TypeFlags,
    SignatureId, Signature, SignatureFlags, SignatureStore,
    Parameter, ParameterFlags, TypeParameter,
    FunctionChecker, FunctionContext, FunctionCheckResult, FunctionError,
    CallChecker, CallContext, CallCheckResult, CallError, Argument,
    OverloadResolver, OverloadResolutionResult,
    // Satisfies operator
    SatisfiesChecker, SatisfiesContext, SatisfiesCheckResult, SatisfiesError,
    // Type assertions
    AssertionChecker, AssertionContext, AssertionCheckResult, AssertionError, AssertionKind,
};

/// Parser configuration options.
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Parse as a module (ESM) rather than a script.
    pub is_module: bool,
    /// Parse JSX syntax.
    pub jsx: bool,
    /// Parse TypeScript-specific syntax.
    pub typescript: bool,
    /// Target ECMAScript version for feature detection.
    pub target: ScriptTarget,
    /// Preserve JSDoc comments in the AST.
    pub preserve_jsdoc: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        ParserConfig {
            is_module: true,
            jsx: true,
            typescript: true,
            target: ScriptTarget::ESNext,
            preserve_jsdoc: true,
        }
    }
}

/// ECMAScript target version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ScriptTarget {
    /// ECMAScript 3
    ES3 = 0,
    /// ECMAScript 5
    ES5 = 1,
    /// ECMAScript 2015 (ES6)
    ES2015 = 2,
    /// ECMAScript 2016
    ES2016 = 3,
    /// ECMAScript 2017
    ES2017 = 4,
    /// ECMAScript 2018
    ES2018 = 5,
    /// ECMAScript 2019
    ES2019 = 6,
    /// ECMAScript 2020
    ES2020 = 7,
    /// ECMAScript 2021
    ES2021 = 8,
    /// ECMAScript 2022
    ES2022 = 9,
    /// Latest ECMAScript features
    #[default]
    ESNext = 99,
}

/// Result of parsing a source file.
#[derive(Debug)]
pub struct ParseResult {
    /// The node arena containing all AST nodes.
    pub arena: NodeArena,
    /// The root node ID (SourceFile).
    pub root: NodeId,
    /// Parse errors encountered.
    pub errors: Vec<ParseError>,
}

impl ParseResult {
    /// Creates an AstView for traversing the parsed AST.
    pub fn view(&self) -> AstView<'_> {
        AstView::new(&self.arena, self.root)
    }

    /// Returns true if parsing completed without errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// A parse error with location information.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Error message.
    pub message: String,
    /// Start position in source.
    pub start: u32,
    /// End position in source.
    pub end: u32,
    /// Error code.
    pub code: u32,
}

impl ParseError {
    /// Creates a new parse error.
    pub fn new(message: impl Into<String>, start: u32, end: u32, code: u32) -> Self {
        ParseError {
            message: message.into(),
            start,
            end,
            code,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error TS{}: {} at {}..{}",
            self.code, self.message, self.start, self.end
        )
    }
}

impl std::error::Error for ParseError {}

/// Diagnostic codes for common parse errors.
pub mod diagnostics {
    /// Unexpected token error code
    pub const UNEXPECTED_TOKEN: u32 = 1012;
    /// Expression expected error code
    pub const EXPRESSION_EXPECTED: u32 = 1109;
    /// Statement expected error code
    pub const STATEMENT_EXPECTED: u32 = 1128;
    /// Declaration expected error code
    pub const DECLARATION_EXPECTED: u32 = 1146;
    /// Identifier expected error code
    pub const IDENTIFIER_EXPECTED: u32 = 1003;
    /// Semicolon expected error code
    pub const SEMICOLON_EXPECTED: u32 = 1005;
    /// Colon expected error code
    pub const COLON_EXPECTED: u32 = 1004;
    /// Close brace expected error code
    pub const CLOSE_BRACE_EXPECTED: u32 = 1006;
    /// Close paren expected error code
    pub const CLOSE_PAREN_EXPECTED: u32 = 1007;
    /// Close bracket expected error code
    pub const CLOSE_BRACKET_EXPECTED: u32 = 1008;
}

/// Returns the version of the parser library.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_crate_level_exports() {
        // Verify types are accessible at crate level
        let _: NodeKind = NodeKind::SourceFile;
        let _: NodeId = NodeId::new(0);
        let _: NodeFlags = NodeFlags::NONE;
        let arena = NodeArena::new();
        assert!(arena.is_empty());
    }

    #[test]
    fn test_parser_config_default() {
        let config = ParserConfig::default();
        assert!(config.is_module);
        assert!(config.jsx);
        assert!(config.typescript);
        assert_eq!(config.target, ScriptTarget::ESNext);
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::new("Unexpected token", 10, 15, diagnostics::UNEXPECTED_TOKEN);
        let s = format!("{}", err);
        assert!(s.contains("1012"));
        assert!(s.contains("Unexpected token"));
    }

    #[test]
    fn test_arena_integration() {
        let mut arena = NodeArena::new();

        // Build a simple program: const x = 42;
        let source_file = arena.alloc_node(NodeKind::SourceFile, 0, 14);
        let var_stmt = arena.alloc_node(NodeKind::VariableStatement, 0, 14);
        let var_list = arena.alloc_node(NodeKind::VariableDeclarationList, 0, 13);
        let var_decl = arena.alloc_node(NodeKind::VariableDeclaration, 6, 13);
        let identifier = arena.alloc_node(NodeKind::Identifier, 6, 7);
        let literal = arena.alloc_node(NodeKind::NumericLiteral, 10, 12);

        // Set up identifier name
        let x_string = arena.intern_string("x");
        arena.set_string(identifier, x_string);

        // Set up numeric literal value
        let num_string = arena.intern_string("42");
        arena.set_string(literal, num_string);

        // Build tree structure
        arena.add_children(var_decl, &[identifier, literal]);
        arena.add_children(var_list, &[var_decl]);
        arena.add_flags(var_list, NodeFlags::CONST);
        arena.add_children(var_stmt, &[var_list]);
        arena.add_children(source_file, &[var_stmt]);

        // Verify structure
        let view = AstView::new(&arena, source_file);
        let root = view.get(source_file).unwrap();
        assert_eq!(root.kind, NodeKind::SourceFile);

        let children = view.children(source_file);
        assert_eq!(children.len(), 1);
        assert_eq!(view.kind(children[0]), Some(NodeKind::VariableStatement));

        // Verify string interning
        let id_node = view.get(identifier).unwrap();
        assert_eq!(arena.get_string(id_node.string_id), Some("x"));
    }
}
