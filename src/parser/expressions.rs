//! Expression parsing with precedence climbing
//!
//! Handles parsing of all TypeScript expression types:
//! - Literals (numbers, strings, booleans, null, undefined)
//! - Identifiers and keywords (this, super)
//! - Binary expressions with proper precedence
//! - Unary expressions (prefix and postfix)
//! - Call expressions
//! - Member expressions
//! - Arrow functions
//! - Object and array literals
//! - Template literals
//! - Conditional (ternary) expressions
//! - Assignment expressions
//! - Await/yield expressions

use crate::thin_parser::TokenKind;

/// Expression precedence levels (higher = binds tighter)
/// Based on JavaScript operator precedence
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Precedence {
    None = 0,
    Comma = 1,           // ,
    Assignment = 2,      // = += -= etc.
    Conditional = 3,     // ?:
    NullishCoalescing = 4, // ??
    LogicalOr = 5,       // ||
    LogicalAnd = 6,      // &&
    BitwiseOr = 7,       // |
    BitwiseXor = 8,      // ^
    BitwiseAnd = 9,      // &
    Equality = 10,       // == != === !==
    Relational = 11,     // < > <= >= in instanceof
    Shift = 12,          // << >> >>>
    Additive = 13,       // + -
    Multiplicative = 14, // * / %
    Exponentiation = 15, // **
    Unary = 16,          // ! ~ + - typeof void delete
    Postfix = 17,        // ++ --
    Call = 18,           // () []
    Member = 19,         // . ?.
    Primary = 20,        // highest
}

impl Precedence {
    /// Get precedence for a binary operator token
    pub fn of_binary_op(kind: TokenKind) -> Self {
        match kind {
            TokenKind::Comma => Precedence::Comma,
            TokenKind::Equals
            | TokenKind::PlusEquals
            | TokenKind::MinusEquals
            | TokenKind::AsteriskEquals
            | TokenKind::SlashEquals => Precedence::Assignment,
            TokenKind::QuestionQuestion => Precedence::NullishCoalescing,
            TokenKind::BarBar => Precedence::LogicalOr,
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
            | TokenKind::GreaterThanEquals => Precedence::Relational,
            TokenKind::Plus | TokenKind::Minus => Precedence::Additive,
            TokenKind::Asterisk | TokenKind::Slash | TokenKind::Percent => {
                Precedence::Multiplicative
            }
            _ => Precedence::None,
        }
    }

    /// Check if this operator is right-associative
    pub fn is_right_associative(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Equals
                | TokenKind::PlusEquals
                | TokenKind::MinusEquals
                | TokenKind::AsteriskEquals
                | TokenKind::SlashEquals
        )
    }
}

/// Expression kind for quick matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
    // Literals
    NumericLiteral,
    StringLiteral,
    BooleanLiteral,
    NullLiteral,
    RegExpLiteral,
    TemplateLiteral,
    // Primary
    Identifier,
    This,
    Super,
    // Compound
    ArrayExpression,
    ObjectExpression,
    FunctionExpression,
    ClassExpression,
    ArrowFunction,
    // Operations
    BinaryExpression,
    UnaryExpression,
    UpdateExpression,
    ConditionalExpression,
    AssignmentExpression,
    // Access
    MemberExpression,
    CallExpression,
    NewExpression,
    TaggedTemplateExpression,
    // Other
    SequenceExpression,
    ParenthesizedExpression,
    AwaitExpression,
    YieldExpression,
    // TypeScript specific
    TypeAssertion,
    AsExpression,
    NonNullAssertion,
    SatisfiesExpression,
}

/// Check if a token starts an expression
pub fn starts_expression(kind: TokenKind, text: &str) -> bool {
    matches!(
        kind,
        TokenKind::Identifier
            | TokenKind::NumericLiteral
            | TokenKind::StringLiteral
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Null
            | TokenKind::This
            | TokenKind::Super
            | TokenKind::New
            | TokenKind::Function
            | TokenKind::Class
            | TokenKind::OpenParen
            | TokenKind::OpenBracket
            | TokenKind::OpenBrace
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Exclamation
            | TokenKind::Tilde
            | TokenKind::PlusPlus
            | TokenKind::MinusMinus
            | TokenKind::LessThan
    ) || text == "async"
        || text == "await"
        || text == "yield"
        || text == "typeof"
        || text == "void"
        || text == "delete"
}

/// Check if a token is a unary prefix operator
pub fn is_prefix_operator(kind: TokenKind, text: &str) -> bool {
    matches!(
        kind,
        TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Exclamation
            | TokenKind::Tilde
            | TokenKind::PlusPlus
            | TokenKind::MinusMinus
    ) || text == "typeof"
        || text == "void"
        || text == "delete"
        || text == "await"
}

