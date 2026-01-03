//! Parser types - AST node definitions for TypeScript.
//!
//! This module defines the AST node types that match TypeScript's parser output.
//! The goal is to produce an identical AST structure that can be serialized and
//! consumed by the TypeScript type checker.
//!
//! DESIGN NOTES:
//! - We use arena allocation (indices) rather than Box/Rc for node references
//! - All nodes have common fields: kind, flags, pos, end
//! - Node-specific data is stored in enum variants
//! - This design allows efficient serialization to/from JavaScript

use wasm_bindgen::prelude::*;
use serde::Serialize;
use crate::scanner::SyntaxKind;

// =============================================================================
// Node Flags
// =============================================================================

/// Node flags indicating various properties of AST nodes.
/// Matches TypeScript's NodeFlags enum exactly.
/// NOTE: wasm_bindgen doesn't support bit-shift expressions, so these are
/// stored as a u32 bitfield in NodeBase. Use the constants below for flag operations.
pub mod node_flags {
    pub const NONE: u32               = 0;
    pub const LET: u32                = 1;       // 1 << 0
    pub const CONST: u32              = 2;       // 1 << 1
    pub const USING: u32              = 4;       // 1 << 2
    pub const AWAIT_USING: u32        = 6;       // Const | Using
    pub const NESTED_NAMESPACE: u32   = 8;       // 1 << 3
    pub const SYNTHESIZED: u32        = 16;      // 1 << 4
    pub const NAMESPACE: u32          = 32;      // 1 << 5
    pub const OPTIONAL_CHAIN: u32     = 64;      // 1 << 6
    pub const EXPORT_CONTEXT: u32     = 128;     // 1 << 7
    pub const CONTAINS_THIS: u32      = 256;     // 1 << 8
    pub const HAS_IMPLICIT_RETURN: u32 = 512;    // 1 << 9
    pub const HAS_EXPLICIT_RETURN: u32 = 1024;   // 1 << 10
    pub const GLOBAL_AUGMENTATION: u32 = 2048;   // 1 << 11
    pub const HAS_ASYNC_FUNCTIONS: u32 = 4096;   // 1 << 12
    pub const DISALLOW_IN_CONTEXT: u32 = 8192;   // 1 << 13
    pub const YIELD_CONTEXT: u32       = 16384;  // 1 << 14
    pub const DECORATOR_CONTEXT: u32   = 32768;  // 1 << 15
    pub const AWAIT_CONTEXT: u32       = 65536;  // 1 << 16
    pub const DISALLOW_CONDITIONAL_TYPES_CONTEXT: u32 = 131072; // 1 << 17
    pub const THIS_NODE_HAS_ERROR: u32 = 262144; // 1 << 18
    pub const JAVASCRIPT_FILE: u32     = 524288; // 1 << 19
    pub const THIS_NODE_OR_ANY_SUB_NODES_HAS_ERROR: u32 = 1048576; // 1 << 20
    pub const HAS_AGGREGATED_CHILD_DATA: u32 = 2097152; // 1 << 21
    pub const POSSIBLY_CONTAINS_DYNAMIC_IMPORT: u32 = 4194304; // 1 << 22
    pub const POSSIBLY_CONTAINS_IMPORT_META: u32 = 8388608; // 1 << 23
    pub const JSDOC: u32              = 16777216;  // 1 << 24
    pub const AMBIENT: u32            = 33554432;  // 1 << 25
    pub const IN_WITH_STATEMENT: u32  = 67108864;  // 1 << 26
    pub const JSON_FILE: u32          = 134217728; // 1 << 27
    pub const TYPE_CACHED: u32        = 268435456; // 1 << 28
    pub const DEPRECATED: u32         = 536870912; // 1 << 29

    // Type-only imports/exports
    pub const TYPE_ONLY: u32          = 1073741824; // 1 << 30
}

// =============================================================================
// Modifier Flags
// =============================================================================

/// Modifier flags for declarations.
/// Matches TypeScript's ModifierFlags enum exactly.
pub mod modifier_flags {
    pub const NONE: u32      = 0;

    // Syntactic/JSDoc modifiers
    pub const PUBLIC: u32    = 1;       // 1 << 0
    pub const PRIVATE: u32   = 2;       // 1 << 1
    pub const PROTECTED: u32 = 4;       // 1 << 2
    pub const READONLY: u32  = 8;       // 1 << 3
    pub const OVERRIDE: u32  = 16;      // 1 << 4

    // Syntactic-only modifiers
    pub const EXPORT: u32    = 32;      // 1 << 5
    pub const ABSTRACT: u32  = 64;      // 1 << 6
    pub const AMBIENT: u32   = 128;     // 1 << 7
    pub const STATIC: u32    = 256;     // 1 << 8
    pub const ACCESSOR: u32  = 512;     // 1 << 9
    pub const ASYNC: u32     = 1024;    // 1 << 10
    pub const DEFAULT: u32   = 2048;    // 1 << 11
    pub const CONST: u32     = 4096;    // 1 << 12
    pub const IN: u32        = 8192;    // 1 << 13
    pub const OUT: u32       = 16384;   // 1 << 14
    pub const DECORATOR: u32 = 32768;   // 1 << 15

    // JSDoc-only modifiers
    pub const DEPRECATED: u32 = 65536;  // 1 << 16
}

// =============================================================================
// Transform Flags
// =============================================================================

/// Transform flags indicate which transformations are needed for emit.
/// Matches TypeScript's TransformFlags enum.
pub mod transform_flags {
    pub const NONE: u32 = 0;

    // Facts about the node
    pub const CONTAINS_TYPESCRIPT: u32 = 1;               // 1 << 0
    pub const CONTAINS_JSX: u32 = 2;                      // 1 << 1
    pub const CONTAINS_ESNEXT: u32 = 4;                   // 1 << 2
    pub const CONTAINS_ES2022: u32 = 8;                   // 1 << 3
    pub const CONTAINS_ES2021: u32 = 16;                  // 1 << 4
    pub const CONTAINS_ES2020: u32 = 32;                  // 1 << 5
    pub const CONTAINS_ES2019: u32 = 64;                  // 1 << 6
    pub const CONTAINS_ES2018: u32 = 128;                 // 1 << 7
    pub const CONTAINS_ES2017: u32 = 256;                 // 1 << 8
    pub const CONTAINS_ES2016: u32 = 512;                 // 1 << 9
    pub const CONTAINS_ES2015: u32 = 1024;                // 1 << 10
    pub const CONTAINS_GENERATOR: u32 = 2048;             // 1 << 11
    pub const CONTAINS_DESTRUCTURING_ASSIGNMENT: u32 = 4096; // 1 << 12

    // Markers
    pub const CONTAINS_TYPESCRIPT_CLASS_SYNTAX: u32 = 8192; // 1 << 13
    pub const CONTAINS_LEXICAL_THIS: u32 = 16384;         // 1 << 14
    pub const CONTAINS_REST_OR_SPREAD: u32 = 32768;       // 1 << 15
    pub const CONTAINS_OBJECT_REST_OR_SPREAD: u32 = 65536; // 1 << 16
    pub const CONTAINS_COMPUTED_PROPERTY_NAME: u32 = 131072; // 1 << 17
    pub const CONTAINS_BLOCK_SCOPED_BINDING: u32 = 262144; // 1 << 18
    pub const CONTAINS_BINDING_PATTERN: u32 = 524288;     // 1 << 19
    pub const CONTAINS_YIELD: u32 = 1048576;              // 1 << 20
    pub const CONTAINS_AWAIT: u32 = 2097152;              // 1 << 21
    pub const CONTAINS_HOISTED_DECLARATION_OR_COMPLETION: u32 = 4194304; // 1 << 22
    pub const CONTAINS_DYNAMIC_IMPORT: u32 = 8388608;     // 1 << 23
    pub const CONTAINS_CLASS_FIELDS: u32 = 16777216;      // 1 << 24
    pub const CONTAINS_DECORATORS: u32 = 33554432;        // 1 << 25
    pub const CONTAINS_POSSIBLE_TOP_LEVEL_AWAIT: u32 = 67108864; // 1 << 26
    pub const CONTAINS_LEXICAL_SUPER: u32 = 134217728;    // 1 << 27
    pub const CONTAINS_UPDATE_EXPRESSION_FOR_IDENTIFIER: u32 = 268435456; // 1 << 28
    pub const CONTAINS_PRIVATE_IDENTIFIER_IN_EXPRESSION: u32 = 536870912; // 1 << 29

    pub const HAS_COMPUTED_FLAGS: u32 = 2147483648;       // 1 << 31
}

// =============================================================================
// Extended SyntaxKind Constants (for node kinds not in scanner)
// =============================================================================

/// Extended SyntaxKind values for AST nodes that are not tokens.
/// These match TypeScript's SyntaxKind enum values exactly.
pub mod syntax_kind_ext {
    // First AST node kinds (after tokens, starting at 167)
    pub const QUALIFIED_NAME: u16 = 167;
    pub const COMPUTED_PROPERTY_NAME: u16 = 168;
    pub const TYPE_PARAMETER: u16 = 169;
    pub const PARAMETER: u16 = 170;
    pub const DECORATOR: u16 = 171;
    pub const PROPERTY_SIGNATURE: u16 = 172;
    pub const PROPERTY_DECLARATION: u16 = 173;
    pub const METHOD_SIGNATURE: u16 = 174;
    pub const METHOD_DECLARATION: u16 = 175;
    pub const CLASS_STATIC_BLOCK_DECLARATION: u16 = 176;
    pub const CONSTRUCTOR: u16 = 177;
    pub const GET_ACCESSOR: u16 = 178;
    pub const SET_ACCESSOR: u16 = 179;
    pub const CALL_SIGNATURE: u16 = 180;
    pub const CONSTRUCT_SIGNATURE: u16 = 181;
    pub const INDEX_SIGNATURE: u16 = 182;

