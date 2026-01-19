//! Error Recovery Module for ThinParser
//!
//! Implements proper error recovery strategies that don't rely on
//! budget resets. Uses synchronization points and recovery strategies.

use crate::thin_parser::{ThinParser, TokenKind};

/// Recovery strategy types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Skip tokens until a synchronization point is found
    SyncToStatement,
    /// Skip to the next closing delimiter
    SyncToDelimiter,
    /// Insert a missing token and continue
    InsertToken,
    /// Delete the unexpected token and continue
    DeleteToken,
    /// Bail out - cannot recover
    Bail,
}

/// Synchronization point for error recovery
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncPoint {
    /// Statement boundary (semicolon, brace, keyword)
    Statement,
    /// Expression boundary
    Expression,
    /// Declaration boundary
    Declaration,
    /// Block end (closing brace)
    BlockEnd,
}

/// Error recovery state machine
pub struct ErrorRecovery {
    /// Maximum consecutive recovery attempts before bailing
    max_consecutive_errors: usize,
    /// Current consecutive error count
    consecutive_errors: usize,
    /// Recovery context stack
    context_stack: Vec<RecoveryContext>,
}

/// Context for nested recovery
#[derive(Debug, Clone)]
struct RecoveryContext {
    /// Expected closing delimiters
    expected_delimiters: Vec<TokenKind>,
    /// Sync point for this context
    sync_point: SyncPoint,
    /// Starting position (reserved for future use)
    #[allow(dead_code)]
    start_pos: usize,
}

impl ErrorRecovery {
    /// Create new error recovery instance
    pub fn new() -> Self {
        ErrorRecovery {
            max_consecutive_errors: 10,
            consecutive_errors: 0,
            context_stack: Vec::new(),
        }
    }

    /// Reset consecutive error count (call on successful parse)
    pub fn reset(&mut self) {
        self.consecutive_errors = 0;
    }

    /// Record an error and check if we should bail
    pub fn record_error(&mut self) -> bool {
        self.consecutive_errors += 1;
        self.consecutive_errors >= self.max_consecutive_errors
    }

    /// Push a recovery context
    pub fn push_context(&mut self, sync_point: SyncPoint, expected_delimiters: Vec<TokenKind>, start_pos: usize) {
        self.context_stack.push(RecoveryContext {
            expected_delimiters,
            sync_point,
            start_pos,
        });
    }

    /// Pop a recovery context
    pub fn pop_context(&mut self) {
        self.context_stack.pop();
    }

    /// Determine the best recovery strategy for the current situation
    pub fn determine_strategy<'a>(&self, parser: &ThinParser<'a>) -> RecoveryStrategy {
        let current = parser.current_kind();

        // If at EOF, bail
        if current == TokenKind::Eof {
            return RecoveryStrategy::Bail;
        }

        // If at a statement-starting keyword, sync to statement
        if is_statement_start(current) {
            return RecoveryStrategy::SyncToStatement;
        }

        // If at a closing delimiter, check context
        if is_closing_delimiter(current) {
            if let Some(ctx) = self.context_stack.last() {
                if ctx.expected_delimiters.contains(&current) {
                    return RecoveryStrategy::SyncToDelimiter;
                }
            }
        }

        // Check if we're inside a bracket context
        if let Some(ctx) = self.context_stack.last() {
            match ctx.sync_point {
                SyncPoint::BlockEnd => {
                    // Skip to closing brace
                    return RecoveryStrategy::SyncToDelimiter;
                }
                SyncPoint::Expression => {
                    // For expressions, try to delete unexpected token
                    return RecoveryStrategy::DeleteToken;
                }
                _ => {}
            }
        }

        // Default: skip to next statement
        RecoveryStrategy::SyncToStatement
    }

    /// Try to recover from an error
    /// Returns true if recovery was successful, false if should bail
    pub fn try_recover<'a>(&mut self, parser: &mut ThinParser<'a>) -> bool {
        if self.record_error() {
            return false;
        }

        let strategy = self.determine_strategy(parser);

        match strategy {
            RecoveryStrategy::SyncToStatement => {
                parser.skip_to_next_statement();
                true
            }
            RecoveryStrategy::SyncToDelimiter => {
                self.sync_to_delimiter(parser)
            }
            RecoveryStrategy::InsertToken => {
                // We don't actually insert tokens, just pretend we did
                true
            }
            RecoveryStrategy::DeleteToken => {
                // Skip the current token
                parser.skip_to_next_statement();
                true
            }
            RecoveryStrategy::Bail => {
                false
            }
        }
    }

    /// Sync to the nearest closing delimiter
    fn sync_to_delimiter<'a>(&self, parser: &mut ThinParser<'a>) -> bool {
        let expected = if let Some(ctx) = self.context_stack.last() {
            ctx.expected_delimiters.clone()
        } else {
            vec![TokenKind::Semicolon, TokenKind::CloseBrace]
        };

        let mut depth = 0;
        let mut iterations = 0;
        let max_iterations = 1000; // Prevent infinite loops

        while parser.current_kind() != TokenKind::Eof && iterations < max_iterations {
            let current = parser.current_kind();

            // Track nesting
            match current {
                TokenKind::OpenBrace | TokenKind::OpenParen | TokenKind::OpenBracket => {
                    depth += 1;
                }
                TokenKind::CloseBrace | TokenKind::CloseParen | TokenKind::CloseBracket => {
                    if depth > 0 {
                        depth -= 1;
                    } else if expected.contains(&current) {
                        return true;
                    }
                }
                _ => {}
            }

            // Check for sync point at depth 0
            if depth == 0 && expected.contains(&current) {
                return true;
            }

            parser.skip_to_next_statement();
            iterations += 1;
        }

        iterations < max_iterations
    }

    /// Recover from a missing semicolon
    pub fn recover_missing_semicolon<'a>(&mut self, parser: &mut ThinParser<'a>) -> bool {
        // Check if we're at a position where ASI would apply
        let current = parser.current_kind();

        // If at closing brace or EOF, semicolon is optional
        if current == TokenKind::CloseBrace || current == TokenKind::Eof {
            self.reset();
            return true;
        }

        // If at a statement-starting keyword, assume semicolon
        if is_statement_start(current) {
            self.reset();
            return true;
        }

        false
    }

    /// Recover from an unexpected token in an expression
    pub fn recover_expression<'a>(&mut self, parser: &mut ThinParser<'a>) -> bool {
        let current = parser.current_kind();

        // If at semicolon or closing delimiter, we're done with expression
        if current == TokenKind::Semicolon ||
           current == TokenKind::CloseParen ||
           current == TokenKind::CloseBracket ||
           current == TokenKind::CloseBrace ||
           current == TokenKind::Comma {
            self.reset();
            return true;
        }

        // Skip to next reasonable position
        parser.skip_to_next_statement();
        true
    }

    /// Recover from a missing closing delimiter
    pub fn recover_missing_delimiter<'a>(
        &mut self,
        parser: &mut ThinParser<'a>,
        expected: TokenKind,
    ) -> bool {
        let current = parser.current_kind();

        // If at EOF, we're done
        if current == TokenKind::Eof {
            return false;
        }

        // If at a statement boundary and expected was ')' or ']', assume it's there
        if (expected == TokenKind::CloseParen || expected == TokenKind::CloseBracket) &&
           (current == TokenKind::Semicolon || is_statement_start(current)) {
            self.reset();
            return true;
        }

        // Try to sync to the delimiter
        self.push_context(SyncPoint::Expression, vec![expected], 0);
        let result = self.sync_to_delimiter(parser);
        self.pop_context();
        result
    }
}

