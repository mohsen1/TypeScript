//! NodeKind enum representing all TypeScript AST node types.
//! Uses u16 for compact storage.

#![allow(missing_docs)]

/// All TypeScript AST node kinds.
/// Stored as u16 for cache-efficient storage in ThinNode arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u16)]
pub enum NodeKind {
    // === Sentinel / Unknown ===
    #[default]
    Unknown = 0,
    EndOfFile = 1,

    // === Trivia ===
    SingleLineComment = 2,
    MultiLineComment = 3,
    NewLine = 4,
    Whitespace = 5,
    ConflictMarker = 6,

    // === Literals ===
    NumericLiteral = 7,
    BigIntLiteral = 8,
    StringLiteral = 9,
    JsxText = 10,
    JsxTextAllWhiteSpaces = 11,
    RegularExpressionLiteral = 12,
    NoSubstitutionTemplateLiteral = 13,

    // === Pseudo-Literals ===
    TemplateHead = 14,
    TemplateMiddle = 15,
    TemplateTail = 16,

    // === Punctuation ===
    OpenBrace = 17,
    CloseBrace = 18,
    OpenParen = 19,
    CloseParen = 20,
    OpenBracket = 21,
    CloseBracket = 22,
    Dot = 23,
    DotDotDot = 24,
    Semicolon = 25,
    Comma = 26,
    QuestionDot = 27,
    LessThan = 28,
    LessThanSlash = 29,
    GreaterThan = 30,
    LessThanEquals = 31,
    GreaterThanEquals = 32,
    EqualsEquals = 33,
    ExclamationEquals = 34,
    EqualsEqualsEquals = 35,
    ExclamationEqualsEquals = 36,
    EqualsGreaterThan = 37,
    Plus = 38,
    Minus = 39,
    Asterisk = 40,
    AsteriskAsterisk = 41,
    Slash = 42,
    Percent = 43,
    PlusPlus = 44,
    MinusMinus = 45,
    LessThanLessThan = 46,
    GreaterThanGreaterThan = 47,
    GreaterThanGreaterThanGreaterThan = 48,
    Ampersand = 49,
    Bar = 50,
    Caret = 51,
    Exclamation = 52,
    Tilde = 53,
    AmpersandAmpersand = 54,
    BarBar = 55,
    Question = 56,
    Colon = 57,
    At = 58,
    QuestionQuestion = 59,
    BackTick = 60,
    Hash = 61,

    // === Assignments ===
    Equals = 62,
    PlusEquals = 63,
    MinusEquals = 64,
    AsteriskEquals = 65,
    AsteriskAsteriskEquals = 66,
    SlashEquals = 67,
    PercentEquals = 68,
    LessThanLessThanEquals = 69,
    GreaterThanGreaterThanEquals = 70,
    GreaterThanGreaterThanGreaterThanEquals = 71,
    AmpersandEquals = 72,
    BarEquals = 73,
    BarBarEquals = 74,
    AmpersandAmpersandEquals = 75,
    QuestionQuestionEquals = 76,
    CaretEquals = 77,

    // === Identifiers and Keywords ===
    Identifier = 78,

    // Reserved Keywords
    Break = 79,
    Case = 80,
    Catch = 81,
    Class = 82,
    Const = 83,
    Continue = 84,
    Debugger = 85,
    Default = 86,
    Delete = 87,
    Do = 88,
    Else = 89,
    Enum = 90,
    Export = 91,
    Extends = 92,
    False = 93,
    Finally = 94,
    For = 95,
    Function = 96,
    If = 97,
    Import = 98,
    In = 99,
    Instanceof = 100,
    New = 101,
    Null = 102,
    Return = 103,
    Super = 104,
    Switch = 105,
    This = 106,
    Throw = 107,
    True = 108,
    Try = 109,
    Typeof = 110,
    Var = 111,
    Void = 112,
    While = 113,
    With = 114,

    // Strict Mode Reserved
    Implements = 115,
    Interface = 116,
    Let = 117,
    Package = 118,
    Private = 119,
    Protected = 120,
    Public = 121,
    Static = 122,
    Yield = 123,

    // Contextual Keywords
    Abstract = 124,
    As = 125,
    Asserts = 126,
    Assert = 127,
    Any = 128,
    Async = 129,
    Await = 130,
    Boolean = 131,
    Constructor = 132,
    Declare = 133,
    Get = 134,
    Infer = 135,
    Intrinsic = 136,
    Is = 137,
    KeyOf = 138,
    Module = 139,
    Namespace = 140,
    Never = 141,
    Out = 142,
    Override = 143,
    Readonly = 144,
    Require = 145,
    Number = 146,
    Object = 147,
    Set = 148,
    String = 149,
    Symbol = 150,
    Type = 151,
    Undefined = 152,
    Unique = 153,
    UnknownKeyword = 154,
    From = 155,
    Global = 156,
    BigInt = 157,
    Of = 158,

    // === Names ===
    QualifiedName = 159,
    ComputedPropertyName = 160,

