//! TypeScript Parser
//!
//! Recursive descent parser for TypeScript source code.
//! Produces an AST from a token stream.

use crate::ast::*;
use crate::lexer::{Lexer, Token};
use zang_core::{Span, StringInterner};

/// Parser for TypeScript source code
pub struct Parser<'a> {
    /// The lexer providing tokens
    lexer: Lexer<'a>,
    /// The source string
    source: &'a str,
    /// Current token
    current: Token,
    /// Previous token (for span tracking)
    previous: Token,
    /// String interner for identifiers
    interner: &'a StringInterner,
    /// Parse errors collected during parsing
    errors: Vec<ParseError>,
}

/// A parse error with location information
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Error message
    pub message: String,
    /// Location of the error
    pub span: Span,
    /// Error code for categorization
    pub code: ParseErrorCode,
}

/// Parse error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorCode {
    /// Unexpected token
    UnexpectedToken,
    /// Expected a specific token
    ExpectedToken,
    /// Invalid syntax
    InvalidSyntax,
    /// Unterminated construct
    Unterminated,
    /// Other error
    Other,
}

/// Result of parsing
pub struct ParseResult {
    /// The source file AST node
    pub source_file: SourceFile,
    /// Parse errors (empty if successful)
    pub errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given source
    pub fn new(
        source: &'a str,
        _arena: &zang_core::Arena, // Reserved for future use
        interner: &'a StringInterner,
    ) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer.next_token();

