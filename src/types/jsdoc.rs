//! JSDoc Type Representations
//!
//! This module provides type definitions for JSDoc type annotations including:
//! - Type expressions ({string}, {number|null}, etc.)
//! - @type, @param, @returns annotations
//! - @typedef and @callback definitions
//! - @template for generics
//! - @extends and @implements

use std::collections::HashMap;

/// Unique identifier for types
pub type TypeId = u64;

/// Represents a JSDoc type expression
#[derive(Debug, Clone, PartialEq)]
pub enum JsDocType {
    /// Primitive types: string, number, boolean, null, undefined, void, never, any, unknown
    Primitive(PrimitiveType),
    /// Named type reference: SomeClass, MyInterface
    TypeReference {
        name: String,
        type_arguments: Vec<JsDocType>,
    },
    /// Union type: string|number
    Union(Vec<JsDocType>),
    /// Intersection type: TypeA & TypeB
    Intersection(Vec<JsDocType>),
    /// Array type: string[] or Array<string>
    Array(Box<JsDocType>),
    /// Tuple type: [string, number]
    Tuple(Vec<JsDocType>),
    /// Function type: function(string, number): boolean
    Function(JsDocFunctionType),
    /// Object type: {name: string, age: number}
    Object(Vec<JsDocPropertySignature>),
    /// Optional type: ?string
    Optional(Box<JsDocType>),
    /// Non-nullable type: !string
    NonNullable(Box<JsDocType>),
    /// Nullable type: ?string (different from optional in some contexts)
    Nullable(Box<JsDocType>),
    /// Rest/variadic type: ...string
    Rest(Box<JsDocType>),
    /// Literal type: "hello", 42, true
    Literal(LiteralType),
    /// typeof expression: typeof SomeClass
    TypeOf(String),
    /// keyof expression: keyof SomeType
    KeyOf(Box<JsDocType>),
    /// Template literal: `hello ${string}`
    TemplateLiteral(Vec<TemplateLiteralPart>),
    /// All type: * (equivalent to any)
    All,
    /// Unknown type: ?
    Unknown,
    /// This type
    This,
    /// Parenthesized for grouping
    Parenthesized(Box<JsDocType>),
    /// Record type: Record<string, number>
    Record {
        key_type: Box<JsDocType>,
        value_type: Box<JsDocType>,
    },
    /// Import type: import("./module").Type
    Import {
        path: String,
        qualifier: Option<String>,
        type_arguments: Vec<JsDocType>,
    },
}

impl JsDocType {
    /// Create a primitive type
    pub fn primitive(kind: PrimitiveType) -> Self {
        JsDocType::Primitive(kind)
    }

    /// Create a type reference
    pub fn type_ref(name: impl Into<String>) -> Self {
        JsDocType::TypeReference {
            name: name.into(),
            type_arguments: Vec::new(),
        }
    }

    /// Create a type reference with type arguments
    pub fn generic_type_ref(name: impl Into<String>, args: Vec<JsDocType>) -> Self {
        JsDocType::TypeReference {
            name: name.into(),
            type_arguments: args,
        }
    }

    /// Create a union type
    pub fn union(types: Vec<JsDocType>) -> Self {
        if types.len() == 1 {
            types.into_iter().next().unwrap()
        } else {
            JsDocType::Union(types)
        }
    }

    /// Create an array type
    pub fn array(element: JsDocType) -> Self {
        JsDocType::Array(Box::new(element))
    }

    /// Create an optional type
    pub fn optional(inner: JsDocType) -> Self {
        JsDocType::Optional(Box::new(inner))
    }

    /// Check if this is a nullable type
    pub fn is_nullable(&self) -> bool {
        matches!(
            self,
            JsDocType::Nullable(_) | JsDocType::Optional(_) | JsDocType::Primitive(PrimitiveType::Null)
        )
    }

    /// Check if this is the any type
    pub fn is_any(&self) -> bool {
        matches!(self, JsDocType::Primitive(PrimitiveType::Any) | JsDocType::All)
    }
}

/// Primitive type kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    String,
    Number,
    Boolean,
    Null,
    Undefined,
    Void,
    Never,
    Any,
    Unknown,
    Symbol,
    BigInt,
    Object,
}

