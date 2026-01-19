//! Main Parser implementation
//!
//! Integrates scanner with statement, expression, and type parsing.
//! Features:
//! - Proper error recovery
//! - Automatic semicolon insertion (ASI)
//! - Precedence climbing for expressions
//! - TypeScript-specific syntax support

use crate::thin_parser::{Scanner, Token, TokenKind, AstNode, ParseError};
use super::statements::{self, StatementKind};
use super::expressions::{self, Precedence};
use super::error_recovery::ErrorRecovery;

/// Parser configuration options
#[derive(Debug, Clone)]
pub struct ParserOptions {
    /// Allow JSX syntax
    pub jsx: bool,
    /// Allow decorators
    pub decorators: bool,
    /// Source file is a module (vs script)
    pub module: bool,
    /// Allow TypeScript syntax
    pub typescript: bool,
    /// Strict mode
    pub strict: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        ParserOptions {
            jsx: false,
            decorators: true,
            module: true,
            typescript: true,
            strict: false,
        }
    }
}

/// Parser state flags
#[derive(Debug, Clone, Copy, Default)]
pub struct ParserState {
    /// In async function context
    pub in_async: bool,
    /// In generator function context
    pub in_generator: bool,
    /// In iteration statement (for, while, do)
    pub in_iteration: bool,
    /// In switch statement
    pub in_switch: bool,
    /// In function context
    pub in_function: bool,
    /// In class context
    pub in_class: bool,
    /// Allow await expressions
    pub allow_await: bool,
    /// Allow yield expressions
    pub allow_yield: bool,
    /// Allow in operator (disabled in for-in header)
    pub allow_in: bool,
    /// Expecting type (affects '<' parsing)
    pub in_type: bool,
}

impl ParserState {
    pub fn new() -> Self {
        Self {
            allow_in: true, // Default is to allow 'in'
            ..Default::default()
        }
    }

    /// Create state for async function
    pub fn with_async(mut self, is_async: bool) -> Self {
        self.in_async = is_async;
        self.allow_await = is_async;
        self
    }

    /// Create state for generator function
    pub fn with_generator(mut self, is_generator: bool) -> Self {
        self.in_generator = is_generator;
        self.allow_yield = is_generator;
        self
    }

    /// Create state for function context
    pub fn in_function_context(mut self) -> Self {
        self.in_function = true;
        self
    }

    /// Create state for iteration context
    pub fn in_iteration_context(mut self) -> Self {
        self.in_iteration = true;
        self
    }

    /// Create state for switch context
    pub fn in_switch_context(mut self) -> Self {
        self.in_switch = true;
        self
    }

    /// Create state for class context
    pub fn in_class_context(mut self) -> Self {
        self.in_class = true;
        self
    }

    /// Create state with 'in' operator disabled
    pub fn without_in(mut self) -> Self {
        self.allow_in = false;
        self
    }

    /// Create state for type parsing
    pub fn in_type_context(mut self) -> Self {
        self.in_type = true;
        self
    }
}

