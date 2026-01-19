//! Abstract Syntax Tree definitions
//!
//! Defines the AST node types for TypeScript.

use zang_core::{InternedString, Span};

/// Node ID for AST nodes
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct NodeId(u32);

impl NodeId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Syntax kind for AST nodes
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    // Tokens
    Unknown = 0,
    EndOfFile,
    SingleLineComment,
    MultiLineComment,
    NewLine,
    Whitespace,

    // Literals
    NumericLiteral,
    BigIntLiteral,
    StringLiteral,
    NoSubstitutionTemplateLiteral,
    TemplateHead,
    TemplateMiddle,
    TemplateTail,
    RegularExpressionLiteral,

    // Punctuation
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Dot,
    DotDotDot,
    Semicolon,
    Comma,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    EqualsEquals,
    ExclamationEquals,
    EqualsEqualsEquals,
    ExclamationEqualsEquals,
    EqualsGreaterThan,
    Plus,
    Minus,
    Asterisk,
    AsteriskAsterisk,
    Slash,
    Percent,
    PlusPlus,
    MinusMinus,
    LessThanLessThan,
    GreaterThanGreaterThan,
    GreaterThanGreaterThanGreaterThan,
    Ampersand,
    Bar,
    Caret,
    Exclamation,
    Tilde,
    AmpersandAmpersand,
    BarBar,
    Question,
    QuestionQuestion,
    QuestionDot,
    Colon,
    At,
    Equals,
    PlusEquals,
    MinusEquals,
    AsteriskEquals,
    SlashEquals,
    PercentEquals,
    AmpersandEquals,
    BarEquals,
    CaretEquals,
    LessThanLessThanEquals,
    GreaterThanGreaterThanEquals,
    GreaterThanGreaterThanGreaterThanEquals,
    AmpersandAmpersandEquals,
    BarBarEquals,
    QuestionQuestionEquals,
    AsteriskAsteriskEquals,
    Hash,

    // Identifiers and keywords
    Identifier,

    // Reserved words
    BreakKeyword,
    CaseKeyword,
    CatchKeyword,
    ClassKeyword,
    ConstKeyword,
    ContinueKeyword,
    DebuggerKeyword,
    DefaultKeyword,
    DeleteKeyword,
    DoKeyword,
    ElseKeyword,
    EnumKeyword,
    ExportKeyword,
    ExtendsKeyword,
    FalseKeyword,
    FinallyKeyword,
    ForKeyword,
    FunctionKeyword,
    IfKeyword,
    ImportKeyword,
    InKeyword,
    InstanceOfKeyword,
    NewKeyword,
    NullKeyword,
    ReturnKeyword,
    SuperKeyword,
    SwitchKeyword,
    ThisKeyword,
    ThrowKeyword,
    TrueKeyword,
    TryKeyword,
    TypeOfKeyword,
    VarKeyword,
    VoidKeyword,
    WhileKeyword,
    WithKeyword,

    // Strict mode reserved words
    ImplementsKeyword,
    InterfaceKeyword,
    LetKeyword,
    PackageKeyword,
    PrivateKeyword,
    ProtectedKeyword,
    PublicKeyword,
    StaticKeyword,
    YieldKeyword,

    // Contextual keywords
    AbstractKeyword,
    AsKeyword,
    AssertsKeyword,
    AnyKeyword,
    AsyncKeyword,
    AwaitKeyword,
    BooleanKeyword,
    ConstructorKeyword,
    DeclareKeyword,
    GetKeyword,
    InferKeyword,
    IsKeyword,
    KeyOfKeyword,
    ModuleKeyword,
    NamespaceKeyword,
    NeverKeyword,
    ReadonlyKeyword,
    RequireKeyword,
    NumberKeyword,
    ObjectKeyword,
    SetKeyword,
    StringKeyword,
    SymbolKeyword,
    TypeKeyword,
    UndefinedKeyword,
    UniqueKeyword,
    UnknownKeyword,
    FromKeyword,
    GlobalKeyword,
    BigIntKeyword,
    OverrideKeyword,
    OfKeyword,

    // Nodes
    SourceFile,
    Block,
    EmptyStatement,
    VariableStatement,
    ExpressionStatement,
    IfStatement,
    DoStatement,
    WhileStatement,
    ForStatement,
    ForInStatement,
    ForOfStatement,
    ContinueStatement,
    BreakStatement,
    ReturnStatement,
    WithStatement,
    SwitchStatement,
    LabeledStatement,
    ThrowStatement,
    TryStatement,
    DebuggerStatement,

    // Declarations
    VariableDeclaration,
    VariableDeclarationList,
    FunctionDeclaration,
    ClassDeclaration,
    InterfaceDeclaration,
    TypeAliasDeclaration,
    EnumDeclaration,
    ModuleDeclaration,
    ImportDeclaration,
    ExportDeclaration,
    ExportAssignment,

    // Expressions
    ArrayLiteralExpression,
    ObjectLiteralExpression,
    PropertyAccessExpression,
    ElementAccessExpression,
    CallExpression,
    NewExpression,
    TaggedTemplateExpression,
    TypeAssertionExpression,
    ParenthesizedExpression,
    FunctionExpression,
    ArrowFunction,
    DeleteExpression,
    TypeOfExpression,
    VoidExpression,
    AwaitExpression,
    PrefixUnaryExpression,
    PostfixUnaryExpression,
    BinaryExpression,
    ConditionalExpression,
    TemplateExpression,
    YieldExpression,
    SpreadElement,
    ClassExpression,
    AsExpression,
    NonNullExpression,

    // Types
    TypeReference,
    FunctionType,
    ConstructorType,
    TypeQuery,
    TypeLiteral,
    ArrayType,
    TupleType,
    UnionType,
    IntersectionType,
    ConditionalType,
    InferType,
    ParenthesizedType,
    TypeOperator,
    IndexedAccessType,
    MappedType,
    LiteralType,
    TemplateLiteralType,
    ImportType,
    ThisType,

    // Other
    Parameter,
    Decorator,
    PropertySignature,
    PropertyDeclaration,
    MethodSignature,
    MethodDeclaration,
    Constructor,
    GetAccessor,
    SetAccessor,
    CallSignature,
    ConstructSignature,
    IndexSignature,
    TypeParameter,
    PropertyAssignment,
    ShorthandPropertyAssignment,
    SpreadAssignment,
    EnumMember,
    CaseClause,
    DefaultClause,
    CatchClause,
    HeritageClause,
    JsxElement,
    JsxSelfClosingElement,
    JsxFragment,
}

