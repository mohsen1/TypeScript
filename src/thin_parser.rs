//! ThinParser - A lightweight TypeScript parser
//!
//! This parser takes a &str source reference and parses TypeScript files
//! using iterative algorithms to avoid stack overflow.

/// Token types for TypeScript
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // End of file
    Eof,
    // Literals
    NumericLiteral,
    StringLiteral,
    Identifier,
    // Keywords
    Let,
    Const,
    Var,
    Function,
    Return,
    If,
    Else,
    For,
    While,
    Do,
    Break,
    Continue,
    Class,
    Interface,
    Type,
    Enum,
    Import,
    Export,
    From,
    As,
    Default,
    Async,
    Await,
    New,
    This,
    Super,
    Null,
    Undefined,
    True,
    False,
    Try,
    Catch,
    Finally,
    Throw,
    // Punctuation
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Semicolon,
    Comma,
    Dot,
    DotDotDot,
    Colon,
    QuestionMark,
    Arrow,
    // Operators
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    PlusPlus,
    MinusMinus,
    Equals,
    PlusEquals,
    MinusEquals,
    AsteriskEquals,
    SlashEquals,
    EqualsEquals,
    EqualsEqualsEquals,
    ExclamationEquals,
    ExclamationEqualsEquals,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    Ampersand,
    AmpersandAmpersand,
    Bar,
    BarBar,
    Caret,
    Tilde,
    Exclamation,
    QuestionQuestion,
    // Comments and whitespace (usually skipped)
    Whitespace,
    Newline,
    SingleLineComment,
    MultiLineComment,
    // Unknown/Error
    Unknown,
}

/// A token with its position in source
#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

