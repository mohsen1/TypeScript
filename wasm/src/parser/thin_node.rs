//! Thin Node Architecture for Cache-Efficient AST
//!
//! This module implements a cache-optimized AST representation where each node
//! is exactly 16 bytes (4 nodes per 64-byte cache line), compared to the
//! previous 208-byte Node enum (0.31 nodes per cache line).
//!
//! # Architecture
//!
//! Instead of a single large enum, we use:
//! 1. `ThinNode` - A 16-byte header containing kind, flags, position, and a data index
//! 2. Typed storage pools - Separate Vec<T> for each node category
//!
//! The `data_index` field points into the appropriate pool based on `kind`.
//!
//! # Performance Impact
//!
//! - **Before**: 208 bytes/node = 0.31 nodes/cache-line
//! - **After**: 16 bytes/node = 4 nodes/cache-line
//! - **Improvement**: 13x better cache locality for AST traversal
//!
//! # Design Principles
//!
//! 1. **Common data inline**: kind, flags, pos, end are accessed constantly
//! 2. **Rare data indirect**: modifiers, type parameters, etc. via index
//! 3. **No heap allocation per node**: All storage in arena vectors
//! 4. **O(1) node access**: Direct index into typed pool

use serde::Serialize;
use super::base::NodeIndex;
use super::ast::NodeList;

/// A thin 16-byte node header for cache-efficient AST storage.
///
/// Layout (16 bytes total):
/// - `kind`: 2 bytes (SyntaxKind value, supports 0-65535)
/// - `flags`: 2 bytes (packed NodeFlags)
/// - `pos`: 4 bytes (start position in source)
/// - `end`: 4 bytes (end position in source)
/// - `data_index`: 4 bytes (index into type-specific pool, u32::MAX = no data)
#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ThinNode {
    /// SyntaxKind value (u16 to support extended kinds up to 400+)
    pub kind: u16,
    /// Packed node flags (subset of NodeFlags that fits in u16)
    pub flags: u16,
    /// Start position in source (character index)
    pub pos: u32,
    /// End position in source (character index)
    pub end: u32,
    /// Index into the type-specific storage pool (u32::MAX = no data)
    pub data_index: u32,
}

impl ThinNode {
    pub const NO_DATA: u32 = u32::MAX;

    /// Create a new thin node with no associated data
    #[inline]
    pub fn new(kind: u16, pos: u32, end: u32) -> ThinNode {
        ThinNode {
            kind,
            flags: 0,
            pos,
            end,
            data_index: Self::NO_DATA,
        }
    }

    /// Create a new thin node with data index
    #[inline]
    pub fn with_data(kind: u16, pos: u32, end: u32, data_index: u32) -> ThinNode {
        ThinNode {
            kind,
            flags: 0,
            pos,
            end,
            data_index,
        }
    }

    /// Check if this node has associated data
    #[inline]
    pub fn has_data(&self) -> bool {
        self.data_index != Self::NO_DATA
    }
}

// =============================================================================
// Node Category Classification
// =============================================================================

/// Categories of nodes that share storage pools.
/// Nodes in the same category have similar data layouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeCategory {
    /// Simple tokens with no additional data (keywords, operators, etc.)
    Token,
    /// Identifiers with text data
    Identifier,
    /// String/numeric/regex literals with text
    Literal,
    /// Binary, unary, conditional expressions
    Expression,
    /// Function declarations and expressions
    Function,
    /// Class declarations
    Class,
    /// Statements (if, for, while, etc.)
    Statement,
    /// Type nodes (TypeReference, UnionType, etc.)
    TypeNode,
    /// Import/export declarations
    Module,
    /// JSX elements
    Jsx,
    /// Source file (only one per parse)
    SourceFile,
}

// =============================================================================
// Typed Data Pools
// =============================================================================

/// Data for identifier nodes (Identifier, PrivateIdentifier)
#[derive(Clone, Debug, Serialize)]
pub struct IdentifierData {
    pub escaped_text: String,
    pub original_text: Option<String>,
    pub type_arguments: Option<NodeList>,
}

/// Data for string literals (StringLiteral, template parts)
#[derive(Clone, Debug, Serialize)]
pub struct LiteralData {
    pub text: String,
    pub raw_text: Option<String>,
    /// For numeric literals only
    pub value: Option<f64>,
}

