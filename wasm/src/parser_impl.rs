//! Parser implementation - the syntactic analyzer for TypeScript.
//!
//! This module implements the core Parser struct that produces an AST from tokens.
//! It's designed to produce the same AST as TypeScript's parser.ts.

use wasm_bindgen::prelude::*;
use crate::scanner::SyntaxKind;
use crate::scanner_impl::ScannerState;
use crate::parser::{
    Node, NodeBase, NodeArena, NodeIndex, NodeList,
    syntax_kind_ext, node_flags,
    // Names and Identifiers
    Identifier,
    // Literals
    NumericLiteral, StringLiteral,
    // Expressions
    BinaryExpression, CallExpression, PropertyAccessExpression,
    ArrayLiteralExpression, ObjectLiteralExpression, PropertyAssignment,
    NewExpression, ElementAccessExpression,
    // Statements
    Block, ExpressionStatement, VariableStatement,
    VariableDeclarationList, VariableDeclaration,
    IfStatement, ReturnStatement, WhileStatement, DoStatement,
    ForStatement, ForInStatement, ForOfStatement,
    SwitchStatement, CaseClause, DefaultClause,
    ThrowStatement, TryStatement, CatchClause,
    BreakStatement, ContinueStatement, LabeledStatement,
    // Declarations
    FunctionDeclaration, ClassDeclaration,
    MethodDeclaration, PropertyDeclaration, ConstructorDeclaration,
    GetAccessorDeclaration, SetAccessorDeclaration,
    InterfaceDeclaration, TypeAliasDeclaration, EnumDeclaration, EnumMember,
    // Import/Export
    ImportDeclaration, ImportClause, NamespaceImport, NamedImports, ImportSpecifier,
    ExportDeclaration, NamedExports, ExportSpecifier, ExportAssignment,
    // Types
    TypeReference, ArrayType, TupleType, UnionType, IntersectionType,
    FunctionType, ConstructorType, TypeLiteral, ParenthesizedType, TypeParameterDeclaration,
    LiteralType, ConditionalType, InferType, TypeOperator, TypeQuery, MappedType, IndexedAccessType,
    // Misc
    SourceFile, HeritageClause, ParameterDeclaration,
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
#[wasm_bindgen]
pub struct ParserState {
    /// The scanner for tokenizing
    scanner: ScannerState,
    /// Arena for allocating AST nodes (not exposed to JS)
    #[wasm_bindgen(skip)]
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

// =============================================================================
// wasm-bindgen Public API
// =============================================================================

#[wasm_bindgen]
impl ParserState {
    /// Create a new parser state for the given source text.
    #[wasm_bindgen(constructor)]
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

    /// Parse a source file and return the root node index.
    /// The AST can be accessed via getSourceFileJson().
    #[wasm_bindgen(js_name = parseSourceFile)]
    pub fn parse_source_file_wasm(&mut self) -> u32 {
        let idx = self.parse_source_file();
        idx.0
    }

    /// Get the source file AST as JSON.
    /// This serializes the entire AST for consumption by JavaScript.
    #[wasm_bindgen(js_name = getSourceFileJson)]
    pub fn get_source_file_json(&self, root_idx: u32) -> String {
        let idx = NodeIndex(root_idx);
        if let Some(node) = self.arena.get(idx) {
            // TODO: Implement proper JSON serialization
            format!("{{\"kind\": {}, \"pos\": {}, \"end\": {}}}",
                node.base().kind,
                node.base().pos,
                node.base().end)
        } else {
            "null".to_string()
        }
    }

    /// Get the number of nodes in the AST.
    #[wasm_bindgen(js_name = getNodeCount)]
    pub fn get_node_count(&self) -> u32 {
        self.node_count
    }

    /// Get the list of identifiers found during parsing.
    #[wasm_bindgen(js_name = getIdentifiers)]
    pub fn get_identifiers(&self) -> Vec<String> {
        self.identifiers.clone()
    }

    /// Get parse diagnostics as JSON.
    #[wasm_bindgen(js_name = getDiagnosticsJson)]
    pub fn get_diagnostics_json(&self) -> String {
        let diagnostics: Vec<String> = self.parse_diagnostics.iter().map(|d| {
            format!("{{\"start\":{},\"length\":{},\"message\":\"{}\",\"code\":{}}}",
                d.start, d.length, d.message, d.code)
        }).collect();
        format!("[{}]", diagnostics.join(","))
    }
}

// =============================================================================
// Internal Implementation
// =============================================================================

impl ParserState {
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

    /// Check if we're in a context that disallows conditional types.
    pub fn in_disallow_conditional_types_context(&self) -> bool {
        (self.context_flags & context_flags::DISALLOW_CONDITIONAL_TYPES) != 0
    }

    /// Run a function allowing conditional types.
    pub fn allow_conditional_types_and<T, F>(&mut self, func: F) -> T
    where F: FnOnce(&mut Self) -> T
    {
        self.without_context(context_flags::DISALLOW_CONDITIONAL_TYPES, func)
    }

    /// Run a function disallowing conditional types.
    pub fn disallow_conditional_types_and<T, F>(&mut self, func: F) -> T
    where F: FnOnce(&mut Self) -> T
    {
        self.with_context(context_flags::DISALLOW_CONDITIONAL_TYPES, func)
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
            SyntaxKind::VarKeyword | SyntaxKind::LetKeyword | SyntaxKind::ConstKeyword => {
                self.parse_variable_statement()
            }
            SyntaxKind::FunctionKeyword => self.parse_function_declaration(),
            SyntaxKind::ClassKeyword => self.parse_class_declaration(),
            SyntaxKind::IfKeyword => self.parse_if_statement(),
            SyntaxKind::WhileKeyword => self.parse_while_statement(),
            SyntaxKind::DoKeyword => self.parse_do_statement(),
            SyntaxKind::ForKeyword => self.parse_for_statement(),
            SyntaxKind::SwitchKeyword => self.parse_switch_statement(),
            SyntaxKind::ReturnKeyword => self.parse_return_statement(),
            SyntaxKind::ThrowKeyword => self.parse_throw_statement(),
            SyntaxKind::TryKeyword => self.parse_try_statement(),
            SyntaxKind::BreakKeyword => self.parse_break_statement(),
            SyntaxKind::ContinueKeyword => self.parse_continue_statement(),
            SyntaxKind::InterfaceKeyword => self.parse_interface_declaration(),
            SyntaxKind::TypeKeyword => self.parse_type_alias_declaration(),
            SyntaxKind::EnumKeyword => self.parse_enum_declaration(),
            SyntaxKind::ImportKeyword => self.parse_import_declaration(),
            SyntaxKind::ExportKeyword => self.parse_export_declaration(),
            _ => self.parse_expression_or_labeled_statement(),
        }
    }

    /// Parse an expression statement or labeled statement.
    fn parse_expression_or_labeled_statement(&mut self) -> NodeIndex {
        // Check for labeled statement: identifier followed by colon
        if self.is_token(SyntaxKind::Identifier) {
            // Look ahead to see if next is colon
            // For now, just parse as expression statement
            // TODO: Implement proper lookahead for labels
        }
        self.parse_expression_statement()
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

    // =========================================================================
    // Loop Statements
    // =========================================================================

    /// Parse a while statement.
    fn parse_while_statement(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::WhileKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression();
        self.parse_expected(SyntaxKind::CloseParenToken);
        let statement = self.parse_statement();
        let end = self.get_token_start();

        let stmt = WhileStatement {
            base: NodeBase::new_ext(syntax_kind_ext::WHILE_STATEMENT, pos, end),
            expression,
            statement,
        };
        self.alloc_node(Node::WhileStatement(stmt))
    }

    /// Parse a do-while statement.
    fn parse_do_statement(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::DoKeyword);
        let statement = self.parse_statement();
        self.parse_expected(SyntaxKind::WhileKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression();
        self.parse_expected(SyntaxKind::CloseParenToken);
        self.parse_semicolon();
        let end = self.get_token_start();

        let stmt = DoStatement {
            base: NodeBase::new_ext(syntax_kind_ext::DO_STATEMENT, pos, end),
            statement,
            expression,
        };
        self.alloc_node(Node::DoStatement(stmt))
    }

    /// Parse a for statement (for, for-in, for-of).
    fn parse_for_statement(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::ForKeyword);

        let await_modifier = self.parse_optional(SyntaxKind::AwaitKeyword);

        self.parse_expected(SyntaxKind::OpenParenToken);

        // Parse initializer
        let initializer = if !self.is_token(SyntaxKind::SemicolonToken) {
            if self.is_token(SyntaxKind::VarKeyword)
                || self.is_token(SyntaxKind::LetKeyword)
                || self.is_token(SyntaxKind::ConstKeyword)
            {
                self.parse_variable_declaration_list()
            } else {
                self.without_context(context_flags::DISALLOW_IN, |p| p.parse_expression())
            }
        } else {
            NodeIndex::NONE
        };

        // Check for for-in or for-of
        if self.parse_optional(SyntaxKind::InKeyword) {
            let expression = self.parse_expression();
            self.parse_expected(SyntaxKind::CloseParenToken);
            let statement = self.parse_statement();
            let end = self.get_token_start();

            let stmt = ForInStatement {
                base: NodeBase::new_ext(syntax_kind_ext::FOR_IN_STATEMENT, pos, end),
                initializer,
                expression,
                statement,
            };
            return self.alloc_node(Node::ForInStatement(stmt));
        }

        if self.parse_optional(SyntaxKind::OfKeyword) {
            let expression = self.parse_assignment_expression_or_higher();
            self.parse_expected(SyntaxKind::CloseParenToken);
            let statement = self.parse_statement();
            let end = self.get_token_start();

            let stmt = ForOfStatement {
                base: NodeBase::new_ext(syntax_kind_ext::FOR_OF_STATEMENT, pos, end),
                await_modifier,
                initializer,
                expression,
                statement,
            };
            return self.alloc_node(Node::ForOfStatement(stmt));
        }

        // Regular for statement
        self.parse_expected(SyntaxKind::SemicolonToken);
        let condition = if !self.is_token(SyntaxKind::SemicolonToken) {
            self.parse_expression()
        } else {
            NodeIndex::NONE
        };
        self.parse_expected(SyntaxKind::SemicolonToken);
        let incrementor = if !self.is_token(SyntaxKind::CloseParenToken) {
            self.parse_expression()
        } else {
            NodeIndex::NONE
        };
        self.parse_expected(SyntaxKind::CloseParenToken);
        let statement = self.parse_statement();
        let end = self.get_token_start();

        let stmt = ForStatement {
            base: NodeBase::new_ext(syntax_kind_ext::FOR_STATEMENT, pos, end),
            initializer,
            condition,
            incrementor,
            statement,
        };
        self.alloc_node(Node::ForStatement(stmt))
    }

    // =========================================================================
    // Control Flow Statements
    // =========================================================================

    /// Parse a switch statement.
    fn parse_switch_statement(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::SwitchKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression();
        self.parse_expected(SyntaxKind::CloseParenToken);
        let case_block = self.parse_case_block();
        let end = self.get_token_start();

        let stmt = SwitchStatement {
            base: NodeBase::new_ext(syntax_kind_ext::SWITCH_STATEMENT, pos, end),
            expression,
            case_block,
        };
        self.alloc_node(Node::SwitchStatement(stmt))
    }

    /// Parse a case block.
    fn parse_case_block(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        let mut clauses = NodeList::new();
        while !self.is_token(SyntaxKind::CloseBraceToken) && !self.at_end() {
            if self.is_token(SyntaxKind::CaseKeyword) {
                clauses.push(self.parse_case_clause());
            } else if self.is_token(SyntaxKind::DefaultKeyword) {
                clauses.push(self.parse_default_clause());
            } else {
                break;
            }
        }

        self.parse_expected(SyntaxKind::CloseBraceToken);
        let end = self.get_token_start();

        let block = crate::parser::CaseBlock {
            base: NodeBase::new_ext(syntax_kind_ext::CASE_BLOCK, pos, end),
            clauses,
        };
        self.alloc_node(Node::CaseBlock(block))
    }

    /// Parse a case clause.
    fn parse_case_clause(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::CaseKeyword);
        let expression = self.parse_expression();
        self.parse_expected(SyntaxKind::ColonToken);

        let statements = self.parse_list(
            |p| !p.is_token(SyntaxKind::CaseKeyword)
                && !p.is_token(SyntaxKind::DefaultKeyword)
                && !p.is_token(SyntaxKind::CloseBraceToken)
                && !p.at_end(),
            |p| p.parse_statement(),
        );

        let end = self.get_token_start();
        let clause = CaseClause {
            base: NodeBase::new_ext(syntax_kind_ext::CASE_CLAUSE, pos, end),
            expression,
            statements,
        };
        self.alloc_node(Node::CaseClause(clause))
    }

    /// Parse a default clause.
    fn parse_default_clause(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::DefaultKeyword);
        self.parse_expected(SyntaxKind::ColonToken);

        let statements = self.parse_list(
            |p| !p.is_token(SyntaxKind::CaseKeyword)
                && !p.is_token(SyntaxKind::DefaultKeyword)
                && !p.is_token(SyntaxKind::CloseBraceToken)
                && !p.at_end(),
            |p| p.parse_statement(),
        );

        let end = self.get_token_start();
        let clause = DefaultClause {
            base: NodeBase::new_ext(syntax_kind_ext::DEFAULT_CLAUSE, pos, end),
            statements,
        };
        self.alloc_node(Node::DefaultClause(clause))
    }

    /// Parse a break statement.
    fn parse_break_statement(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::BreakKeyword);

        let label = if !self.has_preceding_line_break() && self.is_token(SyntaxKind::Identifier) {
            self.parse_identifier()
        } else {
            NodeIndex::NONE
        };

        self.parse_semicolon();
        let end = self.get_token_start();

        let stmt = BreakStatement {
            base: NodeBase::new_ext(syntax_kind_ext::BREAK_STATEMENT, pos, end),
            label,
        };
        self.alloc_node(Node::BreakStatement(stmt))
    }

    /// Parse a continue statement.
    fn parse_continue_statement(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::ContinueKeyword);

        let label = if !self.has_preceding_line_break() && self.is_token(SyntaxKind::Identifier) {
            self.parse_identifier()
        } else {
            NodeIndex::NONE
        };

        self.parse_semicolon();
        let end = self.get_token_start();

        let stmt = ContinueStatement {
            base: NodeBase::new_ext(syntax_kind_ext::CONTINUE_STATEMENT, pos, end),
            label,
        };
        self.alloc_node(Node::ContinueStatement(stmt))
    }

    /// Parse a throw statement.
    fn parse_throw_statement(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::ThrowKeyword);

        // Expression required, no line break allowed
        let expression = if self.has_preceding_line_break() {
            self.parse_error_at_current_token("Line break not permitted here");
            NodeIndex::NONE
        } else {
            self.parse_expression()
        };

        self.parse_semicolon();
        let end = self.get_token_start();

        let stmt = ThrowStatement {
            base: NodeBase::new_ext(syntax_kind_ext::THROW_STATEMENT, pos, end),
            expression,
        };
        self.alloc_node(Node::ThrowStatement(stmt))
    }

    /// Parse a try statement.
    fn parse_try_statement(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::TryKeyword);
        let try_block = self.parse_block();

        let catch_clause = if self.is_token(SyntaxKind::CatchKeyword) {
            self.parse_catch_clause()
        } else {
            NodeIndex::NONE
        };

        let finally_block = if self.parse_optional(SyntaxKind::FinallyKeyword) {
            self.parse_block()
        } else {
            NodeIndex::NONE
        };

        let end = self.get_token_start();

        let stmt = TryStatement {
            base: NodeBase::new_ext(syntax_kind_ext::TRY_STATEMENT, pos, end),
            try_block,
            catch_clause,
            finally_block,
        };
        self.alloc_node(Node::TryStatement(stmt))
    }

    /// Parse a catch clause.
    fn parse_catch_clause(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::CatchKeyword);

        let variable_declaration = if self.parse_optional(SyntaxKind::OpenParenToken) {
            let decl = self.parse_variable_declaration();
            self.parse_expected(SyntaxKind::CloseParenToken);
            decl
        } else {
            NodeIndex::NONE
        };

        let block = self.parse_block();
        let end = self.get_token_start();

        let clause = CatchClause {
            base: NodeBase::new_ext(syntax_kind_ext::CATCH_CLAUSE, pos, end),
            variable_declaration,
            block,
        };
        self.alloc_node(Node::CatchClause(clause))
    }

    // =========================================================================
    // Class Declarations
    // =========================================================================

    /// Parse a class declaration.
    fn parse_class_declaration(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::ClassKeyword);

        // Optional name (anonymous for default exports)
        let name = if self.is_token(SyntaxKind::Identifier) {
            self.parse_identifier()
        } else {
            NodeIndex::NONE
        };

        // Type parameters
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

        // Heritage clauses (extends, implements)
        let heritage_clauses = self.parse_heritage_clauses();

        // Class body
        self.parse_expected(SyntaxKind::OpenBraceToken);
        let members = self.parse_class_members();
        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end = self.get_token_start();

        let decl = ClassDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::CLASS_DECLARATION, pos, end),
            modifiers: None,
            name,
            type_parameters,
            heritage_clauses,
            members,
        };
        self.alloc_node(Node::ClassDeclaration(decl))
    }

    /// Parse heritage clauses (extends, implements).
    fn parse_heritage_clauses(&mut self) -> Option<NodeList> {
        if !self.is_token(SyntaxKind::ExtendsKeyword) && !self.is_token(SyntaxKind::ImplementsKeyword) {
            return None;
        }

        let mut clauses = NodeList::new();

        while self.is_token(SyntaxKind::ExtendsKeyword) || self.is_token(SyntaxKind::ImplementsKeyword) {
            let pos = self.get_full_start();
            let token = self.token();
            self.next_token();

            let mut types = NodeList::new();
            loop {
                let expr = self.parse_left_hand_side_expression();
                types.push(expr);
                if !self.parse_optional(SyntaxKind::CommaToken) {
                    break;
                }
            }

            let end = self.get_token_start();
            let clause = HeritageClause {
                base: NodeBase::new_ext(syntax_kind_ext::HERITAGE_CLAUSE, pos, end),
                token: token as u16,
                types,
            };
            clauses.push(self.alloc_node(Node::HeritageClause(clause)));
        }

        Some(clauses)
    }

    /// Parse class members.
    fn parse_class_members(&mut self) -> NodeList {
        let mut members = NodeList::new();

        while !self.is_token(SyntaxKind::CloseBraceToken) && !self.at_end() {
            // Skip semicolons
            if self.parse_optional(SyntaxKind::SemicolonToken) {
                continue;
            }

            let member = self.parse_class_element();
            members.push(member);
        }

        members
    }

    /// Parse a single class element.
    fn parse_class_element(&mut self) -> NodeIndex {
        let pos = self.get_full_start();

        // TODO: Parse modifiers (public, private, static, etc.)

        // Check for constructor
        if self.is_token(SyntaxKind::ConstructorKeyword) {
            return self.parse_constructor_declaration(pos);
        }

        // Check for getter/setter
        if self.is_token(SyntaxKind::GetKeyword) {
            self.next_token();
            return self.parse_get_accessor(pos);
        }
        if self.is_token(SyntaxKind::SetKeyword) {
            self.next_token();
            return self.parse_set_accessor(pos);
        }

        // Parse as method or property
        let name = self.parse_property_name();

        if self.is_token(SyntaxKind::OpenParenToken) || self.is_token(SyntaxKind::LessThanToken) {
            // Method
            self.parse_method_declaration(pos, name)
        } else {
            // Property
            self.parse_property_declaration(pos, name)
        }
    }

    /// Parse a constructor declaration.
    fn parse_constructor_declaration(&mut self, pos: u32) -> NodeIndex {
        self.parse_expected(SyntaxKind::ConstructorKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);
        let parameters = self.parse_parameter_list();
        self.parse_expected(SyntaxKind::CloseParenToken);

        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            NodeIndex::NONE
        };

        let end = self.get_token_start();

        let decl = ConstructorDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::CONSTRUCTOR, pos, end),
            modifiers: None,
            type_parameters: None,
            parameters,
            body,
        };
        self.alloc_node(Node::ConstructorDeclaration(decl))
    }

    /// Parse a method declaration.
    fn parse_method_declaration(&mut self, pos: u32, name: NodeIndex) -> NodeIndex {
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
            self.parse_semicolon();
            NodeIndex::NONE
        };

        let end = self.get_token_start();

        let decl = MethodDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::METHOD_DECLARATION, pos, end),
            modifiers: None,
            asterisk_token: false,
            name,
            question_token: false,
            type_parameters,
            parameters,
            type_annotation,
            body,
        };
        self.alloc_node(Node::MethodDeclaration(decl))
    }

    /// Parse a property declaration.
    fn parse_property_declaration(&mut self, pos: u32, name: NodeIndex) -> NodeIndex {
        let question_token = self.parse_optional(SyntaxKind::QuestionToken);
        let exclamation_token = self.parse_optional(SyntaxKind::ExclamationToken);

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

        self.parse_semicolon();
        let end = self.get_token_start();

        let decl = PropertyDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::PROPERTY_DECLARATION, pos, end),
            modifiers: None,
            name,
            question_token,
            exclamation_token,
            type_annotation,
            initializer,
        };
        self.alloc_node(Node::PropertyDeclaration(decl))
    }

    /// Parse a get accessor.
    fn parse_get_accessor(&mut self, pos: u32) -> NodeIndex {
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

        let end = self.get_token_start();

        let decl = GetAccessorDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::GET_ACCESSOR, pos, end),
            modifiers: None,
            name,
            type_parameters: None,
            parameters: NodeList::new(),
            type_annotation,
            body,
        };
        self.alloc_node(Node::GetAccessorDeclaration(decl))
    }

    /// Parse a set accessor.
    fn parse_set_accessor(&mut self, pos: u32) -> NodeIndex {
        let name = self.parse_property_name();
        self.parse_expected(SyntaxKind::OpenParenToken);
        let parameters = self.parse_parameter_list();
        self.parse_expected(SyntaxKind::CloseParenToken);

        let body = if self.is_token(SyntaxKind::OpenBraceToken) {
            self.parse_block()
        } else {
            NodeIndex::NONE
        };

        let end = self.get_token_start();

        let decl = SetAccessorDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::SET_ACCESSOR, pos, end),
            modifiers: None,
            name,
            type_parameters: None,
            parameters,
            body,
        };
        self.alloc_node(Node::SetAccessorDeclaration(decl))
    }

    // =========================================================================
    // Type Declarations
    // =========================================================================

    /// Parse an interface declaration.
    fn parse_interface_declaration(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::InterfaceKeyword);
        let name = self.parse_identifier();

        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

        let heritage_clauses = self.parse_heritage_clauses();

        self.parse_expected(SyntaxKind::OpenBraceToken);
        let members = self.parse_type_members();
        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end = self.get_token_start();

        let decl = InterfaceDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::INTERFACE_DECLARATION, pos, end),
            modifiers: None,
            name,
            type_parameters,
            heritage_clauses,
            members,
        };
        self.alloc_node(Node::InterfaceDeclaration(decl))
    }

    /// Parse type members (for interfaces and type literals).
    fn parse_type_members(&mut self) -> NodeList {
        let mut members = NodeList::new();

        while !self.is_token(SyntaxKind::CloseBraceToken) && !self.at_end() {
            // Skip semicolons and commas
            if self.parse_optional(SyntaxKind::SemicolonToken) || self.parse_optional(SyntaxKind::CommaToken) {
                continue;
            }

            // Parse property or method signature
            let pos = self.get_full_start();
            let name = self.parse_property_name();

            let question_token = self.parse_optional(SyntaxKind::QuestionToken);

            if self.is_token(SyntaxKind::OpenParenToken) || self.is_token(SyntaxKind::LessThanToken) {
                // Method signature
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

                let end = self.get_token_start();
                let sig = crate::parser::MethodSignature {
                    base: NodeBase::new_ext(syntax_kind_ext::METHOD_SIGNATURE, pos, end),
                    modifiers: None,
                    name,
                    question_token,
                    type_parameters,
                    parameters,
                    type_annotation,
                };
                members.push(self.alloc_node(Node::MethodSignature(sig)));
            } else {
                // Property signature
                let type_annotation = if self.parse_optional(SyntaxKind::ColonToken) {
                    self.parse_type()
                } else {
                    NodeIndex::NONE
                };

                let end = self.get_token_start();
                let sig = crate::parser::PropertySignature {
                    base: NodeBase::new_ext(syntax_kind_ext::PROPERTY_SIGNATURE, pos, end),
                    modifiers: None,
                    name,
                    question_token,
                    type_annotation,
                    initializer: NodeIndex::NONE,
                };
                members.push(self.alloc_node(Node::PropertySignature(sig)));
            }
        }

        members
    }

    /// Parse a type alias declaration.
    fn parse_type_alias_declaration(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::TypeKeyword);
        let name = self.parse_identifier();

        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

        self.parse_expected(SyntaxKind::EqualsToken);
        let type_node = self.parse_type();
        self.parse_semicolon();

        let end = self.get_token_start();

        let decl = TypeAliasDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::TYPE_ALIAS_DECLARATION, pos, end),
            modifiers: None,
            name,
            type_parameters,
            type_node,
        };
        self.alloc_node(Node::TypeAliasDeclaration(decl))
    }

    /// Parse an enum declaration.
    fn parse_enum_declaration(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::EnumKeyword);
        let name = self.parse_identifier();

        self.parse_expected(SyntaxKind::OpenBraceToken);
        let members = self.parse_enum_members();
        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end = self.get_token_start();

        let decl = EnumDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::ENUM_DECLARATION, pos, end),
            modifiers: None,
            name,
            members,
        };
        self.alloc_node(Node::EnumDeclaration(decl))
    }

    /// Parse enum members.
    fn parse_enum_members(&mut self) -> NodeList {
        let mut members = NodeList::new();

        while !self.is_token(SyntaxKind::CloseBraceToken) && !self.at_end() {
            let pos = self.get_full_start();
            let name = self.parse_property_name();

            let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
                self.parse_assignment_expression_or_higher()
            } else {
                NodeIndex::NONE
            };

            let end = self.get_token_start();
            let member = EnumMember {
                base: NodeBase::new_ext(syntax_kind_ext::ENUM_MEMBER, pos, end),
                name,
                initializer,
            };
            members.push(self.alloc_node(Node::EnumMember(member)));

            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        members
    }

    // =========================================================================
    // Import/Export Declarations
    // =========================================================================

    /// Parse an import declaration.
    fn parse_import_declaration(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::ImportKeyword);

        // Check for import type
        let is_type_only = self.parse_optional(SyntaxKind::TypeKeyword);

        let import_clause = if self.is_token(SyntaxKind::StringLiteral) {
            // import "module" - no import clause
            NodeIndex::NONE
        } else {
            self.parse_import_clause()
        };

        // Module specifier
        let module_specifier = if import_clause != NodeIndex::NONE {
            self.parse_expected(SyntaxKind::FromKeyword);
            self.parse_string_literal()
        } else {
            self.parse_string_literal()
        };

        // Import attributes (assert clause)
        let attributes = if self.is_token(SyntaxKind::WithKeyword) || self.is_token(SyntaxKind::AssertKeyword) {
            self.next_token();
            self.parse_import_attributes()
        } else {
            NodeIndex::NONE
        };

        self.parse_semicolon();
        let end = self.get_token_start();

        let mut base = NodeBase::new_ext(syntax_kind_ext::IMPORT_DECLARATION, pos, end);
        if is_type_only {
            base.flags |= node_flags::TYPE_ONLY;
        }

        let decl = ImportDeclaration {
            base,
            modifiers: None,
            import_clause,
            module_specifier,
            attributes,
        };
        self.alloc_node(Node::ImportDeclaration(decl))
    }

    /// Parse an import clause.
    fn parse_import_clause(&mut self) -> NodeIndex {
        let pos = self.get_full_start();

        let mut name = NodeIndex::NONE;
        let mut named_bindings = NodeIndex::NONE;

        // Default import
        if self.is_token(SyntaxKind::Identifier) {
            name = self.parse_identifier();

            if self.parse_optional(SyntaxKind::CommaToken) {
                named_bindings = self.parse_named_imports_or_namespace_import();
            }
        } else {
            named_bindings = self.parse_named_imports_or_namespace_import();
        }

        let end = self.get_token_start();

        let clause = ImportClause {
            base: NodeBase::new_ext(syntax_kind_ext::IMPORT_CLAUSE, pos, end),
            is_type_only: false,
            name,
            named_bindings,
        };
        self.alloc_node(Node::ImportClause(clause))
    }

    /// Parse named imports or namespace import.
    fn parse_named_imports_or_namespace_import(&mut self) -> NodeIndex {
        if self.is_token(SyntaxKind::AsteriskToken) {
            // Namespace import: * as name
            let pos = self.get_full_start();
            self.next_token();
            self.parse_expected(SyntaxKind::AsKeyword);
            let name = self.parse_identifier();
            let end = self.get_token_start();

            let ns = NamespaceImport {
                base: NodeBase::new_ext(syntax_kind_ext::NAMESPACE_IMPORT, pos, end),
                name,
            };
            self.alloc_node(Node::NamespaceImport(ns))
        } else {
            // Named imports: { a, b as c }
            let pos = self.get_full_start();
            self.parse_expected(SyntaxKind::OpenBraceToken);

            let elements = self.parse_delimited_list(
                SyntaxKind::CloseBraceToken,
                |p| p.is_token(SyntaxKind::Identifier) || p.is_token(SyntaxKind::TypeKeyword),
                |p| p.parse_import_specifier(),
            );

            self.parse_expected(SyntaxKind::CloseBraceToken);
            let end = self.get_token_start();

            let named = NamedImports {
                base: NodeBase::new_ext(syntax_kind_ext::NAMED_IMPORTS, pos, end),
                elements,
            };
            self.alloc_node(Node::NamedImports(named))
        }
    }

    /// Parse an import specifier.
    fn parse_import_specifier(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let is_type_only = self.parse_optional(SyntaxKind::TypeKeyword);

        let property_name: NodeIndex;
        let name: NodeIndex;

        let first = self.parse_identifier();

        if self.parse_optional(SyntaxKind::AsKeyword) {
            property_name = first;
            name = self.parse_identifier();
        } else {
            property_name = NodeIndex::NONE;
            name = first;
        }

        let end = self.get_token_start();

        let spec = ImportSpecifier {
            base: NodeBase::new_ext(syntax_kind_ext::IMPORT_SPECIFIER, pos, end),
            is_type_only,
            property_name,
            name,
        };
        self.alloc_node(Node::ImportSpecifier(spec))
    }

    /// Parse import attributes.
    fn parse_import_attributes(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        let elements = self.parse_delimited_list(
            SyntaxKind::CloseBraceToken,
            |p| p.is_token(SyntaxKind::Identifier) || p.is_token(SyntaxKind::StringLiteral),
            |p| p.parse_import_attribute(),
        );

        self.parse_expected(SyntaxKind::CloseBraceToken);
        let end = self.get_token_start();

        let attrs = crate::parser::ImportAttributes {
            base: NodeBase::new_ext(syntax_kind_ext::IMPORT_ATTRIBUTES, pos, end),
            token: SyntaxKind::WithKeyword as u16,
            elements,
            multi_line: false,
        };
        self.alloc_node(Node::ImportAttributes(attrs))
    }

    /// Parse a single import attribute.
    fn parse_import_attribute(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let name = self.parse_property_name();
        self.parse_expected(SyntaxKind::ColonToken);
        let value = self.parse_assignment_expression_or_higher();
        let end = self.get_token_start();

        let attr = crate::parser::ImportAttribute {
            base: NodeBase::new_ext(syntax_kind_ext::IMPORT_ATTRIBUTE, pos, end),
            name,
            value,
        };
        self.alloc_node(Node::ImportAttribute(attr))
    }

    /// Parse an export declaration.
    fn parse_export_declaration(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::ExportKeyword);

        // export default
        if self.parse_optional(SyntaxKind::DefaultKeyword) {
            return self.parse_export_assignment(pos, true);
        }

        // export =
        if self.parse_optional(SyntaxKind::EqualsToken) {
            return self.parse_export_assignment(pos, false);
        }

        // export type
        let is_type_only = self.parse_optional(SyntaxKind::TypeKeyword);

        // export { ... }
        if self.is_token(SyntaxKind::OpenBraceToken) {
            let export_clause = self.parse_named_exports();

            let module_specifier = if self.parse_optional(SyntaxKind::FromKeyword) {
                self.parse_string_literal()
            } else {
                NodeIndex::NONE
            };

            self.parse_semicolon();
            let end = self.get_token_start();

            let mut base = NodeBase::new_ext(syntax_kind_ext::EXPORT_DECLARATION, pos, end);
            if is_type_only {
                base.flags |= node_flags::TYPE_ONLY;
            }

            let decl = ExportDeclaration {
                base,
                modifiers: None,
                is_type_only,
                export_clause,
                module_specifier,
                attributes: NodeIndex::NONE,
            };
            return self.alloc_node(Node::ExportDeclaration(decl));
        }

        // export * from "module"
        if self.is_token(SyntaxKind::AsteriskToken) {
            self.next_token();

            // export * as ns from "module"
            let export_clause = if self.parse_optional(SyntaxKind::AsKeyword) {
                let ns_pos = self.get_full_start();
                let name = self.parse_identifier();
                let ns_end = self.get_token_start();
                let ns = crate::parser::NamespaceExport {
                    base: NodeBase::new_ext(syntax_kind_ext::NAMESPACE_EXPORT, ns_pos, ns_end),
                    name,
                };
                self.alloc_node(Node::NamespaceExport(ns))
            } else {
                NodeIndex::NONE
            };

            self.parse_expected(SyntaxKind::FromKeyword);
            let module_specifier = self.parse_string_literal();
            self.parse_semicolon();

            let end = self.get_token_start();

            let decl = ExportDeclaration {
                base: NodeBase::new_ext(syntax_kind_ext::EXPORT_DECLARATION, pos, end),
                modifiers: None,
                is_type_only,
                export_clause,
                module_specifier,
                attributes: NodeIndex::NONE,
            };
            return self.alloc_node(Node::ExportDeclaration(decl));
        }

        // export function/class/etc.
        let declaration = self.parse_statement();
        let end = self.get_token_start();

        // Wrap in export declaration
        let decl = ExportDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::EXPORT_DECLARATION, pos, end),
            modifiers: None,
            is_type_only: false,
            export_clause: declaration,
            module_specifier: NodeIndex::NONE,
            attributes: NodeIndex::NONE,
        };
        self.alloc_node(Node::ExportDeclaration(decl))
    }

    /// Parse named exports.
    fn parse_named_exports(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        let elements = self.parse_delimited_list(
            SyntaxKind::CloseBraceToken,
            |p| p.is_token(SyntaxKind::Identifier) || p.is_token(SyntaxKind::TypeKeyword),
            |p| p.parse_export_specifier(),
        );

        self.parse_expected(SyntaxKind::CloseBraceToken);
        let end = self.get_token_start();

        let named = NamedExports {
            base: NodeBase::new_ext(syntax_kind_ext::NAMED_EXPORTS, pos, end),
            elements,
        };
        self.alloc_node(Node::NamedExports(named))
    }

    /// Parse an export specifier.
    fn parse_export_specifier(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let is_type_only = self.parse_optional(SyntaxKind::TypeKeyword);

        let property_name: NodeIndex;
        let name: NodeIndex;

        let first = self.parse_identifier();

        if self.parse_optional(SyntaxKind::AsKeyword) {
            property_name = first;
            name = self.parse_identifier();
        } else {
            property_name = NodeIndex::NONE;
            name = first;
        }

        let end = self.get_token_start();

        let spec = ExportSpecifier {
            base: NodeBase::new_ext(syntax_kind_ext::EXPORT_SPECIFIER, pos, end),
            is_type_only,
            property_name,
            name,
        };
        self.alloc_node(Node::ExportSpecifier(spec))
    }

    /// Parse an export assignment (export default or export =).
    fn parse_export_assignment(&mut self, pos: u32, is_export_equals: bool) -> NodeIndex {
        let expression = self.parse_assignment_expression_or_higher();
        self.parse_semicolon();
        let end = self.get_token_start();

        let decl = ExportAssignment {
            base: NodeBase::new_ext(syntax_kind_ext::EXPORT_ASSIGNMENT, pos, end),
            modifiers: None,
            is_export_equals,
            expression,
        };
        self.alloc_node(Node::ExportAssignment(decl))
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

    /// Parse a keyword as an identifier (for type keywords like string, number, etc.)
    fn parse_keyword_as_identifier(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let text = self.get_token_value();
        self.next_token();
        let end = self.get_token_start();

        let id = Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, pos, end),
            escaped_text: text,
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

    // =========================================================================
    // Type Parsing
    // =========================================================================

    /// Parse a type.
    fn parse_type(&mut self) -> NodeIndex {
        // Check for function type or constructor type first
        if self.is_start_of_function_or_constructor_type() {
            return self.parse_function_or_constructor_type();
        }

        let pos = self.get_full_start();
        let check_type = self.parse_union_or_intersection_type();

        // Check for conditional type: T extends U ? X : Y
        if !self.in_disallow_conditional_types_context()
           && !self.scanner.has_preceding_line_break()
           && self.parse_optional(SyntaxKind::ExtendsKeyword)
        {
            // The type following 'extends' is not permitted to be another conditional type
            let extends_type = self.disallow_conditional_types_and(|p| p.parse_type());
            self.parse_expected(SyntaxKind::QuestionToken);
            let true_type = self.allow_conditional_types_and(|p| p.parse_type());
            self.parse_expected(SyntaxKind::ColonToken);
            let false_type = self.allow_conditional_types_and(|p| p.parse_type());

            let end = self.get_token_start();
            let conditional = ConditionalType {
                base: NodeBase::new_ext(syntax_kind_ext::CONDITIONAL_TYPE, pos, end),
                check_type,
                extends_type,
                true_type,
                false_type,
            };
            return self.alloc_node(Node::ConditionalType(conditional));
        }

        check_type
    }

    /// Check if we're at the start of a function type or constructor type.
    fn is_start_of_function_or_constructor_type(&mut self) -> bool {
        // <T>(...) => ... is a function type
        if self.is_token(SyntaxKind::LessThanToken) {
            return true;
        }
        // (...) => ... might be a function type
        if self.is_token(SyntaxKind::OpenParenToken) && self.look_ahead_is_function_type() {
            return true;
        }
        // new (...) => ... is a constructor type
        if self.is_token(SyntaxKind::NewKeyword) {
            return true;
        }
        // abstract new (...) => ... is a constructor type
        if self.is_token(SyntaxKind::AbstractKeyword) && self.look_ahead_is_new_keyword() {
            return true;
        }
        false
    }

    /// Look ahead to see if we have "new" keyword.
    fn look_ahead_is_new_keyword(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let saved_token = self.current_token;

        self.next_token();
        let is_new = self.is_token(SyntaxKind::NewKeyword);

        // Restore
        self.scanner.restore_state(snapshot);
        self.current_token = saved_token;

        is_new
    }

    /// Look ahead to determine if this is unambiguously a function type.
    /// TypeScript checks: () =>, (x:, (x,, (x?, (x=, (..., (x) =>
    fn look_ahead_is_function_type(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let saved_token = self.current_token;

        // Consume (
        self.next_token();

        let result = if self.is_token(SyntaxKind::CloseParenToken) || self.is_token(SyntaxKind::DotDotDotToken) {
            // () or (...
            true
        } else if self.skip_parameter_start() {
            // We skipped modifiers and an identifier/pattern
            // Check for parameter indicators
            if self.is_token(SyntaxKind::ColonToken) ||
               self.is_token(SyntaxKind::CommaToken) ||
               self.is_token(SyntaxKind::QuestionToken) ||
               self.is_token(SyntaxKind::EqualsToken) {
                // (x:, (x,, (x?, (x=
                true
            } else if self.is_token(SyntaxKind::CloseParenToken) {
                self.next_token();
                // (x) =>
                self.is_token(SyntaxKind::EqualsGreaterThanToken)
            } else {
                false
            }
        } else {
            false
        };

        // Restore
        self.scanner.restore_state(snapshot);
        self.current_token = saved_token;

        result
    }

    /// Skip over parameter start (modifiers, identifier or pattern).
    fn skip_parameter_start(&mut self) -> bool {
        // Skip modifiers
        while self.is_modifier() {
            self.next_token();
        }

        // Skip identifier or this
        if self.scanner.is_identifier() || self.is_token(SyntaxKind::ThisKeyword) {
            self.next_token();
            return true;
        }

        // Skip binding patterns (simplified - just check for matching brackets)
        if self.is_token(SyntaxKind::OpenBracketToken) || self.is_token(SyntaxKind::OpenBraceToken) {
            // For now, assume valid binding pattern starts
            self.skip_matching_brackets();
            return true;
        }

        false
    }

    /// Skip matching brackets for binding patterns.
    fn skip_matching_brackets(&mut self) {
        let open = self.token();
        let close = if open == SyntaxKind::OpenBracketToken {
            SyntaxKind::CloseBracketToken
        } else {
            SyntaxKind::CloseBraceToken
        };

        let mut depth = 1;
        self.next_token();

        while depth > 0 && !self.at_end() {
            if self.is_token(open) {
                depth += 1;
            } else if self.is_token(close) {
                depth -= 1;
            }
            self.next_token();
        }
    }

    /// Check if current token is a modifier.
    fn is_modifier(&self) -> bool {
        matches!(
            self.token(),
            SyntaxKind::PublicKeyword |
            SyntaxKind::PrivateKeyword |
            SyntaxKind::ProtectedKeyword |
            SyntaxKind::StaticKeyword |
            SyntaxKind::ReadonlyKeyword |
            SyntaxKind::AbstractKeyword |
            SyntaxKind::AsyncKeyword |
            SyntaxKind::OverrideKeyword |
            SyntaxKind::ExportKeyword |
            SyntaxKind::DefaultKeyword |
            SyntaxKind::DeclareKeyword |
            SyntaxKind::ConstKeyword
        )
    }

    /// Parse a function type or constructor type.
    fn parse_function_or_constructor_type(&mut self) -> NodeIndex {
        let pos = self.get_full_start();

        // Parse modifiers for constructor type (abstract)
        let modifiers = if self.is_token(SyntaxKind::AbstractKeyword) {
            let mod_pos = self.get_full_start();
            let mut mods = NodeList::new();
            let abstract_node = self.parse_identifier();
            mods.push(abstract_node);
            Some(mods)
        } else {
            None
        };

        // Check for 'new' keyword (constructor type)
        let is_constructor_type = self.parse_optional(SyntaxKind::NewKeyword);

        // Parse type parameters
        let type_parameters = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_parameters())
        } else {
            None
        };

        // Parse parameters
        let parameters = self.parse_function_type_parameters();

        // Parse => and return type
        self.parse_expected(SyntaxKind::EqualsGreaterThanToken);
        let return_type = self.parse_type();

        let end = self.get_token_start();

        if is_constructor_type {
            let ctor_type = ConstructorType {
                base: NodeBase::new_ext(syntax_kind_ext::CONSTRUCTOR_TYPE, pos, end),
                modifiers,
                type_parameters,
                parameters,
                type_node: return_type,
            };
            self.alloc_node(Node::ConstructorType(ctor_type))
        } else {
            let func_type = FunctionType {
                base: NodeBase::new_ext(syntax_kind_ext::FUNCTION_TYPE, pos, end),
                type_parameters,
                parameters,
                type_node: return_type,
            };
            self.alloc_node(Node::FunctionType(func_type))
        }
    }

    /// Parse function type parameters (parameter list in parentheses).
    fn parse_function_type_parameters(&mut self) -> NodeList {
        self.parse_expected(SyntaxKind::OpenParenToken);
        let params = self.parse_delimited_list(
            SyntaxKind::CloseParenToken,
            |p| p.is_parameter_start(),
            |p| p.parse_parameter(),
        );
        self.parse_expected(SyntaxKind::CloseParenToken);
        params
    }

    /// Check if at the start of a parameter.
    fn is_parameter_start(&self) -> bool {
        self.scanner.is_identifier() ||
        self.is_token(SyntaxKind::DotDotDotToken) ||
        self.is_token(SyntaxKind::ThisKeyword) ||
        self.is_token(SyntaxKind::OpenBracketToken) ||
        self.is_token(SyntaxKind::OpenBraceToken) ||
        self.is_modifier()
    }

    /// Parse a union or intersection type (A | B or A & B).
    fn parse_union_or_intersection_type(&mut self) -> NodeIndex {
        let pos = self.get_full_start();

        // Check for leading | or &
        let is_union = self.is_token(SyntaxKind::BarToken);
        let is_intersection = self.is_token(SyntaxKind::AmpersandToken);

        if is_union {
            self.next_token();
        } else if is_intersection {
            self.next_token();
        }

        let first_type = self.parse_intersection_or_primary_type();

        // Check for | or &
        if self.is_token(SyntaxKind::BarToken) {
            // Union type
            let mut types = NodeList::new();
            types.push(first_type);

            while self.parse_optional(SyntaxKind::BarToken) {
                types.push(self.parse_intersection_or_primary_type());
            }

            let end = self.get_token_start();
            let union = UnionType {
                base: NodeBase::new_ext(syntax_kind_ext::UNION_TYPE, pos, end),
                types,
            };
            self.alloc_node(Node::UnionType(union))
        } else if self.is_token(SyntaxKind::AmpersandToken) {
            // Intersection type
            let mut types = NodeList::new();
            types.push(first_type);

            while self.parse_optional(SyntaxKind::AmpersandToken) {
                types.push(self.parse_primary_type());
            }

            let end = self.get_token_start();
            let intersection = IntersectionType {
                base: NodeBase::new_ext(syntax_kind_ext::INTERSECTION_TYPE, pos, end),
                types,
            };
            self.alloc_node(Node::IntersectionType(intersection))
        } else {
            first_type
        }
    }

    /// Parse intersection or primary type (handles & parsing).
    fn parse_intersection_or_primary_type(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let first_type = self.parse_type_operator_or_higher();

        if self.is_token(SyntaxKind::AmpersandToken) {
            let mut types = NodeList::new();
            types.push(first_type);

            while self.parse_optional(SyntaxKind::AmpersandToken) {
                types.push(self.parse_type_operator_or_higher());
            }

            let end = self.get_token_start();
            let intersection = IntersectionType {
                base: NodeBase::new_ext(syntax_kind_ext::INTERSECTION_TYPE, pos, end),
                types,
            };
            self.alloc_node(Node::IntersectionType(intersection))
        } else {
            first_type
        }
    }

    /// Parse type operators: keyof, unique, readonly, infer, typeof
    fn parse_type_operator_or_higher(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let operator = self.token();

        match operator {
            // keyof T, unique symbol, readonly T
            SyntaxKind::KeyOfKeyword | SyntaxKind::UniqueKeyword | SyntaxKind::ReadonlyKeyword => {
                self.next_token();
                let type_node = self.parse_type_operator_or_higher();
                let end = self.get_token_start();
                let type_op = TypeOperator {
                    base: NodeBase::new_ext(syntax_kind_ext::TYPE_OPERATOR, pos, end),
                    operator: operator as u16,
                    type_node,
                };
                self.alloc_node(Node::TypeOperator(type_op))
            }

            // infer T
            SyntaxKind::InferKeyword => {
                self.parse_infer_type()
            }

            // typeof x
            SyntaxKind::TypeOfKeyword => {
                self.parse_type_query()
            }

            // Otherwise, parse postfix types
            _ => {
                self.allow_conditional_types_and(|p| p.parse_postfix_type_or_higher())
            }
        }
    }

    /// Parse infer type: infer T
    fn parse_infer_type(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::InferKeyword);

        // Parse the type parameter
        let type_param_pos = self.get_full_start();
        let name = self.parse_identifier();

        // Try to parse constraint: infer T extends U
        let constraint = self.try_parse_infer_type_constraint();

        let type_param_end = self.get_token_start();
        let type_param = TypeParameterDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::TYPE_PARAMETER, type_param_pos, type_param_end),
            modifiers: None,
            name,
            constraint: constraint.unwrap_or(NodeIndex::NONE),
            default: NodeIndex::NONE,
        };
        let type_parameter = self.alloc_node(Node::TypeParameterDeclaration(type_param));

        let end = self.get_token_start();
        let infer = InferType {
            base: NodeBase::new_ext(syntax_kind_ext::INFER_TYPE, pos, end),
            type_parameter,
        };
        self.alloc_node(Node::InferType(infer))
    }

    /// Try to parse constraint of infer type: extends U (but not if followed by ?)
    fn try_parse_infer_type_constraint(&mut self) -> Option<NodeIndex> {
        if self.parse_optional(SyntaxKind::ExtendsKeyword) {
            let constraint = self.disallow_conditional_types_and(|p| p.parse_type());
            // If we're in disallow conditional types context or next token is not ?,
            // return the constraint
            if self.in_disallow_conditional_types_context() || !self.is_token(SyntaxKind::QuestionToken) {
                return Some(constraint);
            }
            // Otherwise, this extends belongs to a conditional type, not the infer constraint
            // We need to backtrack, but for now we'll just return None
            // TODO: proper backtracking
        }
        None
    }

    /// Parse type query: typeof x
    fn parse_type_query(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::TypeOfKeyword);

        // Parse the expression (entity name)
        let expr_name = self.parse_entity_name();

        // Parse optional type arguments: typeof x<T>
        let type_arguments = if self.is_token(SyntaxKind::LessThanToken) {
            Some(self.parse_type_arguments())
        } else {
            None
        };

        let end = self.get_token_start();
        let query = TypeQuery {
            base: NodeBase::new_ext(syntax_kind_ext::TYPE_QUERY, pos, end),
            expr_name,
            type_arguments,
        };
        self.alloc_node(Node::TypeQuery(query))
    }

    /// Parse an entity name (Identifier or QualifiedName).
    fn parse_entity_name(&mut self) -> NodeIndex {
        // For now, just parse a simple identifier
        // TODO: handle QualifiedName (A.B.C)
        self.parse_identifier()
    }

    /// Look ahead to check if we're at the start of a mapped type.
    /// Mapped types start with: { [+/-] readonly? [ identifier in ... ]
    fn look_ahead_is_start_of_mapped_type(&mut self) -> bool {
        let snapshot = self.scanner.save_state();
        let saved_token = self.current_token;

        // Consume {
        self.next_token();

        // Check for +/- before readonly
        if self.is_token(SyntaxKind::PlusToken) || self.is_token(SyntaxKind::MinusToken) {
            let result = {
                self.next_token();
                self.is_token(SyntaxKind::ReadonlyKeyword)
            };
            self.scanner.restore_state(snapshot);
            self.current_token = saved_token;
            return result;
        }

        // Skip optional readonly
        if self.is_token(SyntaxKind::ReadonlyKeyword) {
            self.next_token();
        }

        // Check for [ identifier in
        let result = self.is_token(SyntaxKind::OpenBracketToken) && {
            self.next_token();
            self.scanner.is_identifier() && {
                self.next_token();
                self.is_token(SyntaxKind::InKeyword)
            }
        };

        self.scanner.restore_state(snapshot);
        self.current_token = saved_token;
        result
    }

    /// Parse a mapped type: { [K in T]: U }
    fn parse_mapped_type(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::OpenBraceToken);

        // Parse optional readonly modifier: +readonly, -readonly, readonly
        let readonly_token = if self.is_token(SyntaxKind::ReadonlyKeyword)
            || self.is_token(SyntaxKind::PlusToken)
            || self.is_token(SyntaxKind::MinusToken)
        {
            let token = self.token();
            self.next_token();
            if token != SyntaxKind::ReadonlyKeyword {
                self.parse_expected(SyntaxKind::ReadonlyKeyword);
            }
            Some(token as u16)
        } else {
            None
        };

        // Parse [K in T]
        self.parse_expected(SyntaxKind::OpenBracketToken);
        let type_parameter = self.parse_mapped_type_parameter();

        // Parse optional 'as NameType'
        let name_type = if self.parse_optional(SyntaxKind::AsKeyword) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };
        self.parse_expected(SyntaxKind::CloseBracketToken);

        // Parse optional question modifier: +?, -?, ?
        let question_token = if self.is_token(SyntaxKind::QuestionToken)
            || self.is_token(SyntaxKind::PlusToken)
            || self.is_token(SyntaxKind::MinusToken)
        {
            let token = self.token();
            self.next_token();
            if token != SyntaxKind::QuestionToken {
                self.parse_expected(SyntaxKind::QuestionToken);
            }
            Some(token as u16)
        } else {
            None
        };

        // Parse optional type annotation
        let type_node = if self.parse_optional(SyntaxKind::ColonToken) {
            self.parse_type()
        } else {
            NodeIndex::NONE
        };

        // Skip optional semicolon
        self.parse_optional(SyntaxKind::SemicolonToken);

        // Parse any remaining members (for type literals with both mapped and regular members)
        let members = if !self.is_token(SyntaxKind::CloseBraceToken) {
            Some(self.parse_type_members())
        } else {
            None
        };

        self.parse_expected(SyntaxKind::CloseBraceToken);

        let end = self.get_token_start();
        let mapped = MappedType {
            base: NodeBase::new_ext(syntax_kind_ext::MAPPED_TYPE, pos, end),
            readonly_token,
            type_parameter,
            name_type,
            question_token,
            type_node,
            members,
        };
        self.alloc_node(Node::MappedType(mapped))
    }

    /// Parse a mapped type parameter: K in T
    fn parse_mapped_type_parameter(&mut self) -> NodeIndex {
        let pos = self.get_full_start();
        let name = self.parse_identifier();
        self.parse_expected(SyntaxKind::InKeyword);
        let constraint = self.parse_type();

        let end = self.get_token_start();
        let type_param = TypeParameterDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::TYPE_PARAMETER, pos, end),
            modifiers: None,
            name,
            constraint,
            default: NodeIndex::NONE,
        };
        self.alloc_node(Node::TypeParameterDeclaration(type_param))
    }

    /// Parse postfix type or higher (handles [] array type).
    fn parse_postfix_type_or_higher(&mut self) -> NodeIndex {
        let type_node = self.parse_primary_type();
        self.parse_type_postfix(type_node)
    }

    /// Parse a primary type (the base types before postfix operators).
    fn parse_primary_type(&mut self) -> NodeIndex {
        let pos = self.get_full_start();

        match self.token() {
            // Parenthesized type
            SyntaxKind::OpenParenToken => {
                self.next_token();
                let type_node = self.parse_type();
                self.parse_expected(SyntaxKind::CloseParenToken);

                let end = self.get_token_start();
                let paren = ParenthesizedType {
                    base: NodeBase::new_ext(syntax_kind_ext::PARENTHESIZED_TYPE, pos, end),
                    type_node,
                };
                let node_idx = self.alloc_node(Node::ParenthesizedType(paren));
                self.parse_type_postfix(node_idx)
            }

            // Tuple type
            SyntaxKind::OpenBracketToken => {
                self.next_token();
                let elements = self.parse_delimited_list(
                    SyntaxKind::CloseBracketToken,
                    |p| p.is_type_start(),
                    |p| p.parse_type(),
                );
                self.parse_expected(SyntaxKind::CloseBracketToken);

                let end = self.get_token_start();
                let tuple = TupleType {
                    base: NodeBase::new_ext(syntax_kind_ext::TUPLE_TYPE, pos, end),
                    elements,
                };
                let node_idx = self.alloc_node(Node::TupleType(tuple));
                self.parse_type_postfix(node_idx)
            }

            // Object type / type literal / mapped type
            SyntaxKind::OpenBraceToken => {
                if self.look_ahead_is_start_of_mapped_type() {
                    self.parse_mapped_type()
                } else {
                    self.next_token();
                    let members = self.parse_type_members();
                    self.parse_expected(SyntaxKind::CloseBraceToken);

                    let end = self.get_token_start();
                    let type_literal = TypeLiteral {
                        base: NodeBase::new_ext(syntax_kind_ext::TYPE_LITERAL, pos, end),
                        members,
                    };
                    let node_idx = self.alloc_node(Node::TypeLiteral(type_literal));
                    self.parse_type_postfix(node_idx)
                }
            }

            // String/number/boolean literals as types
            SyntaxKind::StringLiteral => {
                let literal = self.parse_string_literal();
                let end = self.get_token_start();
                let lit_type = LiteralType {
                    base: NodeBase::new_ext(syntax_kind_ext::LITERAL_TYPE, pos, end),
                    literal,
                };
                let node_idx = self.alloc_node(Node::LiteralType(lit_type));
                self.parse_type_postfix(node_idx)
            }

            SyntaxKind::NumericLiteral => {
                let literal = self.parse_numeric_literal();
                let end = self.get_token_start();
                let lit_type = LiteralType {
                    base: NodeBase::new_ext(syntax_kind_ext::LITERAL_TYPE, pos, end),
                    literal,
                };
                let node_idx = self.alloc_node(Node::LiteralType(lit_type));
                self.parse_type_postfix(node_idx)
            }

            SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword => {
                let literal = self.parse_identifier();
                let end = self.get_token_start();
                let lit_type = LiteralType {
                    base: NodeBase::new_ext(syntax_kind_ext::LITERAL_TYPE, pos, end),
                    literal,
                };
                let node_idx = self.alloc_node(Node::LiteralType(lit_type));
                self.parse_type_postfix(node_idx)
            }

            // Predefined type keywords (string, number, boolean, etc.)
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
            | SyntaxKind::IntrinsicKeyword => {
                // Parse as a type reference with keyword name
                let type_name = self.parse_keyword_as_identifier();
                let end = self.get_token_start();
                let type_ref = TypeReference {
                    base: NodeBase::new_ext(syntax_kind_ext::TYPE_REFERENCE, pos, end),
                    type_name,
                    type_arguments: None,
                };
                let node_idx = self.alloc_node(Node::TypeReference(type_ref));
                self.parse_type_postfix(node_idx)
            }

            // Type reference (identifier, possibly with type arguments)
            _ => {
                let type_name = self.parse_type_name();
                let type_arguments = if self.is_token(SyntaxKind::LessThanToken) {
                    Some(self.parse_type_arguments())
                } else {
                    None
                };

                let end = self.get_token_start();
                let type_ref = TypeReference {
                    base: NodeBase::new_ext(syntax_kind_ext::TYPE_REFERENCE, pos, end),
                    type_name,
                    type_arguments,
                };
                let node_idx = self.alloc_node(Node::TypeReference(type_ref));
                self.parse_type_postfix(node_idx)
            }
        }
    }

    /// Parse type postfix operators ([], [key], etc).
    fn parse_type_postfix(&mut self, mut type_node: NodeIndex) -> NodeIndex {
        while !self.at_end() {
            let pos = {
                let node = self.arena.get(type_node).unwrap();
                node.base().pos
            };

            if self.is_token(SyntaxKind::OpenBracketToken) {
                self.next_token();

                if self.is_token(SyntaxKind::CloseBracketToken) {
                    // Array type: T[]
                    self.next_token();
                    let end = self.get_token_start();
                    let array_type = ArrayType {
                        base: NodeBase::new_ext(syntax_kind_ext::ARRAY_TYPE, pos, end),
                        element_type: type_node,
                    };
                    type_node = self.alloc_node(Node::ArrayType(array_type));
                } else {
                    // Indexed access type: T[K]
                    let index_type = self.parse_type();
                    self.parse_expected(SyntaxKind::CloseBracketToken);
                    let end = self.get_token_start();
                    let indexed_access = IndexedAccessType {
                        base: NodeBase::new_ext(syntax_kind_ext::INDEXED_ACCESS_TYPE, pos, end),
                        object_type: type_node,
                        index_type,
                    };
                    type_node = self.alloc_node(Node::IndexedAccessType(indexed_access));
                }
            } else {
                break;
            }
        }

        type_node
    }

    /// Parse a type name (identifier or qualified name).
    fn parse_type_name(&mut self) -> NodeIndex {
        // For now, just parse an identifier
        self.parse_identifier()
    }

    /// Parse type arguments (<T, U>).
    fn parse_type_arguments(&mut self) -> NodeList {
        let pos = self.get_full_start();
        self.parse_expected(SyntaxKind::LessThanToken);

        let mut args = NodeList::new();
        loop {
            args.push(self.parse_type());
            if !self.parse_optional(SyntaxKind::CommaToken) {
                break;
            }
        }

        self.parse_expected(SyntaxKind::GreaterThanToken);

        let end = self.get_token_start();
        args.pos = pos;
        args.end = end;
        args
    }

    /// Check if current token starts a type.
    fn is_type_start(&self) -> bool {
        match self.token() {
            SyntaxKind::Identifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::OpenParenToken
            | SyntaxKind::OpenBracketToken
            | SyntaxKind::OpenBraceToken
            | SyntaxKind::VoidKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::UndefinedKeyword
            | SyntaxKind::NeverKeyword
            | SyntaxKind::AnyKeyword
            | SyntaxKind::UnknownKeyword
            | SyntaxKind::ObjectKeyword
            | SyntaxKind::StringKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::BigIntKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::SymbolKeyword
            // Type operators
            | SyntaxKind::TypeOfKeyword
            | SyntaxKind::KeyOfKeyword
            | SyntaxKind::UniqueKeyword
            | SyntaxKind::ReadonlyKeyword
            | SyntaxKind::InferKeyword
            // Function/constructor type start
            | SyntaxKind::NewKeyword
            | SyntaxKind::LessThanToken => true,
            _ => false,
        }
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

    #[test]
    fn test_parse_function_type() {
        let mut parser = ParserState::new("test.ts".to_string(), "type Fn = (x: number) => string;".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
            // Verify we parsed a type alias
            let stmt = parser.arena.get(source_file.statements.nodes[0]).unwrap();
            if let Node::TypeAliasDeclaration(type_alias) = stmt {
                // Verify the type is a FunctionType
                let type_node = parser.arena.get(type_alias.type_node).unwrap();
                assert!(matches!(type_node, Node::FunctionType(_)), "Expected FunctionType");
            } else {
                panic!("Expected TypeAliasDeclaration");
            }
        } else {
            panic!("Expected SourceFile");
        }
    }

    #[test]
    fn test_parse_constructor_type() {
        let mut parser = ParserState::new("test.ts".to_string(), "type Ctor = new (x: number) => MyClass;".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
            // Verify we parsed a type alias
            let stmt = parser.arena.get(source_file.statements.nodes[0]).unwrap();
            if let Node::TypeAliasDeclaration(type_alias) = stmt {
                // Verify the type is a ConstructorType
                let type_node = parser.arena.get(type_alias.type_node).unwrap();
                assert!(matches!(type_node, Node::ConstructorType(_)), "Expected ConstructorType");
            } else {
                panic!("Expected TypeAliasDeclaration");
            }
        } else {
            panic!("Expected SourceFile");
        }
    }

    #[test]
    fn test_parse_generic_function_type() {
        let mut parser = ParserState::new("test.ts".to_string(), "type GenericFn = <T>(x: T) => T;".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
            let stmt = parser.arena.get(source_file.statements.nodes[0]).unwrap();
            if let Node::TypeAliasDeclaration(type_alias) = stmt {
                let type_node = parser.arena.get(type_alias.type_node).unwrap();
                if let Node::FunctionType(func_type) = type_node {
                    assert!(func_type.type_parameters.is_some(), "Expected type parameters");
                } else {
                    panic!("Expected FunctionType");
                }
            } else {
                panic!("Expected TypeAliasDeclaration");
            }
        } else {
            panic!("Expected SourceFile");
        }
    }

    #[test]
    fn test_parse_conditional_type() {
        let mut parser = ParserState::new("test.ts".to_string(), "type Check<T> = T extends string ? 'yes' : 'no';".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
            let stmt = parser.arena.get(source_file.statements.nodes[0]).unwrap();
            if let Node::TypeAliasDeclaration(type_alias) = stmt {
                let type_node = parser.arena.get(type_alias.type_node).unwrap();
                assert!(matches!(type_node, Node::ConditionalType(_)), "Expected ConditionalType, got {:?}", type_node);
            } else {
                panic!("Expected TypeAliasDeclaration");
            }
        } else {
            panic!("Expected SourceFile");
        }
    }

    #[test]
    fn test_parse_infer_type() {
        let mut parser = ParserState::new("test.ts".to_string(), "type Unpacked<T> = T extends Array<infer U> ? U : T;".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
            let stmt = parser.arena.get(source_file.statements.nodes[0]).unwrap();
            if let Node::TypeAliasDeclaration(type_alias) = stmt {
                let type_node = parser.arena.get(type_alias.type_node).unwrap();
                // Should be a conditional type with infer inside
                assert!(matches!(type_node, Node::ConditionalType(_)), "Expected ConditionalType");
            } else {
                panic!("Expected TypeAliasDeclaration");
            }
        } else {
            panic!("Expected SourceFile");
        }
    }

    #[test]
    fn test_parse_typeof() {
        let mut parser = ParserState::new("test.ts".to_string(), "type T = typeof myVariable;".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
            let stmt = parser.arena.get(source_file.statements.nodes[0]).unwrap();
            if let Node::TypeAliasDeclaration(type_alias) = stmt {
                let type_node = parser.arena.get(type_alias.type_node).unwrap();
                assert!(matches!(type_node, Node::TypeQuery(_)), "Expected TypeQuery, got {:?}", type_node);
            } else {
                panic!("Expected TypeAliasDeclaration");
            }
        } else {
            panic!("Expected SourceFile");
        }
    }

    #[test]
    fn test_parse_keyof() {
        let mut parser = ParserState::new("test.ts".to_string(), "type Keys = keyof { a: 1; b: 2 };".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
            let stmt = parser.arena.get(source_file.statements.nodes[0]).unwrap();
            if let Node::TypeAliasDeclaration(type_alias) = stmt {
                let type_node = parser.arena.get(type_alias.type_node).unwrap();
                assert!(matches!(type_node, Node::TypeOperator(_)), "Expected TypeOperator (keyof), got {:?}", type_node);
            } else {
                panic!("Expected TypeAliasDeclaration");
            }
        } else {
            panic!("Expected SourceFile");
        }
    }

    #[test]
    fn test_parse_readonly_type() {
        let mut parser = ParserState::new("test.ts".to_string(), "type ReadonlyArr = readonly number[];".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
            let stmt = parser.arena.get(source_file.statements.nodes[0]).unwrap();
            if let Node::TypeAliasDeclaration(type_alias) = stmt {
                let type_node = parser.arena.get(type_alias.type_node).unwrap();
                assert!(matches!(type_node, Node::TypeOperator(_)), "Expected TypeOperator (readonly), got {:?}", type_node);
            } else {
                panic!("Expected TypeAliasDeclaration");
            }
        } else {
            panic!("Expected SourceFile");
        }
    }

    #[test]
    fn test_parse_mapped_type() {
        let mut parser = ParserState::new("test.ts".to_string(), "type Readonly<T> = { readonly [K in keyof T]: T[K] };".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
            let stmt = parser.arena.get(source_file.statements.nodes[0]).unwrap();
            if let Node::TypeAliasDeclaration(type_alias) = stmt {
                let type_node = parser.arena.get(type_alias.type_node).unwrap();
                assert!(matches!(type_node, Node::MappedType(_)), "Expected MappedType, got {:?}", type_node);
            } else {
                panic!("Expected TypeAliasDeclaration");
            }
        } else {
            panic!("Expected SourceFile");
        }
    }

    #[test]
    fn test_parse_indexed_access_type() {
        let mut parser = ParserState::new("test.ts".to_string(), "type PropType = T['prop'];".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
            let stmt = parser.arena.get(source_file.statements.nodes[0]).unwrap();
            if let Node::TypeAliasDeclaration(type_alias) = stmt {
                let type_node = parser.arena.get(type_alias.type_node).unwrap();
                assert!(matches!(type_node, Node::IndexedAccessType(_)), "Expected IndexedAccessType, got {:?}", type_node);
            } else {
                panic!("Expected TypeAliasDeclaration");
            }
        } else {
            panic!("Expected SourceFile");
        }
    }

    #[test]
    fn test_parse_mapped_type_with_as() {
        let mut parser = ParserState::new("test.ts".to_string(), "type Renamed<T> = { [K in keyof T as string]: T[K] };".to_string());
        let sf_idx = parser.parse_source_file();

        let sf = parser.arena.get(sf_idx).unwrap();
        if let Node::SourceFile(source_file) = sf {
            assert_eq!(source_file.statements.len(), 1);
            let stmt = parser.arena.get(source_file.statements.nodes[0]).unwrap();
            if let Node::TypeAliasDeclaration(type_alias) = stmt {
                let type_node = parser.arena.get(type_alias.type_node).unwrap();
                assert!(matches!(type_node, Node::MappedType(_)), "Expected MappedType, got {:?}", type_node);
            } else {
                panic!("Expected TypeAliasDeclaration");
            }
        } else {
            panic!("Expected SourceFile");
        }
    }
}