impl SyntaxKind {
    /// Returns true if this is a keyword
    pub fn is_keyword(self) -> bool {
        matches!(
            self,
            SyntaxKind::BreakKeyword
                | SyntaxKind::CaseKeyword
                | SyntaxKind::CatchKeyword
                | SyntaxKind::ClassKeyword
                | SyntaxKind::ConstKeyword
                | SyntaxKind::ContinueKeyword
                | SyntaxKind::DebuggerKeyword
                | SyntaxKind::DefaultKeyword
                | SyntaxKind::DeleteKeyword
                | SyntaxKind::DoKeyword
                | SyntaxKind::ElseKeyword
                | SyntaxKind::EnumKeyword
                | SyntaxKind::ExportKeyword
                | SyntaxKind::ExtendsKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::FinallyKeyword
                | SyntaxKind::ForKeyword
                | SyntaxKind::FunctionKeyword
                | SyntaxKind::IfKeyword
                | SyntaxKind::ImportKeyword
                | SyntaxKind::InKeyword
                | SyntaxKind::InstanceOfKeyword
                | SyntaxKind::NewKeyword
                | SyntaxKind::NullKeyword
                | SyntaxKind::ReturnKeyword
                | SyntaxKind::SuperKeyword
                | SyntaxKind::SwitchKeyword
                | SyntaxKind::ThisKeyword
                | SyntaxKind::ThrowKeyword
                | SyntaxKind::TrueKeyword
                | SyntaxKind::TryKeyword
                | SyntaxKind::TypeOfKeyword
                | SyntaxKind::VarKeyword
                | SyntaxKind::VoidKeyword
                | SyntaxKind::WhileKeyword
                | SyntaxKind::WithKeyword
        )
    }

    /// Returns true if this is a type keyword
    pub fn is_type_keyword(self) -> bool {
        matches!(
            self,
            SyntaxKind::AnyKeyword
                | SyntaxKind::BooleanKeyword
                | SyntaxKind::NeverKeyword
                | SyntaxKind::NumberKeyword
                | SyntaxKind::ObjectKeyword
                | SyntaxKind::StringKeyword
                | SyntaxKind::SymbolKeyword
                | SyntaxKind::UndefinedKeyword
                | SyntaxKind::UnknownKeyword
                | SyntaxKind::VoidKeyword
                | SyntaxKind::BigIntKeyword
        )
    }
}

/// Base trait for all AST nodes
pub trait Node {
    fn kind(&self) -> SyntaxKind;
    fn span(&self) -> Span;
}

/// A source file node
#[derive(Clone, Debug)]
pub struct SourceFile {
    pub span: Span,
    pub statements: Vec<Statement>,
    pub end_of_file_token: Span,
    pub file_name: InternedString,
}

impl Node for SourceFile {
    fn kind(&self) -> SyntaxKind {
        SyntaxKind::SourceFile
    }

