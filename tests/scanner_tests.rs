// Comprehensive scanner tests for TypeScript tokenization

use ts_scanner::*;

#[test]
fn test_empty_source() {
    let mut scanner = Scanner::new("");
    assert_eq!(scanner.scan(), TokenKind::EndOfFile);
}

#[test]
fn test_whitespace_only() {
    let mut scanner = Scanner::new("   \t\t   ");
    assert_eq!(scanner.scan(), TokenKind::EndOfFile);
}

#[test]
fn test_single_identifier() {
    let mut scanner = Scanner::new("foo");
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert_eq!(scanner.get_token_text(), "foo");
    assert_eq!(scanner.scan(), TokenKind::EndOfFile);
}

#[test]
fn test_identifiers_with_underscores() {
    let mut scanner = Scanner::new("_private __dunder _123");
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert_eq!(scanner.get_token_text(), "_private");
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert_eq!(scanner.get_token_text(), "__dunder");
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert_eq!(scanner.get_token_text(), "_123");
}

#[test]
fn test_identifiers_with_dollars() {
    let mut scanner = Scanner::new("$element $$double a$b");
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert_eq!(scanner.get_token_text(), "$element");
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert_eq!(scanner.get_token_text(), "$$double");
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert_eq!(scanner.get_token_text(), "a$b");
}

#[test]
fn test_number_formats() {
    // Decimal
    let mut scanner = Scanner::new("0 42 123456");
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert_eq!(scanner.get_token_text(), "0");
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert_eq!(scanner.get_token_text(), "42");
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert_eq!(scanner.get_token_text(), "123456");

    // Float
    let mut scanner = Scanner::new("3.14 .5 0.123");
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert_eq!(scanner.get_token_text(), "3.14");
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert_eq!(scanner.get_token_text(), ".5");
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert_eq!(scanner.get_token_text(), "0.123");

    // Scientific notation
    let mut scanner = Scanner::new("1e10 1E-5 1.5e+3");
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert!(scanner.get_token_flags().has(TokenFlags::SCIENTIFIC));
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert!(scanner.get_token_flags().has(TokenFlags::SCIENTIFIC));
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert!(scanner.get_token_flags().has(TokenFlags::SCIENTIFIC));

    // Hex
    let mut scanner = Scanner::new("0x1F 0XFF 0xABCDEF");
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert!(scanner.get_token_flags().has(TokenFlags::HEX_SPECIFIER));
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert!(scanner.get_token_flags().has(TokenFlags::HEX_SPECIFIER));
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert!(scanner.get_token_flags().has(TokenFlags::HEX_SPECIFIER));

    // Binary
    let mut scanner = Scanner::new("0b1010 0B11110000");
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert!(scanner.get_token_flags().has(TokenFlags::BINARY_SPECIFIER));
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert!(scanner.get_token_flags().has(TokenFlags::BINARY_SPECIFIER));

    // Octal
    let mut scanner = Scanner::new("0o755 0O777");
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert!(scanner.get_token_flags().has(TokenFlags::OCTAL_SPECIFIER));
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert!(scanner.get_token_flags().has(TokenFlags::OCTAL_SPECIFIER));

    // Numeric separators
    let mut scanner = Scanner::new("1_000_000 0xFF_FF 0b1111_0000");
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert!(scanner.get_token_flags().has(TokenFlags::CONTAINS_SEPARATOR));
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert!(scanner.get_token_flags().has(TokenFlags::CONTAINS_SEPARATOR));
    assert_eq!(scanner.scan(), TokenKind::NumericLiteral);
    assert!(scanner.get_token_flags().has(TokenFlags::CONTAINS_SEPARATOR));
}

#[test]
fn test_bigint_literals() {
    let mut scanner = Scanner::new("42n 0xFFn 0b1010n 0o777n");
    assert_eq!(scanner.scan(), TokenKind::BigIntLiteral);
    assert_eq!(scanner.get_token_text(), "42n");
    assert_eq!(scanner.scan(), TokenKind::BigIntLiteral);
    assert_eq!(scanner.get_token_text(), "0xFFn");
    assert_eq!(scanner.scan(), TokenKind::BigIntLiteral);
    assert_eq!(scanner.get_token_text(), "0b1010n");
    assert_eq!(scanner.scan(), TokenKind::BigIntLiteral);
    assert_eq!(scanner.get_token_text(), "0o777n");
}