    // Type nodes
    pub const TYPE_PREDICATE: u16 = 183;
    pub const TYPE_REFERENCE: u16 = 184;
    pub const FUNCTION_TYPE: u16 = 185;
    pub const CONSTRUCTOR_TYPE: u16 = 186;
    pub const TYPE_QUERY: u16 = 187;
    pub const TYPE_LITERAL: u16 = 188;
    pub const ARRAY_TYPE: u16 = 189;
    pub const TUPLE_TYPE: u16 = 190;
    pub const OPTIONAL_TYPE: u16 = 191;
    pub const REST_TYPE: u16 = 192;
    pub const UNION_TYPE: u16 = 193;
    pub const INTERSECTION_TYPE: u16 = 194;
    pub const CONDITIONAL_TYPE: u16 = 195;
    pub const INFER_TYPE: u16 = 196;
    pub const PARENTHESIZED_TYPE: u16 = 197;
    pub const THIS_TYPE: u16 = 198;
    pub const TYPE_OPERATOR: u16 = 199;
    pub const INDEXED_ACCESS_TYPE: u16 = 200;
    pub const MAPPED_TYPE: u16 = 201;
    pub const LITERAL_TYPE: u16 = 202;
    pub const NAMED_TUPLE_MEMBER: u16 = 203;
    pub const TEMPLATE_LITERAL_TYPE: u16 = 204;
    pub const TEMPLATE_LITERAL_TYPE_SPAN: u16 = 205;
    pub const IMPORT_TYPE: u16 = 206;

    // Binding patterns
    pub const OBJECT_BINDING_PATTERN: u16 = 207;
    pub const ARRAY_BINDING_PATTERN: u16 = 208;
    pub const BINDING_ELEMENT: u16 = 209;

    // Expression
    pub const ARRAY_LITERAL_EXPRESSION: u16 = 210;
    pub const OBJECT_LITERAL_EXPRESSION: u16 = 211;
    pub const PROPERTY_ACCESS_EXPRESSION: u16 = 212;
    pub const ELEMENT_ACCESS_EXPRESSION: u16 = 213;
    pub const CALL_EXPRESSION: u16 = 214;
    pub const NEW_EXPRESSION: u16 = 215;
    pub const TAGGED_TEMPLATE_EXPRESSION: u16 = 216;
    pub const TYPE_ASSERTION: u16 = 217;
    pub const PARENTHESIZED_EXPRESSION: u16 = 218;
    pub const FUNCTION_EXPRESSION: u16 = 219;
    pub const ARROW_FUNCTION: u16 = 220;
    pub const DELETE_EXPRESSION: u16 = 221;
    pub const TYPE_OF_EXPRESSION: u16 = 222;
    pub const VOID_EXPRESSION: u16 = 223;
    pub const AWAIT_EXPRESSION: u16 = 224;
    pub const PREFIX_UNARY_EXPRESSION: u16 = 225;
    pub const POSTFIX_UNARY_EXPRESSION: u16 = 226;
    pub const BINARY_EXPRESSION: u16 = 227;
    pub const CONDITIONAL_EXPRESSION: u16 = 228;
    pub const TEMPLATE_EXPRESSION: u16 = 229;
    pub const YIELD_EXPRESSION: u16 = 230;
    pub const SPREAD_ELEMENT: u16 = 231;
    pub const CLASS_EXPRESSION: u16 = 232;
    pub const OMITTED_EXPRESSION: u16 = 233;
    pub const EXPRESSION_WITH_TYPE_ARGUMENTS: u16 = 234;
    pub const AS_EXPRESSION: u16 = 235;
    pub const NON_NULL_EXPRESSION: u16 = 236;
    pub const META_PROPERTY: u16 = 237;
    pub const SYNTHETIC_EXPRESSION: u16 = 238;
    pub const SATISFIES_EXPRESSION: u16 = 239;

    // Misc
    pub const TEMPLATE_SPAN: u16 = 240;
    pub const SEMICOLON_CLASS_ELEMENT: u16 = 241;

    // Statements
    pub const BLOCK: u16 = 242;
    pub const EMPTY_STATEMENT: u16 = 243;
    pub const VARIABLE_STATEMENT: u16 = 244;
    pub const EXPRESSION_STATEMENT: u16 = 245;
    pub const IF_STATEMENT: u16 = 246;
    pub const DO_STATEMENT: u16 = 247;
    pub const WHILE_STATEMENT: u16 = 248;
    pub const FOR_STATEMENT: u16 = 249;
    pub const FOR_IN_STATEMENT: u16 = 250;
    pub const FOR_OF_STATEMENT: u16 = 251;
    pub const CONTINUE_STATEMENT: u16 = 252;
    pub const BREAK_STATEMENT: u16 = 253;
    pub const RETURN_STATEMENT: u16 = 254;
    pub const WITH_STATEMENT: u16 = 255;
    pub const SWITCH_STATEMENT: u16 = 256;
    pub const LABELED_STATEMENT: u16 = 257;
    pub const THROW_STATEMENT: u16 = 258;
    pub const TRY_STATEMENT: u16 = 259;
    pub const DEBUGGER_STATEMENT: u16 = 260;

    // Declarations
    pub const VARIABLE_DECLARATION: u16 = 261;
    pub const VARIABLE_DECLARATION_LIST: u16 = 262;
    pub const FUNCTION_DECLARATION: u16 = 263;
    pub const CLASS_DECLARATION: u16 = 264;
    pub const INTERFACE_DECLARATION: u16 = 265;
    pub const TYPE_ALIAS_DECLARATION: u16 = 266;
    pub const ENUM_DECLARATION: u16 = 267;
    pub const MODULE_DECLARATION: u16 = 268;
    pub const MODULE_BLOCK: u16 = 269;
    pub const CASE_BLOCK: u16 = 270;
    pub const NAMESPACE_EXPORT_DECLARATION: u16 = 271;
    pub const IMPORT_EQUALS_DECLARATION: u16 = 272;
    pub const IMPORT_DECLARATION: u16 = 273;
    pub const IMPORT_CLAUSE: u16 = 274;
    pub const NAMESPACE_IMPORT: u16 = 275;
    pub const NAMED_IMPORTS: u16 = 276;
    pub const IMPORT_SPECIFIER: u16 = 277;
    pub const EXPORT_ASSIGNMENT: u16 = 278;
    pub const EXPORT_DECLARATION: u16 = 279;
    pub const NAMED_EXPORTS: u16 = 280;
    pub const NAMESPACE_EXPORT: u16 = 281;
    pub const EXPORT_SPECIFIER: u16 = 282;
    pub const MISSING_DECLARATION: u16 = 283;

    // Module references
    pub const EXTERNAL_MODULE_REFERENCE: u16 = 284;

    // JSX
    pub const JSX_ELEMENT: u16 = 285;
    pub const JSX_SELF_CLOSING_ELEMENT: u16 = 286;
    pub const JSX_OPENING_ELEMENT: u16 = 287;
    pub const JSX_CLOSING_ELEMENT: u16 = 288;
    pub const JSX_FRAGMENT: u16 = 289;
    pub const JSX_OPENING_FRAGMENT: u16 = 290;
    pub const JSX_CLOSING_FRAGMENT: u16 = 291;
    pub const JSX_ATTRIBUTE: u16 = 292;
    pub const JSX_ATTRIBUTES: u16 = 293;
    pub const JSX_SPREAD_ATTRIBUTE: u16 = 294;
    pub const JSX_EXPRESSION: u16 = 295;
    pub const JSX_NAMESPACED_NAME: u16 = 296;

    // Clauses
    pub const CASE_CLAUSE: u16 = 297;
    pub const DEFAULT_CLAUSE: u16 = 298;
    pub const HERITAGE_CLAUSE: u16 = 299;
    pub const CATCH_CLAUSE: u16 = 300;
    pub const IMPORT_ATTRIBUTES: u16 = 301;
    pub const IMPORT_ATTRIBUTE: u16 = 302;

    // Property assignments
    pub const PROPERTY_ASSIGNMENT: u16 = 303;
    pub const SHORTHAND_PROPERTY_ASSIGNMENT: u16 = 304;
    pub const SPREAD_ASSIGNMENT: u16 = 305;

    // Enum
    pub const ENUM_MEMBER: u16 = 306;

    // Unparsed (for incremental)
    pub const UNPARSED_PROLOGUE: u16 = 307;

    // Top-level nodes
    pub const SOURCE_FILE: u16 = 308;
    pub const BUNDLE: u16 = 309;

    // First JSDoc node (310) ... we'll add these as needed
}

// =============================================================================
// Text Range
// =============================================================================

/// A text range with start and end positions.
/// All positions are character indices (not byte indices).
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct TextRange {
    pub pos: u32,  // Start position
    pub end: u32,  // End position
}

#[wasm_bindgen]
impl TextRange {
    #[wasm_bindgen(constructor)]
    pub fn new(pos: u32, end: u32) -> TextRange {
        TextRange { pos, end }
    }
}

// =============================================================================
// Node Index (Arena-based Reference)
// =============================================================================

/// Index into the node arena. Used instead of pointers/references
/// for efficient serialization and memory management.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Hash)]
pub struct NodeIndex(pub u32);

impl NodeIndex {
    pub const NONE: NodeIndex = NodeIndex(u32::MAX);

    pub fn is_none(&self) -> bool {
        self.0 == u32::MAX
    }

    pub fn is_some(&self) -> bool {
        self.0 != u32::MAX
    }
}

// =============================================================================
// Node List (Children)
// =============================================================================

/// A list of node indices, representing children or a node array.
#[derive(Clone, Debug, Default, Serialize)]
pub struct NodeList {
    pub nodes: Vec<NodeIndex>,
    pub pos: u32,
    pub end: u32,
    pub has_trailing_comma: bool,
}

impl NodeList {
    pub fn new() -> NodeList {
        NodeList {
            nodes: Vec::new(),
            pos: 0,
            end: 0,
            has_trailing_comma: false,
        }
    }

