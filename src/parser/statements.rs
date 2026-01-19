//! Statement parsing
//!
//! Handles parsing of all TypeScript statement types:
//! - Variable declarations (let, const, var)
//! - Function declarations
//! - Class declarations
//! - Interface declarations
//! - Type alias declarations
//! - Enum declarations
//! - Namespace/module declarations
//! - Import/export declarations
//! - Control flow statements (if, for, while, etc.)

use crate::thin_parser::TokenKind;

/// Statement kind for quick matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatementKind {
    // Declarations
    VariableDeclaration,
    FunctionDeclaration,
    ClassDeclaration,
    InterfaceDeclaration,
    TypeAliasDeclaration,
    EnumDeclaration,
    NamespaceDeclaration,
    ModuleDeclaration,
    ImportDeclaration,
    ExportDeclaration,
    // Control flow
    BlockStatement,
    IfStatement,
    ForStatement,
    ForInStatement,
    ForOfStatement,
    WhileStatement,
    DoWhileStatement,
    SwitchStatement,
    TryStatement,
    // Jump statements
    ReturnStatement,
    BreakStatement,
    ContinueStatement,
    ThrowStatement,
    // Other
    ExpressionStatement,
    EmptyStatement,
    LabeledStatement,
    WithStatement,
    DebuggerStatement,
}

/// Check if a token starts a statement
pub fn starts_statement(kind: TokenKind, text: &str) -> bool {
    matches!(
        kind,
        TokenKind::Let
            | TokenKind::Const
            | TokenKind::Var
            | TokenKind::Function
            | TokenKind::Class
            | TokenKind::Interface
            | TokenKind::Type
            | TokenKind::Enum
            | TokenKind::Import
            | TokenKind::Export
            | TokenKind::If
            | TokenKind::For
            | TokenKind::While
            | TokenKind::Do
            | TokenKind::Return
            | TokenKind::Break
            | TokenKind::Continue
            | TokenKind::Throw
            | TokenKind::Try
            | TokenKind::OpenBrace
            | TokenKind::Semicolon
    ) || text == "namespace"
        || text == "module"
        || text == "declare"
        || text == "abstract"
        || text == "async"
        || text == "switch"
        || text == "debugger"
        || text == "with"
}

/// Check if a token starts a declaration
pub fn starts_declaration(kind: TokenKind, text: &str) -> bool {
    matches!(
        kind,
        TokenKind::Let
            | TokenKind::Const
            | TokenKind::Var
            | TokenKind::Function
            | TokenKind::Class
            | TokenKind::Interface
            | TokenKind::Type
            | TokenKind::Enum
            | TokenKind::Import
            | TokenKind::Export
    ) || text == "namespace"
        || text == "module"
        || text == "declare"
        || text == "abstract"
        || text == "async"
}

/// Determine statement kind from initial token
pub fn determine_statement_kind(kind: TokenKind, text: &str) -> Option<StatementKind> {
    match kind {
        TokenKind::Let | TokenKind::Const | TokenKind::Var => Some(StatementKind::VariableDeclaration),
        TokenKind::Function => Some(StatementKind::FunctionDeclaration),
        TokenKind::Class => Some(StatementKind::ClassDeclaration),
        TokenKind::Interface => Some(StatementKind::InterfaceDeclaration),
        TokenKind::Type => Some(StatementKind::TypeAliasDeclaration),
        TokenKind::Enum => Some(StatementKind::EnumDeclaration),
        TokenKind::Import => Some(StatementKind::ImportDeclaration),
        TokenKind::Export => Some(StatementKind::ExportDeclaration),
        TokenKind::If => Some(StatementKind::IfStatement),
        TokenKind::For => Some(StatementKind::ForStatement),
        TokenKind::While => Some(StatementKind::WhileStatement),
        TokenKind::Do => Some(StatementKind::DoWhileStatement),
        TokenKind::Return => Some(StatementKind::ReturnStatement),
        TokenKind::Break => Some(StatementKind::BreakStatement),
        TokenKind::Continue => Some(StatementKind::ContinueStatement),
        TokenKind::Throw => Some(StatementKind::ThrowStatement),
        TokenKind::Try => Some(StatementKind::TryStatement),
        TokenKind::OpenBrace => Some(StatementKind::BlockStatement),
        TokenKind::Semicolon => Some(StatementKind::EmptyStatement),
        TokenKind::Identifier => {
            match text {
                "namespace" | "module" => Some(StatementKind::NamespaceDeclaration),
                "declare" => None, // Need to look ahead
                "abstract" => Some(StatementKind::ClassDeclaration),
                "async" => Some(StatementKind::FunctionDeclaration),
                "switch" => Some(StatementKind::SwitchStatement),
                "debugger" => Some(StatementKind::DebuggerStatement),
                "with" => Some(StatementKind::WithStatement),
                _ => Some(StatementKind::ExpressionStatement),
            }
        }
        _ => Some(StatementKind::ExpressionStatement),
    }
}