/// Data for binary expressions
#[derive(Clone, Debug, Serialize)]
pub struct BinaryExprData {
    pub left: NodeIndex,
    pub operator_token: u16,  // SyntaxKind
    pub right: NodeIndex,
}

/// Data for unary expressions (prefix/postfix)
#[derive(Clone, Debug, Serialize)]
pub struct UnaryExprData {
    pub operator: u16,  // SyntaxKind
    pub operand: NodeIndex,
}

/// Data for call/new expressions
#[derive(Clone, Debug, Serialize)]
pub struct CallExprData {
    pub expression: NodeIndex,
    pub type_arguments: Option<NodeList>,
    pub arguments: Option<NodeList>,
}

/// Data for property/element access
#[derive(Clone, Debug, Serialize)]
pub struct AccessExprData {
    pub expression: NodeIndex,
    pub name_or_argument: NodeIndex,
    pub question_dot_token: bool,
}

/// Data for function declarations/expressions/arrows
#[derive(Clone, Debug, Serialize)]
pub struct FunctionData {
    pub modifiers: Option<NodeList>,
    pub asterisk_token: bool,
    pub name: NodeIndex,
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub type_annotation: NodeIndex,
    pub body: NodeIndex,
    pub equals_greater_than_token: bool,  // For arrows
}

/// Data for class declarations
#[derive(Clone, Debug, Serialize)]
pub struct ClassData {
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub type_parameters: Option<NodeList>,
    pub heritage_clauses: Option<NodeList>,
    pub members: NodeList,
}

/// Data for if statements
#[derive(Clone, Debug, Serialize)]
pub struct IfStatementData {
    pub expression: NodeIndex,
    pub then_statement: NodeIndex,
    pub else_statement: NodeIndex,
}

/// Data for for/while/do loops
#[derive(Clone, Debug, Serialize)]
pub struct LoopData {
    pub initializer: NodeIndex,
    pub condition: NodeIndex,
    pub incrementor: NodeIndex,
    pub statement: NodeIndex,
}

/// Data for block statements
#[derive(Clone, Debug, Serialize)]
pub struct BlockData {
    pub statements: NodeList,
    pub multi_line: bool,
}

/// Data for variable declarations
#[derive(Clone, Debug, Serialize)]
pub struct VariableData {
    pub modifiers: Option<NodeList>,
    pub declarations: NodeList,
}

/// Data for type references
#[derive(Clone, Debug, Serialize)]
pub struct TypeRefData {
    pub type_name: NodeIndex,
    pub type_arguments: Option<NodeList>,
}

/// Data for union/intersection types
#[derive(Clone, Debug, Serialize)]
pub struct CompositeTypeData {
    pub types: NodeList,
}

/// Data for conditional expressions (a ? b : c)
#[derive(Clone, Debug, Serialize)]
pub struct ConditionalExprData {
    pub condition: NodeIndex,
    pub when_true: NodeIndex,
    pub when_false: NodeIndex,
}

/// Data for object/array literals
#[derive(Clone, Debug, Serialize)]
pub struct LiteralExprData {
    pub elements: NodeList,
    pub multi_line: bool,
}

/// Data for parenthesized expressions
#[derive(Clone, Debug, Serialize)]
pub struct ParenthesizedData {
    pub expression: NodeIndex,
}

/// Data for spread/await/yield expressions
#[derive(Clone, Debug, Serialize)]
pub struct UnaryExprDataEx {
    pub expression: NodeIndex,
    pub asterisk_token: bool,  // For yield*
}

/// Data for as/satisfies/type assertion expressions
#[derive(Clone, Debug, Serialize)]
pub struct TypeAssertionData {
    pub expression: NodeIndex,
    pub type_node: NodeIndex,
}

/// Data for return/throw statements
#[derive(Clone, Debug, Serialize)]
pub struct ReturnData {
    pub expression: NodeIndex,
}

/// Data for expression statements
#[derive(Clone, Debug, Serialize)]
pub struct ExprStatementData {
    pub expression: NodeIndex,
}

/// Data for switch statements
#[derive(Clone, Debug, Serialize)]
pub struct SwitchData {
    pub expression: NodeIndex,
    pub case_block: NodeIndex,
}