    pub fn with_capacity(capacity: usize) -> NodeList {
        NodeList {
            nodes: Vec::with_capacity(capacity),
            pos: 0,
            end: 0,
            has_trailing_comma: false,
        }
    }

    pub fn push(&mut self, node: NodeIndex) {
        self.nodes.push(node);
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

// =============================================================================
// Base Node (Common fields for all nodes)
// =============================================================================

/// Common fields present in all AST nodes.
/// Note: `kind` is stored as u16 to support both token kinds (from SyntaxKind enum)
/// and extended node kinds (from syntax_kind_ext constants).
#[derive(Clone, Debug, Serialize)]
pub struct NodeBase {
    pub kind: u16,              // SyntaxKind value (u16 to support extended kinds)
    pub flags: u32,             // NodeFlags
    pub modifier_flags: u32,    // ModifierFlags (cached)
    pub transform_flags: u32,   // TransformFlags
    pub pos: u32,               // Start position (character index)
    pub end: u32,               // End position (character index)
    pub parent: NodeIndex,      // Parent node index
    pub id: u32,                // Unique node ID (assigned by binder)
}

impl Default for NodeBase {
    fn default() -> Self {
        NodeBase {
            kind: SyntaxKind::Unknown as u16,
            flags: 0,
            modifier_flags: 0,
            transform_flags: 0,
            pos: 0,
            end: 0,
            parent: NodeIndex::NONE,
            id: 0,
        }
    }
}

impl NodeBase {
    /// Create a new NodeBase with a SyntaxKind (token kind)
    pub fn new(kind: SyntaxKind, pos: u32, end: u32) -> NodeBase {
        NodeBase {
            kind: kind as u16,
            flags: 0,
            modifier_flags: 0,
            transform_flags: 0,
            pos,
            end,
            parent: NodeIndex::NONE,
            id: 0,
        }
    }

    /// Create a new NodeBase with an extended kind (for node types not in scanner)
    pub fn new_ext(kind: u16, pos: u32, end: u32) -> NodeBase {
        NodeBase {
            kind,
            flags: 0,
            modifier_flags: 0,
            transform_flags: 0,
            pos,
            end,
            parent: NodeIndex::NONE,
            id: 0,
        }
    }
}

// =============================================================================
// Identifier
// =============================================================================

/// An identifier node.
#[derive(Clone, Debug, Serialize)]
pub struct Identifier {
    pub base: NodeBase,
    /// The escaped text of the identifier (with unicode escapes processed)
    pub escaped_text: String,
    /// Original text as it appeared in source (for emit)
    pub original_text: Option<String>,
    /// Type arguments (for JSX intrinsic elements)
    pub type_arguments: Option<NodeList>,
}

impl Identifier {
    pub fn new(escaped_text: String, pos: u32, end: u32) -> Identifier {
        Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, pos, end),
            escaped_text,
            original_text: None,
            type_arguments: None,
        }
    }
}

// =============================================================================
// Literals
// =============================================================================

/// A string literal node.
#[derive(Clone, Debug, Serialize)]
pub struct StringLiteral {
    pub base: NodeBase,
    pub text: String,
    pub is_unterminated: bool,
    pub has_extended_unicode_escape: bool,
}

/// A numeric literal node.
#[derive(Clone, Debug, Serialize)]
pub struct NumericLiteral {
    pub base: NodeBase,
    pub text: String,
    /// The numeric value (parsed from text)
    pub value: f64,
}

/// A BigInt literal node.
#[derive(Clone, Debug, Serialize)]
pub struct BigIntLiteral {
    pub base: NodeBase,
    pub text: String,
}

/// A regular expression literal node.
#[derive(Clone, Debug, Serialize)]
pub struct RegularExpressionLiteral {
    pub base: NodeBase,
    pub text: String,
}

/// A template literal span (part of a template expression).
#[derive(Clone, Debug, Serialize)]
pub struct TemplateSpan {
    pub base: NodeBase,
    pub expression: NodeIndex,
    pub literal: NodeIndex,  // TemplateMiddle or TemplateTail
}

// =============================================================================
// Source File
// =============================================================================

/// The root node representing a source file.
#[derive(Clone, Debug, Serialize)]
pub struct SourceFile {
    pub base: NodeBase,
    pub statements: NodeList,
    pub end_of_file_token: NodeIndex,
    pub file_name: String,
    pub text: String,
    pub language_version: u32,
    pub language_variant: u32,
    pub script_kind: u32,
    pub is_declaration_file: bool,
    pub has_no_default_lib: bool,
    /// Identifiers in the file (for binding)
    pub identifiers: Vec<String>,
}

impl SourceFile {
    pub fn new(file_name: String, text: String) -> SourceFile {
        let len = text.len() as u32;
        SourceFile {
            base: NodeBase::new_ext(syntax_kind_ext::SOURCE_FILE, 0, len),
            statements: NodeList::new(),
            end_of_file_token: NodeIndex::NONE,
            file_name,
            text,
            language_version: 0,
            language_variant: 0,
            script_kind: 0,
            is_declaration_file: false,
            has_no_default_lib: false,
            identifiers: Vec::new(),
        }
    }
}

// =============================================================================
// Statements
// =============================================================================

/// Variable declaration kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum VariableDeclarationKind {
    Var,
    Let,
    Const,
    Using,
    AwaitUsing,
}

/// A variable statement (var/let/const declarations).
#[derive(Clone, Debug, Serialize)]
pub struct VariableStatement {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub declaration_list: NodeIndex,  // VariableDeclarationList
}

/// A variable declaration list.
#[derive(Clone, Debug, Serialize)]
pub struct VariableDeclarationList {
    pub base: NodeBase,
    pub declarations: NodeList,  // VariableDeclaration[]
}

/// A single variable declaration.
#[derive(Clone, Debug, Serialize)]
pub struct VariableDeclaration {
    pub base: NodeBase,
    pub name: NodeIndex,            // Identifier or BindingPattern
    pub exclamation_token: bool,    // Definite assignment assertion
    pub type_annotation: NodeIndex, // TypeNode (optional)
    pub initializer: NodeIndex,     // Expression (optional)
}

/// An expression statement.
#[derive(Clone, Debug, Serialize)]
pub struct ExpressionStatement {
    pub base: NodeBase,
    pub expression: NodeIndex,
}

/// An if statement.
#[derive(Clone, Debug, Serialize)]
pub struct IfStatement {
    pub base: NodeBase,
    pub expression: NodeIndex,
    pub then_statement: NodeIndex,
    pub else_statement: NodeIndex,  // Optional
}

/// A return statement.
#[derive(Clone, Debug, Serialize)]
pub struct ReturnStatement {
    pub base: NodeBase,
    pub expression: NodeIndex,  // Optional
}

/// A block statement.
#[derive(Clone, Debug, Serialize)]
pub struct Block {
    pub base: NodeBase,
    pub statements: NodeList,
    pub multi_line: bool,
}

/// A function declaration.
#[derive(Clone, Debug, Serialize)]
pub struct FunctionDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub asterisk_token: bool,  // Generator function
    pub name: NodeIndex,       // Identifier (optional for default exports)
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub type_annotation: NodeIndex,  // Return type (optional)
    pub body: NodeIndex,       // Block (optional for overloads)
}

/// A class declaration.
#[derive(Clone, Debug, Serialize)]
pub struct ClassDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,       // Identifier (optional for default exports)
    pub type_parameters: Option<NodeList>,
    pub heritage_clauses: Option<NodeList>,
    pub members: NodeList,
}

// =============================================================================
// Expressions
// =============================================================================

/// A binary expression (a + b, a = b, etc.).
#[derive(Clone, Debug, Serialize)]
pub struct BinaryExpression {
    pub base: NodeBase,
    pub left: NodeIndex,
    pub operator_token: SyntaxKind,
    pub right: NodeIndex,
}

/// A prefix unary expression (!x, ++x, etc.).
#[derive(Clone, Debug, Serialize)]
pub struct PrefixUnaryExpression {
    pub base: NodeBase,
    pub operator: SyntaxKind,
    pub operand: NodeIndex,
}

/// A postfix unary expression (x++, x--).
#[derive(Clone, Debug, Serialize)]
pub struct PostfixUnaryExpression {
    pub base: NodeBase,
    pub operand: NodeIndex,
    pub operator: SyntaxKind,
}

/// A call expression (fn()).
#[derive(Clone, Debug, Serialize)]
pub struct CallExpression {
    pub base: NodeBase,
    pub expression: NodeIndex,
    pub type_arguments: Option<NodeList>,
    pub arguments: NodeList,
}

/// A property access expression (obj.prop).
#[derive(Clone, Debug, Serialize)]
pub struct PropertyAccessExpression {
    pub base: NodeBase,
    pub expression: NodeIndex,
    pub question_dot_token: bool,  // Optional chaining
    pub name: NodeIndex,           // Identifier or PrivateIdentifier
}

/// An element access expression (arr[idx]).
#[derive(Clone, Debug, Serialize)]
pub struct ElementAccessExpression {
    pub base: NodeBase,
    pub expression: NodeIndex,
    pub question_dot_token: bool,  // Optional chaining
    pub argument_expression: NodeIndex,
}

/// A conditional expression (a ? b : c).
#[derive(Clone, Debug, Serialize)]
pub struct ConditionalExpression {
    pub base: NodeBase,
    pub condition: NodeIndex,
    pub when_true: NodeIndex,
    pub when_false: NodeIndex,
}

/// An arrow function expression.
#[derive(Clone, Debug, Serialize)]
pub struct ArrowFunction {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub type_annotation: NodeIndex,  // Return type (optional)
    pub equals_greater_than_token: bool,
    pub body: NodeIndex,  // Block or Expression
}

/// A function expression.
#[derive(Clone, Debug, Serialize)]
pub struct FunctionExpression {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub asterisk_token: bool,
    pub name: NodeIndex,  // Optional
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub type_annotation: NodeIndex,
    pub body: NodeIndex,
}