/// Check if a token is a postfix operator
pub fn is_postfix_operator(kind: TokenKind) -> bool {
    matches!(kind, TokenKind::PlusPlus | TokenKind::MinusMinus)
}

/// Check if a token is a binary operator
pub fn is_binary_operator(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Asterisk
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::EqualsEquals
            | TokenKind::ExclamationEquals
            | TokenKind::EqualsEqualsEquals
            | TokenKind::ExclamationEqualsEquals
            | TokenKind::LessThan
            | TokenKind::GreaterThan
            | TokenKind::LessThanEquals
            | TokenKind::GreaterThanEquals
            | TokenKind::Ampersand
            | TokenKind::AmpersandAmpersand
            | TokenKind::Bar
            | TokenKind::BarBar
            | TokenKind::Caret
            | TokenKind::QuestionQuestion
    )
}

/// Check if a token is an assignment operator
pub fn is_assignment_operator(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Equals
            | TokenKind::PlusEquals
            | TokenKind::MinusEquals
            | TokenKind::AsteriskEquals
            | TokenKind::SlashEquals
    )
}

/// Check if expression can be an assignment target
pub fn is_valid_assignment_target(kind: ExpressionKind) -> bool {
    matches!(
        kind,
        ExpressionKind::Identifier
            | ExpressionKind::MemberExpression
            | ExpressionKind::ArrayExpression  // Destructuring
            | ExpressionKind::ObjectExpression // Destructuring
    )
}

/// Object literal member kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectMemberKind {
    Property,
    ShorthandProperty,
    Method,
    Getter,
    Setter,
    SpreadElement,
    ComputedProperty,
}

/// Array element kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayElementKind {
    Expression,
    SpreadElement,
    Hole, // elision
}

/// Template literal part
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplatePart {
    Head,   // `start${
    Middle, // }middle${
    Tail,   // }end`
    NoSubstitution, // `no expressions`
}

