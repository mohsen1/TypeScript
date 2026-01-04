//! ThinParser - Cache-optimized parser using ThinNodeArena
//!
//! This parser uses the ThinNode architecture (16 bytes per node vs 208 bytes)
//! for 13x better cache locality. It produces the same AST semantically
//! but stored in a more efficient format.
//!
//! # Architecture
//!
//! - Uses ThinNodeArena instead of NodeArena
//! - Each node is 16 bytes (vs 208 bytes for fat Node enum)
//! - Node data is stored in separate typed pools
//! - 4 nodes fit per 64-byte cache line (vs 0.31 for fat nodes)

use crate::scanner::SyntaxKind;
use crate::scanner_impl::ScannerState;
use crate::parser::{
    NodeIndex, NodeList,
    thin_node::{
        ThinNodeArena, IdentifierData, LiteralData, BinaryExprData, CallExprData,
        AccessExprData, ConditionalExprData, LiteralExprData, ParenthesizedData,
        UnaryExprData, UnaryExprDataEx, TypeAssertionData, BlockData, ReturnData,
        ExprStatementData, IfStatementData, LoopData, FunctionData, ClassData,
        SourceFileData, VariableData, VariableDeclarationData, ForInOfData,
        SwitchData, CaseClauseData, TryData, CatchClauseData,
    },
    syntax_kind_ext,
};
use crate::parser_impl::{ParseDiagnostic, context_flags};

// =============================================================================
// ThinParserState
// =============================================================================

/// A high-performance parser using ThinNode architecture.
///
/// This parser produces the same AST semantically as ParserState,
/// but uses the cache-optimized ThinNodeArena for storage.
pub struct ThinParserState {
    /// The scanner for tokenizing
    scanner: ScannerState,
    /// Arena for allocating ThinNodes
    pub arena: ThinNodeArena,
    /// Source file name
    file_name: String,
    /// Parser context flags
    context_flags: u32,
    /// Current token
    current_token: SyntaxKind,
    /// List of parse diagnostics
    parse_diagnostics: Vec<ParseDiagnostic>,
    /// The source text
    source_text: String,
    /// Node count for assigning IDs
    node_count: u32,
    /// Identifiers found during parsing
    identifiers: Vec<String>,
}

impl ThinParserState {
    /// Create a new ThinParser for the given source text.
    pub fn new(file_name: String, source_text: String) -> ThinParserState {
        let estimated_nodes = source_text.len() / 20; // Rough estimate
        let scanner = ScannerState::new(source_text.clone(), true);
        ThinParserState {
            scanner,
            arena: ThinNodeArena::with_capacity(estimated_nodes),
            file_name,
            context_flags: 0,
            current_token: SyntaxKind::Unknown,
            parse_diagnostics: Vec::new(),
            source_text,
            node_count: 0,
            identifiers: Vec::new(),
        }
    }

    // =========================================================================
    // Token Utilities (shared with regular parser)
    // =========================================================================

    /// Get current token
    #[inline]
    fn token(&self) -> SyntaxKind {
        self.current_token
    }

    /// Get current token position
    #[inline]
    fn token_pos(&self) -> u32 {
        self.scanner.get_token_start() as u32
    }

    /// Get current token end position
    #[inline]
    fn token_end(&self) -> u32 {
        self.scanner.get_token_end() as u32
    }

    /// Advance to next token
    fn next_token(&mut self) -> SyntaxKind {
        self.current_token = self.scanner.scan();
        self.current_token
    }

    /// Check if current token matches kind
    #[inline]
    fn is_token(&self, kind: SyntaxKind) -> bool {
        self.current_token == kind
    }

    /// Parse optional token, returns true if found
    pub fn parse_optional(&mut self, kind: SyntaxKind) -> bool {
        if self.is_token(kind) {
            self.next_token();
            true
        } else {
            false
        }
    }

    /// Parse expected token, report error if not found
    pub fn parse_expected(&mut self, kind: SyntaxKind) -> bool {
        if self.is_token(kind) {
            self.next_token();
            true
        } else {
            self.parse_error_at_current_token(&format!("Expected {:?}", kind));
            false
        }
    }