/// Variable declaration kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarDeclKind {
    Var,
    Let,
    Const,
}

impl VarDeclKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "var" => Some(VarDeclKind::Var),
            "let" => Some(VarDeclKind::Let),
            "const" => Some(VarDeclKind::Const),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            VarDeclKind::Var => "var",
            VarDeclKind::Let => "let",
            VarDeclKind::Const => "const",
        }
    }
}

/// For statement variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForVariant {
    Regular,  // for (;;)
    In,       // for..in
    Of,       // for..of
    Await,    // for await..of
}

/// Export kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Named,      // export { a, b }
    Default,    // export default
    All,        // export *
    Declaration,// export const x = 1
    Namespace,  // export * as ns
}

/// Import kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportKind {
    Named,      // import { a, b }
    Default,    // import x
    Namespace,  // import * as x
    SideEffect, // import "module"
    Equals,     // import x = require("module")
    Type,       // import type { A }
}

/// Automatic Semicolon Insertion (ASI) context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ASIContext {
    /// Normal statement context - ASI applies
    Normal,
    /// For loop header - semicolons are required
    ForHeader,
    /// No ASI - semicolon required
    Required,
}

/// Check if ASI should insert a semicolon
pub fn should_insert_semicolon(
    current_kind: TokenKind,
    previous_line: usize,
    current_line: usize,
) -> bool {
    // ASI inserts semicolon when:
    // 1. There's a line terminator between tokens
    // 2. Current token is }
    // 3. Current token is EOF

    if current_kind == TokenKind::CloseBrace || current_kind == TokenKind::Eof {
        return true;
    }

    // Line terminator between tokens
    if current_line > previous_line {
        // Check if current token can start a statement (restricted productions)
        !matches!(
            current_kind,
            TokenKind::Dot
                | TokenKind::OpenParen
                | TokenKind::OpenBracket
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Asterisk
                | TokenKind::Comma
        )
    } else {
        false
    }
}

