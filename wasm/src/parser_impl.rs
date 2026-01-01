//! Parser implementation - the syntactic analyzer for TypeScript.
//!
//! This module implements the core Parser struct that produces an AST from tokens.
//! It's designed to produce the same AST as TypeScript's parser.ts.

use crate::scanner::SyntaxKind;
use crate::scanner_impl::ScannerState;
use crate::parser::{
    Node, NodeBase, NodeArena, NodeIndex, NodeList,
    syntax_kind_ext, node_flags, modifier_flags,
    Identifier, SourceFile, Block, ExpressionStatement,
    VariableStatement, VariableDeclarationList, VariableDeclaration,
    BinaryExpression, CallExpression, PropertyAccessExpression,
    FunctionDeclaration, IfStatement, ReturnStatement,
    NumericLiteral, StringLiteral, ArrayLiteralExpression,
    ObjectLiteralExpression, PropertyAssignment,
};

// =============================================================================
// Parser Context Flags
// =============================================================================

/// Parser context flags that control parsing behavior.
pub mod context_flags {
    pub const NONE: u32 = 0;
    pub const DISALLOW_IN: u32 = 1 << 0;
    pub const YIELD: u32 = 1 << 1;
    pub const DECORATOR: u32 = 1 << 2;
    pub const AWAIT: u32 = 1 << 3;
    pub const DISALLOW_CONDITIONAL_TYPES: u32 = 1 << 4;
}

// =============================================================================
// Parser State
// =============================================================================

/// The parser state that holds scanner and AST building context.
pub struct ParserState {
    /// The scanner for tokenizing
    scanner: ScannerState,
    /// Arena for allocating AST nodes
    pub arena: NodeArena,
    /// Source file name
    file_name: String,
    /// Parser context flags
    context_flags: u32,
    /// Syntax cursor for incremental parsing (not used yet)
    syntax_cursor: Option<()>,
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

/// A parse diagnostic/error.
#[derive(Clone, Debug)]
pub struct ParseDiagnostic {
    pub start: u32,
    pub length: u32,
    pub message: String,
    pub code: u32,
}

impl ParserState {
    /// Create a new parser state for the given source text.
    pub fn new(file_name: String, source_text: String) -> ParserState {
        let scanner = ScannerState::new(source_text.clone(), true);
        ParserState {
            scanner,
            arena: NodeArena::new(),
            file_name,
            context_flags: 0,
            syntax_cursor: None,
            current_token: SyntaxKind::Unknown,
            parse_diagnostics: Vec::new(),
            source_text,
            node_count: 0,
            identifiers: Vec::new(),
        }
    }

    // =========================================================================
    // Token Utilities
    // =========================================================================

    /// Get the current token.
    pub fn token(&self) -> SyntaxKind {
        self.current_token
    }

    /// Advance to the next token and return it.
    pub fn next_token(&mut self) -> SyntaxKind {
        self.current_token = self.scanner.scan();
        self.current_token
    }

    /// Get the current token's position.
    pub fn get_token_start(&self) -> u32 {
        self.scanner.get_token_start() as u32
    }

    /// Get the current token's end position.
    pub fn get_token_end(&self) -> u32 {
        self.scanner.get_token_end() as u32
    }

    /// Get the full start including leading trivia.
    pub fn get_full_start(&self) -> u32 {
        self.scanner.get_token_full_start() as u32
    }

    /// Get the current token's text value.
    pub fn get_token_value(&self) -> String {
        self.scanner.get_token_value()
    }

    /// Check if we're at end of file.
    pub fn at_end(&self) -> bool {
        self.token() == SyntaxKind::EndOfFileToken
    }

    /// Check if there was a preceding line break.
    pub fn has_preceding_line_break(&self) -> bool {
        self.scanner.has_preceding_line_break()
    }

    // =========================================================================
    // Token Matching
    // =========================================================================

    /// Check if the current token matches the expected kind.
    pub fn is_token(&self, kind: SyntaxKind) -> bool {
        self.token() == kind
    }

