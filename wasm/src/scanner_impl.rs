//! Scanner implementation - the lexical analyzer for TypeScript.
//!
//! This module implements the core Scanner struct that tokenizes TypeScript source code.
//! It's designed to produce the same token stream as TypeScript's scanner.ts.
//!
//! IMPORTANT: All positions are character-based (like JavaScript's string indexing),
//! NOT byte-based. This ensures compatibility with TypeScript's scanner positions.

use wasm_bindgen::prelude::*;
use crate::scanner::SyntaxKind;
use crate::char_codes::CharacterCodes;

// =============================================================================
// Token Flags
// =============================================================================

/// Token flags indicating special properties of scanned tokens.
#[wasm_bindgen]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TokenFlags {
    #[default]
    None = 0,
    PrecedingLineBreak = 1,
    PrecedingJSDocComment = 2,
    Unterminated = 4,
    ExtendedUnicodeEscape = 8,
    Scientific = 16,
    Octal = 32,
    HexSpecifier = 64,
    BinarySpecifier = 128,
    OctalSpecifier = 256,
    ContainsSeparator = 512,
    UnicodeEscape = 1024,
    ContainsInvalidEscape = 2048,
    HexEscape = 4096,
    ContainsLeadingZero = 8192,
    ContainsInvalidSeparator = 16384,
    PrecedingJSDocLeadingAsterisks = 32768,
}

// =============================================================================
// Scanner State
// =============================================================================

/// The scanner state that holds the current position and token information.
/// 
/// All positions (pos, end, token_start, full_start_pos) are CHARACTER indices,
/// not byte indices. This matches TypeScript/JavaScript string indexing.
#[wasm_bindgen]
pub struct ScannerState {
    /// The source text as a Vec<char> for O(1) character access
    chars: Vec<char>,
    /// Current position (character index)
    pos: usize,
    /// End position (character count)
    end: usize,
    /// Full start position including leading trivia
    full_start_pos: usize,
    /// Token start position (excluding trivia)
    token_start: usize,
    /// Current token kind
    token: SyntaxKind,
    /// Current token's string value
    token_value: String,
    /// Token flags
    token_flags: u32,
    /// Whether to skip trivia (whitespace, comments)
    skip_trivia: bool,
}

#[wasm_bindgen]
impl ScannerState {
    /// Create a new scanner state with the given text.
    #[wasm_bindgen(constructor)]
    pub fn new(text: String, skip_trivia: bool) -> ScannerState {
        let chars: Vec<char> = text.chars().collect();
        let end = chars.len();
        ScannerState {
            chars,
            pos: 0,
            end,
            full_start_pos: 0,
            token_start: 0,
            token: SyntaxKind::Unknown,
            token_value: String::new(),
            token_flags: 0,
            skip_trivia,
        }
    }

    /// Get the current position (end position of current token).
    #[wasm_bindgen(js_name = getPos)]
    pub fn get_pos(&self) -> usize {
        self.pos
    }

    /// Get the full start position (including leading trivia).
    #[wasm_bindgen(js_name = getTokenFullStart)]
    pub fn get_token_full_start(&self) -> usize {
        self.full_start_pos
    }

    /// Get the start position of the current token (excluding trivia).
    #[wasm_bindgen(js_name = getTokenStart)]
    pub fn get_token_start(&self) -> usize {
        self.token_start
    }

    /// Get the end position of the current token.
    #[wasm_bindgen(js_name = getTokenEnd)]
    pub fn get_token_end(&self) -> usize {
        self.pos
    }

    /// Get the current token kind.
    #[wasm_bindgen(js_name = getToken)]
    pub fn get_token(&self) -> SyntaxKind {
        self.token
    }

    /// Get the current token's string value.
    #[wasm_bindgen(js_name = getTokenValue)]
    pub fn get_token_value(&self) -> String {
        self.token_value.clone()
    }

    /// Get the current token's text from the source.
    #[wasm_bindgen(js_name = getTokenText)]
    pub fn get_token_text(&self) -> String {
        self.chars[self.token_start..self.pos].iter().collect()
    }

    /// Get the token flags.
    #[wasm_bindgen(js_name = getTokenFlags)]
    pub fn get_token_flags(&self) -> u32 {
        self.token_flags
    }

    /// Check if there was a preceding line break.
    #[wasm_bindgen(js_name = hasPrecedingLineBreak)]
    pub fn has_preceding_line_break(&self) -> bool {
        (self.token_flags & TokenFlags::PrecedingLineBreak as u32) != 0
    }

    /// Check if the token is unterminated.
    #[wasm_bindgen(js_name = isUnterminated)]
    pub fn is_unterminated(&self) -> bool {
        (self.token_flags & TokenFlags::Unterminated as u32) != 0
    }

    /// Check if the current token is an identifier.
    #[wasm_bindgen(js_name = isIdentifier)]
    pub fn is_identifier(&self) -> bool {
        self.token == SyntaxKind::Identifier || (self.token as u16) > (SyntaxKind::WithKeyword as u16)
    }

    /// Check if the current token is a reserved word.
    #[wasm_bindgen(js_name = isReservedWord)]
    pub fn is_reserved_word(&self) -> bool {
        let t = self.token as u16;
        t >= SyntaxKind::BreakKeyword as u16 && t <= SyntaxKind::WithKeyword as u16
    }

