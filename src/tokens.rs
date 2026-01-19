// Token types matching TypeScript's SyntaxKind for scanner tokens
// This is a zero-copy implementation using spans into the source

/// A span representing a range in the source text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Start byte offset (inclusive)
    pub start: usize,
    /// End byte offset (exclusive)
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Token types matching TypeScript's SyntaxKind enum
/// Only includes tokens produced by the scanner (not AST node types)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum TokenKind {
    // Special
    Unknown = 0,
    EndOfFile,

    // Trivia (comments and whitespace)
    SingleLineComment,
    MultiLineComment,
    NewLine,
    Whitespace,
    Shebang,
    ConflictMarker,

    // Literals
    NumericLiteral,
    BigIntLiteral,
    StringLiteral,
    JsxText,
    JsxTextAllWhiteSpaces,
    RegularExpressionLiteral,
    NoSubstitutionTemplateLiteral,

    // Template literals
    TemplateHead,
    TemplateMiddle,
    TemplateTail,

    // Punctuation
    OpenBrace,          // {
    CloseBrace,         // }
    OpenParen,          // (
    CloseParen,         // )
    OpenBracket,        // [
    CloseBracket,       // ]
    Dot,                // .
    DotDotDot,          // ...
    Semicolon,          // ;
    Comma,              // ,
    QuestionDot,        // ?.
    LessThan,           // <
    LessThanSlash,      // </
    GreaterThan,        // >
    LessThanEquals,     // <=
    GreaterThanEquals,  // >=
    EqualsEquals,       // ==
    ExclamationEquals,  // !=
    EqualsEqualsEquals, // ===
    ExclamationEqualsEquals, // !==
    EqualsGreaterThan,  // =>
    Plus,               // +
    Minus,              // -
    Asterisk,           // *
    AsteriskAsterisk,   // **
    Slash,              // /
    Percent,            // %
    PlusPlus,           // ++
    MinusMinus,         // --
    LessThanLessThan,   // <<
    GreaterThanGreaterThan, // >>
    GreaterThanGreaterThanGreaterThan, // >>>
    Ampersand,          // &
    Bar,                // |
    Caret,              // ^
    Exclamation,        // !
    Tilde,              // ~
    AmpersandAmpersand, // &&
    BarBar,             // ||
    Question,           // ?
    Colon,              // :
    At,                 // @
    QuestionQuestion,   // ??
    Backtick,           // `
    Hash,               // #

    // Assignment operators
    Equals,             // =
    PlusEquals,         // +=
    MinusEquals,        // -=
    AsteriskEquals,     // *=
    AsteriskAsteriskEquals, // **=
    SlashEquals,        // /=
    PercentEquals,      // %=
    LessThanLessThanEquals, // <<=
    GreaterThanGreaterThanEquals, // >>=
    GreaterThanGreaterThanGreaterThanEquals, // >>>=
    AmpersandEquals,    // &=
    BarEquals,          // |=
    BarBarEquals,       // ||=
    AmpersandAmpersandEquals, // &&=
    QuestionQuestionEquals, // ??=
    CaretEquals,        // ^=

    // Identifiers
    Identifier,
    PrivateIdentifier,  // #identifier

    // Reserved keywords
    Break,
    Case,
    Catch,
    Class,
    Const,
    Continue,
    Debugger,
    Default,
    Delete,
    Do,
    Else,
    Enum,
    Export,
    Extends,
    False,
    Finally,
    For,
    Function,
    If,
    Import,
    In,
    InstanceOf,
    New,
    Null,
    Return,
    Super,
    Switch,
    This,
    Throw,
    True,
    Try,
    TypeOf,
    Var,
    Void,
    While,
    With,

    // Strict mode reserved words
    Implements,
    Interface,
    Let,
    Package,
    Private,
    Protected,
    Public,
    Static,
    Yield,

    // Contextual keywords
    Abstract,
    As,
    Asserts,
    Assert,
    Any,
    Async,
    Await,
    Boolean,
    Constructor,
    Declare,
    Get,
    Infer,
    Intrinsic,
    Is,
    KeyOf,
    Module,
    Namespace,
    Never,
    Readonly,
    Require,
    Number,
    Object,
    Set,
    String,
    Symbol,
    Type,
    Undefined,
    Unique,
    UnknownKeyword, // TypeScript 'unknown' type keyword (different from Unknown token)
    From,
    Global,
    BigInt,
    Override,
    Of,
}

