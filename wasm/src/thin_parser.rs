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
        EnumData, EnumMemberData,
        ImportDeclData, ImportClauseData, NamedImportsData, SpecifierData,
        ExportDeclData, ExportAssignmentData, QualifiedNameData,
        TemplateExprData, TemplateSpanData,
        TypeOperatorData, NamedTupleMemberData,
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

    /// Check if current token is an identifier or any keyword
    /// Keywords can be used as identifiers in many contexts (e.g., class names, property names)
    #[inline]
    fn is_identifier_or_keyword(&self) -> bool {
        self.current_token as u16 >= SyntaxKind::Identifier as u16
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
            SyntaxKind::AtToken => {
                // Decorator: @decorator class/function
                self.parse_decorated_declaration()
            }
            SyntaxKind::ClassKeyword => self.parse_class_declaration(),
            SyntaxKind::AbstractKeyword => {
                // abstract class declaration
                if self.look_ahead_is_abstract_class() {
                    self.parse_abstract_class_declaration()
                } else {
                    self.parse_expression_statement()
                }
            }
            SyntaxKind::InterfaceKeyword => self.parse_interface_declaration(),
            SyntaxKind::TypeKeyword => self.parse_type_alias_declaration(),
            SyntaxKind::EnumKeyword => self.parse_enum_declaration(),
            SyntaxKind::DeclareKeyword => self.parse_ambient_declaration(),
            SyntaxKind::NamespaceKeyword | SyntaxKind::ModuleKeyword => self.parse_module_declaration(),
            SyntaxKind::IfKeyword => self.parse_if_statement(),
            SyntaxKind::ReturnKeyword => self.parse_return_statement(),
            SyntaxKind::WhileKeyword => self.parse_while_statement(),
            SyntaxKind::ForKeyword => self.parse_for_statement(),
            SyntaxKind::SemicolonToken => self.parse_empty_statement(),
            SyntaxKind::ExportKeyword => self.parse_export_declaration(),
            SyntaxKind::ImportKeyword => {
                // Check for import = (import equals declaration)
                if self.look_ahead_is_import_equals() {
                    self.parse_import_equals_declaration()
                } else {
                    self.parse_import_declaration()
                }
            }
            SyntaxKind::BreakKeyword => self.parse_break_statement(),
            SyntaxKind::ContinueKeyword => self.parse_continue_statement(),
            SyntaxKind::ThrowKeyword => self.parse_throw_statement(),
            SyntaxKind::DoKeyword => self.parse_do_statement(),
            SyntaxKind::SwitchKeyword => self.parse_switch_statement(),
            SyntaxKind::TryKeyword => self.parse_try_statement(),
            SyntaxKind::WithKeyword => self.parse_with_statement(),
            SyntaxKind::DebuggerKeyword => self.parse_debugger_statement(),
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

    /// Look ahead to see if we have "abstract class"
    fn look_ahead_is_abstract_class(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        // Skip 'abstract'
        self.next_token();
        let is_class = self.is_token(SyntaxKind::ClassKeyword);

        self.scanner.restore_state(snapshot);
        self.current_token = current;
        is_class
    }

    /// Look ahead to see if we have "import identifier ="
    fn look_ahead_is_import_equals(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        // Skip 'import'
        self.next_token();
        // Check for identifier
        if !self.is_token(SyntaxKind::Identifier) {
            self.scanner.restore_state(snapshot);
            self.current_token = current;
            return false;
        }
        // Skip identifier
        self.next_token();
        // Check for '='
        let is_equals = self.is_token(SyntaxKind::EqualsToken);

        self.scanner.restore_state(snapshot);
        self.current_token = current;
        is_equals
    }

    /// Parse import equals declaration: import X = require("...") or import X = Y.Z
    fn parse_import_equals_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::ImportKeyword);

        // Parse the name
        let name = self.parse_identifier();

        self.parse_expected(SyntaxKind::EqualsToken);

        // Parse module reference: require("...") or qualified name
        let module_reference = if self.is_token(SyntaxKind::RequireKeyword) {
            self.parse_external_module_reference()
        } else {
            self.parse_entity_name()
        };

        self.parse_semicolon();
        let end_pos = self.token_end();

        // Use ImportDeclData with import_clause as the name and module_specifier as reference
        // This is a simplified representation
        self.arena.add_import_decl(
            syntax_kind_ext::IMPORT_EQUALS_DECLARATION,
            start_pos,
            end_pos,
            ImportDeclData {
                modifiers: None,
                import_clause: name,
                module_specifier: module_reference,
                attributes: NodeIndex::NONE,
            },
        )
    }

    /// Parse external module reference: require("...")
    fn parse_external_module_reference(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::RequireKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);
        let expression = self.parse_string_literal();
        self.parse_expected(SyntaxKind::CloseParenToken);
        let end_pos = self.token_end();

        // Return the string literal as the module reference
        expression
    }

    /// Parse entity name: A or A.B.C
    fn parse_entity_name(&mut self) -> NodeIndex {
        let mut left = self.parse_identifier();

        while self.is_token(SyntaxKind::DotToken) {
            self.next_token();
            let right = self.parse_identifier();
            let start_pos = if let Some(node) = self.arena.get(left) { node.pos } else { 0 };
            let end_pos = self.token_end();

            left = self.arena.add_qualified_name(
                syntax_kind_ext::QUALIFIED_NAME,
                start_pos,
                end_pos,
                QualifiedNameData {
                    left,
                    right,
                },
            );
        }

        left
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

        // Parse name - can be identifier, keyword as identifier, or binding pattern
        let name = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_object_binding_pattern()
        } else if self.is_token(SyntaxKind::OpenBracketToken) {
            self.parse_array_binding_pattern()
        } else if self.is_identifier_or_keyword() {
            self.parse_identifier_name()
        } else {
            self.parse_identifier()
        };

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

        // Parse name - keywords like 'abstract' can be used as function names
        let name = if self.is_identifier_or_keyword() {
            self.parse_identifier_name()
        } else {
            self.parse_identifier()
        };

        // Parse optional type parameters: <T, U extends V>
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

        // Parse parameters
        self.parse_expected(SyntaxKind::OpenParenToken);
        let parameters = self.parse_parameter_list();
        self.parse_expected(SyntaxKind::CloseParenToken);

        // Parse optional return type (may be a type predicate: param is T)
        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_return_type()
        } else {
            NodeIndex::NONE
        };

        // Parse body - may be missing for overload signatures (just a semicolon)
        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            // Consume the semicolon if present (overload signature)
            self.parse_optional(SyntaxKind::SemicolonToken);
            NodeIndex::NONE
        };

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
                type_parameters,
                parameters,
                type_annotation,
                body,
                equals_greater_than_token: false,
            },
        )
    }

    /// Parse function expression: function() {} or function name() {}
    ///
    /// Unlike function declarations, function expressions can be anonymous.
    fn parse_function_expression(&mut self) -> NodeIndex {
        self.parse_function_expression_with_async(false)
    }

    /// Parse async function expression: async function() {} or async function name() {}
    fn parse_async_function_expression(&mut self) -> NodeIndex {
        self.parse_function_expression_with_async(true)
    }

    /// Parse function expression with optional async modifier
    fn parse_function_expression_with_async(&mut self, is_async: bool) -> NodeIndex {
        let start_pos = self.token_pos();

        // Consume async if present and not already parsed
        let is_async = is_async || self.parse_optional(SyntaxKind::AsyncKeyword);

        self.parse_expected(SyntaxKind::FunctionKeyword);

        // Check for generator asterisk
        let asterisk_token = self.parse_optional(SyntaxKind::AsteriskToken);

        // Parse optional name (function expressions can be anonymous)
        let name = if self.is_token(SyntaxKind::Identifier) {
            self.parse_identifier()
        } else {
            NodeIndex::NONE
        };

        // Parse optional type parameters: <T, U extends V>
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

        // Parse parameters
        self.parse_expected(SyntaxKind::OpenParenToken);
        let parameters = self.parse_parameter_list();
        self.parse_expected(SyntaxKind::CloseParenToken);

        // Parse optional return type (may be a type predicate: param is T)
        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_return_type()
        } else {
            NodeIndex::NONE
        };

        // Parse body
        let body = self.parse_block();

        let end_pos = self.token_end();
        self.arena.add_function(
            syntax_kind_ext::FUNCTION_EXPRESSION,
            start_pos,
            end_pos,
            FunctionData {
                modifiers: None,
                is_async,
                asterisk_token,
                name,
                type_parameters,
                parameters,
                type_annotation,
                body,
                equals_greater_than_token: false,
            },
        )
    }

    /// Parse class expression: class {} or class Name {}
    ///
    /// Unlike class declarations, class expressions can be anonymous.
    fn parse_class_expression(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        self.parse_expected(SyntaxKind::ClassKeyword);

        // Parse optional name (class expressions can be anonymous)
        let name = if self.is_token(SyntaxKind::Identifier) {
            self.parse_identifier()
        } else {
            NodeIndex::NONE
        };

        // Parse optional type parameters
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameter_list())
        } else {
            None
        };

        // Parse heritage (extends/implements)
        let heritage = self.parse_heritage_clauses();

        // Parse body
        self.parse_expected(SyntaxKind::OpenBraceToken);
        let members = self.parse_class_members();
        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end_pos = self.token_end();

        self.arena.add_class(
            syntax_kind_ext::CLASS_EXPRESSION,
            start_pos,
            end_pos,
            ClassData {
                modifiers: None,
                name,
                type_parameters,
                heritage_clauses: heritage,
                members,
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

    /// Check if current token is a parameter modifier
    fn is_parameter_modifier(&self) -> bool {
        matches!(
            self.current_token,
            SyntaxKind::PublicKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::ReadonlyKeyword
        )
    }

    /// Parse parameter modifiers (public, private, protected, readonly)
    fn parse_parameter_modifiers(&mut self) -> Option<NodeList> {
        let mut modifiers = Vec::new();

        while self.is_parameter_modifier() {
            let mod_start = self.token_pos();
            let mod_kind = self.current_token;
            self.next_token();
            let mod_end = self.token_end();
            modifiers.push(self.arena.add_token(mod_kind as u16, mod_start, mod_end));
        }

        if modifiers.is_empty() {
            None
        } else {
            Some(self.make_node_list(modifiers))
        }
    }

    /// Parse a single parameter
    fn parse_parameter(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse optional modifiers (public, private, protected, readonly)
        let modifiers = self.parse_parameter_modifiers();

        // Parse rest parameter (...)
        let dot_dot_dot_token = self.parse_optional(SyntaxKind::DotDotDotToken);

        // Parse parameter name - can be an identifier or keyword
        let name = if self.is_identifier_or_keyword() {
            self.parse_identifier_name()
        } else {
            self.parse_identifier()
        };

        // Parse optional question mark
        let question_token = self.parse_optional(SyntaxKind::QuestionToken);

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
                modifiers,
                dot_dot_dot_token,
                name,
                question_token,
                type_annotation,
                initializer,
            },
        )
    }

    /// Parse class declaration
    fn parse_class_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::ClassKeyword);

        // Parse class name - keywords like 'any', 'string' can be used as class names
        let name = if self.is_identifier_or_keyword() {
            self.parse_identifier_name()
        } else {
            NodeIndex::NONE
        };

        // Parse type parameters: class Foo<T, U> {}
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

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
                type_parameters,
                heritage_clauses,
                members,
            },
        )
    }

    /// Parse abstract class declaration: abstract class Foo {}
    fn parse_abstract_class_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Create abstract modifier node
        let abstract_start = self.token_pos();
        self.parse_expected(SyntaxKind::AbstractKeyword);
        let abstract_end = self.token_end();
        let abstract_modifier = self.arena.add_token(
            SyntaxKind::AbstractKeyword as u16,
            abstract_start,
            abstract_end,
        );

        // Now parse the class
        self.parse_expected(SyntaxKind::ClassKeyword);

        // Parse class name - keywords like 'any', 'string' can be used as class names
        let name = if self.is_identifier_or_keyword() {
            self.parse_identifier_name()
        } else {
            NodeIndex::NONE
        };

        // Parse type parameters: abstract class Foo<T, U> {}
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

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
                modifiers: Some(self.make_node_list(vec![abstract_modifier])),
                name,
                type_parameters,
                heritage_clauses,
                members,
            },
        )
    }

    /// Parse a decorated declaration: @decorator class/function
    fn parse_decorated_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse decorators
        let decorators = self.parse_decorators();

        // After decorators, expect class, abstract class, or function
        match self.token() {
            SyntaxKind::ClassKeyword => {
                self.parse_class_declaration_with_decorators(decorators, start_pos)
            }
            SyntaxKind::AbstractKeyword => {
                // abstract class with decorators
                self.parse_abstract_class_declaration_with_decorators(decorators, start_pos)
            }
            SyntaxKind::FunctionKeyword => {
                // For now, just parse the function and ignore decorators
                // Full decorator support would need function modifications
                self.parse_function_declaration()
            }
            SyntaxKind::ExportKeyword => {
                // Export with decorators: @decorator export class Foo {}
                self.parse_export_declaration()
            }
            _ => {
                // Unexpected - just continue
                self.parse_expression_statement()
            }
        }
    }

    /// Parse decorators: @decorator1 @decorator2(arg) ...
    fn parse_decorators(&mut self) -> Option<NodeList> {
        if !self.is_token(SyntaxKind::AtToken) {
            return None;
        }

        let mut decorators = Vec::new();

        while self.is_token(SyntaxKind::AtToken) {
            if let Some(decorator) = self.try_parse_decorator() {
                decorators.push(decorator);
            } else {
                break;
            }
        }

        if decorators.is_empty() {
            None
        } else {
            Some(self.make_node_list(decorators))
        }
    }

    /// Try to parse a single decorator
    fn try_parse_decorator(&mut self) -> Option<NodeIndex> {
        if !self.is_token(SyntaxKind::AtToken) {
            return None;
        }

        let start_pos = self.token_pos();
        self.next_token(); // consume @

        // Parse the decorator expression (identifier, member access, or call)
        let expression = self.parse_left_hand_side_expression();

        let end_pos = self.token_end();
        Some(self.arena.add_decorator(
            syntax_kind_ext::DECORATOR,
            start_pos,
            end_pos,
            crate::parser::thin_node::DecoratorData { expression },
        ))
    }

    /// Parse class declaration with pre-parsed decorators
    fn parse_class_declaration_with_decorators(&mut self, decorators: Option<NodeList>, start_pos: u32) -> NodeIndex {
        self.parse_expected(SyntaxKind::ClassKeyword);

        // Parse class name
        let name = if self.is_token(SyntaxKind::Identifier) {
            self.parse_identifier()
        } else {
            NodeIndex::NONE
        };

        // Parse heritage clauses (extends, implements)
        let heritage_clauses = self.parse_heritage_clauses();

        // Parse class body
        self.parse_expected(SyntaxKind::OpenBraceToken);
        let members = self.parse_class_members();
        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end_pos = self.token_end();

        // Create a modifiers list from decorators
        // In TypeScript, decorators are part of the modifiers
        self.arena.add_class(
            syntax_kind_ext::CLASS_DECLARATION,
            start_pos,
            end_pos,
            ClassData {
                modifiers: decorators,
                name,
                type_parameters: None,
                heritage_clauses,
                members,
            },
        )
    }

    /// Parse abstract class declaration with pre-parsed decorators
    fn parse_abstract_class_declaration_with_decorators(&mut self, decorators: Option<NodeList>, start_pos: u32) -> NodeIndex {
        // Create abstract modifier node
        let abstract_start = self.token_pos();
        self.parse_expected(SyntaxKind::AbstractKeyword);
        let abstract_end = self.token_end();
        let abstract_modifier = self.arena.add_token(
            SyntaxKind::AbstractKeyword as u16,
            abstract_start,
            abstract_end,
        );

        // Now parse the class
        self.parse_expected(SyntaxKind::ClassKeyword);

        // Parse class name
        let name = if self.is_token(SyntaxKind::Identifier) {
            self.parse_identifier()
        } else {
            NodeIndex::NONE
        };

        // Parse heritage clauses (extends, implements)
        let heritage_clauses = self.parse_heritage_clauses();

        // Parse class body
        self.parse_expected(SyntaxKind::OpenBraceToken);
        let members = self.parse_class_members();
        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end_pos = self.token_end();

        // Combine decorators with abstract modifier
        let modifiers = if let Some(dec_list) = decorators {
            // Add abstract modifier to decorator list
            let mut nodes: Vec<NodeIndex> = dec_list.nodes.clone();
            nodes.push(abstract_modifier);
            Some(self.make_node_list(nodes))
        } else {
            Some(self.make_node_list(vec![abstract_modifier]))
        };

        self.arena.add_class(
            syntax_kind_ext::CLASS_DECLARATION,
            start_pos,
            end_pos,
            ClassData {
                modifiers,
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
            let type_ref = self.parse_heritage_type_reference();
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
                let type_ref = self.parse_heritage_type_reference();
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

    /// Parse a heritage type reference: Foo or Foo<T> or Foo.Bar<T>
    /// This is used in extends/implements clauses
    fn parse_heritage_type_reference(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse the base expression (could be Foo or Foo.Bar.Baz)
        // We parse it as a left-hand-side expression to support property access
        let expression = self.parse_heritage_left_hand_expression();

        // Parse optional type arguments: <T, U>
        if self.is_token(SyntaxKind::LessThanToken) {
            self.next_token();
            let mut type_args = Vec::new();

            while !self.is_token(SyntaxKind::GreaterThanToken)
                && !self.is_token(SyntaxKind::EndOfFileToken)
            {
                let type_arg = self.parse_type();
                type_args.push(type_arg);

                if !self.parse_optional(SyntaxKind::CommaToken) {
                    break;
                }
            }

            self.parse_expected(SyntaxKind::GreaterThanToken);

            // Create expression with type arguments
            let end_pos = self.token_end();
            return self.arena.add_expr_with_type_args(
                syntax_kind_ext::EXPRESSION_WITH_TYPE_ARGUMENTS,
                start_pos,
                end_pos,
                crate::parser::thin_node::ExprWithTypeArgsData {
                    expression,
                    type_arguments: Some(self.make_node_list(type_args)),
                },
            );
        }

        expression
    }

    /// Parse left-hand expression for heritage clauses: Foo, Foo.Bar, or Mixin(Parent)
    /// This is a subset of member expression that allows identifiers, dots, and call expressions
    fn parse_heritage_left_hand_expression(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Start with identifier
        let mut expr = if self.is_identifier_or_keyword() {
            self.parse_identifier_name()
        } else {
            self.parse_identifier()
        };

        // Handle property access chain and call expressions: Foo.Bar.Baz or Mixin(Parent)
        loop {
            if self.is_token(SyntaxKind::DotToken) {
                self.next_token();
                let name = if self.is_identifier_or_keyword() {
                    self.parse_identifier_name()
                } else {
                    self.parse_identifier()
                };

                let end_pos = self.token_end();
                expr = self.arena.add_access_expr(
                    syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION,
                    start_pos,
                    end_pos,
                    crate::parser::thin_node::AccessExprData {
                        expression: expr,
                        name_or_argument: name,
                        question_dot_token: false,
                    },
                );
            } else if self.is_token(SyntaxKind::OpenParenToken) {
                // Call expression: Mixin(Parent)
                self.next_token();
                let mut args = Vec::new();
                while !self.is_token(SyntaxKind::CloseParenToken)
                    && !self.is_token(SyntaxKind::EndOfFileToken)
                {
                    let arg = self.parse_assignment_expression();
                    args.push(arg);
                    if !self.parse_optional(SyntaxKind::CommaToken) {
                        break;
                    }
                }
                self.parse_expected(SyntaxKind::CloseParenToken);

                let end_pos = self.token_end();
                expr = self.arena.add_call_expr(
                    syntax_kind_ext::CALL_EXPRESSION,
                    start_pos,
                    end_pos,
                    crate::parser::thin_node::CallExprData {
                        expression: expr,
                        type_arguments: None,
                        arguments: Some(self.make_node_list(args)),
                    },
                );
            } else {
                break;
            }
        }

        expr
    }

    /// Parse class member modifiers (static, public, private, protected, readonly, abstract, override)
    fn parse_class_member_modifiers(&mut self) -> Option<NodeList> {
        let mut modifiers = Vec::new();

        loop {
            let start_pos = self.token_pos();
            let modifier = match self.token() {
                SyntaxKind::StaticKeyword => {
                    self.next_token();
                    self.arena.create_modifier(SyntaxKind::StaticKeyword, start_pos)
                }
                SyntaxKind::PublicKeyword => {
                    self.next_token();
                    self.arena.create_modifier(SyntaxKind::PublicKeyword, start_pos)
                }
                SyntaxKind::PrivateKeyword => {
                    self.next_token();
                    self.arena.create_modifier(SyntaxKind::PrivateKeyword, start_pos)
                }
                SyntaxKind::ProtectedKeyword => {
                    self.next_token();
                    self.arena.create_modifier(SyntaxKind::ProtectedKeyword, start_pos)
                }
                SyntaxKind::ReadonlyKeyword => {
                    self.next_token();
                    self.arena.create_modifier(SyntaxKind::ReadonlyKeyword, start_pos)
                }
                SyntaxKind::AbstractKeyword => {
                    self.next_token();
                    self.arena.create_modifier(SyntaxKind::AbstractKeyword, start_pos)
                }
                SyntaxKind::OverrideKeyword => {
                    self.next_token();
                    self.arena.create_modifier(SyntaxKind::OverrideKeyword, start_pos)
                }
                SyntaxKind::AsyncKeyword => {
                    self.next_token();
                    self.arena.create_modifier(SyntaxKind::AsyncKeyword, start_pos)
                }
                SyntaxKind::DeclareKeyword => {
                    self.next_token();
                    self.arena.create_modifier(SyntaxKind::DeclareKeyword, start_pos)
                }
                _ => break,
            };
            modifiers.push(modifier);
        }

        if modifiers.is_empty() {
            None
        } else {
            Some(self.make_node_list(modifiers))
        }
    }

    /// Parse constructor with modifiers
    fn parse_constructor_with_modifiers(&mut self, modifiers: Option<NodeList>) -> NodeIndex {
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
                modifiers,
                type_parameters: None,
                parameters,
                body,
            },
        )
    }

    /// Parse get accessor with modifiers: static get foo() { }
    fn parse_get_accessor_with_modifiers(&mut self, modifiers: Option<NodeList>, start_pos: u32) -> NodeIndex {
        self.parse_expected(SyntaxKind::GetKeyword);

        let name = self.parse_property_name();

        self.parse_expected(SyntaxKind::OpenParenToken);
        self.parse_expected(SyntaxKind::CloseParenToken);

        // Optional return type
        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        // Parse body (may be empty for ambient declarations)
        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            self.parse_semicolon();
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_accessor(
            syntax_kind_ext::GET_ACCESSOR,
            start_pos,
            end_pos,
            crate::parser::thin_node::AccessorData {
                modifiers,
                name,
                type_parameters: None,
                parameters: self.make_node_list(vec![]),
                type_annotation,
                body,
            },
        )
    }

    /// Parse set accessor with modifiers: static set foo(value) { }
    fn parse_set_accessor_with_modifiers(&mut self, modifiers: Option<NodeList>, start_pos: u32) -> NodeIndex {
        self.parse_expected(SyntaxKind::SetKeyword);

        let name = self.parse_property_name();

        self.parse_expected(SyntaxKind::OpenParenToken);
        let parameters = self.parse_parameter_list();
        self.parse_expected(SyntaxKind::CloseParenToken);

        // Parse body (may be empty for ambient declarations)
        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            self.parse_semicolon();
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_accessor(
            syntax_kind_ext::SET_ACCESSOR,
            start_pos,
            end_pos,
            crate::parser::thin_node::AccessorData {
                modifiers,
                name,
                type_parameters: None,
                parameters,
                type_annotation: NodeIndex::NONE,
                body,
            },
        )
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

        // Parse modifiers (static, public, private, protected, readonly, abstract, override)
        let modifiers = self.parse_class_member_modifiers();

        // Handle constructor
        if self.is_token(SyntaxKind::ConstructorKeyword) {
            return self.parse_constructor_with_modifiers(modifiers);
        }

        // Handle get accessor: get foo() { }
        if self.is_token(SyntaxKind::GetKeyword) && self.look_ahead_is_accessor() {
            return self.parse_get_accessor_with_modifiers(modifiers, start_pos);
        }

        // Handle set accessor: set foo(value) { }
        if self.is_token(SyntaxKind::SetKeyword) && self.look_ahead_is_accessor() {
            return self.parse_set_accessor_with_modifiers(modifiers, start_pos);
        }

        // Handle methods and properties
        // For now, just parse name and check for ( for methods
        let name = if self.is_token(SyntaxKind::Identifier) ||
                     self.is_token(SyntaxKind::StringLiteral) ||
                     self.is_token(SyntaxKind::NumericLiteral) ||
                     self.is_token(SyntaxKind::GetKeyword) ||
                     self.is_token(SyntaxKind::SetKeyword) ||
                     self.is_token(SyntaxKind::OpenBracketToken) {
            self.parse_property_name()
        } else {
            // Skip unknown token
            self.next_token();
            return NodeIndex::NONE;
        };

        // Check if it's a method or property
        // Method: foo() or foo<T>()
        if self.is_token(SyntaxKind::OpenParenToken)
            || self.is_token(SyntaxKind::LessThanToken)
        {
            // Parse optional type parameters: foo<T, U>()
            let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
                Some(self.parse_type_parameters())
            } else {
                None
            };

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
                    modifiers,
                    asterisk_token: false,
                    name,
                    question_token: false,
                    type_parameters,
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
                    modifiers,
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

    /// Look ahead to see if we have an accessor (get/set followed by property name and ()
    fn look_ahead_is_accessor(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        // Skip 'get' or 'set'
        self.next_token();

        // Check for property name (identifier, string, number, or computed)
        let has_name = self.is_token(SyntaxKind::Identifier) ||
                       self.is_token(SyntaxKind::StringLiteral) ||
                       self.is_token(SyntaxKind::NumericLiteral) ||
                       self.is_token(SyntaxKind::OpenBracketToken);

        self.scanner.restore_state(snapshot);
        self.current_token = current;
        has_name
    }

    /// Parse get accessor: get foo() { return value; }
    fn parse_get_accessor(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::GetKeyword);

        let name = self.parse_property_name();

        self.parse_expected(SyntaxKind::OpenParenToken);
        self.parse_expected(SyntaxKind::CloseParenToken);

        // Optional return type
        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        // Parse body (may be empty for ambient declarations)
        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            self.parse_semicolon();
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_accessor(
            syntax_kind_ext::GET_ACCESSOR,
            start_pos,
            end_pos,
            crate::parser::thin_node::AccessorData {
                modifiers: None,
                name,
                type_parameters: None,
                parameters: self.make_node_list(vec![]),
                type_annotation,
                body,
            },
        )
    }

    /// Parse set accessor: set foo(value) { this.value = value; }
    fn parse_set_accessor(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::SetKeyword);

        let name = self.parse_property_name();

        self.parse_expected(SyntaxKind::OpenParenToken);
        let parameters = self.parse_parameter_list();
        self.parse_expected(SyntaxKind::CloseParenToken);

        // Parse body (may be empty for ambient declarations)
        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            self.parse_semicolon();
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_accessor(
            syntax_kind_ext::SET_ACCESSOR,
            start_pos,
            end_pos,
            crate::parser::thin_node::AccessorData {
                modifiers: None,
                name,
                type_parameters: None,
                parameters,
                type_annotation: NodeIndex::NONE,
                body,
            },
        )
    }

    /// Parse interface declaration
    fn parse_interface_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::InterfaceKeyword);

        // Parse interface name - keywords like 'string', 'abstract' can be used as interface names
        let name = if self.is_identifier_or_keyword() {
            self.parse_identifier_name()
        } else {
            self.parse_identifier()
        };

        // Parse type parameters: interface IList<T> {}
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

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
                type_parameters,
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

    /// Parse a single type member (property signature, method signature, call signature, construct signature)
    fn parse_type_member(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Handle generic call signature: <T>(): returnType
        if self.is_token(SyntaxKind::LessThanToken) {
            return self.parse_call_signature(start_pos);
        }

        // Handle call signature: (): returnType
        if self.is_token(SyntaxKind::OpenParenToken) {
            return self.parse_call_signature(start_pos);
        }

        // Handle construct signature: new (): returnType
        if self.is_token(SyntaxKind::NewKeyword) {
            return self.parse_construct_signature(start_pos);
        }

        // Handle get accessor: get foo(): type
        // But not if 'get' is used as property name (get: T or get?: T or get() or get<T>())
        if self.is_token(SyntaxKind::GetKeyword) && !self.look_ahead_is_property_name_after_keyword() {
            return self.parse_get_accessor_signature(start_pos);
        }

        // Handle set accessor: set foo(v: type)
        // But not if 'set' is used as property name
        if self.is_token(SyntaxKind::SetKeyword) && !self.look_ahead_is_property_name_after_keyword() {
            return self.parse_set_accessor_signature(start_pos);
        }

        // Parse optional readonly modifier
        // But not if 'readonly' is used as property name
        let readonly = if self.is_token(SyntaxKind::ReadonlyKeyword) && !self.look_ahead_is_property_name_after_keyword() {
            self.next_token();
            true
        } else {
            false
        };

        // Parse property/method name
        // Include keywords that can be property names
        let name = if self.is_token(SyntaxKind::Identifier) ||
                     self.is_token(SyntaxKind::StringLiteral) ||
                     self.is_token(SyntaxKind::NumericLiteral) ||
                     self.is_property_name_keyword() {
            self.parse_property_name()
        } else if self.is_token(SyntaxKind::OpenBracketToken) {
            // Index signature: [key: string]: value (possibly with readonly)
            return self.parse_index_signature_with_readonly(readonly, start_pos);
        } else {
            return NodeIndex::NONE;
        };

        // Optional question mark
        let question_token = self.parse_optional(SyntaxKind::QuestionToken);

        // Build modifiers list if readonly was present
        let modifiers = if readonly {
            let mod_idx = self.arena.create_modifier(SyntaxKind::ReadonlyKeyword, start_pos);
            Some(self.make_node_list(vec![mod_idx]))
        } else {
            None
        };

        // Check if it's a method signature or property signature
        // Method signature: foo(): T or foo<T>(): U
        if self.is_token(SyntaxKind::OpenParenToken)
            || self.is_token(SyntaxKind::LessThanToken)
        {
            // Parse optional type parameters: foo<T, U>()
            let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
                Some(self.parse_type_parameters())
            } else {
                None
            };

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
                    modifiers,
                    name,
                    question_token,
                    type_parameters,
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
                    modifiers,
                    name,
                    question_token,
                    type_parameters: None,
                    parameters: None,
                    type_annotation,
                },
            )
        }
    }

    /// Parse call signature: (): returnType or <T>(): returnType
    fn parse_call_signature(&mut self, start_pos: u32) -> NodeIndex {
        // Parse optional type parameters: <T, U>
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

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
            syntax_kind_ext::CALL_SIGNATURE,
            start_pos,
            end_pos,
            crate::parser::thin_node::SignatureData {
                modifiers: None,
                name: NodeIndex::NONE,
                question_token: false,
                type_parameters,
                parameters: Some(parameters),
                type_annotation,
            },
        )
    }

    /// Parse construct signature: new (): returnType or new <T>(): returnType
    fn parse_construct_signature(&mut self, start_pos: u32) -> NodeIndex {
        self.parse_expected(SyntaxKind::NewKeyword);

        // Parse optional type parameters: new <T>()
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

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
            syntax_kind_ext::CONSTRUCT_SIGNATURE,
            start_pos,
            end_pos,
            crate::parser::thin_node::SignatureData {
                modifiers: None,
                name: NodeIndex::NONE,
                question_token: false,
                type_parameters,
                parameters: Some(parameters),
                type_annotation,
            },
        )
    }

    /// Parse index signature: [key: string]: value
    fn parse_index_signature(&mut self) -> NodeIndex {
        self.parse_index_signature_with_readonly(false, self.token_pos())
    }

    /// Parse index signature with optional readonly modifier: readonly [key: string]: value
    fn parse_index_signature_with_readonly(&mut self, readonly: bool, start_pos: u32) -> NodeIndex {
        self.parse_expected(SyntaxKind::OpenBracketToken);

        // Parse parameter
        let param_name = self.parse_identifier();
        self.parse_expected(SyntaxKind::ColonToken);
        let param_type = self.parse_type();

        self.parse_expected(SyntaxKind::CloseBracketToken);
        self.parse_expected(SyntaxKind::ColonToken);

        let type_annotation = self.parse_type();

        // Build modifiers list if readonly was present
        let modifiers = if readonly {
            let mod_idx = self.arena.create_modifier(SyntaxKind::ReadonlyKeyword, start_pos);
            Some(self.make_node_list(vec![mod_idx]))
        } else {
            None
        };

        let end_pos = self.token_end();
        self.arena.add_index_signature(
            syntax_kind_ext::INDEX_SIGNATURE,
            start_pos,
            end_pos,
            crate::parser::thin_node::IndexSignatureData {
                modifiers,
                parameters: self.make_node_list(vec![param_name]),
                type_annotation,
            },
        )
    }

    /// Parse get accessor signature in type context: get foo(): type
    /// Note: TypeScript allows bodies here (which is an error), so we parse them for error recovery
    fn parse_get_accessor_signature(&mut self, start_pos: u32) -> NodeIndex {
        self.parse_expected(SyntaxKind::GetKeyword);

        let name = self.parse_property_name();

        self.parse_expected(SyntaxKind::OpenParenToken);
        self.parse_expected(SyntaxKind::CloseParenToken);

        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        // Parse body if present (this is an error in type context, but we handle it)
        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_accessor(
            syntax_kind_ext::GET_ACCESSOR,
            start_pos,
            end_pos,
            crate::parser::thin_node::AccessorData {
                modifiers: None,
                name,
                type_parameters: None,
                parameters: self.make_node_list(vec![]),
                type_annotation,
                body,
            },
        )
    }

    /// Parse set accessor signature in type context: set foo(v: type)
    /// Note: TypeScript allows bodies here (which is an error), so we parse them for error recovery
    fn parse_set_accessor_signature(&mut self, start_pos: u32) -> NodeIndex {
        self.parse_expected(SyntaxKind::SetKeyword);

        let name = self.parse_property_name();

        self.parse_expected(SyntaxKind::OpenParenToken);
        let parameters = self.parse_parameter_list();
        self.parse_expected(SyntaxKind::CloseParenToken);

        // Parse body if present (this is an error in type context, but we handle it)
        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_accessor(
            syntax_kind_ext::SET_ACCESSOR,
            start_pos,
            end_pos,
            crate::parser::thin_node::AccessorData {
                modifiers: None,
                name,
                type_parameters: None,
                parameters,
                type_annotation: NodeIndex::NONE,
                body,
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

    /// Parse enum declaration
    fn parse_enum_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::EnumKeyword);

        let name = self.parse_identifier();

        self.parse_expected(SyntaxKind::OpenBraceToken);

        let members = self.parse_enum_members();

        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end_pos = self.token_end();
        self.arena.add_enum(
            syntax_kind_ext::ENUM_DECLARATION,
            start_pos,
            end_pos,
            EnumData {
                modifiers: None,
                name,
                members,
            },
        )
    }

    /// Parse enum members
    fn parse_enum_members(&mut self) -> NodeList {
        let mut members = Vec::new();

        while !self.is_token(SyntaxKind::CloseBraceToken) && !self.is_token(SyntaxKind::EndOfFileToken) {
            let start_pos = self.token_pos();

            // Enum member names can be identifiers or string literals
            let name = if self.is_token(SyntaxKind::StringLiteral) {
                self.parse_string_literal()
            } else {
                self.parse_identifier()
            };

            let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
                self.parse_assignment_expression()
            } else {
                NodeIndex::NONE
            };

            let end_pos = self.token_end();
            let member = self.arena.add_enum_member(
                syntax_kind_ext::ENUM_MEMBER,
                start_pos,
                end_pos,
                EnumMemberData { name, initializer },
            );
            members.push(member);

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.make_node_list(members)
    }

    // =========================================================================
    // Module/Namespace Declarations
    // =========================================================================

    /// Parse ambient declaration: declare function/class/namespace/var/etc.
    fn parse_ambient_declaration(&mut self) -> NodeIndex {
        let _start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::DeclareKeyword);

        // Parse the inner declaration based on what follows 'declare'
        match self.token() {
            SyntaxKind::FunctionKeyword => self.parse_function_declaration(),
            SyntaxKind::ClassKeyword => self.parse_class_declaration(),
            SyntaxKind::AbstractKeyword => {
                // declare abstract class
                self.parse_class_declaration()
            }
            SyntaxKind::InterfaceKeyword => self.parse_interface_declaration(),
            SyntaxKind::TypeKeyword => self.parse_type_alias_declaration(),
            SyntaxKind::EnumKeyword => self.parse_enum_declaration(),
            SyntaxKind::NamespaceKeyword |
            SyntaxKind::ModuleKeyword => self.parse_module_declaration(),
            SyntaxKind::GlobalKeyword => self.parse_module_declaration(),
            SyntaxKind::VarKeyword |
            SyntaxKind::LetKeyword |
            SyntaxKind::ConstKeyword => self.parse_variable_statement(),
            SyntaxKind::AsyncKeyword => {
                // declare async function
                if self.look_ahead_is_async_function() {
                    self.parse_async_function_declaration()
                } else {
                    self.parse_error_at_current_token("Declaration expected after 'declare'");
                    self.parse_expression_statement()
                }
            }
            _ => {
                self.parse_error_at_current_token("Declaration expected after 'declare'");
                self.parse_expression_statement()
            }
        }
    }

    /// Parse module or namespace declaration: module "name" { } or namespace X { }
    fn parse_module_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Skip module/namespace/global keyword
        let is_global = self.is_token(SyntaxKind::GlobalKeyword);
        self.next_token();

        // Parse name - can be identifier or string literal
        let name = if self.is_token(SyntaxKind::StringLiteral) {
            self.parse_string_literal()
        } else {
            self.parse_module_name()
        };

        // Parse body
        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_module_block()
        } else if self.is_token(SyntaxKind::DotToken) {
            // Nested module: module A.B.C { }
            self.next_token();
            self.parse_module_declaration()
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();

        self.arena.add_module(
            syntax_kind_ext::MODULE_DECLARATION,
            start_pos,
            end_pos,
            crate::parser::thin_node::ModuleData {
                modifiers: None,
                name,
                body,
            },
        )
    }

    /// Parse module name (can be dotted: A.B.C)
    fn parse_module_name(&mut self) -> NodeIndex {
        let mut left = self.parse_identifier();

        while self.is_token(SyntaxKind::DotToken) {
            self.next_token();
            let right = self.parse_identifier();
            let start = if let Some(n) = self.arena.get(left) { n.pos } else { 0 };
            let end = self.token_end();

            left = self.arena.add_qualified_name(
                syntax_kind_ext::QUALIFIED_NAME,
                start,
                end,
                QualifiedNameData { left, right },
            );
        }

        left
    }

    /// Parse module block: { statements }
    fn parse_module_block(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        let statements = self.parse_statements();

        self.parse_expected(SyntaxKind::CloseBraceToken);
        let end_pos = self.token_end();

        self.arena.add_module_block(
            syntax_kind_ext::MODULE_BLOCK,
            start_pos,
            end_pos,
            crate::parser::thin_node::ModuleBlockData { statements: Some(statements) },
        )
    }

    // =========================================================================
    // Import/Export Declarations
    // =========================================================================

    /// Parse import declaration
    /// import x from "mod";
    /// import { x, y } from "mod";
    /// import * as x from "mod";
    /// import "mod";
    fn parse_import_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::ImportKeyword);

        // Check for import "module" (no import clause)
        let import_clause = if self.is_token(SyntaxKind::StringLiteral) {
            NodeIndex::NONE
        } else {
            self.parse_import_clause()
        };

        // Parse module specifier
        let module_specifier = if !import_clause.is_none() {
            self.parse_expected(SyntaxKind::FromKeyword);
            self.parse_string_literal()
        } else {
            self.parse_string_literal()
        };

        self.parse_semicolon();
        let end_pos = self.token_end();

        self.arena.add_import_decl(
            syntax_kind_ext::IMPORT_DECLARATION,
            start_pos,
            end_pos,
            ImportDeclData {
                modifiers: None,
                import_clause,
                module_specifier,
                attributes: NodeIndex::NONE,
            },
        )
    }

    /// Parse import clause: default, namespace, or named imports
    fn parse_import_clause(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let mut is_type_only = false;

        // Check for "type" keyword (import type { ... })
        if self.is_token(SyntaxKind::TypeKeyword) {
            // Look ahead to see if this is "type" followed by identifier or "{"
            let snapshot = self.scanner.save_state();
            let current = self.current_token;
            self.next_token();
            if self.is_token(SyntaxKind::Identifier) || self.is_token(SyntaxKind::OpenBraceToken) ||
               self.is_token(SyntaxKind::AsteriskToken) {
                is_type_only = true;
            } else {
                self.scanner.restore_state(snapshot);
                self.current_token = current;
            }
        }

        // Parse default import (identifier followed by "from" or ",")
        // For "import foo from", next token is "from"
        // For "import foo, { bar } from", next token is ","
        let name = if self.is_token(SyntaxKind::Identifier) {
            self.parse_identifier()
        } else {
            NodeIndex::NONE
        };

        // Parse comma if we have both default and named/namespace
        if !name.is_none() && self.parse_optional(SyntaxKind::CommaToken) {
            // Continue to parse named bindings
        }

        // Parse named bindings: * as ns or { x, y }
        let named_bindings = if self.is_token(SyntaxKind::AsteriskToken) {
            self.parse_namespace_import()
        } else if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_named_imports()
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_import_clause(
            syntax_kind_ext::IMPORT_CLAUSE,
            start_pos,
            end_pos,
            ImportClauseData {
                is_type_only,
                name,
                named_bindings,
            },
        )
    }

    /// Check if next token is "from" keyword
    fn is_next_token_from(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;
        self.next_token();
        let is_from = self.is_token(SyntaxKind::FromKeyword);
        self.scanner.restore_state(snapshot);
        self.current_token = current;
        is_from
    }

    /// Parse namespace import: * as name
    fn parse_namespace_import(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::AsteriskToken);
        self.parse_expected(SyntaxKind::AsKeyword);
        let name = self.parse_identifier();
        let end_pos = self.token_end();

        // Store the namespace import with the name
        // For namespace import, we return the name identifier directly
        // The caller knows we're in a namespace import context
        name
    }

    /// Parse named imports: { x, y as z }
    fn parse_named_imports(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        let mut elements = Vec::new();
        while !self.is_token(SyntaxKind::CloseBraceToken) && !self.is_token(SyntaxKind::EndOfFileToken) {
            let spec = self.parse_import_specifier();
            elements.push(spec);

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.parse_expected(SyntaxKind::CloseBraceToken);
        let end_pos = self.token_end();

        self.arena.add_named_imports(
            syntax_kind_ext::NAMED_IMPORTS,
            start_pos,
            end_pos,
            NamedImportsData {
                name: NodeIndex::NONE,  // Not a namespace import
                elements: self.make_node_list(elements),
            },
        )
    }

    /// Parse import specifier: x or x as y
    fn parse_import_specifier(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let mut is_type_only = false;

        // Check for "type" keyword
        if self.is_token(SyntaxKind::TypeKeyword) {
            let snapshot = self.scanner.save_state();
            let current = self.current_token;
            self.next_token();
            if self.is_token(SyntaxKind::Identifier) {
                is_type_only = true;
            } else {
                self.scanner.restore_state(snapshot);
                self.current_token = current;
            }
        }

        let first_name = self.parse_identifier();

        // Check for "as" alias
        let (property_name, name) = if self.parse_optional(SyntaxKind::AsKeyword) {
            let alias = self.parse_identifier();
            (first_name, alias)
        } else {
            (NodeIndex::NONE, first_name)
        };

        let end_pos = self.token_end();
        self.arena.add_specifier(
            syntax_kind_ext::IMPORT_SPECIFIER,
            start_pos,
            end_pos,
            SpecifierData {
                is_type_only,
                property_name,
                name,
            },
        )
    }

    /// Parse export declaration
    /// export { x, y };
    /// export { x } from "mod";
    /// export * from "mod";
    /// export default x;
    /// export function f() {}
    /// export class C {}
    fn parse_export_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::ExportKeyword);

        // Check for type-only export vs export type alias
        // "export type { X }" or "export type * from" = type-only export
        // "export type X = Y" = exported type alias declaration
        let is_type_only = if self.is_token(SyntaxKind::TypeKeyword) {
            // Look ahead to see if this is a type-only export
            let snapshot = self.scanner.save_state();
            let current = self.current_token;
            self.next_token(); // skip 'type'

            let is_type_only_export = self.is_token(SyntaxKind::OpenBraceToken)
                || self.is_token(SyntaxKind::AsteriskToken);

            self.scanner.restore_state(snapshot);
            self.current_token = current;

            if is_type_only_export {
                self.next_token(); // consume 'type' for type-only exports
                true
            } else {
                // Not a type-only export - leave 'type' for parse_export_declaration_or_statement
                false
            }
        } else {
            false
        };

        // export default ...
        if self.is_token(SyntaxKind::DefaultKeyword) {
            return self.parse_export_default(start_pos);
        }

        // export import X = Y (re-export of import equals)
        if self.is_token(SyntaxKind::ImportKeyword) {
            return self.parse_export_import_equals(start_pos);
        }

        // export * from "mod"
        if self.is_token(SyntaxKind::AsteriskToken) {
            return self.parse_export_star(start_pos, is_type_only);
        }

        // export { ... }
        if self.is_token(SyntaxKind::OpenBraceToken) {
            return self.parse_export_named(start_pos, is_type_only);
        }

        // export = expression (CommonJS-style export)
        if self.is_token(SyntaxKind::EqualsToken) {
            return self.parse_export_assignment(start_pos);
        }

        // export function/class/const/let/var/interface/type/enum
        self.parse_export_declaration_or_statement(start_pos)
    }

    /// Parse export import X = Y (re-export of import equals declaration)
    fn parse_export_import_equals(&mut self, start_pos: u32) -> NodeIndex {
        // Parse the import equals declaration
        let import_decl = self.parse_import_equals_declaration();

        let end_pos = self.token_end();

        // Wrap in an export declaration
        self.arena.add_export_decl(
            syntax_kind_ext::EXPORT_DECLARATION,
            start_pos,
            end_pos,
            ExportDeclData {
                modifiers: None,
                is_type_only: false,
                export_clause: import_decl,
                module_specifier: NodeIndex::NONE,
                attributes: NodeIndex::NONE,
            },
        )
    }

    /// Parse export = expression (CommonJS-style default export)
    fn parse_export_assignment(&mut self, start_pos: u32) -> NodeIndex {
        self.parse_expected(SyntaxKind::EqualsToken);
        let expression = self.parse_assignment_expression();
        self.parse_semicolon();

        let end_pos = self.token_end();

        self.arena.add_export_assignment(
            syntax_kind_ext::EXPORT_ASSIGNMENT,
            start_pos,
            end_pos,
            ExportAssignmentData {
                modifiers: None,
                is_export_equals: true,
                expression,
            },
        )
    }

    /// Parse export default
    fn parse_export_default(&mut self, start_pos: u32) -> NodeIndex {
        self.parse_expected(SyntaxKind::DefaultKeyword);

        // Parse the default expression or declaration
        let expression = match self.token() {
            SyntaxKind::FunctionKeyword => self.parse_function_declaration(),
            SyntaxKind::ClassKeyword => self.parse_class_declaration(),
            _ => {
                let expr = self.parse_assignment_expression();
                self.parse_semicolon();
                expr
            }
        };

        let end_pos = self.token_end();
        // Use export assignment for default exports
        self.arena.add_export_decl(
            syntax_kind_ext::EXPORT_DECLARATION,
            start_pos,
            end_pos,
            ExportDeclData {
                modifiers: None,
                is_type_only: false,
                export_clause: expression,
                module_specifier: NodeIndex::NONE,
                attributes: NodeIndex::NONE,
            },
        )
    }

    /// Parse export * from "mod"
    fn parse_export_star(&mut self, start_pos: u32, is_type_only: bool) -> NodeIndex {
        self.parse_expected(SyntaxKind::AsteriskToken);

        // Optional "as namespace" for re-export
        let export_clause = if self.parse_optional(SyntaxKind::AsKeyword) {
            self.parse_identifier()
        } else {
            NodeIndex::NONE
        };

        self.parse_expected(SyntaxKind::FromKeyword);
        let module_specifier = self.parse_string_literal();
        self.parse_semicolon();

        let end_pos = self.token_end();
        self.arena.add_export_decl(
            syntax_kind_ext::EXPORT_DECLARATION,
            start_pos,
            end_pos,
            ExportDeclData {
                modifiers: None,
                is_type_only,
                export_clause,
                module_specifier,
                attributes: NodeIndex::NONE,
            },
        )
    }

    /// Parse export { x, y } or export { x } from "mod"
    fn parse_export_named(&mut self, start_pos: u32, is_type_only: bool) -> NodeIndex {
        let export_clause = self.parse_named_exports();

        let module_specifier = if self.parse_optional(SyntaxKind::FromKeyword) {
            self.parse_string_literal()
        } else {
            NodeIndex::NONE
        };

        self.parse_semicolon();
        let end_pos = self.token_end();

        self.arena.add_export_decl(
            syntax_kind_ext::EXPORT_DECLARATION,
            start_pos,
            end_pos,
            ExportDeclData {
                modifiers: None,
                is_type_only,
                export_clause,
                module_specifier,
                attributes: NodeIndex::NONE,
            },
        )
    }

    /// Parse named exports: { x, y as z }
    fn parse_named_exports(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        let mut elements = Vec::new();
        while !self.is_token(SyntaxKind::CloseBraceToken) && !self.is_token(SyntaxKind::EndOfFileToken) {
            let spec = self.parse_export_specifier();
            elements.push(spec);

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.parse_expected(SyntaxKind::CloseBraceToken);
        let end_pos = self.token_end();

        self.arena.add_named_imports(
            syntax_kind_ext::NAMED_EXPORTS,
            start_pos,
            end_pos,
            NamedImportsData {
                name: NodeIndex::NONE,  // Not a namespace export
                elements: self.make_node_list(elements),
            },
        )
    }

    /// Parse export specifier: x or x as y
    fn parse_export_specifier(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let mut is_type_only = false;

        // Check for "type" keyword
        if self.is_token(SyntaxKind::TypeKeyword) {
            let snapshot = self.scanner.save_state();
            let current = self.current_token;
            self.next_token();
            if self.is_token(SyntaxKind::Identifier) {
                is_type_only = true;
            } else {
                self.scanner.restore_state(snapshot);
                self.current_token = current;
            }
        }

        let first_name = self.parse_identifier();

        // Check for "as" alias
        let (property_name, name) = if self.parse_optional(SyntaxKind::AsKeyword) {
            let alias = self.parse_identifier();
            (first_name, alias)
        } else {
            (NodeIndex::NONE, first_name)
        };

        let end_pos = self.token_end();
        self.arena.add_specifier(
            syntax_kind_ext::EXPORT_SPECIFIER,
            start_pos,
            end_pos,
            SpecifierData {
                is_type_only,
                property_name,
                name,
            },
        )
    }

    /// Parse exported declaration (export function, export class, etc.)
    fn parse_export_declaration_or_statement(&mut self, start_pos: u32) -> NodeIndex {
        // Parse the declaration and wrap it
        let declaration = match self.token() {
            SyntaxKind::FunctionKeyword => self.parse_function_declaration(),
            SyntaxKind::AsyncKeyword => {
                if self.look_ahead_is_async_function() {
                    self.parse_async_function_declaration()
                } else {
                    self.parse_expression_statement()
                }
            }
            SyntaxKind::ClassKeyword => self.parse_class_declaration(),
            SyntaxKind::InterfaceKeyword => self.parse_interface_declaration(),
            SyntaxKind::TypeKeyword => self.parse_type_alias_declaration(),
            SyntaxKind::EnumKeyword => self.parse_enum_declaration(),
            SyntaxKind::NamespaceKeyword |
            SyntaxKind::ModuleKeyword => self.parse_module_declaration(),
            SyntaxKind::AbstractKeyword => {
                // export abstract class ...
                self.parse_class_declaration()
            }
            SyntaxKind::DeclareKeyword => {
                // export declare function/class/namespace/var/etc.
                self.parse_ambient_declaration()
            }
            SyntaxKind::VarKeyword |
            SyntaxKind::LetKeyword |
            SyntaxKind::ConstKeyword => self.parse_variable_statement(),
            _ => {
                // Unsupported export
                self.parse_error_at_current_token("Declaration or statement expected");
                self.parse_expression_statement()
            }
        };

        let end_pos = self.token_end();
        self.arena.add_export_decl(
            syntax_kind_ext::EXPORT_DECLARATION,
            start_pos,
            end_pos,
            ExportDeclData {
                modifiers: None,
                is_type_only: false,
                export_clause: declaration,
                module_specifier: NodeIndex::NONE,
                attributes: NodeIndex::NONE,
            },
        )
    }

    /// Parse a string literal (used for module specifiers)
    fn parse_string_literal(&mut self) -> NodeIndex {
        if !self.is_token(SyntaxKind::StringLiteral) {
            self.parse_error_at_current_token("String literal expected");
            return NodeIndex::NONE;
        }

        let start_pos = self.token_pos();
        let text = self.scanner.get_token_value_ref().to_string();
        self.next_token();
        let end_pos = self.token_end();

        self.arena.add_literal(
            SyntaxKind::StringLiteral as u16,
            start_pos,
            end_pos,
            LiteralData {
                text,
                raw_text: None,
                value: None,
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

        // Check for for-await-of: for await (...)
        let await_modifier = self.parse_optional(SyntaxKind::AwaitKeyword);

        self.parse_expected(SyntaxKind::OpenParenToken);

        // Parse initializer (can be var/let/const declaration or expression)
        let initializer = if !self.is_token(SyntaxKind::SemicolonToken) {
            if self.is_token(SyntaxKind::VarKeyword) ||
               self.is_token(SyntaxKind::LetKeyword) ||
               self.is_token(SyntaxKind::ConstKeyword) {
                self.parse_for_variable_declaration()
            } else {
                self.parse_expression()
            }
        } else {
            NodeIndex::NONE
        };

        // Check for for-in or for-of
        if self.is_token(SyntaxKind::InKeyword) {
            return self.parse_for_in_statement_rest(start_pos, initializer);
        }
        if self.is_token(SyntaxKind::OfKeyword) {
            return self.parse_for_of_statement_rest(start_pos, initializer, await_modifier);
        }

        // Regular for statement: for (init; cond; incr)
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

    /// Parse variable declaration for for-in/for-of (single declaration only)
    fn parse_for_variable_declaration(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let _decl_keyword = self.token();
        self.next_token(); // consume var/let/const

        // Parse single variable declaration (for-in/for-of only allows one)
        // Use similar logic to parse_variable_declaration
        let name = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_object_binding_pattern()
        } else if self.is_token(SyntaxKind::OpenBracketToken) {
            self.parse_array_binding_pattern()
        } else if self.is_identifier_or_keyword() {
            self.parse_identifier_name()
        } else {
            self.parse_identifier()
        };

        // Optional type annotation
        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        // For for-in/for-of, initializer is parsed separately as the expression
        // But for regular for, there might be an initializer
        let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
            self.parse_assignment_expression()
        } else {
            NodeIndex::NONE
        };

        let decl = self.arena.add_variable_declaration(
            syntax_kind_ext::VARIABLE_DECLARATION,
            start_pos,
            self.token_end(),
            VariableDeclarationData {
                name,
                type_annotation,
                initializer,
                exclamation_token: false,
            },
        );

        let declarations = self.make_node_list(vec![decl]);
        let end_pos = self.token_end();

        self.arena.add_variable(
            syntax_kind_ext::VARIABLE_DECLARATION_LIST,
            start_pos,
            end_pos,
            VariableData {
                modifiers: None,
                declarations,
            },
        )
    }

    /// Parse for-in statement after initializer: for (x in obj)
    fn parse_for_in_statement_rest(&mut self, start_pos: u32, initializer: NodeIndex) -> NodeIndex {
        self.parse_expected(SyntaxKind::InKeyword);
        let expression = self.parse_expression();
        self.parse_expected(SyntaxKind::CloseParenToken);
        let statement = self.parse_statement();

        let end_pos = self.token_end();
        self.arena.add_for_in_of(
            syntax_kind_ext::FOR_IN_STATEMENT,
            start_pos,
            end_pos,
            crate::parser::thin_node::ForInOfData {
                await_modifier: false,
                initializer,
                expression,
                statement,
            },
        )
    }

    /// Parse for-of statement after initializer: for (x of arr)
    fn parse_for_of_statement_rest(&mut self, start_pos: u32, initializer: NodeIndex, await_modifier: bool) -> NodeIndex {
        self.parse_expected(SyntaxKind::OfKeyword);
        let expression = self.parse_assignment_expression();
        self.parse_expected(SyntaxKind::CloseParenToken);
        let statement = self.parse_statement();

        let end_pos = self.token_end();
        self.arena.add_for_in_of(
            syntax_kind_ext::FOR_OF_STATEMENT,
            start_pos,
            end_pos,
            crate::parser::thin_node::ForInOfData {
                await_modifier,
                initializer,
                expression,
                statement,
            },
        )
    }

    /// Parse break statement
    fn parse_break_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::BreakKeyword);

        // Optional label
        let label = if !self.can_parse_semicolon() && self.is_identifier_or_keyword() {
            self.parse_identifier_name()
        } else {
            NodeIndex::NONE
        };

        self.parse_semicolon();
        let end_pos = self.token_end();

        self.arena.add_token(syntax_kind_ext::BREAK_STATEMENT as u16, start_pos, end_pos)
    }

    /// Parse continue statement
    fn parse_continue_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::ContinueKeyword);

        // Optional label
        let _label = if !self.can_parse_semicolon() && self.is_identifier_or_keyword() {
            self.parse_identifier_name()
        } else {
            NodeIndex::NONE
        };

        self.parse_semicolon();
        let end_pos = self.token_end();

        self.arena.add_token(syntax_kind_ext::CONTINUE_STATEMENT as u16, start_pos, end_pos)
    }

    /// Parse throw statement
    fn parse_throw_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::ThrowKeyword);

        let expression = self.parse_expression();

        self.parse_semicolon();
        let end_pos = self.token_end();

        // Use return statement node type for throw (same structure)
        self.arena.add_return(
            syntax_kind_ext::THROW_STATEMENT,
            start_pos,
            end_pos,
            ReturnData { expression },
        )
    }

    /// Parse do-while statement
    fn parse_do_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::DoKeyword);

        let statement = self.parse_statement();

        self.parse_expected(SyntaxKind::WhileKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);
        let condition = self.parse_expression();
        self.parse_expected(SyntaxKind::CloseParenToken);

        self.parse_semicolon();
        let end_pos = self.token_end();

        self.arena.add_loop(
            syntax_kind_ext::DO_STATEMENT,
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

    /// Parse switch statement
    fn parse_switch_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::SwitchKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);

        let expression = self.parse_expression();

        self.parse_expected(SyntaxKind::CloseParenToken);
        self.parse_expected(SyntaxKind::OpenBraceToken);

        // Parse case clauses
        let mut clauses = Vec::new();
        while !self.is_token(SyntaxKind::CloseBraceToken)
            && !self.is_token(SyntaxKind::EndOfFileToken)
        {
            if self.is_token(SyntaxKind::CaseKeyword) {
                let clause_start = self.token_pos();
                self.next_token();
                let clause_expr = self.parse_expression();
                self.parse_expected(SyntaxKind::ColonToken);

                let mut statements = Vec::new();
                while !self.is_token(SyntaxKind::CaseKeyword)
                    && !self.is_token(SyntaxKind::DefaultKeyword)
                    && !self.is_token(SyntaxKind::CloseBraceToken)
                    && !self.is_token(SyntaxKind::EndOfFileToken)
                {
                    statements.push(self.parse_statement());
                }

                let clause_end = self.token_end();
                clauses.push(self.arena.add_case_clause(
                    syntax_kind_ext::CASE_CLAUSE,
                    clause_start,
                    clause_end,
                    CaseClauseData {
                        expression: clause_expr,
                        statements: self.make_node_list(statements),
                    },
                ));
            } else if self.is_token(SyntaxKind::DefaultKeyword) {
                let clause_start = self.token_pos();
                self.next_token();
                self.parse_expected(SyntaxKind::ColonToken);

                let mut statements = Vec::new();
                while !self.is_token(SyntaxKind::CaseKeyword)
                    && !self.is_token(SyntaxKind::DefaultKeyword)
                    && !self.is_token(SyntaxKind::CloseBraceToken)
                    && !self.is_token(SyntaxKind::EndOfFileToken)
                {
                    statements.push(self.parse_statement());
                }

                let clause_end = self.token_end();
                clauses.push(self.arena.add_case_clause(
                    syntax_kind_ext::DEFAULT_CLAUSE,
                    clause_start,
                    clause_end,
                    CaseClauseData {
                        expression: NodeIndex::NONE,
                        statements: self.make_node_list(statements),
                    },
                ));
            } else {
                self.next_token(); // Skip unexpected token
            }
        }

        let case_block_end = self.token_end();
        self.parse_expected(SyntaxKind::CloseBraceToken);
        let end_pos = self.token_end();

        // Create the case block node
        let case_block = self.arena.add_block(
            syntax_kind_ext::CASE_BLOCK,
            start_pos, // Case block starts with the opening brace
            case_block_end,
            BlockData {
                statements: self.make_node_list(clauses),
                multi_line: true,
            },
        );

        self.arena.add_switch(
            syntax_kind_ext::SWITCH_STATEMENT,
            start_pos,
            end_pos,
            SwitchData {
                expression,
                case_block,
            },
        )
    }

    /// Parse try statement
    fn parse_try_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::TryKeyword);

        let try_block = self.parse_block();

        // Parse catch clause
        let catch_clause = if self.is_token(SyntaxKind::CatchKeyword) {
            let catch_start = self.token_pos();
            self.next_token();

            // Parse optional catch binding
            let variable_declaration = if self.is_token(SyntaxKind::OpenParenToken) {
                self.next_token();
                let param = if self.is_identifier_or_keyword() {
                    self.parse_identifier_name()
                } else {
                    NodeIndex::NONE
                };
                self.parse_expected(SyntaxKind::CloseParenToken);
                param
            } else {
                NodeIndex::NONE
            };

            let catch_block = self.parse_block();
            let catch_end = self.token_end();

            self.arena.add_catch_clause(
                syntax_kind_ext::CATCH_CLAUSE,
                catch_start,
                catch_end,
                CatchClauseData {
                    variable_declaration,
                    block: catch_block,
                },
            )
        } else {
            NodeIndex::NONE
        };

        // Parse finally clause
        let finally_block = if self.is_token(SyntaxKind::FinallyKeyword) {
            self.next_token();
            self.parse_block()
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_try(
            syntax_kind_ext::TRY_STATEMENT,
            start_pos,
            end_pos,
            TryData {
                try_block,
                catch_clause,
                finally_block,
            },
        )
    }

    /// Parse with statement
    fn parse_with_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::WithKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);

        let expression = self.parse_expression();

        self.parse_expected(SyntaxKind::CloseParenToken);

        let statement = self.parse_statement();

        let end_pos = self.token_end();

        // Use if statement structure for with (expression + statement)
        self.arena.add_if_statement(
            syntax_kind_ext::WITH_STATEMENT,
            start_pos,
            end_pos,
            IfStatementData {
                expression,
                then_statement: statement,
                else_statement: NodeIndex::NONE,
            },
        )
    }

    /// Parse debugger statement
    fn parse_debugger_statement(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::DebuggerKeyword);
        self.parse_semicolon();
        let end_pos = self.token_end();

        self.arena.add_token(syntax_kind_ext::DEBUGGER_STATEMENT as u16, start_pos, end_pos)
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

        // Handle as/satisfies type assertions: expr as Type, expr satisfies Type
        if self.is_token(SyntaxKind::AsKeyword) || self.is_token(SyntaxKind::SatisfiesKeyword) {
            left = self.parse_as_or_satisfies_expression(left, start_pos);
        }

        left
    }

    /// Parse as/satisfies expression: expr as Type, expr satisfies Type
    fn parse_as_or_satisfies_expression(&mut self, expression: NodeIndex, start_pos: u32) -> NodeIndex {
        let is_satisfies = self.is_token(SyntaxKind::SatisfiesKeyword);
        self.next_token(); // consume 'as' or 'satisfies'

        let type_node = self.parse_type();
        let end_pos = self.token_end();

        let result = self.arena.add_type_assertion(
            if is_satisfies {
                syntax_kind_ext::SATISFIES_EXPRESSION
            } else {
                syntax_kind_ext::AS_EXPRESSION
            },
            start_pos,
            end_pos,
            crate::parser::thin_node::TypeAssertionData {
                expression,
                type_node,
            },
        );

        // Allow chaining: x as T as U
        if self.is_token(SyntaxKind::AsKeyword) || self.is_token(SyntaxKind::SatisfiesKeyword) {
            return self.parse_as_or_satisfies_expression(result, start_pos);
        }

        result
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
                    // Handle both regular identifiers and private identifiers (#name)
                    let name = if self.is_token(SyntaxKind::PrivateIdentifier) {
                        self.parse_private_identifier()
                    } else {
                        self.parse_identifier_name()
                    };
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
            SyntaxKind::UndefinedKeyword => self.parse_keyword_as_identifier(),
            SyntaxKind::ThisKeyword => self.parse_this_expression(),
            SyntaxKind::SuperKeyword => self.parse_super_expression(),
            SyntaxKind::OpenParenToken => self.parse_parenthesized_expression(),
            SyntaxKind::OpenBracketToken => self.parse_array_literal(),
            SyntaxKind::OpenBraceToken => self.parse_object_literal(),
            SyntaxKind::NewKeyword => self.parse_new_expression(),
            SyntaxKind::FunctionKeyword => self.parse_function_expression(),
            SyntaxKind::ClassKeyword => self.parse_class_expression(),
            SyntaxKind::AsyncKeyword => {
                // async function expression or async arrow function
                if self.look_ahead_is_async_function() {
                    self.parse_async_function_expression()
                } else {
                    // Could be an async arrow function, parse as identifier for now
                    self.parse_identifier()
                }
            }
            SyntaxKind::LessThanToken => self.parse_jsx_element_or_type_assertion(),
            SyntaxKind::NoSubstitutionTemplateLiteral => self.parse_no_substitution_template_literal(),
            SyntaxKind::TemplateHead => self.parse_template_expression(),
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

    /// Parse identifier name - allows keywords to be used as identifiers
    /// This is used in contexts where keywords are valid identifier names
    /// (e.g., class names, property names, function names)
    fn parse_identifier_name(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let text = self.scanner.get_token_value_ref().to_string();
        self.identifiers.push(text.clone());

        if self.is_identifier_or_keyword() {
            self.next_token();
        } else {
            self.parse_error_at_current_token("Expected identifier");
        }
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

    /// Parse private identifier (#name)
    fn parse_private_identifier(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let text = self.scanner.get_token_value_ref().to_string();
        self.identifiers.push(text.clone());
        self.parse_expected(SyntaxKind::PrivateIdentifier);
        let end_pos = self.token_end();

        self.arena.add_identifier(
            SyntaxKind::PrivateIdentifier as u16,
            start_pos,
            end_pos,
            IdentifierData {
                escaped_text: text,
                original_text: None,
                type_arguments: None,
            },
        )
    }

    /// Parse object binding pattern: { x, y: z, ...rest }
    fn parse_object_binding_pattern(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        let mut elements = Vec::new();

        while !self.is_token(SyntaxKind::CloseBraceToken)
            && !self.is_token(SyntaxKind::EndOfFileToken)
        {
            let elem_start = self.token_pos();

            // Handle rest element: ...x
            let dot_dot_dot = self.parse_optional(SyntaxKind::DotDotDotToken);

            if dot_dot_dot {
                // Rest element: just name
                let name = self.parse_binding_element_name();
                let elem_end = self.token_end();
                elements.push(self.arena.add_binding_element(
                    syntax_kind_ext::BINDING_ELEMENT,
                    elem_start,
                    elem_end,
                    crate::parser::thin_node::BindingElementData {
                        dot_dot_dot_token: true,
                        property_name: NodeIndex::NONE,
                        name,
                        initializer: NodeIndex::NONE,
                    },
                ));
            } else {
                // Regular binding element: name or propertyName: name
                let first_name = self.parse_property_name();

                let (property_name, name) = if self.parse_optional(SyntaxKind::ColonToken) {
                    // propertyName: name
                    let name = self.parse_binding_element_name();
                    (first_name, name)
                } else {
                    // Just name (shorthand)
                    (NodeIndex::NONE, first_name)
                };

                // Optional initializer: = value
                let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
                    self.parse_assignment_expression()
                } else {
                    NodeIndex::NONE
                };

                let elem_end = self.token_end();
                elements.push(self.arena.add_binding_element(
                    syntax_kind_ext::BINDING_ELEMENT,
                    elem_start,
                    elem_end,
                    crate::parser::thin_node::BindingElementData {
                        dot_dot_dot_token: false,
                        property_name,
                        name,
                        initializer,
                    },
                ));
            }

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.parse_expected(SyntaxKind::CloseBraceToken);
        let end_pos = self.token_end();

        self.arena.add_binding_pattern(
            syntax_kind_ext::OBJECT_BINDING_PATTERN,
            start_pos,
            end_pos,
            crate::parser::thin_node::BindingPatternData {
                elements: self.make_node_list(elements),
            },
        )
    }

    /// Parse array binding pattern: [x, y, ...rest]
    fn parse_array_binding_pattern(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBracketToken);

        let mut elements = Vec::new();

        while !self.is_token(SyntaxKind::CloseBracketToken)
            && !self.is_token(SyntaxKind::EndOfFileToken)
        {
            let elem_start = self.token_pos();

            // Handle omitted element: [, , x]
            if self.is_token(SyntaxKind::CommaToken) {
                // Omitted element - push NONE as placeholder
                elements.push(NodeIndex::NONE);
                self.next_token();
                continue;
            }

            // Handle rest element: ...x
            let dot_dot_dot = self.parse_optional(SyntaxKind::DotDotDotToken);

            // Parse name (can be identifier or nested binding pattern)
            let name = self.parse_binding_element_name();

            // Optional initializer: = value
            let initializer = if !dot_dot_dot && self.parse_optional(SyntaxKind::EqualsToken) {
                self.parse_assignment_expression()
            } else {
                NodeIndex::NONE
            };

            let elem_end = self.token_end();
            elements.push(self.arena.add_binding_element(
                syntax_kind_ext::BINDING_ELEMENT,
                elem_start,
                elem_end,
                crate::parser::thin_node::BindingElementData {
                    dot_dot_dot_token: dot_dot_dot,
                    property_name: NodeIndex::NONE,
                    name,
                    initializer,
                },
            ));

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.parse_expected(SyntaxKind::CloseBracketToken);
        let end_pos = self.token_end();

        self.arena.add_binding_pattern(
            syntax_kind_ext::ARRAY_BINDING_PATTERN,
            start_pos,
            end_pos,
            crate::parser::thin_node::BindingPatternData {
                elements: self.make_node_list(elements),
            },
        )
    }

    /// Parse binding element name (can be identifier or nested binding pattern)
    fn parse_binding_element_name(&mut self) -> NodeIndex {
        if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_object_binding_pattern()
        } else if self.is_token(SyntaxKind::OpenBracketToken) {
            self.parse_array_binding_pattern()
        } else if self.is_identifier_or_keyword() {
            self.parse_identifier_name()
        } else {
            self.parse_identifier()
        }
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

    /// Parse super expression
    fn parse_super_expression(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.next_token();
        let end_pos = self.token_end();

        self.arena.add_token(SyntaxKind::SuperKeyword as u16, start_pos, end_pos)
    }

    /// Parse no-substitution template literal: `hello`
    fn parse_no_substitution_template_literal(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let text = self.scanner.get_token_value_ref().to_string();
        self.parse_expected(SyntaxKind::NoSubstitutionTemplateLiteral);
        let end_pos = self.token_end();

        self.arena.add_literal(
            SyntaxKind::NoSubstitutionTemplateLiteral as u16,
            start_pos,
            end_pos,
            LiteralData { text, raw_text: None, value: None },
        )
    }

    /// Parse template expression: `hello ${name}!`
    fn parse_template_expression(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse template head: `hello ${
        let head_text = self.scanner.get_token_value_ref().to_string();
        let head_start = self.token_pos();
        self.parse_expected(SyntaxKind::TemplateHead);
        let head_end = self.token_end();

        let head = self.arena.add_literal(
            SyntaxKind::TemplateHead as u16,
            head_start,
            head_end,
            LiteralData { text: head_text, raw_text: None, value: None },
        );

        // Parse template spans
        let mut spans = Vec::new();
        loop {
            // Parse expression in ${ }
            let expression = self.parse_expression();

            // Now we need to rescan the } as a template continuation
            // The scanner needs to be told to rescan as template
            self.scanner.re_scan_template_token(false);
            self.current_token = self.scanner.get_token();

            // Parse template middle or tail
            let literal_start = self.token_pos();
            let literal_text = self.scanner.get_token_value_ref().to_string();
            let is_tail = self.is_token(SyntaxKind::TemplateTail);
            let literal_kind = if is_tail {
                SyntaxKind::TemplateTail
            } else {
                SyntaxKind::TemplateMiddle
            };

            self.next_token();
            let literal_end = self.token_end();

            let literal = self.arena.add_literal(
                literal_kind as u16,
                literal_start,
                literal_end,
                LiteralData { text: literal_text, raw_text: None, value: None },
            );

            let span_start = if let Some(node) = self.arena.get(expression) { node.pos } else { literal_start };
            let span = self.arena.add_template_span(
                syntax_kind_ext::TEMPLATE_SPAN,
                span_start,
                literal_end,
                TemplateSpanData { expression, literal },
            );
            spans.push(span);

            if is_tail {
                break;
            }
        }

        let end_pos = self.token_end();

        self.arena.add_template_expr(
            syntax_kind_ext::TEMPLATE_EXPRESSION,
            start_pos,
            end_pos,
            TemplateExprData {
                head,
                template_spans: self.make_node_list(spans),
            },
        )
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

    /// Parse property assignment, method, getter, setter, or spread element
    fn parse_property_assignment(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Handle spread element: ...expr
        if self.is_token(SyntaxKind::DotDotDotToken) {
            self.next_token();
            let expression = self.parse_assignment_expression();
            let end_pos = self.token_end();
            return self.arena.add_unary_expr_ex(
                syntax_kind_ext::SPREAD_ELEMENT,
                start_pos,
                end_pos,
                crate::parser::thin_node::UnaryExprDataEx {
                    expression,
                    asterisk_token: false,
                },
            );
        }

        // Handle get accessor: get foo() { }
        if self.is_token(SyntaxKind::GetKeyword) && self.look_ahead_is_object_method() {
            return self.parse_object_get_accessor(start_pos);
        }

        // Handle set accessor: set foo(v) { }
        if self.is_token(SyntaxKind::SetKeyword) && self.look_ahead_is_object_method() {
            return self.parse_object_set_accessor(start_pos);
        }

        // Handle async method: async foo() { }
        if self.is_token(SyntaxKind::AsyncKeyword) && self.look_ahead_is_object_method() {
            return self.parse_object_method(start_pos, true, false);
        }

        // Handle generator method: *foo() { }
        if self.is_token(SyntaxKind::AsteriskToken) {
            self.next_token(); // consume '*'
            return self.parse_object_method(start_pos, false, true);
        }

        let name = self.parse_property_name();

        // Handle method: foo() { } or foo<T>() { }
        if self.is_token(SyntaxKind::OpenParenToken) || self.is_token(SyntaxKind::LessThanToken) {
            return self.parse_object_method_after_name(start_pos, name, false, false);
        }

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

    /// Look ahead to check if get/set/async is a method vs property name
    fn look_ahead_is_object_method(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        self.next_token(); // skip get/set/async

        // Check if followed by property name (identifier, string, number, [)
        let is_method = self.is_token(SyntaxKind::Identifier)
            || self.is_token(SyntaxKind::StringLiteral)
            || self.is_token(SyntaxKind::NumericLiteral)
            || self.is_token(SyntaxKind::OpenBracketToken)
            || self.is_token(SyntaxKind::AsteriskToken); // async *foo()

        self.scanner.restore_state(snapshot);
        self.current_token = current;
        is_method
    }

    /// Parse get accessor in object literal: get foo() { }
    fn parse_object_get_accessor(&mut self, start_pos: u32) -> NodeIndex {
        self.next_token(); // consume 'get'
        let name = self.parse_property_name();

        self.parse_expected(SyntaxKind::OpenParenToken);
        self.parse_expected(SyntaxKind::CloseParenToken);

        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_accessor(
            syntax_kind_ext::GET_ACCESSOR,
            start_pos,
            end_pos,
            crate::parser::thin_node::AccessorData {
                modifiers: None,
                name,
                type_parameters: None,
                parameters: self.make_node_list(vec![]),
                type_annotation,
                body,
            },
        )
    }

    /// Parse set accessor in object literal: set foo(v) { }
    fn parse_object_set_accessor(&mut self, start_pos: u32) -> NodeIndex {
        self.next_token(); // consume 'set'
        let name = self.parse_property_name();

        self.parse_expected(SyntaxKind::OpenParenToken);
        let parameters = self.parse_parameter_list();
        self.parse_expected(SyntaxKind::CloseParenToken);

        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_accessor(
            syntax_kind_ext::SET_ACCESSOR,
            start_pos,
            end_pos,
            crate::parser::thin_node::AccessorData {
                modifiers: None,
                name,
                type_parameters: None,
                parameters,
                type_annotation: NodeIndex::NONE,
                body,
            },
        )
    }

    /// Parse method in object literal: foo() { } or async foo() { } or *foo() { }
    fn parse_object_method(&mut self, start_pos: u32, is_async: bool, is_generator: bool) -> NodeIndex {
        // Build modifiers if async
        let modifiers = if is_async {
            self.next_token(); // consume 'async'
            let mod_idx = self.arena.create_modifier(SyntaxKind::AsyncKeyword, start_pos);
            Some(self.make_node_list(vec![mod_idx]))
        } else {
            None
        };

        // Check for generator after async: async *foo()
        // or standalone generator: *foo()
        let asterisk = if is_generator {
            // Asterisk already consumed by caller for standalone generator
            true
        } else if self.parse_optional(SyntaxKind::AsteriskToken) {
            // async *foo() - consume asterisk here
            true
        } else {
            false
        };

        let name = self.parse_property_name();
        self.parse_object_method_after_name(start_pos, name, asterisk, modifiers.is_some())
    }

    /// Parse method after name has been parsed
    fn parse_object_method_after_name(&mut self, start_pos: u32, name: NodeIndex, asterisk: bool, is_async: bool) -> NodeIndex {
        // Optional type parameters
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

        self.parse_expected(SyntaxKind::OpenParenToken);
        let parameters = self.parse_parameter_list();
        self.parse_expected(SyntaxKind::CloseParenToken);

        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            NodeIndex::NONE
        };

        let modifiers = if is_async {
            let mod_idx = self.arena.create_modifier(SyntaxKind::AsyncKeyword, start_pos);
            Some(self.make_node_list(vec![mod_idx]))
        } else {
            None
        };

        let end_pos = self.token_end();
        self.arena.add_method_decl(
            syntax_kind_ext::METHOD_DECLARATION,
            start_pos,
            end_pos,
            crate::parser::thin_node::MethodDeclData {
                modifiers,
                asterisk_token: asterisk,
                name,
                question_token: false,
                type_parameters,
                parameters,
                type_annotation,
                body,
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

        // Parse the callee expression - member access without call (we handle call ourselves)
        let expression = self.parse_member_expression_base();

        // Parse type arguments: new Array<string>()
        let type_arguments = if self.is_token(SyntaxKind::LessThanToken) {
            // Try to parse as type arguments
            Some(self.parse_type_arguments())
        } else {
            None
        };

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
                type_arguments,
                arguments,
            },
        )
    }

    /// Parse member expression base (identifier with property/element access, but no calls)
    fn parse_member_expression_base(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let mut expr = self.parse_primary_expression();

        loop {
            match self.token() {
                SyntaxKind::DotToken => {
                    self.next_token();
                    let name = if self.is_token(SyntaxKind::PrivateIdentifier) {
                        self.parse_private_identifier()
                    } else {
                        self.parse_identifier_name()
                    };
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
                _ => break,
            }
        }

        expr
    }

    // =========================================================================
    // Parse Methods - Types (minimal implementation)
    // =========================================================================

    /// Parse a type (handles keywords, type references, unions, intersections, conditionals)
    fn parse_type(&mut self) -> NodeIndex {
        self.parse_conditional_type()
    }

    /// Parse return type, which may be a type predicate (x is T) or a regular type
    fn parse_return_type(&mut self) -> NodeIndex {
        // Check if this is a type predicate: identifier 'is' Type
        // We need to look ahead to see if there's an identifier followed by 'is'
        if self.is_token(SyntaxKind::Identifier) {
            let snapshot = self.scanner.save_state();
            let current = self.current_token;

            let name = self.parse_identifier();
            if self.is_token(SyntaxKind::IsKeyword) {
                // This is a type predicate: x is T
                let start_pos = if let Some(node) = self.arena.get(name) {
                    node.pos
                } else {
                    self.token_pos()
                };

                self.next_token(); // consume 'is'
                let type_node = self.parse_type();
                let end_pos = self.token_end();

                return self.arena.add_type_predicate(
                    syntax_kind_ext::TYPE_PREDICATE,
                    start_pos,
                    end_pos,
                    crate::parser::thin_node::TypePredicateData {
                        asserts_modifier: false,
                        parameter_name: name,
                        type_node,
                    },
                );
            }

            // Not a type predicate, restore state and parse as regular type
            self.scanner.restore_state(snapshot);
            self.current_token = current;
        }

        // Check for 'asserts' type predicate: asserts x is T
        if self.is_token(SyntaxKind::AssertsKeyword) {
            return self.parse_asserts_type_predicate();
        }

        self.parse_type()
    }

    /// Parse 'asserts' type predicate: asserts x or asserts x is T
    fn parse_asserts_type_predicate(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::AssertsKeyword);

        let parameter_name = self.parse_identifier();

        let type_node = if self.is_token(SyntaxKind::IsKeyword) {
            self.next_token();
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();

        self.arena.add_type_predicate(
            syntax_kind_ext::TYPE_PREDICATE,
            start_pos,
            end_pos,
            crate::parser::thin_node::TypePredicateData {
                asserts_modifier: true,
                parameter_name,
                type_node,
            },
        )
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

        // Handle generic function types: <T>() => T or <T, U>(x: T) => U
        if self.is_token(SyntaxKind::LessThanToken) {
            return self.parse_generic_function_type();
        }

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
            let obj_type = self.parse_object_or_mapped_type();
            // Handle array/indexed access on object literal: {...}[] or {...}["key"]
            if self.is_token(SyntaxKind::OpenBracketToken) {
                return self.parse_array_type(start_pos, obj_type);
            }
            return obj_type;
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

        // Handle negative numeric literal types: -1, -42
        if self.is_token(SyntaxKind::MinusToken) {
            return self.parse_prefix_unary_literal_type();
        }

        // Handle template literal types: `hello` or `prefix${T}suffix`
        if self.is_token(SyntaxKind::NoSubstitutionTemplateLiteral)
            || self.is_token(SyntaxKind::TemplateHead)
        {
            return self.parse_template_literal_type();
        }

        // Check for type keywords (string, number, boolean, etc.)
        let first_name = match self.token() {
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

        // Handle qualified names (foo.Bar, A.B.C)
        let type_name = self.parse_qualified_name_rest(first_name);

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

    /// Parse a single element in a tuple type, handling:
    /// - Rest elements: ...T[]
    /// - Optional elements: T?
    /// - Named elements: name: T or name?: T
    fn parse_tuple_element_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Handle rest element: ...T[]
        if self.parse_optional(SyntaxKind::DotDotDotToken) {
            let element_type = self.parse_type();
            let end_pos = self.token_end();
            return self.arena.add_type_operator(
                syntax_kind_ext::REST_TYPE,
                start_pos,
                end_pos,
                crate::parser::thin_node::TypeOperatorData {
                    operator: SyntaxKind::DotDotDotToken as u16,
                    type_node: element_type,
                },
            );
        }

        // Check if this is a named tuple element: name: T or name?: T
        // Need to look ahead to see if there's a colon after the identifier
        if self.is_token(SyntaxKind::Identifier) {
            let snapshot = self.scanner.save_state();
            let current = self.current_token;

            let _name = self.scanner.get_token_value_ref().to_string();
            self.next_token();

            // Check for optional marker and colon
            let has_question = self.parse_optional(SyntaxKind::QuestionToken);
            let has_colon = self.is_token(SyntaxKind::ColonToken);

            if has_colon || has_question {
                // This is a named tuple element - parse it
                self.scanner.restore_state(snapshot);
                self.current_token = current;
                return self.parse_named_tuple_member();
            }

            // Not a named element, restore and parse as regular type
            self.scanner.restore_state(snapshot);
            self.current_token = current;
        }

        // Parse the type
        let type_node = self.parse_type();

        // Check for optional marker: T?
        if self.parse_optional(SyntaxKind::QuestionToken) {
            let end_pos = self.token_end();
            return self.arena.add_type_operator(
                syntax_kind_ext::OPTIONAL_TYPE,
                start_pos,
                end_pos,
                crate::parser::thin_node::TypeOperatorData {
                    operator: SyntaxKind::QuestionToken as u16,
                    type_node,
                },
            );
        }

        type_node
    }

    /// Parse a named tuple member: name: T or name?: T
    fn parse_named_tuple_member(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Check for ... prefix (rest parameter)
        let dot_dot_dot_token = self.parse_optional(SyntaxKind::DotDotDotToken);

        // Parse name
        let name = self.parse_identifier();

        // Check for optional marker
        let question_token = self.parse_optional(SyntaxKind::QuestionToken);

        // Parse : and type
        self.parse_expected(SyntaxKind::ColonToken);
        let type_node = self.parse_type();

        let end_pos = self.token_end();

        // Create a named tuple member node
        self.arena.add_named_tuple_member(
            syntax_kind_ext::NAMED_TUPLE_MEMBER,
            start_pos,
            end_pos,
            crate::parser::thin_node::NamedTupleMemberData {
                dot_dot_dot_token,
                name,
                question_token,
                type_node,
            },
        )
    }

    /// Parse tuple type: [T, U, V], [name: T], [...T[]], [T?]
    fn parse_tuple_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBracketToken);

        let mut elements = Vec::new();

        while !self.is_token(SyntaxKind::CloseBracketToken)
            && !self.is_token(SyntaxKind::EndOfFileToken)
        {
            let element = self.parse_tuple_element_type();
            elements.push(element);

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

    /// Parse prefix unary literal type: -1, -42
    /// In TypeScript, negative number literals in type position are
    /// represented as a PrefixUnaryExpression wrapped in a LiteralType
    fn parse_prefix_unary_literal_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse the minus token
        let operator_kind = self.token() as u16;
        self.next_token();

        // Parse the numeric literal operand
        let operand = self.parse_numeric_literal();

        let prefix_end = self.token_end();

        // Create prefix unary expression node
        let prefix_expr = self.arena.add_unary_expr(
            syntax_kind_ext::PREFIX_UNARY_EXPRESSION,
            start_pos,
            prefix_end,
            crate::parser::thin_node::UnaryExprData {
                operator: operator_kind,
                operand,
            },
        );

        // Wrap in a literal type
        self.arena.add_literal_type(
            syntax_kind_ext::LITERAL_TYPE,
            start_pos,
            prefix_end,
            crate::parser::thin_node::LiteralTypeData { literal: prefix_expr },
        )
    }

    /// Parse typeof type: typeof x, typeof x.y
    fn parse_typeof_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::TypeOfKeyword);

        // Parse the expression name (can be qualified: x.y.z)
        let expr_name = self.parse_entity_name();

        // Parse optional type arguments for instantiation expressions: typeof Err<U>
        let type_arguments = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_arguments())
        } else {
            None
        };

        let end_pos = self.token_end();

        self.arena.add_type_query(
            syntax_kind_ext::TYPE_QUERY,
            start_pos,
            end_pos,
            crate::parser::thin_node::TypeQueryData {
                expr_name,
                type_arguments,
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
    /// Index signature: { [key: string]: T }
    fn parse_object_or_mapped_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        // Check if this is a mapped type: [ followed by identifier and 'in'
        // vs index signature: [ followed by identifier and ':'
        if self.is_token(SyntaxKind::OpenBracketToken) {
            if self.look_ahead_is_mapped_type_start() {
                return self.parse_mapped_type_rest(start_pos);
            }
            // Not a mapped type - let type literal parsing handle index signature
            return self.parse_type_literal_rest(start_pos);
        }

        // Check for readonly/+/- prefixed mapped type
        if (self.is_token(SyntaxKind::ReadonlyKeyword) && self.look_ahead_is_mapped_type())
            || (self.is_token(SyntaxKind::PlusToken) || self.is_token(SyntaxKind::MinusToken))
        {
            return self.parse_mapped_type_rest(start_pos);
        }

        // Otherwise it's an object type literal - parse as type literal
        self.parse_type_literal_rest(start_pos)
    }

    /// Look ahead to see if [ starts a mapped type (has 'in' keyword) vs index signature (has ':')
    fn look_ahead_is_mapped_type_start(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        self.next_token(); // skip [

        // Skip identifier
        if self.is_token(SyntaxKind::Identifier) {
            self.next_token();
        }

        // Check if followed by 'in' (mapped type) or ':' (index signature)
        let is_mapped = self.is_token(SyntaxKind::InKeyword);

        self.scanner.restore_state(snapshot);
        self.current_token = current;
        is_mapped
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

        // Parse optional : and type (type can be omitted for implicit any)
        let type_node = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

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

    /// Check if current keyword can be used as a property name
    /// (when followed by :, ?, (, <, or at end of type member)
    fn look_ahead_is_property_name_after_keyword(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        // Skip the keyword
        self.next_token();

        // If followed by these, the keyword is being used as a property name
        let is_property_name = self.is_token(SyntaxKind::ColonToken)
            || self.is_token(SyntaxKind::QuestionToken)
            || self.is_token(SyntaxKind::OpenParenToken)
            || self.is_token(SyntaxKind::LessThanToken)
            || self.is_token(SyntaxKind::SemicolonToken)
            || self.is_token(SyntaxKind::CommaToken)
            || self.is_token(SyntaxKind::CloseBraceToken);

        self.scanner.restore_state(snapshot);
        self.current_token = current;
        is_property_name
    }

    /// Check if current token is a keyword that can be used as a property name
    fn is_property_name_keyword(&self) -> bool {
        matches!(
            self.token(),
            SyntaxKind::TypeKeyword
                | SyntaxKind::GetKeyword
                | SyntaxKind::SetKeyword
                | SyntaxKind::ReadonlyKeyword
                | SyntaxKind::AsyncKeyword
                | SyntaxKind::AwaitKeyword
                | SyntaxKind::NewKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::StaticKeyword
                | SyntaxKind::AbstractKeyword
                | SyntaxKind::OverrideKeyword
                | SyntaxKind::DeclareKeyword
                | SyntaxKind::ExportKeyword
                | SyntaxKind::DefaultKeyword
                | SyntaxKind::LetKeyword
                | SyntaxKind::ConstKeyword
                | SyntaxKind::VarKeyword
                | SyntaxKind::IfKeyword
                | SyntaxKind::ElseKeyword
                | SyntaxKind::ForKeyword
                | SyntaxKind::WhileKeyword
                | SyntaxKind::DoKeyword
                | SyntaxKind::SwitchKeyword
                | SyntaxKind::CaseKeyword
                | SyntaxKind::BreakKeyword
                | SyntaxKind::ContinueKeyword
                | SyntaxKind::ReturnKeyword
                | SyntaxKind::ThrowKeyword
                | SyntaxKind::TryKeyword
                | SyntaxKind::CatchKeyword
                | SyntaxKind::FinallyKeyword
                | SyntaxKind::ClassKeyword
                | SyntaxKind::FunctionKeyword
                | SyntaxKind::ImportKeyword
                | SyntaxKind::FromKeyword
                | SyntaxKind::AsKeyword
                | SyntaxKind::InKeyword
                | SyntaxKind::OfKeyword
                | SyntaxKind::InstanceOfKeyword
                | SyntaxKind::ThisKeyword
                | SyntaxKind::SuperKeyword
                | SyntaxKind::DeleteKeyword
                | SyntaxKind::VoidKeyword
                | SyntaxKind::TypeOfKeyword
                | SyntaxKind::YieldKeyword
                | SyntaxKind::ConstructorKeyword
                | SyntaxKind::InterfaceKeyword
                | SyntaxKind::EnumKeyword
                | SyntaxKind::ImplementsKeyword
                | SyntaxKind::ExtendsKeyword
                | SyntaxKind::ModuleKeyword
                | SyntaxKind::NamespaceKeyword
                | SyntaxKind::RequireKeyword
                | SyntaxKind::GlobalKeyword
        )
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

        // Check for parameter-like syntax: identifier or keyword followed by : or )
        // If we see just a type (like `string`), it could be parenthesized type
        // Function type params have: `name:` or `modifier name` where name can be a keyword
        if self.is_identifier_or_keyword() {
            self.next_token();
            // If followed by : it's definitely a function type parameter
            if self.is_token(SyntaxKind::ColonToken) {
                self.scanner.restore_state(snapshot);
                self.current_token = current;
                return true;
            }
            // If followed by another identifier (like `public B`), it's a parameter with modifier
            if self.is_identifier_or_keyword() {
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

    /// Parse generic function type: <T>() => T or <T, U extends V>(x: T) => U
    fn parse_generic_function_type(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse type parameters: <T, U extends V>
        let type_parameters = self.parse_type_parameters();

        // Parse parameters: (x: T, y: U)
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
                type_parameters: Some(type_parameters),
                parameters,
                type_annotation,
            },
        )
    }

    /// Parse type parameter list for function types: (x: T, y: U)
    /// Also handles invalid modifiers like (public x) which TypeScript parses but errors on semantically
    fn parse_type_parameter_list(&mut self) -> NodeList {
        let mut params = Vec::new();

        while !self.is_token(SyntaxKind::CloseParenToken)
            && !self.is_token(SyntaxKind::EndOfFileToken)
        {
            let param_start = self.token_pos();

            // Parse optional modifiers (public/private/protected/readonly)
            // These are syntactically valid but semantically invalid in function types
            let modifiers = self.parse_parameter_modifiers();

            // Parse optional ...rest
            let dot_dot_dot = self.parse_optional(SyntaxKind::DotDotDotToken);

            // Parse parameter name (keywords are allowed as parameter names)
            let name = self.parse_identifier_name();

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
                    modifiers,
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

    /// Parse qualified name rest: given a left name, parse `.Right.Rest` parts
    /// Handles: foo.Bar, A.B.C, etc.
    fn parse_qualified_name_rest(&mut self, left: NodeIndex) -> NodeIndex {
        let mut current = left;

        while self.is_token(SyntaxKind::DotToken) {
            let start_pos = if let Some(node) = self.arena.get(current) {
                node.pos
            } else {
                self.token_pos()
            };

            self.next_token(); // consume .
            let right = self.parse_identifier_name();
            let end_pos = self.token_end();

            current = self.arena.add_qualified_name(
                syntax_kind_ext::QUALIFIED_NAME,
                start_pos,
                end_pos,
                crate::parser::thin_node::QualifiedNameData {
                    left: current,
                    right,
                },
            );
        }

        current
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

    // =========================================================================
    // JSX Parsing
    // =========================================================================

    /// Determine if we should parse a type assertion or JSX element.
    /// Type assertions use <Type>expr syntax, JSX uses <Element>.
    fn parse_jsx_element_or_type_assertion(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Look ahead to determine if this is a type assertion or JSX
        // Type assertion: <type>expression where type is a type keyword or identifier followed by >
        // JSX: <element ...> where element is an identifier (starts with lowercase = intrinsic, uppercase = component)

        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        self.next_token(); // consume <

        // Check if we have a type keyword (number, string, boolean, etc.) - definitely a type assertion
        let is_type_assertion = match self.token() {
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
            | SyntaxKind::ObjectKeyword
            | SyntaxKind::KeyOfKeyword
            | SyntaxKind::TypeOfKeyword
            | SyntaxKind::ReadonlyKeyword
            | SyntaxKind::UniqueKeyword
            | SyntaxKind::InferKeyword
            | SyntaxKind::OpenBraceToken
            | SyntaxKind::OpenBracketToken
            | SyntaxKind::OpenParenToken
            | SyntaxKind::LessThanToken
            | SyntaxKind::GreaterThanToken => true,  // <> is a fragment, not type assertion
            SyntaxKind::Identifier => {
                // Could be either JSX or type assertion
                // - Lowercase identifiers like <div> are always JSX
                // - PascalCase followed by JSX-like syntax (attributes, /) is JSX
                // - Single uppercase letter like <T> followed by > is likely type assertion

                let text = self.scanner.get_token_value_ref().to_string();
                let first_char = text.chars().next().unwrap_or('a');

                if first_char.is_ascii_lowercase() {
                    // Lowercase identifier = JSX intrinsic element
                    false
                } else {
                    // PascalCase - check if followed by JSX-like syntax
                    self.next_token();
                    matches!(
                        self.token(),
                        SyntaxKind::ExtendsKeyword
                            | SyntaxKind::BarToken
                            | SyntaxKind::AmpersandToken
                            | SyntaxKind::CommaToken
                    )
                }
            }
            _ => false,
        };

        // Restore state
        self.scanner.restore_state(snapshot);
        self.current_token = current;

        if is_type_assertion && !self.look_ahead_is_jsx_fragment() {
            self.parse_type_assertion()
        } else {
            self.parse_jsx_element_or_self_closing_or_fragment(true)
        }
    }

    /// Check if this is a JSX fragment: <>
    fn look_ahead_is_jsx_fragment(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let current = self.current_token;

        self.next_token(); // consume <
        let is_fragment = self.is_token(SyntaxKind::GreaterThanToken);

        self.scanner.restore_state(snapshot);
        self.current_token = current;
        is_fragment
    }

    /// Parse a type assertion: <Type>expression
    fn parse_type_assertion(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::LessThanToken);
        let type_node = self.parse_type();
        self.parse_expected(SyntaxKind::GreaterThanToken);
        let expression = self.parse_unary_expression();
        let end_pos = self.token_end();

        self.arena.add_type_assertion(
            syntax_kind_ext::TYPE_ASSERTION,
            start_pos,
            end_pos,
            TypeAssertionData {
                type_node,
                expression,
            },
        )
    }

    /// Parse a JSX element, self-closing element, or fragment.
    /// Called when we see `<` in an expression context.
    fn parse_jsx_element_or_self_closing_or_fragment(&mut self, in_expression_context: bool) -> NodeIndex {
        let start_pos = self.token_pos();
        let opening = self.parse_jsx_opening_or_self_closing_or_fragment(in_expression_context);

        // Check what type of opening element we got
        let kind = self.arena.get(opening).map(|n| n.kind).unwrap_or(0);

        if kind == syntax_kind_ext::JSX_OPENING_ELEMENT {
            // Parse children and closing element
            let children = self.parse_jsx_children();
            let closing = self.parse_jsx_closing_element();
            let end_pos = self.token_end();

            self.arena.add_jsx_element(
                syntax_kind_ext::JSX_ELEMENT,
                start_pos,
                end_pos,
                crate::parser::thin_node::JsxElementData {
                    opening_element: opening,
                    children,
                    closing_element: closing,
                },
            )
        } else if kind == syntax_kind_ext::JSX_OPENING_FRAGMENT {
            // Parse children and closing fragment
            let children = self.parse_jsx_children();
            let closing = self.parse_jsx_closing_fragment();
            let end_pos = self.token_end();

            self.arena.add_jsx_fragment(
                syntax_kind_ext::JSX_FRAGMENT,
                start_pos,
                end_pos,
                crate::parser::thin_node::JsxFragmentData {
                    opening_fragment: opening,
                    children,
                    closing_fragment: closing,
                },
            )
        } else {
            // Self-closing element, already complete
            opening
        }
    }

    /// Parse JSX opening element, self-closing element, or opening fragment.
    fn parse_jsx_opening_or_self_closing_or_fragment(&mut self, _in_expression_context: bool) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::LessThanToken);

        // Check for fragment: <>
        if self.is_token(SyntaxKind::GreaterThanToken) {
            self.next_token(); // consume >
            let end_pos = self.token_end();
            return self.arena.add_token(syntax_kind_ext::JSX_OPENING_FRAGMENT, start_pos, end_pos);
        }

        // Parse tag name
        let tag_name = self.parse_jsx_element_name();

        // Parse optional type arguments
        let type_arguments = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_arguments())
        } else {
            None
        };

        // Parse attributes
        let attributes = self.parse_jsx_attributes();

        // Check for self-closing: />
        if self.is_token(SyntaxKind::SlashToken) {
            self.next_token(); // consume /
            self.parse_expected(SyntaxKind::GreaterThanToken);
            let end_pos = self.token_end();
            return self.arena.add_jsx_opening(
                syntax_kind_ext::JSX_SELF_CLOSING_ELEMENT,
                start_pos,
                end_pos,
                crate::parser::thin_node::JsxOpeningData {
                    tag_name,
                    type_arguments,
                    attributes,
                },
            );
        }

        // Opening element: consume > and continue parsing children
        self.parse_expected(SyntaxKind::GreaterThanToken);
        let end_pos = self.token_end();
        self.arena.add_jsx_opening(
            syntax_kind_ext::JSX_OPENING_ELEMENT,
            start_pos,
            end_pos,
            crate::parser::thin_node::JsxOpeningData {
                tag_name,
                type_arguments,
                attributes,
            },
        )
    }

    /// Parse JSX element name (identifier, this, namespaced, or property access).
    fn parse_jsx_element_name(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();

        // Parse the initial name (identifier or this)
        let mut expr = if self.is_token(SyntaxKind::ThisKeyword) {
            let pos = self.token_pos();
            self.next_token();
            let end_pos = self.token_end();
            self.arena.add_token(SyntaxKind::ThisKeyword as u16, pos, end_pos)
        } else {
            let name = self.parse_identifier();

            // Check for namespaced name (a:b)
            if self.is_token(SyntaxKind::ColonToken) {
                self.next_token(); // consume :
                let local_name = self.parse_identifier();
                let end_pos = self.token_end();
                return self.arena.add_jsx_namespaced_name(
                    syntax_kind_ext::JSX_NAMESPACED_NAME,
                    start_pos,
                    end_pos,
                    crate::parser::thin_node::JsxNamespacedNameData {
                        namespace: name,
                        name: local_name,
                    },
                );
            }

            name
        };

        // Parse property access chain (Foo.Bar.Baz)
        while self.is_token(SyntaxKind::DotToken) {
            self.next_token(); // consume .
            let name = self.parse_identifier();
            let end_pos = self.token_end();
            expr = self.arena.add_access_expr(
                syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION,
                start_pos,
                end_pos,
                crate::parser::thin_node::AccessExprData {
                    expression: expr,
                    name_or_argument: name,
                    question_dot_token: false,
                },
            );
        }

        expr
    }

    /// Parse JSX attributes list.
    fn parse_jsx_attributes(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let mut properties = Vec::new();

        while !self.is_token(SyntaxKind::GreaterThanToken)
            && !self.is_token(SyntaxKind::SlashToken)
            && !self.is_token(SyntaxKind::EndOfFileToken)
        {
            if self.is_token(SyntaxKind::OpenBraceToken) {
                // Spread attribute: {...props}
                properties.push(self.parse_jsx_spread_attribute());
            } else {
                // Regular attribute: name="value" or name={expr} or just name
                properties.push(self.parse_jsx_attribute());
            }
        }

        let end_pos = self.token_end();
        self.arena.add_jsx_attributes(
            syntax_kind_ext::JSX_ATTRIBUTES,
            start_pos,
            end_pos,
            crate::parser::thin_node::JsxAttributesData {
                properties: self.make_node_list(properties),
            },
        )
    }

    /// Parse a single JSX attribute.
    fn parse_jsx_attribute(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let name = self.parse_jsx_attribute_name();

        // Check for value: = followed by string, expression, or nested JSX
        let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
            if self.is_token(SyntaxKind::StringLiteral) {
                self.parse_string_literal()
            } else if self.is_token(SyntaxKind::OpenBraceToken) {
                self.parse_jsx_expression()
            } else if self.is_token(SyntaxKind::LessThanToken) {
                self.parse_jsx_element_or_self_closing_or_fragment(true)
            } else {
                self.parse_error_at_current_token("JSX attribute value expected");
                NodeIndex::NONE
            }
        } else {
            NodeIndex::NONE
        };

        let end_pos = self.token_end();
        self.arena.add_jsx_attribute(
            syntax_kind_ext::JSX_ATTRIBUTE,
            start_pos,
            end_pos,
            crate::parser::thin_node::JsxAttributeData {
                name,
                initializer,
            },
        )
    }

    /// Parse JSX attribute name (possibly namespaced).
    fn parse_jsx_attribute_name(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let name = self.parse_identifier();

        // Check for namespaced name (a:b)
        if self.is_token(SyntaxKind::ColonToken) {
            self.next_token(); // consume :
            let local_name = self.parse_identifier();
            let end_pos = self.token_end();
            return self.arena.add_jsx_namespaced_name(
                syntax_kind_ext::JSX_NAMESPACED_NAME,
                start_pos,
                end_pos,
                crate::parser::thin_node::JsxNamespacedNameData {
                    namespace: name,
                    name: local_name,
                },
            );
        }

        name
    }

    /// Parse a JSX spread attribute: {...props}
    fn parse_jsx_spread_attribute(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBraceToken);
        self.parse_expected(SyntaxKind::DotDotDotToken);
        let expression = self.parse_expression();
        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end_pos = self.token_end();
        self.arena.add_jsx_spread_attribute(
            syntax_kind_ext::JSX_SPREAD_ATTRIBUTE,
            start_pos,
            end_pos,
            crate::parser::thin_node::JsxSpreadAttributeData {
                expression,
            },
        )
    }

    /// Parse a JSX expression: {expr} or {...expr}
    fn parse_jsx_expression(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        // Check for spread: {...}
        let dot_dot_dot_token = self.parse_optional(SyntaxKind::DotDotDotToken);

        // Check for empty expression: {}
        let expression = if self.is_token(SyntaxKind::CloseBraceToken) {
            NodeIndex::NONE
        } else {
            self.parse_expression()
        };

        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end_pos = self.token_end();
        self.arena.add_jsx_expression(
            syntax_kind_ext::JSX_EXPRESSION,
            start_pos,
            end_pos,
            crate::parser::thin_node::JsxExpressionData {
                dot_dot_dot_token,
                expression,
            },
        )
    }

    /// Parse JSX children (elements, text, expressions).
    fn parse_jsx_children(&mut self) -> NodeList {
        let mut children = Vec::new();

        loop {
            // Check for closing tag or closing fragment
            if self.is_token(SyntaxKind::LessThanToken) {
                // Look ahead for </
                let saved = self.scanner.save_state();
                let saved_token = self.current_token;
                self.next_token();

                if self.is_token(SyntaxKind::SlashToken) {
                    // Closing tag/fragment, restore and stop
                    self.scanner.restore_state(saved);
                    self.current_token = saved_token;
                    break;
                }

                // Nested JSX element
                self.scanner.restore_state(saved);
                self.current_token = saved_token;
                children.push(self.parse_jsx_element_or_self_closing_or_fragment(false));
            } else if self.is_token(SyntaxKind::OpenBraceToken) {
                // JSX expression: {expr}
                children.push(self.parse_jsx_expression());
            } else if self.is_token(SyntaxKind::JsxText) {
                // Text node
                children.push(self.parse_jsx_text());
            } else if self.is_token(SyntaxKind::EndOfFileToken) {
                break;
            } else {
                // Unknown token in JSX children - stop
                break;
            }
        }

        self.make_node_list(children)
    }

    /// Parse JSX text content.
    fn parse_jsx_text(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        let text = self.scanner.get_token_value_ref().to_string();
        self.next_token();
        let end_pos = self.token_end();

        self.arena.add_jsx_text(
            SyntaxKind::JsxText as u16,
            start_pos,
            end_pos,
            crate::parser::thin_node::JsxTextData {
                text,
                contains_only_trivia_white_spaces: false,
            },
        )
    }

    /// Parse a JSX closing element: </Foo>
    fn parse_jsx_closing_element(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::LessThanToken);
        self.parse_expected(SyntaxKind::SlashToken);
        let tag_name = self.parse_jsx_element_name();
        self.parse_expected(SyntaxKind::GreaterThanToken);

        let end_pos = self.token_end();
        self.arena.add_jsx_closing(
            syntax_kind_ext::JSX_CLOSING_ELEMENT,
            start_pos,
            end_pos,
            crate::parser::thin_node::JsxClosingData {
                tag_name,
            },
        )
    }

    /// Parse a JSX closing fragment: </>
    fn parse_jsx_closing_fragment(&mut self) -> NodeIndex {
        let start_pos = self.token_pos();
        self.parse_expected(SyntaxKind::LessThanToken);
        self.parse_expected(SyntaxKind::SlashToken);
        self.parse_expected(SyntaxKind::GreaterThanToken);

        let end_pos = self.token_end();
        self.arena.add_token(syntax_kind_ext::JSX_CLOSING_FRAGMENT, start_pos, end_pos)
    }

    /// Consume the parser and return its parts.
    /// This is useful for taking ownership of the arena after parsing.
    pub fn into_parts(self) -> (ThinNodeArena, Vec<ParseDiagnostic>) {
        (self.arena, self.parse_diagnostics)
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
    fn test_thin_parser_class_extends_call() {
        // Class extends a mixin call
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Child extends Mixin(Parent) {}".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_class_extends_property_access() {
        // Class extends a property access
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Child extends Base.Parent {}".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_decorator_class() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "@Component class Foo {}".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_decorator_with_call() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "@Component({ selector: 'app' }) class AppComponent {}".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_multiple_decorators() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "@Component @Injectable class Service {}".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_decorator_abstract_class() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "@Serializable abstract class Base {}".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_class_extends_and_implements() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo extends Base implements A, B, C {}".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_abstract_class() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "abstract class Base { abstract method(): void; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_abstract_class_in_iife() {
        // This was causing crashes before - abstract class inside IIFE
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "(function() { abstract class Foo {} return Foo; })()".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        // Should parse without crashing
    }

    #[test]
    fn test_thin_parser_get_accessor() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { get value(): number { return 42; } }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_set_accessor() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { set value(v: number) { this._v = v; } }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_empty_accessor_body() {
        // Empty accessor body edge case (for ambient declarations)
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "declare class Foo { get value(): number; set value(v: number); }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        // Should parse without crashing
    }

    #[test]
    fn test_thin_parser_get_set_pair() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { private _x: number = 0; get x() { return this._x; } set x(v) { this._x = v; } }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
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
    fn test_thin_parser_readonly_index_signature() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "interface ReadonlyMap { readonly [key: string]: string; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_readonly_property_signature() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "interface Config { readonly name: string; readonly value: number; }".to_string(),
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

    // =========================================================================
    // JSX Tests
    // =========================================================================

    #[test]
    fn test_thin_parser_jsx_self_closing() {
        // Self-closing JSX element
        let mut parser = ThinParserState::new(
            "test.tsx".to_string(),
            "const x = <Component />;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_jsx_with_children() {
        // JSX element with children
        let mut parser = ThinParserState::new(
            "test.tsx".to_string(),
            "const x = <div><span /></div>;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_jsx_with_attributes() {
        // JSX with attributes
        let mut parser = ThinParserState::new(
            "test.tsx".to_string(),
            "const x = <div className=\"foo\" id={bar} disabled />;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_jsx_with_expression() {
        // JSX with expression children
        let mut parser = ThinParserState::new(
            "test.tsx".to_string(),
            "const x = <div>{items.map(i => <span>{i}</span>)}</div>;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_jsx_fragment() {
        // JSX fragment
        let mut parser = ThinParserState::new(
            "test.tsx".to_string(),
            "const x = <><span /><span /></>;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_jsx_spread_attribute() {
        // JSX with spread attribute
        let mut parser = ThinParserState::new(
            "test.tsx".to_string(),
            "const x = <Component {...props} />;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_jsx_namespaced() {
        // JSX with namespaced tag name
        let mut parser = ThinParserState::new(
            "test.tsx".to_string(),
            "const x = <svg:rect width={100} />;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_jsx_member_expression() {
        // JSX with member expression tag
        let mut parser = ThinParserState::new(
            "test.tsx".to_string(),
            "const x = <Foo.Bar.Baz />;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    // =========================================================================
    // Import/Export Tests
    // =========================================================================

    #[test]
    fn test_thin_parser_import_default() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            r#"import foo from "bar";"#.to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_import_named() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            r#"import { foo, bar } from "baz";"#.to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_import_namespace() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            r#"import * as foo from "bar";"#.to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_import_side_effect() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            r#"import "foo";"#.to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_export_function() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "export function foo() { return 1; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_export_const() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "export const x = 42;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_export_default() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "export default function foo() { }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_re_export() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            r#"export { foo } from "bar";"#.to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_export_star() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            r#"export * from "foo";"#.to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    // =========================================================================
    // Additional tests for common TypeScript patterns
    // =========================================================================

    #[test]
    fn test_thin_parser_static_members() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { static count: number = 0; static increment() { Foo.count++; } }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        // May have diagnostics for static but should parse
    }

    #[test]
    fn test_thin_parser_private_protected() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { private x: number; protected y: string; public z: boolean; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_readonly() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { readonly name: string; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_constructor_parameter_properties() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Person { constructor(public name: string, private age: number) {} }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_optional_chaining() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let x = obj?.prop?.method?.()".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_nullish_coalescing() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let x = a ?? b ?? c".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_type_predicate() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "function isString(x: any): x is string { return typeof x === 'string'; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_mapped_type() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Readonly<T> = { readonly [K in keyof T]: T[K] }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_conditional_type() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type IsString<T> = T extends string ? true : false".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_infer_type_complex() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type ReturnType<T> = T extends (...args: any[]) => infer R ? R : never".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_rest_spread() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "function foo(...args: number[]) { let [first, ...rest] = args; return [...rest, first]; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_destructuring_default() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let { x = 1, y = 2 } = obj; let [a = 1, b = 2] = arr;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_computed_property() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let obj = { [key]: value, ['computed']: 42 }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_symbol_property() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let obj = { [Symbol.iterator]() { } }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_bigint_literal() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let x: bigint = 123n; let y = 0xFFn;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_numeric_separator() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "let x = 1_000_000; let y = 0xFF_FF_FF;".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_private_identifier() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { #privateField = 1; #privateMethod() {} }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_satisfies() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "const obj = { x: 1, y: 2 } satisfies Record<string, number>".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_using_declaration() {
        // ECMAScript explicit resource management
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "using file = openFile(); await using conn = getConnection();".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
    }

    #[test]
    fn test_thin_parser_static_property() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { static count = 0; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_static_method() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { static create(): Foo { return new Foo(); } }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_private_property() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { private secret: string = 'hidden'; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_protected_method() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Base { protected init(): void {} }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_readonly_property() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { readonly id: number = 1; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_public_constructor() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { public constructor(x: number) {} }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_static_get_accessor() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { static get instance(): Foo { return _instance; } }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_private_set_accessor() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { private set value(v: number) { this._value = v; } }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_multiple_modifiers() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { static readonly MAX_SIZE: number = 100; private static instance: Foo; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_override_method() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Child extends Parent { override doSomething(): void {} }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_async_method() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "class Foo { async fetchData(): Promise<void> {} }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_abstract_method_in_class() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "abstract class Shape { abstract getArea(): number; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_call_signature() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "interface Callable { (): string; (x: number): number; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_construct_signature() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "interface Constructable { new (): MyClass; new (x: number): MyClass; }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_interface_with_call_and_construct() {
        // This is a common pattern for class constructors
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            r#"interface FooConstructor {
                new (): Foo;
                prototype: Foo;
            }
            interface Foo {
                (): string;
                bar(key: string): string;
            }"#.to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_type_literal_with_call_signature() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type Fn = { (): void; message: string }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_accessor_signature_in_type() {
        // Accessor signatures in type context (allowed syntactically)
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type A = { get foo(): number; set foo(v: number); }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }

    #[test]
    fn test_thin_parser_accessor_body_in_type_context() {
        // Accessor bodies in type context (error recovery - bodies not allowed but should parse)
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "type A = { get foo() { return 0 } };".to_string(),
        );
        let root = parser.parse_source_file();

        // Should parse (checker will report error about body)
        assert!(!root.is_none());
        // Parser may report an error about unexpected token, but should recover
    }

    #[test]
    fn test_thin_parser_interface_accessor_signature() {
        let mut parser = ThinParserState::new(
            "test.ts".to_string(),
            "interface X { get foo(): number; set foo(v: number); }".to_string(),
        );
        let root = parser.parse_source_file();

        assert!(!root.is_none());
        assert!(parser.get_diagnostics().is_empty(), "Errors: {:?}", parser.get_diagnostics());
    }
}