impl PrimitiveType {
    /// Parse a primitive type name
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "string" => Some(PrimitiveType::String),
            "number" => Some(PrimitiveType::Number),
            "boolean" | "bool" => Some(PrimitiveType::Boolean),
            "null" => Some(PrimitiveType::Null),
            "undefined" => Some(PrimitiveType::Undefined),
            "void" => Some(PrimitiveType::Void),
            "never" => Some(PrimitiveType::Never),
            "any" => Some(PrimitiveType::Any),
            "unknown" => Some(PrimitiveType::Unknown),
            "symbol" => Some(PrimitiveType::Symbol),
            "bigint" => Some(PrimitiveType::BigInt),
            "object" => Some(PrimitiveType::Object),
            _ => None,
        }
    }

    /// Get the name of this primitive type
    pub fn name(&self) -> &'static str {
        match self {
            PrimitiveType::String => "string",
            PrimitiveType::Number => "number",
            PrimitiveType::Boolean => "boolean",
            PrimitiveType::Null => "null",
            PrimitiveType::Undefined => "undefined",
            PrimitiveType::Void => "void",
            PrimitiveType::Never => "never",
            PrimitiveType::Any => "any",
            PrimitiveType::Unknown => "unknown",
            PrimitiveType::Symbol => "symbol",
            PrimitiveType::BigInt => "bigint",
            PrimitiveType::Object => "object",
        }
    }
}

/// JSDoc function type
#[derive(Debug, Clone, PartialEq)]
pub struct JsDocFunctionType {
    pub parameters: Vec<JsDocParameter>,
    pub return_type: Option<Box<JsDocType>>,
    pub is_constructor: bool,
    pub this_type: Option<Box<JsDocType>>,
    pub is_new: bool,
}

impl JsDocFunctionType {
    pub fn new() -> Self {
        JsDocFunctionType {
            parameters: Vec::new(),
            return_type: None,
            is_constructor: false,
            this_type: None,
            is_new: false,
        }
    }

    pub fn with_params(mut self, params: Vec<JsDocParameter>) -> Self {
        self.parameters = params;
        self
    }

    pub fn with_return(mut self, return_type: JsDocType) -> Self {
        self.return_type = Some(Box::new(return_type));
        self
    }
}

impl Default for JsDocFunctionType {
    fn default() -> Self {
        Self::new()
    }
}

/// JSDoc property signature in object types
#[derive(Debug, Clone, PartialEq)]
pub struct JsDocPropertySignature {
    pub name: String,
    pub type_annotation: JsDocType,
    pub optional: bool,
    pub readonly: bool,
}

impl JsDocPropertySignature {
    pub fn new(name: impl Into<String>, type_annotation: JsDocType) -> Self {
        JsDocPropertySignature {
            name: name.into(),
            type_annotation,
            optional: false,
            readonly: false,
        }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }
}

/// Literal type values
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralType {
    String(String),
    Number(f64),
    Boolean(bool),
    BigInt(String),
}

/// Template literal part
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateLiteralPart {
    Text(String),
    Placeholder(JsDocType),
}

/// JSDoc parameter
#[derive(Debug, Clone, PartialEq)]
pub struct JsDocParameter {
    pub name: Option<String>,
    pub type_annotation: Option<JsDocType>,
    pub optional: bool,
    pub rest: bool,
    pub default_value: Option<String>,
    pub description: Option<String>,
}