/// An object literal expression.
#[derive(Clone, Debug, Serialize)]
pub struct ObjectLiteralExpression {
    pub base: NodeBase,
    pub properties: NodeList,
    pub multi_line: bool,
}

/// An array literal expression.
#[derive(Clone, Debug, Serialize)]
pub struct ArrayLiteralExpression {
    pub base: NodeBase,
    pub elements: NodeList,
    pub multi_line: bool,
}

/// A parenthesized expression.
#[derive(Clone, Debug, Serialize)]
pub struct ParenthesizedExpression {
    pub base: NodeBase,
    pub expression: NodeIndex,
}

/// A new expression (new Foo()).
#[derive(Clone, Debug, Serialize)]
pub struct NewExpression {
    pub base: NodeBase,
    pub expression: NodeIndex,
    pub type_arguments: Option<NodeList>,
    pub arguments: Option<NodeList>,
}

/// A tagged template expression (`tag`template``).
#[derive(Clone, Debug, Serialize)]
pub struct TaggedTemplateExpression {
    pub base: NodeBase,
    pub tag: NodeIndex,
    pub type_arguments: Option<NodeList>,
    pub template: NodeIndex,  // TemplateLiteral
}

/// A template expression (`hello ${world}`).
#[derive(Clone, Debug, Serialize)]
pub struct TemplateExpression {
    pub base: NodeBase,
    pub head: NodeIndex,     // TemplateHead
    pub template_spans: NodeList,  // TemplateSpan[]
}

/// A yield expression (yield x).
#[derive(Clone, Debug, Serialize)]
pub struct YieldExpression {
    pub base: NodeBase,
    pub asterisk_token: bool,
    pub expression: NodeIndex,  // Optional
}

/// An await expression (await x).
#[derive(Clone, Debug, Serialize)]
pub struct AwaitExpression {
    pub base: NodeBase,
    pub expression: NodeIndex,
}

/// A spread element (...x).
#[derive(Clone, Debug, Serialize)]
pub struct SpreadElement {
    pub base: NodeBase,
    pub expression: NodeIndex,
}

/// An as expression (x as Type).
#[derive(Clone, Debug, Serialize)]
pub struct AsExpression {
    pub base: NodeBase,
    pub expression: NodeIndex,
    pub type_node: NodeIndex,
}

/// A satisfies expression (x satisfies Type).
#[derive(Clone, Debug, Serialize)]
pub struct SatisfiesExpression {
    pub base: NodeBase,
    pub expression: NodeIndex,
    pub type_node: NodeIndex,
}

/// A non-null expression (x!).
#[derive(Clone, Debug, Serialize)]
pub struct NonNullExpression {
    pub base: NodeBase,
    pub expression: NodeIndex,
}

/// A type assertion (<Type>x).
#[derive(Clone, Debug, Serialize)]
pub struct TypeAssertion {
    pub base: NodeBase,
    pub type_node: NodeIndex,
    pub expression: NodeIndex,
}

// =============================================================================
// Import/Export Declarations
// =============================================================================

/// An import declaration.
#[derive(Clone, Debug, Serialize)]
pub struct ImportDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub import_clause: NodeIndex,  // Optional
    pub module_specifier: NodeIndex,  // StringLiteral
    pub attributes: NodeIndex,  // ImportAttributes (optional)
}

/// Import clause (the part between 'import' and 'from').
#[derive(Clone, Debug, Serialize)]
pub struct ImportClause {
    pub base: NodeBase,
    pub is_type_only: bool,
    pub name: NodeIndex,  // Identifier (optional - default import)
    pub named_bindings: NodeIndex,  // NamespaceImport or NamedImports (optional)
}

/// Namespace import (* as name).
#[derive(Clone, Debug, Serialize)]
pub struct NamespaceImport {
    pub base: NodeBase,
    pub name: NodeIndex,  // Identifier
}

/// Named imports ({ a, b as c }).
#[derive(Clone, Debug, Serialize)]
pub struct NamedImports {
    pub base: NodeBase,
    pub elements: NodeList,  // ImportSpecifier[]
}

/// A single import specifier (a or a as b).
#[derive(Clone, Debug, Serialize)]
pub struct ImportSpecifier {
    pub base: NodeBase,
    pub is_type_only: bool,
    pub property_name: NodeIndex,  // Optional (when using 'as')
    pub name: NodeIndex,  // Identifier
}

/// An export declaration.
#[derive(Clone, Debug, Serialize)]
pub struct ExportDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub is_type_only: bool,
    pub export_clause: NodeIndex,  // NamedExports or NamespaceExport (optional)
    pub module_specifier: NodeIndex,  // StringLiteral (optional)
    pub attributes: NodeIndex,  // ImportAttributes (optional)
}

/// Named exports ({ a, b as c }).
#[derive(Clone, Debug, Serialize)]
pub struct NamedExports {
    pub base: NodeBase,
    pub elements: NodeList,  // ExportSpecifier[]
}

/// Namespace export (* as name).
#[derive(Clone, Debug, Serialize)]
pub struct NamespaceExport {
    pub base: NodeBase,
    pub name: NodeIndex,  // Identifier
}

/// A single export specifier (a or a as b).
#[derive(Clone, Debug, Serialize)]
pub struct ExportSpecifier {
    pub base: NodeBase,
    pub is_type_only: bool,
    pub property_name: NodeIndex,  // Optional (when using 'as')
    pub name: NodeIndex,  // Identifier
}

/// An export assignment (export = x or export default x).
#[derive(Clone, Debug, Serialize)]
pub struct ExportAssignment {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub is_export_equals: bool,  // true for 'export =', false for 'export default'
    pub expression: NodeIndex,
}

/// Import attributes ({ with: { type: "json" } }).
#[derive(Clone, Debug, Serialize)]
pub struct ImportAttributes {
    pub base: NodeBase,
    pub token: u16,  // WithKeyword or AssertKeyword
    pub elements: NodeList,  // ImportAttribute[]
    pub multi_line: bool,
}

/// A single import attribute.
#[derive(Clone, Debug, Serialize)]
pub struct ImportAttribute {
    pub base: NodeBase,
    pub name: NodeIndex,  // Identifier or StringLiteral
    pub value: NodeIndex,  // Expression
}

// =============================================================================
// Type Nodes
// =============================================================================

/// A type reference (Foo, Foo<T>).
#[derive(Clone, Debug, Serialize)]
pub struct TypeReference {
    pub base: NodeBase,
    pub type_name: NodeIndex,  // Identifier or QualifiedName
    pub type_arguments: Option<NodeList>,
}

/// A function type ((x: number) => string).
#[derive(Clone, Debug, Serialize)]
pub struct FunctionType {
    pub base: NodeBase,
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub type_node: NodeIndex,  // Return type
}

/// A constructor type (new (x: number) => Foo).
#[derive(Clone, Debug, Serialize)]
pub struct ConstructorType {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub type_node: NodeIndex,
}

/// A type query (typeof x).
#[derive(Clone, Debug, Serialize)]
pub struct TypeQuery {
    pub base: NodeBase,
    pub expr_name: NodeIndex,
    pub type_arguments: Option<NodeList>,
}

/// A type literal ({ x: number }).
#[derive(Clone, Debug, Serialize)]
pub struct TypeLiteral {
    pub base: NodeBase,
    pub members: NodeList,
}

/// An array type (number[]).
#[derive(Clone, Debug, Serialize)]
pub struct ArrayType {
    pub base: NodeBase,
    pub element_type: NodeIndex,
}

/// A tuple type ([number, string]).
#[derive(Clone, Debug, Serialize)]
pub struct TupleType {
    pub base: NodeBase,
    pub elements: NodeList,
}

/// An optional type (T?).
#[derive(Clone, Debug, Serialize)]
pub struct OptionalType {
    pub base: NodeBase,
    pub type_node: NodeIndex,
}

/// A rest type (...T).
#[derive(Clone, Debug, Serialize)]
pub struct RestType {
    pub base: NodeBase,
    pub type_node: NodeIndex,
}

/// A union type (A | B).
#[derive(Clone, Debug, Serialize)]
pub struct UnionType {
    pub base: NodeBase,
    pub types: NodeList,
}

/// An intersection type (A & B).
#[derive(Clone, Debug, Serialize)]
pub struct IntersectionType {
    pub base: NodeBase,
    pub types: NodeList,
}

/// A conditional type (T extends U ? X : Y).
#[derive(Clone, Debug, Serialize)]
pub struct ConditionalType {
    pub base: NodeBase,
    pub check_type: NodeIndex,
    pub extends_type: NodeIndex,
    pub true_type: NodeIndex,
    pub false_type: NodeIndex,
}

/// An infer type (infer T).
#[derive(Clone, Debug, Serialize)]
pub struct InferType {
    pub base: NodeBase,
    pub type_parameter: NodeIndex,
}

/// A parenthesized type ((T)).
#[derive(Clone, Debug, Serialize)]
pub struct ParenthesizedType {
    pub base: NodeBase,
    pub type_node: NodeIndex,
}

/// A type operator (keyof T, unique symbol, readonly T).
#[derive(Clone, Debug, Serialize)]
pub struct TypeOperator {
    pub base: NodeBase,
    pub operator: u16,  // KeyOfKeyword, UniqueKeyword, ReadonlyKeyword
    pub type_node: NodeIndex,
}

/// An indexed access type (T[K]).
#[derive(Clone, Debug, Serialize)]
pub struct IndexedAccessType {
    pub base: NodeBase,
    pub object_type: NodeIndex,
    pub index_type: NodeIndex,
}

/// A mapped type ({ [K in T]: U }).
#[derive(Clone, Debug, Serialize)]
pub struct MappedType {
    pub base: NodeBase,
    pub readonly_token: Option<u16>,  // ReadonlyKeyword, PlusToken, MinusToken
    pub type_parameter: NodeIndex,
    pub name_type: NodeIndex,  // Optional
    pub question_token: Option<u16>,
    pub type_node: NodeIndex,  // Optional
    pub members: Option<NodeList>,
}