#[test]
fn test_string_literals() {
    // Single quotes
    let mut scanner = Scanner::new("'hello' 'world'");
    assert_eq!(scanner.scan(), TokenKind::StringLiteral);
    assert_eq!(scanner.get_token_text(), "'hello'");
    assert_eq!(scanner.scan(), TokenKind::StringLiteral);
    assert_eq!(scanner.get_token_text(), "'world'");

    // Double quotes
    let mut scanner = Scanner::new("\"hello\" \"world\"");
    assert_eq!(scanner.scan(), TokenKind::StringLiteral);
    assert_eq!(scanner.get_token_text(), "\"hello\"");
    assert_eq!(scanner.scan(), TokenKind::StringLiteral);
    assert_eq!(scanner.get_token_text(), "\"world\"");

    // Escape sequences
    let mut scanner = Scanner::new(r#"'hello\nworld' "tab\there""#);
    assert_eq!(scanner.scan(), TokenKind::StringLiteral);
    assert_eq!(scanner.scan(), TokenKind::StringLiteral);

    // Escaped quotes
    let mut scanner = Scanner::new(r#"'it\'s' "say \"hi\"""#);
    assert_eq!(scanner.scan(), TokenKind::StringLiteral);
    assert_eq!(scanner.scan(), TokenKind::StringLiteral);
}

#[test]
fn test_template_literals() {
    // Simple template
    let mut scanner = Scanner::new("`hello`");
    assert_eq!(scanner.scan(), TokenKind::NoSubstitutionTemplateLiteral);
    assert_eq!(scanner.get_token_text(), "`hello`");

    // Template with substitution
    let mut scanner = Scanner::new("`hello ${name}!`");
    assert_eq!(scanner.scan(), TokenKind::TemplateHead);
    assert_eq!(scanner.get_token_text(), "`hello ${");

    // Empty template
    let mut scanner = Scanner::new("``");
    assert_eq!(scanner.scan(), TokenKind::NoSubstitutionTemplateLiteral);
}

#[test]
fn test_comments() {
    // Single-line comment
    let mut scanner = Scanner::new("// this is a comment\ncode");
    assert_eq!(scanner.scan(), TokenKind::SingleLineComment);
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert!(scanner.has_preceding_line_break());

    // Multi-line comment
    let mut scanner = Scanner::new("/* block\ncomment */");
    assert_eq!(scanner.scan(), TokenKind::MultiLineComment);

    // Nested /* in multi-line (not really nested, just contains /*)
    let mut scanner = Scanner::new("/* outer /* inner */");
    assert_eq!(scanner.scan(), TokenKind::MultiLineComment);
}

#[test]
fn test_shebang() {
    let mut scanner = Scanner::new("#!/usr/bin/env node\nconst x = 1;");
    assert_eq!(scanner.scan(), TokenKind::Shebang);
    assert_eq!(scanner.get_token_text(), "#!/usr/bin/env node");
    assert_eq!(scanner.scan(), TokenKind::Const);
}

#[test]
fn test_shebang_not_at_start() {
    // # at position > 0 should be hash token, not shebang
    let mut scanner = Scanner::new("x #!");
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert_eq!(scanner.scan(), TokenKind::Hash);
    assert_eq!(scanner.scan(), TokenKind::Exclamation);
}

#[test]
fn test_private_identifiers() {
    let mut scanner = Scanner::new("#private #_internal #$special");
    assert_eq!(scanner.scan(), TokenKind::PrivateIdentifier);
    assert_eq!(scanner.get_token_text(), "#private");
    assert_eq!(scanner.scan(), TokenKind::PrivateIdentifier);
    assert_eq!(scanner.get_token_text(), "#_internal");
    assert_eq!(scanner.scan(), TokenKind::PrivateIdentifier);
    assert_eq!(scanner.get_token_text(), "#$special");
}

#[test]
fn test_line_breaks() {
    // Unix line endings
    let mut scanner = Scanner::new("a\nb");
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert!(scanner.has_preceding_line_break());

    // Windows line endings
    let mut scanner = Scanner::new("a\r\nb");
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert!(scanner.has_preceding_line_break());

    // Old Mac line endings
    let mut scanner = Scanner::new("a\rb");
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert_eq!(scanner.scan(), TokenKind::Identifier);
    assert!(scanner.has_preceding_line_break());
}

#[test]
fn test_unterminated_string() {
    let mut scanner = Scanner::new("\"unterminated");
    assert_eq!(scanner.scan(), TokenKind::StringLiteral);
    assert!(scanner.is_unterminated());

    let mut scanner = Scanner::new("'unterminated");
    assert_eq!(scanner.scan(), TokenKind::StringLiteral);
    assert!(scanner.is_unterminated());
}

#[test]
fn test_unterminated_multiline_comment() {
    let mut scanner = Scanner::new("/* unterminated");
    assert_eq!(scanner.scan(), TokenKind::MultiLineComment);
    assert!(scanner.is_unterminated());
}

#[test]
fn test_rescan_regex() {
    let mut scanner = Scanner::new("/pattern/gi");
    assert_eq!(scanner.scan(), TokenKind::Slash);
    assert_eq!(scanner.rescan_slash_token(), TokenKind::RegularExpressionLiteral);
    assert_eq!(scanner.get_token_text(), "/pattern/gi");
}

#[test]
fn test_regex_with_escapes() {
    let mut scanner = Scanner::new(r"/hello\/world/");
    assert_eq!(scanner.scan(), TokenKind::Slash);
    assert_eq!(scanner.rescan_slash_token(), TokenKind::RegularExpressionLiteral);
}

#[test]
fn test_regex_with_char_class() {
    let mut scanner = Scanner::new(r"/[a-z/]/");
    assert_eq!(scanner.scan(), TokenKind::Slash);
    assert_eq!(scanner.rescan_slash_token(), TokenKind::RegularExpressionLiteral);
    // The / inside [] should not end the regex
}

#[test]
fn test_typescript_type_syntax() {
    let source = "const x: string | number = 42;";
    let tokens = scan_tokens(source);

    let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Const,
            TokenKind::Identifier,
            TokenKind::Colon,
            TokenKind::String,
            TokenKind::Bar,
            TokenKind::Number,
            TokenKind::Equals,
            TokenKind::NumericLiteral,
            TokenKind::Semicolon,
        ]
    );
}