    /// Consume the current token if it matches, otherwise do nothing.
    /// Returns true if token was consumed.
    pub fn parse_optional(&mut self, kind: SyntaxKind) -> bool {
        if self.token() == kind {
            self.next_token();
            true
        } else {
            false
        }
    }

    /// Consume the current token if it matches, otherwise report an error.
    pub fn parse_expected(&mut self, kind: SyntaxKind) -> bool {
        if self.token() == kind {
            self.next_token();
            true
        } else {
            self.parse_error_at_current_token(&format!("'{}' expected", token_to_string(kind)));
            false
        }
    }

    /// Report a parse error at the current token position.
    pub fn parse_error_at_current_token(&mut self, message: &str) {
        let start = self.get_token_start();
        let end = self.get_token_end();
        self.parse_diagnostics.push(ParseDiagnostic {
            start,
            length: end - start,
            message: message.to_string(),
            code: 1005, // Generic syntax error
        });
    }

    // =========================================================================
    // Node Creation
    // =========================================================================

    /// Create a new NodeBase with the current position.
    pub fn create_node_base(&mut self, kind: u16) -> NodeBase {
        let pos = self.get_full_start();
        NodeBase::new_ext(kind, pos, pos)
    }

    /// Finish a node by setting its end position to current position.
    pub fn finish_node<F>(&mut self, mut create: F) -> NodeIndex
    where F: FnMut(&mut Self) -> Node
    {
        let node = create(self);
        let end = self.get_token_start();
        let id = self.node_count;
        self.node_count += 1;
        let idx = self.arena.add(node);
        // Update end position
        if let Some(n) = self.arena.get_mut(idx) {
            n.base_mut().end = end;
            n.base_mut().id = id;
        }
        idx
    }

    /// Allocate a node in the arena.
    pub fn alloc_node(&mut self, mut node: Node) -> NodeIndex {
        node.base_mut().id = self.node_count;
        self.node_count += 1;
        self.arena.add(node)
    }

    // =========================================================================
    // Context Management
    // =========================================================================

    /// Check if we're in a context where 'in' is disallowed.
    pub fn in_disallow_in_context(&self) -> bool {
        (self.context_flags & context_flags::DISALLOW_IN) != 0
    }

    /// Check if we're in a yield context.
    pub fn in_yield_context(&self) -> bool {
        (self.context_flags & context_flags::YIELD) != 0
    }

    /// Check if we're in an await context.
    pub fn in_await_context(&self) -> bool {
        (self.context_flags & context_flags::AWAIT) != 0
    }

    /// Run a function with modified context flags.
    pub fn with_context<T, F>(&mut self, flags: u32, func: F) -> T
    where F: FnOnce(&mut Self) -> T
    {
        let saved = self.context_flags;
        self.context_flags |= flags;
        let result = func(self);
        self.context_flags = saved;
        result
    }

    /// Run a function with context flags cleared.
    pub fn without_context<T, F>(&mut self, flags: u32, func: F) -> T
    where F: FnOnce(&mut Self) -> T
    {
        let saved = self.context_flags;
        self.context_flags &= !flags;
        let result = func(self);
        self.context_flags = saved;
        result
    }
}

// =============================================================================
// Parse Source File
// =============================================================================

impl ParserState {
    /// Parse a source file - the main entry point.
    pub fn parse_source_file(&mut self) -> NodeIndex {
        // Start scanning
        self.next_token();

        let pos = 0u32;
        let statements = self.parse_list(|p| p.is_statement_start(), |p| p.parse_statement());

        // Create end of file token
        let eof_base = NodeBase::new(SyntaxKind::EndOfFileToken, self.get_token_start(), self.get_token_end());
        let eof_idx = self.alloc_node(Node::EndOfFileToken(eof_base));

        let end = self.scanner.get_pos() as u32;

        // Create source file node
        let source_file = SourceFile {
            base: NodeBase::new_ext(syntax_kind_ext::SOURCE_FILE, pos, end),
            statements,
            end_of_file_token: eof_idx,
            file_name: self.file_name.clone(),
            text: self.source_text.clone(),
            language_version: 99, // Latest
            language_variant: 0,  // Standard
            script_kind: 3,       // TS
            is_declaration_file: self.file_name.ends_with(".d.ts"),
            has_no_default_lib: false,
            identifiers: self.identifiers.clone(),
        };

        self.alloc_node(Node::SourceFile(source_file))
    }