    // === Signature Elements ===
    TypeParameter = 161,
    Parameter = 162,
    Decorator = 163,

    // === Type Members ===
    PropertySignature = 164,
    PropertyDeclaration = 165,
    MethodSignature = 166,
    MethodDeclaration = 167,
    ClassStaticBlockDeclaration = 168,
    Constructor_ = 169,
    GetAccessor = 170,
    SetAccessor = 171,
    CallSignature = 172,
    ConstructSignature = 173,
    IndexSignature = 174,

    // === Type Nodes ===
    TypePredicate = 175,
    TypeReference = 176,
    FunctionType = 177,
    ConstructorType = 178,
    TypeQuery = 179,
    TypeLiteral = 180,
    ArrayType = 181,
    TupleType = 182,
    OptionalType = 183,
    RestType = 184,
    UnionType = 185,
    IntersectionType = 186,
    ConditionalType = 187,
    InferType = 188,
    ParenthesizedType = 189,
    ThisType = 190,
    TypeOperator = 191,
    IndexedAccessType = 192,
    MappedType = 193,
    LiteralType = 194,
    NamedTupleMember = 195,
    TemplateLiteralType = 196,
    TemplateLiteralTypeSpan = 197,
    ImportType = 198,

    // === Binding Patterns ===
    ObjectBindingPattern = 199,
    ArrayBindingPattern = 200,
    BindingElement = 201,

    // === Expressions ===
    ArrayLiteralExpression = 202,
    ObjectLiteralExpression = 203,
    PropertyAccessExpression = 204,
    ElementAccessExpression = 205,
    CallExpression = 206,
    NewExpression = 207,
    TaggedTemplateExpression = 208,
    TypeAssertionExpression = 209,
    ParenthesizedExpression = 210,
    FunctionExpression = 211,
    ArrowFunction = 212,
    DeleteExpression = 213,
    TypeOfExpression = 214,
    VoidExpression = 215,
    AwaitExpression = 216,
    PrefixUnaryExpression = 217,
    PostfixUnaryExpression = 218,
    BinaryExpression = 219,
    ConditionalExpression = 220,
    TemplateExpression = 221,
    YieldExpression = 222,
    SpreadElement = 223,
    ClassExpression = 224,
    OmittedExpression = 225,
    ExpressionWithTypeArguments = 226,
    AsExpression = 227,
    NonNullExpression = 228,
    MetaProperty = 229,
    SyntheticExpression = 230,

    // === Object Literal Members ===
    PropertyAssignment = 231,
    ShorthandPropertyAssignment = 232,
    SpreadAssignment = 233,

    // === Enum Member ===
    EnumMember = 234,

    // === Unparsed / Synthetic ===
    Unparsed = 235,
    UnparsedPrologue = 236,
    UnparsedPrepend = 237,
    UnparsedText = 238,
    UnparsedInternalText = 239,
    UnparsedSyntheticReference = 240,

    // === Top Level ===
    SourceFile = 241,
    Bundle = 242,

    // === JSDoc Types ===
    JsDocTypeExpression = 243,
    JsDocNameReference = 244,
    JsDocMemberName = 245,
    JsDocAllType = 246,
    JsDocUnknownType = 247,
    JsDocNullableType = 248,
    JsDocNonNullableType = 249,
    JsDocOptionalType = 250,
    JsDocFunctionType = 251,
    JsDocVariadicType = 252,
    JsDocNamepathType = 253,
    JsDoc = 254,
    JsDocText = 255,
    JsDocTypeLiteral = 256,
    JsDocSignature = 257,
    JsDocLink = 258,
    JsDocLinkCode = 259,
    JsDocLinkPlain = 260,

    // === JSDoc Tags ===
    JsDocTag = 261,
    JsDocAugmentsTag = 262,
    JsDocImplementsTag = 263,
    JsDocAuthorTag = 264,
    JsDocDeprecatedTag = 265,
    JsDocClassTag = 266,
    JsDocPublicTag = 267,
    JsDocPrivateTag = 268,
    JsDocProtectedTag = 269,
    JsDocReadonlyTag = 270,
    JsDocOverrideTag = 271,
    JsDocCallbackTag = 272,
    JsDocEnumTag = 273,
    JsDocParameterTag = 274,
    JsDocReturnTag = 275,
    JsDocThisTag = 276,
    JsDocTypeTag = 277,
    JsDocTemplateTag = 278,
    JsDocTypedefTag = 279,
    JsDocSeeTag = 280,
    JsDocPropertyTag = 281,

    // === Statements ===
    Block = 282,
    VariableStatement = 283,
    EmptyStatement = 284,
    ExpressionStatement = 285,
    IfStatement = 286,
    DoStatement = 287,
    WhileStatement = 288,
    ForStatement = 289,
    ForInStatement = 290,
    ForOfStatement = 291,
    ContinueStatement = 292,
    BreakStatement = 293,
    ReturnStatement = 294,
    WithStatement = 295,
    SwitchStatement = 296,
    LabeledStatement = 297,
    ThrowStatement = 298,
    TryStatement = 299,
    DebuggerStatement = 300,

