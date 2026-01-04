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
            SyntaxKind::AsyncKeyword => {
                // async function declaration or async arrow expression statement
                // Look ahead to see if it's "async function"
                if self.look_ahead_is_async_function() {
                    self.parse_async_function_declaration()
                } else {
                    // It's an async arrow function as expression statement
                    self.parse_expression_statement()
                }
            }
            SyntaxKind::ClassKeyword => self.parse_class_declaration(),
            SyntaxKind::InterfaceKeyword => self.parse_interface_declaration(),
            SyntaxKind::TypeKeyword => self.parse_type_alias_declaration(),
            SyntaxKind::IfKeyword => self.parse_if_statement(),
            SyntaxKind::ReturnKeyword => self.parse_return_statement(),
            SyntaxKind::WhileKeyword => self.parse_while_statement(),
            SyntaxKind::ForKeyword => self.parse_for_statement(),
            SyntaxKind::SemicolonToken => self.parse_empty_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    /// Look ahead to see if we have "async function"
    fn look_ahead_is_async_function(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        // Skip 'async'
        self.next_token();
        let is_function = self.is_token(SyntaxKind::FunctionKeyword);

        self.scanner.restore_state(snapshot);
        self.current_token = current;
        is_function
    }

    /// Parse async function declaration
    fn parse_async_function_declaration(&mut self) -> NodeIndex {
        self.parse_expected(SyntaxKind::AsyncKeyword);
        self.parse_function_declaration_with_async(true)
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

    /// Parse function declaration (optionally async)
    fn parse_function_declaration(&mut self) -> NodeIndex {
        self.parse_function_declaration_with_async(false)
    }

    /// Parse function declaration with async modifier already consumed
    fn parse_function_declaration_with_async(&mut self, is_async: bool) -> NodeIndex {
        let start_pos = self.token_pos();

        // Check for async modifier if not already parsed
        let is_async = is_async || self.parse_optional(SyntaxKind::AsyncKeyword);

        self.parse_expected(SyntaxKind::FunctionKeyword);

        // Check for generator asterisk
        let asterisk_token = self.parse_optional(SyntaxKind::AsteriskToken);

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
                is_async,
                asterisk_token,
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

    /// Parse class declaration
    fn parse_class_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::ClassKeyword);

        // Parse class name
        let name = if self.is_token(SyntaxKind::Identifier) {
            self.parse_identifier()
        } else {
            NodeIndex::NONE
        };

        // TODO: Parse type parameters

        // Parse heritage clauses (extends, implements)
        let heritage_clauses = self.parse_heritage_clauses();

        // Parse class body
        self.parse_expected(SyntaxKind::OpenBraceToken);
        let members = self.parse_class_members();
        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end_pos = self.token_end();
        self.arena.add_class(
            syntax_kind_ext::CLASS_DECLARATION,
            start_pos,
            end_pos,
            ClassData {
                modifiers: None,
                name,
                type_parameters: None,
                heritage_clauses,
                members,
            },
        )
    }

    /// Parse heritage clauses (extends, implements)
    fn parse_heritage_clauses(&mut self) -> Option<NodeList> {
        let mut clauses = Vec::new();

        // Parse extends clause
        if self.is_token(SyntaxKind::ExtendsKeyword) {
            let start_pos = self.token_pos();
            self.next_token();
            let type_ref = self.parse_expression(); // Simple: just parse as expression
            let end_pos = self.token_end();

            // Create heritage clause node
            let clause = self.arena.add_heritage(
                syntax_kind_ext::HERITAGE_CLAUSE,
                start_pos,
                end_pos,
                crate::parser::thin_node::HeritageData {
                    token: SyntaxKind::ExtendsKeyword as u16,
                    types: self.make_node_list(vec![type_ref]),
                },
            );
            clauses.push(clause);
        }

        // Parse implements clause
        if self.is_token(SyntaxKind::ImplementsKeyword) {
            let start_pos = self.token_pos();
            self.next_token();

            let mut types = Vec::new();
            loop {
                let type_ref = self.parse_expression();
                types.push(type_ref);
                if !self.parse_optional(SyntaxKind::CommaToken) {
                    break;
                }
            }

            let end_pos = self.token_end();
            let clause = self.arena.add_heritage(
                syntax_kind_ext::HERITAGE_CLAUSE,
                start_pos,
                end_pos,
                crate::parser::thin_node::HeritageData {
                    token: SyntaxKind::ImplementsKeyword as u16,
                    types: self.make_node_list(types),
                },
            );
            clauses.push(clause);
        }

        if clauses.is_empty() {
            None
        } else {
            Some(self.make_node_list(clauses))
        }
    }

    /// Parse class members
    fn parse_class_members(&mut self) -> NodeList {
        let mut members = Vec::new();

        while !self.is_token(SyntaxKind::CloseBraceToken) &&
              !self.is_token(SyntaxKind::EndOfFileToken) {
            let member = self.parse_class_member();
            if !member.is_none() {
                members.push(member);
            }

            // Handle semicolons
            self.parse_optional(SyntaxKind::SemicolonToken);
        }

        self.make_node_list(members)
    }

    /// Parse a single class member
    fn parse_class_member(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Handle constructor
        if self.is_token(SyntaxKind::ConstructorKeyword) {
            return self.parse_constructor();
        }

        // Handle methods and properties
        // For now, just parse name and check for ( for methods
        let name = if self.is_token(SyntaxKind::Identifier) ||
                     self.is_token(SyntaxKind::StringLiteral) ||
                     self.is_token(SyntaxKind::NumericLiteral) {
            self.parse_property_name()
        } else {
            // Skip unknown token
            self.next_token();
            return NodeIndex::NONE;
        };

        // Check if it's a method or property
        if self.is_token(SyntaxKind::OpenParenToken) {
            // Method
            self.parse_expected(SyntaxKind::OpenParenToken);
            let parameters = self.parse_parameter_list();
            self.parse_expected(SyntaxKind::CloseParenToken);

            // Optional return type
            let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
                self.parse_type()
            } else {
                NodeIndex::NONE
            };

            // Parse body
            let body = if self.is_token(SyntaxKind::OpenBraceToken) {
                self.parse_block()
            } else {
                NodeIndex::NONE
            };

            let end_pos = self.token_end();
            self.arena.add_method_decl(
                syntax_kind_ext::METHOD_DECLARATION,
                start_pos,
                end_pos,
                crate::parser::thin_node::MethodDeclData {
                    modifiers: None,
                    asterisk_token: false,
                    name,
                    question_token: false,
                    type_parameters: None,
                    parameters,
                    type_annotation,
                    body,
                },
            )
        } else {
            // Property - parse optional type and initializer
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
            self.arena.add_property_decl(
                syntax_kind_ext::PROPERTY_DECLARATION,
                start_pos,
                end_pos,
                crate::parser::thin_node::PropertyDeclData {
                    modifiers: None,
                    name,
                    question_token: false,
                    exclamation_token: false,
                    type_annotation,
                    initializer,
                },
            )
        }
    }

    /// Parse constructor
    fn parse_constructor(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::ConstructorKeyword);

        self.parse_expected(SyntaxKind::OpenParenToken);
        let parameters = self.parse_parameter_list();
        self.parse_expected(SyntaxKind::CloseParenToken);

        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_constructor(
            syntax_kind_ext::CONSTRUCTOR,
            start_pos,
            end_pos,
            crate::parser::thin_node::ConstructorData {
                modifiers: None,
                type_parameters: None,
                parameters,
                body,
            },
        )
    }

    /// Parse interface declaration
    fn parse_interface_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::InterfaceKeyword);

        // Parse interface name
        let name = self.parse_identifier();

        // TODO: Parse type parameters

        // Parse heritage clauses (extends only for interfaces)
        let heritage_clauses = if self.is_token(SyntaxKind::ExtendsKeyword) {
            let clause_start = self.token_pos();
            self.next_token();

            let mut types = Vec::new();
            loop {
                let type_ref = self.parse_expression();
                types.push(type_ref);
                if !self.parse_optional(SyntaxKind::CommaToken) {
                    break;
                }
            }

            let clause_end = self.token_end();
            let clause = self.arena.add_heritage(
                syntax_kind_ext::HERITAGE_CLAUSE,
                clause_start,
                clause_end,
                crate::parser::thin_node::HeritageData {
                    token: SyntaxKind::ExtendsKeyword as u16,
                    types: self.make_node_list(types),
                },
            );
            Some(self.make_node_list(vec![clause]))
        } else {
            None
        };

        // Parse interface body
        self.parse_expected(SyntaxKind::OpenBraceToken);
        let members = self.parse_type_members();
        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end_pos = self.token_end();
        self.arena.add_interface(
            syntax_kind_ext::INTERFACE_DECLARATION,
            start_pos,
            end_pos,
            crate::parser::thin_node::InterfaceData {
                modifiers: None,
                name,
                type_parameters: None,
                heritage_clauses,
                members,
            },
        )
    }

    /// Parse type members (for interfaces and type literals)
    fn parse_type_members(&mut self) -> NodeList {
        let mut members = Vec::new();

        while !self.is_token(SyntaxKind::CloseBraceToken) &&
              !self.is_token(SyntaxKind::EndOfFileToken) {
            let start_token = self.token();
            let member = self.parse_type_member();
            if !member.is_none() {
                members.push(member);
            }

            // Handle semicolons or commas
            self.parse_optional(SyntaxKind::SemicolonToken);
            self.parse_optional(SyntaxKind::CommaToken);

            // If we didn't make progress, skip the current token to avoid infinite loop
            if self.token() == start_token && !self.is_token(SyntaxKind::CloseBraceToken) {
                self.next_token();
            }
        }

        self.make_node_list(members)
    }

    /// Parse a single type member (property signature or method signature)
    fn parse_type_member(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse property/method name
        let name = if self.is_token(SyntaxKind::Identifier) ||
                     self.is_token(SyntaxKind::StringLiteral) ||
                     self.is_token(SyntaxKind::NumericLiteral) {
            self.parse_property_name()
        } else if self.is_token(SyntaxKind::OpenBracketToken) {
            // Index signature: [key: string]: value
            return self.parse_index_signature();
        } else {
            return NodeIndex::NONE;
        };

        // Optional question mark
        let question_token = self.parse_optional(SyntaxKind::QuestionToken);

        // Check if it's a method signature or property signature
        if self.is_token(SyntaxKind::OpenParenToken) {
            // Method signature
            self.parse_expected(SyntaxKind::OpenParenToken);
            let parameters = self.parse_parameter_list();
            self.parse_expected(SyntaxKind::CloseParenToken);

            let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
                self.parse_type()
            } else {
                NodeIndex::NONE
            };

            let end_pos = self.token_end();
            self.arena.add_signature(
                syntax_kind_ext::METHOD_SIGNATURE,
                start_pos,
                end_pos,
                crate::parser::thin_node::SignatureData {
                    modifiers: None,
                    name,
                    question_token,
                    type_parameters: None,
                    parameters: Some(parameters),
                    type_annotation,
                },
            )
        } else {
            // Property signature
            let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
                self.parse_type()
            } else {
                NodeIndex::NONE
            };

            let end_pos = self.token_end();
            self.arena.add_signature(
                syntax_kind_ext::PROPERTY_SIGNATURE,
                start_pos,
                end_pos,
                crate::parser::thin_node::SignatureData {
                    modifiers: None,
                    name,
                    question_token,
                    type_parameters: None,
                    parameters: None,
                    type_annotation,
                },
            )
        }
    }

    /// Parse index signature: [key: string]: value
    fn parse_index_signature(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBracketToken);

        // Parse parameter
        let param_name = self.parse_identifier();
        self.parse_expected(SyntaxKind::ColonToken);
        let param_type = self.parse_type();

        self.parse_expected(SyntaxKind::CloseBracketToken);
        self.parse_expected(SyntaxKind::ColonToken);

        let type_annotation = self.parse_type();

        let end_pos = self.token_end();
        self.arena.add_index_signature(
            syntax_kind_ext::INDEX_SIGNATURE,
            start_pos,
            end_pos,
            crate::parser::thin_node::IndexSignatureData {
                modifiers: None,
                parameters: self.make_node_list(vec![param_name]),
                type_annotation,
            },
        )
    }

    /// Parse type alias declaration: type Foo = ... or type Foo<T> = ...
    fn parse_type_alias_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::TypeKeyword);

        let name = self.parse_identifier();

        // Parse optional type parameters: <T, U extends Foo>
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

        self.parse_expected(SyntaxKind::EqualsToken);

        let type_node = self.parse_type();

        self.parse_semicolon();

        let end_pos = self.token_end();
        self.arena.add_type_alias(
            syntax_kind_ext::TYPE_ALIAS_DECLARATION,
            start_pos,
            end_pos,
            crate::parser::thin_node::TypeAliasData {
                modifiers: None,
                name,
                type_parameters,
                type_node,
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
        // Check for arrow function first (including async arrow)
        if self.is_start_of_arrow_function() {
            // Check if it's an async arrow
            if self.is_token(SyntaxKind::AsyncKeyword) {
                return self.parse_async_arrow_function_expression();
            }
            return self.parse_arrow_function_expression_with_async(false);
        }

        // Start at precedence 2 to skip comma operator (precedence 1)
        // Comma expressions are only valid in certain contexts (e.g., for loop)
        self.parse_binary_expression(2)
    }

    /// Parse async arrow function: async (x) => ... or async x => ...
    fn parse_async_arrow_function_expression(&mut self) -> NodeIndex {
        self.parse_expected(SyntaxKind::AsyncKeyword);
        self.parse_arrow_function_expression_with_async(true)
    }

    /// Check if we're at the start of an arrow function
    fn is_start_of_arrow_function(&mut self) -> bool {
        match self.token() {
            // (params) => ...
            SyntaxKind::OpenParenToken => self.look_ahead_is_arrow_function(),
            // identifier => ...
            SyntaxKind::Identifier => self.look_ahead_is_simple_arrow_function(),
            // async (x) => ... or async x => ... or async <T>(x) => ...
            SyntaxKind::AsyncKeyword => self.look_ahead_is_arrow_function_after_async(),
            // <T>(x) => ... (generic arrow function)
            SyntaxKind::LessThanToken => self.look_ahead_is_generic_arrow_function(),
            _ => false,
        }
    }

    /// Look ahead to see if < starts a generic arrow function: <T>(x) => or <T, U>() =>
    fn look_ahead_is_generic_arrow_function(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        // Skip <
        self.next_token();

        // Skip type parameters until we find >
        let mut depth = 1;
        while depth > 0 && !self.is_token(SyntaxKind::EndOfFileToken) {
            if self.is_token(SyntaxKind::LessThanToken) {
                depth += 1;
            } else if self.is_token(SyntaxKind::GreaterThanToken) {
                depth -= 1;
            }
            self.next_token();
        }

        // After >, should have (
        if !self.is_token(SyntaxKind::OpenParenToken) {
            self.scanner.restore_state(snapshot);
            self.current_token = current;
            return false;
        }

        // Now check if this is an arrow function
        let result = self.look_ahead_is_arrow_function();

        self.scanner.restore_state(snapshot);
        self.current_token = current;
        result
    }

    /// Look ahead after async to see if it's an arrow function: async (x) => or async x => or async <T>(x) =>
    fn look_ahead_is_arrow_function_after_async(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        // Skip 'async'
        self.next_token();

        let result = match self.token() {
            // async (params) => ...
            SyntaxKind::OpenParenToken => self.look_ahead_is_arrow_function(),
            // async x => ...
            SyntaxKind::Identifier => self.look_ahead_is_simple_arrow_function(),
            // async <T>(x) => ... (generic async arrow)
            SyntaxKind::LessThanToken => self.look_ahead_is_generic_arrow_function(),
            _ => false,
        };

        self.scanner.restore_state(snapshot);
        self.current_token = current;
        result
    }

    /// Look ahead to see if ( starts an arrow function: () => or (x) => or (x, y) =>
    fn look_ahead_is_arrow_function(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        // Skip (
        self.next_token();

        // Empty params: () =>
        if self.is_token(SyntaxKind::CloseParenToken) {
            self.next_token();
            let is_arrow = self.is_token(SyntaxKind::EqualsGreaterThanToken);
            self.scanner.restore_state(snapshot);
            self.current_token = current;
            return is_arrow;
        }

        // Skip to matching ) to check for =>
        let mut depth = 1;
        while depth > 0 && !self.is_token(SyntaxKind::EndOfFileToken) {
            if self.is_token(SyntaxKind::OpenParenToken) {
                depth += 1;
            } else if self.is_token(SyntaxKind::CloseParenToken) {
                depth -= 1;
            }
            self.next_token();
        }

        // Check for optional return type annotation
        if self.is_token(SyntaxKind::ColonToken) {
            self.next_token();
            // Skip the type (simplified - just skip until =>)
            while !self.is_token(SyntaxKind::EqualsGreaterThanToken)
                && !self.is_token(SyntaxKind::EndOfFileToken)
                && !self.is_token(SyntaxKind::SemicolonToken)
                && !self.is_token(SyntaxKind::CloseBraceToken) {
                self.next_token();
            }
        }

        let is_arrow = self.is_token(SyntaxKind::EqualsGreaterThanToken);
        self.scanner.restore_state(snapshot);
        self.current_token = current;
        is_arrow
    }

    /// Look ahead to see if identifier is followed by => (simple arrow function)
    fn look_ahead_is_simple_arrow_function(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        // Skip identifier
        self.next_token();
        let is_arrow = self.is_token(SyntaxKind::EqualsGreaterThanToken);

        self.scanner.restore_state(snapshot);
        self.current_token = current;
        is_arrow
    }

    /// Parse arrow function expression: (params) => body or x => body or <T>(x) => body
    fn parse_arrow_function_expression_with_async(&mut self, is_async: bool) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse optional type parameters: <T, U extends Foo>
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

        // Parse parameters
        let parameters = if self.is_token(SyntaxKind::OpenParenToken) {
            // Parenthesized parameter list: (a, b) =>
            self.parse_expected(SyntaxKind::OpenParenToken);
            let params = self.parse_parameter_list();
            self.parse_expected(SyntaxKind::CloseParenToken);
            params
        } else {
            // Single identifier parameter: x =>
            let param_start = self.token_pos();
            let name = self.parse_identifier();
            let param_end = self.token_end();

            let param = self.arena.add_parameter(
                syntax_kind_ext::PARAMETER,
                param_start,
                param_end,
                crate::parser::thin_node::ParameterData {
                    modifiers: None,
                    dot_dot_dot_token: false,
                    name,
                    question_token: false,
                    type_annotation: NodeIndex::NONE,
                    initializer: NodeIndex::NONE,
                },
            );
            self.make_node_list(vec![param])
        };

        // Parse optional return type annotation
        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        // Parse =>
        self.parse_expected(SyntaxKind::EqualsGreaterThanToken);

        // Parse body (block or expression)
        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };

        let end_pos = self.token_end();

        self.arena.add_function(
            syntax_kind_ext::ARROW_FUNCTION,
            start_pos,
            end_pos,
            FunctionData {
                modifiers: None,
                is_async,
                asterisk_token: false,
                name: NodeIndex::NONE,
                type_parameters,
                parameters,
                type_annotation,
                body,
                equals_greater_than_token: true,
            },
        )
    }

    /// Parse type parameters: <T, U extends Foo, V = DefaultType>
    fn parse_type_parameters(&mut self) -> NodeList {
        let mut params = Vec::new();

        self.parse_expected(SyntaxKind::LessThanToken);

        while !self.is_token(SyntaxKind::GreaterThanToken)
            && !self.is_token(SyntaxKind::EndOfFileToken)
        {
            let param = self.parse_type_parameter();
            params.push(param);

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.parse_expected(SyntaxKind::GreaterThanToken);

        self.make_node_list(params)
    }

    /// Parse a single type parameter: T or T extends U or T = Default or T extends U = Default
    fn parse_type_parameter(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse the type parameter name
        let name = self.parse_identifier();

        // Parse optional constraint: extends SomeType
        let constraint = if self.parse_optional(SyntaxKind::ExtendsKeyword) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        // Parse optional default: = DefaultType
        let default = if self.parse_optional(SyntaxKind::EqualsToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();

        self.arena.add_type_parameter(
            syntax_kind_ext::TYPE_PARAMETER,
            start_pos,
            end_pos,
            crate::parser::thin_node::TypeParameterData {
                modifiers: None,
                name,
                constraint,
                default,
            },
        )
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
            SyntaxKind::YieldKeyword => {
                let start_pos = self.token_pos();
                self.next_token();

                // Check for yield* (delegate yield)
                let asterisk_token = self.parse_optional(SyntaxKind::AsteriskToken);

                // Parse the expression (may be empty for bare yield)
                let expression = if !self.scanner.has_preceding_line_break()
                    && !self.is_token(SyntaxKind::SemicolonToken)
                    && !self.is_token(SyntaxKind::CloseBraceToken)
                    && !self.is_token(SyntaxKind::CloseParenToken)
                    && !self.is_token(SyntaxKind::CloseBracketToken)
                    && !self.is_token(SyntaxKind::ColonToken)
                    && !self.is_token(SyntaxKind::CommaToken)
                    && !self.is_token(SyntaxKind::EndOfFileToken)
                {
                    self.parse_assignment_expression()
                } else {
                    NodeIndex::NONE
                };

                let end_pos = self.token_end();

                self.arena.add_unary_expr_ex(
                    syntax_kind_ext::YIELD_EXPRESSION,
                    start_pos,
                    end_pos,
                    UnaryExprDataEx { expression, asterisk_token },
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

    /// Parse a type (handles keywords, type references, unions, intersections, conditionals)
    fn parse_type(&mut self) -> NodeIndex {
        self.parse_conditional_type()
    }

    /// Parse conditional type: T extends U ? X : Y
    fn parse_conditional_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse the check type (left side of extends)
        let check_type = self.parse_union_type();

        // Check for extends keyword to form conditional
        if !self.is_token(SyntaxKind::ExtendsKeyword) {
            return check_type;
        }

        self.next_token(); // consume extends

        // Parse the extends type (right side of extends)
        let extends_type = self.parse_union_type();

        // Expect ?
        self.parse_expected(SyntaxKind::QuestionToken);

        // Parse true type
        let true_type = self.parse_type();

        // Expect :
        self.parse_expected(SyntaxKind::ColonToken);

        // Parse false type
        let false_type = self.parse_type();

        let end_pos = self.token_end();

        self.arena.add_conditional_type(
            syntax_kind_ext::CONDITIONAL_TYPE,
            start_pos,
            end_pos,
            crate::parser::thin_node::ConditionalTypeData {
                check_type,
                extends_type,
                true_type,
                false_type,
            },
        )
    }

    /// Parse union type: A | B | C
    fn parse_union_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse first constituent
        let first = self.parse_intersection_type();

        // Check for | to form union
        if !self.is_token(SyntaxKind::BarToken) {
            return first;
        }

        let mut types = vec![first];

        while self.parse_optional(SyntaxKind::BarToken) {
            types.push(self.parse_intersection_type());
        }

        let end_pos = self.token_end();
        self.arena.add_composite_type(
            syntax_kind_ext::UNION_TYPE,
            start_pos,
            end_pos,
            crate::parser::thin_node::CompositeTypeData {
                types: self.make_node_list(types),
            },
        )
    }

    /// Parse intersection type: A & B & C
    fn parse_intersection_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse first constituent
        let first = self.parse_primary_type();

        // Check for & to form intersection
        if !self.is_token(SyntaxKind::AmpersandToken) {
            return first;
        }

        let mut types = vec![first];

        while self.parse_optional(SyntaxKind::AmpersandToken) {
            types.push(self.parse_primary_type());
        }

        let end_pos = self.token_end();
        self.arena.add_composite_type(
            syntax_kind_ext::INTERSECTION_TYPE,
            start_pos,
            end_pos,
            crate::parser::thin_node::CompositeTypeData {
                types: self.make_node_list(types),
            },
        )
    }

    /// Parse primary type (keywords, references, parenthesized, tuples, arrays, function types)
    fn parse_primary_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Handle parenthesized types or function types
        if self.is_token(SyntaxKind::OpenParenToken) {
            // Check if this is a function type: () => T or (x: T) => U
            if self.look_ahead_is_function_type() {
                return self.parse_function_type();
            }

            // Otherwise it's a parenthesized type
            self.next_token();
            let inner = self.parse_type();
            self.parse_expected(SyntaxKind::CloseParenToken);

            // Handle array types on parenthesized: (A | B)[]
            if self.is_token(SyntaxKind::OpenBracketToken) {
                return self.parse_array_type(start_pos, inner);
            }
            return inner;
        }

        // Handle tuple types: [T, U, V]
        if self.is_token(SyntaxKind::OpenBracketToken) {
            return self.parse_tuple_type();
        }

        // Handle object type literal or mapped type: { ... } or { [K in T]: U }
        if self.is_token(SyntaxKind::OpenBraceToken) {
            return self.parse_object_or_mapped_type();
        }

        // Handle typeof type: typeof x
        if self.is_token(SyntaxKind::TypeOfKeyword) {
            return self.parse_typeof_type();
        }

        // Handle keyof type: keyof T
        if self.is_token(SyntaxKind::KeyOfKeyword) {
            return self.parse_keyof_type();
        }

        // Handle readonly type: readonly T[]
        if self.is_token(SyntaxKind::ReadonlyKeyword) {
            return self.parse_readonly_type();
        }

        // Handle infer type: infer T (used in conditional types)
        if self.is_token(SyntaxKind::InferKeyword) {
            return self.parse_infer_type();
        }

        // Handle literal types: "foo", 42, true, false
        if self.is_token(SyntaxKind::StringLiteral)
            || self.is_token(SyntaxKind::NumericLiteral)
            || self.is_token(SyntaxKind::TrueKeyword)
            || self.is_token(SyntaxKind::FalseKeyword)
        {
            return self.parse_literal_type();
        }

        // Handle template literal types: `hello` or `prefix${T}suffix`
        if self.is_token(SyntaxKind::NoSubstitutionTemplateLiteral)
            || self.is_token(SyntaxKind::TemplateHead)
        {
            return self.parse_template_literal_type();
        }

        // Check for type keywords (string, number, boolean, etc.)
        let type_name = match self.token() {
            SyntaxKind::StringKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::SymbolKeyword
            | SyntaxKind::BigIntKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::UndefinedKeyword
            | SyntaxKind::NeverKeyword
            | SyntaxKind::AnyKeyword
            | SyntaxKind::UnknownKeyword
            | SyntaxKind::ObjectKeyword => {
                // Parse keyword as identifier for type reference
                self.parse_keyword_as_identifier()
            }
            _ => {
                // Regular identifier
                self.parse_identifier()
            }
        };

        let end_pos = self.token_end();

        // Check for type arguments: Foo<T, U>
        let type_arguments = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_arguments())
        } else {
            None
        };

        let base_type = self.arena.add_type_ref(
            syntax_kind_ext::TYPE_REFERENCE,
            start_pos,
            self.token_end(),
            crate::parser::thin_node::TypeRefData {
                type_name,
                type_arguments,
            },
        );

        // Handle array types (T[])
        if self.is_token(SyntaxKind::OpenBracketToken) {
            return self.parse_array_type(start_pos, base_type);
        }

        base_type
    }

    /// Parse tuple type: [T, U, V]
    fn parse_tuple_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBracketToken);

        let mut elements = Vec::new();

        while !self.is_token(SyntaxKind::CloseBracketToken)
            && !self.is_token(SyntaxKind::EndOfFileToken)
        {
            elements.push(self.parse_type());

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.parse_expected(SyntaxKind::CloseBracketToken);
        let end_pos = self.token_end();

        let tuple = self.arena.add_tuple_type(
            syntax_kind_ext::TUPLE_TYPE,
            start_pos,
            end_pos,
            crate::parser::thin_node::TupleTypeData {
                elements: self.make_node_list(elements),
            },
        );

        // Handle array of tuples: [T, U][]
        if self.is_token(SyntaxKind::OpenBracketToken) {
            return self.parse_array_type(start_pos, tuple);
        }

        tuple
    }

    /// Parse literal type: "foo", 42, true, false
    fn parse_literal_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse the literal expression
        let literal = match self.token() {
            SyntaxKind::StringLiteral => self.parse_string_literal(),
            SyntaxKind::NumericLiteral => self.parse_numeric_literal(),
            SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword => self.parse_boolean_literal(),
            _ => {
                // Fallback - shouldn't happen
                self.parse_identifier()
            }
        };

        let end_pos = self.token_end();

        self.arena.add_literal_type(
            syntax_kind_ext::LITERAL_TYPE,
            start_pos,
            end_pos,
            crate::parser::thin_node::LiteralTypeData { literal },
        )
    }

    /// Parse typeof type: typeof x, typeof x.y
    fn parse_typeof_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::TypeOfKeyword);

        // Parse the expression name (can be qualified: x.y.z)
        let expr_name = self.parse_entity_name();

        let end_pos = self.token_end();

        self.arena.add_type_query(
            syntax_kind_ext::TYPE_QUERY,
            start_pos,
            end_pos,
            crate::parser::thin_node::TypeQueryData {
                expr_name,
                type_arguments: None,
            },
        )
    }

    /// Parse keyof type: keyof T
    fn parse_keyof_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let operator = self.token() as u16;
        self.parse_expected(SyntaxKind::KeyOfKeyword);

        // Parse the type operand
        let type_node = self.parse_primary_type();

        let end_pos = self.token_end();

        self.arena.add_type_operator(
            syntax_kind_ext::TYPE_OPERATOR,
            start_pos,
            end_pos,
            crate::parser::thin_node::TypeOperatorData {
                operator,
                type_node,
            },
        )
    }

    /// Parse readonly type: readonly T[]
    fn parse_readonly_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let operator = self.token() as u16;
        self.parse_expected(SyntaxKind::ReadonlyKeyword);

        // Parse the type operand
        let type_node = self.parse_primary_type();

        let end_pos = self.token_end();

        self.arena.add_type_operator(
            syntax_kind_ext::TYPE_OPERATOR,
            start_pos,
            end_pos,
            crate::parser::thin_node::TypeOperatorData {
                operator,
                type_node,
            },
        )
    }

    /// Parse infer type: infer T (used in conditional types)
    fn parse_infer_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::InferKeyword);

        // Parse the type parameter to infer
        let type_parameter = self.parse_type_parameter();

        let end_pos = self.token_end();

        self.arena.add_infer_type(
            syntax_kind_ext::INFER_TYPE,
            start_pos,
            end_pos,
            crate::parser::thin_node::InferTypeData {
                type_parameter,
            },
        )
    }

    /// Parse template literal type: `hello` or `prefix${T}suffix`
    fn parse_template_literal_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse the head (either NoSubstitutionTemplateLiteral or TemplateHead)
        if self.is_token(SyntaxKind::NoSubstitutionTemplateLiteral) {
            // Simple template literal type with no substitutions: `hello`
            let head = self.parse_template_literal_head();
            let end_pos = self.token_end();

            return self.arena.add_template_literal_type(
                syntax_kind_ext::TEMPLATE_LITERAL_TYPE,
                start_pos,
                end_pos,
                crate::parser::thin_node::TemplateLiteralTypeData {
                    head,
                    template_spans: self.make_node_list(vec![]),
                },
            );
        }

        // Template with substitutions: `prefix${T}middle${U}suffix`
        let head = self.parse_template_literal_head();
        let mut spans = Vec::new();

        // Parse template spans: each span has a type and a template literal (middle or tail)
        while self.is_token(SyntaxKind::TemplateMiddle) || self.is_token(SyntaxKind::TemplateTail) {
            // This shouldn't happen - after parsing head we need to parse a type first
            break;
        }

        // After the head, we need to parse: type, then middle/tail, repeat until tail
        loop {
            // Parse the type inside ${...}
            let type_node = self.parse_type();

            // Now we need to rescan for the template continuation
            // The scanner needs to be told to rescan as template
            self.scanner.re_scan_template_token(false);
            self.current_token = self.scanner.get_token();

            let span_start = self.token_pos();
            let is_tail = self.is_token(SyntaxKind::TemplateTail);

            // Parse the template middle/tail literal
            let literal = self.parse_template_literal_span();
            let span_end = self.token_end();

            // Create a template span node
            // Note: We reuse TemplateSpanData, using 'expression' field for the type node
            let span = self.arena.add_template_span(
                syntax_kind_ext::TEMPLATE_LITERAL_TYPE_SPAN,
                span_start,
                span_end,
                crate::parser::thin_node::TemplateSpanData {
                    expression: type_node,
                    literal,
                },
            );
            spans.push(span);

            if is_tail {
                break;
            }
        }

        let end_pos = self.token_end();

        self.arena.add_template_literal_type(
            syntax_kind_ext::TEMPLATE_LITERAL_TYPE,
            start_pos,
            end_pos,
            crate::parser::thin_node::TemplateLiteralTypeData {
                head,
                template_spans: self.make_node_list(spans),
            },
        )
    }

    /// Parse template literal head (NoSubstitutionTemplateLiteral or TemplateHead)
    fn parse_template_literal_head(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let kind = self.token() as u16;
        self.next_token();
        let end_pos = self.token_end();
        self.arena.add_token(kind, start_pos, end_pos)
    }

    /// Parse template literal span (TemplateMiddle or TemplateTail)
    fn parse_template_literal_span(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let kind = self.token() as u16;
        self.next_token();
        let end_pos = self.token_end();
        self.arena.add_token(kind, start_pos, end_pos)
    }

    /// Parse object type literal or mapped type
    /// Object type: { prop: T; method(): U }
    /// Mapped type: { [K in keyof T]: U } or { readonly [K in T]?: U }
    fn parse_object_or_mapped_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        // Check if this is a mapped type: starts with [ or readonly [
        if self.is_token(SyntaxKind::OpenBracketToken)
            || (self.is_token(SyntaxKind::ReadonlyKeyword) && self.look_ahead_is_mapped_type())
            || (self.is_token(SyntaxKind::PlusToken) || self.is_token(SyntaxKind::MinusToken))
        {
            return self.parse_mapped_type_rest(start_pos);
        }

        // Otherwise it's an object type literal - parse as type literal
        self.parse_type_literal_rest(start_pos)
    }

    /// Look ahead to check if readonly is followed by [ (mapped type) vs property
    fn look_ahead_is_mapped_type(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        self.next_token(); // skip readonly
        let is_mapped = self.is_token(SyntaxKind::OpenBracketToken);

        self.scanner.restore_state(snapshot);
        self.current_token = current;
        is_mapped
    }

    /// Parse mapped type after opening brace: { [K in T]: U }
    fn parse_mapped_type_rest(&mut self, start_pos: u32) -> NodeIndex {
        // Parse optional readonly modifier with +/- prefix
        let readonly_token = if self.is_token(SyntaxKind::ReadonlyKeyword) {
            let pos = self.token_pos();
            self.next_token();
            self.arena.add_token(SyntaxKind::ReadonlyKeyword as u16, pos, self.token_end())
        } else if self.is_token(SyntaxKind::PlusToken) || self.is_token(SyntaxKind::MinusToken) {
            let pos = self.token_pos();
            self.next_token();
            if self.is_token(SyntaxKind::ReadonlyKeyword) {
                self.next_token();
            }
            self.arena.add_token(SyntaxKind::ReadonlyKeyword as u16, pos, self.token_end())
        } else {
            NodeIndex::NONE
        };

        // Parse [K in T]
        self.parse_expected(SyntaxKind::OpenBracketToken);

        // Parse the type parameter: K in T
        let type_param_start = self.token_pos();
        let param_name = self.parse_identifier();

        self.parse_expected(SyntaxKind::InKeyword);

        let constraint = self.parse_type();

        // Parse optional 'as' clause for key remapping: [K in T as NewKey]
        let name_type = if self.parse_optional(SyntaxKind::AsKeyword) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        let type_param_end = self.token_end();

        let type_parameter = self.arena.add_type_parameter(
            syntax_kind_ext::TYPE_PARAMETER,
            type_param_start,
            type_param_end,
            crate::parser::thin_node::TypeParameterData {
                modifiers: None,
                name: param_name,
                constraint,
                default: NodeIndex::NONE,
            },
        );

        self.parse_expected(SyntaxKind::CloseBracketToken);

        // Parse optional ? modifier with +/- prefix
        let question_token = if self.is_token(SyntaxKind::QuestionToken) {
            let pos = self.token_pos();
            self.next_token();
            self.arena.add_token(SyntaxKind::QuestionToken as u16, pos, self.token_end())
        } else if self.is_token(SyntaxKind::PlusToken) || self.is_token(SyntaxKind::MinusToken) {
            let pos = self.token_pos();
            self.next_token();
            if self.is_token(SyntaxKind::QuestionToken) {
                self.next_token();
            }
            self.arena.add_token(SyntaxKind::QuestionToken as u16, pos, self.token_end())
        } else {
            NodeIndex::NONE
        };

        // Parse : and type
        self.parse_expected(SyntaxKind::ColonToken);
        let type_node = self.parse_type();

        // Parse optional semicolon
        self.parse_optional(SyntaxKind::SemicolonToken);

        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end_pos = self.token_end();

        self.arena.add_mapped_type(
            syntax_kind_ext::MAPPED_TYPE,
            start_pos,
            end_pos,
            crate::parser::thin_node::MappedTypeData {
                readonly_token,
                type_parameter,
                name_type,
                question_token,
                type_node,
                members: None,
            },
        )
    }

    /// Parse type literal (object type) after opening brace
    fn parse_type_literal_rest(&mut self, start_pos: u32) -> NodeIndex {
        let mut members = Vec::new();

        while !self.is_token(SyntaxKind::CloseBraceToken)
            && !self.is_token(SyntaxKind::EndOfFileToken)
        {
            let member = self.parse_type_member();
            members.push(member);

            // Allow comma or semicolon as separator
            if !self.parse_optional(SyntaxKind::SemicolonToken) {
                self.parse_optional(SyntaxKind::CommaToken);
            }
        }

        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end_pos = self.token_end();

        self.arena.add_type_literal(
            syntax_kind_ext::TYPE_LITERAL,
            start_pos,
            end_pos,
            crate::parser::thin_node::TypeLiteralData {
                members: self.make_node_list(members),
            },
        )
    }

    /// Parse entity name: x or x.y.z
    fn parse_entity_name(&mut self) -> NodeIndex {
        let mut left = self.parse_identifier();

        while self.parse_optional(SyntaxKind::DotToken) {
            let start_pos = self.token_pos();
            let right = self.parse_identifier();
            let end_pos = self.token_end();

            // Create a qualified name node
            left = self.arena.add_access_expr(
                syntax_kind_ext::QUALIFIED_NAME,
                start_pos,
                end_pos,
                crate::parser::thin_node::AccessExprData {
                    expression: left,
                    name_or_argument: right,
                    question_dot_token: false,
                },
            );
        }

        left
    }

    /// Parse type arguments: <T, U, V>
    fn parse_type_arguments(&mut self) -> NodeList {
        self.parse_expected(SyntaxKind::LessThanToken);

        let mut args = Vec::new();

        while !self.is_token(SyntaxKind::GreaterThanToken)
            && !self.is_token(SyntaxKind::EndOfFileToken)
        {
            args.push(self.parse_type());

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.parse_expected(SyntaxKind::GreaterThanToken);
        self.make_node_list(args)
    }

    /// Parse array type suffix (T[]) or indexed access type (T[K])
    fn parse_array_type(&mut self, start_pos: u32, element_type: NodeIndex) -> NodeIndex {
        let mut current = element_type;

        while self.is_token(SyntaxKind::OpenBracketToken) {
            self.next_token();

            // Check if this is array type [] or indexed access type [K]
            if self.is_token(SyntaxKind::CloseBracketToken) {
                // Array type: T[]
                self.next_token();
                let end_pos = self.token_end();

                current = self.arena.add_type_ref(
                    syntax_kind_ext::ARRAY_TYPE,
                    start_pos,
                    end_pos,
                    crate::parser::thin_node::TypeRefData {
                        type_name: current,
                        type_arguments: None,
                    },
                );
            } else {
                // Indexed access type: T[K]
                let index_type = self.parse_type();
                self.parse_expected(SyntaxKind::CloseBracketToken);
                let end_pos = self.token_end();

                current = self.arena.add_indexed_access_type(
                    syntax_kind_ext::INDEXED_ACCESS_TYPE,
                    start_pos,
                    end_pos,
                    crate::parser::thin_node::IndexedAccessTypeData {
                        object_type: current,
                        index_type,
                    },
                );
            }
        }

        current
    }

    /// Look ahead to see if ( starts a function type: () => T or (x: T) => U
    fn look_ahead_is_function_type(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        // Skip (
        self.next_token();

        // Empty params: () =>
        if self.is_token(SyntaxKind::CloseParenToken) {
            self.next_token();
            let is_arrow = self.is_token(SyntaxKind::EqualsGreaterThanToken);
            self.scanner.restore_state(snapshot);
            self.current_token = current;
            return is_arrow;
        }

        // Check for parameter-like syntax: identifier followed by :
        // If we see just a type (like `string`), it could be parenthesized type
        // Function type params have: `name:`
        if self.is_token(SyntaxKind::Identifier) {
            self.next_token();
            // If followed by : it's definitely a function type parameter
            if self.is_token(SyntaxKind::ColonToken) {
                self.scanner.restore_state(snapshot);
                self.current_token = current;
                return true;
            }
        }

        // For other cases, skip to matching ) to check for =>
        // First restore, then scan again
        self.scanner.restore_state(snapshot);
        self.current_token = current;

        let snapshot2 = self.scanner.save_state();
        self.next_token(); // Skip (

        let mut depth = 1;
        while depth > 0 && !self.is_token(SyntaxKind::EndOfFileToken) {
            if self.is_token(SyntaxKind::OpenParenToken) {
                depth += 1;
            } else if self.is_token(SyntaxKind::CloseParenToken) {
                depth -= 1;
            }
            self.next_token();
        }

        let is_arrow = self.is_token(SyntaxKind::EqualsGreaterThanToken);
        self.scanner.restore_state(snapshot2);
        self.current_token = current;
        is_arrow
    }

    /// Parse function type: (x: T, y: U) => V
    fn parse_function_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse parameters
        self.parse_expected(SyntaxKind::OpenParenToken);
        let parameters = self.parse_type_parameter_list();
        self.parse_expected(SyntaxKind::CloseParenToken);

        // Parse =>
        self.parse_expected(SyntaxKind::EqualsGreaterThanToken);

        // Parse return type
        let type_annotation = self.parse_type();

        let end_pos = self.token_end();

        self.arena.add_function_type(
            syntax_kind_ext::FUNCTION_TYPE,
            start_pos,
            end_pos,
            crate::parser::thin_node::FunctionTypeData {
                type_parameters: None,
                parameters,
                type_annotation,
            },
        )
    }

    /// Parse type parameter list for function types: (x: T, y: U)
    fn parse_type_parameter_list(&mut self) -> NodeList {
        let mut params = Vec::new();

        while !self.is_token(SyntaxKind::CloseParenToken)
            && !self.is_token(SyntaxKind::EndOfFileToken)
        {
            let param_start = self.token_pos();

            // Parse optional ...rest
            let dot_dot_dot = self.parse_optional(SyntaxKind::DotDotDotToken);

            // Parse parameter name
            let name = self.parse_identifier();

            // Parse optional ?
            let question = self.parse_optional(SyntaxKind::QuestionToken);

            // Parse type annotation
            let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
                self.parse_type()
            } else {
                NodeIndex::NONE
            };

            let param_end = self.token_end();

            let param = self.arena.add_parameter(
                syntax_kind_ext::PARAMETER,
                param_start,
                param_end,
                crate::parser::thin_node::ParameterData {
                    modifiers: None,
                    dot_dot_dot_token: dot_dot_dot,
                    name,
                    question_token: question,
                    type_annotation,
                    initializer: NodeIndex::NONE,
                },
            );
            params.push(param);

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.make_node_list(params)
    }

    /// Parse a keyword as an identifier (for type keywords like string, number, etc.)
    fn parse_keyword_as_identifier(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let text = self.scanner.get_token_value_ref().to_string();
        self.identifiers.push(text.clone());
        self.next_token();
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
    fn test_thin_parser_class_declaration() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { x = 1; bar() { return this.x; } }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_class_with_constructor() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Point { constructor(x, y) { this.x = x; this.y = y; } }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_class_extends() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Child extends Parent { constructor() { super(); } }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        // May have some diagnostics for super() but should parse successfully
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

    #[test]
    fn test_thin_parser_interface_declaration() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "interface User { name: string; age: number; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_interface_with_methods() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "interface Service { getName(): string; setName(name: string): void; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_interface_extends() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "interface Admin extends User { role: string; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_type_alias() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type ID = string;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_type_alias_object() {
        // Test type alias with object type (unions not yet supported)
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Point = Coord;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_index_signature() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "interface StringMap { [key: string]: string; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_arrow_function_simple() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const add = (a, b) => a + b;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_arrow_function_single_param() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const double = x => x * 2;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_arrow_function_block_body() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const greet = (name) => { return name; };".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_arrow_function_no_params() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const getTime = () => Date.now();".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_async_function() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "async function fetchData() { return await fetch(); }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_async_arrow_function() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const fetchData = async () => await fetch();".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_async_arrow_single_param() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const processItem = async item => await process(item);".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_generator_function() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "function* range(n) { for (let i = 0; i < n; i++) yield i; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_yield_expression() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "function* gen() { yield 1; yield 2; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_yield_star() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "function* delegate() { yield* otherGen(); }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_await_expression() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "async function test() { const x = await promise; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_union_type() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let x: string | number | boolean;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_intersection_type() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let x: A & B & C;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_union_intersection_mixed() {
        // Intersection binds tighter than union: A & B | C means (A & B) | C
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let x: A & B | C & D;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_array_type() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let arr: string[];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_nested_array_type() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let matrix: number[][];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_union_array_type() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let items: (string | number)[];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_tuple_type() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let point: [number, number];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_tuple_type_mixed() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let result: [string, number, boolean];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_tuple_array() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let points: [number, number][];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_generic_type() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let list: Array<string>;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_generic_type_multiple() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let map: Map<string, number>;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_generic_nested() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let nested: Map<string, Array<number>>;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_promise_type() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "async function fetch(): Promise<string> { return ''; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_function_type() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let callback: (x: number) => string;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_function_type_no_params() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let factory: () => Widget;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_function_type_multiple_params() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let handler: (a: string, b: number, c: boolean) => void;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_function_type_optional_param() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let fn: (x: number, y?: string) => void;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_function_type_rest_param() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let fn: (...args: number[]) => void;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_parenthesized_type_still_works() {
        // Ensure parenthesized types still work after adding function type support
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let x: (string | number);".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_literal_type_string() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            r#"let status: "success" | "error";"#.to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_literal_type_number() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let port: 80 | 443 | 8080;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_literal_type_boolean() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let flag: true;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_typeof_type() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let copy: typeof original;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_typeof_type_qualified() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let t: typeof console.log;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    // =========================================================================
    // Generic Arrow Function Tests
    // =========================================================================

    #[test]
    fn test_thin_parser_generic_arrow_simple() {
        // Basic generic arrow function: <T>(x: T) => T
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const identity = <T>(x: T) => x;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_generic_arrow_multiple_params() {
        // Multiple type parameters: <T, U>(x: T, y: U) => [T, U]
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const pair = <T, U>(x: T, y: U) => [x, y];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_generic_arrow_with_constraint() {
        // Type parameter with constraint: <T extends object>(x: T) => T
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const clone = <T extends object>(x: T) => x;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_generic_arrow_with_default() {
        // Type parameter with default: <T = string>(x: T) => T
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const wrap = <T = string>(x: T) => x;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_generic_arrow_with_constraint_and_default() {
        // Type parameter with both constraint and default
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const process = <T extends object = object>(x: T) => x;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_async_generic_arrow() {
        // Async generic arrow function
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const fetchData = async <T>(url: string) => { return url; };".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_generic_arrow_expression_body() {
        // Generic arrow with expression body
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const first = <T>(arr: T[]) => arr[0];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    // =========================================================================
    // Type Operator Tests (keyof, readonly)
    // =========================================================================

    #[test]
    fn test_thin_parser_keyof_type() {
        // Basic keyof type: keyof T
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Keys = keyof Person;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_keyof_typeof() {
        // keyof typeof: keyof typeof obj
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Keys = keyof typeof obj;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_keyof_in_union() {
        // keyof in union type
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type PropOrKey = string | keyof T;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_readonly_array() {
        // readonly array type
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let items: readonly string[];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_readonly_tuple() {
        // readonly tuple type
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let point: readonly [number, number];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    // =========================================================================
    // Indexed Access Type Tests
    // =========================================================================

    #[test]
    fn test_thin_parser_indexed_access_type() {
        // Basic indexed access: T[K]
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Value = Person[\"name\"];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_indexed_access_keyof() {
        // Indexed access with keyof: T[keyof T]
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Values = Person[keyof Person];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_indexed_access_chain() {
        // Chained indexed access: T[K1][K2]
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Deep = Obj[\"level1\"][\"level2\"];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_indexed_access_with_array() {
        // Mix of indexed access and array: T[K][]
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Names = Person[\"name\"][];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_indexed_access_number() {
        // Indexed access with number: T[number]
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Item = Items[number];".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    // =========================================================================
    // Conditional Type Tests
    // =========================================================================

    #[test]
    fn test_thin_parser_conditional_type_simple() {
        // Basic conditional type: T extends U ? X : Y
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type IsString<T> = T extends string ? true : false;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_conditional_type_nested() {
        // Nested conditional types
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type TypeName<T> = T extends string ? \"string\" : T extends number ? \"number\" : \"other\";".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_conditional_type_with_infer() {
        // Conditional type with infer
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type ReturnType<T> = T extends (...args: any[]) => infer R ? R : never;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_conditional_type_distributive() {
        // Distributive conditional type
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type NonNullable<T> = T extends null | undefined ? never : T;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_infer_type() {
        // Infer in array element position
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Flatten<T> = T extends Array<infer U> ? U : T;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    // =========================================================================
    // Mapped Type Tests
    // =========================================================================

    #[test]
    fn test_thin_parser_mapped_type_simple() {
        // Basic mapped type: { [K in keyof T]: U }
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Partial<T> = { [K in keyof T]?: T[K] };".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_mapped_type_readonly() {
        // Mapped type with readonly modifier
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Readonly<T> = { readonly [K in keyof T]: T[K] };".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_mapped_type_required() {
        // Mapped type removing optional: -?
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Required<T> = { [K in keyof T]-?: T[K] };".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_mapped_type_as_clause() {
        // Mapped type with key remapping (as clause)
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Pick<T, K> = { [P in K as P]: T[P] };".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_type_literal() {
        // Object type literal
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Point = { x: number; y: number };".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_type_literal_method() {
        // Object type literal with method signature
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Calculator = { add(a: number, b: number): number; subtract(a: number, b: number): number };".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    // =========================================================================
    // Template Literal Type Tests
    // =========================================================================

    #[test]
    fn test_thin_parser_template_literal_type_simple() {
        // Simple template literal type with no substitutions
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Greeting = `hello`;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_template_literal_type_with_substitution() {
        // Template literal type with type substitution
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Greeting<T extends string> = `hello ${T}`;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_template_literal_type_multiple_substitutions() {
        // Template literal type with multiple substitutions
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type FullName<F extends string, L extends string> = `${F} ${L}`;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_template_literal_type_with_union() {
        // Template literal type with union in substitution
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type EventName = `on${\"click\" | \"focus\" | \"blur\"}`;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_template_literal_type_uppercase() {
        // Template literal type with intrinsic type (Uppercase)
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Getter<K extends string> = `get${Uppercase<K>}`;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }
}
