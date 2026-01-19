// TypeScript Scanner - Zero-copy implementation
// Takes &str reference (not owned String) and produces tokens with spans

use crate::tokens::{keyword_from_str, Span, Token, TokenKind};

/// Scanner configuration options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptTarget {
    ES3,
    ES5,
    ES2015,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ESNext,
}

impl Default for ScriptTarget {
    fn default() -> Self {
        ScriptTarget::ESNext
    }
}

/// Language variant for parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LanguageVariant {
    #[default]
    Standard,
    JSX,
}

/// Token flags providing additional information about tokens
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TokenFlags {
    bits: u32,
}

impl TokenFlags {
    pub const NONE: u32 = 0;
    pub const PRECEDING_LINE_BREAK: u32 = 1 << 0;
    pub const PRECEDING_JSX_LINE_BREAK: u32 = 1 << 1;
    pub const UNTERMINATED: u32 = 1 << 2;
    pub const EXTENDED_UNICODE_ESCAPE: u32 = 1 << 3;
    pub const SCIENTIFIC: u32 = 1 << 4;
    pub const OCTAL: u32 = 1 << 5;
    pub const HEX_SPECIFIER: u32 = 1 << 6;
    pub const BINARY_SPECIFIER: u32 = 1 << 7;
    pub const OCTAL_SPECIFIER: u32 = 1 << 8;
    pub const CONTAINS_SEPARATOR: u32 = 1 << 9;
    pub const UNICODE_ESCAPE: u32 = 1 << 10;
    pub const CONTAINS_INVALID_ESCAPE: u32 = 1 << 11;
    pub const BINARY_OR_OCTAL_SPECIFIER: u32 = Self::BINARY_SPECIFIER | Self::OCTAL_SPECIFIER;
    pub const NUMERIC_LITERAL_FLAGS: u32 = Self::SCIENTIFIC
        | Self::OCTAL
        | Self::HEX_SPECIFIER
        | Self::BINARY_OR_OCTAL_SPECIFIER
        | Self::CONTAINS_SEPARATOR;

    pub fn new() -> Self {
        Self { bits: 0 }
    }

    pub fn has(&self, flag: u32) -> bool {
        (self.bits & flag) != 0
    }

    pub fn set(&mut self, flag: u32) {
        self.bits |= flag;
    }

    pub fn clear(&mut self) {
        self.bits = 0;
    }
}

/// Error callback type for reporting scanner errors
pub type ErrorCallback = fn(message: &str, pos: usize, length: usize);

/// A zero-copy scanner for TypeScript/JavaScript source code
/// Takes a reference to the source string and produces tokens with spans
pub struct Scanner<'a> {
    /// The source text being scanned
    source: &'a str,
    /// Source as bytes for fast access
    bytes: &'a [u8],
    /// Current position in the source
    pos: usize,
    /// End position (for scanning subranges)
    end: usize,
    /// Start position of current scan range
    start_pos: usize,
    /// Position where token started
    token_pos: usize,
    /// Current token
    token: TokenKind,
    /// Token flags
    token_flags: TokenFlags,
    /// Script target version
    script_target: ScriptTarget,
    /// Language variant (standard or JSX)
    language_variant: LanguageVariant,
    /// Error callback
    on_error: Option<ErrorCallback>,
    /// Whether we're scanning in JSDoc
    in_jsdoc_type: bool,
}

impl<'a> Scanner<'a> {
    /// Create a new scanner for the given source text
    pub fn new(source: &'a str) -> Self {
        Self::with_options(source, ScriptTarget::default(), LanguageVariant::default())
    }

    /// Create a new scanner with specific options
    pub fn with_options(
        source: &'a str,
        script_target: ScriptTarget,
        language_variant: LanguageVariant,
    ) -> Self {
        let bytes = source.as_bytes();
        let end = source.len();
        Self {
            source,
            bytes,
            pos: 0,
            end,
            start_pos: 0,
            token_pos: 0,
            token: TokenKind::Unknown,
            token_flags: TokenFlags::new(),
            script_target,
            language_variant,
            on_error: None,
            in_jsdoc_type: false,
        }
    }