    /// Set the text to scan.
    #[wasm_bindgen(js_name = setText)]
    pub fn set_text(&mut self, text: String, start: Option<usize>, length: Option<usize>) {
        self.chars = text.chars().collect();
        let start = start.unwrap_or(0);
        let len = length.unwrap_or(self.chars.len() - start);
        self.pos = start;
        self.end = start + len;
        self.full_start_pos = start;
        self.token_start = start;
        self.token = SyntaxKind::Unknown;
        self.token_value = String::new();
        self.token_flags = 0;
    }

    /// Reset the token state to a specific position.
    #[wasm_bindgen(js_name = resetTokenState)]
    pub fn reset_token_state(&mut self, new_pos: usize) {
        self.pos = new_pos;
        self.full_start_pos = new_pos;
        self.token_start = new_pos;
        self.token = SyntaxKind::Unknown;
        self.token_value = String::new();
        self.token_flags = 0;
    }

    /// Get the source text.
    #[wasm_bindgen(js_name = getText)]
    pub fn get_text(&self) -> String {
        self.chars.iter().collect()
    }

    // =========================================================================
    // Helper methods (character-indexed for TypeScript compatibility)
    // =========================================================================

    /// Get the character code at the given character index.
    /// Returns None if out of bounds.
    #[inline]
    fn char_code_at(&self, index: usize) -> Option<u32> {
        if index < self.chars.len() {
            Some(self.chars[index] as u32)
        } else {
            None
        }
    }

    /// Get character code at index, returns 0 if out of bounds.
    #[inline]
    fn char_code_unchecked(&self, index: usize) -> u32 {
        if index < self.chars.len() {
            self.chars[index] as u32
        } else {
            0
        }
    }

    #[inline]
    fn is_at_end(&self) -> bool {
        self.pos >= self.end
    }
    
    /// Get a substring from start to end character indices.
    #[inline]
    fn substring(&self, start: usize, end: usize) -> String {
        self.chars[start..end].iter().collect()
    }

    // =========================================================================
    // Scanning methods
    // =========================================================================

    /// Scan the next token.
    #[wasm_bindgen]
    pub fn scan(&mut self) -> SyntaxKind {
        self.full_start_pos = self.pos;
        self.token_flags = 0;

        loop {
            self.token_start = self.pos;
            
            if self.pos >= self.end {
                self.token = SyntaxKind::EndOfFileToken;
                return self.token;
            }

            let ch = self.char_code_unchecked(self.pos);

            match ch {
                // Newlines
                CharacterCodes::LINE_FEED | CharacterCodes::CARRIAGE_RETURN => {
                    self.token_flags |= TokenFlags::PrecedingLineBreak as u32;
                    if self.skip_trivia {
                        self.pos += 1;
                        if ch == CharacterCodes::CARRIAGE_RETURN 
                            && self.pos < self.end 
                            && self.char_code_unchecked(self.pos) == CharacterCodes::LINE_FEED 
                        {
                            self.pos += 1;
                        }
                        continue;
                    } else {
                        if ch == CharacterCodes::CARRIAGE_RETURN 
                            && self.pos + 1 < self.end 
                            && self.char_code_unchecked(self.pos + 1) == CharacterCodes::LINE_FEED 
                        {
                            self.pos += 2;
                        } else {
                            self.pos += 1;
                        }
                        self.token = SyntaxKind::NewLineTrivia;
                        return self.token;
                    }
                }

                // Whitespace
                CharacterCodes::TAB 
                | CharacterCodes::VERTICAL_TAB 
                | CharacterCodes::FORM_FEED 
                | CharacterCodes::SPACE 
                | CharacterCodes::NON_BREAKING_SPACE => {
                    if self.skip_trivia {
                        self.pos += 1;
                        while self.pos < self.end && is_white_space_single_line(self.char_code_unchecked(self.pos)) {
                            self.pos += 1;
                        }
                        continue;
                    } else {
                        while self.pos < self.end && is_white_space_single_line(self.char_code_unchecked(self.pos)) {
                            self.pos += 1;
                        }
                        self.token = SyntaxKind::WhitespaceTrivia;
                        return self.token;
                    }
                }

                // Punctuation - Single characters
                CharacterCodes::OPEN_BRACE => {
                    self.pos += 1;
                    self.token = SyntaxKind::OpenBraceToken;
                    return self.token;
                }
                CharacterCodes::CLOSE_BRACE => {
                    self.pos += 1;
                    self.token = SyntaxKind::CloseBraceToken;
                    return self.token;
                }
                CharacterCodes::OPEN_PAREN => {
                    self.pos += 1;
                    self.token = SyntaxKind::OpenParenToken;
                    return self.token;
                }
                CharacterCodes::CLOSE_PAREN => {
                    self.pos += 1;
                    self.token = SyntaxKind::CloseParenToken;
                    return self.token;
                }
                CharacterCodes::OPEN_BRACKET => {
                    self.pos += 1;
                    self.token = SyntaxKind::OpenBracketToken;
                    return self.token;
                }
                CharacterCodes::CLOSE_BRACKET => {
                    self.pos += 1;
                    self.token = SyntaxKind::CloseBracketToken;
                    return self.token;
                }
                CharacterCodes::SEMICOLON => {
                    self.pos += 1;
                    self.token = SyntaxKind::SemicolonToken;
                    return self.token;
                }
                CharacterCodes::COMMA => {
                    self.pos += 1;
                    self.token = SyntaxKind::CommaToken;
                    return self.token;
                }
                CharacterCodes::TILDE => {
                    self.pos += 1;
                    self.token = SyntaxKind::TildeToken;
                    return self.token;
                }
                CharacterCodes::AT => {
                    self.pos += 1;
                    self.token = SyntaxKind::AtToken;
                    return self.token;
                }
                CharacterCodes::COLON => {
                    self.pos += 1;
                    self.token = SyntaxKind::ColonToken;
                    return self.token;
                }

                // Multi-character punctuation
                CharacterCodes::DOT => {
                    if self.pos + 1 < self.end && is_digit(self.char_code_unchecked(self.pos + 1)) {
                        self.scan_number();
                        return self.token;
                    }
                    if self.pos + 2 < self.end 
                        && self.char_code_unchecked(self.pos + 1) == CharacterCodes::DOT 
                        && self.char_code_unchecked(self.pos + 2) == CharacterCodes::DOT 
                    {
                        self.pos += 3;
                        self.token = SyntaxKind::DotDotDotToken;
                        return self.token;
                    }
                    self.pos += 1;
                    self.token = SyntaxKind::DotToken;
                    return self.token;
                }

                // Exclamation
                CharacterCodes::EXCLAMATION => {
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::EQUALS) {
                        if self.char_code_at(self.pos + 2) == Some(CharacterCodes::EQUALS) {
                            self.pos += 3;
                            self.token = SyntaxKind::ExclamationEqualsEqualsToken;
                            return self.token;
                        }
                        self.pos += 2;
                        self.token = SyntaxKind::ExclamationEqualsToken;
                        return self.token;
                    }
                    self.pos += 1;
                    self.token = SyntaxKind::ExclamationToken;
                    return self.token;
                }

                // Equals
                CharacterCodes::EQUALS => {
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::EQUALS) {
                        if self.char_code_at(self.pos + 2) == Some(CharacterCodes::EQUALS) {
                            self.pos += 3;
                            self.token = SyntaxKind::EqualsEqualsEqualsToken;
                            return self.token;
                        }
                        self.pos += 2;
                        self.token = SyntaxKind::EqualsEqualsToken;
                        return self.token;
                    }
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::GREATER_THAN) {
                        self.pos += 2;
                        self.token = SyntaxKind::EqualsGreaterThanToken;
                        return self.token;
                    }
                    self.pos += 1;
                    self.token = SyntaxKind::EqualsToken;
                    return self.token;
                }

