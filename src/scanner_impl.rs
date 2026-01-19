// Scanner implementation details and utilities
// This module contains additional scanning functionality and helpers

use crate::scanner::Scanner;
use crate::tokens::{Token, TokenKind};

/// Result of scanning all tokens from source
#[derive(Debug)]
pub struct ScanResult<'a> {
    /// Source text reference
    pub source: &'a str,
    /// All scanned tokens (excluding trivia)
    pub tokens: Vec<Token>,
    /// All trivia tokens (comments, whitespace)
    pub trivia: Vec<Token>,
    /// Whether any errors occurred
    pub has_errors: bool,
}

impl<'a> ScanResult<'a> {
    /// Get token text from source
    pub fn token_text(&self, token: &Token) -> &'a str {
        &self.source[token.span.start..token.span.end]
    }
}

/// Scan all tokens from source, separating tokens from trivia
pub fn scan_all_tokens(source: &str) -> ScanResult<'_> {
    let mut scanner = Scanner::new(source);
    let mut tokens = Vec::new();
    let mut trivia = Vec::new();
    let mut has_errors = false;

    loop {
        let kind = scanner.scan();

        if kind == TokenKind::EndOfFile {
            break;
        }

        if scanner.is_unterminated() {
            has_errors = true;
        }

        let token = scanner.get_current_token();

        if kind.is_trivia() {
            trivia.push(token);
        } else {
            tokens.push(token);
        }
    }

    ScanResult {
        source,
        tokens,
        trivia,
        has_errors,
    }
}

/// Scan source and return only non-trivia tokens
pub fn scan_tokens(source: &str) -> Vec<Token> {
    let mut scanner = Scanner::new(source);
    let mut tokens = Vec::new();

    loop {
        let kind = scanner.scan();

        if kind == TokenKind::EndOfFile {
            break;
        }

        if !kind.is_trivia() {
            tokens.push(scanner.get_current_token());
        }
    }

    tokens
}

/// Iterator over tokens in source
pub struct TokenIterator<'a> {
    scanner: Scanner<'a>,
    include_trivia: bool,
    done: bool,
}

impl<'a> TokenIterator<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
            include_trivia: false,
            done: false,
        }
    }

    pub fn with_trivia(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
            include_trivia: true,
            done: false,
        }
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        loop {
            let kind = self.scanner.scan();

            if kind == TokenKind::EndOfFile {
                self.done = true;
                return None;
            }

            if self.include_trivia || !kind.is_trivia() {
                return Some(self.scanner.get_current_token());
            }
        }
    }
}

/// Compute line starts (positions of each line beginning) for source text
pub fn compute_line_starts(text: &str) -> Vec<usize> {
    let mut result = vec![0];
    let bytes = text.as_bytes();
    let mut pos = 0;

    while pos < bytes.len() {
        let ch = bytes[pos];
        pos += 1;

        match ch {
            b'\r' => {
                if pos < bytes.len() && bytes[pos] == b'\n' {
                    pos += 1;
                }
                result.push(pos);
            }
            b'\n' => {
                result.push(pos);
            }
            // Handle other Unicode line terminators
            0xE2 if pos + 1 < bytes.len() => {
                // Line separator (U+2028) and Paragraph separator (U+2029)
                // UTF-8: E2 80 A8 and E2 80 A9
                if bytes[pos] == 0x80 && (bytes[pos + 1] == 0xA8 || bytes[pos + 1] == 0xA9) {
                    pos += 2;
                    result.push(pos);
                }
            }
            _ => {}
        }
    }

    result
}

/// Get line and character position for a given offset
pub fn get_line_and_character(text: &str, offset: usize) -> (usize, usize) {
    let line_starts = compute_line_starts(text);
    let mut line = 0;

    for (i, &start) in line_starts.iter().enumerate() {
        if start > offset {
            break;
        }
        line = i;
    }

    let character = offset - line_starts[line];
    (line, character)
}

/// Get position from line and character
pub fn get_position_of_line_and_character(
    line_starts: &[usize],
    line: usize,
    character: usize,
) -> usize {
    if line >= line_starts.len() {
        return line_starts.last().copied().unwrap_or(0);
    }
    line_starts[line] + character
}