    /// Get the source text
    pub fn get_text(&self) -> &'a str {
        self.source
    }

    /// Get the start position of the current scan range
    pub fn get_start_pos(&self) -> usize {
        self.start_pos
    }

    /// Get current position in source
    pub fn get_text_pos(&self) -> usize {
        self.pos
    }

    /// Get position where current token started
    pub fn get_token_pos(&self) -> usize {
        self.token_pos
    }

    /// Get current token kind
    pub fn get_token(&self) -> TokenKind {
        self.token
    }

    /// Get the text of the current token (zero-copy slice)
    pub fn get_token_text(&self) -> &'a str {
        &self.source[self.token_pos..self.pos]
    }

    /// Get the current token as a Token struct
    pub fn get_current_token(&self) -> Token {
        Token::new(self.token, Span::new(self.token_pos, self.pos))
    }

    /// Get current token flags
    pub fn get_token_flags(&self) -> TokenFlags {
        self.token_flags
    }

    /// Check if there was a preceding line break
    pub fn has_preceding_line_break(&self) -> bool {
        self.token_flags.has(TokenFlags::PRECEDING_LINE_BREAK)
    }

    /// Check if current token is unterminated
    pub fn is_unterminated(&self) -> bool {
        self.token_flags.has(TokenFlags::UNTERMINATED)
    }

    /// Check if current token is an identifier
    pub fn is_identifier(&self) -> bool {
        self.token == TokenKind::Identifier || self.token.is_keyword()
    }

    /// Check if current token is a reserved word
    pub fn is_reserved_word(&self) -> bool {
        self.token.is_reserved_word()
    }

    /// Set error callback
    pub fn set_on_error(&mut self, callback: Option<ErrorCallback>) {
        self.on_error = callback;
    }

    /// Set script target version
    pub fn set_script_target(&mut self, target: ScriptTarget) {
        self.script_target = target;
    }

    /// Set language variant
    pub fn set_language_variant(&mut self, variant: LanguageVariant) {
        self.language_variant = variant;
    }

    /// Set text position
    pub fn set_text_pos(&mut self, pos: usize) {
        self.pos = pos;
        self.token_pos = pos;
        self.token = TokenKind::Unknown;
        self.token_flags.clear();
    }

    /// Set whether scanning in JSDoc type
    pub fn set_in_jsdoc_type(&mut self, in_type: bool) {
        self.in_jsdoc_type = in_type;
    }

    /// Set the text to scan, optionally with a subrange
    pub fn set_text(&mut self, text: &'a str, start: Option<usize>, length: Option<usize>) {
        self.source = text;
        self.bytes = text.as_bytes();
        self.end = length.map(|l| start.unwrap_or(0) + l).unwrap_or(text.len());
        self.pos = start.unwrap_or(0);
        self.start_pos = self.pos;
        self.token_pos = self.pos;
        self.token = TokenKind::Unknown;
        self.token_flags.clear();
    }

    /// Report an error
    #[allow(dead_code)]
    fn error(&self, message: &str, pos: usize, length: usize) {
        if let Some(callback) = self.on_error {
            callback(message, pos, length);
        }
    }

    // Helper methods for character access
    #[inline]
    fn current_char(&self) -> Option<u8> {
        if self.pos < self.end {
            Some(self.bytes[self.pos])
        } else {
            None
        }
    }

    #[inline]
    fn peek_char(&self, offset: usize) -> Option<u8> {
        let pos = self.pos + offset;
        if pos < self.end {
            Some(self.bytes[pos])
        } else {
            None
        }
    }

    #[inline]
    fn advance(&mut self) {
        self.pos += 1;
    }

    #[inline]
    fn advance_by(&mut self, n: usize) {
        self.pos += n;
    }

    /// Scan and return the next token
    pub fn scan(&mut self) -> TokenKind {
        self.token = self.scan_token();
        self.token
    }

    /// Main scanning logic - implemented in scanner_impl.rs
    fn scan_token(&mut self) -> TokenKind {
        self.token_flags.clear();

        loop {
            self.token_pos = self.pos;

            if self.pos >= self.end {
                return TokenKind::EndOfFile;
            }

            let ch = self.bytes[self.pos];

            match ch {
                // Whitespace
                b' ' | b'\t' | 0x0B | 0x0C => {
                    self.scan_whitespace();
                    continue;
                }

                // Line breaks
                b'\n' => {
                    self.advance();
                    self.token_flags.set(TokenFlags::PRECEDING_LINE_BREAK);
                    continue;
                }
                b'\r' => {
                    self.advance();
                    if self.current_char() == Some(b'\n') {
                        self.advance();
                    }
                    self.token_flags.set(TokenFlags::PRECEDING_LINE_BREAK);
                    continue;
                }

                // Shebang
                b'#' => {
                    if self.pos == self.start_pos && self.peek_char(1) == Some(b'!') {
                        return self.scan_shebang();
                    }
                    // Private identifier or hash token
                    return self.scan_hash_token();
                }

                // Identifiers and keywords
                b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => {
                    return self.scan_identifier();
                }

                // Numbers
                b'0'..=b'9' => {
                    return self.scan_number();
                }

                // Strings
                b'"' | b'\'' => {
                    return self.scan_string(ch);
                }

                // Template literals
                b'`' => {
                    return self.scan_template_literal();
                }

                // Operators and punctuation
                b'{' => {
                    self.advance();
                    return TokenKind::OpenBrace;
                }
                b'}' => {
                    self.advance();
                    return TokenKind::CloseBrace;
                }
                b'(' => {
                    self.advance();
                    return TokenKind::OpenParen;
                }
                b')' => {
                    self.advance();
                    return TokenKind::CloseParen;
                }
                b'[' => {
                    self.advance();
                    return TokenKind::OpenBracket;
                }
                b']' => {
                    self.advance();
                    return TokenKind::CloseBracket;
                }
                b';' => {
                    self.advance();
                    return TokenKind::Semicolon;
                }
                b',' => {
                    self.advance();
                    return TokenKind::Comma;
                }
                b':' => {
                    self.advance();
                    return TokenKind::Colon;
                }
                b'@' => {
                    self.advance();
                    return TokenKind::At;
                }
                b'~' => {
                    self.advance();
                    return TokenKind::Tilde;
                }

                b'.' => return self.scan_dot(),
                b'+' => return self.scan_plus(),
                b'-' => return self.scan_minus(),
                b'*' => return self.scan_asterisk(),
                b'/' => return self.scan_slash(),
                b'%' => return self.scan_percent(),
                b'<' => return self.scan_less_than(),
                b'>' => return self.scan_greater_than(),
                b'=' => return self.scan_equals(),
                b'!' => return self.scan_exclamation(),
                b'&' => return self.scan_ampersand(),
                b'|' => return self.scan_bar(),
                b'^' => return self.scan_caret(),
                b'?' => return self.scan_question(),

                // Handle extended ASCII and Unicode
                _ => {
                    // Check for Unicode identifier start
                    if self.is_identifier_start(self.pos) {
                        return self.scan_identifier();
                    }

                    // Check for Unicode whitespace/line break
                    if self.is_whitespace_like(ch) {
                        self.advance();
                        continue;
                    }

                    self.advance();
                    return TokenKind::Unknown;
                }
            }
        }
    }

    fn scan_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            match ch {
                b' ' | b'\t' | 0x0B | 0x0C => self.advance(),
                _ => break,
            }
        }
    }

    fn is_whitespace_like(&self, ch: u8) -> bool {
        // Handle common ASCII whitespace and some Unicode whitespace markers
        matches!(ch, 0xA0 | 0x85) // NBSP, NEL
    }

    fn scan_shebang(&mut self) -> TokenKind {
        self.advance_by(2); // skip #!
        while let Some(ch) = self.current_char() {
            if ch == b'\n' || ch == b'\r' {
                break;
            }
            self.advance();
        }
        TokenKind::Shebang
    }

    fn scan_hash_token(&mut self) -> TokenKind {
        self.advance(); // skip #
        if let Some(ch) = self.current_char() {
            if is_identifier_start_char(ch) {
                // Scan private identifier
                while let Some(ch) = self.current_char() {
                    if !is_identifier_char(ch) {
                        break;
                    }
                    self.advance();
                }
                return TokenKind::PrivateIdentifier;
            }
        }
        TokenKind::Hash
    }

    fn scan_identifier(&mut self) -> TokenKind {
        let start = self.pos;
        self.advance();

        while let Some(ch) = self.current_char() {
            if is_identifier_char(ch) {
                self.advance();
            } else if !ch.is_ascii() && self.is_identifier_part(self.pos) {
                self.advance_unicode_char();
            } else {
                break;
            }
        }

        let text = &self.source[start..self.pos];

        // Check if it's a keyword
        if let Some(kind) = keyword_from_str(text) {
            kind
        } else {
            TokenKind::Identifier
        }
    }

    fn is_identifier_start(&self, pos: usize) -> bool {
        if pos >= self.end {
            return false;
        }
        let ch = self.bytes[pos];
        if ch.is_ascii() {
            return is_identifier_start_char(ch);
        }
        // For non-ASCII, check Unicode identifier start
        // Simplified check - in production would use full Unicode tables
        true
    }

    fn is_identifier_part(&self, pos: usize) -> bool {
        if pos >= self.end {
            return false;
        }
        let ch = self.bytes[pos];
        if ch.is_ascii() {
            return is_identifier_char(ch);
        }
        // For non-ASCII, check Unicode identifier continue
        true
    }

    fn advance_unicode_char(&mut self) {
        // Advance past a UTF-8 character
        if self.pos >= self.end {
            return;
        }
        let first = self.bytes[self.pos];
        let len = if first < 0x80 {
            1
        } else if first < 0xE0 {
            2
        } else if first < 0xF0 {
            3
        } else {
            4
        };
        self.pos = (self.pos + len).min(self.end);
    }

    fn scan_number(&mut self) -> TokenKind {
        let first = self.bytes[self.pos];

        if first == b'0' {
            if let Some(next) = self.peek_char(1) {
                match next {
                    b'x' | b'X' => return self.scan_hex_number(),
                    b'b' | b'B' => return self.scan_binary_number(),
                    b'o' | b'O' => return self.scan_octal_number(),
                    _ => {}
                }
            }
        }

        // Scan integer part
        self.scan_digits();

        // Check for decimal part
        if self.current_char() == Some(b'.') {
            if let Some(next) = self.peek_char(1) {
                if next.is_ascii_digit() {
                    self.advance(); // skip .
                    self.scan_digits();
                }
            }
        }

        // Check for exponent
        if let Some(ch) = self.current_char() {
            if ch == b'e' || ch == b'E' {
                self.token_flags.set(TokenFlags::SCIENTIFIC);
                self.advance();
                if let Some(sign) = self.current_char() {
                    if sign == b'+' || sign == b'-' {
                        self.advance();
                    }
                }
                self.scan_digits();
            }
        }

        // Check for BigInt suffix
        if self.current_char() == Some(b'n') {
            self.advance();
            return TokenKind::BigIntLiteral;
        }

        TokenKind::NumericLiteral
    }

    fn scan_digits(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == b'_' {
                self.token_flags.set(TokenFlags::CONTAINS_SEPARATOR);
                self.advance();
            } else {
                break;
            }
        }
    }

    fn scan_hex_number(&mut self) -> TokenKind {
        self.token_flags.set(TokenFlags::HEX_SPECIFIER);
        self.advance_by(2); // skip 0x

        while let Some(ch) = self.current_char() {
            if ch.is_ascii_hexdigit() {
                self.advance();
            } else if ch == b'_' {
                self.token_flags.set(TokenFlags::CONTAINS_SEPARATOR);
                self.advance();
            } else {
                break;
            }
        }

        if self.current_char() == Some(b'n') {
            self.advance();
            return TokenKind::BigIntLiteral;
        }

        TokenKind::NumericLiteral
    }

    fn scan_binary_number(&mut self) -> TokenKind {
        self.token_flags.set(TokenFlags::BINARY_SPECIFIER);
        self.advance_by(2); // skip 0b

        while let Some(ch) = self.current_char() {
            if ch == b'0' || ch == b'1' {
                self.advance();
            } else if ch == b'_' {
                self.token_flags.set(TokenFlags::CONTAINS_SEPARATOR);
                self.advance();
            } else {
                break;
            }
        }

        if self.current_char() == Some(b'n') {
            self.advance();
            return TokenKind::BigIntLiteral;
        }

        TokenKind::NumericLiteral
    }

    fn scan_octal_number(&mut self) -> TokenKind {
        self.token_flags.set(TokenFlags::OCTAL_SPECIFIER);
        self.advance_by(2); // skip 0o

        while let Some(ch) = self.current_char() {
            if ch >= b'0' && ch <= b'7' {
                self.advance();
            } else if ch == b'_' {
                self.token_flags.set(TokenFlags::CONTAINS_SEPARATOR);
                self.advance();
            } else {
                break;
            }
        }

        if self.current_char() == Some(b'n') {
            self.advance();
            return TokenKind::BigIntLiteral;
        }

        TokenKind::NumericLiteral
    }

    fn scan_string(&mut self, quote: u8) -> TokenKind {
        self.advance(); // skip opening quote

        while let Some(ch) = self.current_char() {
            if ch == quote {
                self.advance();
                return TokenKind::StringLiteral;
            }
            if ch == b'\\' {
                self.advance();
                if self.current_char().is_some() {
                    self.advance();
                }
                continue;
            }
            if ch == b'\n' || ch == b'\r' {
                self.token_flags.set(TokenFlags::UNTERMINATED);
                return TokenKind::StringLiteral;
            }
            self.advance();
        }

        self.token_flags.set(TokenFlags::UNTERMINATED);
        TokenKind::StringLiteral
    }

    fn scan_template_literal(&mut self) -> TokenKind {
        self.advance(); // skip `

        while let Some(ch) = self.current_char() {
            if ch == b'`' {
                self.advance();
                return TokenKind::NoSubstitutionTemplateLiteral;
            }
            if ch == b'$' && self.peek_char(1) == Some(b'{') {
                self.advance_by(2);
                return TokenKind::TemplateHead;
            }
            if ch == b'\\' {
                self.advance();
                if self.current_char().is_some() {
                    self.advance();
                }
                continue;
            }
            self.advance();
        }

        self.token_flags.set(TokenFlags::UNTERMINATED);
        TokenKind::NoSubstitutionTemplateLiteral
    }

    /// Re-scan template token after expression in template literal
    pub fn rescan_template_token(&mut self) -> TokenKind {
        self.token_pos = self.pos;
        self.token_flags.clear();

        // We're after a }, scan the rest of the template
        while let Some(ch) = self.current_char() {
            if ch == b'`' {
                self.advance();
                self.token = TokenKind::TemplateTail;
                return self.token;
            }
            if ch == b'$' && self.peek_char(1) == Some(b'{') {
                self.advance_by(2);
                self.token = TokenKind::TemplateMiddle;
                return self.token;
            }
            if ch == b'\\' {
                self.advance();
                if self.current_char().is_some() {
                    self.advance();
                }
                continue;
            }
            self.advance();
        }

        self.token_flags.set(TokenFlags::UNTERMINATED);
        self.token = TokenKind::TemplateTail;
        self.token
    }

    fn scan_dot(&mut self) -> TokenKind {
        self.advance();
        if self.current_char() == Some(b'.') && self.peek_char(1) == Some(b'.') {
            self.advance_by(2);
            return TokenKind::DotDotDot;
        }
        // Check for number starting with .
        if let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                return self.scan_decimal_after_dot();
            }
        }
        TokenKind::Dot
    }

    fn scan_decimal_after_dot(&mut self) -> TokenKind {
        self.scan_digits();
        if let Some(ch) = self.current_char() {
            if ch == b'e' || ch == b'E' {
                self.token_flags.set(TokenFlags::SCIENTIFIC);
                self.advance();
                if let Some(sign) = self.current_char() {
                    if sign == b'+' || sign == b'-' {
                        self.advance();
                    }
                }
                self.scan_digits();
            }
        }
        TokenKind::NumericLiteral
    }

    fn scan_plus(&mut self) -> TokenKind {
        self.advance();
        match self.current_char() {
            Some(b'+') => {
                self.advance();
                TokenKind::PlusPlus
            }
            Some(b'=') => {
                self.advance();
                TokenKind::PlusEquals
            }
            _ => TokenKind::Plus,
        }
    }

    fn scan_minus(&mut self) -> TokenKind {
        self.advance();
        match self.current_char() {
            Some(b'-') => {
                self.advance();
                TokenKind::MinusMinus
            }
            Some(b'=') => {
                self.advance();
                TokenKind::MinusEquals
            }
            _ => TokenKind::Minus,
        }
    }

    fn scan_asterisk(&mut self) -> TokenKind {
        self.advance();
        match self.current_char() {
            Some(b'*') => {
                self.advance();
                if self.current_char() == Some(b'=') {
                    self.advance();
                    TokenKind::AsteriskAsteriskEquals
                } else {
                    TokenKind::AsteriskAsterisk
                }
            }
            Some(b'=') => {
                self.advance();
                TokenKind::AsteriskEquals
            }
            _ => TokenKind::Asterisk,
        }
    }

    fn scan_slash(&mut self) -> TokenKind {
        self.advance();
        match self.current_char() {
            Some(b'/') => self.scan_single_line_comment(),
            Some(b'*') => self.scan_multi_line_comment(),
            Some(b'=') => {
                self.advance();
                TokenKind::SlashEquals
            }
            _ => TokenKind::Slash,
        }
    }

    fn scan_single_line_comment(&mut self) -> TokenKind {
        self.advance(); // skip second /
        while let Some(ch) = self.current_char() {
            if ch == b'\n' || ch == b'\r' {
                break;
            }
            self.advance();
        }
        TokenKind::SingleLineComment
    }

    fn scan_multi_line_comment(&mut self) -> TokenKind {
        self.advance(); // skip *
        while let Some(ch) = self.current_char() {
            if ch == b'*' && self.peek_char(1) == Some(b'/') {
                self.advance_by(2);
                return TokenKind::MultiLineComment;
            }
            if ch == b'\n' || ch == b'\r' {
                self.token_flags.set(TokenFlags::PRECEDING_LINE_BREAK);
            }
            self.advance();
        }
        self.token_flags.set(TokenFlags::UNTERMINATED);
        TokenKind::MultiLineComment
    }

    fn scan_percent(&mut self) -> TokenKind {
        self.advance();
        if self.current_char() == Some(b'=') {
            self.advance();
            TokenKind::PercentEquals
        } else {
            TokenKind::Percent
        }
    }

    fn scan_less_than(&mut self) -> TokenKind {
        self.advance();
        match self.current_char() {
            Some(b'<') => {
                self.advance();
                if self.current_char() == Some(b'=') {
                    self.advance();
                    TokenKind::LessThanLessThanEquals
                } else {
                    TokenKind::LessThanLessThan
                }
            }
            Some(b'=') => {
                self.advance();
                TokenKind::LessThanEquals
            }
            Some(b'/') => {
                self.advance();
                TokenKind::LessThanSlash
            }
            _ => TokenKind::LessThan,
        }
    }

    fn scan_greater_than(&mut self) -> TokenKind {
        self.advance();
        match self.current_char() {
            Some(b'>') => {
                self.advance();
                match self.current_char() {
                    Some(b'>') => {
                        self.advance();
                        if self.current_char() == Some(b'=') {
                            self.advance();
                            TokenKind::GreaterThanGreaterThanGreaterThanEquals
                        } else {
                            TokenKind::GreaterThanGreaterThanGreaterThan
                        }
                    }
                    Some(b'=') => {
                        self.advance();
                        TokenKind::GreaterThanGreaterThanEquals
                    }
                    _ => TokenKind::GreaterThanGreaterThan,
                }
            }
            Some(b'=') => {
                self.advance();
                TokenKind::GreaterThanEquals
            }
            _ => TokenKind::GreaterThan,
        }
    }

    fn scan_equals(&mut self) -> TokenKind {
        self.advance();
        match self.current_char() {
            Some(b'=') => {
                self.advance();
                if self.current_char() == Some(b'=') {
                    self.advance();
                    TokenKind::EqualsEqualsEquals
                } else {
                    TokenKind::EqualsEquals
                }
            }
            Some(b'>') => {
                self.advance();
                TokenKind::EqualsGreaterThan
            }
            _ => TokenKind::Equals,
        }
    }

    fn scan_exclamation(&mut self) -> TokenKind {
        self.advance();
        match self.current_char() {
            Some(b'=') => {
                self.advance();
                if self.current_char() == Some(b'=') {
                    self.advance();
                    TokenKind::ExclamationEqualsEquals
                } else {
                    TokenKind::ExclamationEquals
                }
            }
            _ => TokenKind::Exclamation,
        }
    }

    fn scan_ampersand(&mut self) -> TokenKind {
        self.advance();
        match self.current_char() {
            Some(b'&') => {
                self.advance();
                if self.current_char() == Some(b'=') {
                    self.advance();
                    TokenKind::AmpersandAmpersandEquals
                } else {
                    TokenKind::AmpersandAmpersand
                }
            }
            Some(b'=') => {
                self.advance();
                TokenKind::AmpersandEquals
            }
            _ => TokenKind::Ampersand,
        }
    }

    fn scan_bar(&mut self) -> TokenKind {
        self.advance();
        match self.current_char() {
            Some(b'|') => {
                self.advance();
                if self.current_char() == Some(b'=') {
                    self.advance();
                    TokenKind::BarBarEquals
                } else {
                    TokenKind::BarBar
                }
            }
            Some(b'=') => {
                self.advance();
                TokenKind::BarEquals
            }
            _ => TokenKind::Bar,
        }
    }

    fn scan_caret(&mut self) -> TokenKind {
        self.advance();
        if self.current_char() == Some(b'=') {
            self.advance();
            TokenKind::CaretEquals
        } else {
            TokenKind::Caret
        }
    }

    fn scan_question(&mut self) -> TokenKind {
        self.advance();
        match self.current_char() {
            Some(b'?') => {
                self.advance();
                if self.current_char() == Some(b'=') {
                    self.advance();
                    TokenKind::QuestionQuestionEquals
                } else {
                    TokenKind::QuestionQuestion
                }
            }
            Some(b'.') => {
                // Check it's not a number (?.0 would be questionable followed by a number)
                // If there's no next character or it's not a digit, it's QuestionDot
                match self.peek_char(1) {
                    Some(next) if next.is_ascii_digit() => TokenKind::Question,
                    _ => {
                        self.advance();
                        TokenKind::QuestionDot
                    }
                }
            }
            _ => TokenKind::Question,
        }
    }

    /// Rescan slash as regex when context allows
    pub fn rescan_slash_token(&mut self) -> TokenKind {
        if self.token != TokenKind::Slash && self.token != TokenKind::SlashEquals {
            return self.token;
        }

        // Reset position to token start
        self.pos = self.token_pos;
        self.token_pos = self.pos;
        self.advance(); // skip /

        let mut in_escape = false;
        let mut in_char_class = false;

        while let Some(ch) = self.current_char() {
            if ch == b'\n' || ch == b'\r' {
                self.token_flags.set(TokenFlags::UNTERMINATED);
                self.token = TokenKind::RegularExpressionLiteral;
                return self.token;
            }

            if in_escape {
                in_escape = false;
            } else if ch == b'\\' {
                in_escape = true;
            } else if ch == b'[' {
                in_char_class = true;
            } else if ch == b']' {
                in_char_class = false;
            } else if ch == b'/' && !in_char_class {
                self.advance();
                // Scan regex flags
                while let Some(flag) = self.current_char() {
                    if is_identifier_char(flag) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.token = TokenKind::RegularExpressionLiteral;
                return self.token;
            }
            self.advance();
        }

        self.token_flags.set(TokenFlags::UNTERMINATED);
        self.token = TokenKind::RegularExpressionLiteral;
        self.token
    }
}