    /// Parse a list of nodes until a terminator or end.
    fn parse_list<F, G>(&mut self, is_element: F, parse_element: G) -> NodeList
    where
        F: Fn(&Self) -> bool,
        G: Fn(&mut Self) -> NodeIndex,
    {
        let pos = self.get_full_start();
        let mut list = NodeList::new();

        while !self.at_end() && is_element(self) {
            let element = parse_element(self);
            list.push(element);
        }

        list.pos = pos;
        list.end = self.get_token_start();
        list
    }

    /// Check if the current token can start a statement.
    fn is_statement_start(&self) -> bool {
        match self.token() {
            SyntaxKind::SemicolonToken
            | SyntaxKind::OpenBraceToken
            | SyntaxKind::VarKeyword
            | SyntaxKind::LetKeyword
            | SyntaxKind::ConstKeyword
            | SyntaxKind::FunctionKeyword
            | SyntaxKind::ClassKeyword
            | SyntaxKind::EnumKeyword
            | SyntaxKind::IfKeyword
            | SyntaxKind::DoKeyword
            | SyntaxKind::WhileKeyword
            | SyntaxKind::ForKeyword
            | SyntaxKind::ContinueKeyword
            | SyntaxKind::BreakKeyword
            | SyntaxKind::ReturnKeyword
            | SyntaxKind::WithKeyword
            | SyntaxKind::SwitchKeyword
            | SyntaxKind::ThrowKeyword
            | SyntaxKind::TryKeyword
            | SyntaxKind::DebuggerKeyword
            | SyntaxKind::AtToken  // Decorator
            | SyntaxKind::AsyncKeyword
            | SyntaxKind::InterfaceKeyword
            | SyntaxKind::TypeKeyword
            | SyntaxKind::ModuleKeyword
            | SyntaxKind::NamespaceKeyword
            | SyntaxKind::DeclareKeyword
            | SyntaxKind::ExportKeyword
            | SyntaxKind::ImportKeyword => true,
            _ => self.is_expression_start(),
        }
    }

    /// Check if the current token can start an expression.
    fn is_expression_start(&self) -> bool {
        match self.token() {
            SyntaxKind::Identifier
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateHead
            | SyntaxKind::OpenParenToken
            | SyntaxKind::OpenBracketToken
            | SyntaxKind::OpenBraceToken
            | SyntaxKind::FunctionKeyword
            | SyntaxKind::ClassKeyword
            | SyntaxKind::NewKeyword
            | SyntaxKind::SlashToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::PlusToken
            | SyntaxKind::MinusToken
            | SyntaxKind::TildeToken
            | SyntaxKind::ExclamationToken
            | SyntaxKind::DeleteKeyword
            | SyntaxKind::TypeOfKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::PlusPlusToken
            | SyntaxKind::MinusMinusToken
            | SyntaxKind::LessThanToken
            | SyntaxKind::AwaitKeyword
            | SyntaxKind::YieldKeyword
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword => true,
            _ => false,
        }
    }
}

// =============================================================================
// Statement Parsing
// =============================================================================

impl ParserState {
    /// Parse a statement.
    pub fn parse_statement(&mut self) -> NodeIndex {
        match self.token() {
            SyntaxKind::SemicolonToken => self.parse_empty_statement(),
            SyntaxKind::OpenBraceToken => self.parse_block(),
            SyntaxKind::VarKeyword => self.parse_variable_statement(),
            SyntaxKind::LetKeyword => self.parse_variable_statement(),
            SyntaxKind::ConstKeyword => self.parse_variable_statement(),
            SyntaxKind::FunctionKeyword => self.parse_function_declaration(),
            SyntaxKind::IfKeyword => self.parse_if_statement(),
            SyntaxKind::ReturnKeyword => self.parse_return_statement(),
            // TODO: Add more statement types
            _ => self.parse_expression_statement(),
        }
    }

    /// Parse an empty statement (;).
    fn parse_empty_statement(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::SemicolonToken);
        let end = self.get_token_start();

