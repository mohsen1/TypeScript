//! JSDoc Comment Parser
//!
//! Parses JSDoc comments and extracts type information including:
//! - @type, @param, @returns annotations
//! - @typedef and @callback definitions
//! - @template for generic type parameters
//! - @extends and @implements clauses
//! - JSDoc type expressions ({string}, {number|null}, etc.)
//! - @deprecated and other modifier tags

use std::collections::HashMap;

/// Unique identifier for types
pub type TypeId = u64;

/// Text span for source locations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSpan {
    pub start: u32,
    pub end: u32,
}

impl TextSpan {
    pub fn new(start: u32, end: u32) -> Self {
        TextSpan { start, end }
    }
}

/// Parser error
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Option<TextSpan>,
}

impl ParseError {
    pub fn new(message: impl Into<String>) -> Self {
        ParseError {
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(mut self, span: TextSpan) -> Self {
        self.span = Some(span);
        self
    }
}

/// JSDoc type expression
#[derive(Debug, Clone, PartialEq)]
pub enum JsDocTypeExpr {
    /// Primitive: string, number, boolean, etc.
    Primitive(String),
    /// Named reference: SomeClass, MyType
    Reference { name: String, type_args: Vec<JsDocTypeExpr> },
    /// Union: A | B
    Union(Vec<JsDocTypeExpr>),
    /// Intersection: A & B
    Intersection(Vec<JsDocTypeExpr>),
    /// Array: T[] or Array<T>
    Array(Box<JsDocTypeExpr>),
    /// Tuple: [A, B, C]
    Tuple(Vec<JsDocTypeExpr>),
    /// Function: function(A, B): C
    Function {
        params: Vec<JsDocTypeExpr>,
        return_type: Option<Box<JsDocTypeExpr>>,
        this_type: Option<Box<JsDocTypeExpr>>,
        is_new: bool,
    },
    /// Object literal: { a: A, b: B }
    Object(Vec<PropertySignature>),
    /// Optional: ?T
    Optional(Box<JsDocTypeExpr>),
    /// Nullable: ?T (closure compiler style)
    Nullable(Box<JsDocTypeExpr>),
    /// Non-nullable: !T
    NonNullable(Box<JsDocTypeExpr>),
    /// Rest: ...T
    Rest(Box<JsDocTypeExpr>),
    /// Literal: "string", 42, true
    Literal(LiteralValue),
    /// typeof T
    TypeOf(String),
    /// keyof T
    KeyOf(Box<JsDocTypeExpr>),
    /// * (any type)
    Star,
    /// ? (unknown type)
    Unknown,
    /// void
    Void,
    /// this
    This,
    /// Parenthesized: (A | B)
    Parenthesized(Box<JsDocTypeExpr>),
    /// import("path").Type
    Import { path: String, qualifier: Option<String> },
}

/// Literal value in type expressions
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
    BigInt(String),
    Null,
    Undefined,
}

/// Property signature in object types
#[derive(Debug, Clone, PartialEq)]
pub struct PropertySignature {
    pub name: String,
    pub type_expr: JsDocTypeExpr,
    pub optional: bool,
    pub readonly: bool,
}

/// Parsed @param tag
#[derive(Debug, Clone, PartialEq)]
pub struct ParamTag {
    pub name: String,
    pub type_expr: Option<JsDocTypeExpr>,
    pub description: Option<String>,
    pub optional: bool,
    pub default_value: Option<String>,
}

/// Parsed @returns/@return tag
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnsTag {
    pub type_expr: Option<JsDocTypeExpr>,
    pub description: Option<String>,
}

/// Parsed @typedef tag
#[derive(Debug, Clone, PartialEq)]
pub struct TypedefTag {
    pub name: String,
    pub type_expr: Option<JsDocTypeExpr>,
    pub properties: Vec<PropertyTag>,
}

/// Parsed @property tag (used in @typedef)
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyTag {
    pub name: String,
    pub type_expr: Option<JsDocTypeExpr>,
    pub description: Option<String>,
    pub optional: bool,
}

/// Parsed @callback tag
#[derive(Debug, Clone, PartialEq)]
pub struct CallbackTag {
    pub name: String,
    pub params: Vec<ParamTag>,
    pub returns: Option<ReturnsTag>,
}

/// Parsed @template tag
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateTag {
    pub names: Vec<String>,
    pub constraint: Option<JsDocTypeExpr>,
    pub default: Option<JsDocTypeExpr>,
    pub description: Option<String>,
}

/// Parsed @extends/@augments tag
#[derive(Debug, Clone, PartialEq)]
pub struct ExtendsTag {
    pub type_expr: JsDocTypeExpr,
}

/// Parsed @implements tag
#[derive(Debug, Clone, PartialEq)]
pub struct ImplementsTag {
    pub type_expr: JsDocTypeExpr,
}

/// Modifier tags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierTag {
    Deprecated,
    Readonly,
    Private,
    Protected,
    Public,
    Abstract,
    Override,
    Static,
    Const,
    Final,
    Internal,
}