/// A literal type ("foo", 42, true).
#[derive(Clone, Debug, Serialize)]
pub struct LiteralType {
    pub base: NodeBase,
    pub literal: NodeIndex,
}

/// A template literal type (`${T}`).
#[derive(Clone, Debug, Serialize)]
pub struct TemplateLiteralType {
    pub base: NodeBase,
    pub head: NodeIndex,
    pub template_spans: NodeList,
}

/// A named tuple member (name: Type or name?: Type).
#[derive(Clone, Debug, Serialize)]
pub struct NamedTupleMember {
    pub base: NodeBase,
    pub dot_dot_dot_token: bool,
    pub name: NodeIndex,
    pub question_token: bool,
    pub type_node: NodeIndex,
}

// =============================================================================
// More Statements
// =============================================================================

/// A while statement.
#[derive(Clone, Debug, Serialize)]
pub struct WhileStatement {
    pub base: NodeBase,
    pub expression: NodeIndex,
    pub statement: NodeIndex,
}

/// A do-while statement.
#[derive(Clone, Debug, Serialize)]
pub struct DoStatement {
    pub base: NodeBase,
    pub statement: NodeIndex,
    pub expression: NodeIndex,
}

/// A for statement.
#[derive(Clone, Debug, Serialize)]
pub struct ForStatement {
    pub base: NodeBase,
    pub initializer: NodeIndex,  // Optional
    pub condition: NodeIndex,    // Optional
    pub incrementor: NodeIndex,  // Optional
    pub statement: NodeIndex,
}

/// A for-in statement.
#[derive(Clone, Debug, Serialize)]
pub struct ForInStatement {
    pub base: NodeBase,
    pub initializer: NodeIndex,
    pub expression: NodeIndex,
    pub statement: NodeIndex,
}

/// A for-of statement.
#[derive(Clone, Debug, Serialize)]
pub struct ForOfStatement {
    pub base: NodeBase,
    pub await_modifier: bool,
    pub initializer: NodeIndex,
    pub expression: NodeIndex,
    pub statement: NodeIndex,
}

/// A switch statement.
#[derive(Clone, Debug, Serialize)]
pub struct SwitchStatement {
    pub base: NodeBase,
    pub expression: NodeIndex,
    pub case_block: NodeIndex,
}

/// A case block (the part with case/default clauses).
#[derive(Clone, Debug, Serialize)]
pub struct CaseBlock {
    pub base: NodeBase,
    pub clauses: NodeList,
}

/// A case clause.
#[derive(Clone, Debug, Serialize)]
pub struct CaseClause {
    pub base: NodeBase,
    pub expression: NodeIndex,
    pub statements: NodeList,
}

/// A default clause.
#[derive(Clone, Debug, Serialize)]
pub struct DefaultClause {
    pub base: NodeBase,
    pub statements: NodeList,
}

/// A throw statement.
#[derive(Clone, Debug, Serialize)]
pub struct ThrowStatement {
    pub base: NodeBase,
    pub expression: NodeIndex,
}

/// A try statement.
#[derive(Clone, Debug, Serialize)]
pub struct TryStatement {
    pub base: NodeBase,
    pub try_block: NodeIndex,
    pub catch_clause: NodeIndex,  // Optional
    pub finally_block: NodeIndex,  // Optional
}

/// A catch clause.
#[derive(Clone, Debug, Serialize)]
pub struct CatchClause {
    pub base: NodeBase,
    pub variable_declaration: NodeIndex,  // Optional
    pub block: NodeIndex,
}

/// A labeled statement.
#[derive(Clone, Debug, Serialize)]
pub struct LabeledStatement {
    pub base: NodeBase,
    pub label: NodeIndex,
    pub statement: NodeIndex,
}

/// A break statement.
#[derive(Clone, Debug, Serialize)]
pub struct BreakStatement {
    pub base: NodeBase,
    pub label: NodeIndex,  // Optional
}

/// A continue statement.
#[derive(Clone, Debug, Serialize)]
pub struct ContinueStatement {
    pub base: NodeBase,
    pub label: NodeIndex,  // Optional
}

/// A with statement.
#[derive(Clone, Debug, Serialize)]
pub struct WithStatement {
    pub base: NodeBase,
    pub expression: NodeIndex,
    pub statement: NodeIndex,
}

/// A debugger statement.
#[derive(Clone, Debug, Serialize)]
pub struct DebuggerStatement {
    pub base: NodeBase,
}

/// An empty statement (;).
#[derive(Clone, Debug, Serialize)]
pub struct EmptyStatement {
    pub base: NodeBase,
}

// =============================================================================
// More Declarations
// =============================================================================

/// An interface declaration.
#[derive(Clone, Debug, Serialize)]
pub struct InterfaceDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub type_parameters: Option<NodeList>,
    pub heritage_clauses: Option<NodeList>,
    pub members: NodeList,
}

/// A property signature (in interface or type literal).
#[derive(Clone, Debug, Serialize)]
pub struct PropertySignature {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub question_token: bool,
    pub type_annotation: NodeIndex,  // Optional
    pub initializer: NodeIndex,      // Optional
}

/// A method signature (in interface or type literal).
#[derive(Clone, Debug, Serialize)]
pub struct MethodSignature {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub question_token: bool,
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub type_annotation: NodeIndex,  // Optional
}

/// An index signature declaration (e.g., [key: string]: number)
#[derive(Clone, Debug, Serialize)]
pub struct IndexSignatureDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub parameters: NodeList,         // The index parameter(s)
    pub type_annotation: NodeIndex,   // The value type
}

/// A type alias declaration.
#[derive(Clone, Debug, Serialize)]
pub struct TypeAliasDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub type_parameters: Option<NodeList>,
    pub type_node: NodeIndex,
}

/// An enum declaration.
#[derive(Clone, Debug, Serialize)]
pub struct EnumDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub members: NodeList,
}

/// An enum member.
#[derive(Clone, Debug, Serialize)]
pub struct EnumMember {
    pub base: NodeBase,
    pub name: NodeIndex,
    pub initializer: NodeIndex,  // Optional
}

/// A module/namespace declaration.
#[derive(Clone, Debug, Serialize)]
pub struct ModuleDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,  // Identifier or StringLiteral
    pub body: NodeIndex,  // ModuleBlock or ModuleDeclaration
}

/// A module block (the { } body of a module).
#[derive(Clone, Debug, Serialize)]
pub struct ModuleBlock {
    pub base: NodeBase,
    pub statements: NodeList,
}

// =============================================================================
// Class Members
// =============================================================================

/// A property declaration in a class.
#[derive(Clone, Debug, Serialize)]
pub struct PropertyDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub question_token: bool,
    pub exclamation_token: bool,
    pub type_annotation: NodeIndex,
    pub initializer: NodeIndex,
}

/// A method declaration.
#[derive(Clone, Debug, Serialize)]
pub struct MethodDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub asterisk_token: bool,
    pub name: NodeIndex,
    pub question_token: bool,
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub type_annotation: NodeIndex,
    pub body: NodeIndex,
}

/// A constructor declaration.
#[derive(Clone, Debug, Serialize)]
pub struct ConstructorDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub body: NodeIndex,
}

/// A get accessor declaration.
#[derive(Clone, Debug, Serialize)]
pub struct GetAccessorDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub type_annotation: NodeIndex,
    pub body: NodeIndex,
}

/// A set accessor declaration.
#[derive(Clone, Debug, Serialize)]
pub struct SetAccessorDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub type_parameters: Option<NodeList>,
    pub parameters: NodeList,
    pub body: NodeIndex,
}

/// A parameter declaration.
#[derive(Clone, Debug, Serialize)]
pub struct ParameterDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub dot_dot_dot_token: bool,
    pub name: NodeIndex,
    pub question_token: bool,
    pub type_annotation: NodeIndex,
    pub initializer: NodeIndex,
}

/// A type parameter declaration.
#[derive(Clone, Debug, Serialize)]
pub struct TypeParameterDeclaration {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,  // in/out variance modifiers
    pub name: NodeIndex,
    pub constraint: NodeIndex,  // Optional
    pub default: NodeIndex,     // Optional
}

/// A decorator.
#[derive(Clone, Debug, Serialize)]
pub struct Decorator {
    pub base: NodeBase,
    pub expression: NodeIndex,
}

/// A heritage clause (extends/implements).
#[derive(Clone, Debug, Serialize)]
pub struct HeritageClause {
    pub base: NodeBase,
    pub token: u16,  // ExtendsKeyword or ImplementsKeyword
    pub types: NodeList,
}

/// Expression with type arguments (used in heritage clauses).
#[derive(Clone, Debug, Serialize)]
pub struct ExpressionWithTypeArguments {
    pub base: NodeBase,
    pub expression: NodeIndex,
    pub type_arguments: Option<NodeList>,
}

// =============================================================================
// Binding Patterns
// =============================================================================

/// An object binding pattern ({ a, b }).
#[derive(Clone, Debug, Serialize)]
pub struct ObjectBindingPattern {
    pub base: NodeBase,
    pub elements: NodeList,
}

/// An array binding pattern ([a, b]).
#[derive(Clone, Debug, Serialize)]
pub struct ArrayBindingPattern {
    pub base: NodeBase,
    pub elements: NodeList,
}

/// A binding element (a or a = default or ...rest).
#[derive(Clone, Debug, Serialize)]
pub struct BindingElement {
    pub base: NodeBase,
    pub dot_dot_dot_token: bool,
    pub property_name: NodeIndex,  // Optional
    pub name: NodeIndex,
    pub initializer: NodeIndex,  // Optional
}

// =============================================================================
// Object Literal Members
// =============================================================================

/// A property assignment (a: value).
#[derive(Clone, Debug, Serialize)]
pub struct PropertyAssignment {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub initializer: NodeIndex,
}