#[test]
fn test_typescript_generics() {
    // Note: >> is scanned as a single GreaterThanGreaterThan token
    // The parser would need to handle splitting it for type context
    // This is the same behavior as TypeScript's scanner
    let source = "Array<Map<string, number>>";
    let tokens = scan_tokens(source);

    let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Identifier, // Array
            TokenKind::LessThan,
            TokenKind::Identifier, // Map
            TokenKind::LessThan,
            TokenKind::String,
            TokenKind::Comma,
            TokenKind::Number,
            // >> is scanned as one token - parser handles splitting for generics
            TokenKind::GreaterThanGreaterThan,
        ]
    );
}

#[test]
fn test_typescript_optional_chaining() {
    let source = "obj?.prop?.[0]?.method?.()";
    let tokens = scan_tokens(source);

    let question_dots = tokens
        .iter()
        .filter(|t| t.kind == TokenKind::QuestionDot)
        .count();
    assert_eq!(question_dots, 4);
}

#[test]
fn test_typescript_nullish_assignment() {
    let source = "a ??= b; c ||= d; e &&= f;";
    let tokens = scan_tokens(source);

    assert!(tokens.iter().any(|t| t.kind == TokenKind::QuestionQuestionEquals));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::BarBarEquals));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::AmpersandAmpersandEquals));
}

#[test]
fn test_arrow_function() {
    let source = "(x: number): number => x * 2";
    let tokens = scan_tokens(source);

    let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::OpenParen,
            TokenKind::Identifier,
            TokenKind::Colon,
            TokenKind::Number,
            TokenKind::CloseParen,
            TokenKind::Colon,
            TokenKind::Number,
            TokenKind::EqualsGreaterThan,
            TokenKind::Identifier,
            TokenKind::Asterisk,
            TokenKind::NumericLiteral,
        ]
    );
}

#[test]
fn test_async_await() {
    let source = "async function f() { await promise; }";
    let tokens = scan_tokens(source);

    assert!(tokens.iter().any(|t| t.kind == TokenKind::Async));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Await));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Function));
}

#[test]
fn test_class_syntax() {
    let source = "class Foo extends Bar { private readonly x: number; }";
    let tokens = scan_tokens(source);

    assert!(tokens.iter().any(|t| t.kind == TokenKind::Class));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Extends));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Private));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Readonly));
}

#[test]
fn test_interface_syntax() {
    let source = "interface Foo { readonly prop?: string; }";
    let tokens = scan_tokens(source);

    assert!(tokens.iter().any(|t| t.kind == TokenKind::Interface));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Readonly));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Question));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::String));
}

#[test]
fn test_type_alias() {
    let source = "type Nullable<T> = T | null | undefined;";
    let tokens = scan_tokens(source);

    assert!(tokens.iter().any(|t| t.kind == TokenKind::Type));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Null));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Undefined));
}

#[test]
fn test_decorator() {
    let source = "@Component({ selector: 'app' })";
    let tokens = scan_tokens(source);

    assert!(tokens.iter().any(|t| t.kind == TokenKind::At));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Identifier));
}

#[test]
fn test_spread_operator() {
    let source = "const arr = [...a, ...b];";
    let tokens = scan_tokens(source);

    let spreads = tokens
        .iter()
        .filter(|t| t.kind == TokenKind::DotDotDot)
        .count();
    assert_eq!(spreads, 2);
}

#[test]
fn test_computed_property() {
    let source = "const obj = { [key]: value, ['literal']: 42 };";
    let tokens = scan_tokens(source);

    let open_brackets = tokens
        .iter()
        .filter(|t| t.kind == TokenKind::OpenBracket)
        .count();
    assert_eq!(open_brackets, 2);
}