/// Data for case/default clauses
#[derive(Clone, Debug, Serialize)]
pub struct CaseClauseData {
    pub expression: NodeIndex,  // NONE for default clause
    pub statements: NodeList,
}

/// Data for try statements
#[derive(Clone, Debug, Serialize)]
pub struct TryData {
    pub try_block: NodeIndex,
    pub catch_clause: NodeIndex,
    pub finally_block: NodeIndex,
}

/// Data for catch clauses
#[derive(Clone, Debug, Serialize)]
pub struct CatchClauseData {
    pub variable_declaration: NodeIndex,
    pub block: NodeIndex,
}

/// Data for labeled statements
#[derive(Clone, Debug, Serialize)]
pub struct LabeledData {
    pub label: NodeIndex,
    pub statement: NodeIndex,
}

/// Data for break/continue statements
#[derive(Clone, Debug, Serialize)]
pub struct JumpData {
    pub label: NodeIndex,
}

/// Data for with statements
#[derive(Clone, Debug, Serialize)]
pub struct WithData {
    pub expression: NodeIndex,
    pub statement: NodeIndex,
}

/// Data for interface declarations
#[derive(Clone, Debug, Serialize)]
pub struct InterfaceData {
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub type_parameters: Option<NodeList>,
    pub heritage_clauses: Option<NodeList>,
    pub members: NodeList,
}

/// Data for type alias declarations
#[derive(Clone, Debug, Serialize)]
pub struct TypeAliasData {
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub type_parameters: Option<NodeList>,
    pub type_node: NodeIndex,
}

/// Data for enum declarations
#[derive(Clone, Debug, Serialize)]
pub struct EnumData {
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub members: NodeList,
}

/// Data for enum members
#[derive(Clone, Debug, Serialize)]
pub struct EnumMemberData {
    pub name: NodeIndex,
    pub initializer: NodeIndex,
}

/// Data for module/namespace declarations
#[derive(Clone, Debug, Serialize)]
pub struct ModuleData {
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub body: NodeIndex,
}

/// Data for property/method signatures
#[derive(Clone, Debug, Serialize)]
pub struct SignatureData {
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub question_token: bool,
    pub type_parameters: Option<NodeList>,
    pub parameters: Option<NodeList>,
    pub type_annotation: NodeIndex,
}

/// Data for index signatures
#[derive(Clone, Debug, Serialize)]
pub struct IndexSignatureData {
    pub modifiers: Option<NodeList>,
    pub parameters: NodeList,
    pub type_annotation: NodeIndex,
}

/// Data for property declarations
#[derive(Clone, Debug, Serialize)]
pub struct PropertyDeclData {
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub question_token: bool,
    pub exclamation_token: bool,
    pub type_annotation: NodeIndex,
    pub initializer: NodeIndex,
}

/// Data for method declarations (class methods)
#[derive(Clone, Debug, Serialize)]
pub struct MethodDeclData {
    pub modifiers: Option<NodeList>,
    pub asterisk_token: bool,
    pub name: NodeIndex,
    pub question_token: bool,
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub type_annotation: NodeIndex,
    pub body: NodeIndex,
}

/// Data for constructor declarations
#[derive(Clone, Debug, Serialize)]
pub struct ConstructorData {
    pub modifiers: Option<NodeList>,
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub body: NodeIndex,
}

/// Data for accessor declarations (get/set)
#[derive(Clone, Debug, Serialize)]
pub struct AccessorData {
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub type_annotation: NodeIndex,
    pub body: NodeIndex,
}

/// Data for parameter declarations
#[derive(Clone, Debug, Serialize)]
pub struct ParameterData {
    pub modifiers: Option<NodeList>,
    pub dot_dot_dot_token: bool,
    pub name: NodeIndex,
    pub question_token: bool,
    pub type_annotation: NodeIndex,
    pub initializer: NodeIndex,
}

/// Data for type parameter declarations
#[derive(Clone, Debug, Serialize)]
pub struct TypeParameterData {
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub constraint: NodeIndex,
    pub default: NodeIndex,
}

/// Data for decorator nodes
#[derive(Clone, Debug, Serialize)]
pub struct DecoratorData {
    pub expression: NodeIndex,
}