    fn span(&self) -> Span {
        self.span
    }
}

/// Statement node types
#[derive(Clone, Debug)]
pub enum Statement {
    Block(BlockStatement),
    Variable(VariableStatement),
    Expression(ExpressionStatement),
    If(IfStatement),
    Return(ReturnStatement),
    Function(FunctionDeclaration),
    Class(ClassDeclaration),
    Import(ImportDeclaration),
    Export(ExportDeclaration),
    ExportAssignment(ExportAssignment),
    // ... more statement types
    Empty(Span),
}

impl Node for Statement {
    fn kind(&self) -> SyntaxKind {
        match self {
            Statement::Block(_) => SyntaxKind::Block,
            Statement::Variable(_) => SyntaxKind::VariableStatement,
            Statement::Expression(_) => SyntaxKind::ExpressionStatement,
            Statement::If(_) => SyntaxKind::IfStatement,
            Statement::Return(_) => SyntaxKind::ReturnStatement,
            Statement::Function(_) => SyntaxKind::FunctionDeclaration,
            Statement::Class(_) => SyntaxKind::ClassDeclaration,
            Statement::Import(_) => SyntaxKind::ImportDeclaration,
            Statement::Export(_) => SyntaxKind::ExportDeclaration,
            Statement::ExportAssignment(_) => SyntaxKind::ExportAssignment,
            Statement::Empty(_) => SyntaxKind::EmptyStatement,
        }
    }

    fn span(&self) -> Span {
        match self {
            Statement::Block(s) => s.span,
            Statement::Variable(s) => s.span,
            Statement::Expression(s) => s.span,
            Statement::If(s) => s.span,
            Statement::Return(s) => s.span,
            Statement::Function(s) => s.span,
            Statement::Class(s) => s.span,
            Statement::Import(s) => s.span,
            Statement::Export(s) => s.span,
            Statement::ExportAssignment(s) => s.span,
            Statement::Empty(span) => *span,
        }
    }
}

/// Block statement
#[derive(Clone, Debug)]
pub struct BlockStatement {
    pub span: Span,
    pub statements: Vec<Statement>,
}

/// Variable statement
#[derive(Clone, Debug)]
pub struct VariableStatement {
    pub span: Span,
    pub declaration_list: VariableDeclarationList,
}

/// Variable declaration list
#[derive(Clone, Debug)]
pub struct VariableDeclarationList {
    pub span: Span,
    pub declarations: Vec<VariableDeclaration>,
    pub flags: VariableDeclarationKind,
}

/// Variable declaration kind
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VariableDeclarationKind {
    Var,
    Let,
    Const,
}

/// Variable declaration
#[derive(Clone, Debug)]
pub struct VariableDeclaration {
    pub span: Span,
    pub name: BindingName,
    pub type_annotation: Option<TypeNode>,
    pub initializer: Option<Expression>,
}

/// Binding name (identifier or pattern)
#[derive(Clone, Debug)]
pub enum BindingName {
    Identifier(Identifier),
    ObjectPattern(ObjectBindingPattern),
    ArrayPattern(ArrayBindingPattern),
}

/// Identifier
#[derive(Clone, Debug)]
pub struct Identifier {
    pub span: Span,
    pub name: InternedString,
}

/// Object binding pattern
#[derive(Clone, Debug)]
pub struct ObjectBindingPattern {
    pub span: Span,
    pub elements: Vec<BindingElement>,
}

/// Array binding pattern
#[derive(Clone, Debug)]
pub struct ArrayBindingPattern {
    pub span: Span,
    pub elements: Vec<Option<BindingElement>>,
}

/// Binding element
#[derive(Clone, Debug)]
pub struct BindingElement {
    pub span: Span,
    pub property_name: Option<InternedString>,
    pub name: BindingName,
    pub initializer: Option<Expression>,
}

/// Expression statement
#[derive(Clone, Debug)]
pub struct ExpressionStatement {
    pub span: Span,
    pub expression: Expression,
}

/// If statement
#[derive(Clone, Debug)]
pub struct IfStatement {
    pub span: Span,
    pub condition: Expression,
    pub then_statement: Box<Statement>,
    pub else_statement: Option<Box<Statement>>,
}

/// Return statement
#[derive(Clone, Debug)]
pub struct ReturnStatement {
    pub span: Span,
    pub expression: Option<Expression>,
}

/// Function declaration
#[derive(Clone, Debug)]
pub struct FunctionDeclaration {
    pub span: Span,
    pub name: Option<Identifier>,
    pub type_parameters: Option<Vec<TypeParameterNode>>,
    pub parameters: Vec<ParameterNode>,
    pub return_type: Option<TypeNode>,
    pub body: Option<BlockStatement>,
    pub is_async: bool,
    pub is_generator: bool,
}