/// Determines if we're in a context where '<' starts type arguments vs comparison
pub fn could_be_type_arguments(previous_kind: TokenKind) -> bool {
    matches!(
        previous_kind,
        TokenKind::Identifier | TokenKind::CloseParen | TokenKind::CloseBracket
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precedence_ordering() {
        assert!(Precedence::Member > Precedence::Call);
        assert!(Precedence::Call > Precedence::Unary);
        assert!(Precedence::Unary > Precedence::Multiplicative);
        assert!(Precedence::Multiplicative > Precedence::Additive);
        assert!(Precedence::Additive > Precedence::Relational);
        assert!(Precedence::Relational > Precedence::Equality);
        assert!(Precedence::Equality > Precedence::BitwiseAnd);
        assert!(Precedence::BitwiseAnd > Precedence::BitwiseXor);
        assert!(Precedence::BitwiseXor > Precedence::BitwiseOr);
        assert!(Precedence::BitwiseOr > Precedence::LogicalAnd);
        assert!(Precedence::LogicalAnd > Precedence::LogicalOr);
        assert!(Precedence::LogicalOr > Precedence::Conditional);
        assert!(Precedence::Conditional > Precedence::Assignment);
        assert!(Precedence::Assignment > Precedence::Comma);
    }

    #[test]
    fn test_precedence_of_binary_op() {
        assert_eq!(Precedence::of_binary_op(TokenKind::Plus), Precedence::Additive);
        assert_eq!(Precedence::of_binary_op(TokenKind::Asterisk), Precedence::Multiplicative);
        assert_eq!(Precedence::of_binary_op(TokenKind::EqualsEquals), Precedence::Equality);
        assert_eq!(Precedence::of_binary_op(TokenKind::AmpersandAmpersand), Precedence::LogicalAnd);
        assert_eq!(Precedence::of_binary_op(TokenKind::BarBar), Precedence::LogicalOr);
        assert_eq!(Precedence::of_binary_op(TokenKind::Equals), Precedence::Assignment);
    }

    #[test]
    fn test_right_associative() {
        assert!(Precedence::is_right_associative(TokenKind::Equals));
        assert!(Precedence::is_right_associative(TokenKind::PlusEquals));
        assert!(!Precedence::is_right_associative(TokenKind::Plus));
        assert!(!Precedence::is_right_associative(TokenKind::Asterisk));
    }

    #[test]
    fn test_starts_expression() {
        assert!(starts_expression(TokenKind::Identifier, "foo"));
        assert!(starts_expression(TokenKind::NumericLiteral, "42"));
        assert!(starts_expression(TokenKind::StringLiteral, "\"hello\""));
        assert!(starts_expression(TokenKind::True, "true"));
        assert!(starts_expression(TokenKind::OpenParen, "("));
        assert!(starts_expression(TokenKind::OpenBracket, "["));
        assert!(starts_expression(TokenKind::OpenBrace, "{"));
        assert!(starts_expression(TokenKind::Plus, "+"));
        assert!(starts_expression(TokenKind::Minus, "-"));
        assert!(starts_expression(TokenKind::Identifier, "async"));
        assert!(starts_expression(TokenKind::Identifier, "await"));
    }

    #[test]
    fn test_is_prefix_operator() {
        assert!(is_prefix_operator(TokenKind::Plus, "+"));
        assert!(is_prefix_operator(TokenKind::Minus, "-"));
        assert!(is_prefix_operator(TokenKind::Exclamation, "!"));
        assert!(is_prefix_operator(TokenKind::Tilde, "~"));
        assert!(is_prefix_operator(TokenKind::PlusPlus, "++"));
        assert!(is_prefix_operator(TokenKind::MinusMinus, "--"));
        assert!(is_prefix_operator(TokenKind::Identifier, "typeof"));
        assert!(is_prefix_operator(TokenKind::Identifier, "void"));
        assert!(is_prefix_operator(TokenKind::Identifier, "delete"));
        assert!(!is_prefix_operator(TokenKind::Asterisk, "*"));
    }

    #[test]
    fn test_is_postfix_operator() {
        assert!(is_postfix_operator(TokenKind::PlusPlus));
        assert!(is_postfix_operator(TokenKind::MinusMinus));
        assert!(!is_postfix_operator(TokenKind::Plus));
        assert!(!is_postfix_operator(TokenKind::Exclamation));
    }

    #[test]
    fn test_is_binary_operator() {
        assert!(is_binary_operator(TokenKind::Plus));
        assert!(is_binary_operator(TokenKind::Minus));
        assert!(is_binary_operator(TokenKind::Asterisk));
        assert!(is_binary_operator(TokenKind::Slash));
        assert!(is_binary_operator(TokenKind::EqualsEquals));
        assert!(is_binary_operator(TokenKind::AmpersandAmpersand));
        assert!(is_binary_operator(TokenKind::BarBar));
        assert!(!is_binary_operator(TokenKind::Equals)); // Assignment, not binary
        assert!(!is_binary_operator(TokenKind::OpenParen));
    }

    #[test]
    fn test_is_assignment_operator() {
        assert!(is_assignment_operator(TokenKind::Equals));
        assert!(is_assignment_operator(TokenKind::PlusEquals));
        assert!(is_assignment_operator(TokenKind::MinusEquals));
        assert!(is_assignment_operator(TokenKind::AsteriskEquals));
        assert!(is_assignment_operator(TokenKind::SlashEquals));
        assert!(!is_assignment_operator(TokenKind::Plus));
        assert!(!is_assignment_operator(TokenKind::EqualsEquals));
    }

    #[test]
    fn test_is_valid_assignment_target() {
        assert!(is_valid_assignment_target(ExpressionKind::Identifier));
        assert!(is_valid_assignment_target(ExpressionKind::MemberExpression));
        assert!(is_valid_assignment_target(ExpressionKind::ArrayExpression));
        assert!(is_valid_assignment_target(ExpressionKind::ObjectExpression));
        assert!(!is_valid_assignment_target(ExpressionKind::NumericLiteral));
        assert!(!is_valid_assignment_target(ExpressionKind::BinaryExpression));
        assert!(!is_valid_assignment_target(ExpressionKind::CallExpression));
    }

    #[test]
    fn test_could_be_type_arguments() {
        assert!(could_be_type_arguments(TokenKind::Identifier));
        assert!(could_be_type_arguments(TokenKind::CloseParen));
        assert!(could_be_type_arguments(TokenKind::CloseBracket));
        assert!(!could_be_type_arguments(TokenKind::Plus));
        assert!(!could_be_type_arguments(TokenKind::NumericLiteral));
    }

    #[test]
    fn test_object_member_kind() {
        assert_ne!(ObjectMemberKind::Property, ObjectMemberKind::Method);
        assert_ne!(ObjectMemberKind::Getter, ObjectMemberKind::Setter);
    }

    #[test]
    fn test_array_element_kind() {
        assert_ne!(ArrayElementKind::Expression, ArrayElementKind::SpreadElement);
        assert_ne!(ArrayElementKind::SpreadElement, ArrayElementKind::Hole);
    }

    #[test]
    fn test_template_part() {
        assert_ne!(TemplatePart::Head, TemplatePart::Middle);
        assert_ne!(TemplatePart::Middle, TemplatePart::Tail);
    }
}