/// Data for heritage clauses
#[derive(Clone, Debug, Serialize)]
pub struct HeritageData {
    pub token: u16,  // ExtendsKeyword or ImplementsKeyword
    pub types: NodeList,
}

/// Data for expression with type arguments
#[derive(Clone, Debug, Serialize)]
pub struct ExprWithTypeArgsData {
    pub expression: NodeIndex,
    pub type_arguments: Option<NodeList>,
}

/// Data for import declarations
#[derive(Clone, Debug, Serialize)]
pub struct ImportDeclData {
    pub modifiers: Option<NodeList>,
    pub import_clause: NodeIndex,
    pub module_specifier: NodeIndex,
    pub attributes: NodeIndex,
}

/// Data for import clauses
#[derive(Clone, Debug, Serialize)]
pub struct ImportClauseData {
    pub is_type_only: bool,
    pub name: NodeIndex,
    pub named_bindings: NodeIndex,
}

/// Data for namespace/named imports
#[derive(Clone, Debug, Serialize)]
pub struct NamedImportsData {
    pub name: NodeIndex,       // For namespace import
    pub elements: NodeList,    // For named imports
}

/// Data for import/export specifiers
#[derive(Clone, Debug, Serialize)]
pub struct SpecifierData {
    pub is_type_only: bool,
    pub property_name: NodeIndex,
    pub name: NodeIndex,
}

/// Data for export declarations
#[derive(Clone, Debug, Serialize)]
pub struct ExportDeclData {
    pub modifiers: Option<NodeList>,
    pub is_type_only: bool,
    pub export_clause: NodeIndex,
    pub module_specifier: NodeIndex,
    pub attributes: NodeIndex,
}

/// Data for export assignments
#[derive(Clone, Debug, Serialize)]
pub struct ExportAssignmentData {
    pub modifiers: Option<NodeList>,
    pub is_export_equals: bool,
    pub expression: NodeIndex,
}

/// Data for import attributes
#[derive(Clone, Debug, Serialize)]
pub struct ImportAttributesData {
    pub token: u16,
    pub elements: NodeList,
    pub multi_line: bool,
}

/// Data for import attribute
#[derive(Clone, Debug, Serialize)]
pub struct ImportAttributeData {
    pub name: NodeIndex,
    pub value: NodeIndex,
}

/// Data for binding patterns
#[derive(Clone, Debug, Serialize)]
pub struct BindingPatternData {
    pub elements: NodeList,
}

/// Data for binding elements
#[derive(Clone, Debug, Serialize)]
pub struct BindingElementData {
    pub dot_dot_dot_token: bool,
    pub property_name: NodeIndex,
    pub name: NodeIndex,
    pub initializer: NodeIndex,
}

/// Data for property assignments
#[derive(Clone, Debug, Serialize)]
pub struct PropertyAssignmentData {
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub initializer: NodeIndex,
}

/// Data for shorthand property assignments
#[derive(Clone, Debug, Serialize)]
pub struct ShorthandPropertyData {
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub equals_token: bool,
    pub object_assignment_initializer: NodeIndex,
}

/// Data for spread assignments
#[derive(Clone, Debug, Serialize)]
pub struct SpreadData {
    pub expression: NodeIndex,
}

/// Data for template expressions
#[derive(Clone, Debug, Serialize)]
pub struct TemplateExprData {
    pub head: NodeIndex,
    pub template_spans: NodeList,
}

/// Data for template spans
#[derive(Clone, Debug, Serialize)]
pub struct TemplateSpanData {
    pub expression: NodeIndex,
    pub literal: NodeIndex,
}

/// Data for tagged template expressions
#[derive(Clone, Debug, Serialize)]
pub struct TaggedTemplateData {
    pub tag: NodeIndex,
    pub type_arguments: Option<NodeList>,
    pub template: NodeIndex,
}

/// Data for qualified names
#[derive(Clone, Debug, Serialize)]
pub struct QualifiedNameData {
    pub left: NodeIndex,
    pub right: NodeIndex,
}

/// Data for computed property names
#[derive(Clone, Debug, Serialize)]
pub struct ComputedPropertyData {
    pub expression: NodeIndex,
}

/// Data for type nodes (function type, constructor type)
#[derive(Clone, Debug, Serialize)]
pub struct FunctionTypeData {
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub type_annotation: NodeIndex,
}

