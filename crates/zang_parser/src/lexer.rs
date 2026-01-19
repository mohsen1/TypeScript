//! Lexer for TypeScript source code
//!
//! Provides tokenization of TypeScript source code.

use crate::ast::SyntaxKind;
use zang_core::Span;

/// A token produced by the lexer
#[derive(Clone, Debug)]
pub struct Token {
    pub kind: SyntaxKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: SyntaxKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Lexer for TypeScript source code
pub struct Lexer<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    token_start: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            token_start: 0,
        }
    }

    /// Returns the next token
    pub fn next_token(&mut self) -> Token {
        self.skip_trivia();
        self.token_start = self.pos;

        if self.is_eof() {
            return Token::new(SyntaxKind::EndOfFile, self.current_span());
        }

        let c = self.current_byte();

        // Identifiers and keywords
        if is_identifier_start(c) {
            return self.scan_identifier();
        }

        // Numbers
        if c.is_ascii_digit() {
            return self.scan_number();
        }

        // Strings
        if c == b'"' || c == b'\'' {
            return self.scan_string(c);
        }

        // Template literals
        if c == b'`' {
            return self.scan_template_literal();
        }

        // Punctuation
        self.scan_punctuation()
    }

    /// Skips whitespace and comments
    fn skip_trivia(&mut self) {
        while !self.is_eof() {
            let c = self.current_byte();

            if c == b' ' || c == b'\t' || c == b'\r' || c == b'\n' {
                self.advance();
            }
            else if c == b'/' && self.peek_byte() == Some(b'/') {
                self.skip_line_comment();
            }
            else if c == b'/' && self.peek_byte() == Some(b'*') {
                self.skip_block_comment();
            }
            else {
                break;
            }
        }
    }

    /// Skips a line comment
    fn skip_line_comment(&mut self) {
        self.advance(); // /
        self.advance(); // /
        while !self.is_eof() && self.current_byte() != b'\n' {
            self.advance();
        }
    }

    /// Skips a block comment
    fn skip_block_comment(&mut self) {
        self.advance(); // /
        self.advance(); // *
        while !self.is_eof() {
            if self.current_byte() == b'*' && self.peek_byte() == Some(b'/') {
                self.advance();
                self.advance();
                break;
            }
            self.advance();
        }
    }

    /// Scans an identifier or keyword
    fn scan_identifier(&mut self) -> Token {
        while !self.is_eof() && is_identifier_part(self.current_byte()) {
            self.advance();
        }

        let text = &self.source[self.token_start..self.pos];
        let kind = keyword_kind(text).unwrap_or(SyntaxKind::Identifier);

        Token::new(kind, self.current_span())
    }

    /// Scans a number
    fn scan_number(&mut self) -> Token {
        // Handle hex, binary, octal
        if self.current_byte() == b'0' {
            if let Some(next) = self.peek_byte() {
                match next {
                    b'x' | b'X' => return self.scan_hex_number(),
                    b'b' | b'B' => return self.scan_binary_number(),
                    b'o' | b'O' => return self.scan_octal_number(),
                    _ => {}
                }
            }
        }

        self.scan_decimal_number()
    }

    fn scan_decimal_number(&mut self) -> Token {
        while !self.is_eof() && self.current_byte().is_ascii_digit() {
            self.advance();
        }

        // Decimal point
        if !self.is_eof() && self.current_byte() == b'.' {
            if let Some(next) = self.peek_byte() {
                if next.is_ascii_digit() {
                    self.advance(); // .
                    while !self.is_eof() && self.current_byte().is_ascii_digit() {
                        self.advance();
                    }
                }
            }
        }

        // Exponent
        if !self.is_eof() && (self.current_byte() == b'e' || self.current_byte() == b'E') {
            self.advance();
            if !self.is_eof() && (self.current_byte() == b'+' || self.current_byte() == b'-') {
                self.advance();
            }
            while !self.is_eof() && self.current_byte().is_ascii_digit() {
                self.advance();
            }
        }

        // BigInt suffix
        if !self.is_eof() && self.current_byte() == b'n' {
            self.advance();
            return Token::new(SyntaxKind::BigIntLiteral, self.current_span());
        }

        Token::new(SyntaxKind::NumericLiteral, self.current_span())
    }

    fn scan_hex_number(&mut self) -> Token {
        self.advance(); // 0
        self.advance(); // x
        while !self.is_eof() && self.current_byte().is_ascii_hexdigit() {
            self.advance();
        }
        if !self.is_eof() && self.current_byte() == b'n' {
            self.advance();
            return Token::new(SyntaxKind::BigIntLiteral, self.current_span());
        }
        Token::new(SyntaxKind::NumericLiteral, self.current_span())
    }

    fn scan_binary_number(&mut self) -> Token {
        self.advance(); // 0
        self.advance(); // b
        while !self.is_eof() && (self.current_byte() == b'0' || self.current_byte() == b'1') {
            self.advance();
        }
        if !self.is_eof() && self.current_byte() == b'n' {
            self.advance();
            return Token::new(SyntaxKind::BigIntLiteral, self.current_span());
        }
        Token::new(SyntaxKind::NumericLiteral, self.current_span())
    }

    fn scan_octal_number(&mut self) -> Token {
        self.advance(); // 0
        self.advance(); // o
        while !self.is_eof() && self.current_byte() >= b'0' && self.current_byte() <= b'7' {
            self.advance();
        }
        if !self.is_eof() && self.current_byte() == b'n' {
            self.advance();
            return Token::new(SyntaxKind::BigIntLiteral, self.current_span());
        }
        Token::new(SyntaxKind::NumericLiteral, self.current_span())
    }

    /// Scans a string literal
    fn scan_string(&mut self, quote: u8) -> Token {
        self.advance(); // opening quote

        while !self.is_eof() {
            let c = self.current_byte();
            if c == quote {
                self.advance();
                break;
            }
            if c == b'\\' {
                self.advance(); // backslash
                if !self.is_eof() {
                    self.advance(); // escaped char
                }
            }
            else {
                self.advance();
            }
        }

        Token::new(SyntaxKind::StringLiteral, self.current_span())
    }

    /// Scans a template literal
    fn scan_template_literal(&mut self) -> Token {
        self.advance(); // `

        while !self.is_eof() {
            let c = self.current_byte();
            if c == b'`' {
                self.advance();
                return Token::new(SyntaxKind::NoSubstitutionTemplateLiteral, self.current_span());
            }
            if c == b'$' && self.peek_byte() == Some(b'{') {
                return Token::new(SyntaxKind::TemplateHead, self.current_span());
            }
            if c == b'\\' {
                self.advance();
                if !self.is_eof() {
                    self.advance();
                }
            }
            else {
                self.advance();
            }
        }

        Token::new(SyntaxKind::NoSubstitutionTemplateLiteral, self.current_span())
    }

    /// Scans punctuation
    fn scan_punctuation(&mut self) -> Token {
        let c = self.current_byte();
        self.advance();

        let kind = match c {
            b'{' => SyntaxKind::OpenBrace,
            b'}' => SyntaxKind::CloseBrace,
            b'(' => SyntaxKind::OpenParen,
            b')' => SyntaxKind::CloseParen,
            b'[' => SyntaxKind::OpenBracket,
            b']' => SyntaxKind::CloseBracket,
            b';' => SyntaxKind::Semicolon,
            b',' => SyntaxKind::Comma,
            b':' => SyntaxKind::Colon,
            b'@' => SyntaxKind::At,
            b'#' => SyntaxKind::Hash,
            b'~' => SyntaxKind::Tilde,

            b'.' => {
                if self.match_byte(b'.') && self.match_byte(b'.') {
                    SyntaxKind::DotDotDot
                }
                else {
                    SyntaxKind::Dot
                }
            }

            b'<' => {
                if self.match_byte(b'<') {
                    if self.match_byte(b'=') {
                        SyntaxKind::LessThanLessThanEquals
                    }
                    else {
                        SyntaxKind::LessThanLessThan
                    }
                }
                else if self.match_byte(b'=') {
                    SyntaxKind::LessThanEquals
                }
                else {
                    SyntaxKind::LessThan
                }
            }

            b'>' => {
                if self.match_byte(b'>') {
                    if self.match_byte(b'>') {
                        if self.match_byte(b'=') {
                            SyntaxKind::GreaterThanGreaterThanGreaterThanEquals
                        }
                        else {
                            SyntaxKind::GreaterThanGreaterThanGreaterThan
                        }
                    }
                    else if self.match_byte(b'=') {
                        SyntaxKind::GreaterThanGreaterThanEquals
                    }
                    else {
                        SyntaxKind::GreaterThanGreaterThan
                    }
                }
                else if self.match_byte(b'=') {
                    SyntaxKind::GreaterThanEquals
                }
                else {
                    SyntaxKind::GreaterThan
                }
            }

            b'=' => {
                if self.match_byte(b'=') {
                    if self.match_byte(b'=') {
                        SyntaxKind::EqualsEqualsEquals
                    }
                    else {
                        SyntaxKind::EqualsEquals
                    }
                }
                else if self.match_byte(b'>') {
                    SyntaxKind::EqualsGreaterThan
                }
                else {
                    SyntaxKind::Equals
                }
            }

            b'!' => {
                if self.match_byte(b'=') {
                    if self.match_byte(b'=') {
                        SyntaxKind::ExclamationEqualsEquals
                    }
                    else {
                        SyntaxKind::ExclamationEquals
                    }
                }
                else {
                    SyntaxKind::Exclamation
                }
            }

            b'+' => {
                if self.match_byte(b'+') {
                    SyntaxKind::PlusPlus
                }
                else if self.match_byte(b'=') {
                    SyntaxKind::PlusEquals
                }
                else {
                    SyntaxKind::Plus
                }
            }

            b'-' => {
                if self.match_byte(b'-') {
                    SyntaxKind::MinusMinus
                }
                else if self.match_byte(b'=') {
                    SyntaxKind::MinusEquals
                }
                else {
                    SyntaxKind::Minus
                }
            }

            b'*' => {
                if self.match_byte(b'*') {
                    if self.match_byte(b'=') {
                        SyntaxKind::AsteriskAsteriskEquals
                    }
                    else {
                        SyntaxKind::AsteriskAsterisk
                    }
                }
                else if self.match_byte(b'=') {
                    SyntaxKind::AsteriskEquals
                }
                else {
                    SyntaxKind::Asterisk
                }
            }

            b'/' => {
                if self.match_byte(b'=') {
                    SyntaxKind::SlashEquals
                }
                else {
                    SyntaxKind::Slash
                }
            }

            b'%' => {
                if self.match_byte(b'=') {
                    SyntaxKind::PercentEquals
                }
                else {
                    SyntaxKind::Percent
                }
            }

            b'&' => {
                if self.match_byte(b'&') {
                    if self.match_byte(b'=') {
                        SyntaxKind::AmpersandAmpersandEquals
                    }
                    else {
                        SyntaxKind::AmpersandAmpersand
                    }
                }
                else if self.match_byte(b'=') {
                    SyntaxKind::AmpersandEquals
                }
                else {
                    SyntaxKind::Ampersand
                }
            }

            b'|' => {
                if self.match_byte(b'|') {
                    if self.match_byte(b'=') {
                        SyntaxKind::BarBarEquals
                    }
                    else {
                        SyntaxKind::BarBar
                    }
                }
                else if self.match_byte(b'=') {
                    SyntaxKind::BarEquals
                }
                else {
                    SyntaxKind::Bar
                }
            }

            b'^' => {
                if self.match_byte(b'=') {
                    SyntaxKind::CaretEquals
                }
                else {
                    SyntaxKind::Caret
                }
            }

            b'?' => {
                if self.match_byte(b'?') {
                    if self.match_byte(b'=') {
                        SyntaxKind::QuestionQuestionEquals
                    }
                    else {
                        SyntaxKind::QuestionQuestion
                    }
                }
                else if self.match_byte(b'.') {
                    SyntaxKind::QuestionDot
                }
                else {
                    SyntaxKind::Question
                }
            }

            _ => SyntaxKind::Unknown,
        };

        Token::new(kind, self.current_span())
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn current_byte(&self) -> u8 {
        self.bytes[self.pos]
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos + 1).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn match_byte(&mut self, expected: u8) -> bool {
        if !self.is_eof() && self.current_byte() == expected {
            self.advance();
            true
        }
        else {
            false
        }
    }

    fn current_span(&self) -> Span {
        Span::new(self.token_start as u32, self.pos as u32)
    }
}