    // === Declarations ===
    VariableDeclaration = 301,
    VariableDeclarationList = 302,
    FunctionDeclaration = 303,
    ClassDeclaration = 304,
    InterfaceDeclaration = 305,
    TypeAliasDeclaration = 306,
    EnumDeclaration = 307,
    ModuleDeclaration = 308,
    ModuleBlock = 309,
    CaseBlock = 310,
    NamespaceExportDeclaration = 311,
    ImportEqualsDeclaration = 312,
    ImportDeclaration = 313,
    ImportClause = 314,
    NamespaceImport = 315,
    NamedImports = 316,
    ImportSpecifier = 317,
    ExportAssignment = 318,
    ExportDeclaration = 319,
    NamedExports = 320,
    NamespaceExport = 321,
    ExportSpecifier = 322,
    MissingDeclaration = 323,

    // === Module References ===
    ExternalModuleReference = 324,

    // === JSX ===
    JsxElement = 325,
    JsxSelfClosingElement = 326,
    JsxOpeningElement = 327,
    JsxClosingElement = 328,
    JsxFragment = 329,
    JsxOpeningFragment = 330,
    JsxClosingFragment = 331,
    JsxAttribute = 332,
    JsxAttributes = 333,
    JsxSpreadAttribute = 334,
    JsxExpression = 335,

    // === Clauses ===
    CaseClause = 336,
    DefaultClause = 337,
    HeritageClause = 338,
    CatchClause = 339,

    // === Assert ===
    AssertClause = 340,
    AssertEntry = 341,

    // === Property Assignment ===
    PropertyAccessChain = 342,
    ElementAccessChain = 343,
    CallChain = 344,
    OptionalChain = 345,

    // === Special ===
    TemplateSpan = 346,
    SemicolonClassElement = 347,

    // === Synthesized ===
    SyntheticReferenceExpression = 348,

    // === Private Identifier ===
    PrivateIdentifier = 349,

    // === Sentinel ===
    Count = 350,
}

impl NodeKind {
    /// Returns true if this kind represents a literal token.
    #[inline]
    pub const fn is_literal(self) -> bool {
        matches!(
            self,
            NodeKind::NumericLiteral
                | NodeKind::BigIntLiteral
                | NodeKind::StringLiteral
                | NodeKind::RegularExpressionLiteral
                | NodeKind::NoSubstitutionTemplateLiteral
        )
    }

    /// Returns true if this kind represents a keyword.
    #[inline]
    pub const fn is_keyword(self) -> bool {
        let v = self as u16;
        v >= NodeKind::Break as u16 && v <= NodeKind::Of as u16
    }

    /// Returns true if this kind represents a reserved keyword.
    #[inline]
    pub const fn is_reserved_keyword(self) -> bool {
        let v = self as u16;
        v >= NodeKind::Break as u16 && v <= NodeKind::With as u16
    }

    /// Returns true if this kind represents a contextual keyword.
    #[inline]
    pub const fn is_contextual_keyword(self) -> bool {
        let v = self as u16;
        v >= NodeKind::Abstract as u16 && v <= NodeKind::Of as u16
    }

    /// Returns true if this kind represents an assignment operator.
    #[inline]
    pub const fn is_assignment_operator(self) -> bool {
        let v = self as u16;
        v >= NodeKind::Equals as u16 && v <= NodeKind::CaretEquals as u16
    }

    /// Returns true if this kind represents a binary operator.
    #[inline]
    pub const fn is_binary_operator(self) -> bool {
        matches!(
            self,
            NodeKind::LessThan
                | NodeKind::GreaterThan
                | NodeKind::LessThanEquals
                | NodeKind::GreaterThanEquals
                | NodeKind::EqualsEquals
                | NodeKind::ExclamationEquals
                | NodeKind::EqualsEqualsEquals
                | NodeKind::ExclamationEqualsEquals
                | NodeKind::Plus
                | NodeKind::Minus
                | NodeKind::Asterisk
                | NodeKind::AsteriskAsterisk
                | NodeKind::Slash
                | NodeKind::Percent
                | NodeKind::LessThanLessThan
                | NodeKind::GreaterThanGreaterThan
                | NodeKind::GreaterThanGreaterThanGreaterThan
                | NodeKind::Ampersand
                | NodeKind::Bar
                | NodeKind::Caret
                | NodeKind::AmpersandAmpersand
                | NodeKind::BarBar
                | NodeKind::QuestionQuestion
                | NodeKind::In
                | NodeKind::Instanceof
        ) || self.is_assignment_operator()
    }

    /// Returns true if this kind represents a unary operator.
    #[inline]
    pub const fn is_unary_operator(self) -> bool {
        matches!(
            self,
            NodeKind::Plus
                | NodeKind::Minus
                | NodeKind::PlusPlus
                | NodeKind::MinusMinus
                | NodeKind::Exclamation
                | NodeKind::Tilde
                | NodeKind::Delete
                | NodeKind::Typeof
                | NodeKind::Void
                | NodeKind::Await
        )
    }