                // Plus
                CharacterCodes::PLUS => {
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::PLUS) {
                        self.pos += 2;
                        self.token = SyntaxKind::PlusPlusToken;
                        return self.token;
                    }
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::EQUALS) {
                        self.pos += 2;
                        self.token = SyntaxKind::PlusEqualsToken;
                        return self.token;
                    }
                    self.pos += 1;
                    self.token = SyntaxKind::PlusToken;
                    return self.token;
                }

                // Minus
                CharacterCodes::MINUS => {
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::MINUS) {
                        self.pos += 2;
                        self.token = SyntaxKind::MinusMinusToken;
                        return self.token;
                    }
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::EQUALS) {
                        self.pos += 2;
                        self.token = SyntaxKind::MinusEqualsToken;
                        return self.token;
                    }
                    self.pos += 1;
                    self.token = SyntaxKind::MinusToken;
                    return self.token;
                }

                // Asterisk
                CharacterCodes::ASTERISK => {
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::ASTERISK) {
                        if self.char_code_at(self.pos + 2) == Some(CharacterCodes::EQUALS) {
                            self.pos += 3;
                            self.token = SyntaxKind::AsteriskAsteriskEqualsToken;
                            return self.token;
                        }
                        self.pos += 2;
                        self.token = SyntaxKind::AsteriskAsteriskToken;
                        return self.token;
                    }
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::EQUALS) {
                        self.pos += 2;
                        self.token = SyntaxKind::AsteriskEqualsToken;
                        return self.token;
                    }
                    self.pos += 1;
                    self.token = SyntaxKind::AsteriskToken;
                    return self.token;
                }

                // Percent
                CharacterCodes::PERCENT => {
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::EQUALS) {
                        self.pos += 2;
                        self.token = SyntaxKind::PercentEqualsToken;
                        return self.token;
                    }
                    self.pos += 1;
                    self.token = SyntaxKind::PercentToken;
                    return self.token;
                }

                // Ampersand
                CharacterCodes::AMPERSAND => {
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::AMPERSAND) {
                        if self.char_code_at(self.pos + 2) == Some(CharacterCodes::EQUALS) {
                            self.pos += 3;
                            self.token = SyntaxKind::AmpersandAmpersandEqualsToken;
                            return self.token;
                        }
                        self.pos += 2;
                        self.token = SyntaxKind::AmpersandAmpersandToken;
                        return self.token;
                    }
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::EQUALS) {
                        self.pos += 2;
                        self.token = SyntaxKind::AmpersandEqualsToken;
                        return self.token;
                    }
                    self.pos += 1;
                    self.token = SyntaxKind::AmpersandToken;
                    return self.token;
                }

                // Bar (pipe)
                CharacterCodes::BAR => {
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::BAR) {
                        if self.char_code_at(self.pos + 2) == Some(CharacterCodes::EQUALS) {
                            self.pos += 3;
                            self.token = SyntaxKind::BarBarEqualsToken;
                            return self.token;
                        }
                        self.pos += 2;
                        self.token = SyntaxKind::BarBarToken;
                        return self.token;
                    }
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::EQUALS) {
                        self.pos += 2;
                        self.token = SyntaxKind::BarEqualsToken;
                        return self.token;
                    }
                    self.pos += 1;
                    self.token = SyntaxKind::BarToken;
                    return self.token;
                }

                // Caret
                CharacterCodes::CARET => {
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::EQUALS) {
                        self.pos += 2;
                        self.token = SyntaxKind::CaretEqualsToken;
                        return self.token;
                    }
                    self.pos += 1;
                    self.token = SyntaxKind::CaretToken;
                    return self.token;
                }

                // Question mark
                CharacterCodes::QUESTION => {
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::DOT) 
                        && !is_digit(self.char_code_at(self.pos + 2).unwrap_or(0)) 
                    {
                        self.pos += 2;
                        self.token = SyntaxKind::QuestionDotToken;
                        return self.token;
                    }
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::QUESTION) {
                        if self.char_code_at(self.pos + 2) == Some(CharacterCodes::EQUALS) {
                            self.pos += 3;
                            self.token = SyntaxKind::QuestionQuestionEqualsToken;
                            return self.token;
                        }
                        self.pos += 2;
                        self.token = SyntaxKind::QuestionQuestionToken;
                        return self.token;
                    }
                    self.pos += 1;
                    self.token = SyntaxKind::QuestionToken;
                    return self.token;
                }

                // Less than
                // Note: `</` (LessThanSlashToken) is only used in JSX mode.
                // In regular mode, `<` and `/` are separate tokens.
                CharacterCodes::LESS_THAN => {
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::LESS_THAN) {
                        if self.char_code_at(self.pos + 2) == Some(CharacterCodes::EQUALS) {
                            self.pos += 3;
                            self.token = SyntaxKind::LessThanLessThanEqualsToken;
                            return self.token;
                        }
                        self.pos += 2;
                        self.token = SyntaxKind::LessThanLessThanToken;
                        return self.token;
                    }
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::EQUALS) {
                        self.pos += 2;
                        self.token = SyntaxKind::LessThanEqualsToken;
                        return self.token;
                    }
                    // LessThanSlashToken is JSX-only, not returned in regular scanning
                    self.pos += 1;
                    self.token = SyntaxKind::LessThanToken;
                    return self.token;
                }

                // Greater than - only return GreaterThanToken
                // The parser calls reScanGreaterToken() to get >=, >>, >>>, >>=, >>>=
                CharacterCodes::GREATER_THAN => {
                    self.pos += 1;
                    self.token = SyntaxKind::GreaterThanToken;
                    return self.token;
                }

                // Slash - comment or division
                CharacterCodes::SLASH => {
                    // Check for comments
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::SLASH) {
                        self.pos += 2;
                        while self.pos < self.end {
                            let c = self.char_code_unchecked(self.pos);
                            if c == CharacterCodes::LINE_FEED || c == CharacterCodes::CARRIAGE_RETURN {
                                break;
                            }
                            self.pos += 1;
                        }
                        if self.skip_trivia {
                            continue;
                        } else {
                            self.token = SyntaxKind::SingleLineCommentTrivia;
                            return self.token;
                        }
                    }
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::ASTERISK) {
                        self.pos += 2;
                        let mut comment_closed = false;
                        while self.pos < self.end {
                            let c = self.char_code_unchecked(self.pos);
                            if c == CharacterCodes::ASTERISK 
                                && self.char_code_at(self.pos + 1) == Some(CharacterCodes::SLASH) 
                            {
                                self.pos += 2;
                                comment_closed = true;
                                break;
                            }
                            if c == CharacterCodes::LINE_FEED || c == CharacterCodes::CARRIAGE_RETURN {
                                self.token_flags |= TokenFlags::PrecedingLineBreak as u32;
                            }
                            self.pos += 1;
                        }
                        if !comment_closed {
                            self.token_flags |= TokenFlags::Unterminated as u32;
                        }
                        if self.skip_trivia {
                            continue;
                        } else {
                            self.token = SyntaxKind::MultiLineCommentTrivia;
                            return self.token;
                        }
                    }
                    if self.char_code_at(self.pos + 1) == Some(CharacterCodes::EQUALS) {
                        self.pos += 2;
                        self.token = SyntaxKind::SlashEqualsToken;
                        return self.token;
                    }
                    self.pos += 1;
                    self.token = SyntaxKind::SlashToken;
                    return self.token;
                }

                // String literals
                CharacterCodes::DOUBLE_QUOTE | CharacterCodes::SINGLE_QUOTE => {
                    self.scan_string(ch);
                    return self.token;
                }

                // Backtick (template literal)
                CharacterCodes::BACKTICK => {
                    self.scan_template_literal();
                    return self.token;
                }

                // Hash (private identifier)
                CharacterCodes::HASH => {
                    // Simplified: just treat as hash token
                    // Full implementation would check for private identifier
                    self.pos += 1;
                    if self.pos < self.end && is_identifier_start(self.char_code_unchecked(self.pos)) {
                        self.pos += 1;
                        while self.pos < self.end && is_identifier_part(self.char_code_unchecked(self.pos)) {
                            self.pos += 1;
                        }
                        self.token_value = self.substring(self.token_start, self.pos);
                        self.token = SyntaxKind::PrivateIdentifier;
                    } else {
                        self.token = SyntaxKind::HashToken;
                    }
                    return self.token;
                }

                // Numbers
                CharacterCodes::_0 ..= CharacterCodes::_9 => {
                    self.scan_number();
                    return self.token;
                }

                // Default: identifier or unknown
                _ => {
                    if is_identifier_start(ch) {
                        self.scan_identifier();
                        return self.token;
                    }
                    // Skip unknown character
                    self.pos += 1;
                    self.token = SyntaxKind::Unknown;
                    return self.token;
                }
            }
        }
    }

    /// Scan a string literal.
    fn scan_string(&mut self, quote: u32) {
        self.pos += 1; // Skip opening quote
        let mut result = String::new();
        
        while self.pos < self.end {
            let ch = self.char_code_unchecked(self.pos);
            if ch == quote {
                self.pos += 1; // Closing quote
                self.token_value = result;
                self.token = SyntaxKind::StringLiteral;
                return;
            }
            if ch == CharacterCodes::BACKSLASH {
                // Handle escape sequences
                self.pos += 1;
                if self.pos < self.end {
                    let escaped = self.char_code_unchecked(self.pos);
                    self.pos += 1;
                    match escaped {
                        CharacterCodes::LOWER_N => result.push('\n'),
                        CharacterCodes::LOWER_R => result.push('\r'),
                        CharacterCodes::LOWER_T => result.push('\t'),
                        CharacterCodes::BACKSLASH => result.push('\\'),
                        c if c == quote => result.push(char::from_u32(quote).unwrap_or('\0')),
                        _ => {
                            result.push('\\');
                            if let Some(c) = char::from_u32(escaped) {
                                result.push(c);
                            }
                        }
                    }
                }
            } else if ch == CharacterCodes::LINE_FEED || ch == CharacterCodes::CARRIAGE_RETURN {
                // Unterminated string
                self.token_flags |= TokenFlags::Unterminated as u32;
                self.token_value = result;
                self.token = SyntaxKind::StringLiteral;
                return;
            } else {
                if let Some(c) = char::from_u32(ch) {
                    result.push(c);
                }
                self.pos += 1;
            }
        }
        
        // Unterminated string
        self.token_flags |= TokenFlags::Unterminated as u32;
        self.token_value = result;
        self.token = SyntaxKind::StringLiteral;
    }

    /// Scan a template literal (simplified).
    fn scan_template_literal(&mut self) {
        self.pos += 1; // Skip backtick
        let mut result = String::new();
        
        while self.pos < self.end {
            let ch = self.char_code_unchecked(self.pos);
            if ch == CharacterCodes::BACKTICK {
                self.pos += 1;
                self.token_value = result;
                self.token = SyntaxKind::NoSubstitutionTemplateLiteral;
                return;
            }
            if ch == CharacterCodes::DOLLAR 
                && self.char_code_at(self.pos + 1) == Some(CharacterCodes::OPEN_BRACE) 
            {
                self.pos += 2;
                self.token_value = result;
                self.token = SyntaxKind::TemplateHead;
                return;
            }
            if ch == CharacterCodes::BACKSLASH {
                self.pos += 1;
                if self.pos < self.end {
                    let escaped = self.char_code_unchecked(self.pos);
                    self.pos += 1;
                    match escaped {
                        CharacterCodes::LOWER_N => result.push('\n'),
                        CharacterCodes::LOWER_R => result.push('\r'),
                        CharacterCodes::LOWER_T => result.push('\t'),
                        CharacterCodes::BACKTICK => result.push('`'),
                        CharacterCodes::DOLLAR => result.push('$'),
                        CharacterCodes::BACKSLASH => result.push('\\'),
                        _ => {
                            result.push('\\');
                            if let Some(c) = char::from_u32(escaped) {
                                result.push(c);
                            }
                        }
                    }
                }
            } else {
                if ch == CharacterCodes::LINE_FEED || ch == CharacterCodes::CARRIAGE_RETURN {
                    self.token_flags |= TokenFlags::PrecedingLineBreak as u32;
                }
                if let Some(c) = char::from_u32(ch) {
                    result.push(c);
                }
                self.pos += 1;
            }
        }
        
        self.token_flags |= TokenFlags::Unterminated as u32;
        self.token_value = result;
        self.token = SyntaxKind::NoSubstitutionTemplateLiteral;
    }

    /// Scan a number literal (simplified).
    fn scan_number(&mut self) {
        let start = self.pos;
        
        // Check for hex, octal, binary
        if self.char_code_unchecked(self.pos) == CharacterCodes::_0 {
            let next = self.char_code_at(self.pos + 1).unwrap_or(0);
            if next == CharacterCodes::LOWER_X || next == CharacterCodes::UPPER_X {
                // Hex number
                self.pos += 2;
                self.token_flags |= TokenFlags::HexSpecifier as u32;
                while self.pos < self.end && is_hex_digit(self.char_code_unchecked(self.pos)) {
                    self.pos += 1;
                }
                self.token_value = self.substring(start, self.pos);
                self.token = SyntaxKind::NumericLiteral;
                return;
            }
            if next == CharacterCodes::LOWER_B || next == CharacterCodes::UPPER_B {
                // Binary number
                self.pos += 2;
                self.token_flags |= TokenFlags::BinarySpecifier as u32;
                while self.pos < self.end {
                    let ch = self.char_code_unchecked(self.pos);
                    if ch != CharacterCodes::_0 && ch != CharacterCodes::_1 {
                        break;
                    }
                    self.pos += 1;
                }
                self.token_value = self.substring(start, self.pos);
                self.token = SyntaxKind::NumericLiteral;
                return;
            }
            if next == CharacterCodes::LOWER_O || next == CharacterCodes::UPPER_O {
                // Octal number
                self.pos += 2;
                self.token_flags |= TokenFlags::OctalSpecifier as u32;
                while self.pos < self.end && is_octal_digit(self.char_code_unchecked(self.pos)) {
                    self.pos += 1;
                }
                self.token_value = self.substring(start, self.pos);
                self.token = SyntaxKind::NumericLiteral;
                return;
            }
        }

        // Decimal number
        while self.pos < self.end && is_digit(self.char_code_unchecked(self.pos)) {
            self.pos += 1;
        }
        
        // Decimal point
        if self.pos < self.end && self.char_code_unchecked(self.pos) == CharacterCodes::DOT {
            self.pos += 1;
            while self.pos < self.end && is_digit(self.char_code_unchecked(self.pos)) {
                self.pos += 1;
            }
        }
        
        // Exponent
        if self.pos < self.end {
            let ch = self.char_code_unchecked(self.pos);
            if ch == CharacterCodes::LOWER_E || ch == CharacterCodes::UPPER_E {
                self.pos += 1;
                self.token_flags |= TokenFlags::Scientific as u32;
                if self.pos < self.end {
                    let sign = self.char_code_unchecked(self.pos);
                    if sign == CharacterCodes::PLUS || sign == CharacterCodes::MINUS {
                        self.pos += 1;
                    }
                }
                while self.pos < self.end && is_digit(self.char_code_unchecked(self.pos)) {
                    self.pos += 1;
                }
            }
        }
        
        // BigInt suffix
        if self.pos < self.end && self.char_code_unchecked(self.pos) == CharacterCodes::LOWER_N {
            self.pos += 1;
            self.token_value = self.substring(start, self.pos);
            self.token = SyntaxKind::BigIntLiteral;
            return;
        }
        
        self.token_value = self.substring(start, self.pos);
        self.token = SyntaxKind::NumericLiteral;
    }

    /// Scan an identifier.
    fn scan_identifier(&mut self) {
        let start = self.pos;
        // Advance past first character
        self.pos += 1;
        
        while self.pos < self.end {
            let ch = self.char_code_unchecked(self.pos);
            if !is_identifier_part(ch) {
                break;
            }
            self.pos += 1;
        }
        
        let text = self.substring(start, self.pos);
        self.token_value = text.clone();
        
        // Check if it's a keyword
        self.token = crate::scanner::text_to_keyword(&text).unwrap_or(SyntaxKind::Identifier);
    }

    // =========================================================================
    // Rescan methods - for context-sensitive parsing
    // =========================================================================

    /// Re-scan the current `>` token to see if it should be `>=`, `>>`, `>>>`, `>>=`, or `>>>=`.
    /// This is used by the parser for type arguments and bitwise operators.
    #[wasm_bindgen(js_name = reScanGreaterToken)]
    pub fn re_scan_greater_token(&mut self) -> SyntaxKind {
        if self.token == SyntaxKind::GreaterThanToken {
            let next_char = self.char_code_unchecked(self.pos);
            if next_char == CharacterCodes::GREATER_THAN {
                let next_next = self.char_code_unchecked(self.pos + 1);
                if next_next == CharacterCodes::GREATER_THAN {
                    // >>>
                    let next_next_next = self.char_code_unchecked(self.pos + 2);
                    if next_next_next == CharacterCodes::EQUALS {
                        // >>>=
                        self.pos += 3;
                        self.token = SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken;
                        return self.token;
                    }
                    self.pos += 2;
                    self.token = SyntaxKind::GreaterThanGreaterThanGreaterThanToken;
                    return self.token;
                }
                if next_next == CharacterCodes::EQUALS {
                    // >>=
                    self.pos += 2;
                    self.token = SyntaxKind::GreaterThanGreaterThanEqualsToken;
                    return self.token;
                }
                // >>
                self.pos += 1;
                self.token = SyntaxKind::GreaterThanGreaterThanToken;
                return self.token;
            }
            if next_char == CharacterCodes::EQUALS {
                // >=
                self.pos += 1;
                self.token = SyntaxKind::GreaterThanEqualsToken;
                return self.token;
            }
        }
        self.token
    }

    /// Re-scan the current `/` or `/=` token as a regex literal.
    /// This is used by the parser when it determines the context requires a regex.
    #[wasm_bindgen(js_name = reScanSlashToken)]
    pub fn re_scan_slash_token(&mut self) -> SyntaxKind {
        if self.token == SyntaxKind::SlashToken || self.token == SyntaxKind::SlashEqualsToken {
            // Start scanning from after the initial /
            let start_of_regex_body = self.token_start + 1;
            self.pos = start_of_regex_body;
            let mut in_escape = false;
            let mut in_character_class = false;

            // Scan until we find the closing /
            while self.pos < self.end {
                let ch = self.char_code_unchecked(self.pos);
                
                // Unterminated regex if we hit a newline
                if is_line_break(ch) {
                    self.token_flags |= TokenFlags::Unterminated as u32;
                    break;
                }

                if in_escape {
                    // After backslash, just consume the next character
                    in_escape = false;
                } else if ch == CharacterCodes::SLASH && !in_character_class {
                    // Found the closing /
                    break;
                } else if ch == CharacterCodes::OPEN_BRACKET {
                    in_character_class = true;
                } else if ch == CharacterCodes::BACKSLASH {
                    in_escape = true;
                } else if ch == CharacterCodes::CLOSE_BRACKET {
                    in_character_class = false;
                }
                self.pos += 1;
            }

            if (self.token_flags & TokenFlags::Unterminated as u32) == 0 {
                // Consume the closing /
                self.pos += 1;
                
                // Scan regex flags (g, i, m, s, u, v, y, d)
                while self.pos < self.end {
                    let ch = self.char_code_unchecked(self.pos);
                    if !is_regex_flag(ch) && !is_identifier_part(ch) {
                        break;
                    }
                    self.pos += 1;
                }
            }

            self.token_value = self.substring(self.token_start, self.pos);
            self.token = SyntaxKind::RegularExpressionLiteral;
        }
        self.token
    }

    /// Re-scan the current `*=` token as `*` followed by `=`.
    /// Used when parsing computed property names.
    #[wasm_bindgen(js_name = reScanAsteriskEqualsToken)]
    pub fn re_scan_asterisk_equals_token(&mut self) -> SyntaxKind {
        if self.token == SyntaxKind::AsteriskEqualsToken {
            self.pos = self.token_start + 1;
            self.token = SyntaxKind::EqualsToken;
        }
        self.token
    }
}