        let base = NodeBase::new_ext(syntax_kind_ext::EMPTY_STATEMENT, pos, end);
        self.alloc_node(Node::EmptyStatement(crate::parser::EmptyStatement { base }))
    }

    /// Parse a block statement.
    fn parse_block(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        let statements = self.parse_list(
            |p| !p.is_token(SyntaxKind::CloseBraceToken) && !p.at_end(),
            |p| p.parse_statement()
        );

        self.parse_expected(SyntaxKind::CloseBraceToken);
        let end = self.get_token_start();

        let block = Block {
            base: NodeBase::new_ext(syntax_kind_ext::BLOCK, pos, end),
            statements,
            multi_line: true, // TODO: determine from token positions
        };

        self.alloc_node(Node::Block(block))
    }

    /// Parse a variable statement (var/let/const).
    fn parse_variable_statement(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let declaration_list = self.parse_variable_declaration_list();
        self.parse_semicolon();
        let end = self.get_token_start();

        let stmt = VariableStatement {
            base: NodeBase::new_ext(syntax_kind_ext::VARIABLE_STATEMENT, pos, end),
            modifiers: None,
            declaration_list,
        };

        self.alloc_node(Node::VariableStatement(stmt))
    }

    /// Parse a variable declaration list.
    fn parse_variable_declaration_list(&mut self) -> NodeIndex {
        let pos = self.get_full_start();

        // Get the declaration kind and set flags
        let flags = match self.token() {
            SyntaxKind::VarKeyword => 0,
            SyntaxKind::LetKeyword => node_flags::LET,
            SyntaxKind::ConstKeyword => node_flags::CONST,
            _ => 0,
        };

        self.next_token(); // consume var/let/const

        // Parse declarations
        let mut declarations = NodeList::new();
        loop {
            let decl = self.parse_variable_declaration();
            declarations.push(decl);

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        let end = self.get_token_start();

        let mut base = NodeBase::new_ext(syntax_kind_ext::VARIABLE_DECLARATION_LIST, pos, end);
        base.flags = flags;

        let list = VariableDeclarationList {
            base,
            declarations,
        };

        self.alloc_node(Node::VariableDeclarationList(list))
    }

    /// Parse a single variable declaration.
    fn parse_variable_declaration(&mut self) -> NodeIndex {
        let pos = self.get_full_start();

        // Parse binding name (identifier or pattern)
        let name = self.parse_binding_name();

        // Parse optional type annotation
        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        // Parse optional initializer
        let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
            self.parse_assignment_expression_or_higher()
        } else {
            NodeIndex::NONE
        };

        let end = self.get_token_start();

        let decl = VariableDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::VARIABLE_DECLARATION, pos, end),
            name,
            exclamation_token: false,
            type_annotation,
            initializer,
        };

        self.alloc_node(Node::VariableDeclaration(decl))
    }

    /// Parse a binding name (identifier for now).
    fn parse_binding_name(&mut self) -> NodeIndex {
        self.parse_identifier()
    }

    /// Parse a function declaration.
    fn parse_function_declaration(&mut self) -> NodeIndex {
        let pos = self.get_full_start();

        self.parse_expected(SyntaxKind::FunctionKeyword);

        let asterisk = self.parse_optional(SyntaxKind::AsteriskToken);
        let name = self.parse_identifier();

        // Parse type parameters
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

        // Parse parameters
        self.parse_expected(SyntaxKind::OpenParenToken);
        let parameters = self.parse_parameter_list();
        self.parse_expected(SyntaxKind::CloseParenToken);

        // Parse return type
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

        let end = self.get_token_start();

        let decl = FunctionDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::FUNCTION_DECLARATION, pos, end),
            modifiers: None,
            asterisk_token: asterisk,
            name,
            type_parameters,
            parameters,
            type_annotation,
            body,
        };

        self.alloc_node(Node::FunctionDeclaration(decl))
    }

    /// Parse an if statement.
    fn parse_if_statement(&mut self) -> NodeIndex {
        let pos = self.get_full_start();

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

        let end = self.get_token_start();

        let stmt = IfStatement {
            base: NodeBase::new_ext(syntax_kind_ext::IF_STATEMENT, pos, end),
            expression,
            then_statement,
            else_statement,
        };

        self.alloc_node(Node::IfStatement(stmt))
    }

    /// Parse a return statement.
    fn parse_return_statement(&mut self) -> NodeIndex {
        let pos = self.get_full_start();

        self.parse_expected(SyntaxKind::ReturnKeyword);

        // Expression is optional and not allowed with line break
        let expression = if !self.has_preceding_line_break() && self.is_expression_start() {
            self.parse_expression()
        } else {
            NodeIndex::NONE
        };

        self.parse_semicolon();
        let end = self.get_token_start();

        let stmt = ReturnStatement {
            base: NodeBase::new_ext(syntax_kind_ext::RETURN_STATEMENT, pos, end),
            expression,
        };

        self.alloc_node(Node::ReturnStatement(stmt))
    }

    /// Parse an expression statement.
    fn parse_expression_statement(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let expression = self.parse_expression();
        self.parse_semicolon();
        let end = self.get_token_start();

        let stmt = ExpressionStatement {
            base: NodeBase::new_ext(syntax_kind_ext::EXPRESSION_STATEMENT, pos, end),
            expression,
        };

        self.alloc_node(Node::ExpressionStatement(stmt))
    }

    /// Parse a semicolon, handling automatic semicolon insertion.
    fn parse_semicolon(&mut self) {
        if self.is_token(SyntaxKind::SemicolonToken) {
            self.next_token();
        } else if self.is_token(SyntaxKind::CloseBraceToken) || self.at_end() || self.has_preceding_line_break() {
            // ASI applies
        } else {
            self.parse_error_at_current_token("';' expected");
        }
    }
}