    /// Returns true if this kind represents a statement.
    #[inline]
    pub const fn is_statement(self) -> bool {
        let v = self as u16;
        v >= NodeKind::Block as u16 && v <= NodeKind::DebuggerStatement as u16
    }

    /// Returns true if this kind represents a declaration.
    #[inline]
    pub const fn is_declaration(self) -> bool {
        let v = self as u16;
        v >= NodeKind::VariableDeclaration as u16 && v <= NodeKind::MissingDeclaration as u16
    }

    /// Returns true if this kind represents a type node.
    #[inline]
    pub const fn is_type_node(self) -> bool {
        let v = self as u16;
        v >= NodeKind::TypePredicate as u16 && v <= NodeKind::ImportType as u16
    }

    /// Returns true if this kind represents a JSX element.
    #[inline]
    pub const fn is_jsx(self) -> bool {
        let v = self as u16;
        v >= NodeKind::JsxElement as u16 && v <= NodeKind::JsxExpression as u16
    }

    /// Returns true if this kind represents a JSDoc node.
    #[inline]
    pub const fn is_jsdoc(self) -> bool {
        let v = self as u16;
        v >= NodeKind::JsDocTypeExpression as u16 && v <= NodeKind::JsDocPropertyTag as u16
    }

    /// Returns true if this kind represents an expression.
    #[inline]
    pub const fn is_expression(self) -> bool {
        let v = self as u16;
        (v >= NodeKind::ArrayLiteralExpression as u16
            && v <= NodeKind::SyntheticExpression as u16)
            || self.is_literal()
            || matches!(
                self,
                NodeKind::Identifier
                    | NodeKind::This
                    | NodeKind::Super
                    | NodeKind::True
                    | NodeKind::False
                    | NodeKind::Null
            )
    }

