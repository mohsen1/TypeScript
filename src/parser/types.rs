//! Type annotation parsing
//!
//! Handles parsing of TypeScript type annotations including:
//! - Basic types (string, number, boolean, etc.)
//! - Type references with generics
//! - Union and intersection types
//! - Array and tuple types
//! - Function types
//! - Object/literal types
//! - Conditional types
//! - Mapped types
//! - Type predicates

use crate::thin_parser::TokenKind;

/// Type annotation AST node
#[derive(Debug, Clone)]
pub enum TypeNode<'a> {
    /// Keyword type: string, number, boolean, any, unknown, never, void, null, undefined
    Keyword(&'a str),
    /// Type reference: MyType, Array<T>
    Reference {
        name: &'a str,
        type_args: Vec<TypeNode<'a>>,
    },
    /// Array type: T[]
    Array(Box<TypeNode<'a>>),
    /// Tuple type: [T, U, V]
    Tuple(Vec<TypeNode<'a>>),
    /// Union type: T | U | V
    Union(Vec<TypeNode<'a>>),
    /// Intersection type: T & U & V
    Intersection(Vec<TypeNode<'a>>),
    /// Function type: (x: T, y: U) => R
    Function {
        params: Vec<TypeParam<'a>>,
        return_type: Box<TypeNode<'a>>,
    },
    /// Object/literal type: { x: T; y: U }
    Object(Vec<ObjectMember<'a>>),
    /// Indexed access type: T[K]
    IndexedAccess {
        object_type: Box<TypeNode<'a>>,
        index_type: Box<TypeNode<'a>>,
    },
    /// Conditional type: T extends U ? X : Y
    Conditional {
        check_type: Box<TypeNode<'a>>,
        extends_type: Box<TypeNode<'a>>,
        true_type: Box<TypeNode<'a>>,
        false_type: Box<TypeNode<'a>>,
    },
    /// Mapped type: { [K in T]: U }
    Mapped {
        type_param: &'a str,
        constraint: Box<TypeNode<'a>>,
        name_type: Option<Box<TypeNode<'a>>>,
        value_type: Box<TypeNode<'a>>,
        readonly_modifier: Option<Modifier>,
        optional_modifier: Option<Modifier>,
    },
    /// Literal type: "hello", 42, true
    Literal(&'a str),
    /// typeof type
    TypeOf(&'a str),
    /// keyof type
    KeyOf(Box<TypeNode<'a>>),
    /// readonly type
    Readonly(Box<TypeNode<'a>>),
    /// unique symbol
    UniqueSymbol,
    /// infer T
    Infer(&'a str),
    /// this type
    This,
    /// Parenthesized type
    Parenthesized(Box<TypeNode<'a>>),
    /// Rest type in tuple: ...T
    Rest(Box<TypeNode<'a>>),
    /// Optional type in tuple: T?
    Optional(Box<TypeNode<'a>>),
    /// Template literal type: `hello ${string}`
    TemplateLiteral {
        head: &'a str,
        spans: Vec<TemplateLiteralSpan<'a>>,
    },
    /// Type predicate: x is T
    Predicate {
        param_name: &'a str,
        type_node: Box<TypeNode<'a>>,
        asserts: bool,
    },
    /// Import type: import("module").Type
    Import {
        module: &'a str,
        qualifier: Option<&'a str>,
        type_args: Vec<TypeNode<'a>>,
    },
}

/// Modifier for mapped types (+/- readonly, +/- optional)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    Plus,
    Minus,
    None,
}

/// Type parameter for function types
#[derive(Debug, Clone)]
pub struct TypeParam<'a> {
    pub name: &'a str,
    pub type_annotation: Option<Box<TypeNode<'a>>>,
    pub optional: bool,
    pub rest: bool,
}

/// Object type member
#[derive(Debug, Clone)]
pub enum ObjectMember<'a> {
    /// Property: name: Type
    Property {
        name: &'a str,
        type_annotation: TypeNode<'a>,
        optional: bool,
        readonly: bool,
    },
    /// Method: name(params): ReturnType
    Method {
        name: &'a str,
        params: Vec<TypeParam<'a>>,
        return_type: Option<TypeNode<'a>>,
        optional: bool,
    },
    /// Index signature: [key: string]: Type
    IndexSignature {
        param_name: &'a str,
        param_type: TypeNode<'a>,
        value_type: TypeNode<'a>,
        readonly: bool,
    },
    /// Call signature: (params): ReturnType
    CallSignature {
        params: Vec<TypeParam<'a>>,
        return_type: Option<TypeNode<'a>>,
    },
    /// Construct signature: new (params): ReturnType
    ConstructSignature {
        params: Vec<TypeParam<'a>>,
        return_type: Option<TypeNode<'a>>,
    },
}

/// Template literal span
#[derive(Debug, Clone)]
pub struct TemplateLiteralSpan<'a> {
    pub type_node: TypeNode<'a>,
    pub literal: &'a str,
}

/// Type parameter declaration (for generics)
#[derive(Debug, Clone)]
pub struct TypeParameterDecl<'a> {
    pub name: &'a str,
    pub constraint: Option<Box<TypeNode<'a>>>,
    pub default: Option<Box<TypeNode<'a>>>,
}

/// Check if a token is a type keyword
pub fn is_type_keyword(text: &str) -> bool {
    matches!(
        text,
        "string"
            | "number"
            | "boolean"
            | "any"
            | "unknown"
            | "never"
            | "void"
            | "null"
            | "undefined"
            | "object"
            | "symbol"
            | "bigint"
    )
}

/// Check if a token starts a type
pub fn starts_type(kind: TokenKind, text: &str) -> bool {
    matches!(
        kind,
        TokenKind::Identifier
            | TokenKind::OpenParen
            | TokenKind::OpenBrace
            | TokenKind::OpenBracket
            | TokenKind::LessThan
            | TokenKind::StringLiteral
            | TokenKind::NumericLiteral
            | TokenKind::True
            | TokenKind::False
            | TokenKind::This
            | TokenKind::Null
            | TokenKind::Undefined
    ) || text == "typeof"
        || text == "keyof"
        || text == "readonly"
        || text == "unique"
        || text == "infer"
        || text == "new"
        || text == "import"
        || is_type_keyword(text)
}

/// Get precedence for type operators (for parsing union/intersection)
pub fn type_operator_precedence(kind: TokenKind) -> u8 {
    match kind {
        TokenKind::Bar => 1,             // Union: lowest
        TokenKind::Ampersand => 2,       // Intersection
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_type_keyword() {
        assert!(is_type_keyword("string"));
        assert!(is_type_keyword("number"));
        assert!(is_type_keyword("boolean"));
        assert!(is_type_keyword("any"));
        assert!(is_type_keyword("unknown"));
        assert!(is_type_keyword("never"));
        assert!(is_type_keyword("void"));
        assert!(!is_type_keyword("MyType"));
        assert!(!is_type_keyword("let"));
    }

    #[test]
    fn test_starts_type() {
        assert!(starts_type(TokenKind::Identifier, "string"));
        assert!(starts_type(TokenKind::Identifier, "MyType"));
        assert!(starts_type(TokenKind::OpenParen, "("));
        assert!(starts_type(TokenKind::OpenBrace, "{"));
        assert!(starts_type(TokenKind::OpenBracket, "["));
        assert!(starts_type(TokenKind::Identifier, "typeof"));
        assert!(starts_type(TokenKind::Identifier, "keyof"));
    }

    #[test]
    fn test_type_operator_precedence() {
        assert!(type_operator_precedence(TokenKind::Ampersand) > type_operator_precedence(TokenKind::Bar));
        assert_eq!(type_operator_precedence(TokenKind::Plus), 0);
    }

    #[test]
    fn test_type_node_keyword() {
        let node = TypeNode::Keyword("string");
        match node {
            TypeNode::Keyword(kw) => assert_eq!(kw, "string"),
            _ => panic!("Expected Keyword"),
        }
    }

    #[test]
    fn test_type_node_reference() {
        let node = TypeNode::Reference {
            name: "Array",
            type_args: vec![TypeNode::Keyword("number")],
        };
        match node {
            TypeNode::Reference { name, type_args } => {
                assert_eq!(name, "Array");
                assert_eq!(type_args.len(), 1);
            }
            _ => panic!("Expected Reference"),
        }
    }

    #[test]
    fn test_type_node_union() {
        let node = TypeNode::Union(vec![
            TypeNode::Keyword("string"),
            TypeNode::Keyword("number"),
        ]);
        match node {
            TypeNode::Union(types) => {
                assert_eq!(types.len(), 2);
            }
            _ => panic!("Expected Union"),
        }
    }

    #[test]
    fn test_type_node_array() {
        let node = TypeNode::Array(Box::new(TypeNode::Keyword("string")));
        match node {
            TypeNode::Array(inner) => {
                match *inner {
                    TypeNode::Keyword(kw) => assert_eq!(kw, "string"),
                    _ => panic!("Expected Keyword"),
                }
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_type_param() {
        let param = TypeParam {
            name: "x",
            type_annotation: Some(Box::new(TypeNode::Keyword("number"))),
            optional: false,
            rest: false,
        };
        assert_eq!(param.name, "x");
        assert!(!param.optional);
    }

    #[test]
    fn test_object_member_property() {
        let member = ObjectMember::Property {
            name: "foo",
            type_annotation: TypeNode::Keyword("string"),
            optional: true,
            readonly: false,
        };
        match member {
            ObjectMember::Property { name, optional, .. } => {
                assert_eq!(name, "foo");
                assert!(optional);
            }
            _ => panic!("Expected Property"),
        }
    }

    #[test]
    fn test_modifier() {
        assert_eq!(Modifier::Plus, Modifier::Plus);
        assert_ne!(Modifier::Plus, Modifier::Minus);
    }
}
