//! AST Node definitions with NodeKind enum and node flags
//!
//! All TypeScript AST node kinds in a compact u16 enum.

#![allow(missing_docs)]

use super::arena::{NodeId, Span, StringId, ChildList};

/// All TypeScript AST node kinds - compact u16 representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u16)]
pub enum NodeKind {
    // === Sentinel ===
    #[default]
    Unknown = 0,
    EndOfFile = 1,

    // === Trivia ===
    SingleLineComment = 2,
    MultiLineComment = 3,
    NewLine = 4,
    Whitespace = 5,

    // === Literals ===
    NumericLiteral = 10,
    BigIntLiteral = 11,
    StringLiteral = 12,
    RegexLiteral = 13,
    NoSubstitutionTemplate = 14,
    TemplateHead = 15,
    TemplateMiddle = 16,
    TemplateTail = 17,
    JsxText = 18,

    // === Punctuation ===
    OpenBrace = 20,
    CloseBrace = 21,
    OpenParen = 22,
    CloseParen = 23,
    OpenBracket = 24,
    CloseBracket = 25,
    Dot = 26,
    DotDotDot = 27,
    Semicolon = 28,
    Comma = 29,
    QuestionDot = 30,
    LessThan = 31,
    GreaterThan = 32,
    LessThanEquals = 33,
    GreaterThanEquals = 34,
    EqualsEquals = 35,
    ExclamationEquals = 36,
    EqualsEqualsEquals = 37,
    ExclamationEqualsEquals = 38,
    EqualsGreaterThan = 39,
    Plus = 40,
    Minus = 41,
    Asterisk = 42,
    AsteriskAsterisk = 43,
    Slash = 44,
    Percent = 45,
    PlusPlus = 46,
    MinusMinus = 47,
    LessThanLessThan = 48,
    GreaterThanGreaterThan = 49,
    GreaterThanGreaterThanGreaterThan = 50,
    Ampersand = 51,
    Bar = 52,
    Caret = 53,
    Exclamation = 54,
    Tilde = 55,
    AmpersandAmpersand = 56,
    BarBar = 57,
    Question = 58,
    Colon = 59,
    At = 60,
    QuestionQuestion = 61,
    BackTick = 62,
    Hash = 63,
    Equals = 64,
    PlusEquals = 65,
    MinusEquals = 66,
    AsteriskEquals = 67,
    AsteriskAsteriskEquals = 68,
    SlashEquals = 69,
    PercentEquals = 70,
    LessThanLessThanEquals = 71,
    GreaterThanGreaterThanEquals = 72,
    GreaterThanGreaterThanGreaterThanEquals = 73,
    AmpersandEquals = 74,
    BarEquals = 75,
    BarBarEquals = 76,
    AmpersandAmpersandEquals = 77,
    QuestionQuestionEquals = 78,
    CaretEquals = 79,

    // === Identifiers & Keywords ===
    Identifier = 80,
    PrivateIdentifier = 81,

    // Reserved Keywords
    BreakKeyword = 90,
    CaseKeyword = 91,
    CatchKeyword = 92,
    ClassKeyword = 93,
    ConstKeyword = 94,
    ContinueKeyword = 95,
    DebuggerKeyword = 96,
    DefaultKeyword = 97,
    DeleteKeyword = 98,
    DoKeyword = 99,
    ElseKeyword = 100,
    EnumKeyword = 101,
    ExportKeyword = 102,
    ExtendsKeyword = 103,
    FalseKeyword = 104,
    FinallyKeyword = 105,
    ForKeyword = 106,
    FunctionKeyword = 107,
    IfKeyword = 108,
    ImportKeyword = 109,
    InKeyword = 110,
    InstanceofKeyword = 111,
    NewKeyword = 112,
    NullKeyword = 113,
    ReturnKeyword = 114,
    SuperKeyword = 115,
    SwitchKeyword = 116,
    ThisKeyword = 117,
    ThrowKeyword = 118,
    TrueKeyword = 119,
    TryKeyword = 120,
    TypeofKeyword = 121,
    VarKeyword = 122,
    VoidKeyword = 123,
    WhileKeyword = 124,
    WithKeyword = 125,