impl TokenKind {
    /// Returns true if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Break
                | TokenKind::Case
                | TokenKind::Catch
                | TokenKind::Class
                | TokenKind::Const
                | TokenKind::Continue
                | TokenKind::Debugger
                | TokenKind::Default
                | TokenKind::Delete
                | TokenKind::Do
                | TokenKind::Else
                | TokenKind::Enum
                | TokenKind::Export
                | TokenKind::Extends
                | TokenKind::False
                | TokenKind::Finally
                | TokenKind::For
                | TokenKind::Function
                | TokenKind::If
                | TokenKind::Import
                | TokenKind::In
                | TokenKind::InstanceOf
                | TokenKind::New
                | TokenKind::Null
                | TokenKind::Return
                | TokenKind::Super
                | TokenKind::Switch
                | TokenKind::This
                | TokenKind::Throw
                | TokenKind::True
                | TokenKind::Try
                | TokenKind::TypeOf
                | TokenKind::Var
                | TokenKind::Void
                | TokenKind::While
                | TokenKind::With
                | TokenKind::Implements
                | TokenKind::Interface
                | TokenKind::Let
                | TokenKind::Package
                | TokenKind::Private
                | TokenKind::Protected
                | TokenKind::Public
                | TokenKind::Static
                | TokenKind::Yield
                | TokenKind::Abstract
                | TokenKind::As
                | TokenKind::Asserts
                | TokenKind::Assert
                | TokenKind::Any
                | TokenKind::Async
                | TokenKind::Await
                | TokenKind::Boolean
                | TokenKind::Constructor
                | TokenKind::Declare
                | TokenKind::Get
                | TokenKind::Infer
                | TokenKind::Intrinsic
                | TokenKind::Is
                | TokenKind::KeyOf
                | TokenKind::Module
                | TokenKind::Namespace
                | TokenKind::Never
                | TokenKind::Readonly
                | TokenKind::Require
                | TokenKind::Number
                | TokenKind::Object
                | TokenKind::Set
                | TokenKind::String
                | TokenKind::Symbol
                | TokenKind::Type
                | TokenKind::Undefined
                | TokenKind::Unique
                | TokenKind::UnknownKeyword
                | TokenKind::From
                | TokenKind::Global
                | TokenKind::BigInt
                | TokenKind::Override
                | TokenKind::Of
        )
    }

    /// Returns true if this token is a reserved word (cannot be used as identifier)
    pub fn is_reserved_word(&self) -> bool {
        matches!(
            self,
            TokenKind::Break
                | TokenKind::Case
                | TokenKind::Catch
                | TokenKind::Class
                | TokenKind::Const
                | TokenKind::Continue
                | TokenKind::Debugger
                | TokenKind::Default
                | TokenKind::Delete
                | TokenKind::Do
                | TokenKind::Else
                | TokenKind::Enum
                | TokenKind::Export
                | TokenKind::Extends
                | TokenKind::False
                | TokenKind::Finally
                | TokenKind::For
                | TokenKind::Function
                | TokenKind::If
                | TokenKind::Import
                | TokenKind::In
                | TokenKind::InstanceOf
                | TokenKind::New
                | TokenKind::Null
                | TokenKind::Return
                | TokenKind::Super
                | TokenKind::Switch
                | TokenKind::This
                | TokenKind::Throw
                | TokenKind::True
                | TokenKind::Try
                | TokenKind::TypeOf
                | TokenKind::Var
                | TokenKind::Void
                | TokenKind::While
                | TokenKind::With
        )
    }

    /// Returns true if this token is a strict mode reserved word
    pub fn is_strict_mode_reserved_word(&self) -> bool {
        matches!(
            self,
            TokenKind::Implements
                | TokenKind::Interface
                | TokenKind::Let
                | TokenKind::Package
                | TokenKind::Private
                | TokenKind::Protected
                | TokenKind::Public
                | TokenKind::Static
                | TokenKind::Yield
        )
    }

    /// Returns true if this is an identifier or keyword
    pub fn is_identifier_or_keyword(&self) -> bool {
        *self == TokenKind::Identifier || self.is_keyword()
    }

    /// Returns true if this is trivia (whitespace or comments)
    pub fn is_trivia(&self) -> bool {
        matches!(
            self,
            TokenKind::SingleLineComment
                | TokenKind::MultiLineComment
                | TokenKind::NewLine
                | TokenKind::Whitespace
                | TokenKind::Shebang
                | TokenKind::ConflictMarker
        )
    }

    /// Returns true if this is a literal token
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            TokenKind::NumericLiteral
                | TokenKind::BigIntLiteral
                | TokenKind::StringLiteral
                | TokenKind::RegularExpressionLiteral
                | TokenKind::NoSubstitutionTemplateLiteral
        )
    }

    /// Returns true if this is a template literal part
    pub fn is_template_literal(&self) -> bool {
        matches!(
            self,
            TokenKind::NoSubstitutionTemplateLiteral
                | TokenKind::TemplateHead
                | TokenKind::TemplateMiddle
                | TokenKind::TemplateTail
        )
    }

    /// Returns true if this is an assignment operator
    pub fn is_assignment_operator(&self) -> bool {
        matches!(
            self,
            TokenKind::Equals
                | TokenKind::PlusEquals
                | TokenKind::MinusEquals
                | TokenKind::AsteriskEquals
                | TokenKind::AsteriskAsteriskEquals
                | TokenKind::SlashEquals
                | TokenKind::PercentEquals
                | TokenKind::LessThanLessThanEquals
                | TokenKind::GreaterThanGreaterThanEquals
                | TokenKind::GreaterThanGreaterThanGreaterThanEquals
                | TokenKind::AmpersandEquals
                | TokenKind::BarEquals
                | TokenKind::BarBarEquals
                | TokenKind::AmpersandAmpersandEquals
                | TokenKind::QuestionQuestionEquals
                | TokenKind::CaretEquals
        )
    }
}