/// Data for type query (typeof)
#[derive(Clone, Debug, Serialize)]
pub struct TypeQueryData {
    pub expr_name: NodeIndex,
    pub type_arguments: Option<NodeList>,
}

/// Data for type literal
#[derive(Clone, Debug, Serialize)]
pub struct TypeLiteralData {
    pub members: NodeList,
}

/// Data for array type
#[derive(Clone, Debug, Serialize)]
pub struct ArrayTypeData {
    pub element_type: NodeIndex,
}

/// Data for tuple type
#[derive(Clone, Debug, Serialize)]
pub struct TupleTypeData {
    pub elements: NodeList,
}

/// Data for optional/rest types
#[derive(Clone, Debug, Serialize)]
pub struct WrappedTypeData {
    pub type_node: NodeIndex,
}

/// Data for conditional types
#[derive(Clone, Debug, Serialize)]
pub struct ConditionalTypeData {
    pub check_type: NodeIndex,
    pub extends_type: NodeIndex,
    pub true_type: NodeIndex,
    pub false_type: NodeIndex,
}

/// Data for infer type
#[derive(Clone, Debug, Serialize)]
pub struct InferTypeData {
    pub type_parameter: NodeIndex,
}

/// Data for type operator (keyof, unique, readonly)
#[derive(Clone, Debug, Serialize)]
pub struct TypeOperatorData {
    pub operator: u16,
    pub type_node: NodeIndex,
}

/// Data for indexed access type
#[derive(Clone, Debug, Serialize)]
pub struct IndexedAccessTypeData {
    pub object_type: NodeIndex,
    pub index_type: NodeIndex,
}

/// Data for mapped type
#[derive(Clone, Debug, Serialize)]
pub struct MappedTypeData {
    pub readonly_token: NodeIndex,
    pub type_parameter: NodeIndex,
    pub name_type: NodeIndex,
    pub question_token: NodeIndex,
    pub type_node: NodeIndex,
    pub members: Option<NodeList>,
}

/// Data for literal types
#[derive(Clone, Debug, Serialize)]
pub struct LiteralTypeData {
    pub literal: NodeIndex,
}

/// Data for template literal types
#[derive(Clone, Debug, Serialize)]
pub struct TemplateLiteralTypeData {
    pub head: NodeIndex,
    pub template_spans: NodeList,
}

/// Data for named tuple member
#[derive(Clone, Debug, Serialize)]
pub struct NamedTupleMemberData {
    pub dot_dot_dot_token: bool,
    pub name: NodeIndex,
    pub question_token: bool,
    pub type_node: NodeIndex,
}

/// Data for type predicate
#[derive(Clone, Debug, Serialize)]
pub struct TypePredicateData {
    pub asserts_modifier: bool,
    pub parameter_name: NodeIndex,
    pub type_node: NodeIndex,
}

/// Data for JSX elements
#[derive(Clone, Debug, Serialize)]
pub struct JsxElementData {
    pub opening_element: NodeIndex,
    pub children: NodeList,
    pub closing_element: NodeIndex,
}

/// Data for JSX self-closing/opening elements
#[derive(Clone, Debug, Serialize)]
pub struct JsxOpeningData {
    pub tag_name: NodeIndex,
    pub type_arguments: Option<NodeList>,
    pub attributes: NodeIndex,
}

/// Data for JSX closing elements
#[derive(Clone, Debug, Serialize)]
pub struct JsxClosingData {
    pub tag_name: NodeIndex,
}

/// Data for JSX fragments
#[derive(Clone, Debug, Serialize)]
pub struct JsxFragmentData {
    pub opening_fragment: NodeIndex,
    pub children: NodeList,
    pub closing_fragment: NodeIndex,
}

/// Data for JSX attributes
#[derive(Clone, Debug, Serialize)]
pub struct JsxAttributesData {
    pub properties: NodeList,
}

/// Data for JSX attribute
#[derive(Clone, Debug, Serialize)]
pub struct JsxAttributeData {
    pub name: NodeIndex,
    pub initializer: NodeIndex,
}

/// Data for JSX spread attribute
#[derive(Clone, Debug, Serialize)]
pub struct JsxSpreadAttributeData {
    pub expression: NodeIndex,
}

/// Data for JSX expression
#[derive(Clone, Debug, Serialize)]
pub struct JsxExpressionData {
    pub dot_dot_dot_token: bool,
    pub expression: NodeIndex,
}