// =============================================================================
// Helper functions
// =============================================================================

fn is_white_space_single_line(ch: u32) -> bool {
    ch == CharacterCodes::SPACE
        || ch == CharacterCodes::TAB
        || ch == CharacterCodes::VERTICAL_TAB
        || ch == CharacterCodes::FORM_FEED
        || ch == CharacterCodes::NON_BREAKING_SPACE
        || ch == CharacterCodes::OGHAM
        || (ch >= CharacterCodes::EN_QUAD && ch <= CharacterCodes::ZERO_WIDTH_SPACE)
        || ch == CharacterCodes::NARROW_NO_BREAK_SPACE
        || ch == CharacterCodes::MATHEMATICAL_SPACE
        || ch == CharacterCodes::IDEOGRAPHIC_SPACE
        || ch == CharacterCodes::BYTE_ORDER_MARK
}

fn is_digit(ch: u32) -> bool {
    ch >= CharacterCodes::_0 && ch <= CharacterCodes::_9
}

fn is_octal_digit(ch: u32) -> bool {
    ch >= CharacterCodes::_0 && ch <= CharacterCodes::_7
}

fn is_hex_digit(ch: u32) -> bool {
    is_digit(ch) 
        || (ch >= CharacterCodes::UPPER_A && ch <= CharacterCodes::UPPER_F)
        || (ch >= CharacterCodes::LOWER_A && ch <= CharacterCodes::LOWER_F)
}