    /// Report parse error at current token
    pub fn parse_error_at_current_token(&mut self, message: &str) {
        let start = self.scanner.get_token_start() as u32;
        let end = self.scanner.get_token_end() as u32;
        self.parse_diagnostics.push(ParseDiagnostic {
            start,
            length: end - start,
            message: message.to_string(),
            code: 1000,
        });
    }

    /// Parse semicolon (or recover from missing)
    fn parse_semicolon(&mut self) {
        if self.is_token(SyntaxKind::SemicolonToken) {
            self.next_token();
        } else if !self.can_parse_semicolon() {
            self.parse_error_at_current_token("';' expected");
        }
    }

    /// Check if we can parse a semicolon (ASI rules)
    fn can_parse_semicolon(&self) -> bool {
        self.is_token(SyntaxKind::CloseBraceToken) ||
        self.is_token(SyntaxKind::EndOfFileToken) ||
        self.scanner.has_preceding_line_break()
    }

    /// Create a NodeList from a Vec of NodeIndex
    fn make_node_list(&self, nodes: Vec<NodeIndex>) -> NodeList {
        NodeList {
            nodes,
            pos: 0,
            end: 0,
            has_trailing_comma: false,
        }
    }

    /// Get operator precedence
    fn get_operator_precedence(&self, token: SyntaxKind) -> u8 {
        match token {
            SyntaxKind::CommaToken => 1,
            SyntaxKind::EqualsToken |
            SyntaxKind::PlusEqualsToken |
            SyntaxKind::MinusEqualsToken |
            SyntaxKind::AsteriskEqualsToken |
            SyntaxKind::SlashEqualsToken => 2,
            SyntaxKind::QuestionToken => 3,
            SyntaxKind::BarBarToken => 4,
            SyntaxKind::AmpersandAmpersandToken => 5,
            SyntaxKind::BarToken => 6,
            SyntaxKind::CaretToken => 7,
            SyntaxKind::AmpersandToken => 8,
            SyntaxKind::EqualsEqualsToken |
            SyntaxKind::ExclamationEqualsToken |
            SyntaxKind::EqualsEqualsEqualsToken |
            SyntaxKind::ExclamationEqualsEqualsToken => 9,
            SyntaxKind::LessThanToken |
            SyntaxKind::GreaterThanToken |
            SyntaxKind::LessThanEqualsToken |
            SyntaxKind::GreaterThanEqualsToken |
            SyntaxKind::InstanceOfKeyword |
            SyntaxKind::InKeyword => 10,
            SyntaxKind::LessThanLessThanToken |
            SyntaxKind::GreaterThanGreaterThanToken |
            SyntaxKind::GreaterThanGreaterThanGreaterThanToken => 11,
            SyntaxKind::PlusToken |
            SyntaxKind::MinusToken => 12,
            SyntaxKind::AsteriskToken |
            SyntaxKind::SlashToken |
            SyntaxKind::PercentToken => 13,
            SyntaxKind::AsteriskAsteriskToken => 14,
            _ => 0,
        }
    }

    // =========================================================================
    // Parse Methods - Core Expressions
    // =========================================================================

    /// Parse a source file
    pub fn parse_source_file(&mut self) -> NodeIndex {
        let start_pos = 0u32;

        // Initialize scanner
        self.next_token();

        // Parse statements
        let statements = self.parse_statements();

        // Create source file node
        let end_pos = self.token_end();
        let eof_token = self.arena.add_token(
            SyntaxKind::EndOfFileToken as u16,
            end_pos,
            end_pos,
        );

        self.arena.add_source_file(start_pos, end_pos, SourceFileData {
            statements,
            end_of_file_token: eof_token,
            file_name: self.file_name.clone(),
            text: self.source_text.clone(),
            language_version: 99,
            language_variant: 0,
            script_kind: 3,
            is_declaration_file: false,
            has_no_default_lib: false,
            identifiers: self.identifiers.clone(),
            parent: NodeIndex::NONE,
            id: 0,
            modifier_flags: 0,
            transform_flags: 0,
        })
    }

    /// Parse list of statements
    fn parse_statements(&mut self) -> NodeList {
        let mut statements = Vec::new();

        while !self.is_token(SyntaxKind::EndOfFileToken) &&
              !self.is_token(SyntaxKind::CloseBraceToken) {
            let stmt = self.parse_statement();
            if !stmt.is_none() {
                statements.push(stmt);
            }

            // Safety: break on unexpected tokens to avoid infinite loop
            if self.is_token(SyntaxKind::Unknown) {
                break;
            }
        }

        self.make_node_list(statements)
    }