// =============================================================================
// Expression Parsing
// =============================================================================

impl ParserState {
    /// Parse an expression.
    pub fn parse_expression(&mut self) -> NodeIndex {
        self.parse_assignment_expression_or_higher()
    }

    /// Parse an assignment expression or higher precedence.
    fn parse_assignment_expression_or_higher(&mut self) -> NodeIndex {
        // For simplicity, just parse binary expressions for now
        // A full implementation would handle assignment, conditional, etc.
        self.parse_binary_expression(0)
    }

    /// Parse a binary expression with operator precedence.
    fn parse_binary_expression(&mut self, min_precedence: u8) -> NodeIndex {
        let mut left = self.parse_unary_expression();

        loop {
            let precedence = get_operator_precedence(self.token());
            if precedence == 0 || precedence < min_precedence {
                break;
            }

            let operator = self.token();
            self.next_token();

            let right = self.parse_binary_expression(precedence + 1);

            let pos = self.arena.get(left).map(|n| n.base().pos).unwrap_or(0);
            let end = self.arena.get(right).map(|n| n.base().end).unwrap_or(0);

            let expr = BinaryExpression {
                base: NodeBase::new_ext(syntax_kind_ext::BINARY_EXPRESSION, pos, end),
                left,
                operator_token: operator,
                right,
            };

            left = self.alloc_node(Node::BinaryExpression(expr));
        }

        left
    }

    /// Parse a unary expression.
    fn parse_unary_expression(&mut self) -> NodeIndex {
        // TODO: Handle prefix operators
        self.parse_postfix_expression()
    }

    /// Parse a postfix expression.
    fn parse_postfix_expression(&mut self) -> NodeIndex {
        let expr = self.parse_left_hand_side_expression();

        // TODO: Handle postfix ++/--

        expr
    }