fn is_identifier_start(ch: u32) -> bool {
    (ch >= CharacterCodes::UPPER_A && ch <= CharacterCodes::UPPER_Z)
        || (ch >= CharacterCodes::LOWER_A && ch <= CharacterCodes::LOWER_Z)
        || ch == CharacterCodes::UNDERSCORE
        || ch == CharacterCodes::DOLLAR
        || ch > 127 // Unicode letter (simplified check)
}

fn is_identifier_part(ch: u32) -> bool {
    is_identifier_start(ch) || is_digit(ch)
}

fn is_line_break(ch: u32) -> bool {
    ch == CharacterCodes::LINE_FEED
        || ch == CharacterCodes::CARRIAGE_RETURN
        || ch == CharacterCodes::LINE_SEPARATOR
        || ch == CharacterCodes::PARAGRAPH_SEPARATOR
}

/// Check if a character is a valid regex flag (g, i, m, s, u, v, y, d)
fn is_regex_flag(ch: u32) -> bool {
    matches!(
        ch,
        CharacterCodes::LOWER_G  // g - global
        | CharacterCodes::LOWER_I  // i - ignore case
        | CharacterCodes::LOWER_M  // m - multiline
        | CharacterCodes::LOWER_S  // s - dotAll
        | CharacterCodes::LOWER_U  // u - unicode
        | CharacterCodes::LOWER_V  // v - unicode sets
        | CharacterCodes::LOWER_Y  // y - sticky
        | CharacterCodes::LOWER_D  // d - has indices
    )
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_empty() {
        let mut scanner = ScannerState::new(String::new(), true);
        assert_eq!(scanner.scan(), SyntaxKind::EndOfFileToken);
    }

    #[test]
    fn test_scan_whitespace() {
        let mut scanner = ScannerState::new("   ".to_string(), false);
        assert_eq!(scanner.scan(), SyntaxKind::WhitespaceTrivia);
        assert_eq!(scanner.scan(), SyntaxKind::EndOfFileToken);
    }

    #[test]
    fn test_scan_whitespace_skip() {
        let mut scanner = ScannerState::new("   foo".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::Identifier);
        assert_eq!(scanner.get_token_value(), "foo");
    }

    #[test]
    fn test_scan_newline() {
        let mut scanner = ScannerState::new("\n".to_string(), false);
        assert_eq!(scanner.scan(), SyntaxKind::NewLineTrivia);
        assert!(scanner.has_preceding_line_break());
    }

    #[test]
    fn test_scan_punctuation() {
        let mut scanner = ScannerState::new("{}()[];,".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::OpenBraceToken);
        assert_eq!(scanner.scan(), SyntaxKind::CloseBraceToken);
        assert_eq!(scanner.scan(), SyntaxKind::OpenParenToken);
        assert_eq!(scanner.scan(), SyntaxKind::CloseParenToken);
        assert_eq!(scanner.scan(), SyntaxKind::OpenBracketToken);
        assert_eq!(scanner.scan(), SyntaxKind::CloseBracketToken);
        assert_eq!(scanner.scan(), SyntaxKind::SemicolonToken);
        assert_eq!(scanner.scan(), SyntaxKind::CommaToken);
    }

    #[test]
    fn test_scan_operators() {
        let mut scanner = ScannerState::new("+ - * / =".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::PlusToken);
        assert_eq!(scanner.scan(), SyntaxKind::MinusToken);
        assert_eq!(scanner.scan(), SyntaxKind::AsteriskToken);
        assert_eq!(scanner.scan(), SyntaxKind::SlashToken);
        assert_eq!(scanner.scan(), SyntaxKind::EqualsToken);
    }

    #[test]
    fn test_scan_compound_operators() {
        let mut scanner = ScannerState::new("=== !== == != => && || ??".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::EqualsEqualsEqualsToken);
        assert_eq!(scanner.scan(), SyntaxKind::ExclamationEqualsEqualsToken);
        assert_eq!(scanner.scan(), SyntaxKind::EqualsEqualsToken);
        assert_eq!(scanner.scan(), SyntaxKind::ExclamationEqualsToken);
        assert_eq!(scanner.scan(), SyntaxKind::EqualsGreaterThanToken);
        assert_eq!(scanner.scan(), SyntaxKind::AmpersandAmpersandToken);
        assert_eq!(scanner.scan(), SyntaxKind::BarBarToken);
        assert_eq!(scanner.scan(), SyntaxKind::QuestionQuestionToken);
    }

    #[test]
    fn test_scan_string_literal() {
        let mut scanner = ScannerState::new("\"hello\"".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::StringLiteral);
        assert_eq!(scanner.get_token_value(), "hello");
    }

    #[test]
    fn test_scan_string_with_escapes() {
        let mut scanner = ScannerState::new("\"hello\\nworld\"".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::StringLiteral);
        assert_eq!(scanner.get_token_value(), "hello\nworld");
    }

    #[test]
    fn test_scan_single_quote_string() {
        let mut scanner = ScannerState::new("'test'".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::StringLiteral);
        assert_eq!(scanner.get_token_value(), "test");
    }

    #[test]
    fn test_scan_number() {
        let mut scanner = ScannerState::new("42".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::NumericLiteral);
        assert_eq!(scanner.get_token_value(), "42");
    }

    #[test]
    fn test_scan_decimal_number() {
        let mut scanner = ScannerState::new("3.14".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::NumericLiteral);
        assert_eq!(scanner.get_token_value(), "3.14");
    }

    #[test]
    fn test_scan_hex_number() {
        let mut scanner = ScannerState::new("0xFF".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::NumericLiteral);
        assert_eq!(scanner.get_token_value(), "0xFF");
    }

    #[test]
    fn test_scan_bigint() {
        let mut scanner = ScannerState::new("123n".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::BigIntLiteral);
        assert_eq!(scanner.get_token_value(), "123n");
    }

    #[test]
    fn test_scan_identifier() {
        let mut scanner = ScannerState::new("myVar".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::Identifier);
        assert_eq!(scanner.get_token_value(), "myVar");
    }

    #[test]
    fn test_scan_keyword() {
        let mut scanner = ScannerState::new("const".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::ConstKeyword);
    }

    #[test]
    fn test_scan_let_keyword() {
        let mut scanner = ScannerState::new("let".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::LetKeyword);
    }

    #[test]
    fn test_scan_comment() {
        let mut scanner = ScannerState::new("// comment\nfoo".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::Identifier);
        assert_eq!(scanner.get_token_value(), "foo");
    }

    #[test]
    fn test_scan_multiline_comment() {
        let mut scanner = ScannerState::new("/* comment */foo".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::Identifier);
        assert_eq!(scanner.get_token_value(), "foo");
    }

    #[test]
    fn test_scan_template_literal() {
        let mut scanner = ScannerState::new("`hello`".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::NoSubstitutionTemplateLiteral);
        assert_eq!(scanner.get_token_value(), "hello");
    }

    #[test]
    fn test_scan_template_head() {
        let mut scanner = ScannerState::new("`hello ${".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::TemplateHead);
        assert_eq!(scanner.get_token_value(), "hello ");
    }

    #[test]
    fn test_scan_expression() {
        let mut scanner = ScannerState::new("const x = 42;".to_string(), true);
        assert_eq!(scanner.scan(), SyntaxKind::ConstKeyword);
        assert_eq!(scanner.scan(), SyntaxKind::Identifier);
        assert_eq!(scanner.get_token_value(), "x");
        assert_eq!(scanner.scan(), SyntaxKind::EqualsToken);
        assert_eq!(scanner.scan(), SyntaxKind::NumericLiteral);
        assert_eq!(scanner.get_token_value(), "42");
        assert_eq!(scanner.scan(), SyntaxKind::SemicolonToken);
        assert_eq!(scanner.scan(), SyntaxKind::EndOfFileToken);
    }
}