/// A shorthand property assignment (a).
#[derive(Clone, Debug, Serialize)]
pub struct ShorthandPropertyAssignment {
    pub base: NodeBase,
    pub modifiers: Option<NodeList>,
    pub name: NodeIndex,
    pub equals_token: bool,
    pub object_assignment_initializer: NodeIndex,  // Optional
}

/// A spread assignment (...x).
#[derive(Clone, Debug, Serialize)]
pub struct SpreadAssignment {
    pub base: NodeBase,
    pub expression: NodeIndex,
}

// =============================================================================
// JSX Nodes
// =============================================================================

/// A JSX element (<Foo>children</Foo>).
#[derive(Clone, Debug, Serialize)]
pub struct JsxElement {
    pub base: NodeBase,
    pub opening_element: NodeIndex,  // JsxOpeningElement
    pub children: NodeList,          // JsxChild[]
    pub closing_element: NodeIndex,  // JsxClosingElement
}

/// A JSX self-closing element (<Foo />).
#[derive(Clone, Debug, Serialize)]
pub struct JsxSelfClosingElement {
    pub base: NodeBase,
    pub tag_name: NodeIndex,         // JsxTagNameExpression
    pub type_arguments: Option<NodeList>,
    pub attributes: NodeIndex,       // JsxAttributes
}

/// A JSX opening element (<Foo attr="value">).
#[derive(Clone, Debug, Serialize)]
pub struct JsxOpeningElement {
    pub base: NodeBase,
    pub tag_name: NodeIndex,         // JsxTagNameExpression
    pub type_arguments: Option<NodeList>,
    pub attributes: NodeIndex,       // JsxAttributes
}

/// A JSX closing element (</Foo>).
#[derive(Clone, Debug, Serialize)]
pub struct JsxClosingElement {
    pub base: NodeBase,
    pub tag_name: NodeIndex,         // JsxTagNameExpression
}

/// A JSX fragment (<>children</>).
#[derive(Clone, Debug, Serialize)]
pub struct JsxFragment {
    pub base: NodeBase,
    pub opening_fragment: NodeIndex,  // JsxOpeningFragment
    pub children: NodeList,           // JsxChild[]
    pub closing_fragment: NodeIndex,  // JsxClosingFragment
}

/// A JSX opening fragment (<>).
#[derive(Clone, Debug, Serialize)]
pub struct JsxOpeningFragment {
    pub base: NodeBase,
}

/// A JSX closing fragment (</>).
#[derive(Clone, Debug, Serialize)]
pub struct JsxClosingFragment {
    pub base: NodeBase,
}

/// JSX attributes container ({ className: "foo" }).
#[derive(Clone, Debug, Serialize)]
pub struct JsxAttributes {
    pub base: NodeBase,
    pub properties: NodeList,  // JsxAttributeLike[]
}

/// A JSX attribute (name="value" or name={expr}).
#[derive(Clone, Debug, Serialize)]
pub struct JsxAttribute {
    pub base: NodeBase,
    pub name: NodeIndex,         // Identifier or JsxNamespacedName
    pub initializer: NodeIndex,  // StringLiteral or JsxExpression (optional)
}

/// A JSX spread attribute ({...props}).
#[derive(Clone, Debug, Serialize)]
pub struct JsxSpreadAttribute {
    pub base: NodeBase,
    pub expression: NodeIndex,
}

/// A JSX expression container ({expression}).
#[derive(Clone, Debug, Serialize)]
pub struct JsxExpression {
    pub base: NodeBase,
    pub dot_dot_dot_token: bool,  // For spread in expression position
    pub expression: NodeIndex,    // Expression (optional)
}

/// A JSX text node (plain text between tags).
#[derive(Clone, Debug, Serialize)]
pub struct JsxText {
    pub base: NodeBase,
    pub text: String,
    pub contains_only_trivia_white_spaces: bool,
}

/// A JSX namespaced name (ns:name).
#[derive(Clone, Debug, Serialize)]
pub struct JsxNamespacedName {
    pub base: NodeBase,
    pub namespace: NodeIndex,  // Identifier
    pub name: NodeIndex,       // Identifier
}

// =============================================================================
// Node Enum (All possible node types)
// =============================================================================

/// The main AST node enum containing all possible node types.
/// Uses enum variants to store node-specific data while sharing common fields.
#[derive(Clone, Debug, Serialize)]
pub enum Node {
    // Tokens (no additional data needed, just use SyntaxKind)
    Token(NodeBase),

    // Names
    Identifier(Identifier),
    PrivateIdentifier(Identifier),
    QualifiedName { base: NodeBase, left: NodeIndex, right: NodeIndex },
    ComputedPropertyName { base: NodeBase, expression: NodeIndex },

    // Literals
    StringLiteral(StringLiteral),
    NumericLiteral(NumericLiteral),
    BigIntLiteral(BigIntLiteral),
    RegularExpressionLiteral(RegularExpressionLiteral),
    NoSubstitutionTemplateLiteral(StringLiteral),
    TemplateHead(StringLiteral),
    TemplateMiddle(StringLiteral),
    TemplateTail(StringLiteral),

    // Expressions
    BinaryExpression(BinaryExpression),
    PrefixUnaryExpression(PrefixUnaryExpression),
    PostfixUnaryExpression(PostfixUnaryExpression),
    CallExpression(CallExpression),
    NewExpression(NewExpression),
    TaggedTemplateExpression(TaggedTemplateExpression),
    TemplateExpression(TemplateExpression),
    PropertyAccessExpression(PropertyAccessExpression),
    ElementAccessExpression(ElementAccessExpression),
    ConditionalExpression(ConditionalExpression),
    ArrowFunction(ArrowFunction),
    FunctionExpression(FunctionExpression),
    ObjectLiteralExpression(ObjectLiteralExpression),
    ArrayLiteralExpression(ArrayLiteralExpression),
    ParenthesizedExpression(ParenthesizedExpression),
    YieldExpression(YieldExpression),
    AwaitExpression(AwaitExpression),
    SpreadElement(SpreadElement),
    AsExpression(AsExpression),
    SatisfiesExpression(SatisfiesExpression),
    NonNullExpression(NonNullExpression),
    TypeAssertion(TypeAssertion),

    // Statements
    VariableStatement(VariableStatement),
    VariableDeclarationList(VariableDeclarationList),
    VariableDeclaration(VariableDeclaration),
    ExpressionStatement(ExpressionStatement),
    IfStatement(IfStatement),
    WhileStatement(WhileStatement),
    DoStatement(DoStatement),
    ForStatement(ForStatement),
    ForInStatement(ForInStatement),
    ForOfStatement(ForOfStatement),
    SwitchStatement(SwitchStatement),
    CaseBlock(CaseBlock),
    CaseClause(CaseClause),
    DefaultClause(DefaultClause),
    ReturnStatement(ReturnStatement),
    ThrowStatement(ThrowStatement),
    TryStatement(TryStatement),
    CatchClause(CatchClause),
    LabeledStatement(LabeledStatement),
    BreakStatement(BreakStatement),
    ContinueStatement(ContinueStatement),
    WithStatement(WithStatement),
    DebuggerStatement(DebuggerStatement),
    EmptyStatement(EmptyStatement),
    Block(Block),

    // Declarations
    FunctionDeclaration(FunctionDeclaration),
    ClassDeclaration(ClassDeclaration),
    InterfaceDeclaration(InterfaceDeclaration),
    PropertySignature(PropertySignature),
    MethodSignature(MethodSignature),
    IndexSignatureDeclaration(IndexSignatureDeclaration),
    TypeAliasDeclaration(TypeAliasDeclaration),
    EnumDeclaration(EnumDeclaration),
    EnumMember(EnumMember),
    ModuleDeclaration(ModuleDeclaration),
    ModuleBlock(ModuleBlock),

    // Import/Export
    ImportDeclaration(ImportDeclaration),
    ImportClause(ImportClause),
    NamespaceImport(NamespaceImport),
    NamedImports(NamedImports),
    ImportSpecifier(ImportSpecifier),
    ExportDeclaration(ExportDeclaration),
    NamedExports(NamedExports),
    NamespaceExport(NamespaceExport),
    ExportSpecifier(ExportSpecifier),
    ExportAssignment(ExportAssignment),
    ImportAttributes(ImportAttributes),
    ImportAttribute(ImportAttribute),

    // Type nodes
    TypeReference(TypeReference),
    FunctionType(FunctionType),
    ConstructorType(ConstructorType),
    TypeQuery(TypeQuery),
    TypeLiteral(TypeLiteral),
    ArrayType(ArrayType),
    TupleType(TupleType),
    OptionalType(OptionalType),
    RestType(RestType),
    UnionType(UnionType),
    IntersectionType(IntersectionType),
    ConditionalType(ConditionalType),
    InferType(InferType),
    ParenthesizedType(ParenthesizedType),
    TypeOperator(TypeOperator),
    IndexedAccessType(IndexedAccessType),
    MappedType(MappedType),
    LiteralType(LiteralType),
    TemplateLiteralType(TemplateLiteralType),
    NamedTupleMember(NamedTupleMember),

    // Class members
    PropertyDeclaration(PropertyDeclaration),
    MethodDeclaration(MethodDeclaration),
    ConstructorDeclaration(ConstructorDeclaration),
    GetAccessorDeclaration(GetAccessorDeclaration),
    SetAccessorDeclaration(SetAccessorDeclaration),
    ParameterDeclaration(ParameterDeclaration),
    TypeParameterDeclaration(TypeParameterDeclaration),
    Decorator(Decorator),
    HeritageClause(HeritageClause),
    ExpressionWithTypeArguments(ExpressionWithTypeArguments),

    // Binding patterns
    ObjectBindingPattern(ObjectBindingPattern),
    ArrayBindingPattern(ArrayBindingPattern),
    BindingElement(BindingElement),