    /// Parse a left-hand-side expression.
    fn parse_left_hand_side_expression(&mut self) -> NodeIndex {
        let mut expr = self.parse_primary_expression();

        loop {
            match self.token() {
                SyntaxKind::DotToken => {
                    // Property access: expr.name
                    self.next_token();
                    let pos = self.arena.get(expr).map(|n| n.base().pos).unwrap_or(0);
                    let name = self.parse_identifier();
                    let end = self.get_token_start();

                    let access = PropertyAccessExpression {
                        base: NodeBase::new_ext(syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION, pos, end),
                        expression: expr,
                        question_dot_token: false,
                        name,
                    };
                    expr = self.alloc_node(Node::PropertyAccessExpression(access));
                }
                SyntaxKind::OpenParenToken => {
                    // Call expression: expr(args)
                    let pos = self.arena.get(expr).map(|n| n.base().pos).unwrap_or(0);
                    let arguments = self.parse_argument_list();
                    let end = self.get_token_start();

                    let call = CallExpression {
                        base: NodeBase::new_ext(syntax_kind_ext::CALL_EXPRESSION, pos, end),
                        expression: expr,
                        type_arguments: None,
                        arguments,
                    };
                    expr = self.alloc_node(Node::CallExpression(call));
                }
                // TODO: Handle bracket access, optional chaining, etc.
                _ => break,
            }
        }

        expr
    }

    /// Parse a primary expression.
    fn parse_primary_expression(&mut self) -> NodeIndex {
        match self.token() {
            SyntaxKind::Identifier => self.parse_identifier(),
            SyntaxKind::NumericLiteral => self.parse_numeric_literal(),
            SyntaxKind::StringLiteral => self.parse_string_literal(),
            SyntaxKind::OpenBracketToken => self.parse_array_literal(),
            SyntaxKind::OpenBraceToken => self.parse_object_literal(),
            SyntaxKind::OpenParenToken => self.parse_parenthesized_expression(),
            SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword => self.parse_boolean_literal(),
            SyntaxKind::NullKeyword => self.parse_null_literal(),
            SyntaxKind::ThisKeyword => self.parse_this_expression(),
            _ => {
                // Create a missing identifier
                self.parse_error_at_current_token("Expression expected");
                self.create_missing_identifier()
            }
        }
    }

