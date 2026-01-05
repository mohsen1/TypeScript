//! Tests for scanner.rs

use crate::scanner::*;

#[test]
fn test_token_is_keyword() {
    assert!(token_is_keyword(SyntaxKind::BreakKeyword));
    assert!(token_is_keyword(SyntaxKind::ConstKeyword));
    assert!(token_is_keyword(SyntaxKind::DeferKeyword));
    assert!(!token_is_keyword(SyntaxKind::Identifier));
    assert!(!token_is_keyword(SyntaxKind::OpenBraceToken));
}

#[test]
fn test_token_is_identifier_or_keyword() {
    assert!(token_is_identifier_or_keyword(SyntaxKind::Identifier));
    assert!(token_is_identifier_or_keyword(SyntaxKind::BreakKeyword));
    assert!(!token_is_identifier_or_keyword(SyntaxKind::OpenBraceToken));
}

#[test]
fn test_token_is_punctuation() {
    assert!(token_is_punctuation(SyntaxKind::OpenBraceToken));
    assert!(token_is_punctuation(SyntaxKind::EqualsToken));
    assert!(!token_is_punctuation(SyntaxKind::Identifier));
}

#[test]
fn test_token_is_assignment_operator() {
    assert!(token_is_assignment_operator(SyntaxKind::EqualsToken));
    assert!(token_is_assignment_operator(SyntaxKind::PlusEqualsToken));
    assert!(!token_is_assignment_operator(SyntaxKind::PlusToken));
}

#[test]
fn test_keyword_to_text() {
    assert_eq!(keyword_to_text(SyntaxKind::BreakKeyword), Some("break".into()));
    assert_eq!(keyword_to_text(SyntaxKind::ConstKeyword), Some("const".into()));
    assert_eq!(keyword_to_text(SyntaxKind::AsyncKeyword), Some("async".into()));
    assert_eq!(keyword_to_text(SyntaxKind::Identifier), None);
}

#[test]
fn test_punctuation_to_text() {
    assert_eq!(punctuation_to_text(SyntaxKind::OpenBraceToken), Some("{".into()));
    assert_eq!(punctuation_to_text(SyntaxKind::EqualsEqualsEqualsToken), Some("===".into()));
    assert_eq!(punctuation_to_text(SyntaxKind::EqualsGreaterThanToken), Some("=>".into()));
    assert_eq!(punctuation_to_text(SyntaxKind::Identifier), None);
}

#[test]
fn test_syntax_kind_values() {
    // Verify some key values match TypeScript
    assert_eq!(SyntaxKind::Unknown as u16, 0);
    assert_eq!(SyntaxKind::EndOfFileToken as u16, 1);
    assert_eq!(SyntaxKind::Identifier as u16, 80);
    assert_eq!(SyntaxKind::BreakKeyword as u16, 83);
}

#[test]
fn test_text_to_keyword() {
    // Reserved words
    assert_eq!(text_to_keyword("const"), Some(SyntaxKind::ConstKeyword));
    assert_eq!(text_to_keyword("function"), Some(SyntaxKind::FunctionKeyword));
    assert_eq!(text_to_keyword("return"), Some(SyntaxKind::ReturnKeyword));
    // Strict mode reserved
    assert_eq!(text_to_keyword("let"), Some(SyntaxKind::LetKeyword));
    assert_eq!(text_to_keyword("yield"), Some(SyntaxKind::YieldKeyword));
    // Contextual keywords
    assert_eq!(text_to_keyword("async"), Some(SyntaxKind::AsyncKeyword));
    assert_eq!(text_to_keyword("await"), Some(SyntaxKind::AwaitKeyword));
    assert_eq!(text_to_keyword("type"), Some(SyntaxKind::TypeKeyword));
    // Not a keyword
    assert_eq!(text_to_keyword("foo"), None);
    assert_eq!(text_to_keyword("bar"), None);
    assert_eq!(text_to_keyword("CONST"), None); // Case sensitive
}

#[test]
fn test_string_to_token() {
    assert_eq!(string_to_token("const"), SyntaxKind::ConstKeyword);
    assert_eq!(string_to_token("async"), SyntaxKind::AsyncKeyword);
    assert_eq!(string_to_token("foo"), SyntaxKind::Identifier);
    assert_eq!(string_to_token("myVar"), SyntaxKind::Identifier);
}