/// Data for JSX text
#[derive(Clone, Debug, Serialize)]
pub struct JsxTextData {
    pub text: String,
    pub contains_only_trivia_white_spaces: bool,
}

/// Data for JSX namespaced name
#[derive(Clone, Debug, Serialize)]
pub struct JsxNamespacedNameData {
    pub namespace: NodeIndex,
    pub name: NodeIndex,
}

/// Data for source files
#[derive(Clone, Debug, Serialize)]
pub struct SourceFileData {
    pub statements: NodeList,
    pub end_of_file_token: NodeIndex,
    pub file_name: String,
    pub text: String,
    pub language_version: u32,
    pub language_variant: u32,
    pub script_kind: u32,
    pub is_declaration_file: bool,
    pub has_no_default_lib: bool,
    pub identifiers: Vec<String>,
    // Extended node info (parent, id, modifiers, transform_flags)
    pub parent: NodeIndex,
    pub id: u32,
    pub modifier_flags: u32,
    pub transform_flags: u32,
}

// =============================================================================
// Thin Node Arena
// =============================================================================

/// Arena for thin nodes with typed data pools.
/// Provides O(1) allocation and cache-efficient storage.
#[derive(Debug, Default, Serialize)]
pub struct ThinNodeArena {
    /// The thin node headers (16 bytes each)
    pub nodes: Vec<ThinNode>,

    // Typed data pools
    pub identifiers: Vec<IdentifierData>,
    pub literals: Vec<LiteralData>,
    pub binary_exprs: Vec<BinaryExprData>,
    pub unary_exprs: Vec<UnaryExprData>,
    pub call_exprs: Vec<CallExprData>,
    pub access_exprs: Vec<AccessExprData>,
    pub functions: Vec<FunctionData>,
    pub classes: Vec<ClassData>,
    pub if_statements: Vec<IfStatementData>,
    pub loops: Vec<LoopData>,
    pub blocks: Vec<BlockData>,
    pub variables: Vec<VariableData>,
    pub type_refs: Vec<TypeRefData>,
    pub composite_types: Vec<CompositeTypeData>,
    pub source_files: Vec<SourceFileData>,

    // Extended node info (for nodes that need parent, id, full flags)
    pub extended_info: Vec<ExtendedNodeInfo>,
}

/// Extended node info for nodes that need more than what fits in ThinNode
#[derive(Clone, Debug, Default, Serialize)]
pub struct ExtendedNodeInfo {
    pub parent: NodeIndex,
    pub id: u32,
    pub modifier_flags: u32,
    pub transform_flags: u32,
}

impl ThinNodeArena {
    pub fn new() -> ThinNodeArena {
        ThinNodeArena::default()
    }

    pub fn with_capacity(capacity: usize) -> ThinNodeArena {
        ThinNodeArena {
            nodes: Vec::with_capacity(capacity),
            identifiers: Vec::with_capacity(capacity / 4),
            literals: Vec::with_capacity(capacity / 8),
            binary_exprs: Vec::with_capacity(capacity / 8),
            unary_exprs: Vec::with_capacity(capacity / 16),
            call_exprs: Vec::with_capacity(capacity / 8),
            access_exprs: Vec::with_capacity(capacity / 8),
            functions: Vec::with_capacity(capacity / 16),
            classes: Vec::with_capacity(capacity / 32),
            if_statements: Vec::with_capacity(capacity / 16),
            loops: Vec::with_capacity(capacity / 32),
            blocks: Vec::with_capacity(capacity / 8),
            variables: Vec::with_capacity(capacity / 16),
            type_refs: Vec::with_capacity(capacity / 8),
            composite_types: Vec::with_capacity(capacity / 16),
            source_files: Vec::with_capacity(1),
            extended_info: Vec::with_capacity(capacity),
        }
    }

    /// Add a token node (no additional data)
    pub fn add_token(&mut self, kind: u16, pos: u32, end: u32) -> NodeIndex {
        let index = self.nodes.len() as u32;
        self.nodes.push(ThinNode::new(kind, pos, end));
        self.extended_info.push(ExtendedNodeInfo::default());
        NodeIndex(index)
    }