#[test]
fn test_token_spans_accuracy() {
    let source = "const foo = 42;";
    let tokens = scan_tokens(source);

    // Verify each token's span matches its text
    for token in &tokens {
        let text = &source[token.span.start..token.span.end];
        assert_eq!(text, token.text(source));
    }

    // Check specific positions
    assert_eq!(tokens[0].span, Span::new(0, 5)); // const
    assert_eq!(tokens[1].span, Span::new(6, 9)); // foo
    assert_eq!(tokens[2].span, Span::new(10, 11)); // =
    assert_eq!(tokens[3].span, Span::new(12, 14)); // 42
    assert_eq!(tokens[4].span, Span::new(14, 15)); // ;
}

#[test]
fn test_token_iterator() {
    let source = "let x = 1;";
    let tokens: Vec<_> = TokenIterator::new(source).collect();

    assert_eq!(tokens.len(), 5);
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[4].kind, TokenKind::Semicolon);
}

#[test]
fn test_token_iterator_with_trivia() {
    let source = "let /* comment */ x";
    let tokens: Vec<_> = TokenIterator::with_trivia(source).collect();

    assert!(tokens.iter().any(|t| t.kind == TokenKind::MultiLineComment));
}

#[test]
fn test_diagnostic_scanner() {
    let source = "let x = \"unterminated";
    let mut scanner = DiagnosticScanner::new(source);

    loop {
        if scanner.scan() == TokenKind::EndOfFile {
            break;
        }
    }

    let diagnostics = scanner.get_diagnostics();
    assert!(!diagnostics.is_empty());
    assert!(diagnostics[0].message.contains("Unterminated"));
}

#[test]
fn test_line_starts() {
    let source = "line1\nline2\nline3";
    let starts = compute_line_starts(source);

    assert_eq!(starts.len(), 3);
    assert_eq!(starts[0], 0);
    assert_eq!(starts[1], 6);
    assert_eq!(starts[2], 12);
}

#[test]
fn test_line_and_character() {
    let source = "let x\nconst y";
    let (line, char) = get_line_and_character(source, 7);

    assert_eq!(line, 1);
    assert_eq!(char, 1);
}

#[test]
fn test_operator_precedence() {
    #[allow(unused_imports)]
    use Precedence::*;

    assert!(get_binary_operator_precedence(TokenKind::Asterisk) > get_binary_operator_precedence(TokenKind::Plus));
    assert!(get_binary_operator_precedence(TokenKind::Plus) > get_binary_operator_precedence(TokenKind::EqualsEquals));
    assert!(get_binary_operator_precedence(TokenKind::AmpersandAmpersand) > get_binary_operator_precedence(TokenKind::BarBar));
}

#[test]
fn test_real_world_typescript_sample() {
    // Note: Template literals with interpolation require parser context
    // to properly rescan (the scanner returns TemplateHead, then parser
    // calls rescan_template_token after the expression).
    // This test uses simpler TypeScript without interpolated templates.
    let source = r#"
import { useState, useEffect } from 'react';

interface User {
    id: number;
    name: string;
    email?: string;
}

type UserState = User | null;

async function fetchUser(id: number): Promise<User> {
    const response = await fetch('/api/users/' + id);
    if (!response.ok) {
        throw new Error('Failed to fetch user');
    }
    return response.json();
}

export function useUser(id: number): { user: UserState; loading: boolean; error: Error | null } {
    const [user, setUser] = useState<UserState>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<Error | null>(null);

    useEffect(() => {
        fetchUser(id)
            .then(setUser)
            .catch(setError)
            .finally(() => setLoading(false));
    }, [id]);

    return { user, loading, error };
}
"#;

    let result = scan_all_tokens(source);

    // Verify key constructs are tokenized
    assert!(result.tokens.iter().any(|t| t.kind == TokenKind::Import));
    assert!(result.tokens.iter().any(|t| t.kind == TokenKind::Interface));
    assert!(result.tokens.iter().any(|t| t.kind == TokenKind::Type));
    assert!(result.tokens.iter().any(|t| t.kind == TokenKind::Async));
    assert!(result.tokens.iter().any(|t| t.kind == TokenKind::Function));
    assert!(result.tokens.iter().any(|t| t.kind == TokenKind::Await));
    assert!(result.tokens.iter().any(|t| t.kind == TokenKind::Export));
    assert!(result.tokens.iter().any(|t| t.kind == TokenKind::Const));
    assert!(result.tokens.iter().any(|t| t.kind == TokenKind::If));
    assert!(result.tokens.iter().any(|t| t.kind == TokenKind::Throw));
    assert!(result.tokens.iter().any(|t| t.kind == TokenKind::New));
    assert!(result.tokens.iter().any(|t| t.kind == TokenKind::Return));
}