impl JsDocParameter {
    pub fn new() -> Self {
        JsDocParameter {
            name: None,
            type_annotation: None,
            optional: false,
            rest: false,
            default_value: None,
            description: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_type(mut self, type_annotation: JsDocType) -> Self {
        self.type_annotation = Some(type_annotation);
        self
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn rest(mut self) -> Self {
        self.rest = true;
        self
    }
}

impl Default for JsDocParameter {
    fn default() -> Self {
        Self::new()
    }
}

/// JSDoc @typedef definition
#[derive(Debug, Clone, PartialEq)]
pub struct JsDocTypedef {
    pub name: String,
    pub type_expression: JsDocType,
    pub type_parameters: Vec<JsDocTypeParameter>,
    pub description: Option<String>,
}

impl JsDocTypedef {
    pub fn new(name: impl Into<String>, type_expression: JsDocType) -> Self {
        JsDocTypedef {
            name: name.into(),
            type_expression,
            type_parameters: Vec::new(),
            description: None,
        }
    }

    pub fn with_type_params(mut self, params: Vec<JsDocTypeParameter>) -> Self {
        self.type_parameters = params;
        self
    }
}

/// JSDoc @callback definition
#[derive(Debug, Clone, PartialEq)]
pub struct JsDocCallback {
    pub name: String,
    pub parameters: Vec<JsDocParameter>,
    pub return_type: Option<JsDocType>,
    pub type_parameters: Vec<JsDocTypeParameter>,
    pub description: Option<String>,
}

impl JsDocCallback {
    pub fn new(name: impl Into<String>) -> Self {
        JsDocCallback {
            name: name.into(),
            parameters: Vec::new(),
            return_type: None,
            type_parameters: Vec::new(),
            description: None,
        }
    }

    pub fn with_params(mut self, params: Vec<JsDocParameter>) -> Self {
        self.parameters = params;
        self
    }

    pub fn with_return(mut self, return_type: JsDocType) -> Self {
        self.return_type = Some(return_type);
        self
    }
}

/// JSDoc @template type parameter
#[derive(Debug, Clone, PartialEq)]
pub struct JsDocTypeParameter {
    pub name: String,
    pub constraint: Option<JsDocType>,
    pub default: Option<JsDocType>,
    pub description: Option<String>,
}

impl JsDocTypeParameter {
    pub fn new(name: impl Into<String>) -> Self {
        JsDocTypeParameter {
            name: name.into(),
            constraint: None,
            default: None,
            description: None,
        }
    }

    pub fn with_constraint(mut self, constraint: JsDocType) -> Self {
        self.constraint = Some(constraint);
        self
    }

    pub fn with_default(mut self, default: JsDocType) -> Self {
        self.default = Some(default);
        self
    }
}

/// JSDoc @extends or @implements clause
#[derive(Debug, Clone, PartialEq)]
pub struct JsDocHeritageClause {
    pub kind: HeritageClauseKind,
    pub type_expression: JsDocType,
}

/// Kind of heritage clause
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeritageClauseKind {
    Extends,
    Implements,
    Augments, // Alternative for @extends
}

/// JSDoc modifier tags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsDocModifier {
    Deprecated,
    Readonly,
    Private,
    Protected,
    Public,
    Abstract,
    Override,
    Virtual,
    Static,
    Const,
    Final,
    Internal,
    Experimental,
    Beta,
    Alpha,
}

impl JsDocModifier {
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "deprecated" => Some(JsDocModifier::Deprecated),
            "readonly" => Some(JsDocModifier::Readonly),
            "private" => Some(JsDocModifier::Private),
            "protected" => Some(JsDocModifier::Protected),
            "public" => Some(JsDocModifier::Public),
            "abstract" => Some(JsDocModifier::Abstract),
            "override" => Some(JsDocModifier::Override),
            "virtual" => Some(JsDocModifier::Virtual),
            "static" => Some(JsDocModifier::Static),
            "const" => Some(JsDocModifier::Const),
            "final" => Some(JsDocModifier::Final),
            "internal" => Some(JsDocModifier::Internal),
            "experimental" => Some(JsDocModifier::Experimental),
            "beta" => Some(JsDocModifier::Beta),
            "alpha" => Some(JsDocModifier::Alpha),
            _ => None,
        }
    }
}

/// Complete JSDoc comment with all parsed information
#[derive(Debug, Clone, Default)]
pub struct JsDocComment {
    /// Description text
    pub description: Option<String>,
    /// @type annotation
    pub type_annotation: Option<JsDocType>,
    /// @param annotations
    pub params: Vec<JsDocParameter>,
    /// @returns annotation
    pub returns: Option<JsDocReturns>,
    /// @typedef definitions
    pub typedefs: Vec<JsDocTypedef>,
    /// @callback definitions
    pub callbacks: Vec<JsDocCallback>,
    /// @template annotations
    pub type_parameters: Vec<JsDocTypeParameter>,
    /// @extends/@augments clauses
    pub extends: Vec<JsDocHeritageClause>,
    /// @implements clauses
    pub implements: Vec<JsDocHeritageClause>,
    /// Modifier tags (@deprecated, @readonly, etc.)
    pub modifiers: Vec<JsDocModifier>,
    /// @see references
    pub see: Vec<String>,
    /// @example code
    pub examples: Vec<String>,
    /// @throws/@exception
    pub throws: Vec<JsDocThrows>,
    /// @author
    pub author: Option<String>,
    /// @version
    pub version: Option<String>,
    /// @since
    pub since: Option<String>,
    /// @enum
    pub enum_type: Option<JsDocType>,
    /// @class/@constructor
    pub is_class: bool,
    /// @interface
    pub is_interface: bool,
    /// @this type
    pub this_type: Option<JsDocType>,
    /// @satisfies type
    pub satisfies: Option<JsDocType>,
    /// @overload
    pub is_overload: bool,
    /// Raw tags for custom handling
    pub custom_tags: HashMap<String, Vec<String>>,
}