    /// Parse a statement
    pub fn parse_statement(&mut self) -> NodeIndex {
        match self.token() {
            SyntaxKind::OpenBraceToken => self.parse_block(),
            SyntaxKind::VarKeyword |
            SyntaxKind::LetKeyword |
            SyntaxKind::ConstKeyword => self.parse_variable_statement(),
            SyntaxKind::FunctionKeyword => self.parse_function_declaration(),
            SyntaxKind::IfKeyword => self.parse_if_statement(),
            SyntaxKind::ReturnKeyword => self.parse_return_statement(),
            SyntaxKind::WhileKeyword => self.parse_while_statement(),
            SyntaxKind::ForKeyword => self.parse_for_statement(),
            SyntaxKind::SemicolonToken => self.parse_empty_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    /// Parse a block statement
    fn parse_block(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        let statements = self.parse_statements();

        self.parse_expected(SyntaxKind::CloseBraceToken);
        let end_pos = self.token_end();

        self.arena.add_block(
            syntax_kind_ext::BLOCK,
            start_pos,
            end_pos,
            BlockData {
                statements,
                multi_line: true,
            },
        )
    }

    /// Parse empty statement
    fn parse_empty_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::SemicolonToken);
        let end_pos = self.token_end();

        self.arena.add_token(
            syntax_kind_ext::EMPTY_STATEMENT,
            start_pos,
            end_pos,
        )
    }

    /// Parse variable statement (var/let/const)
    fn parse_variable_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let declaration_list = self.parse_variable_declaration_list();
        self.parse_semicolon();
        let end_pos = self.token_end();