/// Class declaration
#[derive(Clone, Debug)]
pub struct ClassDeclaration {
    pub span: Span,
    pub name: Option<Identifier>,
    pub type_parameters: Option<Vec<TypeParameterNode>>,
    pub heritage_clauses: Vec<HeritageClause>,
    pub members: Vec<ClassMember>,
}

/// Heritage clause
#[derive(Clone, Debug)]
pub struct HeritageClause {
    pub span: Span,
    pub token: HeritageClauseKind,
    pub types: Vec<ExpressionWithTypeArguments>,
}

/// Heritage clause kind
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeritageClauseKind {
    Extends,
    Implements,
}

/// Expression with type arguments
#[derive(Clone, Debug)]
pub struct ExpressionWithTypeArguments {
    pub span: Span,
    pub expression: Expression,
    pub type_arguments: Option<Vec<TypeNode>>,
}

/// Class member
#[derive(Clone, Debug)]
pub enum ClassMember {
    Property(PropertyDeclaration),
    Method(MethodDeclaration),
    Constructor(ConstructorDeclaration),
    GetAccessor(GetAccessorDeclaration),
    SetAccessor(SetAccessorDeclaration),
}

/// Property declaration
#[derive(Clone, Debug)]
pub struct PropertyDeclaration {
    pub span: Span,
    pub name: PropertyName,
    pub type_annotation: Option<TypeNode>,
    pub initializer: Option<Expression>,
    pub modifiers: Modifiers,
}

/// Method declaration
#[derive(Clone, Debug)]
pub struct MethodDeclaration {
    pub span: Span,
    pub name: PropertyName,
    pub type_parameters: Option<Vec<TypeParameterNode>>,
    pub parameters: Vec<ParameterNode>,
    pub return_type: Option<TypeNode>,
    pub body: Option<BlockStatement>,
    pub modifiers: Modifiers,
}

/// Constructor declaration
#[derive(Clone, Debug)]
pub struct ConstructorDeclaration {
    pub span: Span,
    pub parameters: Vec<ParameterNode>,
    pub body: Option<BlockStatement>,
    pub modifiers: Modifiers,
}

/// Get accessor
#[derive(Clone, Debug)]
pub struct GetAccessorDeclaration {
    pub span: Span,
    pub name: PropertyName,
    pub return_type: Option<TypeNode>,
    pub body: Option<BlockStatement>,
    pub modifiers: Modifiers,
}

/// Set accessor
#[derive(Clone, Debug)]
pub struct SetAccessorDeclaration {
    pub span: Span,
    pub name: PropertyName,
    pub parameter: ParameterNode,
    pub body: Option<BlockStatement>,
    pub modifiers: Modifiers,
}

/// Property name
#[derive(Clone, Debug)]
pub enum PropertyName {
    Identifier(Identifier),
    StringLiteral(StringLiteral),
    NumericLiteral(NumericLiteral),
    Computed(Box<Expression>),
}

/// String literal
#[derive(Clone, Debug)]
pub struct StringLiteral {
    pub span: Span,
    pub value: InternedString,
}

/// Numeric literal
#[derive(Clone, Debug)]
pub struct NumericLiteral {
    pub span: Span,
    pub value: f64,
}

/// Modifiers
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers(u16);