/// Check if character is a line break
#[inline]
pub fn is_line_break(ch: char) -> bool {
    matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// Check if character is whitespace (excluding line breaks)
#[inline]
pub fn is_white_space_single_line(ch: char) -> bool {
    matches!(
        ch,
        ' ' | '\t'
            | '\u{000B}'  // vertical tab
            | '\u{000C}'  // form feed
            | '\u{00A0}'  // non-breaking space
            | '\u{1680}'  // ogham space mark
            | '\u{2000}'..='\u{200A}'  // various spaces
            | '\u{202F}'  // narrow no-break space
            | '\u{205F}'  // medium mathematical space
            | '\u{3000}'  // ideographic space
            | '\u{FEFF}'  // byte order mark
    )
}

/// Token precedence for operator parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None = 0,
    Comma = 1,
    Assignment = 2,
    Conditional = 3,
    LogicalOr = 4,
    LogicalAnd = 5,
    BitwiseOr = 6,
    BitwiseXor = 7,
    BitwiseAnd = 8,
    Equality = 9,
    Relational = 10,
    Shift = 11,
    Additive = 12,
    Multiplicative = 13,
    Exponentiation = 14,
    Unary = 15,
    Postfix = 16,
    Member = 17,
    Primary = 18,
}

/// Get binary operator precedence
pub fn get_binary_operator_precedence(kind: TokenKind) -> Precedence {
    match kind {
        TokenKind::BarBar => Precedence::LogicalOr,
        TokenKind::QuestionQuestion => Precedence::LogicalOr, // Same as ||
        TokenKind::AmpersandAmpersand => Precedence::LogicalAnd,
        TokenKind::Bar => Precedence::BitwiseOr,
        TokenKind::Caret => Precedence::BitwiseXor,
        TokenKind::Ampersand => Precedence::BitwiseAnd,
        TokenKind::EqualsEquals
        | TokenKind::ExclamationEquals
        | TokenKind::EqualsEqualsEquals
        | TokenKind::ExclamationEqualsEquals => Precedence::Equality,
        TokenKind::LessThan
        | TokenKind::GreaterThan
        | TokenKind::LessThanEquals
        | TokenKind::GreaterThanEquals
        | TokenKind::InstanceOf
        | TokenKind::In
        | TokenKind::As => Precedence::Relational,
        TokenKind::LessThanLessThan
        | TokenKind::GreaterThanGreaterThan
        | TokenKind::GreaterThanGreaterThanGreaterThan => Precedence::Shift,
        TokenKind::Plus | TokenKind::Minus => Precedence::Additive,
        TokenKind::Asterisk | TokenKind::Slash | TokenKind::Percent => Precedence::Multiplicative,
        TokenKind::AsteriskAsterisk => Precedence::Exponentiation,
        _ => Precedence::None,
    }
}

/// Check if token is an assignment operator
pub fn is_assignment_operator(kind: TokenKind) -> bool {
    kind.is_assignment_operator()
}

/// Diagnostic message structure
#[derive(Debug, Clone)]
pub struct DiagnosticMessage {
    pub message: String,
    pub start: usize,
    pub length: usize,
    pub line: usize,
    pub character: usize,
}

/// Scanner with diagnostic collection
pub struct DiagnosticScanner<'a> {
    scanner: Scanner<'a>,
    diagnostics: Vec<DiagnosticMessage>,
}