    /// Add an identifier node
    pub fn add_identifier(&mut self, kind: u16, pos: u32, end: u32, data: IdentifierData) -> NodeIndex {
        let data_index = self.identifiers.len() as u32;
        self.identifiers.push(data);
        let index = self.nodes.len() as u32;
        self.nodes.push(ThinNode::with_data(kind, pos, end, data_index));
        self.extended_info.push(ExtendedNodeInfo::default());
        NodeIndex(index)
    }

    /// Add a literal node
    pub fn add_literal(&mut self, kind: u16, pos: u32, end: u32, data: LiteralData) -> NodeIndex {
        let data_index = self.literals.len() as u32;
        self.literals.push(data);
        let index = self.nodes.len() as u32;
        self.nodes.push(ThinNode::with_data(kind, pos, end, data_index));
        self.extended_info.push(ExtendedNodeInfo::default());
        NodeIndex(index)
    }

    /// Add a binary expression
    pub fn add_binary_expr(&mut self, kind: u16, pos: u32, end: u32, data: BinaryExprData) -> NodeIndex {
        let data_index = self.binary_exprs.len() as u32;
        self.binary_exprs.push(data);
        let index = self.nodes.len() as u32;
        self.nodes.push(ThinNode::with_data(kind, pos, end, data_index));
        self.extended_info.push(ExtendedNodeInfo::default());
        NodeIndex(index)
    }

    /// Add a call expression
    pub fn add_call_expr(&mut self, kind: u16, pos: u32, end: u32, data: CallExprData) -> NodeIndex {
        let data_index = self.call_exprs.len() as u32;
        self.call_exprs.push(data);
        let index = self.nodes.len() as u32;
        self.nodes.push(ThinNode::with_data(kind, pos, end, data_index));
        self.extended_info.push(ExtendedNodeInfo::default());
        NodeIndex(index)
    }

    /// Add a function node
    pub fn add_function(&mut self, kind: u16, pos: u32, end: u32, data: FunctionData) -> NodeIndex {
        let data_index = self.functions.len() as u32;
        self.functions.push(data);
        let index = self.nodes.len() as u32;
        self.nodes.push(ThinNode::with_data(kind, pos, end, data_index));
        self.extended_info.push(ExtendedNodeInfo::default());
        NodeIndex(index)
    }

    /// Add a class node
    pub fn add_class(&mut self, kind: u16, pos: u32, end: u32, data: ClassData) -> NodeIndex {
        let data_index = self.classes.len() as u32;
        self.classes.push(data);
        let index = self.nodes.len() as u32;
        self.nodes.push(ThinNode::with_data(kind, pos, end, data_index));
        self.extended_info.push(ExtendedNodeInfo::default());
        NodeIndex(index)
    }

    /// Add a block node
    pub fn add_block(&mut self, kind: u16, pos: u32, end: u32, data: BlockData) -> NodeIndex {
        let data_index = self.blocks.len() as u32;
        self.blocks.push(data);
        let index = self.nodes.len() as u32;
        self.nodes.push(ThinNode::with_data(kind, pos, end, data_index));
        self.extended_info.push(ExtendedNodeInfo::default());
        NodeIndex(index)
    }

    /// Add a source file node
    pub fn add_source_file(&mut self, pos: u32, end: u32, data: SourceFileData) -> NodeIndex {
        use super::syntax_kind_ext::SOURCE_FILE;
        let data_index = self.source_files.len() as u32;
        self.source_files.push(data);
        let index = self.nodes.len() as u32;
        self.nodes.push(ThinNode::with_data(SOURCE_FILE, pos, end, data_index));
        self.extended_info.push(ExtendedNodeInfo::default());
        NodeIndex(index)
    }

    /// Get a thin node by index
    #[inline]
    pub fn get(&self, index: NodeIndex) -> Option<&ThinNode> {
        if index.is_none() {
            None
        } else {
            self.nodes.get(index.0 as usize)
        }
    }

    /// Get a mutable thin node by index
    #[inline]
    pub fn get_mut(&mut self, index: NodeIndex) -> Option<&mut ThinNode> {
        if index.is_none() {
            None
        } else {
            self.nodes.get_mut(index.0 as usize)
        }
    }

    /// Get extended info for a node
    #[inline]
    pub fn get_extended(&self, index: NodeIndex) -> Option<&ExtendedNodeInfo> {
        if index.is_none() {
            None
        } else {
            self.extended_info.get(index.0 as usize)
        }
    }