impl Modifiers {
    pub const NONE: Self = Self(0);
    pub const EXPORT: Self = Self(1 << 0);
    pub const AMBIENT: Self = Self(1 << 1);
    pub const PUBLIC: Self = Self(1 << 2);
    pub const PRIVATE: Self = Self(1 << 3);
    pub const PROTECTED: Self = Self(1 << 4);
    pub const STATIC: Self = Self(1 << 5);
    pub const READONLY: Self = Self(1 << 6);
    pub const ABSTRACT: Self = Self(1 << 7);
    pub const ASYNC: Self = Self(1 << 8);
    pub const DEFAULT: Self = Self(1 << 9);
    pub const CONST: Self = Self(1 << 10);
    pub const OVERRIDE: Self = Self(1 << 11);

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for Modifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

/// Parameter node
#[derive(Clone, Debug)]
pub struct ParameterNode {
    pub span: Span,
    pub name: BindingName,
    pub type_annotation: Option<TypeNode>,
    pub initializer: Option<Expression>,
    pub is_optional: bool,
    pub is_rest: bool,
    pub modifiers: Modifiers,
}

/// Type parameter node
#[derive(Clone, Debug)]
pub struct TypeParameterNode {
    pub span: Span,
    pub name: Identifier,
    pub constraint: Option<Box<TypeNode>>,
    pub default: Option<Box<TypeNode>>,
}

/// Type node
#[derive(Clone, Debug)]
pub enum TypeNode {
    Keyword(KeywordTypeNode),
    Reference(TypeReferenceNode),
    Array(ArrayTypeNode),
    Tuple(TupleTypeNode),
    Union(UnionTypeNode),
    Intersection(IntersectionTypeNode),
    Function(FunctionTypeNode),
    Conditional(ConditionalTypeNode),
    Indexed(IndexedAccessTypeNode),
    Mapped(Box<MappedTypeNode>),
    TypeLiteral(TypeLiteralNode),
    Literal(LiteralTypeNode),
    Infer(InferTypeNode),
    Parenthesized(Box<TypeNode>),
}

/// Keyword type node
#[derive(Clone, Debug)]
pub struct KeywordTypeNode {
    pub span: Span,
    pub keyword: SyntaxKind,
}

/// Type reference node
#[derive(Clone, Debug)]
pub struct TypeReferenceNode {
    pub span: Span,
    pub type_name: EntityName,
    pub type_arguments: Option<Vec<TypeNode>>,
}

/// Entity name
#[derive(Clone, Debug)]
pub enum EntityName {
    Identifier(Identifier),
    Qualified(QualifiedName),
}

/// Qualified name
#[derive(Clone, Debug)]
pub struct QualifiedName {
    pub span: Span,
    pub left: Box<EntityName>,
    pub right: Identifier,
}

/// Array type node
#[derive(Clone, Debug)]
pub struct ArrayTypeNode {
    pub span: Span,
    pub element_type: Box<TypeNode>,
}

/// Tuple type node
#[derive(Clone, Debug)]
pub struct TupleTypeNode {
    pub span: Span,
    pub elements: Vec<TypeNode>,
}

/// Union type node
#[derive(Clone, Debug)]
pub struct UnionTypeNode {
    pub span: Span,
    pub types: Vec<TypeNode>,
}

/// Intersection type node
#[derive(Clone, Debug)]
pub struct IntersectionTypeNode {
    pub span: Span,
    pub types: Vec<TypeNode>,
}

/// Function type node
#[derive(Clone, Debug)]
pub struct FunctionTypeNode {
    pub span: Span,
    pub type_parameters: Option<Vec<TypeParameterNode>>,
    pub parameters: Vec<ParameterNode>,
    pub return_type: Box<TypeNode>,
}

/// Conditional type node
#[derive(Clone, Debug)]
pub struct ConditionalTypeNode {
    pub span: Span,
    pub check_type: Box<TypeNode>,
    pub extends_type: Box<TypeNode>,
    pub true_type: Box<TypeNode>,
    pub false_type: Box<TypeNode>,
}

/// Indexed access type node
#[derive(Clone, Debug)]
pub struct IndexedAccessTypeNode {
    pub span: Span,
    pub object_type: Box<TypeNode>,
    pub index_type: Box<TypeNode>,
}

/// Mapped type node
#[derive(Clone, Debug)]
pub struct MappedTypeNode {
    pub span: Span,
    pub type_parameter: Box<TypeParameterNode>,
    pub name_type: Option<Box<TypeNode>>,
    pub type_annotation: Option<Box<TypeNode>>,
    pub readonly_token: Option<Span>,
    pub question_token: Option<Span>,
}

/// Type literal node
#[derive(Clone, Debug)]
pub struct TypeLiteralNode {
    pub span: Span,
    pub members: Vec<TypeElement>,
}

/// Type element
#[derive(Clone, Debug)]
pub enum TypeElement {
    Property(PropertySignature),
    Method(MethodSignature),
    Index(IndexSignature),
    Call(CallSignature),
    Construct(ConstructSignature),
}

/// Property signature
#[derive(Clone, Debug)]
pub struct PropertySignature {
    pub span: Span,
    pub name: PropertyName,
    pub type_annotation: Option<TypeNode>,
    pub is_optional: bool,
    pub is_readonly: bool,
}

/// Method signature
#[derive(Clone, Debug)]
pub struct MethodSignature {
    pub span: Span,
    pub name: PropertyName,
    pub type_parameters: Option<Vec<TypeParameterNode>>,
    pub parameters: Vec<ParameterNode>,
    pub return_type: Option<TypeNode>,
    pub is_optional: bool,
}

/// Index signature
#[derive(Clone, Debug)]
pub struct IndexSignature {
    pub span: Span,
    pub parameter: ParameterNode,
    pub type_annotation: TypeNode,
    pub is_readonly: bool,
}

/// Call signature
#[derive(Clone, Debug)]
pub struct CallSignature {
    pub span: Span,
    pub type_parameters: Option<Vec<TypeParameterNode>>,
    pub parameters: Vec<ParameterNode>,
    pub return_type: Option<TypeNode>,
}

/// Construct signature
#[derive(Clone, Debug)]
pub struct ConstructSignature {
    pub span: Span,
    pub type_parameters: Option<Vec<TypeParameterNode>>,
    pub parameters: Vec<ParameterNode>,
    pub return_type: Option<TypeNode>,
}

/// Literal type node
#[derive(Clone, Debug)]
pub struct LiteralTypeNode {
    pub span: Span,
    pub literal: LiteralExpression,
}

/// Infer type node
#[derive(Clone, Debug)]
pub struct InferTypeNode {
    pub span: Span,
    pub type_parameter: TypeParameterNode,
}

/// Expression node
#[derive(Clone, Debug)]
pub enum Expression {
    Identifier(Identifier),
    StringLiteral(StringLiteral),
    NumericLiteral(NumericLiteral),
    BigIntLiteral(BigIntLiteral),
    BooleanLiteral(BooleanLiteral),
    NullLiteral(NullLiteral),
    Array(ArrayLiteralExpression),
    Object(ObjectLiteralExpression),
    PropertyAccess(PropertyAccessExpression),
    ElementAccess(ElementAccessExpression),
    Call(CallExpression),
    New(NewExpression),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Conditional(ConditionalExpression),
    Function(FunctionExpression),
    Arrow(ArrowFunction),
    Class(ClassExpression),
    Parenthesized(Box<Expression>),
    // ... more expression types
}

/// Literal expression (for type literals)
#[derive(Clone, Debug)]
pub enum LiteralExpression {
    String(StringLiteral),
    Numeric(NumericLiteral),
    BigInt(BigIntLiteral),
    Boolean(BooleanLiteral),
    Null(NullLiteral),
}

/// BigInt literal
#[derive(Clone, Debug)]
pub struct BigIntLiteral {
    pub span: Span,
    pub value: InternedString,
}

/// Boolean literal
#[derive(Clone, Debug)]
pub struct BooleanLiteral {
    pub span: Span,
    pub value: bool,
}

/// Null literal
#[derive(Clone, Debug)]
pub struct NullLiteral {
    pub span: Span,
}

/// Array literal expression
#[derive(Clone, Debug)]
pub struct ArrayLiteralExpression {
    pub span: Span,
    pub elements: Vec<Option<Expression>>,
}

/// Object literal expression
#[derive(Clone, Debug)]
pub struct ObjectLiteralExpression {
    pub span: Span,
    pub properties: Vec<ObjectLiteralElement>,
}

/// Object literal element
#[derive(Clone, Debug)]
pub enum ObjectLiteralElement {
    Property(PropertyAssignment),
    Shorthand(ShorthandPropertyAssignment),
    Spread(SpreadAssignment),
    Method(MethodDeclaration),
    GetAccessor(GetAccessorDeclaration),
    SetAccessor(SetAccessorDeclaration),
}

/// Property assignment
#[derive(Clone, Debug)]
pub struct PropertyAssignment {
    pub span: Span,
    pub name: PropertyName,
    pub initializer: Expression,
}

/// Shorthand property assignment
#[derive(Clone, Debug)]
pub struct ShorthandPropertyAssignment {
    pub span: Span,
    pub name: Identifier,
    pub object_assignment_initializer: Option<Expression>,
}

/// Spread assignment
#[derive(Clone, Debug)]
pub struct SpreadAssignment {
    pub span: Span,
    pub expression: Expression,
}

/// Property access expression
#[derive(Clone, Debug)]
pub struct PropertyAccessExpression {
    pub span: Span,
    pub expression: Box<Expression>,
    pub name: Identifier,
}

/// Element access expression
#[derive(Clone, Debug)]
pub struct ElementAccessExpression {
    pub span: Span,
    pub expression: Box<Expression>,
    pub argument: Box<Expression>,
}

/// Call expression
#[derive(Clone, Debug)]
pub struct CallExpression {
    pub span: Span,
    pub expression: Box<Expression>,
    pub type_arguments: Option<Vec<TypeNode>>,
    pub arguments: Vec<Expression>,
}

/// New expression
#[derive(Clone, Debug)]
pub struct NewExpression {
    pub span: Span,
    pub expression: Box<Expression>,
    pub type_arguments: Option<Vec<TypeNode>>,
    pub arguments: Option<Vec<Expression>>,
}

/// Binary expression
#[derive(Clone, Debug)]
pub struct BinaryExpression {
    pub span: Span,
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
}

/// Binary operator
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    Equals,
    NotEquals,
    StrictEquals,
    StrictNotEquals,
    LeftShift,
    RightShift,
    UnsignedRightShift,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LogicalAnd,
    LogicalOr,
    NullishCoalescing,
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,
    PowerAssign,
    LeftShiftAssign,
    RightShiftAssign,
    UnsignedRightShiftAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    LogicalAndAssign,
    LogicalOrAssign,
    NullishCoalescingAssign,
    Comma,
    In,
    InstanceOf,
}