fn is_identifier_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'$'
}

fn is_identifier_part(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

fn keyword_kind(text: &str) -> Option<SyntaxKind> {
    match text {
        "break" => Some(SyntaxKind::BreakKeyword),
        "case" => Some(SyntaxKind::CaseKeyword),
        "catch" => Some(SyntaxKind::CatchKeyword),
        "class" => Some(SyntaxKind::ClassKeyword),
        "const" => Some(SyntaxKind::ConstKeyword),
        "continue" => Some(SyntaxKind::ContinueKeyword),
        "debugger" => Some(SyntaxKind::DebuggerKeyword),
        "default" => Some(SyntaxKind::DefaultKeyword),
        "delete" => Some(SyntaxKind::DeleteKeyword),
        "do" => Some(SyntaxKind::DoKeyword),
        "else" => Some(SyntaxKind::ElseKeyword),
        "enum" => Some(SyntaxKind::EnumKeyword),
        "export" => Some(SyntaxKind::ExportKeyword),
        "extends" => Some(SyntaxKind::ExtendsKeyword),
        "false" => Some(SyntaxKind::FalseKeyword),
        "finally" => Some(SyntaxKind::FinallyKeyword),
        "for" => Some(SyntaxKind::ForKeyword),
        "function" => Some(SyntaxKind::FunctionKeyword),
        "if" => Some(SyntaxKind::IfKeyword),
        "import" => Some(SyntaxKind::ImportKeyword),
        "in" => Some(SyntaxKind::InKeyword),
        "instanceof" => Some(SyntaxKind::InstanceOfKeyword),
        "new" => Some(SyntaxKind::NewKeyword),
        "null" => Some(SyntaxKind::NullKeyword),
        "return" => Some(SyntaxKind::ReturnKeyword),
        "super" => Some(SyntaxKind::SuperKeyword),
        "switch" => Some(SyntaxKind::SwitchKeyword),
        "this" => Some(SyntaxKind::ThisKeyword),
        "throw" => Some(SyntaxKind::ThrowKeyword),
        "true" => Some(SyntaxKind::TrueKeyword),
        "try" => Some(SyntaxKind::TryKeyword),
        "typeof" => Some(SyntaxKind::TypeOfKeyword),
        "var" => Some(SyntaxKind::VarKeyword),
        "void" => Some(SyntaxKind::VoidKeyword),
        "while" => Some(SyntaxKind::WhileKeyword),
        "with" => Some(SyntaxKind::WithKeyword),
        // Strict mode
        "implements" => Some(SyntaxKind::ImplementsKeyword),
        "interface" => Some(SyntaxKind::InterfaceKeyword),
        "let" => Some(SyntaxKind::LetKeyword),
        "package" => Some(SyntaxKind::PackageKeyword),
        "private" => Some(SyntaxKind::PrivateKeyword),
        "protected" => Some(SyntaxKind::ProtectedKeyword),
        "public" => Some(SyntaxKind::PublicKeyword),
        "static" => Some(SyntaxKind::StaticKeyword),
        "yield" => Some(SyntaxKind::YieldKeyword),
        // Contextual
        "abstract" => Some(SyntaxKind::AbstractKeyword),
        "as" => Some(SyntaxKind::AsKeyword),
        "asserts" => Some(SyntaxKind::AssertsKeyword),
        "any" => Some(SyntaxKind::AnyKeyword),
        "async" => Some(SyntaxKind::AsyncKeyword),
        "await" => Some(SyntaxKind::AwaitKeyword),
        "boolean" => Some(SyntaxKind::BooleanKeyword),
        "constructor" => Some(SyntaxKind::ConstructorKeyword),
        "declare" => Some(SyntaxKind::DeclareKeyword),
        "get" => Some(SyntaxKind::GetKeyword),
        "infer" => Some(SyntaxKind::InferKeyword),
        "is" => Some(SyntaxKind::IsKeyword),
        "keyof" => Some(SyntaxKind::KeyOfKeyword),
        "module" => Some(SyntaxKind::ModuleKeyword),
        "namespace" => Some(SyntaxKind::NamespaceKeyword),
        "never" => Some(SyntaxKind::NeverKeyword),
        "readonly" => Some(SyntaxKind::ReadonlyKeyword),
        "require" => Some(SyntaxKind::RequireKeyword),
        "number" => Some(SyntaxKind::NumberKeyword),
        "object" => Some(SyntaxKind::ObjectKeyword),
        "set" => Some(SyntaxKind::SetKeyword),
        "string" => Some(SyntaxKind::StringKeyword),
        "symbol" => Some(SyntaxKind::SymbolKeyword),
        "type" => Some(SyntaxKind::TypeKeyword),
        "undefined" => Some(SyntaxKind::UndefinedKeyword),
        "unique" => Some(SyntaxKind::UniqueKeyword),
        "unknown" => Some(SyntaxKind::UnknownKeyword),
        "from" => Some(SyntaxKind::FromKeyword),
        "global" => Some(SyntaxKind::GlobalKeyword),
        "bigint" => Some(SyntaxKind::BigIntKeyword),
        "override" => Some(SyntaxKind::OverrideKeyword),
        "of" => Some(SyntaxKind::OfKeyword),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_identifier() {
        let mut lexer = Lexer::new("hello world");
        let token = lexer.next_token();
        assert_eq!(token.kind, SyntaxKind::Identifier);

        let token = lexer.next_token();
        assert_eq!(token.kind, SyntaxKind::Identifier);
    }

    #[test]
    fn test_scan_keyword() {
        let mut lexer = Lexer::new("function const let");
        assert_eq!(lexer.next_token().kind, SyntaxKind::FunctionKeyword);
        assert_eq!(lexer.next_token().kind, SyntaxKind::ConstKeyword);
        assert_eq!(lexer.next_token().kind, SyntaxKind::LetKeyword);
    }

    #[test]
    fn test_scan_number() {
        let mut lexer = Lexer::new("42 3.14 1e10");
        assert_eq!(lexer.next_token().kind, SyntaxKind::NumericLiteral);
        assert_eq!(lexer.next_token().kind, SyntaxKind::NumericLiteral);
        assert_eq!(lexer.next_token().kind, SyntaxKind::NumericLiteral);
    }

    #[test]
    fn test_scan_string() {
        let mut lexer = Lexer::new("\"hello\" 'world'");
        assert_eq!(lexer.next_token().kind, SyntaxKind::StringLiteral);
        assert_eq!(lexer.next_token().kind, SyntaxKind::StringLiteral);
    }

    #[test]
    fn test_scan_punctuation() {
        let mut lexer = Lexer::new("{ } ( ) ===");
        assert_eq!(lexer.next_token().kind, SyntaxKind::OpenBrace);
        assert_eq!(lexer.next_token().kind, SyntaxKind::CloseBrace);
        assert_eq!(lexer.next_token().kind, SyntaxKind::OpenParen);
        assert_eq!(lexer.next_token().kind, SyntaxKind::CloseParen);
        assert_eq!(lexer.next_token().kind, SyntaxKind::EqualsEqualsEquals);
    }

    #[test]
    fn test_skip_comments() {
        let mut lexer = Lexer::new("hello // comment\nworld /* block */ end");
        assert_eq!(lexer.next_token().kind, SyntaxKind::Identifier);
        assert_eq!(lexer.next_token().kind, SyntaxKind::Identifier);
        assert_eq!(lexer.next_token().kind, SyntaxKind::Identifier);
    }
}