    // Strict mode reserved
    ImplementsKeyword = 130,
    InterfaceKeyword = 131,
    LetKeyword = 132,
    PackageKeyword = 133,
    PrivateKeyword = 134,
    ProtectedKeyword = 135,
    PublicKeyword = 136,
    StaticKeyword = 137,
    YieldKeyword = 138,

    // Contextual keywords
    AbstractKeyword = 140,
    AsKeyword = 141,
    AssertsKeyword = 142,
    AssertKeyword = 143,
    AnyKeyword = 144,
    AsyncKeyword = 145,
    AwaitKeyword = 146,
    BooleanKeyword = 147,
    ConstructorKeyword = 148,
    DeclareKeyword = 149,
    GetKeyword = 150,
    InferKeyword = 151,
    IntrinsicKeyword = 152,
    IsKeyword = 153,
    KeyOfKeyword = 154,
    ModuleKeyword = 155,
    NamespaceKeyword = 156,
    NeverKeyword = 157,
    OutKeyword = 158,
    OverrideKeyword = 159,
    ReadonlyKeyword = 160,
    RequireKeyword = 161,
    NumberKeyword = 162,
    ObjectKeyword = 163,
    SetKeyword = 164,
    StringKeyword = 165,
    SymbolKeyword = 166,
    TypeKeyword = 167,
    UndefinedKeyword = 168,
    UniqueKeyword = 169,
    UnknownKeyword = 170,
    FromKeyword = 171,
    GlobalKeyword = 172,
    BigIntKeyword = 173,
    OfKeyword = 174,
    SatisfiesKeyword = 175,

    // === Names ===
    QualifiedName = 180,
    ComputedPropertyName = 181,

    // === Signature Elements ===
    TypeParameter = 185,
    Parameter = 186,
    Decorator = 187,

    // === Type Members ===
    PropertySignature = 190,
    PropertyDeclaration = 191,
    MethodSignature = 192,
    MethodDeclaration = 193,
    ClassStaticBlockDeclaration = 194,
    ConstructorDeclaration = 195,
    GetAccessor = 196,
    SetAccessor = 197,
    CallSignature = 198,
    ConstructSignature = 199,
    IndexSignature = 200,

    // === Type Nodes ===
    TypePredicate = 210,
    TypeReference = 211,
    FunctionType = 212,
    ConstructorType = 213,
    TypeQuery = 214,
    TypeLiteral = 215,
    ArrayType = 216,
    TupleType = 217,
    OptionalType = 218,
    RestType = 219,
    UnionType = 220,
    IntersectionType = 221,
    ConditionalType = 222,
    InferType = 223,
    ParenthesizedType = 224,
    ThisType = 225,
    TypeOperator = 226,
    IndexedAccessType = 227,
    MappedType = 228,
    LiteralType = 229,
    NamedTupleMember = 230,
    TemplateLiteralType = 231,
    TemplateLiteralTypeSpan = 232,
    ImportType = 233,

    // === Binding Patterns ===
    ObjectBindingPattern = 240,
    ArrayBindingPattern = 241,
    BindingElement = 242,

    // === Expressions ===
    ArrayLiteralExpression = 250,
    ObjectLiteralExpression = 251,
    PropertyAccessExpression = 252,
    ElementAccessExpression = 253,
    CallExpression = 254,
    NewExpression = 255,
    TaggedTemplateExpression = 256,
    TypeAssertionExpression = 257,
    ParenthesizedExpression = 258,
    FunctionExpression = 259,
    ArrowFunction = 260,
    DeleteExpression = 261,
    TypeOfExpression = 262,
    VoidExpression = 263,
    AwaitExpression = 264,
    PrefixUnaryExpression = 265,
    PostfixUnaryExpression = 266,
    BinaryExpression = 267,
    ConditionalExpression = 268,
    TemplateExpression = 269,
    YieldExpression = 270,
    SpreadElement = 271,
    ClassExpression = 272,
    OmittedExpression = 273,
    ExpressionWithTypeArguments = 274,
    AsExpression = 275,
    NonNullExpression = 276,
    MetaProperty = 277,
    SyntheticExpression = 278,
    SatisfiesExpression = 279,