/// Check if statement requires terminating semicolon
pub fn requires_semicolon(kind: StatementKind) -> bool {
    matches!(
        kind,
        StatementKind::VariableDeclaration
            | StatementKind::ExpressionStatement
            | StatementKind::ReturnStatement
            | StatementKind::BreakStatement
            | StatementKind::ContinueStatement
            | StatementKind::ThrowStatement
            | StatementKind::ImportDeclaration
            | StatementKind::ExportDeclaration
            | StatementKind::TypeAliasDeclaration
            | StatementKind::DebuggerStatement
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starts_statement() {
        assert!(starts_statement(TokenKind::Let, "let"));
        assert!(starts_statement(TokenKind::Const, "const"));
        assert!(starts_statement(TokenKind::Function, "function"));
        assert!(starts_statement(TokenKind::Class, "class"));
        assert!(starts_statement(TokenKind::If, "if"));
        assert!(starts_statement(TokenKind::For, "for"));
        assert!(starts_statement(TokenKind::While, "while"));
        assert!(starts_statement(TokenKind::Return, "return"));
        assert!(starts_statement(TokenKind::Identifier, "namespace"));
        assert!(starts_statement(TokenKind::Identifier, "declare"));
    }

    #[test]
    fn test_starts_declaration() {
        assert!(starts_declaration(TokenKind::Let, "let"));
        assert!(starts_declaration(TokenKind::Function, "function"));
        assert!(starts_declaration(TokenKind::Class, "class"));
        assert!(starts_declaration(TokenKind::Interface, "interface"));
        assert!(starts_declaration(TokenKind::Identifier, "namespace"));
        assert!(!starts_declaration(TokenKind::If, "if"));
        assert!(!starts_declaration(TokenKind::Return, "return"));
    }

    #[test]
    fn test_determine_statement_kind() {
        assert_eq!(
            determine_statement_kind(TokenKind::Let, "let"),
            Some(StatementKind::VariableDeclaration)
        );
        assert_eq!(
            determine_statement_kind(TokenKind::If, "if"),
            Some(StatementKind::IfStatement)
        );
        assert_eq!(
            determine_statement_kind(TokenKind::For, "for"),
            Some(StatementKind::ForStatement)
        );
        assert_eq!(
            determine_statement_kind(TokenKind::Identifier, "namespace"),
            Some(StatementKind::NamespaceDeclaration)
        );
    }

    #[test]
    fn test_var_decl_kind() {
        assert_eq!(VarDeclKind::from_str("let"), Some(VarDeclKind::Let));
        assert_eq!(VarDeclKind::from_str("const"), Some(VarDeclKind::Const));
        assert_eq!(VarDeclKind::from_str("var"), Some(VarDeclKind::Var));
        assert_eq!(VarDeclKind::from_str("invalid"), None);

        assert_eq!(VarDeclKind::Let.as_str(), "let");
        assert_eq!(VarDeclKind::Const.as_str(), "const");
        assert_eq!(VarDeclKind::Var.as_str(), "var");
    }

    #[test]
    fn test_asi_should_insert() {
        // Should insert at close brace
        assert!(should_insert_semicolon(TokenKind::CloseBrace, 1, 1));

        // Should insert at EOF
        assert!(should_insert_semicolon(TokenKind::Eof, 1, 1));

        // Should insert when line changes and next token starts statement
        assert!(should_insert_semicolon(TokenKind::Identifier, 1, 2));

        // Should NOT insert when next token continues expression
        assert!(!should_insert_semicolon(TokenKind::Dot, 1, 2));
        assert!(!should_insert_semicolon(TokenKind::OpenParen, 1, 2));
    }

    #[test]
    fn test_requires_semicolon() {
        assert!(requires_semicolon(StatementKind::VariableDeclaration));
        assert!(requires_semicolon(StatementKind::ExpressionStatement));
        assert!(requires_semicolon(StatementKind::ReturnStatement));
        assert!(requires_semicolon(StatementKind::ImportDeclaration));

        assert!(!requires_semicolon(StatementKind::BlockStatement));
        assert!(!requires_semicolon(StatementKind::IfStatement));
        assert!(!requires_semicolon(StatementKind::ForStatement));
        assert!(!requires_semicolon(StatementKind::FunctionDeclaration));
    }

    #[test]
    fn test_for_variant() {
        assert_ne!(ForVariant::Regular, ForVariant::In);
        assert_ne!(ForVariant::In, ForVariant::Of);
        assert_ne!(ForVariant::Of, ForVariant::Await);
    }

    #[test]
    fn test_export_kind() {
        assert_ne!(ExportKind::Named, ExportKind::Default);
        assert_ne!(ExportKind::All, ExportKind::Declaration);
    }

    #[test]
    fn test_import_kind() {
        assert_ne!(ImportKind::Named, ImportKind::Default);
        assert_ne!(ImportKind::Namespace, ImportKind::SideEffect);
    }
}