/// Complete parsed JSDoc comment
#[derive(Debug, Clone, Default)]
pub struct ParsedJsDoc {
    pub description: Option<String>,
    pub type_tag: Option<JsDocTypeExpr>,
    pub params: Vec<ParamTag>,
    pub returns: Option<ReturnsTag>,
    pub typedefs: Vec<TypedefTag>,
    pub callbacks: Vec<CallbackTag>,
    pub templates: Vec<TemplateTag>,
    pub extends: Vec<ExtendsTag>,
    pub implements: Vec<ImplementsTag>,
    pub modifiers: Vec<ModifierTag>,
    pub this_tag: Option<JsDocTypeExpr>,
    pub enum_tag: Option<JsDocTypeExpr>,
    pub satisfies_tag: Option<JsDocTypeExpr>,
    pub throws: Vec<JsDocTypeExpr>,
    pub see: Vec<String>,
    pub examples: Vec<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub since: Option<String>,
    pub is_class: bool,
    pub is_interface: bool,
    pub is_overload: bool,
    pub custom_tags: HashMap<String, Vec<String>>,
}

/// JSDoc parser
pub struct JsDocParser<'a> {
    input: &'a str,
    pos: usize,
    errors: Vec<ParseError>,
}

impl<'a> JsDocParser<'a> {
    pub fn new(input: &'a str) -> Self {
        JsDocParser {
            input,
            pos: 0,
            errors: Vec::new(),
        }
    }

    /// Parse a complete JSDoc comment (including /** and */)
    pub fn parse_comment(input: &str) -> Result<ParsedJsDoc, Vec<ParseError>> {
        let content = Self::extract_content(input);
        let mut parser = JsDocParser::new(&content);
        let result = parser.parse();

        if parser.errors.is_empty() {
            Ok(result)
        } else {
            Err(parser.errors)
        }
    }