/// Unary expression
#[derive(Clone, Debug)]
pub struct UnaryExpression {
    pub span: Span,
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
    pub is_prefix: bool,
}

/// Unary operator
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOperator {
    Plus,
    Minus,
    BitwiseNot,
    LogicalNot,
    Increment,
    Decrement,
    TypeOf,
    Void,
    Delete,
    Await,
}

/// Conditional expression
#[derive(Clone, Debug)]
pub struct ConditionalExpression {
    pub span: Span,
    pub condition: Box<Expression>,
    pub when_true: Box<Expression>,
    pub when_false: Box<Expression>,
}

/// Function expression
#[derive(Clone, Debug)]
pub struct FunctionExpression {
    pub span: Span,
    pub name: Option<Identifier>,
    pub type_parameters: Option<Vec<TypeParameterNode>>,
    pub parameters: Vec<ParameterNode>,
    pub return_type: Option<TypeNode>,
    pub body: BlockStatement,
    pub is_async: bool,
    pub is_generator: bool,
}

/// Arrow function
#[derive(Clone, Debug)]
pub struct ArrowFunction {
    pub span: Span,
    pub type_parameters: Option<Vec<TypeParameterNode>>,
    pub parameters: Vec<ParameterNode>,
    pub return_type: Option<TypeNode>,
    pub body: ArrowFunctionBody,
    pub is_async: bool,
}