    // === Object Literal Members ===
    PropertyAssignment = 285,
    ShorthandPropertyAssignment = 286,
    SpreadAssignment = 287,

    // === Enum Member ===
    EnumMember = 290,

    // === Top Level ===
    SourceFile = 295,
    Bundle = 296,

    // === Statements ===
    Block = 300,
    VariableStatement = 301,
    EmptyStatement = 302,
    ExpressionStatement = 303,
    IfStatement = 304,
    DoStatement = 305,
    WhileStatement = 306,
    ForStatement = 307,
    ForInStatement = 308,
    ForOfStatement = 309,
    ContinueStatement = 310,
    BreakStatement = 311,
    ReturnStatement = 312,
    WithStatement = 313,
    SwitchStatement = 314,
    LabeledStatement = 315,
    ThrowStatement = 316,
    TryStatement = 317,
    DebuggerStatement = 318,

    // === Declarations ===
    VariableDeclaration = 325,
    VariableDeclarationList = 326,
    FunctionDeclaration = 327,
    ClassDeclaration = 328,
    InterfaceDeclaration = 329,
    TypeAliasDeclaration = 330,
    EnumDeclaration = 331,
    ModuleDeclaration = 332,
    ModuleBlock = 333,
    CaseBlock = 334,
    NamespaceExportDeclaration = 335,
    ImportEqualsDeclaration = 336,
    ImportDeclaration = 337,
    ImportClause = 338,
    NamespaceImport = 339,
    NamedImports = 340,
    ImportSpecifier = 341,
    ExportAssignment = 342,
    ExportDeclaration = 343,
    NamedExports = 344,
    NamespaceExport = 345,
    ExportSpecifier = 346,
    MissingDeclaration = 347,
    ExternalModuleReference = 348,

    // === JSX ===
    JsxElement = 360,
    JsxSelfClosingElement = 361,
    JsxOpeningElement = 362,
    JsxClosingElement = 363,
    JsxFragment = 364,
    JsxOpeningFragment = 365,
    JsxClosingFragment = 366,
    JsxAttribute = 367,
    JsxAttributes = 368,
    JsxSpreadAttribute = 369,
    JsxExpression = 370,

    // === Clauses ===
    CaseClause = 380,
    DefaultClause = 381,
    HeritageClause = 382,
    CatchClause = 383,
    AssertClause = 384,
    AssertEntry = 385,
    ImportTypeAssertionContainer = 386,

    // === Misc ===
    TemplateSpan = 390,
    SemicolonClassElement = 391,
    SyntheticReferenceExpression = 392,

    // === JSDoc ===
    JsDocTypeExpression = 400,
    JsDocComment = 401,
    JsDocTag = 402,
    JsDocParameterTag = 403,
    JsDocReturnTag = 404,
    JsDocTypeTag = 405,
    JsDocTemplateTag = 406,

    // === Count sentinel ===
    Count = 500,
}

impl NodeKind {
    #[inline]
    pub const fn is_keyword(self) -> bool {
        let v = self as u16;
        v >= NodeKind::BreakKeyword as u16 && v <= NodeKind::SatisfiesKeyword as u16
    }

    #[inline]
    pub const fn is_literal(self) -> bool {
        let v = self as u16;
        v >= NodeKind::NumericLiteral as u16 && v <= NodeKind::JsxText as u16
    }

    #[inline]
    pub const fn is_statement(self) -> bool {
        let v = self as u16;
        v >= NodeKind::Block as u16 && v <= NodeKind::DebuggerStatement as u16
    }

    #[inline]
    pub const fn is_declaration(self) -> bool {
        let v = self as u16;
        v >= NodeKind::VariableDeclaration as u16 && v <= NodeKind::ExternalModuleReference as u16
    }

    #[inline]
    pub const fn is_expression(self) -> bool {
        let v = self as u16;
        (v >= NodeKind::ArrayLiteralExpression as u16 && v <= NodeKind::SatisfiesExpression as u16)
            || self.is_literal()
            || matches!(
                self,
                NodeKind::Identifier
                    | NodeKind::ThisKeyword
                    | NodeKind::SuperKeyword
                    | NodeKind::TrueKeyword
                    | NodeKind::FalseKeyword
                    | NodeKind::NullKeyword
            )
    }