    // Object literal members
    PropertyAssignment(PropertyAssignment),
    ShorthandPropertyAssignment(ShorthandPropertyAssignment),
    SpreadAssignment(SpreadAssignment),

    // JSX nodes
    JsxElement(JsxElement),
    JsxSelfClosingElement(JsxSelfClosingElement),
    JsxOpeningElement(JsxOpeningElement),
    JsxClosingElement(JsxClosingElement),
    JsxFragment(JsxFragment),
    JsxOpeningFragment(JsxOpeningFragment),
    JsxClosingFragment(JsxClosingFragment),
    JsxAttributes(JsxAttributes),
    JsxAttribute(JsxAttribute),
    JsxSpreadAttribute(JsxSpreadAttribute),
    JsxExpression(JsxExpression),
    JsxText(JsxText),
    JsxNamespacedName(JsxNamespacedName),

    // Misc
    TemplateSpan(TemplateSpan),

    // Source file
    SourceFile(SourceFile),

    // End of file token
    EndOfFileToken(NodeBase),
}

impl Node {
    /// Get the base node data (kind, flags, pos, end, etc.)
    pub fn base(&self) -> &NodeBase {
        match self {
            Node::Token(base) | Node::EndOfFileToken(base) => base,
            Node::Identifier(n) | Node::PrivateIdentifier(n) => &n.base,
            Node::QualifiedName { base, .. } | Node::ComputedPropertyName { base, .. } => base,
            Node::StringLiteral(n) | Node::NoSubstitutionTemplateLiteral(n)
                | Node::TemplateHead(n) | Node::TemplateMiddle(n) | Node::TemplateTail(n) => &n.base,
            Node::NumericLiteral(n) => &n.base,
            Node::BigIntLiteral(n) => &n.base,
            Node::RegularExpressionLiteral(n) => &n.base,
            Node::BinaryExpression(n) => &n.base,
            Node::PrefixUnaryExpression(n) => &n.base,
            Node::PostfixUnaryExpression(n) => &n.base,
            Node::CallExpression(n) => &n.base,
            Node::NewExpression(n) => &n.base,
            Node::TaggedTemplateExpression(n) => &n.base,
            Node::TemplateExpression(n) => &n.base,
            Node::PropertyAccessExpression(n) => &n.base,
            Node::ElementAccessExpression(n) => &n.base,
            Node::ConditionalExpression(n) => &n.base,
            Node::ArrowFunction(n) => &n.base,
            Node::FunctionExpression(n) => &n.base,
            Node::ObjectLiteralExpression(n) => &n.base,
            Node::ArrayLiteralExpression(n) => &n.base,
            Node::ParenthesizedExpression(n) => &n.base,
            Node::YieldExpression(n) => &n.base,
            Node::AwaitExpression(n) => &n.base,
            Node::SpreadElement(n) => &n.base,
            Node::AsExpression(n) => &n.base,
            Node::SatisfiesExpression(n) => &n.base,
            Node::NonNullExpression(n) => &n.base,
            Node::TypeAssertion(n) => &n.base,
            Node::VariableStatement(n) => &n.base,
            Node::VariableDeclarationList(n) => &n.base,
            Node::VariableDeclaration(n) => &n.base,
            Node::ExpressionStatement(n) => &n.base,
            Node::IfStatement(n) => &n.base,
            Node::WhileStatement(n) => &n.base,
            Node::DoStatement(n) => &n.base,
            Node::ForStatement(n) => &n.base,
            Node::ForInStatement(n) => &n.base,
            Node::ForOfStatement(n) => &n.base,
            Node::SwitchStatement(n) => &n.base,
            Node::CaseBlock(n) => &n.base,
            Node::CaseClause(n) => &n.base,
            Node::DefaultClause(n) => &n.base,
            Node::ReturnStatement(n) => &n.base,
            Node::ThrowStatement(n) => &n.base,
            Node::TryStatement(n) => &n.base,
            Node::CatchClause(n) => &n.base,
            Node::LabeledStatement(n) => &n.base,
            Node::BreakStatement(n) => &n.base,
            Node::ContinueStatement(n) => &n.base,
            Node::WithStatement(n) => &n.base,
            Node::DebuggerStatement(n) => &n.base,
            Node::EmptyStatement(n) => &n.base,
            Node::Block(n) => &n.base,
            Node::FunctionDeclaration(n) => &n.base,
            Node::ClassDeclaration(n) => &n.base,
            Node::InterfaceDeclaration(n) => &n.base,
            Node::PropertySignature(n) => &n.base,
            Node::MethodSignature(n) => &n.base,
            Node::IndexSignatureDeclaration(n) => &n.base,
            Node::TypeAliasDeclaration(n) => &n.base,
            Node::EnumDeclaration(n) => &n.base,
            Node::EnumMember(n) => &n.base,
            Node::ModuleDeclaration(n) => &n.base,
            Node::ModuleBlock(n) => &n.base,
            Node::ImportDeclaration(n) => &n.base,
            Node::ImportClause(n) => &n.base,
            Node::NamespaceImport(n) => &n.base,
            Node::NamedImports(n) => &n.base,
            Node::ImportSpecifier(n) => &n.base,
            Node::ExportDeclaration(n) => &n.base,
            Node::NamedExports(n) => &n.base,
            Node::NamespaceExport(n) => &n.base,
            Node::ExportSpecifier(n) => &n.base,
            Node::ExportAssignment(n) => &n.base,
            Node::ImportAttributes(n) => &n.base,
            Node::ImportAttribute(n) => &n.base,
            Node::TypeReference(n) => &n.base,
            Node::FunctionType(n) => &n.base,
            Node::ConstructorType(n) => &n.base,
            Node::TypeQuery(n) => &n.base,
            Node::TypeLiteral(n) => &n.base,
            Node::ArrayType(n) => &n.base,
            Node::TupleType(n) => &n.base,
            Node::OptionalType(n) => &n.base,
            Node::RestType(n) => &n.base,
            Node::UnionType(n) => &n.base,
            Node::IntersectionType(n) => &n.base,
            Node::ConditionalType(n) => &n.base,
            Node::InferType(n) => &n.base,
            Node::ParenthesizedType(n) => &n.base,
            Node::TypeOperator(n) => &n.base,
            Node::IndexedAccessType(n) => &n.base,
            Node::MappedType(n) => &n.base,
            Node::LiteralType(n) => &n.base,
            Node::TemplateLiteralType(n) => &n.base,
            Node::NamedTupleMember(n) => &n.base,
            Node::PropertyDeclaration(n) => &n.base,
            Node::MethodDeclaration(n) => &n.base,
            Node::ConstructorDeclaration(n) => &n.base,
            Node::GetAccessorDeclaration(n) => &n.base,
            Node::SetAccessorDeclaration(n) => &n.base,
            Node::ParameterDeclaration(n) => &n.base,
            Node::TypeParameterDeclaration(n) => &n.base,
            Node::Decorator(n) => &n.base,
            Node::HeritageClause(n) => &n.base,
            Node::ExpressionWithTypeArguments(n) => &n.base,
            Node::ObjectBindingPattern(n) => &n.base,
            Node::ArrayBindingPattern(n) => &n.base,
            Node::BindingElement(n) => &n.base,
            Node::PropertyAssignment(n) => &n.base,
            Node::ShorthandPropertyAssignment(n) => &n.base,
            Node::SpreadAssignment(n) => &n.base,
            Node::JsxElement(n) => &n.base,
            Node::JsxSelfClosingElement(n) => &n.base,
            Node::JsxOpeningElement(n) => &n.base,
            Node::JsxClosingElement(n) => &n.base,
            Node::JsxFragment(n) => &n.base,
            Node::JsxOpeningFragment(n) => &n.base,
            Node::JsxClosingFragment(n) => &n.base,
            Node::JsxAttributes(n) => &n.base,
            Node::JsxAttribute(n) => &n.base,
            Node::JsxSpreadAttribute(n) => &n.base,
            Node::JsxExpression(n) => &n.base,
            Node::JsxText(n) => &n.base,
            Node::JsxNamespacedName(n) => &n.base,
            Node::TemplateSpan(n) => &n.base,
            Node::SourceFile(n) => &n.base,
        }
    }