/// AST Node types
#[derive(Debug, Clone)]
pub enum AstNode<'a> {
    // Program root
    Program { statements: Vec<AstNode<'a>> },

    // Statements
    VariableDeclaration {
        kind: &'a str,  // "let", "const", "var"
        declarations: Vec<AstNode<'a>>,
    },
    VariableDeclarator {
        name: &'a str,
        type_annotation: Option<Box<AstNode<'a>>>,
        init: Option<Box<AstNode<'a>>>,
    },
    FunctionDeclaration {
        name: Option<&'a str>,
        params: Vec<AstNode<'a>>,
        return_type: Option<Box<AstNode<'a>>>,
        body: Option<Box<AstNode<'a>>>,
        is_async: bool,
    },
    BlockStatement {
        statements: Vec<AstNode<'a>>,
    },
    ExpressionStatement {
        expression: Box<AstNode<'a>>,
    },
    ReturnStatement {
        argument: Option<Box<AstNode<'a>>>,
    },
    IfStatement {
        test: Box<AstNode<'a>>,
        consequent: Box<AstNode<'a>>,
        alternate: Option<Box<AstNode<'a>>>,
    },
    WhileStatement {
        test: Box<AstNode<'a>>,
        body: Box<AstNode<'a>>,
    },
    ForStatement {
        init: Option<Box<AstNode<'a>>>,
        test: Option<Box<AstNode<'a>>>,
        update: Option<Box<AstNode<'a>>>,
        body: Box<AstNode<'a>>,
    },
    BreakStatement,
    ContinueStatement,
    EmptyStatement,

    // Declarations
    ClassDeclaration {
        name: Option<&'a str>,
        super_class: Option<Box<AstNode<'a>>>,
        body: Vec<AstNode<'a>>,
    },
    InterfaceDeclaration {
        name: &'a str,
        extends: Vec<AstNode<'a>>,
        body: Vec<AstNode<'a>>,
    },
    TypeAliasDeclaration {
        name: &'a str,
        type_params: Vec<AstNode<'a>>,
        type_annotation: Box<AstNode<'a>>,
    },
    ImportDeclaration {
        specifiers: Vec<AstNode<'a>>,
        source: &'a str,
    },
    ExportDeclaration {
        declaration: Option<Box<AstNode<'a>>>,
    },

    // Expressions
    Identifier { name: &'a str },
    NumericLiteral { value: &'a str },
    StringLiteral { value: &'a str },
    BooleanLiteral { value: bool },
    NullLiteral,
    ArrayExpression { elements: Vec<AstNode<'a>> },
    ObjectExpression { properties: Vec<AstNode<'a>> },
    Property {
        key: Box<AstNode<'a>>,
        value: Box<AstNode<'a>>,
        shorthand: bool,
    },
    BinaryExpression {
        left: Box<AstNode<'a>>,
        operator: &'a str,
        right: Box<AstNode<'a>>,
    },
    UnaryExpression {
        operator: &'a str,
        argument: Box<AstNode<'a>>,
        prefix: bool,
    },
    CallExpression {
        callee: Box<AstNode<'a>>,
        arguments: Vec<AstNode<'a>>,
    },
    MemberExpression {
        object: Box<AstNode<'a>>,
        property: Box<AstNode<'a>>,
        computed: bool,
    },
    ArrowFunctionExpression {
        params: Vec<AstNode<'a>>,
        body: Box<AstNode<'a>>,
        is_async: bool,
    },
    AssignmentExpression {
        left: Box<AstNode<'a>>,
        operator: &'a str,
        right: Box<AstNode<'a>>,
    },
    ConditionalExpression {
        test: Box<AstNode<'a>>,
        consequent: Box<AstNode<'a>>,
        alternate: Box<AstNode<'a>>,
    },
    NewExpression {
        callee: Box<AstNode<'a>>,
        arguments: Vec<AstNode<'a>>,
    },

    // Type annotations
    TypeAnnotation { type_node: Box<AstNode<'a>> },
    TypeReference { name: &'a str, type_args: Vec<AstNode<'a>> },
    ArrayType { element_type: Box<AstNode<'a>> },
    UnionType { types: Vec<AstNode<'a>> },
    FunctionType {
        params: Vec<AstNode<'a>>,
        return_type: Box<AstNode<'a>>,
    },

    // Parameters
    Parameter {
        name: &'a str,
        type_annotation: Option<Box<AstNode<'a>>>,
        optional: bool,
        default_value: Option<Box<AstNode<'a>>>,
    },

    // Error node for recovery
    ErrorNode { message: String },
}

/// Parse error with location info
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

/// Scanner for tokenizing TypeScript source
pub struct Scanner<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    current_pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            chars: source.char_indices().peekable(),
            current_pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    fn next_char(&mut self) -> Option<(usize, char)> {
        let result = self.chars.next();
        if let Some((pos, c)) = result {
            self.current_pos = pos + c.len_utf8();
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        result
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(c) = self.peek_char() {
            if c == '\n' {
                break;
            }
            self.next_char();
        }
    }

    fn skip_block_comment(&mut self) {
        while let Some((_, c)) = self.next_char() {
            if c == '*' {
                if let Some('/') = self.peek_char() {
                    self.next_char();
                    break;
                }
            }
        }
    }

    fn scan_identifier(&mut self, start: usize) -> Token<'a> {
        let start_line = self.line;
        let start_col = self.column;

        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' || c == '$' {
                self.next_char();
            } else {
                break;
            }
        }

        let text = &self.source[start..self.current_pos];
        let kind = match text {
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "var" => TokenKind::Var,
            "function" => TokenKind::Function,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "for" => TokenKind::For,
            "while" => TokenKind::While,
            "do" => TokenKind::Do,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "class" => TokenKind::Class,
            "interface" => TokenKind::Interface,
            "type" => TokenKind::Type,
            "enum" => TokenKind::Enum,
            "import" => TokenKind::Import,
            "export" => TokenKind::Export,
            "from" => TokenKind::From,
            "as" => TokenKind::As,
            "default" => TokenKind::Default,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "new" => TokenKind::New,
            "this" => TokenKind::This,
            "super" => TokenKind::Super,
            "null" => TokenKind::Null,
            "undefined" => TokenKind::Undefined,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            "finally" => TokenKind::Finally,
            "throw" => TokenKind::Throw,
            _ => TokenKind::Identifier,
        };

        Token {
            kind,
            text,
            start,
            end: self.current_pos,
            line: start_line,
            column: start_col,
        }
    }

    fn scan_number(&mut self, start: usize) -> Token<'a> {
        let start_line = self.line;
        let start_col = self.column;

        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '_' {
                self.next_char();
            } else {
                break;
            }
        }

        Token {
            kind: TokenKind::NumericLiteral,
            text: &self.source[start..self.current_pos],
            start,
            end: self.current_pos,
            line: start_line,
            column: start_col,
        }
    }

    fn scan_string(&mut self, quote: char, start: usize) -> Token<'a> {
        let start_line = self.line;
        let start_col = self.column;

        while let Some((_, c)) = self.next_char() {
            if c == quote {
                break;
            }
            if c == '\\' {
                self.next_char(); // Skip escaped char
            }
        }

        Token {
            kind: TokenKind::StringLiteral,
            text: &self.source[start..self.current_pos],
            start,
            end: self.current_pos,
            line: start_line,
            column: start_col,
        }
    }

    pub fn next_token(&mut self) -> Token<'a> {
        // Skip whitespace and comments
        loop {
            self.skip_whitespace();

            if let Some('/') = self.peek_char() {
                let start = self.current_pos;
                self.next_char();
                match self.peek_char() {
                    Some('/') => {
                        self.skip_line_comment();
                        continue;
                    }
                    Some('*') => {
                        self.next_char();
                        self.skip_block_comment();
                        continue;
                    }
                    Some('=') => {
                        self.next_char();
                        return Token {
                            kind: TokenKind::SlashEquals,
                            text: &self.source[start..self.current_pos],
                            start,
                            end: self.current_pos,
                            line: self.line,
                            column: self.column,
                        };
                    }
                    _ => {
                        return Token {
                            kind: TokenKind::Slash,
                            text: &self.source[start..self.current_pos],
                            start,
                            end: self.current_pos,
                            line: self.line,
                            column: self.column,
                        };
                    }
                }
            }
            break;
        }

        let start = self.current_pos;
        let start_line = self.line;
        let start_col = self.column;

        let Some((_, c)) = self.next_char() else {
            return Token {
                kind: TokenKind::Eof,
                text: "",
                start,
                end: start,
                line: start_line,
                column: start_col,
            };
        };

        match c {
            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' | '$' => self.scan_identifier(start),

            // Numbers
            '0'..='9' => self.scan_number(start),

            // Strings
            '"' | '\'' | '`' => self.scan_string(c, start),

            // Punctuation
            '{' => Token { kind: TokenKind::OpenBrace, text: "{", start, end: self.current_pos, line: start_line, column: start_col },
            '}' => Token { kind: TokenKind::CloseBrace, text: "}", start, end: self.current_pos, line: start_line, column: start_col },
            '(' => Token { kind: TokenKind::OpenParen, text: "(", start, end: self.current_pos, line: start_line, column: start_col },
            ')' => Token { kind: TokenKind::CloseParen, text: ")", start, end: self.current_pos, line: start_line, column: start_col },
            '[' => Token { kind: TokenKind::OpenBracket, text: "[", start, end: self.current_pos, line: start_line, column: start_col },
            ']' => Token { kind: TokenKind::CloseBracket, text: "]", start, end: self.current_pos, line: start_line, column: start_col },
            ';' => Token { kind: TokenKind::Semicolon, text: ";", start, end: self.current_pos, line: start_line, column: start_col },
            ',' => Token { kind: TokenKind::Comma, text: ",", start, end: self.current_pos, line: start_line, column: start_col },
            ':' => Token { kind: TokenKind::Colon, text: ":", start, end: self.current_pos, line: start_line, column: start_col },

            '.' => {
                if let Some('.') = self.peek_char() {
                    self.next_char();
                    if let Some('.') = self.peek_char() {
                        self.next_char();
                        Token { kind: TokenKind::DotDotDot, text: "...", start, end: self.current_pos, line: start_line, column: start_col }
                    } else {
                        Token { kind: TokenKind::Unknown, text: &self.source[start..self.current_pos], start, end: self.current_pos, line: start_line, column: start_col }
                    }
                } else {
                    Token { kind: TokenKind::Dot, text: ".", start, end: self.current_pos, line: start_line, column: start_col }
                }
            }

            // Operators
            '+' => {
                match self.peek_char() {
                    Some('+') => { self.next_char(); Token { kind: TokenKind::PlusPlus, text: "++", start, end: self.current_pos, line: start_line, column: start_col } }
                    Some('=') => { self.next_char(); Token { kind: TokenKind::PlusEquals, text: "+=", start, end: self.current_pos, line: start_line, column: start_col } }
                    _ => Token { kind: TokenKind::Plus, text: "+", start, end: self.current_pos, line: start_line, column: start_col }
                }
            }
            '-' => {
                match self.peek_char() {
                    Some('-') => { self.next_char(); Token { kind: TokenKind::MinusMinus, text: "--", start, end: self.current_pos, line: start_line, column: start_col } }
                    Some('=') => { self.next_char(); Token { kind: TokenKind::MinusEquals, text: "-=", start, end: self.current_pos, line: start_line, column: start_col } }
                    _ => Token { kind: TokenKind::Minus, text: "-", start, end: self.current_pos, line: start_line, column: start_col }
                }
            }
            '*' => {
                match self.peek_char() {
                    Some('=') => { self.next_char(); Token { kind: TokenKind::AsteriskEquals, text: "*=", start, end: self.current_pos, line: start_line, column: start_col } }
                    _ => Token { kind: TokenKind::Asterisk, text: "*", start, end: self.current_pos, line: start_line, column: start_col }
                }
            }
            '%' => Token { kind: TokenKind::Percent, text: "%", start, end: self.current_pos, line: start_line, column: start_col },

            '=' => {
                match self.peek_char() {
                    Some('=') => {
                        self.next_char();
                        if let Some('=') = self.peek_char() {
                            self.next_char();
                            Token { kind: TokenKind::EqualsEqualsEquals, text: "===", start, end: self.current_pos, line: start_line, column: start_col }
                        } else {
                            Token { kind: TokenKind::EqualsEquals, text: "==", start, end: self.current_pos, line: start_line, column: start_col }
                        }
                    }
                    Some('>') => { self.next_char(); Token { kind: TokenKind::Arrow, text: "=>", start, end: self.current_pos, line: start_line, column: start_col } }
                    _ => Token { kind: TokenKind::Equals, text: "=", start, end: self.current_pos, line: start_line, column: start_col }
                }
            }

            '!' => {
                match self.peek_char() {
                    Some('=') => {
                        self.next_char();
                        if let Some('=') = self.peek_char() {
                            self.next_char();
                            Token { kind: TokenKind::ExclamationEqualsEquals, text: "!==", start, end: self.current_pos, line: start_line, column: start_col }
                        } else {
                            Token { kind: TokenKind::ExclamationEquals, text: "!=", start, end: self.current_pos, line: start_line, column: start_col }
                        }
                    }
                    _ => Token { kind: TokenKind::Exclamation, text: "!", start, end: self.current_pos, line: start_line, column: start_col }
                }
            }

            '<' => {
                match self.peek_char() {
                    Some('=') => { self.next_char(); Token { kind: TokenKind::LessThanEquals, text: "<=", start, end: self.current_pos, line: start_line, column: start_col } }
                    _ => Token { kind: TokenKind::LessThan, text: "<", start, end: self.current_pos, line: start_line, column: start_col }
                }
            }
            '>' => {
                match self.peek_char() {
                    Some('=') => { self.next_char(); Token { kind: TokenKind::GreaterThanEquals, text: ">=", start, end: self.current_pos, line: start_line, column: start_col } }
                    _ => Token { kind: TokenKind::GreaterThan, text: ">", start, end: self.current_pos, line: start_line, column: start_col }
                }
            }

            '&' => {
                match self.peek_char() {
                    Some('&') => { self.next_char(); Token { kind: TokenKind::AmpersandAmpersand, text: "&&", start, end: self.current_pos, line: start_line, column: start_col } }
                    _ => Token { kind: TokenKind::Ampersand, text: "&", start, end: self.current_pos, line: start_line, column: start_col }
                }
            }
            '|' => {
                match self.peek_char() {
                    Some('|') => { self.next_char(); Token { kind: TokenKind::BarBar, text: "||", start, end: self.current_pos, line: start_line, column: start_col } }
                    _ => Token { kind: TokenKind::Bar, text: "|", start, end: self.current_pos, line: start_line, column: start_col }
                }
            }
            '^' => Token { kind: TokenKind::Caret, text: "^", start, end: self.current_pos, line: start_line, column: start_col },
            '~' => Token { kind: TokenKind::Tilde, text: "~", start, end: self.current_pos, line: start_line, column: start_col },
            '?' => {
                match self.peek_char() {
                    Some('?') => { self.next_char(); Token { kind: TokenKind::QuestionQuestion, text: "??", start, end: self.current_pos, line: start_line, column: start_col } }
                    _ => Token { kind: TokenKind::QuestionMark, text: "?", start, end: self.current_pos, line: start_line, column: start_col }
                }
            }

            _ => Token { kind: TokenKind::Unknown, text: &self.source[start..self.current_pos], start, end: self.current_pos, line: start_line, column: start_col }
        }
    }
}

/// ThinParser - Main parser struct
pub struct ThinParser<'a> {
    #[allow(dead_code)]
    source: &'a str,
    scanner: Scanner<'a>,
    current_token: Token<'a>,
    errors: Vec<ParseError>,
    /// Maximum consecutive recovery attempts before bailing
    max_consecutive_errors: usize,
    /// Current consecutive error count
    consecutive_errors: usize,
}