/// Check if byte is valid ASCII identifier start character
#[inline]
fn is_identifier_start_char(ch: u8) -> bool {
    matches!(ch, b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$')
}

/// Check if byte is valid ASCII identifier character
#[inline]
fn is_identifier_char(ch: u8) -> bool {
    matches!(ch, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_new() {
        let source = "let x = 42;";
        let scanner = Scanner::new(source);
        assert_eq!(scanner.get_text(), source);
        assert_eq!(scanner.get_text_pos(), 0);
    }

    #[test]
    fn test_scan_identifier() {
        let source = "myVariable";
        let mut scanner = Scanner::new(source);
        assert_eq!(scanner.scan(), TokenKind::Identifier);
        assert_eq!(scanner.get_token_text(), "myVariable");
    }

    #[test]
    fn test_scan_keywords() {
        let keywords = [
            ("let", TokenKind::Let),
            ("const", TokenKind::Const),
            ("function", TokenKind::Function),
            ("class", TokenKind::Class),
            ("if", TokenKind::If),
            ("else", TokenKind::Else),
            ("return", TokenKind::Return),
            ("async", TokenKind::Async),
            ("await", TokenKind::Await),
        ];

        for (text, expected) in keywords {
            let mut scanner = Scanner::new(text);
            assert_eq!(scanner.scan(), expected, "Failed for keyword: {}", text);
        }
    }

    #[test]
    fn test_scan_operators() {
        let operators = [
            ("+", TokenKind::Plus),
            ("++", TokenKind::PlusPlus),
            ("+=", TokenKind::PlusEquals),
            ("-", TokenKind::Minus),
            ("--", TokenKind::MinusMinus),
            ("*", TokenKind::Asterisk),
            ("**", TokenKind::AsteriskAsterisk),
            ("===", TokenKind::EqualsEqualsEquals),
            ("!==", TokenKind::ExclamationEqualsEquals),
            ("=>", TokenKind::EqualsGreaterThan),
            ("...", TokenKind::DotDotDot),
            ("?.", TokenKind::QuestionDot),
            ("??", TokenKind::QuestionQuestion),
            ("&&=", TokenKind::AmpersandAmpersandEquals),
            ("||=", TokenKind::BarBarEquals),
            ("??=", TokenKind::QuestionQuestionEquals),
        ];

        for (text, expected) in operators {
            let mut scanner = Scanner::new(text);
            assert_eq!(scanner.scan(), expected, "Failed for operator: {}", text);
        }
    }

    #[test]
    fn test_scan_numbers() {
        let mut scanner = Scanner::new("42");
        assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
        assert_eq!(scanner.get_token_text(), "42");

        let mut scanner = Scanner::new("3.14");
        assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
        assert_eq!(scanner.get_token_text(), "3.14");

        let mut scanner = Scanner::new("1e10");
        assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
        assert!(scanner.get_token_flags().has(TokenFlags::SCIENTIFIC));

        let mut scanner = Scanner::new("0xFF");
        assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
        assert!(scanner.get_token_flags().has(TokenFlags::HEX_SPECIFIER));

        let mut scanner = Scanner::new("0b1010");
        assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
        assert!(scanner.get_token_flags().has(TokenFlags::BINARY_SPECIFIER));

        let mut scanner = Scanner::new("0o777");
        assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
        assert!(scanner.get_token_flags().has(TokenFlags::OCTAL_SPECIFIER));

        let mut scanner = Scanner::new("42n");
        assert_eq!(scanner.scan(), TokenKind::BigIntLiteral);
    }

    #[test]
    fn test_scan_strings() {
        let mut scanner = Scanner::new("\"hello\"");
        assert_eq!(scanner.scan(), TokenKind::StringLiteral);
        assert_eq!(scanner.get_token_text(), "\"hello\"");

        let mut scanner = Scanner::new("'world'");
        assert_eq!(scanner.scan(), TokenKind::StringLiteral);
        assert_eq!(scanner.get_token_text(), "'world'");
    }

    #[test]
    fn test_scan_template_literal() {
        let mut scanner = Scanner::new("`hello`");
        assert_eq!(scanner.scan(), TokenKind::NoSubstitutionTemplateLiteral);

        let mut scanner = Scanner::new("`hello ${");
        assert_eq!(scanner.scan(), TokenKind::TemplateHead);
    }

    #[test]
    fn test_scan_comments() {
        let mut scanner = Scanner::new("// comment\nx");
        assert_eq!(scanner.scan(), TokenKind::SingleLineComment);
        assert_eq!(scanner.scan(), TokenKind::Identifier);
        assert!(scanner.has_preceding_line_break());

        let mut scanner = Scanner::new("/* block */");
        assert_eq!(scanner.scan(), TokenKind::MultiLineComment);
    }

    #[test]
    fn test_scan_shebang() {
        let mut scanner = Scanner::new("#!/usr/bin/env node\nconst");
        assert_eq!(scanner.scan(), TokenKind::Shebang);
        assert_eq!(scanner.scan(), TokenKind::Const);
    }

    #[test]
    fn test_scan_private_identifier() {
        let mut scanner = Scanner::new("#privateField");
        assert_eq!(scanner.scan(), TokenKind::PrivateIdentifier);
        assert_eq!(scanner.get_token_text(), "#privateField");
    }

    #[test]
    fn test_scan_complex_expression() {
        let source = "const x = 42 + y;";
        let mut scanner = Scanner::new(source);

        assert_eq!(scanner.scan(), TokenKind::Const);
        assert_eq!(scanner.scan(), TokenKind::Identifier);
        assert_eq!(scanner.scan(), TokenKind::Equals);
        assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
        assert_eq!(scanner.scan(), TokenKind::Plus);
        assert_eq!(scanner.scan(), TokenKind::Identifier);
        assert_eq!(scanner.scan(), TokenKind::Semicolon);
        assert_eq!(scanner.scan(), TokenKind::EndOfFile);
    }

    #[test]
    fn test_scan_arrow_function() {
        let source = "(x) => x * 2";
        let mut scanner = Scanner::new(source);

        assert_eq!(scanner.scan(), TokenKind::OpenParen);
        assert_eq!(scanner.scan(), TokenKind::Identifier);
        assert_eq!(scanner.scan(), TokenKind::CloseParen);
        assert_eq!(scanner.scan(), TokenKind::EqualsGreaterThan);
        assert_eq!(scanner.scan(), TokenKind::Identifier);
        assert_eq!(scanner.scan(), TokenKind::Asterisk);
        assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    }

    #[test]
    fn test_zero_copy() {
        let source = "identifier keyword";
        let scanner = Scanner::new(source);

        // Verify scanner holds a reference, not a copy
        assert_eq!(scanner.source.as_ptr(), source.as_ptr());
    }

    #[test]
    fn test_token_spans() {
        let source = "let x = 42;";
        let mut scanner = Scanner::new(source);

        scanner.scan(); // let
        let token = scanner.get_current_token();
        assert_eq!(token.span.start, 0);
        assert_eq!(token.span.end, 3);
        assert_eq!(token.text(source), "let");

        scanner.scan(); // x
        let token = scanner.get_current_token();
        assert_eq!(token.span.start, 4);
        assert_eq!(token.span.end, 5);
        assert_eq!(token.text(source), "x");
    }

    #[test]
    fn test_unterminated_string() {
        let mut scanner = Scanner::new("\"unterminated");
        scanner.scan();
        assert!(scanner.is_unterminated());
    }

    #[test]
    fn test_typescript_sample() {
        let source = r#"
interface User {
    name: string;
    age: number;
}

function greet(user: User): string {
    return `Hello, ${user.name}!`;
}
"#;
        let mut scanner = Scanner::new(source);
        let mut tokens = Vec::new();

        loop {
            let kind = scanner.scan();
            if kind == TokenKind::EndOfFile {
                break;
            }
            // Skip trivia for this test
            if !kind.is_trivia() {
                tokens.push((kind, scanner.get_token_text().to_string()));
            }
        }

        // Verify some key tokens
        assert!(tokens.iter().any(|(k, _t)| *k == TokenKind::Interface));
        assert!(tokens.iter().any(|(k, t)| *k == TokenKind::Identifier && t == "User"));
        assert!(tokens.iter().any(|(k, _t)| *k == TokenKind::String));
        assert!(tokens.iter().any(|(k, _t)| *k == TokenKind::Number));
        assert!(tokens.iter().any(|(k, _t)| *k == TokenKind::Function));
        assert!(tokens.iter().any(|(k, _t)| *k == TokenKind::Return));
        assert!(tokens.iter().any(|(k, _t)| *k == TokenKind::TemplateHead));
    }

    #[test]
    fn test_rescan_regex() {
        let source = "/pattern/gi";
        let mut scanner = Scanner::new(source);

        assert_eq!(scanner.scan(), TokenKind::Slash);
        assert_eq!(scanner.rescan_slash_token(), TokenKind::RegularExpressionLiteral);
        assert_eq!(scanner.get_token_text(), "/pattern/gi");
    }

    #[test]
    fn test_numeric_separators() {
        let mut scanner = Scanner::new("1_000_000");
        assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
        assert!(scanner.get_token_flags().has(TokenFlags::CONTAINS_SEPARATOR));
    }
}