    /// Get mutable extended info for a node
    #[inline]
    pub fn get_extended_mut(&mut self, index: NodeIndex) -> Option<&mut ExtendedNodeInfo> {
        if index.is_none() {
            None
        } else {
            self.extended_info.get_mut(index.0 as usize)
        }
    }

    /// Get identifier data for a node
    #[inline]
    pub fn get_identifier(&self, node: &ThinNode) -> Option<&IdentifierData> {
        if node.has_data() {
            self.identifiers.get(node.data_index as usize)
        } else {
            None
        }
    }

    /// Get literal data for a node
    #[inline]
    pub fn get_literal(&self, node: &ThinNode) -> Option<&LiteralData> {
        if node.has_data() {
            self.literals.get(node.data_index as usize)
        } else {
            None
        }
    }

    /// Get binary expression data
    #[inline]
    pub fn get_binary_expr(&self, node: &ThinNode) -> Option<&BinaryExprData> {
        if node.has_data() {
            self.binary_exprs.get(node.data_index as usize)
        } else {
            None
        }
    }

    /// Get call expression data
    #[inline]
    pub fn get_call_expr(&self, node: &ThinNode) -> Option<&CallExprData> {
        if node.has_data() {
            self.call_exprs.get(node.data_index as usize)
        } else {
            None
        }
    }

    /// Get function data
    #[inline]
    pub fn get_function(&self, node: &ThinNode) -> Option<&FunctionData> {
        if node.has_data() {
            self.functions.get(node.data_index as usize)
        } else {
            None
        }
    }

    /// Get class data
    #[inline]
    pub fn get_class(&self, node: &ThinNode) -> Option<&ClassData> {
        if node.has_data() {
            self.classes.get(node.data_index as usize)
        } else {
            None
        }
    }

    /// Get block data
    #[inline]
    pub fn get_block(&self, node: &ThinNode) -> Option<&BlockData> {
        if node.has_data() {
            self.blocks.get(node.data_index as usize)
        } else {
            None
        }
    }

    /// Get source file data
    #[inline]
    pub fn get_source_file(&self, node: &ThinNode) -> Option<&SourceFileData> {
        if node.has_data() {
            self.source_files.get(node.data_index as usize)
        } else {
            None
        }
    }

    /// Number of nodes in the arena
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if arena is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_thin_node_size() {
        // This is the critical test - ThinNode MUST be 16 bytes
        assert_eq!(size_of::<ThinNode>(), 16, "ThinNode must be exactly 16 bytes");

        // 4 nodes per cache line
        let nodes_per_cache_line = 64 / size_of::<ThinNode>();
        assert_eq!(nodes_per_cache_line, 4, "Should fit 4 ThinNodes per 64-byte cache line");
    }

    #[test]
    fn test_thin_node_arena_basic() {
        let mut arena = ThinNodeArena::new();

        // Add a token (no data)
        let token = arena.add_token(42, 0, 5);
        assert_eq!(token.0, 0);

        // Add an identifier
        let ident = arena.add_identifier(
            79,  // SyntaxKind::Identifier
            10,
            15,
            IdentifierData {
                escaped_text: "hello".to_string(),
                original_text: None,
                type_arguments: None,
            },
        );
        assert_eq!(ident.0, 1);

        // Verify we can retrieve them
        let node = arena.get(token).unwrap();
        assert_eq!(node.kind, 42);
        assert_eq!(node.pos, 0);
        assert_eq!(node.end, 5);
        assert!(!node.has_data());

        let node = arena.get(ident).unwrap();
        assert_eq!(node.kind, 79);
        assert!(node.has_data());

        let data = arena.get_identifier(node).unwrap();
        assert_eq!(data.escaped_text, "hello");
    }

    #[test]
    fn test_data_pool_sizes() {
        // Verify data pool element sizes are reasonable
        assert!(size_of::<IdentifierData>() <= 120, "IdentifierData too large");
        assert!(size_of::<FunctionData>() <= 168, "FunctionData too large");
        assert!(size_of::<ClassData>() <= 200, "ClassData too large");
        assert!(size_of::<SourceFileData>() <= 200, "SourceFileData too large");
    }
}