impl<'a> ThinParser<'a> {
    /// Create a new parser from source string
    pub fn new(source: &'a str) -> Self {
        let mut scanner = Scanner::new(source);
        let current_token = scanner.next_token();
        ThinParser {
            source,
            scanner,
            current_token,
            errors: Vec::new(),
            max_consecutive_errors: 10,
            consecutive_errors: 0,
        }
    }

    /// Reset consecutive error count (call on successful parse)
    fn reset_recovery(&mut self) {
        self.consecutive_errors = 0;
    }

    /// Record an error and check if we should bail
    fn should_bail(&mut self) -> bool {
        self.consecutive_errors += 1;
        self.consecutive_errors >= self.max_consecutive_errors
    }

    /// Try to recover from an error, returns true if recovery was successful
    fn try_recover(&mut self) -> bool {
        if self.should_bail() {
            return false;
        }

        self.skip_to_next_statement();
        true
    }

    /// Parse the source and return the AST
    pub fn parse(&mut self) -> Result<AstNode<'a>, Vec<ParseError>> {
        let program = self.parse_program();
        if self.errors.is_empty() {
            Ok(program)
        } else {
            Err(self.errors.clone())
        }
    }

    /// Get collected errors
    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    fn advance(&mut self) {
        self.current_token = self.scanner.next_token();
    }

    fn expect(&mut self, kind: TokenKind) -> bool {
        if self.current_token.kind == kind {
            self.advance();
            true
        } else {
            self.error(format!("Expected {:?}, found {:?}", kind, self.current_token.kind));
            false
        }
    }

    fn error(&mut self, message: String) {
        self.errors.push(ParseError {
            message,
            start: self.current_token.start,
            end: self.current_token.end,
            line: self.current_token.line,
            column: self.current_token.column,
        });
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.current_token.kind == kind
    }

    fn check_any(&self, kinds: &[TokenKind]) -> bool {
        kinds.contains(&self.current_token.kind)
    }

    /// Parse program (iteratively to avoid stack overflow)
    fn parse_program(&mut self) -> AstNode<'a> {
        let mut statements = Vec::new();

        while self.current_token.kind != TokenKind::Eof {
            match self.parse_statement() {
                Some(stmt) => {
                    self.reset_recovery();
                    statements.push(stmt);
                }
                None => {
                    // Error recovery: skip to next statement
                    if !self.try_recover() {
                        break;
                    }
                }
            }
        }

        AstNode::Program { statements }
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> Option<AstNode<'a>> {
        match self.current_token.kind {
            TokenKind::Let | TokenKind::Const | TokenKind::Var => {
                self.parse_variable_declaration()
            }
            TokenKind::Function => self.parse_function_declaration(),
            TokenKind::Class => self.parse_class_declaration(),
            TokenKind::Interface => self.parse_interface_declaration(),
            TokenKind::Type => self.parse_type_alias(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Break => {
                self.advance();
                self.expect(TokenKind::Semicolon);
                Some(AstNode::BreakStatement)
            }
            TokenKind::Continue => {
                self.advance();
                self.expect(TokenKind::Semicolon);
                Some(AstNode::ContinueStatement)
            }
            TokenKind::OpenBrace => self.parse_block_statement(),
            TokenKind::Semicolon => {
                self.advance();
                Some(AstNode::EmptyStatement)
            }
            TokenKind::Import => self.parse_import_declaration(),
            TokenKind::Export => self.parse_export_declaration(),
            _ => self.parse_expression_statement(),
        }
    }

    /// Parse variable declaration (let, const, var)
    fn parse_variable_declaration(&mut self) -> Option<AstNode<'a>> {
        let kind = self.current_token.text;
        self.advance();

        let mut declarations = Vec::new();

        loop {
            let name = if self.check(TokenKind::Identifier) {
                let n = self.current_token.text;
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

            let init = if self.check(TokenKind::Equals) {
                self.advance();
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            declarations.push(AstNode::VariableDeclarator {
                name,
                type_annotation,
                init,
            });

            if !self.check(TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        // Semicolon is optional in some contexts (not consumed in for loop init)
        if self.check(TokenKind::Semicolon) {
            self.advance();
        }

        Some(AstNode::VariableDeclaration { kind, declarations })
    }

    /// Parse variable declaration without consuming trailing semicolon (for for-loop init)
    fn parse_variable_declaration_no_semi(&mut self) -> Option<AstNode<'a>> {
        let kind = self.current_token.text;
        self.advance();

        let mut declarations = Vec::new();

        loop {
            let name = if self.check(TokenKind::Identifier) {
                let n = self.current_token.text;
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

            let init = if self.check(TokenKind::Equals) {
                self.advance();
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            declarations.push(AstNode::VariableDeclarator {
                name,
                type_annotation,
                init,
            });

            if !self.check(TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        // Don't consume semicolon - caller handles it
        Some(AstNode::VariableDeclaration { kind, declarations })
    }

    /// Parse function declaration
    fn parse_function_declaration(&mut self) -> Option<AstNode<'a>> {
        let is_async = self.check(TokenKind::Async);
        if is_async {
            self.advance();
        }

        self.expect(TokenKind::Function);

        let name = if self.check(TokenKind::Identifier) {
            let n = self.current_token.text;
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

    /// Parse parameters
    fn parse_parameters(&mut self) -> Vec<AstNode<'a>> {
        let mut params = Vec::new();

        while !self.check(TokenKind::CloseParen) && !self.check(TokenKind::Eof) {
            if let Some(param) = self.parse_parameter() {
                params.push(param);
            }

            if !self.check(TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        params
    }

    /// Parse a single parameter
    fn parse_parameter(&mut self) -> Option<AstNode<'a>> {
        let name = if self.check(TokenKind::Identifier) {
            let n = self.current_token.text;
            self.advance();
            n
        } else {
            return None;
        };

        let optional = if self.check(TokenKind::QuestionMark) {
            self.advance();
            true
        } else {
            false
        };

        let type_annotation = if self.check(TokenKind::Colon) {
            self.advance();
            Some(Box::new(self.parse_type_annotation()?))
        } else {
            None
        };

        let default_value = if self.check(TokenKind::Equals) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
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

    /// Parse block statement
    fn parse_block_statement(&mut self) -> Option<AstNode<'a>> {
        self.expect(TokenKind::OpenBrace);

        let mut statements = Vec::new();

        while !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::Eof) {
            if let Some(stmt) = self.parse_statement() {
                self.reset_recovery();
                statements.push(stmt);
            } else {
                // Recovery: skip to next recognizable token
                if !self.try_recover() {
                    break;
                }
            }
        }

        self.expect(TokenKind::CloseBrace);

        Some(AstNode::BlockStatement { statements })
    }

    /// Parse return statement
    fn parse_return_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'return'

        let argument = if !self.check(TokenKind::Semicolon) && !self.check(TokenKind::CloseBrace) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        if self.check(TokenKind::Semicolon) {
            self.advance();
        }

        Some(AstNode::ReturnStatement { argument })
    }

    /// Parse if statement
    fn parse_if_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'if'
        self.expect(TokenKind::OpenParen);
        let test = Box::new(self.parse_expression()?);
        self.expect(TokenKind::CloseParen);

        let consequent = Box::new(self.parse_statement()?);

        let alternate = if self.check(TokenKind::Else) {
            self.advance();
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Some(AstNode::IfStatement {
            test,
            consequent,
            alternate,
        })
    }

    /// Parse while statement
    fn parse_while_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'while'
        self.expect(TokenKind::OpenParen);
        let test = Box::new(self.parse_expression()?);
        self.expect(TokenKind::CloseParen);
        let body = Box::new(self.parse_statement()?);

        Some(AstNode::WhileStatement { test, body })
    }

    /// Parse for statement
    fn parse_for_statement(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'for'
        self.expect(TokenKind::OpenParen);

        let init = if self.check(TokenKind::Semicolon) {
            None
        } else if self.check_any(&[TokenKind::Let, TokenKind::Const, TokenKind::Var]) {
            Some(Box::new(self.parse_variable_declaration_no_semi()?))
        } else {
            Some(Box::new(self.parse_expression()?))
        };

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

        let body = Box::new(self.parse_statement()?);

        Some(AstNode::ForStatement {
            init,
            test,
            update,
            body,
        })
    }

    /// Parse class declaration
    fn parse_class_declaration(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'class'

        let name = if self.check(TokenKind::Identifier) {
            let n = self.current_token.text;
            self.advance();
            Some(n)
        } else {
            None
        };

        let super_class = if self.current_token.text == "extends" {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.expect(TokenKind::OpenBrace);

        let body = Vec::new();
        while !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::Eof) {
            // Skip class members for now
            self.advance();
        }

        self.expect(TokenKind::CloseBrace);

        Some(AstNode::ClassDeclaration {
            name,
            super_class,
            body,
        })
    }

    /// Parse interface declaration
    fn parse_interface_declaration(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'interface'

        let name = if self.check(TokenKind::Identifier) {
            let n = self.current_token.text;
            self.advance();
            n
        } else {
            self.error("Expected interface name".to_string());
            return None;
        };

        let mut extends = Vec::new();
        if self.current_token.text == "extends" {
            self.advance();
            // Parse extends list
            loop {
                if self.check(TokenKind::Identifier) {
                    extends.push(AstNode::Identifier { name: self.current_token.text });
                    self.advance();
                }
                if !self.check(TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.expect(TokenKind::OpenBrace);

        let body = Vec::new();
        while !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::Eof) {
            // Skip interface members for now
            self.advance();
        }

        self.expect(TokenKind::CloseBrace);

        Some(AstNode::InterfaceDeclaration { name, extends, body })
    }

    /// Parse type alias
    fn parse_type_alias(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'type'

        let name = if self.check(TokenKind::Identifier) {
            let n = self.current_token.text;
            self.advance();
            n
        } else {
            self.error("Expected type name".to_string());
            return None;
        };

        let type_params = Vec::new(); // TODO: parse type parameters

        self.expect(TokenKind::Equals);

        let type_annotation = Box::new(self.parse_type_annotation()?);

        if self.check(TokenKind::Semicolon) {
            self.advance();
        }

        Some(AstNode::TypeAliasDeclaration {
            name,
            type_params,
            type_annotation,
        })
    }

    /// Parse import declaration
    fn parse_import_declaration(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'import'

        let specifiers = Vec::new();

        // Simple import: import "module"
        if self.check(TokenKind::StringLiteral) {
            let source = self.current_token.text;
            self.advance();
            if self.check(TokenKind::Semicolon) {
                self.advance();
            }
            return Some(AstNode::ImportDeclaration { specifiers, source });
        }

        // Skip complex import parsing for now
        while !self.check(TokenKind::From) && !self.check(TokenKind::Eof) && !self.check(TokenKind::Semicolon) {
            self.advance();
        }

        if self.check(TokenKind::From) {
            self.advance();
        }

        let source = if self.check(TokenKind::StringLiteral) {
            let s = self.current_token.text;
            self.advance();
            s
        } else {
            ""
        };

        if self.check(TokenKind::Semicolon) {
            self.advance();
        }

        Some(AstNode::ImportDeclaration { specifiers, source })
    }

    /// Parse export declaration
    fn parse_export_declaration(&mut self) -> Option<AstNode<'a>> {
        self.advance(); // consume 'export'

        let declaration = if self.check(TokenKind::Default) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else if self.check_any(&[TokenKind::Function, TokenKind::Class, TokenKind::Const, TokenKind::Let, TokenKind::Var, TokenKind::Interface, TokenKind::Type]) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        if self.check(TokenKind::Semicolon) {
            self.advance();
        }

        Some(AstNode::ExportDeclaration { declaration })
    }

    /// Parse expression statement
    fn parse_expression_statement(&mut self) -> Option<AstNode<'a>> {
        let expression = Box::new(self.parse_expression()?);

        if self.check(TokenKind::Semicolon) {
            self.advance();
        }

        Some(AstNode::ExpressionStatement { expression })
    }

    /// Parse type annotation
    fn parse_type_annotation(&mut self) -> Option<AstNode<'a>> {
        let type_node = self.parse_union_type()?;
        Some(AstNode::TypeAnnotation {
            type_node: Box::new(type_node),
        })
    }

    /// Parse union type (iterative)
    fn parse_union_type(&mut self) -> Option<AstNode<'a>> {
        let mut types = vec![self.parse_primary_type()?];

        while self.check(TokenKind::Bar) {
            self.advance();
            types.push(self.parse_primary_type()?);
        }

        if types.len() == 1 {
            Some(types.remove(0))
        } else {
            Some(AstNode::UnionType { types })
        }
    }

    /// Parse primary type
    fn parse_primary_type(&mut self) -> Option<AstNode<'a>> {
        match self.current_token.kind {
            TokenKind::Identifier => {
                let name = self.current_token.text;
                self.advance();

                let type_args = if self.check(TokenKind::LessThan) {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.check(TokenKind::GreaterThan) && !self.check(TokenKind::Eof) {
                        args.push(self.parse_type_annotation()?);
                        if !self.check(TokenKind::Comma) {
                            break;
                        }
                        self.advance();
                    }
                    self.expect(TokenKind::GreaterThan);
                    args
                } else {
                    Vec::new()
                };

                let mut result = AstNode::TypeReference { name, type_args };

                // Array type suffix
                while self.check(TokenKind::OpenBracket) {
                    self.advance();
                    self.expect(TokenKind::CloseBracket);
                    result = AstNode::ArrayType {
                        element_type: Box::new(result),
                    };
                }

                Some(result)
            }
            TokenKind::OpenParen => {
                // Function type
                self.advance();
                let params = self.parse_parameters();
                self.expect(TokenKind::CloseParen);
                self.expect(TokenKind::Arrow);
                let return_type = Box::new(self.parse_type_annotation()?);
                Some(AstNode::FunctionType { params, return_type })
            }
            _ => {
                self.error(format!("Unexpected token in type: {:?}", self.current_token.kind));
                None
            }
        }
    }

    /// Parse expression (iterative using precedence climbing)
    fn parse_expression(&mut self) -> Option<AstNode<'a>> {
        self.parse_assignment_expression()
    }

    /// Parse assignment expression
    fn parse_assignment_expression(&mut self) -> Option<AstNode<'a>> {
        let left = self.parse_conditional_expression()?;

        if self.check_any(&[
            TokenKind::Equals,
            TokenKind::PlusEquals,
            TokenKind::MinusEquals,
            TokenKind::AsteriskEquals,
            TokenKind::SlashEquals,
        ]) {
            let operator = self.current_token.text;
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

    /// Parse conditional (ternary) expression
    fn parse_conditional_expression(&mut self) -> Option<AstNode<'a>> {
        let test = self.parse_binary_expression(0)?;

        if self.check(TokenKind::QuestionMark) {
            self.advance();
            let consequent = self.parse_expression()?;
            self.expect(TokenKind::Colon);
            let alternate = self.parse_expression()?;
            return Some(AstNode::ConditionalExpression {
                test: Box::new(test),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
            });
        }

        Some(test)
    }

    /// Get operator precedence
    fn get_precedence(&self, kind: TokenKind) -> i32 {
        match kind {
            TokenKind::BarBar | TokenKind::QuestionQuestion => 1,
            TokenKind::AmpersandAmpersand => 2,
            TokenKind::Bar => 3,
            TokenKind::Caret => 4,
            TokenKind::Ampersand => 5,
            TokenKind::EqualsEquals | TokenKind::ExclamationEquals |
            TokenKind::EqualsEqualsEquals | TokenKind::ExclamationEqualsEquals => 6,
            TokenKind::LessThan | TokenKind::GreaterThan |
            TokenKind::LessThanEquals | TokenKind::GreaterThanEquals => 7,
            TokenKind::Plus | TokenKind::Minus => 9,
            TokenKind::Asterisk | TokenKind::Slash | TokenKind::Percent => 10,
            _ => 0,
        }
    }

    /// Parse binary expression using precedence climbing (iterative)
    fn parse_binary_expression(&mut self, min_prec: i32) -> Option<AstNode<'a>> {
        let mut left = self.parse_unary_expression()?;

        loop {
            let prec = self.get_precedence(self.current_token.kind);
            if prec <= min_prec {
                break;
            }

            let operator = self.current_token.text;
            self.advance();

            let right = self.parse_binary_expression(prec)?;
            left = AstNode::BinaryExpression {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Some(left)
    }

    /// Parse unary expression
    fn parse_unary_expression(&mut self) -> Option<AstNode<'a>> {
        if self.check_any(&[
            TokenKind::Exclamation,
            TokenKind::Minus,
            TokenKind::Plus,
            TokenKind::Tilde,
            TokenKind::PlusPlus,
            TokenKind::MinusMinus,
        ]) {
            let operator = self.current_token.text;
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

    /// Parse postfix expression (call, member access, etc.) - iterative
    fn parse_postfix_expression(&mut self) -> Option<AstNode<'a>> {
        let mut expr = self.parse_primary_expression()?;

        loop {
            match self.current_token.kind {
                TokenKind::OpenParen => {
                    self.advance();
                    let arguments = self.parse_arguments();
                    self.expect(TokenKind::CloseParen);
                    expr = AstNode::CallExpression {
                        callee: Box::new(expr),
                        arguments,
                    };
                }
                TokenKind::Dot => {
                    self.advance();
                    if self.check(TokenKind::Identifier) {
                        let property = AstNode::Identifier {
                            name: self.current_token.text,
                        };
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
                TokenKind::PlusPlus | TokenKind::MinusMinus => {
                    let operator = self.current_token.text;
                    self.advance();
                    expr = AstNode::UnaryExpression {
                        operator,
                        argument: Box::new(expr),
                        prefix: false,
                    };
                }
                _ => break,
            }
        }

        Some(expr)
    }

    /// Parse arguments
    fn parse_arguments(&mut self) -> Vec<AstNode<'a>> {
        let mut args = Vec::new();

        while !self.check(TokenKind::CloseParen) && !self.check(TokenKind::Eof) {
            if let Some(arg) = self.parse_expression() {
                args.push(arg);
            }

            if !self.check(TokenKind::Comma) {
                break;
            }
            self.advance();
        }

        args
    }

    /// Parse arrow function parameters or a parenthesized expression
    /// Returns either parameters (for arrow function) or expressions (for paren expr)
    fn parse_arrow_params_or_expr(&mut self) -> Option<Vec<AstNode<'a>>> {
        let mut items = Vec::new();

        loop {
            let name = if self.check(TokenKind::Identifier) {
                let n = self.current_token.text;
                self.advance();
                n
            } else {
                break;
            };

            // Check for optional marker
            let optional = if self.check(TokenKind::QuestionMark) {
                self.advance();
                true
            } else {
                false
            };

            // Check for type annotation - this distinguishes from expression
            let type_annotation = if self.check(TokenKind::Colon) {
                self.advance();
                Some(Box::new(self.parse_type_annotation()?))
            } else {
                None
            };

            // Check for default value
            let default_value = if self.check(TokenKind::Equals) {
                self.advance();
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            // If we have type annotation or optional marker, it's definitely a parameter
            if type_annotation.is_some() || optional || default_value.is_some() {
                items.push(AstNode::Parameter {
                    name,
                    type_annotation,
                    optional,
                    default_value,
                });
            } else {
                // Could be either - we'll treat as identifier that might become param
                items.push(AstNode::Identifier { name });
            }

            if self.check(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(TokenKind::CloseParen);
        Some(items)
    }

    /// Parse primary expression
    fn parse_primary_expression(&mut self) -> Option<AstNode<'a>> {
        match self.current_token.kind {
            TokenKind::Identifier => {
                let name = self.current_token.text;
                self.advance();

                // Check for arrow function: x => expr
                if self.check(TokenKind::Arrow) {
                    self.advance();
                    let body = if self.check(TokenKind::OpenBrace) {
                        self.parse_block_statement()?
                    } else {
                        self.parse_expression()?
                    };
                    return Some(AstNode::ArrowFunctionExpression {
                        params: vec![AstNode::Parameter {
                            name,
                            type_annotation: None,
                            optional: false,
                            default_value: None,
                        }],
                        body: Box::new(body),
                        is_async: false,
                    });
                }

                Some(AstNode::Identifier { name })
            }
            TokenKind::NumericLiteral => {
                let value = self.current_token.text;
                self.advance();
                Some(AstNode::NumericLiteral { value })
            }
            TokenKind::StringLiteral => {
                let value = self.current_token.text;
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

                // Check if this looks like arrow function parameters
                // Pattern: () => ... or (identifier[?][:type][, ...]) => ...
                if self.check(TokenKind::CloseParen) {
                    // () => ...
                    self.advance();
                    if self.check(TokenKind::Arrow) {
                        self.advance();
                        let body = if self.check(TokenKind::OpenBrace) {
                            self.parse_block_statement()?
                        } else {
                            self.parse_expression()?
                        };
                        return Some(AstNode::ArrowFunctionExpression {
                            params: vec![],
                            body: Box::new(body),
                            is_async: false,
                        });
                    }
                    // Empty parentheses but not arrow - weird but allow
                    return Some(AstNode::Identifier { name: "()" });
                }

                // Try to parse as arrow function parameters if it looks like typed params
                // Check if first token is identifier followed by : or ? or , or )
                if self.check(TokenKind::Identifier) {
                    // Parse as potential arrow function parameters
                    let params = self.parse_arrow_params_or_expr()?;

                    if self.check(TokenKind::Arrow) {
                        self.advance();
                        let body = if self.check(TokenKind::OpenBrace) {
                            self.parse_block_statement()?
                        } else {
                            self.parse_expression()?
                        };
                        return Some(AstNode::ArrowFunctionExpression {
                            params,
                            body: Box::new(body),
                            is_async: false,
                        });
                    }

                    // It was a parenthesized expression after all
                    // params should have exactly one element which is the expr
                    if params.len() == 1 {
                        return Some(params.into_iter().next().unwrap());
                    }

                    // Multiple comma-separated expressions treated as sequence
                    // Just return the last one (simplified)
                    return params.into_iter().last();
                }

                // Just a parenthesized expression
                let expr = self.parse_expression()?;
                self.expect(TokenKind::CloseParen);

                // Check for arrow function after parenthesized expr
                if self.check(TokenKind::Arrow) {
                    self.advance();
                    let body = if self.check(TokenKind::OpenBrace) {
                        self.parse_block_statement()?
                    } else {
                        self.parse_expression()?
                    };
                    return Some(AstNode::ArrowFunctionExpression {
                        params: vec![expr],
                        body: Box::new(body),
                        is_async: false,
                    });
                }

                Some(expr)
            }
            TokenKind::OpenBracket => {
                self.advance();
                let mut elements = Vec::new();

                while !self.check(TokenKind::CloseBracket) && !self.check(TokenKind::Eof) {
                    if let Some(elem) = self.parse_expression() {
                        elements.push(elem);
                    }

                    if !self.check(TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }

                self.expect(TokenKind::CloseBracket);
                Some(AstNode::ArrayExpression { elements })
            }
            TokenKind::OpenBrace => {
                self.advance();
                let mut properties = Vec::new();

                while !self.check(TokenKind::CloseBrace) && !self.check(TokenKind::Eof) {
                    if let Some(prop) = self.parse_object_property() {
                        properties.push(prop);
                    }

                    if !self.check(TokenKind::Comma) {
                        break;
                    }
                    self.advance();
                }

                self.expect(TokenKind::CloseBrace);
                Some(AstNode::ObjectExpression { properties })
            }
            TokenKind::New => {
                self.advance();
                let callee = self.parse_primary_expression()?;

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
            TokenKind::Async => {
                self.advance();
                if self.check(TokenKind::Function) {
                    self.parse_function_declaration()
                } else {
                    // Async arrow function
                    self.error("Expected function after async".to_string());
                    None
                }
            }
            _ => {
                self.error(format!("Unexpected token: {:?}", self.current_token.kind));
                None
            }
        }
    }

    /// Parse object property
    fn parse_object_property(&mut self) -> Option<AstNode<'a>> {
        let key = if self.check(TokenKind::Identifier) || self.check(TokenKind::StringLiteral) {
            let name = self.current_token.text;
            self.advance();
            AstNode::Identifier { name }
        } else {
            return None;
        };

        // Shorthand property
        if !self.check(TokenKind::Colon) {
            return Some(AstNode::Property {
                key: Box::new(key.clone()),
                value: Box::new(key),
                shorthand: true,
            });
        }

        self.advance(); // consume ':'
        let value = self.parse_expression()?;

        Some(AstNode::Property {
            key: Box::new(key),
            value: Box::new(value),
            shorthand: false,
        })
    }

    /// Skip to next statement (for error recovery)
    pub fn skip_to_next_statement(&mut self) {
        while !self.check(TokenKind::Eof) {
            if self.check(TokenKind::Semicolon) {
                self.advance();
                return;
            }
            if self.check(TokenKind::CloseBrace) {
                return;
            }
            if self.check_any(&[
                TokenKind::Let,
                TokenKind::Const,
                TokenKind::Var,
                TokenKind::Function,
                TokenKind::Class,
                TokenKind::If,
                TokenKind::For,
                TokenKind::While,
                TokenKind::Return,
                TokenKind::Import,
                TokenKind::Export,
            ]) {
                return;
            }
            self.advance();
        }
    }

    /// Get current token kind
    pub fn current_kind(&self) -> TokenKind {
        self.current_token.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_variable_declaration() {
        let source = "let x = 5;";
        let mut parser = ThinParser::new(source);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_function() {
        let source = "function add(a: number, b: number): number { return a + b; }";
        let mut parser = ThinParser::new(source);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_class() {
        let source = "class Foo { }";
        let mut parser = ThinParser::new(source);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_interface() {
        let source = "interface Bar { }";
        let mut parser = ThinParser::new(source);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_expression() {
        let source = "1 + 2 * 3;";
        let mut parser = ThinParser::new(source);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_statement() {
        let source = "if (x > 0) { return x; }";
        let mut parser = ThinParser::new(source);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_import() {
        let source = r#"import { foo } from "bar";"#;
        let mut parser = ThinParser::new(source);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_recovery() {
        let source = "let x = ; let y = 5;";
        let mut parser = ThinParser::new(source);
        let _result = parser.parse();
        // Should recover and parse second declaration
        assert!(!parser.errors().is_empty());
    }
}