impl JsDocComment {
    pub fn new() -> Self {
        JsDocComment::default()
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_type(mut self, type_annotation: JsDocType) -> Self {
        self.type_annotation = Some(type_annotation);
        self
    }

    pub fn add_param(mut self, param: JsDocParameter) -> Self {
        self.params.push(param);
        self
    }

    pub fn with_returns(mut self, returns: JsDocReturns) -> Self {
        self.returns = Some(returns);
        self
    }

    pub fn add_modifier(mut self, modifier: JsDocModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    pub fn is_deprecated(&self) -> bool {
        self.modifiers.contains(&JsDocModifier::Deprecated)
    }

    pub fn has_type_info(&self) -> bool {
        self.type_annotation.is_some()
            || !self.params.is_empty()
            || self.returns.is_some()
            || !self.typedefs.is_empty()
            || !self.callbacks.is_empty()
    }
}

/// @returns annotation
#[derive(Debug, Clone, PartialEq)]
pub struct JsDocReturns {
    pub type_annotation: Option<JsDocType>,
    pub description: Option<String>,
}

impl JsDocReturns {
    pub fn new() -> Self {
        JsDocReturns {
            type_annotation: None,
            description: None,
        }
    }

    pub fn with_type(mut self, type_annotation: JsDocType) -> Self {
        self.type_annotation = Some(type_annotation);
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

impl Default for JsDocReturns {
    fn default() -> Self {
        Self::new()
    }
}

/// @throws/@exception annotation
#[derive(Debug, Clone, PartialEq)]
pub struct JsDocThrows {
    pub type_annotation: Option<JsDocType>,
    pub description: Option<String>,
}

impl JsDocThrows {
    pub fn new() -> Self {
        JsDocThrows {
            type_annotation: None,
            description: None,
        }
    }

    pub fn with_type(mut self, type_annotation: JsDocType) -> Self {
        self.type_annotation = Some(type_annotation);
        self
    }
}

impl Default for JsDocThrows {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_type_parsing() {
        assert_eq!(PrimitiveType::from_str("string"), Some(PrimitiveType::String));
        assert_eq!(PrimitiveType::from_str("number"), Some(PrimitiveType::Number));
        assert_eq!(PrimitiveType::from_str("boolean"), Some(PrimitiveType::Boolean));
        assert_eq!(PrimitiveType::from_str("bool"), Some(PrimitiveType::Boolean));
        assert_eq!(PrimitiveType::from_str("null"), Some(PrimitiveType::Null));
        assert_eq!(PrimitiveType::from_str("undefined"), Some(PrimitiveType::Undefined));
        assert_eq!(PrimitiveType::from_str("any"), Some(PrimitiveType::Any));
        assert_eq!(PrimitiveType::from_str("SomeClass"), None);
    }

    #[test]
    fn test_jsdoc_type_creation() {
        let string_type = JsDocType::primitive(PrimitiveType::String);
        assert!(matches!(string_type, JsDocType::Primitive(PrimitiveType::String)));

        let ref_type = JsDocType::type_ref("MyClass");
        assert!(matches!(ref_type, JsDocType::TypeReference { name, .. } if name == "MyClass"));

        let array_type = JsDocType::array(JsDocType::primitive(PrimitiveType::String));
        assert!(matches!(array_type, JsDocType::Array(_)));
    }

    #[test]
    fn test_union_type() {
        let union = JsDocType::union(vec![
            JsDocType::primitive(PrimitiveType::String),
            JsDocType::primitive(PrimitiveType::Number),
        ]);
        assert!(matches!(union, JsDocType::Union(types) if types.len() == 2));

        // Single type should not create union
        let single = JsDocType::union(vec![JsDocType::primitive(PrimitiveType::String)]);
        assert!(matches!(single, JsDocType::Primitive(PrimitiveType::String)));
    }

    #[test]
    fn test_generic_type_ref() {
        let generic = JsDocType::generic_type_ref(
            "Array",
            vec![JsDocType::primitive(PrimitiveType::String)],
        );
        match generic {
            JsDocType::TypeReference { name, type_arguments } => {
                assert_eq!(name, "Array");
                assert_eq!(type_arguments.len(), 1);
            }
            _ => panic!("Expected TypeReference"),
        }
    }

    #[test]
    fn test_jsdoc_parameter() {
        let param = JsDocParameter::new()
            .with_name("name")
            .with_type(JsDocType::primitive(PrimitiveType::String))
            .optional();

        assert_eq!(param.name, Some("name".to_string()));
        assert!(param.optional);
        assert!(param.type_annotation.is_some());
    }

    #[test]
    fn test_jsdoc_typedef() {
        let typedef = JsDocTypedef::new(
            "MyType",
            JsDocType::type_ref("SomeClass"),
        ).with_type_params(vec![
            JsDocTypeParameter::new("T"),
        ]);

        assert_eq!(typedef.name, "MyType");
        assert_eq!(typedef.type_parameters.len(), 1);
    }

    #[test]
    fn test_jsdoc_callback() {
        let callback = JsDocCallback::new("MyCallback")
            .with_params(vec![
                JsDocParameter::new()
                    .with_name("value")
                    .with_type(JsDocType::primitive(PrimitiveType::String)),
            ])
            .with_return(JsDocType::primitive(PrimitiveType::Boolean));

        assert_eq!(callback.name, "MyCallback");
        assert_eq!(callback.parameters.len(), 1);
        assert!(callback.return_type.is_some());
    }

    #[test]
    fn test_jsdoc_type_parameter() {
        let param = JsDocTypeParameter::new("T")
            .with_constraint(JsDocType::type_ref("Comparable"))
            .with_default(JsDocType::primitive(PrimitiveType::String));

        assert_eq!(param.name, "T");
        assert!(param.constraint.is_some());
        assert!(param.default.is_some());
    }

    #[test]
    fn test_jsdoc_modifier() {
        assert_eq!(JsDocModifier::from_tag("deprecated"), Some(JsDocModifier::Deprecated));
        assert_eq!(JsDocModifier::from_tag("readonly"), Some(JsDocModifier::Readonly));
        assert_eq!(JsDocModifier::from_tag("private"), Some(JsDocModifier::Private));
        assert_eq!(JsDocModifier::from_tag("unknown"), None);
    }

    #[test]
    fn test_jsdoc_comment() {
        let comment = JsDocComment::new()
            .with_description("A test function")
            .with_type(JsDocType::primitive(PrimitiveType::String))
            .add_param(JsDocParameter::new().with_name("arg"))
            .add_modifier(JsDocModifier::Deprecated);

        assert!(comment.description.is_some());
        assert!(comment.type_annotation.is_some());
        assert_eq!(comment.params.len(), 1);
        assert!(comment.is_deprecated());
        assert!(comment.has_type_info());
    }

    #[test]
    fn test_function_type() {
        let func = JsDocFunctionType::new()
            .with_params(vec![
                JsDocParameter::new()
                    .with_name("x")
                    .with_type(JsDocType::primitive(PrimitiveType::Number)),
            ])
            .with_return(JsDocType::primitive(PrimitiveType::String));

        assert_eq!(func.parameters.len(), 1);
        assert!(func.return_type.is_some());
        assert!(!func.is_constructor);
    }

    #[test]
    fn test_property_signature() {
        let prop = JsDocPropertySignature::new(
            "name",
            JsDocType::primitive(PrimitiveType::String),
        ).optional().readonly();

        assert_eq!(prop.name, "name");
        assert!(prop.optional);
        assert!(prop.readonly);
    }

    #[test]
    fn test_nullable_type() {
        let nullable = JsDocType::Nullable(Box::new(JsDocType::primitive(PrimitiveType::String)));
        assert!(nullable.is_nullable());

        let optional = JsDocType::optional(JsDocType::primitive(PrimitiveType::String));
        assert!(optional.is_nullable());

        let string = JsDocType::primitive(PrimitiveType::String);
        assert!(!string.is_nullable());
    }

    #[test]
    fn test_any_type() {
        let any = JsDocType::primitive(PrimitiveType::Any);
        assert!(any.is_any());

        let all = JsDocType::All;
        assert!(all.is_any());

        let string = JsDocType::primitive(PrimitiveType::String);
        assert!(!string.is_any());
    }
}