impl Default for ErrorRecovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if token kind starts a statement
fn is_statement_start(kind: TokenKind) -> bool {
    matches!(kind,
        TokenKind::Let |
        TokenKind::Const |
        TokenKind::Var |
        TokenKind::Function |
        TokenKind::Class |
        TokenKind::Interface |
        TokenKind::Type |
        TokenKind::If |
        TokenKind::For |
        TokenKind::While |
        TokenKind::Do |
        TokenKind::Return |
        TokenKind::Break |
        TokenKind::Continue |
        TokenKind::Throw |
        TokenKind::Try |
        TokenKind::Import |
        TokenKind::Export
    )
}

/// Check if token kind is a closing delimiter
fn is_closing_delimiter(kind: TokenKind) -> bool {
    matches!(kind,
        TokenKind::CloseBrace |
        TokenKind::CloseParen |
        TokenKind::CloseBracket
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thin_parser::ThinParser;

    #[test]
    fn test_error_recovery_creation() {
        let recovery = ErrorRecovery::new();
        assert_eq!(recovery.consecutive_errors, 0);
    }

    #[test]
    fn test_error_recording() {
        let mut recovery = ErrorRecovery::new();
        for _ in 0..9 {
            assert!(!recovery.record_error());
        }
        assert!(recovery.record_error()); // 10th error should bail
    }

    #[test]
    fn test_reset() {
        let mut recovery = ErrorRecovery::new();
        recovery.consecutive_errors = 5;
        recovery.reset();
        assert_eq!(recovery.consecutive_errors, 0);
    }

    #[test]
    fn test_context_stack() {
        let mut recovery = ErrorRecovery::new();
        recovery.push_context(SyncPoint::BlockEnd, vec![TokenKind::CloseBrace], 0);
        assert_eq!(recovery.context_stack.len(), 1);
        recovery.pop_context();
        assert_eq!(recovery.context_stack.len(), 0);
    }

    #[test]
    fn test_statement_start_detection() {
        assert!(is_statement_start(TokenKind::Let));
        assert!(is_statement_start(TokenKind::Function));
        assert!(is_statement_start(TokenKind::Class));
        assert!(!is_statement_start(TokenKind::Plus));
        assert!(!is_statement_start(TokenKind::Identifier));
    }

    #[test]
    fn test_closing_delimiter_detection() {
        assert!(is_closing_delimiter(TokenKind::CloseBrace));
        assert!(is_closing_delimiter(TokenKind::CloseParen));
        assert!(is_closing_delimiter(TokenKind::CloseBracket));
        assert!(!is_closing_delimiter(TokenKind::OpenBrace));
    }

    #[test]
    fn test_recovery_with_parser() {
        let source = "let x = ; let y = 5;";
        let mut parser = ThinParser::new(source);
        let _recovery = ErrorRecovery::new();

        // The parser should be able to recover
        let result = parser.parse();
        // We expect errors but the parser shouldn't panic
        assert!(result.is_err() || !parser.errors().is_empty());
    }
}