    /// Extract content from JSDoc comment, removing /** */ and leading *
    fn extract_content(input: &str) -> String {
        let s = input.trim();
        let s = s.strip_prefix("/**").unwrap_or(s);
        let s = s.strip_suffix("*/").unwrap_or(s);

        s.lines()
            .map(|line| {
                let trimmed = line.trim();
                trimmed.strip_prefix("*").unwrap_or(trimmed).trim()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Parse the JSDoc content
    fn parse(&mut self) -> ParsedJsDoc {
        let mut result = ParsedJsDoc::default();
        let mut description_lines = Vec::new();
        let mut in_description = true;

        for line in self.input.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with('@') {
                in_description = false;
                self.parse_tag(trimmed, &mut result);
            } else if in_description && !trimmed.is_empty() {
                description_lines.push(trimmed);
            }
        }

        if !description_lines.is_empty() {
            result.description = Some(description_lines.join(" "));
        }

        result
    }

    /// Parse a single tag line
    fn parse_tag(&mut self, line: &str, result: &mut ParsedJsDoc) {
        let line = &line[1..]; // Skip @
        let (tag_name, rest) = Self::split_tag(line);

        match tag_name {
            "type" => {
                if let Some(type_expr) = self.parse_type_from_braces(rest) {
                    result.type_tag = Some(type_expr);
                }
            }
            "param" | "arg" | "argument" => {
                if let Some(param) = self.parse_param_tag(rest) {
                    result.params.push(param);
                }
            }
            "returns" | "return" => {
                result.returns = Some(self.parse_returns_tag(rest));
            }
            "typedef" => {
                if let Some(typedef) = self.parse_typedef_tag(rest) {
                    result.typedefs.push(typedef);
                }
            }
            "callback" => {
                if let Some(callback) = self.parse_callback_tag(rest) {
                    result.callbacks.push(callback);
                }
            }
            "template" => {
                if let Some(template) = self.parse_template_tag(rest) {
                    result.templates.push(template);
                }
            }
            "extends" | "augments" => {
                if let Some(type_expr) = self.parse_type_from_braces(rest) {
                    result.extends.push(ExtendsTag { type_expr });
                }
            }
            "implements" => {
                if let Some(type_expr) = self.parse_type_from_braces(rest) {
                    result.implements.push(ImplementsTag { type_expr });
                }
            }
            "this" => {
                result.this_tag = self.parse_type_from_braces(rest);
            }
            "enum" => {
                result.enum_tag = self.parse_type_from_braces(rest);
            }
            "satisfies" => {
                result.satisfies_tag = self.parse_type_from_braces(rest);
            }
            "throws" | "exception" => {
                if let Some(type_expr) = self.parse_type_from_braces(rest) {
                    result.throws.push(type_expr);
                }
            }
            "deprecated" => result.modifiers.push(ModifierTag::Deprecated),
            "readonly" => result.modifiers.push(ModifierTag::Readonly),
            "private" => result.modifiers.push(ModifierTag::Private),
            "protected" => result.modifiers.push(ModifierTag::Protected),
            "public" => result.modifiers.push(ModifierTag::Public),
            "abstract" => result.modifiers.push(ModifierTag::Abstract),
            "override" => result.modifiers.push(ModifierTag::Override),
            "static" => result.modifiers.push(ModifierTag::Static),
            "const" => result.modifiers.push(ModifierTag::Const),
            "final" => result.modifiers.push(ModifierTag::Final),
            "internal" => result.modifiers.push(ModifierTag::Internal),
            "class" | "constructor" => result.is_class = true,
            "interface" => result.is_interface = true,
            "overload" => result.is_overload = true,
            "see" => result.see.push(rest.trim().to_string()),
            "example" => result.examples.push(rest.trim().to_string()),
            "author" => result.author = Some(rest.trim().to_string()),
            "version" => result.version = Some(rest.trim().to_string()),
            "since" => result.since = Some(rest.trim().to_string()),
            "property" | "prop" => {
                // Properties are handled within typedef parsing
                if let Some(prop) = self.parse_property_tag(rest) {
                    if let Some(typedef) = result.typedefs.last_mut() {
                        typedef.properties.push(prop);
                    }
                }
            }
            _ => {
                // Store unknown tags
                result.custom_tags
                    .entry(tag_name.to_string())
                    .or_insert_with(Vec::new)
                    .push(rest.trim().to_string());
            }
        }
    }

    /// Split tag name from rest of content
    fn split_tag(line: &str) -> (&str, &str) {
        if let Some(idx) = line.find(|c: char| c.is_whitespace()) {
            (&line[..idx], &line[idx..])
        } else {
            (line, "")
        }
    }

    /// Parse type expression from braces: {type}
    fn parse_type_from_braces(&mut self, s: &str) -> Option<JsDocTypeExpr> {
        let s = s.trim();
        if !s.starts_with('{') {
            return None;
        }

        let mut depth = 0;
        let mut end = 0;
        for (i, c) in s.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        if end == 0 {
            self.errors.push(ParseError::new("Unclosed type expression"));
            return None;
        }

        let type_str = &s[1..end];
        self.parse_type_expr(type_str)
    }

    /// Parse a type expression
    fn parse_type_expr(&mut self, s: &str) -> Option<JsDocTypeExpr> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        // Handle parenthesized types
        if s.starts_with('(') && s.ends_with(')') {
            return self.parse_type_expr(&s[1..s.len() - 1])
                .map(|t| JsDocTypeExpr::Parenthesized(Box::new(t)));
        }

        // Handle union types at top level (but not inside generics)
        if let Some(union) = self.try_parse_union(s) {
            return Some(union);
        }

        // Handle intersection types
        if let Some(intersection) = self.try_parse_intersection(s) {
            return Some(intersection);
        }

        // Handle prefix operators
        if s.starts_with('?') {
            return self.parse_type_expr(&s[1..])
                .map(|t| JsDocTypeExpr::Nullable(Box::new(t)));
        }
        if s.starts_with('!') {
            return self.parse_type_expr(&s[1..])
                .map(|t| JsDocTypeExpr::NonNullable(Box::new(t)));
        }
        if s.starts_with("...") {
            return self.parse_type_expr(&s[3..])
                .map(|t| JsDocTypeExpr::Rest(Box::new(t)));
        }

        // Handle array suffix
        if s.ends_with("[]") {
            return self.parse_type_expr(&s[..s.len() - 2])
                .map(|t| JsDocTypeExpr::Array(Box::new(t)));
        }

        // Handle special types
        match s {
            "*" => return Some(JsDocTypeExpr::Star),
            "?" => return Some(JsDocTypeExpr::Unknown),
            "void" => return Some(JsDocTypeExpr::Void),
            "this" => return Some(JsDocTypeExpr::This),
            "null" => return Some(JsDocTypeExpr::Literal(LiteralValue::Null)),
            "undefined" => return Some(JsDocTypeExpr::Literal(LiteralValue::Undefined)),
            "true" => return Some(JsDocTypeExpr::Literal(LiteralValue::Boolean(true))),
            "false" => return Some(JsDocTypeExpr::Literal(LiteralValue::Boolean(false))),
            _ => {}
        }

        // Handle typeof
        if s.starts_with("typeof ") {
            return Some(JsDocTypeExpr::TypeOf(s[7..].trim().to_string()));
        }

        // Handle keyof
        if s.starts_with("keyof ") {
            return self.parse_type_expr(&s[6..])
                .map(|t| JsDocTypeExpr::KeyOf(Box::new(t)));
        }

        // Handle string literals
        if (s.starts_with('"') && s.ends_with('"')) ||
           (s.starts_with('\'') && s.ends_with('\'')) {
            let content = &s[1..s.len() - 1];
            return Some(JsDocTypeExpr::Literal(LiteralValue::String(content.to_string())));
        }

        // Handle number literals
        if let Ok(n) = s.parse::<f64>() {
            return Some(JsDocTypeExpr::Literal(LiteralValue::Number(n)));
        }

        // Handle function type
        if s.starts_with("function") {
            return self.parse_function_type(s);
        }

        // Handle object type { ... }
        if s.starts_with('{') && s.ends_with('}') {
            return self.parse_object_type(&s[1..s.len() - 1]);
        }

        // Handle tuple type [A, B, C]
        if s.starts_with('[') && s.ends_with(']') {
            return self.parse_tuple_type(&s[1..s.len() - 1]);
        }

        // Handle import type
        if s.starts_with("import(") {
            return self.parse_import_type(s);
        }

        // Handle generic type reference: Name<A, B>
        if let Some(idx) = s.find('<') {
            if s.ends_with('>') {
                let name = &s[..idx];
                let args_str = &s[idx + 1..s.len() - 1];
                let type_args = self.parse_type_list(args_str);
                return Some(JsDocTypeExpr::Reference {
                    name: name.to_string(),
                    type_args,
                });
            }
        }

        // Simple type reference or primitive
        let primitives = ["string", "number", "boolean", "object", "symbol", "bigint", "any", "unknown", "never"];
        if primitives.contains(&s) {
            Some(JsDocTypeExpr::Primitive(s.to_string()))
        } else {
            Some(JsDocTypeExpr::Reference {
                name: s.to_string(),
                type_args: Vec::new(),
            })
        }
    }

    /// Try to parse a union type
    fn try_parse_union(&mut self, s: &str) -> Option<JsDocTypeExpr> {
        let parts = self.split_at_top_level(s, '|');
        if parts.len() > 1 {
            let types: Vec<_> = parts.into_iter()
                .filter_map(|p| self.parse_type_expr(p.trim()))
                .collect();
            if types.len() > 1 {
                return Some(JsDocTypeExpr::Union(types));
            }
        }
        None
    }

    /// Try to parse an intersection type
    fn try_parse_intersection(&mut self, s: &str) -> Option<JsDocTypeExpr> {
        let parts = self.split_at_top_level(s, '&');
        if parts.len() > 1 {
            let types: Vec<_> = parts.into_iter()
                .filter_map(|p| self.parse_type_expr(p.trim()))
                .collect();
            if types.len() > 1 {
                return Some(JsDocTypeExpr::Intersection(types));
            }
        }
        None
    }

    /// Split string at character but only at top level (not inside brackets)
    fn split_at_top_level<'b>(&self, s: &'b str, sep: char) -> Vec<&'b str> {
        let mut result = Vec::new();
        let mut depth = 0;
        let mut start = 0;