/// A token with its kind and span in the source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Get the text of this token from the source
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.span.start..self.span.end]
    }
}

/// Lookup a keyword from its string representation
pub fn keyword_from_str(s: &str) -> Option<TokenKind> {
    match s {
        "abstract" => Some(TokenKind::Abstract),
        "any" => Some(TokenKind::Any),
        "as" => Some(TokenKind::As),
        "asserts" => Some(TokenKind::Asserts),
        "assert" => Some(TokenKind::Assert),
        "bigint" => Some(TokenKind::BigInt),
        "boolean" => Some(TokenKind::Boolean),
        "break" => Some(TokenKind::Break),
        "case" => Some(TokenKind::Case),
        "catch" => Some(TokenKind::Catch),
        "class" => Some(TokenKind::Class),
        "continue" => Some(TokenKind::Continue),
        "const" => Some(TokenKind::Const),
        "constructor" => Some(TokenKind::Constructor),
        "debugger" => Some(TokenKind::Debugger),
        "declare" => Some(TokenKind::Declare),
        "default" => Some(TokenKind::Default),
        "delete" => Some(TokenKind::Delete),
        "do" => Some(TokenKind::Do),
        "else" => Some(TokenKind::Else),
        "enum" => Some(TokenKind::Enum),
        "export" => Some(TokenKind::Export),
        "extends" => Some(TokenKind::Extends),
        "false" => Some(TokenKind::False),
        "finally" => Some(TokenKind::Finally),
        "for" => Some(TokenKind::For),
        "from" => Some(TokenKind::From),
        "function" => Some(TokenKind::Function),
        "get" => Some(TokenKind::Get),
        "if" => Some(TokenKind::If),
        "implements" => Some(TokenKind::Implements),
        "import" => Some(TokenKind::Import),
        "in" => Some(TokenKind::In),
        "infer" => Some(TokenKind::Infer),
        "instanceof" => Some(TokenKind::InstanceOf),
        "interface" => Some(TokenKind::Interface),
        "intrinsic" => Some(TokenKind::Intrinsic),
        "is" => Some(TokenKind::Is),
        "keyof" => Some(TokenKind::KeyOf),
        "let" => Some(TokenKind::Let),
        "module" => Some(TokenKind::Module),
        "namespace" => Some(TokenKind::Namespace),
        "never" => Some(TokenKind::Never),
        "new" => Some(TokenKind::New),
        "null" => Some(TokenKind::Null),
        "number" => Some(TokenKind::Number),
        "object" => Some(TokenKind::Object),
        "package" => Some(TokenKind::Package),
        "private" => Some(TokenKind::Private),
        "protected" => Some(TokenKind::Protected),
        "public" => Some(TokenKind::Public),
        "override" => Some(TokenKind::Override),
        "readonly" => Some(TokenKind::Readonly),
        "require" => Some(TokenKind::Require),
        "global" => Some(TokenKind::Global),
        "return" => Some(TokenKind::Return),
        "set" => Some(TokenKind::Set),
        "static" => Some(TokenKind::Static),
        "string" => Some(TokenKind::String),
        "super" => Some(TokenKind::Super),
        "switch" => Some(TokenKind::Switch),
        "symbol" => Some(TokenKind::Symbol),
        "this" => Some(TokenKind::This),
        "throw" => Some(TokenKind::Throw),
        "true" => Some(TokenKind::True),
        "try" => Some(TokenKind::Try),
        "type" => Some(TokenKind::Type),
        "typeof" => Some(TokenKind::TypeOf),
        "undefined" => Some(TokenKind::Undefined),
        "unique" => Some(TokenKind::Unique),
        "unknown" => Some(TokenKind::UnknownKeyword),
        "var" => Some(TokenKind::Var),
        "void" => Some(TokenKind::Void),
        "while" => Some(TokenKind::While),
        "with" => Some(TokenKind::With),
        "yield" => Some(TokenKind::Yield),
        "async" => Some(TokenKind::Async),
        "await" => Some(TokenKind::Await),
        "of" => Some(TokenKind::Of),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(0, 5);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
        assert_eq!(span.len(), 5);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_span_empty() {
        let span = Span::new(5, 5);
        assert!(span.is_empty());
        assert_eq!(span.len(), 0);
    }

    #[test]
    fn test_token_text() {
        let source = "let x = 42;";
        let token = Token::new(TokenKind::Let, Span::new(0, 3));
        assert_eq!(token.text(source), "let");
    }

    #[test]
    fn test_keyword_classification() {
        assert!(TokenKind::Break.is_keyword());
        assert!(TokenKind::Break.is_reserved_word());
        assert!(!TokenKind::Identifier.is_keyword());

        assert!(TokenKind::Let.is_keyword());
        assert!(!TokenKind::Let.is_reserved_word());
        assert!(TokenKind::Let.is_strict_mode_reserved_word());

        assert!(TokenKind::Abstract.is_keyword());
        assert!(!TokenKind::Abstract.is_reserved_word());
        assert!(!TokenKind::Abstract.is_strict_mode_reserved_word());
    }

    #[test]
    fn test_keyword_from_str() {
        assert_eq!(keyword_from_str("let"), Some(TokenKind::Let));
        assert_eq!(keyword_from_str("const"), Some(TokenKind::Const));
        assert_eq!(keyword_from_str("function"), Some(TokenKind::Function));
        assert_eq!(keyword_from_str("notakeyword"), None);
    }

    #[test]
    fn test_trivia_classification() {
        assert!(TokenKind::Whitespace.is_trivia());
        assert!(TokenKind::SingleLineComment.is_trivia());
        assert!(TokenKind::MultiLineComment.is_trivia());
        assert!(TokenKind::NewLine.is_trivia());
        assert!(!TokenKind::Identifier.is_trivia());
    }

    #[test]
    fn test_literal_classification() {
        assert!(TokenKind::NumericLiteral.is_literal());
        assert!(TokenKind::StringLiteral.is_literal());
        assert!(TokenKind::RegularExpressionLiteral.is_literal());
        assert!(!TokenKind::Identifier.is_literal());
    }

    #[test]
    fn test_assignment_operator_classification() {
        assert!(TokenKind::Equals.is_assignment_operator());
        assert!(TokenKind::PlusEquals.is_assignment_operator());
        assert!(TokenKind::AmpersandAmpersandEquals.is_assignment_operator());
        assert!(!TokenKind::Plus.is_assignment_operator());
        assert!(!TokenKind::EqualsEquals.is_assignment_operator());
    }

    #[test]
    fn test_template_literal_classification() {
        assert!(TokenKind::TemplateHead.is_template_literal());
        assert!(TokenKind::TemplateMiddle.is_template_literal());
        assert!(TokenKind::TemplateTail.is_template_literal());
        assert!(TokenKind::NoSubstitutionTemplateLiteral.is_template_literal());
        assert!(!TokenKind::StringLiteral.is_template_literal());
    }
}
