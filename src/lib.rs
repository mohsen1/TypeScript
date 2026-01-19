// TypeScript Scanner - Rust Implementation
// A zero-copy scanner for TypeScript/JavaScript source code

pub mod tokens;
pub mod scanner;
pub mod scanner_impl;
pub mod builtins;

pub use tokens::{keyword_from_str, Span, Token, TokenKind};
pub use scanner::{LanguageVariant, Scanner, ScriptTarget, TokenFlags};
pub use scanner_impl::{
    compute_line_starts,
    get_binary_operator_precedence,
    get_line_and_character,
    get_position_of_line_and_character,
    is_assignment_operator,
    is_line_break,
    is_white_space_single_line,
    scan_all_tokens,
    scan_tokens,
    DiagnosticMessage,
    DiagnosticScanner,
    Precedence,
    ScanResult,
    TokenIterator,
};
pub use builtins::{
    Type,
    TypeParameter,
    ParameterDeclaration,
    PropertySignature,
    MethodSignature,
    InterfaceDeclaration,
    TypeAliasDeclaration,
    VariableDeclaration,
    FunctionDeclaration,
    LibDeclarations,
    LibLoader,
    LibLoaderConfig,
    LibFile,
};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_typescript_file() {
        // Note: This test uses TypeScript without JSX to avoid template literal
        // rescanning issues (template literal continuation requires parser context)
        let source = r#"
// TypeScript example file
import { Component } from 'react';

interface Props {
    name: string;
    count?: number;
}

interface State {
    items: string[];
}

class MyComponent extends Component<Props, State> {
    private readonly id: number;

    constructor(props: Props) {
        super(props);
        this.id = Math.random();
        this.state = { items: [] };
    }

    async fetchItems(): Promise<void> {
        const response = await fetch('/api/items');
        const data = await response.json();
        this.setState({ items: data ?? [] });
    }

    render() {
        const { name, count = 0 } = this.props;
        const { items } = this.state;
        return items.length;
    }
}

export default MyComponent;

// Generic function
function identity<T>(arg: T): T {
    return arg;
}

// Arrow function with type
const add = (a: number, b: number): number => a + b;

// Nullish coalescing and optional chaining
const value = obj?.nested?.property ?? 'default';

// Simple template literal (no substitution)
const message = `Hello, world!`;

// BigInt and numeric separators
const bigNum = 1_000_000n;
const hexNum = 0xFF_FF;

// Async generator
async function* asyncGen() {
    yield await Promise.resolve(1);
    yield await Promise.resolve(2);
}

// Decorators
@decorator
class Decorated {
    @property accessor field = 42;
}
"#;

        let result = scan_all_tokens(source);

        // Should scan without errors (except maybe unterminated JSX which is expected)
        // The main goal is that it tokenizes the TypeScript syntax correctly

        // Verify key TypeScript tokens are recognized
        let has_interface = result.tokens.iter().any(|t| t.kind == TokenKind::Interface);
        let has_class = result.tokens.iter().any(|t| t.kind == TokenKind::Class);
        let has_async = result.tokens.iter().any(|t| t.kind == TokenKind::Async);
        let has_await = result.tokens.iter().any(|t| t.kind == TokenKind::Await);
        let has_private = result.tokens.iter().any(|t| t.kind == TokenKind::Private);
        let has_readonly = result.tokens.iter().any(|t| t.kind == TokenKind::Readonly);
        let has_export = result.tokens.iter().any(|t| t.kind == TokenKind::Export);
        let has_import = result.tokens.iter().any(|t| t.kind == TokenKind::Import);
        let has_extends = result.tokens.iter().any(|t| t.kind == TokenKind::Extends);
        let has_return = result.tokens.iter().any(|t| t.kind == TokenKind::Return);
        let has_const = result.tokens.iter().any(|t| t.kind == TokenKind::Const);
        let has_function = result.tokens.iter().any(|t| t.kind == TokenKind::Function);
        let has_yield = result.tokens.iter().any(|t| t.kind == TokenKind::Yield);

        assert!(has_interface, "Should recognize 'interface' keyword");
        assert!(has_class, "Should recognize 'class' keyword");
        assert!(has_async, "Should recognize 'async' keyword");
        assert!(has_await, "Should recognize 'await' keyword");
        assert!(has_private, "Should recognize 'private' keyword");
        assert!(has_readonly, "Should recognize 'readonly' keyword");
        assert!(has_export, "Should recognize 'export' keyword");
        assert!(has_import, "Should recognize 'import' keyword");
        assert!(has_extends, "Should recognize 'extends' keyword");
        assert!(has_return, "Should recognize 'return' keyword");
        assert!(has_const, "Should recognize 'const' keyword");
        assert!(has_function, "Should recognize 'function' keyword");
        assert!(has_yield, "Should recognize 'yield' keyword");

        // Verify operators
        let has_arrow = result.tokens.iter().any(|t| t.kind == TokenKind::EqualsGreaterThan);
        let has_optional_chain = result.tokens.iter().any(|t| t.kind == TokenKind::QuestionDot);
        let has_nullish = result.tokens.iter().any(|t| t.kind == TokenKind::QuestionQuestion);

        assert!(has_arrow, "Should recognize '=>' operator");
        assert!(has_optional_chain, "Should recognize '?.' operator");
        assert!(has_nullish, "Should recognize '??' operator");

        // Verify literals
        let has_bigint = result.tokens.iter().any(|t| t.kind == TokenKind::BigIntLiteral);
        let has_template = result.tokens.iter().any(|t| t.kind == TokenKind::TemplateHead || t.kind == TokenKind::NoSubstitutionTemplateLiteral);
        let has_string = result.tokens.iter().any(|t| t.kind == TokenKind::StringLiteral);
        let has_number = result.tokens.iter().any(|t| t.kind == TokenKind::NumericLiteral);

        assert!(has_bigint, "Should recognize BigInt literals");
        assert!(has_template, "Should recognize template literals");
        assert!(has_string, "Should recognize string literals");
        assert!(has_number, "Should recognize numeric literals");

        // Verify decorator
        let has_at = result.tokens.iter().any(|t| t.kind == TokenKind::At);
        assert!(has_at, "Should recognize '@' decorator syntax");

        // Verify comments were captured as trivia
        let has_comments = result.trivia.iter().any(|t| t.kind == TokenKind::SingleLineComment);
        assert!(has_comments, "Should capture comments as trivia");
    }

    #[test]
    fn test_scanner_zero_copy() {
        // Verify that scanner truly uses zero-copy
        let source = "const value = 123;";
        let source_ptr = source.as_ptr();

        let mut scanner = Scanner::new(source);

        // Scanner should hold reference to same memory
        assert_eq!(scanner.get_text().as_ptr(), source_ptr);

        // Token text should be slice of same memory
        scanner.scan(); // const
        let token_text = scanner.get_token_text();
        assert!(token_text.as_ptr() >= source_ptr);
        assert!(token_text.as_ptr() < unsafe { source_ptr.add(source.len()) });
    }

    #[test]
    fn test_scanner_spans() {
        let source = "let x = 42;";
        let tokens = scan_tokens(source);

        // Verify all spans are valid
        for token in &tokens {
            assert!(token.span.start <= token.span.end);
            assert!(token.span.end <= source.len());

            // Token text from span should match
            let text = token.text(source);
            assert!(!text.is_empty() || token.kind == TokenKind::EndOfFile);
        }

        // Verify specific spans
        assert_eq!(tokens[0].text(source), "let");
        assert_eq!(tokens[1].text(source), "x");
        assert_eq!(tokens[2].text(source), "=");
        assert_eq!(tokens[3].text(source), "42");
        assert_eq!(tokens[4].text(source), ";");
    }

    #[test]
    fn test_all_operators() {
        let operators = [
            // Punctuation
            ("{", TokenKind::OpenBrace),
            ("}", TokenKind::CloseBrace),
            ("(", TokenKind::OpenParen),
            (")", TokenKind::CloseParen),
            ("[", TokenKind::OpenBracket),
            ("]", TokenKind::CloseBracket),
            (".", TokenKind::Dot),
            ("...", TokenKind::DotDotDot),
            (";", TokenKind::Semicolon),
            (",", TokenKind::Comma),
            (":", TokenKind::Colon),
            ("@", TokenKind::At),
            ("~", TokenKind::Tilde),
            // Comparison
            ("<", TokenKind::LessThan),
            (">", TokenKind::GreaterThan),
            ("<=", TokenKind::LessThanEquals),
            (">=", TokenKind::GreaterThanEquals),
            ("==", TokenKind::EqualsEquals),
            ("!=", TokenKind::ExclamationEquals),
            ("===", TokenKind::EqualsEqualsEquals),
            ("!==", TokenKind::ExclamationEqualsEquals),
            // Arithmetic
            ("+", TokenKind::Plus),
            ("-", TokenKind::Minus),
            ("*", TokenKind::Asterisk),
            ("/", TokenKind::Slash),
            ("%", TokenKind::Percent),
            ("**", TokenKind::AsteriskAsterisk),
            ("++", TokenKind::PlusPlus),
            ("--", TokenKind::MinusMinus),
            // Bitwise
            ("&", TokenKind::Ampersand),
            ("|", TokenKind::Bar),
            ("^", TokenKind::Caret),
            ("!", TokenKind::Exclamation),
            ("<<", TokenKind::LessThanLessThan),
            (">>", TokenKind::GreaterThanGreaterThan),
            (">>>", TokenKind::GreaterThanGreaterThanGreaterThan),
            // Logical
            ("&&", TokenKind::AmpersandAmpersand),
            ("||", TokenKind::BarBar),
            ("??", TokenKind::QuestionQuestion),
            // Assignment
            ("=", TokenKind::Equals),
            ("+=", TokenKind::PlusEquals),
            ("-=", TokenKind::MinusEquals),
            ("*=", TokenKind::AsteriskEquals),
            ("**=", TokenKind::AsteriskAsteriskEquals),
            ("/=", TokenKind::SlashEquals),
            ("%=", TokenKind::PercentEquals),
            ("<<=", TokenKind::LessThanLessThanEquals),
            (">>=", TokenKind::GreaterThanGreaterThanEquals),
            (">>>=", TokenKind::GreaterThanGreaterThanGreaterThanEquals),
            ("&=", TokenKind::AmpersandEquals),
            ("|=", TokenKind::BarEquals),
            ("^=", TokenKind::CaretEquals),
            ("&&=", TokenKind::AmpersandAmpersandEquals),
            ("||=", TokenKind::BarBarEquals),
            ("??=", TokenKind::QuestionQuestionEquals),
            // Other
            ("?", TokenKind::Question),
            ("?.", TokenKind::QuestionDot),
            ("=>", TokenKind::EqualsGreaterThan),
            ("</", TokenKind::LessThanSlash),
        ];

        for (text, expected) in operators {
            let mut scanner = Scanner::new(text);
            let kind = scanner.scan();
            assert_eq!(kind, expected, "Failed for operator: '{}'", text);
        }
    }

    #[test]
    fn test_all_keywords() {
        let keywords = [
            // Reserved words
            ("break", TokenKind::Break),
            ("case", TokenKind::Case),
            ("catch", TokenKind::Catch),
            ("class", TokenKind::Class),
            ("const", TokenKind::Const),
            ("continue", TokenKind::Continue),
            ("debugger", TokenKind::Debugger),
            ("default", TokenKind::Default),
            ("delete", TokenKind::Delete),
            ("do", TokenKind::Do),
            ("else", TokenKind::Else),
            ("enum", TokenKind::Enum),
            ("export", TokenKind::Export),
            ("extends", TokenKind::Extends),
            ("false", TokenKind::False),
            ("finally", TokenKind::Finally),
            ("for", TokenKind::For),
            ("function", TokenKind::Function),
            ("if", TokenKind::If),
            ("import", TokenKind::Import),
            ("in", TokenKind::In),
            ("instanceof", TokenKind::InstanceOf),
            ("new", TokenKind::New),
            ("null", TokenKind::Null),
            ("return", TokenKind::Return),
            ("super", TokenKind::Super),
            ("switch", TokenKind::Switch),
            ("this", TokenKind::This),
            ("throw", TokenKind::Throw),
            ("true", TokenKind::True),
            ("try", TokenKind::Try),
            ("typeof", TokenKind::TypeOf),
            ("var", TokenKind::Var),
            ("void", TokenKind::Void),
            ("while", TokenKind::While),
            ("with", TokenKind::With),
            // Strict mode reserved
            ("implements", TokenKind::Implements),
            ("interface", TokenKind::Interface),
            ("let", TokenKind::Let),
            ("package", TokenKind::Package),
            ("private", TokenKind::Private),
            ("protected", TokenKind::Protected),
            ("public", TokenKind::Public),
            ("static", TokenKind::Static),
            ("yield", TokenKind::Yield),
            // Contextual keywords
            ("abstract", TokenKind::Abstract),
            ("as", TokenKind::As),
            ("asserts", TokenKind::Asserts),
            ("assert", TokenKind::Assert),
            ("any", TokenKind::Any),
            ("async", TokenKind::Async),
            ("await", TokenKind::Await),
            ("boolean", TokenKind::Boolean),
            ("constructor", TokenKind::Constructor),
            ("declare", TokenKind::Declare),
            ("get", TokenKind::Get),
            ("infer", TokenKind::Infer),
            ("intrinsic", TokenKind::Intrinsic),
            ("is", TokenKind::Is),
            ("keyof", TokenKind::KeyOf),
            ("module", TokenKind::Module),
            ("namespace", TokenKind::Namespace),
            ("never", TokenKind::Never),
            ("readonly", TokenKind::Readonly),
            ("require", TokenKind::Require),
            ("number", TokenKind::Number),
            ("object", TokenKind::Object),
            ("set", TokenKind::Set),
            ("string", TokenKind::String),
            ("symbol", TokenKind::Symbol),
            ("type", TokenKind::Type),
            ("undefined", TokenKind::Undefined),
            ("unique", TokenKind::Unique),
            ("unknown", TokenKind::UnknownKeyword),
            ("from", TokenKind::From),
            ("global", TokenKind::Global),
            ("bigint", TokenKind::BigInt),
            ("override", TokenKind::Override),
            ("of", TokenKind::Of),
        ];

        for (text, expected) in keywords {
            let mut scanner = Scanner::new(text);
            let kind = scanner.scan();
            assert_eq!(kind, expected, "Failed for keyword: '{}'", text);
        }
    }
}