/// Arrow function body
#[derive(Clone, Debug)]
pub enum ArrowFunctionBody {
    Expression(Box<Expression>),
    Block(BlockStatement),
}

/// Class expression
#[derive(Clone, Debug)]
pub struct ClassExpression {
    pub span: Span,
    pub name: Option<Identifier>,
    pub type_parameters: Option<Vec<TypeParameterNode>>,
    pub heritage_clauses: Vec<HeritageClause>,
    pub members: Vec<ClassMember>,
}

// ============================================================================
// Import/Export Declarations
// ============================================================================

/// Import declaration
/// Handles: import x from "mod", import { x } from "mod", import * as x from "mod", import "mod"
#[derive(Clone, Debug)]
pub struct ImportDeclaration {
    pub span: Span,
    /// The import clause (if any)
    pub import_clause: Option<ImportClause>,
    /// The module specifier (e.g., "./module" or "lodash")
    pub module_specifier: StringLiteral,
    /// Whether this is a type-only import (import type { ... })
    pub is_type_only: bool,
}

/// Import clause - the part between "import" and "from"
#[derive(Clone, Debug)]
pub struct ImportClause {
    pub span: Span,
    /// Default import binding (import X from "mod")
    pub name: Option<Identifier>,
    /// Named or namespace bindings
    pub named_bindings: Option<NamedImportBindings>,
    /// Whether this is a type-only import
    pub is_type_only: bool,
}

/// Named import bindings - either namespace import or named imports
#[derive(Clone, Debug)]
pub enum NamedImportBindings {
    /// Namespace import (import * as ns from "mod")
    Namespace(NamespaceImport),
    /// Named imports (import { a, b as c } from "mod")
    Named(NamedImports),
}

/// Namespace import (import * as ns from "mod")
#[derive(Clone, Debug)]
pub struct NamespaceImport {
    pub span: Span,
    /// The local binding name (ns in `import * as ns`)
    pub name: Identifier,
}

/// Named imports (import { a, b as c } from "mod")
#[derive(Clone, Debug)]
pub struct NamedImports {
    pub span: Span,
    /// The individual import specifiers
    pub elements: Vec<ImportSpecifier>,
}

/// An individual import specifier (a or b as c in `import { a, b as c }`)
#[derive(Clone, Debug)]
pub struct ImportSpecifier {
    pub span: Span,
    /// The exported name from the module (might be "default")
    pub property_name: Option<Identifier>,
    /// The local binding name
    pub name: Identifier,
    /// Whether this is a type-only import (import { type X })
    pub is_type_only: bool,
}

/// Export declaration
/// Handles: export { x }, export { x } from "mod", export * from "mod", export const/function/class
#[derive(Clone, Debug)]
pub struct ExportDeclaration {
    pub span: Span,
    /// The export clause (for re-exports and named exports)
    pub export_clause: Option<NamedExportBindings>,
    /// The module specifier (for re-exports: export { x } from "mod")
    pub module_specifier: Option<StringLiteral>,
    /// Whether this is a type-only export (export type { ... })
    pub is_type_only: bool,
    /// The declaration being exported (export const x = 1)
    pub declaration: Option<Box<ExportableDeclaration>>,
}