        Self {
            lexer,
            source,
            current,
            previous: Token::new(SyntaxKind::EndOfFile, Span::new(0, 0)),
            interner,
            errors: Vec::new(),
        }
    }

    /// Parses the source and returns a SourceFile
    pub fn parse(mut self, file_name: &str) -> ParseResult {
        let start = self.current.span.start;
        let mut statements = Vec::new();

        // Parse statements until EOF
        while !self.is_at_end() {
            match self.parse_statement() {
                Some(stmt) => statements.push(stmt),
                None => {
                    // Error recovery: skip to next statement
                    self.advance();
                }
            }
        }

        let end = self.current.span.end;
        let file_name_interned = self.interner.intern(file_name);

        let source_file = SourceFile {
            span: Span::new(start, end),
            statements,
            end_of_file_token: self.current.span,
            file_name: file_name_interned,
        };

        ParseResult {
            source_file,
            errors: self.errors,
        }
    }

    // ========================================================================
    // Token management
    // ========================================================================

    /// Returns true if we're at the end of file
    fn is_at_end(&self) -> bool {
        self.current.kind == SyntaxKind::EndOfFile
    }

    /// Advances to the next token
    fn advance(&mut self) -> Token {
        self.previous = self.current.clone();
        self.current = self.lexer.next_token();
        self.previous.clone()
    }

    /// Gets the text for the current token
    fn current_text(&self) -> &str {
        let start = self.current.span.start as usize;
        let end = self.current.span.end as usize;
        &self.source[start..end]
    }

    /// Checks if the current token matches the expected kind
    fn check(&self, kind: SyntaxKind) -> bool {
        self.current.kind == kind
    }

    /// Consumes the current token if it matches, returns true if consumed
    fn match_token(&mut self, kind: SyntaxKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Expects and consumes a specific token, reports error if not found
    fn expect(&mut self, kind: SyntaxKind, message: &str) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            self.error(ParseErrorCode::ExpectedToken, message);
            false
        }
    }

    /// Reports a parse error at the current position
    fn error(&mut self, code: ParseErrorCode, message: &str) {
        self.errors.push(ParseError {
            message: message.to_string(),
            span: self.current.span,
            code,
        });
    }

    // ========================================================================
    // Statement parsing (minimal implementation)
    // ========================================================================

    /// Parses a statement
    fn parse_statement(&mut self) -> Option<Statement> {
        match self.current.kind {
            SyntaxKind::LetKeyword | SyntaxKind::ConstKeyword | SyntaxKind::VarKeyword => {
                self.parse_variable_statement()
            }
            SyntaxKind::Semicolon => {
                let span = self.current.span;
                self.advance();
                Some(Statement::Empty(span))
            }
            _ => self.parse_expression_statement(),
        }
    }

    /// Parses a variable statement
    fn parse_variable_statement(&mut self) -> Option<Statement> {
        let start = self.current.span.start;

        let flags = match self.current.kind {
            SyntaxKind::LetKeyword => VariableDeclarationKind::Let,
            SyntaxKind::ConstKeyword => VariableDeclarationKind::Const,
            SyntaxKind::VarKeyword => VariableDeclarationKind::Var,
            _ => return None,
        };
        self.advance();

        let mut declarations = Vec::new();

        loop {
            if let Some(decl) = self.parse_variable_declaration() {
                declarations.push(decl);
            } else {
                break;
            }

            if !self.match_token(SyntaxKind::Comma) {
                break;
            }
        }

        self.expect(SyntaxKind::Semicolon, "Expected ';'");
        let end = self.previous.span.end;

        let declaration_list = VariableDeclarationList {
            span: Span::new(start, end),
            declarations,
            flags,
        };

        Some(Statement::Variable(VariableStatement {
            span: Span::new(start, end),
            declaration_list,
        }))
    }

    /// Parses a variable declaration
    fn parse_variable_declaration(&mut self) -> Option<VariableDeclaration> {
        let start = self.current.span.start;

        let name = self.parse_binding_name()?;

        // Optional type annotation
        let type_annotation = if self.match_token(SyntaxKind::Colon) {
            self.parse_type()
        } else {
            None
        };

        // Optional initializer
        let initializer = if self.match_token(SyntaxKind::Equals) {
            self.parse_expression()
        } else {
            None
        };

        let end = self.previous.span.end;

        Some(VariableDeclaration {
            span: Span::new(start, end),
            name,
            type_annotation,
            initializer,
        })
    }

    /// Parses a binding name
    fn parse_binding_name(&mut self) -> Option<BindingName> {
        if self.check(SyntaxKind::Identifier) {
            let ident = self.parse_identifier()?;
            Some(BindingName::Identifier(ident))
        } else {
            self.error(ParseErrorCode::ExpectedToken, "Expected identifier");
            None
        }
    }

    /// Parses an identifier
    fn parse_identifier(&mut self) -> Option<Identifier> {
        if self.check(SyntaxKind::Identifier) {
            let span = self.current.span;
            let text = self.current_text();
            let name = self.interner.intern(text);
            self.advance();
            Some(Identifier { span, name })
        } else {
            None
        }
    }

    /// Parses an expression statement
    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let start = self.current.span.start;
        let expression = self.parse_expression()?;
        self.expect(SyntaxKind::Semicolon, "Expected ';'");
        let end = self.previous.span.end;

        Some(Statement::Expression(ExpressionStatement {
            span: Span::new(start, end),
            expression,
        }))
    }

    // ========================================================================
    // Type parsing (minimal)
    // ========================================================================

    /// Parses a type
    fn parse_type(&mut self) -> Option<TypeNode> {
        match self.current.kind {
            SyntaxKind::Identifier => {
                let start = self.current.span.start;
                let ident = self.parse_identifier()?;
                let end = self.previous.span.end;
                Some(TypeNode::Reference(TypeReferenceNode {
                    span: Span::new(start, end),
                    type_name: EntityName::Identifier(ident),
                    type_arguments: None,
                }))
            }
            SyntaxKind::StringKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::AnyKeyword => {
                let span = self.current.span;
                let keyword = self.current.kind;
                self.advance();
                Some(TypeNode::Keyword(KeywordTypeNode { span, keyword }))
            }
            _ => {
                self.error(ParseErrorCode::UnexpectedToken, "Expected type");
                None
            }
        }
    }

    // ========================================================================
    // Expression parsing (minimal)
    // ========================================================================

    /// Parses an expression
    fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_primary_expression()
    }

    /// Parses a primary expression
    fn parse_primary_expression(&mut self) -> Option<Expression> {
        match self.current.kind {
            SyntaxKind::Identifier => {
                let ident = self.parse_identifier()?;
                Some(Expression::Identifier(ident))
            }
            SyntaxKind::NumericLiteral => {
                let span = self.current.span;
                let text = self.current_text();
                let value = text.parse().unwrap_or(0.0);
                self.advance();
                Some(Expression::NumericLiteral(NumericLiteral { span, value }))
            }
            SyntaxKind::StringLiteral => {
                let span = self.current.span;
                let text = self.current_text();
                let inner = if text.len() >= 2 { &text[1..text.len() - 1] } else { text };
                let value = self.interner.intern(inner);
                self.advance();
                Some(Expression::StringLiteral(StringLiteral { span, value }))
            }
            SyntaxKind::TrueKeyword => {
                let span = self.current.span;
                self.advance();
                Some(Expression::BooleanLiteral(BooleanLiteral { span, value: true }))
            }
            SyntaxKind::FalseKeyword => {
                let span = self.current.span;
                self.advance();
                Some(Expression::BooleanLiteral(BooleanLiteral { span, value: false }))
            }
            SyntaxKind::NullKeyword => {
                let span = self.current.span;
                self.advance();
                Some(Expression::NullLiteral(NullLiteral { span }))
            }
            _ => {
                self.error(ParseErrorCode::UnexpectedToken, "Expected expression");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zang_core::Arena;

    #[test]
    fn test_parser_creation() {
        let arena = Arena::new();
        let interner = StringInterner::new();
        let _parser = Parser::new("let x = 1;", &arena, &interner);
    }
}