    /// Returns the name of this node kind as a static string.
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            NodeKind::Unknown => "Unknown",
            NodeKind::EndOfFile => "EndOfFile",
            NodeKind::SingleLineComment => "SingleLineComment",
            NodeKind::MultiLineComment => "MultiLineComment",
            NodeKind::NewLine => "NewLine",
            NodeKind::Whitespace => "Whitespace",
            NodeKind::ConflictMarker => "ConflictMarker",
            NodeKind::NumericLiteral => "NumericLiteral",
            NodeKind::BigIntLiteral => "BigIntLiteral",
            NodeKind::StringLiteral => "StringLiteral",
            NodeKind::JsxText => "JsxText",
            NodeKind::JsxTextAllWhiteSpaces => "JsxTextAllWhiteSpaces",
            NodeKind::RegularExpressionLiteral => "RegularExpressionLiteral",
            NodeKind::NoSubstitutionTemplateLiteral => "NoSubstitutionTemplateLiteral",
            NodeKind::TemplateHead => "TemplateHead",
            NodeKind::TemplateMiddle => "TemplateMiddle",
            NodeKind::TemplateTail => "TemplateTail",
            NodeKind::OpenBrace => "OpenBrace",
            NodeKind::CloseBrace => "CloseBrace",
            NodeKind::OpenParen => "OpenParen",
            NodeKind::CloseParen => "CloseParen",
            NodeKind::OpenBracket => "OpenBracket",
            NodeKind::CloseBracket => "CloseBracket",
            NodeKind::Dot => "Dot",
            NodeKind::DotDotDot => "DotDotDot",
            NodeKind::Semicolon => "Semicolon",
            NodeKind::Comma => "Comma",
            NodeKind::QuestionDot => "QuestionDot",
            NodeKind::LessThan => "LessThan",
            NodeKind::LessThanSlash => "LessThanSlash",
            NodeKind::GreaterThan => "GreaterThan",
            NodeKind::LessThanEquals => "LessThanEquals",
            NodeKind::GreaterThanEquals => "GreaterThanEquals",
            NodeKind::EqualsEquals => "EqualsEquals",
            NodeKind::ExclamationEquals => "ExclamationEquals",
            NodeKind::EqualsEqualsEquals => "EqualsEqualsEquals",
            NodeKind::ExclamationEqualsEquals => "ExclamationEqualsEquals",
            NodeKind::EqualsGreaterThan => "EqualsGreaterThan",
            NodeKind::Plus => "Plus",
            NodeKind::Minus => "Minus",
            NodeKind::Asterisk => "Asterisk",
            NodeKind::AsteriskAsterisk => "AsteriskAsterisk",
            NodeKind::Slash => "Slash",
            NodeKind::Percent => "Percent",
            NodeKind::PlusPlus => "PlusPlus",
            NodeKind::MinusMinus => "MinusMinus",
            NodeKind::LessThanLessThan => "LessThanLessThan",
            NodeKind::GreaterThanGreaterThan => "GreaterThanGreaterThan",
            NodeKind::GreaterThanGreaterThanGreaterThan => "GreaterThanGreaterThanGreaterThan",
            NodeKind::Ampersand => "Ampersand",
            NodeKind::Bar => "Bar",
            NodeKind::Caret => "Caret",
            NodeKind::Exclamation => "Exclamation",
            NodeKind::Tilde => "Tilde",
            NodeKind::AmpersandAmpersand => "AmpersandAmpersand",
            NodeKind::BarBar => "BarBar",
            NodeKind::Question => "Question",
            NodeKind::Colon => "Colon",
            NodeKind::At => "At",
            NodeKind::QuestionQuestion => "QuestionQuestion",
            NodeKind::BackTick => "BackTick",
            NodeKind::Hash => "Hash",
            NodeKind::Equals => "Equals",
            NodeKind::PlusEquals => "PlusEquals",
            NodeKind::MinusEquals => "MinusEquals",
            NodeKind::AsteriskEquals => "AsteriskEquals",
            NodeKind::AsteriskAsteriskEquals => "AsteriskAsteriskEquals",
            NodeKind::SlashEquals => "SlashEquals",
            NodeKind::PercentEquals => "PercentEquals",
            NodeKind::LessThanLessThanEquals => "LessThanLessThanEquals",
            NodeKind::GreaterThanGreaterThanEquals => "GreaterThanGreaterThanEquals",
            NodeKind::GreaterThanGreaterThanGreaterThanEquals => "GreaterThanGreaterThanGreaterThanEquals",
            NodeKind::AmpersandEquals => "AmpersandEquals",
            NodeKind::BarEquals => "BarEquals",
            NodeKind::BarBarEquals => "BarBarEquals",
            NodeKind::AmpersandAmpersandEquals => "AmpersandAmpersandEquals",
            NodeKind::QuestionQuestionEquals => "QuestionQuestionEquals",
            NodeKind::CaretEquals => "CaretEquals",
            NodeKind::Identifier => "Identifier",
            NodeKind::Break => "Break",
            NodeKind::Case => "Case",
            NodeKind::Catch => "Catch",
            NodeKind::Class => "Class",
            NodeKind::Const => "Const",
            NodeKind::Continue => "Continue",
            NodeKind::Debugger => "Debugger",
            NodeKind::Default => "Default",
            NodeKind::Delete => "Delete",
            NodeKind::Do => "Do",
            NodeKind::Else => "Else",
            NodeKind::Enum => "Enum",
            NodeKind::Export => "Export",
            NodeKind::Extends => "Extends",
            NodeKind::False => "False",
            NodeKind::Finally => "Finally",
            NodeKind::For => "For",
            NodeKind::Function => "Function",
            NodeKind::If => "If",
            NodeKind::Import => "Import",
            NodeKind::In => "In",
            NodeKind::Instanceof => "Instanceof",
            NodeKind::New => "New",
            NodeKind::Null => "Null",
            NodeKind::Return => "Return",
            NodeKind::Super => "Super",
            NodeKind::Switch => "Switch",
            NodeKind::This => "This",
            NodeKind::Throw => "Throw",
            NodeKind::True => "True",
            NodeKind::Try => "Try",
            NodeKind::Typeof => "Typeof",
            NodeKind::Var => "Var",
            NodeKind::Void => "Void",
            NodeKind::While => "While",
            NodeKind::With => "With",
            NodeKind::Implements => "Implements",
            NodeKind::Interface => "Interface",
            NodeKind::Let => "Let",
            NodeKind::Package => "Package",
            NodeKind::Private => "Private",
            NodeKind::Protected => "Protected",
            NodeKind::Public => "Public",
            NodeKind::Static => "Static",
            NodeKind::Yield => "Yield",
            NodeKind::Abstract => "Abstract",
            NodeKind::As => "As",
            NodeKind::Asserts => "Asserts",
            NodeKind::Assert => "Assert",
            NodeKind::Any => "Any",
            NodeKind::Async => "Async",
            NodeKind::Await => "Await",
            NodeKind::Boolean => "Boolean",
            NodeKind::Constructor => "Constructor",
            NodeKind::Declare => "Declare",
            NodeKind::Get => "Get",
            NodeKind::Infer => "Infer",
            NodeKind::Intrinsic => "Intrinsic",
            NodeKind::Is => "Is",
            NodeKind::KeyOf => "KeyOf",
            NodeKind::Module => "Module",
            NodeKind::Namespace => "Namespace",
            NodeKind::Never => "Never",
            NodeKind::Out => "Out",
            NodeKind::Override => "Override",
            NodeKind::Readonly => "Readonly",
            NodeKind::Require => "Require",
            NodeKind::Number => "Number",
            NodeKind::Object => "Object",
            NodeKind::Set => "Set",
            NodeKind::String => "String",
            NodeKind::Symbol => "Symbol",
            NodeKind::Type => "Type",
            NodeKind::Undefined => "Undefined",
            NodeKind::Unique => "Unique",
            NodeKind::UnknownKeyword => "unknown",
            NodeKind::From => "From",
            NodeKind::Global => "Global",
            NodeKind::BigInt => "BigInt",
            NodeKind::Of => "Of",
            NodeKind::QualifiedName => "QualifiedName",
            NodeKind::ComputedPropertyName => "ComputedPropertyName",
            NodeKind::TypeParameter => "TypeParameter",
            NodeKind::Parameter => "Parameter",
            NodeKind::Decorator => "Decorator",
            NodeKind::PropertySignature => "PropertySignature",
            NodeKind::PropertyDeclaration => "PropertyDeclaration",
            NodeKind::MethodSignature => "MethodSignature",
            NodeKind::MethodDeclaration => "MethodDeclaration",
            NodeKind::ClassStaticBlockDeclaration => "ClassStaticBlockDeclaration",
            NodeKind::Constructor_ => "Constructor",
            NodeKind::GetAccessor => "GetAccessor",
            NodeKind::SetAccessor => "SetAccessor",
            NodeKind::CallSignature => "CallSignature",
            NodeKind::ConstructSignature => "ConstructSignature",
            NodeKind::IndexSignature => "IndexSignature",
            NodeKind::TypePredicate => "TypePredicate",
            NodeKind::TypeReference => "TypeReference",
            NodeKind::FunctionType => "FunctionType",
            NodeKind::ConstructorType => "ConstructorType",
            NodeKind::TypeQuery => "TypeQuery",
            NodeKind::TypeLiteral => "TypeLiteral",
            NodeKind::ArrayType => "ArrayType",
            NodeKind::TupleType => "TupleType",
            NodeKind::OptionalType => "OptionalType",
            NodeKind::RestType => "RestType",
            NodeKind::UnionType => "UnionType",
            NodeKind::IntersectionType => "IntersectionType",
            NodeKind::ConditionalType => "ConditionalType",
            NodeKind::InferType => "InferType",
            NodeKind::ParenthesizedType => "ParenthesizedType",
            NodeKind::ThisType => "ThisType",
            NodeKind::TypeOperator => "TypeOperator",
            NodeKind::IndexedAccessType => "IndexedAccessType",
            NodeKind::MappedType => "MappedType",
            NodeKind::LiteralType => "LiteralType",
            NodeKind::NamedTupleMember => "NamedTupleMember",
            NodeKind::TemplateLiteralType => "TemplateLiteralType",
            NodeKind::TemplateLiteralTypeSpan => "TemplateLiteralTypeSpan",
            NodeKind::ImportType => "ImportType",
            NodeKind::ObjectBindingPattern => "ObjectBindingPattern",
            NodeKind::ArrayBindingPattern => "ArrayBindingPattern",
            NodeKind::BindingElement => "BindingElement",
            NodeKind::ArrayLiteralExpression => "ArrayLiteralExpression",
            NodeKind::ObjectLiteralExpression => "ObjectLiteralExpression",
            NodeKind::PropertyAccessExpression => "PropertyAccessExpression",
            NodeKind::ElementAccessExpression => "ElementAccessExpression",
            NodeKind::CallExpression => "CallExpression",
            NodeKind::NewExpression => "NewExpression",
            NodeKind::TaggedTemplateExpression => "TaggedTemplateExpression",
            NodeKind::TypeAssertionExpression => "TypeAssertionExpression",
            NodeKind::ParenthesizedExpression => "ParenthesizedExpression",
            NodeKind::FunctionExpression => "FunctionExpression",
            NodeKind::ArrowFunction => "ArrowFunction",
            NodeKind::DeleteExpression => "DeleteExpression",
            NodeKind::TypeOfExpression => "TypeOfExpression",
            NodeKind::VoidExpression => "VoidExpression",
            NodeKind::AwaitExpression => "AwaitExpression",
            NodeKind::PrefixUnaryExpression => "PrefixUnaryExpression",
            NodeKind::PostfixUnaryExpression => "PostfixUnaryExpression",
            NodeKind::BinaryExpression => "BinaryExpression",
            NodeKind::ConditionalExpression => "ConditionalExpression",
            NodeKind::TemplateExpression => "TemplateExpression",
            NodeKind::YieldExpression => "YieldExpression",
            NodeKind::SpreadElement => "SpreadElement",
            NodeKind::ClassExpression => "ClassExpression",
            NodeKind::OmittedExpression => "OmittedExpression",
            NodeKind::ExpressionWithTypeArguments => "ExpressionWithTypeArguments",
            NodeKind::AsExpression => "AsExpression",
            NodeKind::NonNullExpression => "NonNullExpression",
            NodeKind::MetaProperty => "MetaProperty",
            NodeKind::SyntheticExpression => "SyntheticExpression",
            NodeKind::PropertyAssignment => "PropertyAssignment",
            NodeKind::ShorthandPropertyAssignment => "ShorthandPropertyAssignment",
            NodeKind::SpreadAssignment => "SpreadAssignment",
            NodeKind::EnumMember => "EnumMember",
            NodeKind::Unparsed => "Unparsed",
            NodeKind::UnparsedPrologue => "UnparsedPrologue",
            NodeKind::UnparsedPrepend => "UnparsedPrepend",
            NodeKind::UnparsedText => "UnparsedText",
            NodeKind::UnparsedInternalText => "UnparsedInternalText",
            NodeKind::UnparsedSyntheticReference => "UnparsedSyntheticReference",
            NodeKind::SourceFile => "SourceFile",
            NodeKind::Bundle => "Bundle",
            NodeKind::JsDocTypeExpression => "JsDocTypeExpression",
            NodeKind::JsDocNameReference => "JsDocNameReference",
            NodeKind::JsDocMemberName => "JsDocMemberName",
            NodeKind::JsDocAllType => "JsDocAllType",
            NodeKind::JsDocUnknownType => "JsDocUnknownType",
            NodeKind::JsDocNullableType => "JsDocNullableType",
            NodeKind::JsDocNonNullableType => "JsDocNonNullableType",
            NodeKind::JsDocOptionalType => "JsDocOptionalType",
            NodeKind::JsDocFunctionType => "JsDocFunctionType",
            NodeKind::JsDocVariadicType => "JsDocVariadicType",
            NodeKind::JsDocNamepathType => "JsDocNamepathType",
            NodeKind::JsDoc => "JsDoc",
            NodeKind::JsDocText => "JsDocText",
            NodeKind::JsDocTypeLiteral => "JsDocTypeLiteral",
            NodeKind::JsDocSignature => "JsDocSignature",
            NodeKind::JsDocLink => "JsDocLink",
            NodeKind::JsDocLinkCode => "JsDocLinkCode",
            NodeKind::JsDocLinkPlain => "JsDocLinkPlain",
            NodeKind::JsDocTag => "JsDocTag",
            NodeKind::JsDocAugmentsTag => "JsDocAugmentsTag",
            NodeKind::JsDocImplementsTag => "JsDocImplementsTag",
            NodeKind::JsDocAuthorTag => "JsDocAuthorTag",
            NodeKind::JsDocDeprecatedTag => "JsDocDeprecatedTag",
            NodeKind::JsDocClassTag => "JsDocClassTag",
            NodeKind::JsDocPublicTag => "JsDocPublicTag",
            NodeKind::JsDocPrivateTag => "JsDocPrivateTag",
            NodeKind::JsDocProtectedTag => "JsDocProtectedTag",
            NodeKind::JsDocReadonlyTag => "JsDocReadonlyTag",
            NodeKind::JsDocOverrideTag => "JsDocOverrideTag",
            NodeKind::JsDocCallbackTag => "JsDocCallbackTag",
            NodeKind::JsDocEnumTag => "JsDocEnumTag",
            NodeKind::JsDocParameterTag => "JsDocParameterTag",
            NodeKind::JsDocReturnTag => "JsDocReturnTag",
            NodeKind::JsDocThisTag => "JsDocThisTag",
            NodeKind::JsDocTypeTag => "JsDocTypeTag",
            NodeKind::JsDocTemplateTag => "JsDocTemplateTag",
            NodeKind::JsDocTypedefTag => "JsDocTypedefTag",
            NodeKind::JsDocSeeTag => "JsDocSeeTag",
            NodeKind::JsDocPropertyTag => "JsDocPropertyTag",
            NodeKind::Block => "Block",
            NodeKind::VariableStatement => "VariableStatement",
            NodeKind::EmptyStatement => "EmptyStatement",
            NodeKind::ExpressionStatement => "ExpressionStatement",
            NodeKind::IfStatement => "IfStatement",
            NodeKind::DoStatement => "DoStatement",
            NodeKind::WhileStatement => "WhileStatement",
            NodeKind::ForStatement => "ForStatement",
            NodeKind::ForInStatement => "ForInStatement",
            NodeKind::ForOfStatement => "ForOfStatement",
            NodeKind::ContinueStatement => "ContinueStatement",
            NodeKind::BreakStatement => "BreakStatement",
            NodeKind::ReturnStatement => "ReturnStatement",
            NodeKind::WithStatement => "WithStatement",
            NodeKind::SwitchStatement => "SwitchStatement",
            NodeKind::LabeledStatement => "LabeledStatement",
            NodeKind::ThrowStatement => "ThrowStatement",
            NodeKind::TryStatement => "TryStatement",
            NodeKind::DebuggerStatement => "DebuggerStatement",
            NodeKind::VariableDeclaration => "VariableDeclaration",
            NodeKind::VariableDeclarationList => "VariableDeclarationList",
            NodeKind::FunctionDeclaration => "FunctionDeclaration",
            NodeKind::ClassDeclaration => "ClassDeclaration",
            NodeKind::InterfaceDeclaration => "InterfaceDeclaration",
            NodeKind::TypeAliasDeclaration => "TypeAliasDeclaration",
            NodeKind::EnumDeclaration => "EnumDeclaration",
            NodeKind::ModuleDeclaration => "ModuleDeclaration",
            NodeKind::ModuleBlock => "ModuleBlock",
            NodeKind::CaseBlock => "CaseBlock",
            NodeKind::NamespaceExportDeclaration => "NamespaceExportDeclaration",
            NodeKind::ImportEqualsDeclaration => "ImportEqualsDeclaration",
            NodeKind::ImportDeclaration => "ImportDeclaration",
            NodeKind::ImportClause => "ImportClause",
            NodeKind::NamespaceImport => "NamespaceImport",
            NodeKind::NamedImports => "NamedImports",
            NodeKind::ImportSpecifier => "ImportSpecifier",
            NodeKind::ExportAssignment => "ExportAssignment",
            NodeKind::ExportDeclaration => "ExportDeclaration",
            NodeKind::NamedExports => "NamedExports",
            NodeKind::NamespaceExport => "NamespaceExport",
            NodeKind::ExportSpecifier => "ExportSpecifier",
            NodeKind::MissingDeclaration => "MissingDeclaration",
            NodeKind::ExternalModuleReference => "ExternalModuleReference",
            NodeKind::JsxElement => "JsxElement",
            NodeKind::JsxSelfClosingElement => "JsxSelfClosingElement",
            NodeKind::JsxOpeningElement => "JsxOpeningElement",
            NodeKind::JsxClosingElement => "JsxClosingElement",
            NodeKind::JsxFragment => "JsxFragment",
            NodeKind::JsxOpeningFragment => "JsxOpeningFragment",
            NodeKind::JsxClosingFragment => "JsxClosingFragment",
            NodeKind::JsxAttribute => "JsxAttribute",
            NodeKind::JsxAttributes => "JsxAttributes",
            NodeKind::JsxSpreadAttribute => "JsxSpreadAttribute",
            NodeKind::JsxExpression => "JsxExpression",
            NodeKind::CaseClause => "CaseClause",
            NodeKind::DefaultClause => "DefaultClause",
            NodeKind::HeritageClause => "HeritageClause",
            NodeKind::CatchClause => "CatchClause",
            NodeKind::AssertClause => "AssertClause",
            NodeKind::AssertEntry => "AssertEntry",
            NodeKind::PropertyAccessChain => "PropertyAccessChain",
            NodeKind::ElementAccessChain => "ElementAccessChain",
            NodeKind::CallChain => "CallChain",
            NodeKind::OptionalChain => "OptionalChain",
            NodeKind::TemplateSpan => "TemplateSpan",
            NodeKind::SemicolonClassElement => "SemicolonClassElement",
            NodeKind::SyntheticReferenceExpression => "SyntheticReferenceExpression",
            NodeKind::PrivateIdentifier => "PrivateIdentifier",
            NodeKind::Count => "Count",
        }
    }

    /// Convert from u16 to NodeKind, returning Unknown for invalid values.
    #[inline]
    pub const fn from_u16(value: u16) -> Self {
        if value >= NodeKind::Count as u16 {
            return NodeKind::Unknown;
        }
        // SAFETY: We've verified the value is within range
        unsafe { std::mem::transmute(value) }
    }
}