/// Named export bindings - either namespace export or named exports
#[derive(Clone, Debug)]
pub enum NamedExportBindings {
    /// Namespace export (export * from "mod" or export * as ns from "mod")
    Namespace(NamespaceExport),
    /// Named exports (export { a, b as c })
    Named(NamedExports),
}

/// Namespace export (export * as ns from "mod")
#[derive(Clone, Debug)]
pub struct NamespaceExport {
    pub span: Span,
    /// The export name (ns in `export * as ns`), None for bare `export *`
    pub name: Option<Identifier>,
}

/// Named exports (export { a, b as c })
#[derive(Clone, Debug)]
pub struct NamedExports {
    pub span: Span,
    /// The individual export specifiers
    pub elements: Vec<ExportSpecifier>,
}

/// An individual export specifier (a or b as c in `export { a, b as c }`)
#[derive(Clone, Debug)]
pub struct ExportSpecifier {
    pub span: Span,
    /// The local name (b in `export { b as c }`)
    pub property_name: Option<Identifier>,
    /// The exported name (c in `export { b as c }`, or a in `export { a }`)
    pub name: Identifier,
    /// Whether this is a type-only export (export { type X })
    pub is_type_only: bool,
}

/// Declarations that can be exported directly
#[derive(Clone, Debug)]
pub enum ExportableDeclaration {
    Variable(VariableStatement),
    Function(FunctionDeclaration),
    Class(ClassDeclaration),
    Interface(InterfaceDeclaration),
    TypeAlias(TypeAliasDeclaration),
    Enum(EnumDeclaration),
}

/// Export assignment (export = expr or export default expr)
#[derive(Clone, Debug)]
pub struct ExportAssignment {
    pub span: Span,
    /// The expression being exported
    pub expression: Expression,
    /// True for `export default`, false for `export =`
    pub is_export_equals: bool,
}

/// Interface declaration
#[derive(Clone, Debug)]
pub struct InterfaceDeclaration {
    pub span: Span,
    pub name: Identifier,
    pub type_parameters: Option<Vec<TypeParameterNode>>,
    pub heritage_clauses: Vec<HeritageClause>,
    pub members: Vec<TypeElement>,
}

/// Type alias declaration
#[derive(Clone, Debug)]
pub struct TypeAliasDeclaration {
    pub span: Span,
    pub name: Identifier,
    pub type_parameters: Option<Vec<TypeParameterNode>>,
    pub type_node: TypeNode,
}

/// Enum declaration
#[derive(Clone, Debug)]
pub struct EnumDeclaration {
    pub span: Span,
    pub name: Identifier,
    pub members: Vec<EnumMemberNode>,
    pub is_const: bool,
}

/// Enum member node
#[derive(Clone, Debug)]
pub struct EnumMemberNode {
    pub span: Span,
    pub name: PropertyName,
    pub initializer: Option<Expression>,
}

/// Module declaration (for module augmentation)
#[derive(Clone, Debug)]
pub struct ModuleDeclaration {
    pub span: Span,
    pub name: ModuleName,
    pub body: Option<ModuleBody>,
    pub modifiers: Modifiers,
}

/// Module name
#[derive(Clone, Debug)]
pub enum ModuleName {
    Identifier(Identifier),
    StringLiteral(StringLiteral),
}

/// Module body
#[derive(Clone, Debug)]
pub enum ModuleBody {
    Block(ModuleBlock),
    Declaration(Box<ModuleDeclaration>),
}

/// Module block
#[derive(Clone, Debug)]
pub struct ModuleBlock {
    pub span: Span,
    pub statements: Vec<Statement>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_kind_is_keyword() {
        assert!(SyntaxKind::FunctionKeyword.is_keyword());
        assert!(SyntaxKind::ClassKeyword.is_keyword());
        assert!(!SyntaxKind::Identifier.is_keyword());
    }

    #[test]
    fn test_syntax_kind_is_type_keyword() {
        assert!(SyntaxKind::StringKeyword.is_type_keyword());
        assert!(SyntaxKind::NumberKeyword.is_type_keyword());
        assert!(!SyntaxKind::FunctionKeyword.is_type_keyword());
    }

    #[test]
    fn test_modifiers() {
        let mods = Modifiers::PUBLIC | Modifiers::STATIC;
        assert!(mods.contains(Modifiers::PUBLIC));
        assert!(mods.contains(Modifiers::STATIC));
        assert!(!mods.contains(Modifiers::PRIVATE));
    }
}