impl<'a> DiagnosticScanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let scanner = Scanner::new(source);
        Self {
            scanner,
            diagnostics: Vec::new(),
        }
    }

    pub fn scan(&mut self) -> TokenKind {
        let kind = self.scanner.scan();

        if self.scanner.is_unterminated() {
            let pos = self.scanner.get_token_pos();
            let len = self.scanner.get_text_pos() - pos;
            let (line, character) = get_line_and_character(self.scanner.get_text(), pos);

            self.diagnostics.push(DiagnosticMessage {
                message: format!("Unterminated {:?}", kind),
                start: pos,
                length: len,
                line,
                character,
            });
        }

        kind
    }

    pub fn get_diagnostics(&self) -> &[DiagnosticMessage] {
        &self.diagnostics
    }

    pub fn get_scanner(&self) -> &Scanner<'a> {
        &self.scanner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_all_tokens() {
        let source = "let x = 42; // comment";
        let result = scan_all_tokens(source);

        assert!(!result.has_errors);
        assert_eq!(result.tokens.len(), 5); // let, x, =, 42, ;
        assert_eq!(result.trivia.len(), 1); // comment
    }

    #[test]
    fn test_token_iterator() {
        let source = "let x = 42";
        let tokens: Vec<_> = TokenIterator::new(source).collect();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].kind, TokenKind::Equals);
        assert_eq!(tokens[3].kind, TokenKind::NumericLiteral);
    }

    #[test]
    fn test_token_iterator_with_trivia() {
        let source = "let /* comment */ x";
        let tokens: Vec<_> = TokenIterator::with_trivia(source).collect();

        assert!(tokens.iter().any(|t| t.kind == TokenKind::MultiLineComment));
    }

    #[test]
    fn test_compute_line_starts() {
        let text = "line1\nline2\r\nline3";
        let starts = compute_line_starts(text);

        assert_eq!(starts, vec![0, 6, 13]);
    }

    #[test]
    fn test_get_line_and_character() {
        let text = "let x\nconst y";
        let (line, char) = get_line_and_character(text, 7); // 'o' in 'const'

        assert_eq!(line, 1);
        assert_eq!(char, 1);
    }

    #[test]
    fn test_binary_operator_precedence() {
        assert!(get_binary_operator_precedence(TokenKind::Asterisk)
            > get_binary_operator_precedence(TokenKind::Plus));
        assert!(get_binary_operator_precedence(TokenKind::AsteriskAsterisk)
            > get_binary_operator_precedence(TokenKind::Asterisk));
        assert!(get_binary_operator_precedence(TokenKind::AmpersandAmpersand)
            > get_binary_operator_precedence(TokenKind::BarBar));
    }

    #[test]
    fn test_diagnostic_scanner() {
        let source = "let x = \"unterminated";
        let mut scanner = DiagnosticScanner::new(source);

        loop {
            let kind = scanner.scan();
            if kind == TokenKind::EndOfFile {
                break;
            }
        }

        let diagnostics = scanner.get_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("Unterminated"));
    }

    #[test]
    fn test_is_white_space() {
        assert!(is_white_space_single_line(' '));
        assert!(is_white_space_single_line('\t'));
        assert!(!is_white_space_single_line('\n'));
        assert!(!is_white_space_single_line('a'));
    }

    #[test]
    fn test_is_line_break() {
        assert!(is_line_break('\n'));
        assert!(is_line_break('\r'));
        assert!(is_line_break('\u{2028}'));
        assert!(!is_line_break(' '));
    }

    #[test]
    fn test_scan_typescript_interface() {
        let source = r#"
interface Point {
    x: number;
    y: number;
}
"#;
        let tokens = scan_tokens(source);
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();

        assert!(kinds.contains(&TokenKind::Interface));
        assert!(kinds.contains(&TokenKind::Identifier));
        assert!(kinds.contains(&TokenKind::OpenBrace));
        assert!(kinds.contains(&TokenKind::Colon));
        assert!(kinds.contains(&TokenKind::Number));
        assert!(kinds.contains(&TokenKind::Semicolon));
        assert!(kinds.contains(&TokenKind::CloseBrace));
    }

    #[test]
    fn test_scan_typescript_generics() {
        let source = "Array<string>";
        let tokens = scan_tokens(source);

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::LessThan);
        assert_eq!(tokens[2].kind, TokenKind::String);
        assert_eq!(tokens[3].kind, TokenKind::GreaterThan);
    }

    #[test]
    fn test_scan_optional_chaining() {
        let source = "obj?.property";
        let tokens = scan_tokens(source);

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::QuestionDot);
        assert_eq!(tokens[2].kind, TokenKind::Identifier);
    }

    #[test]
    fn test_scan_nullish_coalescing() {
        let source = "a ?? b";
        let tokens = scan_tokens(source);

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::QuestionQuestion);
        assert_eq!(tokens[2].kind, TokenKind::Identifier);
    }

    #[test]
    fn test_scan_complex_typescript() {
        let source = r#"
async function fetchData<T>(url: string): Promise<T> {
    const response = await fetch(url);
    return response.json() as T;
}
"#;
        let result = scan_all_tokens(source);

        assert!(!result.has_errors);

        let kinds: Vec<_> = result.tokens.iter().map(|t| t.kind).collect();
        assert!(kinds.contains(&TokenKind::Async));
        assert!(kinds.contains(&TokenKind::Function));
        assert!(kinds.contains(&TokenKind::LessThan));
        assert!(kinds.contains(&TokenKind::GreaterThan));
        assert!(kinds.contains(&TokenKind::Colon));
        assert!(kinds.contains(&TokenKind::Const));
        assert!(kinds.contains(&TokenKind::Await));
        assert!(kinds.contains(&TokenKind::Return));
        assert!(kinds.contains(&TokenKind::As));
    }

    #[test]
    fn test_zero_copy_verification() {
        let source = String::from("const value = 42;");
        let result = scan_all_tokens(&source);

        // Verify tokens reference original source
        for token in &result.tokens {
            let text = result.token_text(token);
            // Text should be a slice of original source
            assert!(source.contains(text));
        }
    }
}