        for (i, c) in s.char_indices() {
            match c {
                '(' | '<' | '[' | '{' => depth += 1,
                ')' | '>' | ']' | '}' => depth -= 1,
                c if c == sep && depth == 0 => {
                    result.push(&s[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }
        result.push(&s[start..]);
        result
    }

    /// Parse a comma-separated list of types
    fn parse_type_list(&mut self, s: &str) -> Vec<JsDocTypeExpr> {
        self.split_at_top_level(s, ',')
            .into_iter()
            .filter_map(|p| self.parse_type_expr(p.trim()))
            .collect()
    }

    /// Parse function type: function(A, B): C
    fn parse_function_type(&mut self, s: &str) -> Option<JsDocTypeExpr> {
        let s = s.strip_prefix("function")?.trim();
        let is_new = s.starts_with("new");
        let s = if is_new { &s[3..].trim() } else { s };

        if !s.starts_with('(') {
            return Some(JsDocTypeExpr::Function {
                params: Vec::new(),
                return_type: None,
                this_type: None,
                is_new,
            });
        }

        // Find matching )
        let mut depth = 0;
        let mut params_end = 0;
        for (i, c) in s.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        params_end = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        let params_str = &s[1..params_end];
        let params = self.parse_type_list(params_str);

        let rest = &s[params_end + 1..].trim();
        let return_type = if rest.starts_with(':') {
            self.parse_type_expr(&rest[1..].trim())
                .map(Box::new)
        } else {
            None
        };

        Some(JsDocTypeExpr::Function {
            params,
            return_type,
            this_type: None,
            is_new,
        })
    }

    /// Parse object type: { a: A, b?: B }
    fn parse_object_type(&mut self, s: &str) -> Option<JsDocTypeExpr> {
        let parts = self.split_at_top_level(s, ',');
        let mut props = Vec::new();

        for part in parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let (name, rest, optional) = if let Some(idx) = part.find(':') {
                let name_part = &part[..idx].trim();
                let optional = name_part.ends_with('?');
                let name = if optional {
                    name_part[..name_part.len() - 1].trim()
                } else {
                    name_part
                };
                (name.to_string(), &part[idx + 1..], optional)
            } else {
                continue;
            };

            let readonly = name.starts_with("readonly ");
            let name = if readonly {
                name[9..].trim().to_string()
            } else {
                name
            };

            if let Some(type_expr) = self.parse_type_expr(rest.trim()) {
                props.push(PropertySignature {
                    name,
                    type_expr,
                    optional,
                    readonly,
                });
            }
        }

        Some(JsDocTypeExpr::Object(props))
    }

    /// Parse tuple type: [A, B, C]
    fn parse_tuple_type(&mut self, s: &str) -> Option<JsDocTypeExpr> {
        let types = self.parse_type_list(s);
        Some(JsDocTypeExpr::Tuple(types))
    }

    /// Parse import type: import("./module").Type
    fn parse_import_type(&mut self, s: &str) -> Option<JsDocTypeExpr> {
        let s = s.strip_prefix("import(")?;
        let end = s.find(')')?;
        let path = &s[..end].trim();
        let path = path.trim_matches(|c| c == '"' || c == '\'');

        let rest = &s[end + 1..];
        let qualifier = if rest.starts_with('.') {
            Some(rest[1..].to_string())
        } else {
            None
        };

        Some(JsDocTypeExpr::Import {
            path: path.to_string(),
            qualifier,
        })
    }

    /// Parse @param tag
    fn parse_param_tag(&mut self, s: &str) -> Option<ParamTag> {
        let s = s.trim();

        // Parse type if present
        let (type_expr, rest) = if s.starts_with('{') {
            let type_expr = self.parse_type_from_braces(s);
            let end = s.find('}').map(|i| i + 1).unwrap_or(0);
            (type_expr, &s[end..])
        } else {
            (None, s)
        };

        let rest = rest.trim();
        if rest.is_empty() {
            return None;
        }

        // Parse parameter name (may be [name], [name=default], etc.)
        let (name, optional, default_value, description) = if rest.starts_with('[') {
            // Optional parameter
            let end = rest.find(']')?;
            let inner = &rest[1..end];
            let (name, default) = if let Some(eq) = inner.find('=') {
                (inner[..eq].trim().to_string(), Some(inner[eq + 1..].trim().to_string()))
            } else {
                (inner.trim().to_string(), None)
            };
            let desc = rest[end + 1..].trim();
            let desc = if desc.is_empty() { None } else { Some(desc.to_string()) };
            (name, true, default, desc)
        } else {
            // Required parameter
            let (name, desc) = if let Some(idx) = rest.find(|c: char| c.is_whitespace()) {
                (&rest[..idx], Some(rest[idx..].trim().to_string()))
            } else {
                (rest, None)
            };
            (name.to_string(), false, None, desc.filter(|s| !s.is_empty()))
        };

        Some(ParamTag {
            name,
            type_expr,
            description,
            optional,
            default_value,
        })
    }

    /// Parse @returns tag
    fn parse_returns_tag(&mut self, s: &str) -> ReturnsTag {
        let s = s.trim();

        let (type_expr, rest) = if s.starts_with('{') {
            let type_expr = self.parse_type_from_braces(s);
            let end = s.find('}').map(|i| i + 1).unwrap_or(0);
            (type_expr, &s[end..])
        } else {
            (None, s)
        };

        let description = rest.trim();
        let description = if description.is_empty() {
            None
        } else {
            Some(description.to_string())
        };

        ReturnsTag {
            type_expr,
            description,
        }
    }

    /// Parse @typedef tag
    fn parse_typedef_tag(&mut self, s: &str) -> Option<TypedefTag> {
        let s = s.trim();

        let (type_expr, rest) = if s.starts_with('{') {
            let type_expr = self.parse_type_from_braces(s);
            let end = s.find('}').map(|i| i + 1).unwrap_or(0);
            (type_expr, &s[end..])
        } else {
            (None, s)
        };

        let name = rest.trim().split_whitespace().next()?.to_string();

        Some(TypedefTag {
            name,
            type_expr,
            properties: Vec::new(),
        })
    }

    /// Parse @callback tag
    fn parse_callback_tag(&mut self, s: &str) -> Option<CallbackTag> {
        let name = s.trim().split_whitespace().next()?.to_string();
        Some(CallbackTag {
            name,
            params: Vec::new(),
            returns: None,
        })
    }

    /// Parse @template tag
    fn parse_template_tag(&mut self, s: &str) -> Option<TemplateTag> {
        let s = s.trim();

        // Check for constraint: @template {Constraint} T
        let (constraint, rest) = if s.starts_with('{') {
            let constraint = self.parse_type_from_braces(s);
            let end = s.find('}').map(|i| i + 1).unwrap_or(0);
            (constraint, &s[end..])
        } else {
            (None, s)
        };

        let rest = rest.trim();

        // Parse comma-separated names (and optionally = default)
        let names: Vec<String> = rest.split(',')
            .map(|p| p.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if names.is_empty() {
            return None;
        }

        Some(TemplateTag {
            names,
            constraint,
            default: None,
            description: None,
        })
    }

    /// Parse @property tag
    fn parse_property_tag(&mut self, s: &str) -> Option<PropertyTag> {
        let s = s.trim();

        let (type_expr, rest) = if s.starts_with('{') {
            let type_expr = self.parse_type_from_braces(s);
            let end = s.find('}').map(|i| i + 1).unwrap_or(0);
            (type_expr, &s[end..])
        } else {
            (None, s)
        };

        let rest = rest.trim();

        let (name, optional, description) = if rest.starts_with('[') {
            let end = rest.find(']')?;
            let name = rest[1..end].trim().to_string();
            let desc = rest[end + 1..].trim();
            (name, true, if desc.is_empty() { None } else { Some(desc.to_string()) })
        } else {
            let (name, desc) = if let Some(idx) = rest.find(|c: char| c.is_whitespace()) {
                (&rest[..idx], Some(rest[idx..].trim().to_string()))
            } else {
                (rest, None)
            };
            (name.to_string(), false, desc.filter(|s| !s.is_empty()))
        };

        Some(PropertyTag {
            name,
            type_expr,
            description,
            optional,
        })
    }

    /// Get parsing errors
    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_content() {
        let input = "/**\n * Description\n * @param x\n */";
        let content = JsDocParser::extract_content(input);
        assert!(content.contains("Description"));
        assert!(content.contains("@param x"));
    }

    #[test]
    fn test_parse_primitive_types() {
        let mut parser = JsDocParser::new("");

        let string = parser.parse_type_expr("string").unwrap();
        assert!(matches!(string, JsDocTypeExpr::Primitive(s) if s == "string"));

        let number = parser.parse_type_expr("number").unwrap();
        assert!(matches!(number, JsDocTypeExpr::Primitive(s) if s == "number"));
    }

    #[test]
    fn test_parse_union_type() {
        let mut parser = JsDocParser::new("");
        let union = parser.parse_type_expr("string|number").unwrap();

        match union {
            JsDocTypeExpr::Union(types) => {
                assert_eq!(types.len(), 2);
            }
            _ => panic!("Expected union"),
        }
    }

    #[test]
    fn test_parse_array_type() {
        let mut parser = JsDocParser::new("");
        let array = parser.parse_type_expr("string[]").unwrap();

        assert!(matches!(array, JsDocTypeExpr::Array(_)));
    }

    #[test]
    fn test_parse_nullable_type() {
        let mut parser = JsDocParser::new("");
        let nullable = parser.parse_type_expr("?string").unwrap();

        assert!(matches!(nullable, JsDocTypeExpr::Nullable(_)));
    }

    #[test]
    fn test_parse_generic_type() {
        let mut parser = JsDocParser::new("");
        let generic = parser.parse_type_expr("Array<string>").unwrap();

        match generic {
            JsDocTypeExpr::Reference { name, type_args } => {
                assert_eq!(name, "Array");
                assert_eq!(type_args.len(), 1);
            }
            _ => panic!("Expected reference with type args"),
        }
    }

    #[test]
    fn test_parse_function_type() {
        let mut parser = JsDocParser::new("");
        let func = parser.parse_type_expr("function(string, number): boolean").unwrap();

        match func {
            JsDocTypeExpr::Function { params, return_type, .. } => {
                assert_eq!(params.len(), 2);
                assert!(return_type.is_some());
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_parse_object_type() {
        let mut parser = JsDocParser::new("");
        let obj = parser.parse_type_expr("{ name: string, age?: number }").unwrap();

        match obj {
            JsDocTypeExpr::Object(props) => {
                assert_eq!(props.len(), 2);
                assert!(!props[0].optional);
                assert!(props[1].optional);
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_tuple_type() {
        let mut parser = JsDocParser::new("");
        let tuple = parser.parse_type_expr("[string, number, boolean]").unwrap();

        match tuple {
            JsDocTypeExpr::Tuple(types) => {
                assert_eq!(types.len(), 3);
            }
            _ => panic!("Expected tuple"),
        }
    }

    #[test]
    fn test_parse_literal_types() {
        let mut parser = JsDocParser::new("");

        let str_lit = parser.parse_type_expr("\"hello\"").unwrap();
        assert!(matches!(str_lit, JsDocTypeExpr::Literal(LiteralValue::String(s)) if s == "hello"));

        let num_lit = parser.parse_type_expr("42").unwrap();
        assert!(matches!(num_lit, JsDocTypeExpr::Literal(LiteralValue::Number(n)) if n == 42.0));

        let bool_lit = parser.parse_type_expr("true").unwrap();
        assert!(matches!(bool_lit, JsDocTypeExpr::Literal(LiteralValue::Boolean(true))));
    }

    #[test]
    fn test_parse_param_tag() {
        let mut parser = JsDocParser::new("");

        let param = parser.parse_param_tag("{string} name The name").unwrap();
        assert_eq!(param.name, "name");
        assert!(param.type_expr.is_some());
        assert_eq!(param.description, Some("The name".to_string()));
        assert!(!param.optional);
    }

    #[test]
    fn test_parse_optional_param() {
        let mut parser = JsDocParser::new("");

        let param = parser.parse_param_tag("{string} [name] Optional name").unwrap();
        assert_eq!(param.name, "name");
        assert!(param.optional);
    }

    #[test]
    fn test_parse_param_with_default() {
        let mut parser = JsDocParser::new("");

        let param = parser.parse_param_tag("{string} [name=\"default\"] Name with default").unwrap();
        assert_eq!(param.name, "name");
        assert!(param.optional);
        assert_eq!(param.default_value, Some("\"default\"".to_string()));
    }

    #[test]
    fn test_parse_full_jsdoc() {
        let input = r#"/**
         * A function description
         * @param {string} name The name
         * @param {number} age The age
         * @returns {boolean} Success
         * @deprecated
         */
        "#;

        let result = JsDocParser::parse_comment(input).unwrap();

        assert!(result.description.is_some());
        assert_eq!(result.params.len(), 2);
        assert!(result.returns.is_some());
        assert!(result.modifiers.contains(&ModifierTag::Deprecated));
    }

    #[test]
    fn test_parse_typedef() {
        let input = r#"/**
         * @typedef {Object} Person
         * @property {string} name
         * @property {number} age
         */
        "#;

        let result = JsDocParser::parse_comment(input).unwrap();

        assert_eq!(result.typedefs.len(), 1);
        assert_eq!(result.typedefs[0].name, "Person");
    }

    #[test]
    fn test_parse_template() {
        let input = r#"/**
         * @template T
         * @template {Comparable} U
         * @param {T} value
         * @returns {U}
         */
        "#;

        let result = JsDocParser::parse_comment(input).unwrap();

        assert_eq!(result.templates.len(), 2);
        assert!(result.templates[0].constraint.is_none());
        assert!(result.templates[1].constraint.is_some());
    }

    #[test]
    fn test_parse_extends() {
        let input = r#"/**
         * @extends {BaseClass<string>}
         * @implements {IComparable}
         */
        "#;

        let result = JsDocParser::parse_comment(input).unwrap();

        assert_eq!(result.extends.len(), 1);
        assert_eq!(result.implements.len(), 1);
    }

    #[test]
    fn test_parse_modifiers() {
        let input = r#"/**
         * @deprecated Use newMethod instead
         * @readonly
         * @private
         */
        "#;

        let result = JsDocParser::parse_comment(input).unwrap();

        assert!(result.modifiers.contains(&ModifierTag::Deprecated));
        assert!(result.modifiers.contains(&ModifierTag::Readonly));
        assert!(result.modifiers.contains(&ModifierTag::Private));
    }

    #[test]
    fn test_parse_special_types() {
        let mut parser = JsDocParser::new("");

        assert!(matches!(parser.parse_type_expr("*"), Some(JsDocTypeExpr::Star)));
        assert!(matches!(parser.parse_type_expr("?"), Some(JsDocTypeExpr::Unknown)));
        assert!(matches!(parser.parse_type_expr("void"), Some(JsDocTypeExpr::Void)));
        assert!(matches!(parser.parse_type_expr("this"), Some(JsDocTypeExpr::This)));
    }

    #[test]
    fn test_parse_typeof() {
        let mut parser = JsDocParser::new("");
        let typeof_expr = parser.parse_type_expr("typeof MyClass").unwrap();

        assert!(matches!(typeof_expr, JsDocTypeExpr::TypeOf(s) if s == "MyClass"));
    }

    #[test]
    fn test_parse_import_type() {
        let mut parser = JsDocParser::new("");
        let import_expr = parser.parse_type_expr("import(\"./module\").Type").unwrap();

        match import_expr {
            JsDocTypeExpr::Import { path, qualifier } => {
                assert_eq!(path, "./module");
                assert_eq!(qualifier, Some("Type".to_string()));
            }
            _ => panic!("Expected import"),
        }
    }

    #[test]
    fn test_parse_callback() {
        let input = r#"/**
         * @callback MyCallback
         * @param {string} value
         * @returns {boolean}
         */
        "#;

        let result = JsDocParser::parse_comment(input).unwrap();

        assert_eq!(result.callbacks.len(), 1);
        assert_eq!(result.callbacks[0].name, "MyCallback");
    }

    #[test]
    fn test_parse_intersection() {
        let mut parser = JsDocParser::new("");
        let intersection = parser.parse_type_expr("TypeA & TypeB").unwrap();

        match intersection {
            JsDocTypeExpr::Intersection(types) => {
                assert_eq!(types.len(), 2);
            }
            _ => panic!("Expected intersection"),
        }
    }

    #[test]
    fn test_parse_complex_union() {
        let mut parser = JsDocParser::new("");
        let complex = parser.parse_type_expr("string | number | null").unwrap();

        match complex {
            JsDocTypeExpr::Union(types) => {
                assert_eq!(types.len(), 3);
            }
            _ => panic!("Expected union"),
        }
    }
}