    #[inline]
    pub const fn is_type_node(self) -> bool {
        let v = self as u16;
        v >= NodeKind::TypePredicate as u16 && v <= NodeKind::ImportType as u16
    }

    #[inline]
    pub const fn is_jsx(self) -> bool {
        let v = self as u16;
        v >= NodeKind::JsxElement as u16 && v <= NodeKind::JsxExpression as u16
    }

    pub const fn name(self) -> &'static str {
        match self {
            NodeKind::Unknown => "Unknown",
            NodeKind::EndOfFile => "EndOfFile",
            NodeKind::Identifier => "Identifier",
            NodeKind::NumericLiteral => "NumericLiteral",
            NodeKind::StringLiteral => "StringLiteral",
            NodeKind::SourceFile => "SourceFile",
            NodeKind::Block => "Block",
            NodeKind::VariableStatement => "VariableStatement",
            NodeKind::VariableDeclaration => "VariableDeclaration",
            NodeKind::VariableDeclarationList => "VariableDeclarationList",
            NodeKind::FunctionDeclaration => "FunctionDeclaration",
            NodeKind::ClassDeclaration => "ClassDeclaration",
            NodeKind::IfStatement => "IfStatement",
            NodeKind::ForStatement => "ForStatement",
            NodeKind::WhileStatement => "WhileStatement",
            NodeKind::ReturnStatement => "ReturnStatement",
            NodeKind::ExpressionStatement => "ExpressionStatement",
            NodeKind::CallExpression => "CallExpression",
            NodeKind::BinaryExpression => "BinaryExpression",
            NodeKind::ArrowFunction => "ArrowFunction",
            NodeKind::TypeReference => "TypeReference",
            NodeKind::UnionType => "UnionType",
            NodeKind::IntersectionType => "IntersectionType",
            _ => "Node",
        }
    }

    #[inline]
    pub const fn from_u16(value: u16) -> Self {
        if value >= NodeKind::Count as u16 {
            return NodeKind::Unknown;
        }
        unsafe { std::mem::transmute(value) }
    }
}

/// Node flags - packed into u16
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct NodeFlags(pub u16);

impl NodeFlags {
    pub const NONE: NodeFlags = NodeFlags(0);
    pub const LET: NodeFlags = NodeFlags(1 << 0);
    pub const CONST: NodeFlags = NodeFlags(1 << 1);
    pub const NESTED_NAMESPACE: NodeFlags = NodeFlags(1 << 2);
    pub const SYNTHESIZED: NodeFlags = NodeFlags(1 << 3);
    pub const NAMESPACE: NodeFlags = NodeFlags(1 << 4);
    pub const OPTIONAL_CHAIN: NodeFlags = NodeFlags(1 << 5);
    pub const EXPORT_CONTEXT: NodeFlags = NodeFlags(1 << 6);
    pub const AMBIENT: NodeFlags = NodeFlags(1 << 7);
    pub const IN_WITH_STATEMENT: NodeFlags = NodeFlags(1 << 8);
    pub const JSON_FILE: NodeFlags = NodeFlags(1 << 9);
    pub const TYPE_CACHED: NodeFlags = NodeFlags(1 << 10);
    pub const DEPRECATED: NodeFlags = NodeFlags(1 << 11);
    pub const HAS_IMPLICIT_RETURN: NodeFlags = NodeFlags(1 << 12);
    pub const HAS_EXPLICIT_RETURN: NodeFlags = NodeFlags(1 << 13);
    pub const GLOBAL_AUGMENTATION: NodeFlags = NodeFlags(1 << 14);
    pub const HAS_ASYNC_FUNCTIONS: NodeFlags = NodeFlags(1 << 15);

    #[inline]
    pub const fn contains(self, other: NodeFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn union(self, other: NodeFlags) -> NodeFlags {
        NodeFlags(self.0 | other.0)
    }
}

impl std::ops::BitOr for NodeFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self { self.union(rhs) }
}