    /// Get a mutable reference to the base node data
    pub fn base_mut(&mut self) -> &mut NodeBase {
        match self {
            Node::Token(base) | Node::EndOfFileToken(base) => base,
            Node::Identifier(n) | Node::PrivateIdentifier(n) => &mut n.base,
            Node::QualifiedName { base, .. } | Node::ComputedPropertyName { base, .. } => base,
            Node::StringLiteral(n) | Node::NoSubstitutionTemplateLiteral(n)
                | Node::TemplateHead(n) | Node::TemplateMiddle(n) | Node::TemplateTail(n) => &mut n.base,
            Node::NumericLiteral(n) => &mut n.base,
            Node::BigIntLiteral(n) => &mut n.base,
            Node::RegularExpressionLiteral(n) => &mut n.base,
            Node::BinaryExpression(n) => &mut n.base,
            Node::PrefixUnaryExpression(n) => &mut n.base,
            Node::PostfixUnaryExpression(n) => &mut n.base,
            Node::CallExpression(n) => &mut n.base,
            Node::NewExpression(n) => &mut n.base,
            Node::TaggedTemplateExpression(n) => &mut n.base,
            Node::TemplateExpression(n) => &mut n.base,
            Node::PropertyAccessExpression(n) => &mut n.base,
            Node::ElementAccessExpression(n) => &mut n.base,
            Node::ConditionalExpression(n) => &mut n.base,
            Node::ArrowFunction(n) => &mut n.base,
            Node::FunctionExpression(n) => &mut n.base,
            Node::ObjectLiteralExpression(n) => &mut n.base,
            Node::ArrayLiteralExpression(n) => &mut n.base,
            Node::ParenthesizedExpression(n) => &mut n.base,
            Node::YieldExpression(n) => &mut n.base,
            Node::AwaitExpression(n) => &mut n.base,
            Node::SpreadElement(n) => &mut n.base,
            Node::AsExpression(n) => &mut n.base,
            Node::SatisfiesExpression(n) => &mut n.base,
            Node::NonNullExpression(n) => &mut n.base,
            Node::TypeAssertion(n) => &mut n.base,
            Node::VariableStatement(n) => &mut n.base,
            Node::VariableDeclarationList(n) => &mut n.base,
            Node::VariableDeclaration(n) => &mut n.base,
            Node::ExpressionStatement(n) => &mut n.base,
            Node::IfStatement(n) => &mut n.base,
            Node::WhileStatement(n) => &mut n.base,
            Node::DoStatement(n) => &mut n.base,
            Node::ForStatement(n) => &mut n.base,
            Node::ForInStatement(n) => &mut n.base,
            Node::ForOfStatement(n) => &mut n.base,
            Node::SwitchStatement(n) => &mut n.base,
            Node::CaseBlock(n) => &mut n.base,
            Node::CaseClause(n) => &mut n.base,
            Node::DefaultClause(n) => &mut n.base,
            Node::ReturnStatement(n) => &mut n.base,
            Node::ThrowStatement(n) => &mut n.base,
            Node::TryStatement(n) => &mut n.base,
            Node::CatchClause(n) => &mut n.base,
            Node::LabeledStatement(n) => &mut n.base,
            Node::BreakStatement(n) => &mut n.base,
            Node::ContinueStatement(n) => &mut n.base,
            Node::WithStatement(n) => &mut n.base,
            Node::DebuggerStatement(n) => &mut n.base,
            Node::EmptyStatement(n) => &mut n.base,
            Node::Block(n) => &mut n.base,
            Node::FunctionDeclaration(n) => &mut n.base,
            Node::ClassDeclaration(n) => &mut n.base,
            Node::InterfaceDeclaration(n) => &mut n.base,
            Node::PropertySignature(n) => &mut n.base,
            Node::MethodSignature(n) => &mut n.base,
            Node::IndexSignatureDeclaration(n) => &mut n.base,
            Node::TypeAliasDeclaration(n) => &mut n.base,
            Node::EnumDeclaration(n) => &mut n.base,
            Node::EnumMember(n) => &mut n.base,
            Node::ModuleDeclaration(n) => &mut n.base,
            Node::ModuleBlock(n) => &mut n.base,
            Node::ImportDeclaration(n) => &mut n.base,
            Node::ImportClause(n) => &mut n.base,
            Node::NamespaceImport(n) => &mut n.base,
            Node::NamedImports(n) => &mut n.base,
            Node::ImportSpecifier(n) => &mut n.base,
            Node::ExportDeclaration(n) => &mut n.base,
            Node::NamedExports(n) => &mut n.base,
            Node::NamespaceExport(n) => &mut n.base,
            Node::ExportSpecifier(n) => &mut n.base,
            Node::ExportAssignment(n) => &mut n.base,
            Node::ImportAttributes(n) => &mut n.base,
            Node::ImportAttribute(n) => &mut n.base,
            Node::TypeReference(n) => &mut n.base,
            Node::FunctionType(n) => &mut n.base,
            Node::ConstructorType(n) => &mut n.base,
            Node::TypeQuery(n) => &mut n.base,
            Node::TypeLiteral(n) => &mut n.base,
            Node::ArrayType(n) => &mut n.base,
            Node::TupleType(n) => &mut n.base,
            Node::OptionalType(n) => &mut n.base,
            Node::RestType(n) => &mut n.base,
            Node::UnionType(n) => &mut n.base,
            Node::IntersectionType(n) => &mut n.base,
            Node::ConditionalType(n) => &mut n.base,
            Node::InferType(n) => &mut n.base,
            Node::ParenthesizedType(n) => &mut n.base,
            Node::TypeOperator(n) => &mut n.base,
            Node::IndexedAccessType(n) => &mut n.base,
            Node::MappedType(n) => &mut n.base,
            Node::LiteralType(n) => &mut n.base,
            Node::TemplateLiteralType(n) => &mut n.base,
            Node::NamedTupleMember(n) => &mut n.base,
            Node::PropertyDeclaration(n) => &mut n.base,
            Node::MethodDeclaration(n) => &mut n.base,
            Node::ConstructorDeclaration(n) => &mut n.base,
            Node::GetAccessorDeclaration(n) => &mut n.base,
            Node::SetAccessorDeclaration(n) => &mut n.base,
            Node::ParameterDeclaration(n) => &mut n.base,
            Node::TypeParameterDeclaration(n) => &mut n.base,
            Node::Decorator(n) => &mut n.base,
            Node::HeritageClause(n) => &mut n.base,
            Node::ExpressionWithTypeArguments(n) => &mut n.base,
            Node::ObjectBindingPattern(n) => &mut n.base,
            Node::ArrayBindingPattern(n) => &mut n.base,
            Node::BindingElement(n) => &mut n.base,
            Node::PropertyAssignment(n) => &mut n.base,
            Node::ShorthandPropertyAssignment(n) => &mut n.base,
            Node::SpreadAssignment(n) => &mut n.base,
            Node::JsxElement(n) => &mut n.base,
            Node::JsxSelfClosingElement(n) => &mut n.base,
            Node::JsxOpeningElement(n) => &mut n.base,
            Node::JsxClosingElement(n) => &mut n.base,
            Node::JsxFragment(n) => &mut n.base,
            Node::JsxOpeningFragment(n) => &mut n.base,
            Node::JsxClosingFragment(n) => &mut n.base,
            Node::JsxAttributes(n) => &mut n.base,
            Node::JsxAttribute(n) => &mut n.base,
            Node::JsxSpreadAttribute(n) => &mut n.base,
            Node::JsxExpression(n) => &mut n.base,
            Node::JsxText(n) => &mut n.base,
            Node::JsxNamespacedName(n) => &mut n.base,
            Node::TemplateSpan(n) => &mut n.base,
            Node::SourceFile(n) => &mut n.base,
        }
    }

    /// Get the SyntaxKind value for this node (as u16, may be extended kind)
    pub fn kind(&self) -> u16 {
        self.base().kind
    }

    /// Get the SyntaxKind for this node (only valid for token kinds)
    pub fn kind_as_syntax_kind(&self) -> SyntaxKind {
        // Safety: This conversion is only valid for token kinds (0-166)
        // For extended kinds, use kind() and compare with syntax_kind_ext constants
        unsafe { std::mem::transmute(self.base().kind) }
    }

    /// Get the start position
    pub fn pos(&self) -> u32 {
        self.base().pos
    }

    /// Get the end position
    pub fn end(&self) -> u32 {
        self.base().end
    }
}

// =============================================================================
// Node Arena (Storage for all nodes)
// =============================================================================

/// Arena-based storage for AST nodes.
/// Nodes are stored contiguously and referenced by index.
#[derive(Debug, Default, Serialize)]
pub struct NodeArena {
    pub nodes: Vec<Node>,
}

impl NodeArena {
    pub fn new() -> NodeArena {
        NodeArena { nodes: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> NodeArena {
        NodeArena { nodes: Vec::with_capacity(capacity) }
    }

    /// Add a node to the arena and return its index
    pub fn add(&mut self, node: Node) -> NodeIndex {
        let index = self.nodes.len() as u32;
        self.nodes.push(node);
        NodeIndex(index)
    }

    /// Get a node by index
    pub fn get(&self, index: NodeIndex) -> Option<&Node> {
        if index.is_none() {
            None
        } else {
            self.nodes.get(index.0 as usize)
        }
    }

    /// Get a mutable node by index
    pub fn get_mut(&mut self, index: NodeIndex) -> Option<&mut Node> {
        if index.is_none() {
            None
        } else {
            self.nodes.get_mut(index.0 as usize)
        }
    }

    /// Get the number of nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the arena is empty
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

    #[test]
    fn test_node_flags() {
        assert_eq!(node_flags::NONE, 0);
        assert_eq!(node_flags::LET, 1);
        assert_eq!(node_flags::CONST, 2);
        assert_eq!(node_flags::AWAIT_USING, 6);  // Const | Using
    }

    #[test]
    fn test_modifier_flags() {
        assert_eq!(modifier_flags::NONE, 0);
        assert_eq!(modifier_flags::PUBLIC, 1);
        assert_eq!(modifier_flags::EXPORT, 32);
        assert_eq!(modifier_flags::ASYNC, 1024);
    }

    #[test]
    fn test_node_index() {
        let index = NodeIndex(0);
        assert!(index.is_some());
        assert!(!index.is_none());

        let none = NodeIndex::NONE;
        assert!(none.is_none());
        assert!(!none.is_some());
    }

    #[test]
    fn test_node_arena() {
        let mut arena = NodeArena::new();

        let id = Identifier::new("test".to_string(), 0, 4);
        let idx = arena.add(Node::Identifier(id));

        assert_eq!(idx.0, 0);
        assert_eq!(arena.len(), 1);

        let node = arena.get(idx).unwrap();
        assert_eq!(node.kind(), SyntaxKind::Identifier as u16);
        assert_eq!(node.pos(), 0);
        assert_eq!(node.end(), 4);
    }

    #[test]
    fn test_identifier() {
        let id = Identifier::new("myVar".to_string(), 10, 15);
        assert_eq!(id.escaped_text, "myVar");
        assert_eq!(id.base.kind, SyntaxKind::Identifier as u16);
        assert_eq!(id.base.pos, 10);
        assert_eq!(id.base.end, 15);
    }

    #[test]
    fn test_source_file() {
        let sf = SourceFile::new("test.ts".to_string(), "const x = 1;".to_string());
        // SourceFile kind is 308 in TypeScript, stored directly in kind field
        assert_eq!(sf.base.kind as u16, syntax_kind_ext::SOURCE_FILE);
        assert_eq!(sf.file_name, "test.ts");
        assert_eq!(sf.text.len(), 12);
    }
}