impl From<u16> for NodeKind {
    #[inline]
    fn from(value: u16) -> Self {
        NodeKind::from_u16(value)
    }
}

impl From<NodeKind> for u16 {
    #[inline]
    fn from(kind: NodeKind) -> Self {
        kind as u16
    }
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
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
    fn test_node_kind_roundtrip() {
        for i in 0..NodeKind::Count as u16 {
            let kind = NodeKind::from_u16(i);
            assert_eq!(kind as u16, i);
        }
    }

    #[test]
    fn test_is_keyword() {
        assert!(NodeKind::If.is_keyword());
        assert!(NodeKind::For.is_keyword());
        assert!(NodeKind::Async.is_keyword());
        assert!(!NodeKind::Identifier.is_keyword());
        assert!(!NodeKind::NumericLiteral.is_keyword());
    }

    #[test]
    fn test_is_literal() {
        assert!(NodeKind::NumericLiteral.is_literal());
        assert!(NodeKind::StringLiteral.is_literal());
        assert!(!NodeKind::Identifier.is_literal());
        assert!(!NodeKind::If.is_literal());
    }

    #[test]
    fn test_is_statement() {
        assert!(NodeKind::Block.is_statement());
        assert!(NodeKind::IfStatement.is_statement());
        assert!(NodeKind::ForStatement.is_statement());
        assert!(!NodeKind::Identifier.is_statement());
    }

    #[test]
    fn test_is_expression() {
        assert!(NodeKind::Identifier.is_expression());
        assert!(NodeKind::CallExpression.is_expression());
        assert!(NodeKind::NumericLiteral.is_expression());
        assert!(!NodeKind::IfStatement.is_expression());
    }

    #[test]
    fn test_name() {
        assert_eq!(NodeKind::Identifier.name(), "Identifier");
        assert_eq!(NodeKind::IfStatement.name(), "IfStatement");
        assert_eq!(NodeKind::NumericLiteral.name(), "NumericLiteral");
    }
}