/// Main Parser struct
pub struct Parser<'a> {
    /// Source code (reserved for future use)
    #[allow(dead_code)]
    source: &'a str,
    /// Scanner for tokenization
    scanner: Scanner<'a>,
    /// Current token
    current: Token<'a>,
    /// Previous token (for ASI)
    previous: Option<Token<'a>>,
    /// Parser options
    options: ParserOptions,
    /// Parser state
    state: ParserState,
    /// Collected errors
    errors: Vec<ParseError>,
    /// Error recovery helper (reserved for future use)
    #[allow(dead_code)]
    recovery: ErrorRecovery,
    /// Consecutive error count
    consecutive_errors: usize,
    /// Max consecutive errors before bailing
    max_errors: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser
    pub fn new(source: &'a str) -> Self {
        Self::with_options(source, ParserOptions::default())
    }

    /// Create a new parser with options
    pub fn with_options(source: &'a str, options: ParserOptions) -> Self {
        let mut scanner = Scanner::new(source);
        let current = scanner.next_token();

        Parser {
            source,
            scanner,
            current,
            previous: None,
            options,
            state: ParserState::new(),
            errors: Vec::new(),
            recovery: ErrorRecovery::new(),
            consecutive_errors: 0,
            max_errors: 10,
        }
    }

    /// Parse the source and return AST
    pub fn parse(&mut self) -> Result<AstNode<'a>, Vec<ParseError>> {
        let program = self.parse_program();

        if self.errors.is_empty() {
            Ok(program)
        } else {
            Err(self.errors.clone())
        }
    }

    /// Parse a program (module or script)
    fn parse_program(&mut self) -> AstNode<'a> {
        let mut statements = Vec::new();

        while !self.at_end() {
            match self.parse_statement_or_declaration() {
                Some(stmt) => {
                    self.consecutive_errors = 0;
                    statements.push(stmt);
                }
                None => {
                    if !self.try_recover() {
                        break;
                    }
                }
            }
        }

        AstNode::Program { statements }
    }

    /// Parse a statement or declaration
    fn parse_statement_or_declaration(&mut self) -> Option<AstNode<'a>> {
        // Handle modifiers (reserved for future modifier handling)
        let _is_export = self.check(TokenKind::Export);
        let _is_declare = self.current.text == "declare";
        let _is_abstract = self.current.text == "abstract";
        let _is_async = self.current.text == "async";

        // Determine statement kind
        let kind = statements::determine_statement_kind(self.current.kind, self.current.text);

        match kind {
            Some(StatementKind::VariableDeclaration) => self.parse_variable_declaration(),
            Some(StatementKind::FunctionDeclaration) => self.parse_function_declaration(),
            Some(StatementKind::ClassDeclaration) => self.parse_class_declaration(),
            Some(StatementKind::InterfaceDeclaration) => self.parse_interface_declaration(),
            Some(StatementKind::TypeAliasDeclaration) => self.parse_type_alias_declaration(),
            Some(StatementKind::EnumDeclaration) => self.parse_enum_declaration(),
            Some(StatementKind::NamespaceDeclaration) => self.parse_namespace_declaration(),
            Some(StatementKind::ImportDeclaration) => self.parse_import_declaration(),
            Some(StatementKind::ExportDeclaration) => self.parse_export_declaration(),
            Some(StatementKind::BlockStatement) => self.parse_block_statement(),
            Some(StatementKind::IfStatement) => self.parse_if_statement(),
            Some(StatementKind::ForStatement) => self.parse_for_statement(),
            Some(StatementKind::WhileStatement) => self.parse_while_statement(),
            Some(StatementKind::DoWhileStatement) => self.parse_do_while_statement(),
            Some(StatementKind::SwitchStatement) => self.parse_switch_statement(),
            Some(StatementKind::TryStatement) => self.parse_try_statement(),
            Some(StatementKind::ReturnStatement) => self.parse_return_statement(),
            Some(StatementKind::BreakStatement) => self.parse_break_statement(),
            Some(StatementKind::ContinueStatement) => self.parse_continue_statement(),
            Some(StatementKind::ThrowStatement) => self.parse_throw_statement(),
            Some(StatementKind::EmptyStatement) => self.parse_empty_statement(),
            Some(StatementKind::DebuggerStatement) => self.parse_debugger_statement(),
            Some(StatementKind::ExpressionStatement) | None => self.parse_expression_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    // =========================================================================
    // Token manipulation
    // =========================================================================

    /// Advance to the next token
    fn advance(&mut self) {
        self.previous = Some(self.current.clone());
        self.current = self.scanner.next_token();
    }

    /// Check if current token matches kind
    fn check(&self, kind: TokenKind) -> bool {
        self.current.kind == kind
    }

    /// Check if current token matches any of the kinds
    fn check_any(&self, kinds: &[TokenKind]) -> bool {
        kinds.contains(&self.current.kind)
    }

    /// Check if at end of input
    fn at_end(&self) -> bool {
        self.current.kind == TokenKind::Eof
    }

    /// Expect a specific token, report error if not found
    fn expect(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            self.error(format!("Expected {:?}, found {:?}", kind, self.current.kind));
            false
        }
    }

    /// Optional consume - advance if matches
    fn consume(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Report a parse error
    fn error(&mut self, message: String) {
        self.errors.push(ParseError {
            message,
            start: self.current.start,
            end: self.current.end,
            line: self.current.line,
            column: self.current.column,
        });
        self.consecutive_errors += 1;
    }

    /// Try to recover from error
    fn try_recover(&mut self) -> bool {
        if self.consecutive_errors >= self.max_errors {
            return false;
        }

        // Skip to next statement boundary
        while !self.at_end() {
            if self.check(TokenKind::Semicolon) {
                self.advance();
                return true;
            }
            if self.check(TokenKind::CloseBrace) {
                return true;
            }
            if statements::starts_statement(self.current.kind, self.current.text) {
                return true;
            }
            self.advance();
        }

        false
    }

    /// Handle automatic semicolon insertion
    fn expect_semicolon(&mut self) {
        if self.check(TokenKind::Semicolon) {
            self.advance();
            return;
        }

        // ASI: insert semicolon if:
        // 1. Current token is }
        // 2. Current token is EOF
        // 3. Previous token was followed by line terminator
        if self.check(TokenKind::CloseBrace) || self.at_end() {
            return;
        }

        if let Some(ref prev) = self.previous {
            if self.current.line > prev.line {
                return;
            }
        }

        self.error("Expected ';'".to_string());
    }

    // =========================================================================
    // Statement parsing (delegated to thin_parser for now)
    // =========================================================================

    fn parse_variable_declaration(&mut self) -> Option<AstNode<'a>> {
        let kind = self.current.text;
        self.advance();

        let mut declarations = Vec::new();

        loop {
            if !self.check(TokenKind::Identifier) {
                self.error("Expected identifier".to_string());
                return None;
            }

            let name = self.current.text;
            self.advance();

            let type_annotation = if self.check(TokenKind::Colon) {
                self.advance();
                Some(Box::new(self.parse_type_annotation()?))
            } else {
                None
            };

            let init = if self.check(TokenKind::Equals) {
                self.advance();
                Some(Box::new(self.parse_assignment_expression()?))
            } else {
                None
            };

            declarations.push(AstNode::VariableDeclarator {
                name,
                type_annotation,
                init,
            });

            if !self.consume(TokenKind::Comma) {
                break;
            }
        }

        self.expect_semicolon();

        Some(AstNode::VariableDeclaration { kind, declarations })
    }

    fn parse_function_declaration(&mut self) -> Option<AstNode<'a>> {
        let is_async = self.current.text == "async";
        if is_async {
            self.advance();
        }

        self.expect(TokenKind::Function);

        let name = if self.check(TokenKind::Identifier) {
            let n = self.current.text;
            self.advance();
            Some(n)
        } else {
            None
        };

        self.expect(TokenKind::OpenParen);
        let params = self.parse_parameters();
        self.expect(TokenKind::CloseParen);

        let return_type = if self.check(TokenKind::Colon) {
            self.advance();
            Some(Box::new(self.parse_type_annotation()?))
        } else {
            None
        };

        let body = if self.check(TokenKind::OpenBrace) {
            Some(Box::new(self.parse_block_statement()?))
        } else {
            None
        };

        Some(AstNode::FunctionDeclaration {
            name,
            params,
            return_type,
            body,
            is_async,
        })
    }

    fn parse_class_declaration(&mut self) -> Option<AstNode<'a>> {
        self.expect(TokenKind::Class);

        let name = if self.check(TokenKind::Identifier) {
            let n = self.current.text;
            self.advance();
            Some(n)
        } else {
            None
        };

        let super_class = if self.current.text == "extends" {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.expect(TokenKind::OpenBrace);

        // Skip class body for now
        let body = Vec::new();
        while !self.check(TokenKind::CloseBrace) && !self.at_end() {
            self.advance();
        }

        self.expect(TokenKind::CloseBrace);

        Some(AstNode::ClassDeclaration {
            name,
            super_class,
            body,
        })
    }

    fn parse_interface_declaration(&mut self) -> Option<AstNode<'a>> {
        self.expect(TokenKind::Interface);

        let name = if self.check(TokenKind::Identifier) {
            let n = self.current.text;
            self.advance();
            n
        } else {
            self.error("Expected interface name".to_string());
            return None;
        };

        let mut extends = Vec::new();
        if self.current.text == "extends" {
            self.advance();
            loop {
                if self.check(TokenKind::Identifier) {
                    extends.push(AstNode::Identifier { name: self.current.text });
                    self.advance();
                }
                if !self.consume(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(TokenKind::OpenBrace);

        let body = Vec::new();
        while !self.check(TokenKind::CloseBrace) && !self.at_end() {
            self.advance();
        }

        self.expect(TokenKind::CloseBrace);

        Some(AstNode::InterfaceDeclaration { name, extends, body })
    }

    fn parse_type_alias_declaration(&mut self) -> Option<AstNode<'a>> {
        self.expect(TokenKind::Type);

        let name = if self.check(TokenKind::Identifier) {
            let n = self.current.text;
            self.advance();
            n
        } else {
            self.error("Expected type name".to_string());
            return None;
        };

        let type_params = Vec::new();

        self.expect(TokenKind::Equals);

        let type_annotation = Box::new(self.parse_type_annotation()?);

        self.expect_semicolon();

        Some(AstNode::TypeAliasDeclaration {
            name,
            type_params,
            type_annotation,
        })
    }

    fn parse_enum_declaration(&mut self) -> Option<AstNode<'a>> {
        // Simplified: skip for now
        while !self.check(TokenKind::CloseBrace) && !self.at_end() {
            self.advance();
        }
        self.advance();
        Some(AstNode::EmptyStatement)
    }

    fn parse_namespace_declaration(&mut self) -> Option<AstNode<'a>> {
        // Skip 'namespace' keyword
        self.advance();
        // Simplified: skip for now
        while !self.check(TokenKind::CloseBrace) && !self.at_end() {
            self.advance();
        }
        if self.check(TokenKind::CloseBrace) {
            self.advance();
        }
        Some(AstNode::EmptyStatement)
    }

    fn parse_import_declaration(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'import'

        let specifiers = Vec::new();

        // Side-effect import
        if self.check(TokenKind::StringLiteral) {
            let source = self.current.text;
            self.advance();
            self.expect_semicolon();
            return Some(AstNode::ImportDeclaration { specifiers, source });
        }

        // Skip complex imports for now
        while !self.check(TokenKind::From) && !self.at_end() && !self.check(TokenKind::Semicolon) {
            self.advance();
        }

        if self.consume(TokenKind::From) {
            // nothing
        }

        let source = if self.check(TokenKind::StringLiteral) {
            let s = self.current.text;
            self.advance();
            s
        } else {
            ""
        };

        self.expect_semicolon();

        Some(AstNode::ImportDeclaration { specifiers, source })
    }

    fn parse_export_declaration(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'export'

        let declaration = if self.check(TokenKind::Default) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else if statements::starts_declaration(self.current.kind, self.current.text) {
            Some(Box::new(self.parse_statement_or_declaration()?))
        } else {
            None
        };

        if declaration.is_none() {
            self.expect_semicolon();
        }

        Some(AstNode::ExportDeclaration { declaration })
    }

    fn parse_block_statement(&mut self) -> Option<AstNode<'a>> {
        self.expect(TokenKind::OpenBrace);

        let mut statements = Vec::new();

        while !self.check(TokenKind::CloseBrace) && !self.at_end() {
            if let Some(stmt) = self.parse_statement_or_declaration() {
                self.consecutive_errors = 0;
                statements.push(stmt);
            } else {
                if !self.try_recover() {
                    break;
                }
            }
        }

        self.expect(TokenKind::CloseBrace);

        Some(AstNode::BlockStatement { statements })
    }

    fn parse_if_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'if'
        self.expect(TokenKind::OpenParen);
        let test = Box::new(self.parse_expression()?);
        self.expect(TokenKind::CloseParen);

        let consequent = Box::new(self.parse_statement_or_declaration()?);

        let alternate = if self.check(TokenKind::Else) {
            self.advance();
            Some(Box::new(self.parse_statement_or_declaration()?))
        } else {
            None
        };

        Some(AstNode::IfStatement {
            test,
            consequent,
            alternate,
        })
    }

    fn parse_for_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'for'
        self.expect(TokenKind::OpenParen);

        // Save state, disable 'in' operator
        let old_state = self.state;
        self.state = self.state.without_in();

        let init = if self.check(TokenKind::Semicolon) {
            None
        } else if self.check_any(&[TokenKind::Let, TokenKind::Const, TokenKind::Var]) {
            let kind = self.current.text;
            self.advance();

            let mut declarations = Vec::new();
            let name = if self.check(TokenKind::Identifier) {
                let n = self.current.text;
                self.advance();
                n
            } else {
                self.error("Expected identifier".to_string());
                return None;
            };

            let type_annotation = if self.check(TokenKind::Colon) {
                self.advance();
                Some(Box::new(self.parse_type_annotation()?))
            } else {
                None
            };

            let init_val = if self.check(TokenKind::Equals) {
                self.advance();
                Some(Box::new(self.parse_assignment_expression()?))
            } else {
                None
            };

            declarations.push(AstNode::VariableDeclarator {
                name,
                type_annotation,
                init: init_val,
            });

            // More declarations?
            while self.consume(TokenKind::Comma) {
                let name = if self.check(TokenKind::Identifier) {
                    let n = self.current.text;
                    self.advance();
                    n
                } else {
                    break;
                };

                let type_annotation = if self.check(TokenKind::Colon) {
                    self.advance();
                    Some(Box::new(self.parse_type_annotation()?))
                } else {
                    None
                };

                let init_val = if self.check(TokenKind::Equals) {
                    self.advance();
                    Some(Box::new(self.parse_assignment_expression()?))
                } else {
                    None
                };

                declarations.push(AstNode::VariableDeclarator {
                    name,
                    type_annotation,
                    init: init_val,
                });
            }

            Some(Box::new(AstNode::VariableDeclaration { kind, declarations }))
        } else {
            Some(Box::new(self.parse_expression()?))
        };

        // Restore state
        self.state = old_state;

        self.expect(TokenKind::Semicolon);

        let test = if self.check(TokenKind::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };
        self.expect(TokenKind::Semicolon);

        let update = if self.check(TokenKind::CloseParen) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };
        self.expect(TokenKind::CloseParen);

        let body = Box::new(self.parse_statement_or_declaration()?);

        Some(AstNode::ForStatement {
            init,
            test,
            update,
            body,
        })
    }

    fn parse_while_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'while'
        self.expect(TokenKind::OpenParen);
        let test = Box::new(self.parse_expression()?);
        self.expect(TokenKind::CloseParen);
        let body = Box::new(self.parse_statement_or_declaration()?);

        Some(AstNode::WhileStatement { test, body })
    }

    fn parse_do_while_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'do'
        let body = Box::new(self.parse_statement_or_declaration()?);
        self.expect(TokenKind::While);
        self.expect(TokenKind::OpenParen);
        let test = Box::new(self.parse_expression()?);
        self.expect(TokenKind::CloseParen);
        self.expect_semicolon();

        Some(AstNode::WhileStatement { test, body })
    }

    fn parse_switch_statement(&mut self) -> Option<AstNode<'a>> {
        // Simplified
        while !self.check(TokenKind::CloseBrace) && !self.at_end() {
            self.advance();
        }
        self.advance();
        Some(AstNode::EmptyStatement)
    }

    fn parse_try_statement(&mut self) -> Option<AstNode<'a>> {
        // Simplified
        while !self.check(TokenKind::CloseBrace) && !self.at_end() {
            self.advance();
        }
        self.advance();
        // Handle catch/finally
        while self.check(TokenKind::Catch) || self.current.text == "finally" {
            self.advance();
            while !self.check(TokenKind::CloseBrace) && !self.at_end() {
                self.advance();
            }
            self.advance();
        }
        Some(AstNode::EmptyStatement)
    }

    fn parse_return_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'return'

        let argument = if !self.check(TokenKind::Semicolon) && !self.check(TokenKind::CloseBrace) && !self.at_end() {
            // Check for ASI
            if let Some(ref prev) = self.previous {
                if self.current.line > prev.line {
                    None
                } else {
                    Some(Box::new(self.parse_expression()?))
                }
            } else {
                Some(Box::new(self.parse_expression()?))
            }
        } else {
            None
        };

        self.expect_semicolon();

        Some(AstNode::ReturnStatement { argument })
    }

    fn parse_break_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'break'
        self.expect_semicolon();
        Some(AstNode::BreakStatement)
    }

    fn parse_continue_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'continue'
        self.expect_semicolon();
        Some(AstNode::ContinueStatement)
    }

    fn parse_throw_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'throw'
        let argument = Some(Box::new(self.parse_expression()?));
        self.expect_semicolon();
        Some(AstNode::ReturnStatement { argument })
    }

    fn parse_empty_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume ';'
        Some(AstNode::EmptyStatement)
    }

    fn parse_debugger_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'debugger'
        self.expect_semicolon();
        Some(AstNode::EmptyStatement)
    }

    fn parse_expression_statement(&mut self) -> Option<AstNode<'a>> {
        let expression = Box::new(self.parse_expression()?);
        self.expect_semicolon();
        Some(AstNode::ExpressionStatement { expression })
    }

    // =========================================================================
    // Expression parsing
    // =========================================================================

    fn parse_expression(&mut self) -> Option<AstNode<'a>> {
        self.parse_assignment_expression()
    }

    fn parse_assignment_expression(&mut self) -> Option<AstNode<'a>> {
        let left = self.parse_conditional_expression()?;

        if expressions::is_assignment_operator(self.current.kind) {
            let operator = self.current.text;
            self.advance();
            let right = self.parse_assignment_expression()?;
            return Some(AstNode::AssignmentExpression {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            });
        }

        Some(left)
    }

    fn parse_conditional_expression(&mut self) -> Option<AstNode<'a>> {
        let test = self.parse_binary_expression(Precedence::None as u8)?;

        if self.check(TokenKind::QuestionMark) {
            self.advance();
            let consequent = self.parse_assignment_expression()?;
            self.expect(TokenKind::Colon);
            let alternate = self.parse_assignment_expression()?;
            return Some(AstNode::ConditionalExpression {
                test: Box::new(test),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
            });
        }

        Some(test)
    }

    fn parse_binary_expression(&mut self, min_prec: u8) -> Option<AstNode<'a>> {
        let mut left = self.parse_unary_expression()?;

        loop {
            let prec = Precedence::of_binary_op(self.current.kind) as u8;
            if prec <= min_prec {
                break;
            }

            // Check 'in' operator restriction
            if self.current.text == "in" && !self.state.allow_in {
                break;
            }

            let operator = self.current.text;
            self.advance();

            let right_prec = if Precedence::is_right_associative(self.current.kind) {
                prec - 1
            } else {
                prec
            };

            let right = self.parse_binary_expression(right_prec)?;
            left = AstNode::BinaryExpression {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_unary_expression(&mut self) -> Option<AstNode<'a>> {
        if expressions::is_prefix_operator(self.current.kind, self.current.text) {
            let operator = self.current.text;
            self.advance();
            let argument = self.parse_unary_expression()?;
            return Some(AstNode::UnaryExpression {
                operator,
                argument: Box::new(argument),
                prefix: true,
            });
        }

        self.parse_postfix_expression()
    }

    fn parse_postfix_expression(&mut self) -> Option<AstNode<'a>> {
        let expr = self.parse_call_expression()?;

        // No line terminator before ++ or --
        if let Some(ref prev) = self.previous {
            if self.current.line > prev.line {
                return Some(expr);
            }
        }

        if expressions::is_postfix_operator(self.current.kind) {
            let operator = self.current.text;
            self.advance();
            return Some(AstNode::UnaryExpression {
                operator,
                argument: Box::new(expr),
                prefix: false,
            });
        }

        Some(expr)
    }

    fn parse_call_expression(&mut self) -> Option<AstNode<'a>> {
        let mut expr = self.parse_member_expression()?;

        loop {
            match self.current.kind {
                TokenKind::OpenParen => {
                    self.advance();
                    let arguments = self.parse_arguments();
                    self.expect(TokenKind::CloseParen);
                    expr = AstNode::CallExpression {
                        callee: Box::new(expr),
                        arguments,
                    };
                }
                TokenKind::LessThan if self.options.typescript && expressions::could_be_type_arguments(
                    self.previous.as_ref().map(|t| t.kind).unwrap_or(TokenKind::Unknown)
                ) => {
                    // Type arguments - skip for now
                    break;
                }
                _ => break,
            }
        }

        Some(expr)
    }

    fn parse_member_expression(&mut self) -> Option<AstNode<'a>> {
        let mut expr = self.parse_primary_expression()?;

        loop {
            match self.current.kind {
                TokenKind::Dot => {
                    self.advance();
                    if self.check(TokenKind::Identifier) {
                        let property = AstNode::Identifier { name: self.current.text };
                        self.advance();
                        expr = AstNode::MemberExpression {
                            object: Box::new(expr),
                            property: Box::new(property),
                            computed: false,
                        };
                    } else {
                        self.error("Expected property name".to_string());
                        break;
                    }
                }
                TokenKind::OpenBracket => {
                    self.advance();
                    let property = self.parse_expression()?;
                    self.expect(TokenKind::CloseBracket);
                    expr = AstNode::MemberExpression {
                        object: Box::new(expr),
                        property: Box::new(property),
                        computed: true,
                    };
                }
                _ => break,
            }
        }

        Some(expr)
    }

    fn parse_primary_expression(&mut self) -> Option<AstNode<'a>> {
        match self.current.kind {
            TokenKind::Identifier => {
                let name = self.current.text;
                self.advance();

                // Arrow function: x => expr
                if self.check(TokenKind::Arrow) {
                    return self.finish_arrow_function(vec![AstNode::Parameter {
                        name,
                        type_annotation: None,
                        optional: false,
                        default_value: None,
                    }]);
                }

                Some(AstNode::Identifier { name })
            }
            TokenKind::NumericLiteral => {
                let value = self.current.text;
                self.advance();
                Some(AstNode::NumericLiteral { value })
            }
            TokenKind::StringLiteral => {
                let value = self.current.text;
                self.advance();
                Some(AstNode::StringLiteral { value })
            }
            TokenKind::True => {
                self.advance();
                Some(AstNode::BooleanLiteral { value: true })
            }
            TokenKind::False => {
                self.advance();
                Some(AstNode::BooleanLiteral { value: false })
            }
            TokenKind::Null => {
                self.advance();
                Some(AstNode::NullLiteral)
            }
            TokenKind::This => {
                self.advance();
                Some(AstNode::Identifier { name: "this" })
            }
            TokenKind::OpenParen => {
                self.advance();

                // Empty parens: ()
                if self.check(TokenKind::CloseParen) {
                    self.advance();
                    if self.check(TokenKind::Arrow) {
                        return self.finish_arrow_function(vec![]);
                    }
                    return Some(AstNode::Identifier { name: "()" });
                }

                // Could be arrow function or parenthesized expression
                if self.check(TokenKind::Identifier) {
                    let params = self.parse_arrow_params_or_expr()?;
                    if self.check(TokenKind::Arrow) {
                        return self.finish_arrow_function(params);
                    }
                    // Single expression
                    if params.len() == 1 {
                        return Some(params.into_iter().next().unwrap());
                    }
                    return params.into_iter().last();
                }

                let expr = self.parse_expression()?;
                self.expect(TokenKind::CloseParen);

                if self.check(TokenKind::Arrow) {
                    return self.finish_arrow_function(vec![expr]);
                }

                Some(expr)
            }
            TokenKind::OpenBracket => {
                self.advance();
                let mut elements = Vec::new();

                while !self.check(TokenKind::CloseBracket) && !self.at_end() {
                    if let Some(elem) = self.parse_assignment_expression() {
                        elements.push(elem);
                    }
                    if !self.consume(TokenKind::Comma) {
                        break;
                    }
                }

                self.expect(TokenKind::CloseBracket);
                Some(AstNode::ArrayExpression { elements })
            }
            TokenKind::OpenBrace => {
                self.advance();
                let mut properties = Vec::new();

                while !self.check(TokenKind::CloseBrace) && !self.at_end() {
                    if let Some(prop) = self.parse_object_property() {
                        properties.push(prop);
                    }
                    if !self.consume(TokenKind::Comma) {
                        break;
                    }
                }

                self.expect(TokenKind::CloseBrace);
                Some(AstNode::ObjectExpression { properties })
            }
            TokenKind::New => {
                self.advance();
                let callee = self.parse_member_expression()?;
                let arguments = if self.check(TokenKind::OpenParen) {
                    self.advance();
                    let args = self.parse_arguments();
                    self.expect(TokenKind::CloseParen);
                    args
                } else {
                    Vec::new()
                };
                Some(AstNode::NewExpression {
                    callee: Box::new(callee),
                    arguments,
                })
            }
            TokenKind::Function => {
                self.parse_function_declaration()
            }
            _ => {
                self.error(format!("Unexpected token: {:?}", self.current.kind));
                None
            }
        }
    }

    fn finish_arrow_function(&mut self, params: Vec<AstNode<'a>>) -> Option<AstNode<'a>> {
        self.advance(); // consume '=>'
        let body = if self.check(TokenKind::OpenBrace) {
            self.parse_block_statement()?
        } else {
            self.parse_assignment_expression()?
        };
        Some(AstNode::ArrowFunctionExpression {
            params,
            body: Box::new(body),
            is_async: false,
        })
    }

    fn parse_arrow_params_or_expr(&mut self) -> Option<Vec<AstNode<'a>>> {
        let mut items = Vec::new();

        loop {
            if !self.check(TokenKind::Identifier) {
                break;
            }

            let name = self.current.text;
            self.advance();

            let optional = self.consume(TokenKind::QuestionMark);

            let type_annotation = if self.check(TokenKind::Colon) {
                self.advance();
                Some(Box::new(self.parse_type_annotation()?))
            } else {
                None
            };

            let default_value = if self.check(TokenKind::Equals) {
                self.advance();
                Some(Box::new(self.parse_assignment_expression()?))
            } else {
                None
            };

            if type_annotation.is_some() || optional || default_value.is_some() {
                items.push(AstNode::Parameter {
                    name,
                    type_annotation,
                    optional,
                    default_value,
                });
            } else {
                items.push(AstNode::Identifier { name });
            }

            if !self.consume(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::CloseParen);
        Some(items)
    }

    fn parse_parameters(&mut self) -> Vec<AstNode<'a>> {
        let mut params = Vec::new();

        while !self.check(TokenKind::CloseParen) && !self.at_end() {
            if let Some(param) = self.parse_parameter() {
                params.push(param);
            }
            if !self.consume(TokenKind::Comma) {
                break;
            }
        }

        params
    }

    fn parse_parameter(&mut self) -> Option<AstNode<'a>> {
        if !self.check(TokenKind::Identifier) {
            return None;
        }

        let name = self.current.text;
        self.advance();

        let optional = self.consume(TokenKind::QuestionMark);

        let type_annotation = if self.check(TokenKind::Colon) {
            self.advance();
            Some(Box::new(self.parse_type_annotation()?))
        } else {
            None
        };

        let default_value = if self.check(TokenKind::Equals) {
            self.advance();
            Some(Box::new(self.parse_assignment_expression()?))
        } else {
            None
        };

        Some(AstNode::Parameter {
            name,
            type_annotation,
            optional,
            default_value,
        })
    }

    fn parse_arguments(&mut self) -> Vec<AstNode<'a>> {
        let mut args = Vec::new();

        while !self.check(TokenKind::CloseParen) && !self.at_end() {
            if let Some(arg) = self.parse_assignment_expression() {
                args.push(arg);
            }
            if !self.consume(TokenKind::Comma) {
                break;
            }
        }

        args
    }

    fn parse_object_property(&mut self) -> Option<AstNode<'a>> {
        if !self.check(TokenKind::Identifier) && !self.check(TokenKind::StringLiteral) {
            return None;
        }

        let key = AstNode::Identifier { name: self.current.text };
        self.advance();

        if !self.check(TokenKind::Colon) {
            return Some(AstNode::Property {
                key: Box::new(key.clone()),
                value: Box::new(key),
                shorthand: true,
            });
        }

        self.advance();
        let value = self.parse_assignment_expression()?;

        Some(AstNode::Property {
            key: Box::new(key),
            value: Box::new(value),
            shorthand: false,
        })
    }

    // =========================================================================
    // Type parsing
    // =========================================================================

    fn parse_type_annotation(&mut self) -> Option<AstNode<'a>> {
        let type_node = self.parse_union_type()?;
        Some(AstNode::TypeAnnotation {
            type_node: Box::new(type_node),
        })
    }

    fn parse_union_type(&mut self) -> Option<AstNode<'a>> {
        let mut types = vec![self.parse_intersection_type()?];

        while self.check(TokenKind::Bar) {
            self.advance();
            types.push(self.parse_intersection_type()?);
        }

        if types.len() == 1 {
            Some(types.remove(0))
        } else {
            Some(AstNode::Identifier { name: "union" }) // Simplified
        }
    }

    fn parse_intersection_type(&mut self) -> Option<AstNode<'a>> {
        let mut types = vec![self.parse_primary_type()?];

        while self.check(TokenKind::Ampersand) {
            self.advance();
            types.push(self.parse_primary_type()?);
        }

        if types.len() == 1 {
            Some(types.remove(0))
        } else {
            Some(AstNode::Identifier { name: "intersection" }) // Simplified
        }
    }

    fn parse_primary_type(&mut self) -> Option<AstNode<'a>> {
        match self.current.kind {
            TokenKind::Identifier => {
                let name = self.current.text;
                self.advance();

                // Generic type arguments
                if self.check(TokenKind::LessThan) {
                    self.advance();
                    let mut type_args = Vec::new();
                    while !self.check(TokenKind::GreaterThan) && !self.at_end() {
                        type_args.push(self.parse_type_annotation()?);
                        if !self.consume(TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::GreaterThan);
                }

                // Array suffix
                let mut result = AstNode::Identifier { name };
                while self.check(TokenKind::OpenBracket) {
                    self.advance();
                    self.expect(TokenKind::CloseBracket);
                    result = AstNode::Identifier { name: "array" }; // Simplified
                }

                Some(result)
            }
            TokenKind::OpenParen => {
                self.advance();
                // Function type or parenthesized
                let inner = self.parse_union_type()?;
                self.expect(TokenKind::CloseParen);

                if self.check(TokenKind::Arrow) {
                    self.advance();
                    let _return_type = self.parse_type_annotation()?;
                }

                Some(inner)
            }
            TokenKind::OpenBrace => {
                // Object type - simplified
                self.advance();
                while !self.check(TokenKind::CloseBrace) && !self.at_end() {
                    self.advance();
                }
                self.expect(TokenKind::CloseBrace);
                Some(AstNode::Identifier { name: "object" })
            }
            TokenKind::OpenBracket => {
                // Tuple type - simplified
                self.advance();
                while !self.check(TokenKind::CloseBracket) && !self.at_end() {
                    self.advance();
                }
                self.expect(TokenKind::CloseBracket);
                Some(AstNode::Identifier { name: "tuple" })
            }
            _ => {
                self.error(format!("Unexpected token in type: {:?}", self.current.kind));
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new("let x = 5;");
        assert!(!parser.at_end());
    }

    #[test]
    fn test_parse_variable() {
        let mut parser = Parser::new("let x = 5;");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_function() {
        let mut parser = Parser::new("function foo(x: number): number { return x; }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_class() {
        let mut parser = Parser::new("class Foo {}");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_interface() {
        let mut parser = Parser::new("interface Bar {}");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_expression() {
        let mut parser = Parser::new("1 + 2 * 3;");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_arrow_function() {
        let mut parser = Parser::new("const f = (x: number) => x * 2;");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_for_loop() {
        let mut parser = Parser::new("for (let i = 0; i < 10; i++) {}");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_else() {
        let mut parser = Parser::new("if (x > 0) { return x; } else { return -x; }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_import() {
        let mut parser = Parser::new(r#"import { foo } from "bar";"#);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_options() {
        let opts = ParserOptions {
            jsx: true,
            decorators: true,
            module: true,
            typescript: true,
            strict: false,
        };
        let parser = Parser::with_options("let x;", opts);
        assert!(parser.options.jsx);
    }

    #[test]
    fn test_parser_state() {
        let state = ParserState::new()
            .with_async(true)
            .in_function_context();
        assert!(state.in_async);
        assert!(state.allow_await);
        assert!(state.in_function);
    }
}