impl std::ops::BitOrAssign for NodeFlags {
    fn bitor_assign(&mut self, rhs: Self) { self.0 |= rhs.0; }
}

/// Modifier flags for declarations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct ModifierFlags(pub u32);

impl ModifierFlags {
    pub const NONE: ModifierFlags = ModifierFlags(0);
    pub const EXPORT: ModifierFlags = ModifierFlags(1 << 0);
    pub const AMBIENT: ModifierFlags = ModifierFlags(1 << 1);
    pub const PUBLIC: ModifierFlags = ModifierFlags(1 << 2);
    pub const PRIVATE: ModifierFlags = ModifierFlags(1 << 3);
    pub const PROTECTED: ModifierFlags = ModifierFlags(1 << 4);
    pub const STATIC: ModifierFlags = ModifierFlags(1 << 5);
    pub const READONLY: ModifierFlags = ModifierFlags(1 << 6);
    pub const ACCESSOR: ModifierFlags = ModifierFlags(1 << 7);
    pub const ABSTRACT: ModifierFlags = ModifierFlags(1 << 8);
    pub const ASYNC: ModifierFlags = ModifierFlags(1 << 9);
    pub const DEFAULT: ModifierFlags = ModifierFlags(1 << 10);
    pub const CONST: ModifierFlags = ModifierFlags(1 << 11);
    pub const DEPRECATED: ModifierFlags = ModifierFlags(1 << 12);
    pub const OVERRIDE: ModifierFlags = ModifierFlags(1 << 13);
    pub const IN: ModifierFlags = ModifierFlags(1 << 14);
    pub const OUT: ModifierFlags = ModifierFlags(1 << 15);

    pub const ACCESSIBILITY: ModifierFlags =
        ModifierFlags(Self::PUBLIC.0 | Self::PRIVATE.0 | Self::PROTECTED.0);

    #[inline]
    pub const fn contains(self, other: ModifierFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn union(self, other: ModifierFlags) -> ModifierFlags {
        ModifierFlags(self.0 | other.0)
    }
}

impl std::ops::BitOr for ModifierFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self { self.union(rhs) }
}

/// ThinNode - Compact 32-byte node representation for cache efficiency
/// This is a view struct for convenient access to arena data
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ThinNode {
    pub kind: NodeKind,       // 2 bytes
    pub flags: NodeFlags,     // 2 bytes
    pub modifiers: ModifierFlags, // 4 bytes
    pub span: Span,           // 8 bytes
    pub parent: NodeId,       // 4 bytes
    pub children: ChildList,  // 8 bytes
    pub string_id: StringId,  // 4 bytes
}

impl ThinNode {
    pub const fn new(kind: NodeKind, span: Span) -> Self {
        ThinNode {
            kind,
            flags: NodeFlags::NONE,
            modifiers: ModifierFlags::NONE,
            span,
            parent: NodeId::NONE,
            children: ChildList::EMPTY,
            string_id: StringId::EMPTY,
        }
    }
}

impl Default for ThinNode {
    fn default() -> Self {
        ThinNode::new(NodeKind::Unknown, Span::new(0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_kind_size() {
        assert_eq!(std::mem::size_of::<NodeKind>(), 2);
    }

    #[test]
    fn test_thin_node_size() {
        assert_eq!(std::mem::size_of::<ThinNode>(), 32);
    }

    #[test]
    fn test_node_flags_size() {
        assert_eq!(std::mem::size_of::<NodeFlags>(), 2);
    }

    #[test]
    fn test_modifier_flags_size() {
        assert_eq!(std::mem::size_of::<ModifierFlags>(), 4);
    }

    #[test]
    fn test_is_statement() {
        assert!(NodeKind::Block.is_statement());
        assert!(NodeKind::IfStatement.is_statement());
        assert!(!NodeKind::Identifier.is_statement());
    }

    #[test]
    fn test_is_expression() {
        assert!(NodeKind::Identifier.is_expression());
        assert!(NodeKind::CallExpression.is_expression());
        assert!(!NodeKind::IfStatement.is_expression());
    }
}