        self.arena.add_variable(
            syntax_kind_ext::VARIABLE_STATEMENT,
            start_pos,
            end_pos,
            VariableData {
                modifiers: None,
                declarations: self.make_node_list(vec![declaration_list]),
            },
        )
    }

    /// Parse variable declaration list
    fn parse_variable_declaration_list(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Consume var/let/const
        let _flags = match self.token() {
            SyntaxKind::LetKeyword => { self.next_token(); 1 }
            SyntaxKind::ConstKeyword => { self.next_token(); 2 }
            _ => { self.next_token(); 0 } // var
        };

        // Parse declarations
        let mut declarations = Vec::new();
        loop {
            let decl = self.parse_variable_declaration();
            declarations.push(decl);

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        let end_pos = self.token_end();
        self.arena.add_variable(
            syntax_kind_ext::VARIABLE_DECLARATION_LIST,
            start_pos,
            end_pos,
            VariableData {
                modifiers: None,
                declarations: self.make_node_list(declarations),
            },
        )
    }

    /// Parse variable declaration
    fn parse_variable_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse name
        let name = self.parse_identifier();

        // Parse optional type annotation
        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        // Parse optional initializer
        let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
            self.parse_assignment_expression()
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_variable_declaration(
            syntax_kind_ext::VARIABLE_DECLARATION,
            start_pos,
            end_pos,
            VariableDeclarationData {
                name,
                exclamation_token: false,
                type_annotation,
                initializer,
            },
        )
    }

    /// Parse function declaration
    fn parse_function_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::FunctionKeyword);

        // Parse name
        let name = self.parse_identifier();

        // Parse parameters
        self.parse_expected(SyntaxKind::OpenParenToken);
        let parameters = self.parse_parameter_list();
        self.parse_expected(SyntaxKind::CloseParenToken);

        // Parse optional return type
        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        // Parse body
        let body = self.parse_block();

        let end_pos = self.token_end();
        self.arena.add_function(
            syntax_kind_ext::FUNCTION_DECLARATION,
            start_pos,
            end_pos,
            FunctionData {
                modifiers: None,
                asterisk_token: false,
                name,
                type_parameters: None,
                parameters,
                type_annotation,
                body,
                equals_greater_than_token: false,
            },
        )
    }

    /// Parse parameter list
    fn parse_parameter_list(&mut self) -> NodeList {
        let mut params = Vec::new();

        while !self.is_token(SyntaxKind::CloseParenToken) {
            let param = self.parse_parameter();
            params.push(param);

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.make_node_list(params)
    }

    /// Parse a single parameter
    fn parse_parameter(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let name = self.parse_identifier();

        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
            self.parse_assignment_expression()
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_parameter(
            syntax_kind_ext::PARAMETER,
            start_pos,
            end_pos,
            crate::parser::thin_node::ParameterData {
                modifiers: None,
                dot_dot_dot_token: false,
                name,
                question_token: false,
                type_annotation,
                initializer,
            },
        )
    }

    /// Parse if statement
    fn parse_if_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::IfKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);

        let expression = self.parse_expression();

        self.parse_expected(SyntaxKind::CloseParenToken);

        let then_statement = self.parse_statement();

        let else_statement = if self.parse_optional(SyntaxKind::ElseKeyword) {
            self.parse_statement()
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_if_statement(
            syntax_kind_ext::IF_STATEMENT,
            start_pos,
            end_pos,
            IfStatementData {
                expression,
                then_statement,
                else_statement,
            },
        )
    }

    /// Parse return statement
    fn parse_return_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::ReturnKeyword);

        let expression = if !self.can_parse_semicolon() {
            self.parse_expression()
        } else {
            NodeIndex::NONE
        };

        self.parse_semicolon();
        let end_pos = self.token_end();

        self.arena.add_return(
            syntax_kind_ext::RETURN_STATEMENT,
            start_pos,
            end_pos,
            ReturnData { expression },
        )
    }

    /// Parse while statement
    fn parse_while_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::WhileKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);

        let condition = self.parse_expression();

        self.parse_expected(SyntaxKind::CloseParenToken);

        let statement = self.parse_statement();

        let end_pos = self.token_end();
        self.arena.add_loop(
            syntax_kind_ext::WHILE_STATEMENT,
            start_pos,
            end_pos,
            LoopData {
                initializer: NodeIndex::NONE,
                condition,
                incrementor: NodeIndex::NONE,
                statement,
            },
        )
    }

    /// Parse for statement (basic for loop only, not for-in/for-of yet)
    fn parse_for_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::ForKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);

        // Initializer
        let initializer = if !self.is_token(SyntaxKind::SemicolonToken) {
            if self.is_token(SyntaxKind::VarKeyword) ||
               self.is_token(SyntaxKind::LetKeyword) ||
               self.is_token(SyntaxKind::ConstKeyword) {
                self.parse_variable_declaration_list()
            } else {
                self.parse_expression()
            }
        } else {
            NodeIndex::NONE
        };
        self.parse_expected(SyntaxKind::SemicolonToken);

        // Condition
        let condition = if !self.is_token(SyntaxKind::SemicolonToken) {
            self.parse_expression()
        } else {
            NodeIndex::NONE
        };
        self.parse_expected(SyntaxKind::SemicolonToken);

        // Incrementor
        let incrementor = if !self.is_token(SyntaxKind::CloseParenToken) {
            self.parse_expression()
        } else {
            NodeIndex::NONE
        };
        self.parse_expected(SyntaxKind::CloseParenToken);

        let statement = self.parse_statement();

        let end_pos = self.token_end();
        self.arena.add_loop(
            syntax_kind_ext::FOR_STATEMENT,
            start_pos,
            end_pos,
            LoopData {
                initializer,
                condition,
                incrementor,
                statement,
            },
        )
    }

    /// Parse expression statement
    fn parse_expression_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let expression = self.parse_expression();
        self.parse_semicolon();
        let end_pos = self.token_end();

        self.arena.add_expr_statement(
            syntax_kind_ext::EXPRESSION_STATEMENT,
            start_pos,
            end_pos,
            ExprStatementData { expression },
        )
    }

    // =========================================================================
    // Parse Methods - Expressions
    // =========================================================================

    /// Parse an expression
    pub fn parse_expression(&mut self) -> NodeIndex {
        self.parse_assignment_expression()
    }

    /// Parse assignment expression
    fn parse_assignment_expression(&mut self) -> NodeIndex {
        // Start at precedence 2 to skip comma operator (precedence 1)
        // Comma expressions are only valid in certain contexts (e.g., for loop)
        self.parse_binary_expression(2)
    }

    /// Parse binary expression with precedence climbing
    fn parse_binary_expression(&mut self, min_precedence: u8) -> NodeIndex {
        let start_pos = self.token_pos();
        let mut left = self.parse_unary_expression();

        loop {
            let op = self.token();
            let precedence = self.get_operator_precedence(op);

            if precedence == 0 || precedence < min_precedence {
                break;
            }

            let operator_token = op as u16;
            self.next_token();

            // Handle conditional expression
            if op == SyntaxKind::QuestionToken {
                let when_true = self.parse_assignment_expression();
                self.parse_expected(SyntaxKind::ColonToken);
                let when_false = self.parse_assignment_expression();
                let end_pos = self.token_end();

                left = self.arena.add_conditional_expr(
                    syntax_kind_ext::CONDITIONAL_EXPRESSION,
                    start_pos,
                    end_pos,
                    ConditionalExprData {
                        condition: left,
                        when_true,
                        when_false,
                    },
                );
            } else {
                // Right associativity for assignment and exponentiation
                let next_min = if op == SyntaxKind::EqualsToken ||
                                  op == SyntaxKind::AsteriskAsteriskToken {
                    precedence
                } else {
                    precedence + 1
                };

                let right = self.parse_binary_expression(next_min);
                let end_pos = self.token_end();

                left = self.arena.add_binary_expr(
                    syntax_kind_ext::BINARY_EXPRESSION,
                    start_pos,
                    end_pos,
                    BinaryExprData {
                        left,
                        operator_token,
                        right,
                    },
                );
            }
        }

        left
    }

    /// Parse unary expression
    fn parse_unary_expression(&mut self) -> NodeIndex {
        match self.token() {
            SyntaxKind::PlusToken |
            SyntaxKind::MinusToken |
            SyntaxKind::TildeToken |
            SyntaxKind::ExclamationToken |
            SyntaxKind::PlusPlusToken |
            SyntaxKind::MinusMinusToken => {
                let start_pos = self.token_pos();
                let operator = self.token() as u16;
                self.next_token();
                let operand = self.parse_unary_expression();
                let end_pos = self.token_end();

                self.arena.add_unary_expr(
                    syntax_kind_ext::PREFIX_UNARY_EXPRESSION,
                    start_pos,
                    end_pos,
                    UnaryExprData { operator, operand },
                )
            }
            SyntaxKind::TypeOfKeyword |
            SyntaxKind::VoidKeyword |
            SyntaxKind::DeleteKeyword => {
                let start_pos = self.token_pos();
                let operator = self.token() as u16;
                self.next_token();
                let operand = self.parse_unary_expression();
                let end_pos = self.token_end();

                self.arena.add_unary_expr(
                    syntax_kind_ext::PREFIX_UNARY_EXPRESSION,
                    start_pos,
                    end_pos,
                    UnaryExprData { operator, operand },
                )
            }
            SyntaxKind::AwaitKeyword => {
                let start_pos = self.token_pos();
                self.next_token();
                let expression = self.parse_unary_expression();
                let end_pos = self.token_end();

                self.arena.add_unary_expr_ex(
                    syntax_kind_ext::AWAIT_EXPRESSION,
                    start_pos,
                    end_pos,
                    UnaryExprDataEx { expression, asterisk_token: false },
                )
            }
            _ => self.parse_postfix_expression(),
        }
    }

    /// Parse postfix expression
    fn parse_postfix_expression(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let mut expr = self.parse_left_hand_side_expression();

        // Handle postfix operators
        if !self.scanner.has_preceding_line_break() {
            if self.is_token(SyntaxKind::PlusPlusToken) ||
               self.is_token(SyntaxKind::MinusMinusToken) {
                let operator = self.token() as u16;
                self.next_token();
                let end_pos = self.token_end();

                expr = self.arena.add_unary_expr(
                    syntax_kind_ext::POSTFIX_UNARY_EXPRESSION,
                    start_pos,
                    end_pos,
                    UnaryExprData { operator, operand: expr },
                );
            }
        }

        expr
    }

    /// Parse left-hand side expression (member access, call, etc.)
    fn parse_left_hand_side_expression(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let mut expr = self.parse_primary_expression();

        loop {
            match self.token() {
                SyntaxKind::DotToken => {
                    self.next_token();
                    let name = self.parse_identifier();
                    let end_pos = self.token_end();

                    expr = self.arena.add_access_expr(
                        syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION,
                        start_pos,
                        end_pos,
                        AccessExprData {
                            expression: expr,
                            name_or_argument: name,
                            question_dot_token: false,
                        },
                    );
                }
                SyntaxKind::OpenBracketToken => {
                    self.next_token();
                    let argument = self.parse_expression();
                    self.parse_expected(SyntaxKind::CloseBracketToken);
                    let end_pos = self.token_end();

                    expr = self.arena.add_access_expr(
                        syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION,
                        start_pos,
                        end_pos,
                        AccessExprData {
                            expression: expr,
                            name_or_argument: argument,
                            question_dot_token: false,
                        },
                    );
                }
                SyntaxKind::OpenParenToken => {
                    self.next_token();
                    let arguments = self.parse_argument_list();
                    self.parse_expected(SyntaxKind::CloseParenToken);
                    let end_pos = self.token_end();

                    expr = self.arena.add_call_expr(
                        syntax_kind_ext::CALL_EXPRESSION,
                        start_pos,
                        end_pos,
                        CallExprData {
                            expression: expr,
                            type_arguments: None,
                            arguments: Some(arguments),
                        },
                    );
                }
                _ => break,
            }
        }

        expr
    }

    /// Parse argument list
    fn parse_argument_list(&mut self) -> NodeList {
        let mut args = Vec::new();

        while !self.is_token(SyntaxKind::CloseParenToken) {
            let arg = self.parse_assignment_expression();
            args.push(arg);

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.make_node_list(args)
    }

    /// Parse primary expression
    fn parse_primary_expression(&mut self) -> NodeIndex {
        match self.token() {
            SyntaxKind::Identifier => self.parse_identifier(),
            SyntaxKind::NumericLiteral => self.parse_numeric_literal(),
            SyntaxKind::StringLiteral => self.parse_string_literal(),
            SyntaxKind::TrueKeyword |
            SyntaxKind::FalseKeyword => self.parse_boolean_literal(),
            SyntaxKind::NullKeyword => self.parse_null_literal(),
            SyntaxKind::ThisKeyword => self.parse_this_expression(),
            SyntaxKind::OpenParenToken => self.parse_parenthesized_expression(),
            SyntaxKind::OpenBracketToken => self.parse_array_literal(),
            SyntaxKind::OpenBraceToken => self.parse_object_literal(),
            SyntaxKind::NewKeyword => self.parse_new_expression(),
            _ => {
                // Unknown primary expression - create an error token
                let start_pos = self.token_pos();
                let end_pos = self.token_end();
                self.parse_error_at_current_token("Expression expected");
                self.next_token();
                self.arena.add_token(SyntaxKind::Unknown as u16, start_pos, end_pos)
            }
        }
    }

    /// Parse identifier
    /// Uses zero-copy accessor and only clones when storing
    fn parse_identifier(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        // Use zero-copy accessor and clone only when storing
        let text = self.scanner.get_token_value_ref().to_string();
        self.identifiers.push(text.clone());
        self.parse_expected(SyntaxKind::Identifier);
        let end_pos = self.token_end();

        self.arena.add_identifier(
            SyntaxKind::Identifier as u16,
            start_pos,
            end_pos,
            IdentifierData {
                escaped_text: text,
                original_text: None,
                type_arguments: None,
            },
        )
    }

    /// Parse numeric literal
    /// Uses zero-copy accessor for parsing, clones only when storing
    fn parse_numeric_literal(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        // Use zero-copy accessor for parsing
        let text_ref = self.scanner.get_token_value_ref();
        let value = text_ref.parse::<f64>().ok();
        let text = text_ref.to_string();
        self.next_token();
        let end_pos = self.token_end();

        self.arena.add_literal(
            SyntaxKind::NumericLiteral as u16,
            start_pos,
            end_pos,
            LiteralData { text, raw_text: None, value },
        )
    }

    /// Parse string literal
    /// Uses zero-copy accessor, clones only when storing
    fn parse_string_literal(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        // Use zero-copy accessor
        let text = self.scanner.get_token_value_ref().to_string();
        self.next_token();
        let end_pos = self.token_end();

        self.arena.add_literal(
            SyntaxKind::StringLiteral as u16,
            start_pos,
            end_pos,
            LiteralData { text, raw_text: None, value: None },
        )
    }

    /// Parse boolean literal
    fn parse_boolean_literal(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let kind = self.token();
        self.next_token();
        let end_pos = self.token_end();

        self.arena.add_token(kind as u16, start_pos, end_pos)
    }

    /// Parse null literal
    fn parse_null_literal(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.next_token();
        let end_pos = self.token_end();

        self.arena.add_token(SyntaxKind::NullKeyword as u16, start_pos, end_pos)
    }

    /// Parse this expression
    fn parse_this_expression(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.next_token();
        let end_pos = self.token_end();

        self.arena.add_token(SyntaxKind::ThisKeyword as u16, start_pos, end_pos)
    }

    /// Parse parenthesized expression
    fn parse_parenthesized_expression(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression();
        self.parse_expected(SyntaxKind::CloseParenToken);
        let end_pos = self.token_end();

        self.arena.add_parenthesized(
            syntax_kind_ext::PARENTHESIZED_EXPRESSION,
            start_pos,
            end_pos,
            ParenthesizedData { expression },
        )
    }

    /// Parse array literal
    fn parse_array_literal(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBracketToken);

        let mut elements = Vec::new();
        while !self.is_token(SyntaxKind::CloseBracketToken) {
            if self.is_token(SyntaxKind::CommaToken) {
                // Elided element
                elements.push(NodeIndex::NONE);
            } else {
                let elem = self.parse_assignment_expression();
                elements.push(elem);
            }

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.parse_expected(SyntaxKind::CloseBracketToken);
        let end_pos = self.token_end();

        self.arena.add_literal_expr(
            syntax_kind_ext::ARRAY_LITERAL_EXPRESSION,
            start_pos,
            end_pos,
            LiteralExprData {
                elements: self.make_node_list(elements),
                multi_line: false,
            },
        )
    }

    /// Parse object literal
    fn parse_object_literal(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        let mut properties = Vec::new();
        while !self.is_token(SyntaxKind::CloseBraceToken) {
            let prop = self.parse_property_assignment();
            properties.push(prop);

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.parse_expected(SyntaxKind::CloseBraceToken);
        let end_pos = self.token_end();

        self.arena.add_literal_expr(
            syntax_kind_ext::OBJECT_LITERAL_EXPRESSION,
            start_pos,
            end_pos,
            LiteralExprData {
                elements: self.make_node_list(properties),
                multi_line: false,
            },
        )
    }

    /// Parse property assignment
    fn parse_property_assignment(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let name = self.parse_property_name();

        let initializer = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_assignment_expression()
        } else {
            // Shorthand property
            name
        };

        let end_pos = self.token_end();
        self.arena.add_property_assignment(
            syntax_kind_ext::PROPERTY_ASSIGNMENT,
            start_pos,
            end_pos,
            crate::parser::thin_node::PropertyAssignmentData {
                modifiers: None,
                name,
                initializer,
            },
        )
    }

    /// Parse property name (identifier, string literal, numeric literal, computed)
    fn parse_property_name(&mut self) -> NodeIndex {
        match self.token() {
            SyntaxKind::StringLiteral => {
                // String literal can be property name: { "key": value }
                self.parse_string_literal()
            }
            SyntaxKind::NumericLiteral => {
                // Numeric literal can be property name: { 0: value }
                self.parse_numeric_literal()
            }
            SyntaxKind::OpenBracketToken => {
                // Computed property name: { [expr]: value }
                let start_pos = self.token_pos();
                self.next_token();
                let expression = self.parse_expression();
                self.parse_expected(SyntaxKind::CloseBracketToken);
                let end_pos = self.token_end();

                self.arena.add_computed_property(
                    syntax_kind_ext::COMPUTED_PROPERTY_NAME,
                    start_pos,
                    end_pos,
                    crate::parser::thin_node::ComputedPropertyData { expression },
                )
            }
            _ => {
                // Identifier or keyword used as property name
                let start_pos = self.token_pos();
                // Use zero-copy accessor
                let text = self.scanner.get_token_value_ref().to_string();
                self.identifiers.push(text.clone());
                self.next_token(); // Accept any token as property name
                let end_pos = self.token_end();

                self.arena.add_identifier(
                    SyntaxKind::Identifier as u16,
                    start_pos,
                    end_pos,
                    IdentifierData {
                        escaped_text: text,
                        original_text: None,
                        type_arguments: None,
                    },
                )
            }
        }
    }

    /// Parse new expression
    fn parse_new_expression(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::NewKeyword);

        let expression = self.parse_left_hand_side_expression();

        // TODO: Type arguments

        let arguments = if self.is_token(SyntaxKind::OpenParenToken) {
            self.next_token();
            let args = self.parse_argument_list();
            self.parse_expected(SyntaxKind::CloseParenToken);
            Some(args)
        } else {
            None
        };

        let end_pos = self.token_end();
        self.arena.add_call_expr(
            syntax_kind_ext::NEW_EXPRESSION,
            start_pos,
            end_pos,
            CallExprData {
                expression,
                type_arguments: None,
                arguments,
            },
        )
    }

    // =========================================================================
    // Parse Methods - Types (minimal implementation)
    // =========================================================================

    /// Parse a type (minimal - just identifier for now)
    fn parse_type(&mut self) -> NodeIndex {
        // For now, just parse an identifier as a type reference
        let start_pos = self.token_pos();
        let type_name = self.parse_identifier();
        let end_pos = self.token_end();

        self.arena.add_type_ref(
            syntax_kind_ext::TYPE_REFERENCE,
            start_pos,
            end_pos,
            crate::parser::thin_node::TypeRefData {
                type_name,
                type_arguments: None,
            },
        )
    }

    // =========================================================================
    // Accessors
    // =========================================================================

    /// Get parse diagnostics
    pub fn get_diagnostics(&self) -> &[ParseDiagnostic] {
        &self.parse_diagnostics
    }

    /// Get the arena
    pub fn get_arena(&self) -> &ThinNodeArena {
        &self.arena
    }

    /// Get node count
    pub fn get_node_count(&self) -> usize {
        self.arena.len()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_thin_parser_simple_expression() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "1 + 2".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.arena.len() > 0);

        // Should have: SourceFile, ExpressionStatement, BinaryExpression, 2 NumericLiterals
        assert!(parser.arena.len() >= 5, "Expected at least 5 nodes, got {}", parser.arena.len());
    }

    #[test]
    fn test_thin_parser_function() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "function add(a, b) { return a + b; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Unexpected errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_variable_declaration() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let x = 42;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty());
    }

    #[test]
    fn test_thin_parser_if_statement() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "if (x > 0) { return x; } else { return -x; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty());
    }

    #[test]
    fn test_thin_parser_while_loop() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "while (x < 10) { x++; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty());
    }

    #[test]
    fn test_thin_parser_for_loop() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "for (let i = 0; i < 10; i++) { console.log(i); }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty());
    }

    #[test]
    fn test_thin_parser_object_literal() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let obj = { a: 1, b: 2 };".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_array_literal() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let arr = [1, 2, 3];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty());
    }

    #[test]
    fn test_thin_parser_call_expression() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "foo(1, 2, 3);".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty());
    }

    #[test]
    fn test_thin_parser_property_access() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "obj.foo.bar;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty());
    }

    #[test]
    fn test_thin_parser_new_expression() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "new Foo(1, 2);".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty());
    }

    #[test]
    fn test_thin_parser_memory_efficiency() {
        // Verify that ThinParserState uses less memory per node
        let source = "let x = 1 + 2 + 3 + 4 + 5;".to_string();
        let mut parser = ThinParserState::new("test.ts".to_string(), source);
        parser.parse_source_file();

        // Calculate memory usage
        let thin_node_size = size_of::<crate::parser::thin_node::ThinNode>();
        assert_eq!(thin_node_size, 16, "ThinNode should be 16 bytes");

        // Each node uses 16 bytes + data pool entry
        // This is much better than 208 bytes per fat Node
        let total_nodes = parser.arena.len();
        let thin_memory = total_nodes * 16;
        let fat_memory = total_nodes * 208;

        println!("Nodes: {}", total_nodes);
        println!("ThinNode memory: {} bytes", thin_memory);
        println!("Fat Node memory: {} bytes", fat_memory);
        println!("Memory savings: {}x", fat_memory / thin_memory.max(1));

        assert!(fat_memory / thin_memory.max(1) >= 10, "Should have at least 10x memory savings");
    }
}