    /// Parse an identifier.
    fn parse_identifier(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let text = self.get_token_value();

        if self.is_token(SyntaxKind::Identifier) {
            self.identifiers.push(text.clone());
            self.next_token();
        } else {
            self.parse_error_at_current_token("Identifier expected");
        }

        let end = self.get_token_start();

        let id = Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, pos, end),
            escaped_text: text,
            original_text: None,
            type_arguments: None,
        };

        self.alloc_node(Node::Identifier(id))
    }

    /// Create a missing identifier node for error recovery.
    fn create_missing_identifier(&mut self) -> NodeIndex {
        let pos = self.get_token_start();
        let id = Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, pos, pos),
            escaped_text: String::new(),
            original_text: None,
            type_arguments: None,
        };
        self.alloc_node(Node::Identifier(id))
    }

    /// Parse a numeric literal.
    fn parse_numeric_literal(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let text = self.get_token_value();
        let value = text.parse::<f64>().unwrap_or(0.0);
        self.next_token();
        let end = self.get_token_start();

        let lit = NumericLiteral {
            base: NodeBase::new(SyntaxKind::NumericLiteral, pos, end),
            text,
            value,
        };

        self.alloc_node(Node::NumericLiteral(lit))
    }

    /// Parse a string literal.
    fn parse_string_literal(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let text = self.get_token_value();
        self.next_token();
        let end = self.get_token_start();

        let lit = StringLiteral {
            base: NodeBase::new(SyntaxKind::StringLiteral, pos, end),
            text,
            is_unterminated: false,
            has_extended_unicode_escape: false,
        };

        self.alloc_node(Node::StringLiteral(lit))
    }

    /// Parse an array literal.
    fn parse_array_literal(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::OpenBracketToken);

        let elements = self.parse_delimited_list(
            SyntaxKind::CloseBracketToken,
            |p| p.is_expression_start(),
            |p| p.parse_assignment_expression_or_higher(),
        );

        self.parse_expected(SyntaxKind::CloseBracketToken);
        let end = self.get_token_start();

        let lit = ArrayLiteralExpression {
            base: NodeBase::new_ext(syntax_kind_ext::ARRAY_LITERAL_EXPRESSION, pos, end),
            elements,
            multi_line: false,
        };

        self.alloc_node(Node::ArrayLiteralExpression(lit))
    }

    /// Parse an object literal.
    fn parse_object_literal(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        let properties = self.parse_delimited_list(
            SyntaxKind::CloseBraceToken,
            |p| p.is_property_name(),
            |p| p.parse_property_assignment(),
        );

        self.parse_expected(SyntaxKind::CloseBraceToken);
        let end = self.get_token_start();

        let lit = ObjectLiteralExpression {
            base: NodeBase::new_ext(syntax_kind_ext::OBJECT_LITERAL_EXPRESSION, pos, end),
            properties,
            multi_line: false,
        };

        self.alloc_node(Node::ObjectLiteralExpression(lit))
    }

    /// Check if current token can be a property name.
    fn is_property_name(&self) -> bool {
        matches!(self.token(),
            SyntaxKind::Identifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::OpenBracketToken
        )
    }

    /// Parse a property assignment.
    fn parse_property_assignment(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let name = self.parse_property_name();

        // For now just handle simple case: name: value
        self.parse_expected(SyntaxKind::ColonToken);
        let initializer = self.parse_assignment_expression_or_higher();

        let end = self.get_token_start();

        let prop = PropertyAssignment {
            base: NodeBase::new_ext(syntax_kind_ext::PROPERTY_ASSIGNMENT, pos, end),
            modifiers: None,
            name,
            initializer,
        };

        self.alloc_node(Node::PropertyAssignment(prop))
    }

    /// Parse a property name.
    fn parse_property_name(&mut self) -> NodeIndex {
        match self.token() {
            SyntaxKind::StringLiteral => self.parse_string_literal(),
            SyntaxKind::NumericLiteral => self.parse_numeric_literal(),
            // TODO: Handle computed property names
            _ => self.parse_identifier(),
        }
    }

    /// Parse a parenthesized expression.
    fn parse_parenthesized_expression(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression();
        self.parse_expected(SyntaxKind::CloseParenToken);
        let end = self.get_token_start();

        let expr = crate::parser::ParenthesizedExpression {
            base: NodeBase::new_ext(syntax_kind_ext::PARENTHESIZED_EXPRESSION, pos, end),
            expression,
        };

        self.alloc_node(Node::ParenthesizedExpression(expr))
    }

    /// Parse a boolean literal.
    fn parse_boolean_literal(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let kind = self.token();
        self.next_token();
        let end = self.get_token_start();

        let base = NodeBase::new(kind, pos, end);
        self.alloc_node(Node::Token(base))
    }

    /// Parse a null literal.
    fn parse_null_literal(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.next_token();
        let end = self.get_token_start();

        let base = NodeBase::new(SyntaxKind::NullKeyword, pos, end);
        self.alloc_node(Node::Token(base))
    }

    /// Parse a this expression.
    fn parse_this_expression(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.next_token();
        let end = self.get_token_start();

        let base = NodeBase::new(SyntaxKind::ThisKeyword, pos, end);
        self.alloc_node(Node::Token(base))
    }

    /// Parse a comma-delimited list.
    fn parse_delimited_list<F, G>(&mut self, close_token: SyntaxKind, is_element: F, parse_element: G) -> NodeList
    where
        F: Fn(&Self) -> bool,
        G: Fn(&mut Self) -> NodeIndex,
    {
        let pos = self.get_full_start();
        let mut list = NodeList::new();

        while !self.is_token(close_token) && !self.at_end() {
            if is_element(self) {
                let element = parse_element(self);
                list.push(element);

                if !self.parse_optional(SyntaxKind::CommaToken) {
                    break;
                }
            } else {
                break;
            }
        }

        list.pos = pos;
        list.end = self.get_token_start();
        list
    }

    /// Parse an argument list.
    fn parse_argument_list(&mut self) -> NodeList {
        self.parse_expected(SyntaxKind::OpenParenToken);
        let args = self.parse_delimited_list(
            SyntaxKind::CloseParenToken,
            |p| p.is_expression_start(),
            |p| p.parse_assignment_expression_or_higher(),
        );
        self.parse_expected(SyntaxKind::CloseParenToken);
        args
    }

    /// Parse a parameter list (simplified).
    fn parse_parameter_list(&mut self) -> NodeList {
        self.parse_delimited_list(
            SyntaxKind::CloseParenToken,
            |p| p.is_token(SyntaxKind::Identifier) || p.is_token(SyntaxKind::DotDotDotToken),
            |p| p.parse_parameter(),
        )
    }

    /// Parse a parameter (simplified).
    fn parse_parameter(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let name = self.parse_identifier();

        let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
            self.parse_assignment_expression_or_higher()
        } else {
            NodeIndex::NONE
        };

        let end = self.get_token_start();

        let param = crate::parser::ParameterDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::PARAMETER, pos, end),
            modifiers: None,
            dot_dot_dot_token: false,
            name,
            question_token: false,
            type_annotation,
            initializer,
        };

        self.alloc_node(Node::ParameterDeclaration(param))
    }

    /// Parse type parameters (simplified - just skip for now).
    fn parse_type_parameters(&mut self) -> NodeList {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::LessThanToken);

        // Skip type parameters for now
        let mut depth = 1;
        while depth > 0 && !self.at_end() {
            match self.token() {
                SyntaxKind::LessThanToken => depth += 1,
                SyntaxKind::GreaterThanToken => depth -= 1,
                _ => {}
            }
            self.next_token();
        }

        let end = self.get_token_start();
        let mut list = NodeList::new();
        list.pos = pos;
        list.end = end;
        list
    }

    /// Parse a type (simplified - just create an identifier for now).
    fn parse_type(&mut self) -> NodeIndex {
        // For now, just parse an identifier as a type reference
        self.parse_identifier()
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Get the precedence of a binary operator.
fn get_operator_precedence(token: SyntaxKind) -> u8 {
    match token {
        SyntaxKind::BarBarToken => 1,
        SyntaxKind::AmpersandAmpersandToken => 2,
        SyntaxKind::BarToken => 3,
        SyntaxKind::CaretToken => 4,
        SyntaxKind::AmpersandToken => 5,
        SyntaxKind::EqualsEqualsToken
        | SyntaxKind::ExclamationEqualsToken
        | SyntaxKind::EqualsEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsEqualsToken => 6,
        SyntaxKind::LessThanToken
        | SyntaxKind::GreaterThanToken
        | SyntaxKind::LessThanEqualsToken
        | SyntaxKind::GreaterThanEqualsToken
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::InKeyword => 7,
        SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken => 8,
        SyntaxKind::PlusToken | SyntaxKind::MinusToken => 9,
        SyntaxKind::AsteriskToken | SyntaxKind::SlashToken | SyntaxKind::PercentToken => 10,
        SyntaxKind::AsteriskAsteriskToken => 11,
        _ => 0,
    }
}

/// Convert a token kind to a display string.
fn token_to_string(token: SyntaxKind) -> &'static str {
    match token {
        SyntaxKind::SemicolonToken => ";",
        SyntaxKind::OpenBraceToken => "{",
        SyntaxKind::CloseBraceToken => "}",
        SyntaxKind::OpenParenToken => "(",
        SyntaxKind::CloseParenToken => ")",
        SyntaxKind::OpenBracketToken => "[",
        SyntaxKind::CloseBracketToken => "]",
        SyntaxKind::ColonToken => ":",
        SyntaxKind::CommaToken => ",",
        SyntaxKind::EqualsToken => "=",
        _ => "token",
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let mut parser = ParserState::new("test.ts".to_string(), "".to_string());
        let sf_idx = parser.parse_source_file();

        assert!(parser.arena.get(sf_idx).is_some());
    }

    #[test]
    fn test_parse_variable_declaration() {
        let mut parser = ParserState::new("test.ts".to_string(), "const x = 1;".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
        } else {
            panic!("Expected SourceFile");
        }
    }

    #[test]
    fn test_parse_expression() {
        let mut parser = ParserState::new("test.ts".to_string(), "1 + 2;".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
        } else {
            panic!("Expected SourceFile");
        }
    }

    #[test]
    fn test_parse_function() {
        let mut parser = ParserState::new("test.ts".to_string(), "function foo() { return 1; }".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
        } else {
            panic!("Expected SourceFile");
        }
    }
}
