//! Diagnostic generation for the solver.
//!
//! This module provides error message generation for type checking failures.
//! It produces human-readable diagnostics with source locations and context.

use std::sync::Arc;
use crate::solver::types::*;
use crate::solver::intern::TypeInterner;

/// Diagnostic severity level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Suggestion,
    Message,
}

/// A source location span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceSpan {
    /// Start position (byte offset)
    pub start: u32,
    /// Length in bytes
    pub length: u32,
    /// File path or name
    pub file: Arc<str>,
}

impl SourceSpan {
    pub fn new(file: impl Into<Arc<str>>, start: u32, length: u32) -> Self {
        SourceSpan {
            start,
            length,
            file: file.into(),
        }
    }
}

/// Related diagnostic information (e.g., "see declaration here").
#[derive(Clone, Debug)]
pub struct RelatedInformation {
    pub span: SourceSpan,
    pub message: String,
}

/// A type checking diagnostic.
#[derive(Clone, Debug)]
pub struct TypeDiagnostic {
    /// The main error message
    pub message: String,
    /// Diagnostic code (e.g., 2322 for "Type X is not assignable to type Y")
    pub code: u32,
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Primary source location
    pub span: Option<SourceSpan>,
    /// Related information (additional locations)
    pub related: Vec<RelatedInformation>,
}

impl TypeDiagnostic {
    /// Create a new error diagnostic.
    pub fn error(message: impl Into<String>, code: u32) -> Self {
        TypeDiagnostic {
            message: message.into(),
            code,
            severity: DiagnosticSeverity::Error,
            span: None,
            related: Vec::new(),
        }
    }

    /// Add a source span to this diagnostic.
    pub fn with_span(mut self, span: SourceSpan) -> Self {
        self.span = Some(span);
        self
    }

    /// Add related information.
    pub fn with_related(mut self, span: SourceSpan, message: impl Into<String>) -> Self {
        self.related.push(RelatedInformation {
            span,
            message: message.into(),
        });
        self
    }
}

// =============================================================================
// Diagnostic Codes (matching TypeScript's)
// =============================================================================

/// TypeScript diagnostic codes for type errors.
pub mod codes {
    /// Type '{0}' is not assignable to type '{1}'.
    pub const TYPE_NOT_ASSIGNABLE: u32 = 2322;

    /// Argument of type '{0}' is not assignable to parameter of type '{1}'.
    pub const ARG_NOT_ASSIGNABLE: u32 = 2345;

    /// Property '{0}' is missing in type '{1}' but required in type '{2}'.
    pub const PROPERTY_MISSING: u32 = 2741;

    /// Property '{0}' does not exist on type '{1}'.
    pub const PROPERTY_NOT_EXIST: u32 = 2339;

    /// Type '{0}' has no properties in common with type '{1}'.
    pub const NO_COMMON_PROPERTIES: u32 = 2559;

    /// Cannot assign to '{0}' because it is a read-only property.
    pub const READONLY_PROPERTY: u32 = 2540;

    /// Type '{0}' is not assignable to type '{1}'.
    /// '{2}' is assignable to the constraint of type '{3}', but '{3}' could be instantiated with a different subtype.
    pub const CONSTRAINT_NOT_SATISFIED: u32 = 2344;

    /// Argument of type '{0}' is not assignable to parameter of type '{1}'.
    /// Types of property '{2}' are incompatible.
    pub const NESTED_TYPE_MISMATCH: u32 = 2322;

    /// The 'this' context of type '{0}' is not assignable to method's 'this' of type '{1}'.
    pub const THIS_CONTEXT_MISMATCH: u32 = 2684;

    /// Type 'never' is not a valid return type for an async function.
    pub const NEVER_ASYNC_RETURN: u32 = 1064;

    /// Cannot find name '{0}'.
    pub const CANNOT_FIND_NAME: u32 = 2304;

    /// This expression is not callable. Type '{0}' has no call signatures.
    pub const NOT_CALLABLE: u32 = 2349;

    /// Expected {0} arguments, but got {1}.
    pub const ARG_COUNT_MISMATCH: u32 = 2554;

    /// Object is possibly 'undefined'.
    pub const OBJECT_POSSIBLY_UNDEFINED: u32 = 2532;

    /// Object is possibly 'null'.
    pub const OBJECT_POSSIBLY_NULL: u32 = 2531;

    /// Object is of type 'unknown'.
    pub const OBJECT_IS_UNKNOWN: u32 = 2571;
}

// =============================================================================
// Type Formatting
// =============================================================================

/// Context for generating type strings.
pub struct TypeFormatter<'a> {
    interner: &'a TypeInterner,
    /// Maximum depth for nested type printing
    max_depth: u32,
    /// Current depth
    current_depth: u32,
}

impl<'a> TypeFormatter<'a> {
    pub fn new(interner: &'a TypeInterner) -> Self {
        TypeFormatter {
            interner,
            max_depth: 5,
            current_depth: 0,
        }
    }

    /// Format a type as a human-readable string.
    pub fn format(&mut self, type_id: TypeId) -> String {
        if self.current_depth >= self.max_depth {
            return "...".to_string();
        }

        // Handle intrinsic types
        match type_id {
            TypeId::NEVER => return "never".to_string(),
            TypeId::UNKNOWN => return "unknown".to_string(),
            TypeId::ANY => return "any".to_string(),
            TypeId::VOID => return "void".to_string(),
            TypeId::UNDEFINED => return "undefined".to_string(),
            TypeId::NULL => return "null".to_string(),
            TypeId::BOOLEAN => return "boolean".to_string(),
            TypeId::NUMBER => return "number".to_string(),
            TypeId::STRING => return "string".to_string(),
            TypeId::BIGINT => return "bigint".to_string(),
            TypeId::SYMBOL => return "symbol".to_string(),
            TypeId::OBJECT => return "object".to_string(),
            TypeId::ERROR => return "error".to_string(),
            _ => {}
        }

        let key = match self.interner.lookup(type_id) {
            Some(k) => k,
            None => return format!("Type({})", type_id.0),
        };

        self.current_depth += 1;
        let result = self.format_key(&key);
        self.current_depth -= 1;
        result
    }

    fn format_key(&mut self, key: &TypeKey) -> String {
        match key {
            TypeKey::Intrinsic(kind) => self.format_intrinsic(*kind),
            TypeKey::Literal(lit) => self.format_literal(lit),
            TypeKey::Object(props) => self.format_object(props),
            TypeKey::ObjectWithIndex(shape) => self.format_object_with_index(shape),
            TypeKey::Union(members) => self.format_union(members),
            TypeKey::Intersection(members) => self.format_intersection(members),
            TypeKey::Array(elem) => format!("{}[]", self.format(*elem)),
            TypeKey::Tuple(elements) => self.format_tuple(elements),
            TypeKey::Function(shape) => self.format_function(shape),
            TypeKey::Callable(shape) => self.format_callable(shape),
            TypeKey::TypeParameter(info) => info.name.to_string(),
            TypeKey::Ref(sym) => format!("Ref({})", sym.0),
            TypeKey::Conditional(cond) => self.format_conditional(cond),
            TypeKey::Mapped(mapped) => self.format_mapped(mapped),
            TypeKey::IndexAccess(obj, idx) => {
                format!("{}[{}]", self.format(*obj), self.format(*idx))
            }
            TypeKey::TemplateLiteral(spans) => self.format_template_literal(spans),
            TypeKey::TypeQuery(sym) => format!("typeof Ref({})", sym.0),
            TypeKey::KeyOf(operand) => format!("keyof {}", self.format(*operand)),
            TypeKey::ReadonlyType(inner) => format!("readonly {}", self.format(*inner)),
            TypeKey::UniqueSymbol(sym) => format!("unique symbol ({})", sym.0),
            TypeKey::Infer(info) => format!("infer {}", info.name),
            TypeKey::ThisType => "this".to_string(),
            TypeKey::Error => "error".to_string(),
        }
    }

    fn format_intrinsic(&self, kind: IntrinsicKind) -> String {
        match kind {
            IntrinsicKind::Any => "any",
            IntrinsicKind::Unknown => "unknown",
            IntrinsicKind::Never => "never",
            IntrinsicKind::Void => "void",
            IntrinsicKind::Null => "null",
            IntrinsicKind::Undefined => "undefined",
            IntrinsicKind::Boolean => "boolean",
            IntrinsicKind::Number => "number",
            IntrinsicKind::String => "string",
            IntrinsicKind::Bigint => "bigint",
            IntrinsicKind::Symbol => "symbol",
            IntrinsicKind::Object => "object",
        }.to_string()
    }

    fn format_literal(&self, lit: &LiteralValue) -> String {
        match lit {
            LiteralValue::String(s) => format!("\"{}\"", s),
            LiteralValue::Number(n) => format!("{}", n.0),
            LiteralValue::BigInt(b) => format!("{}n", b),
            LiteralValue::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
        }
    }

    fn format_object(&mut self, props: &[PropertyInfo]) -> String {
        if props.is_empty() {
            return "{}".to_string();
        }
        if props.len() > 3 {
            let first_three: Vec<String> = props.iter().take(3)
                .map(|p| self.format_property(p))
                .collect();
            return format!("{{ {}; ... }}", first_three.join("; "));
        }
        let formatted: Vec<String> = props.iter()
            .map(|p| self.format_property(p))
            .collect();
        format!("{{ {} }}", formatted.join("; "))
    }

    fn format_property(&mut self, prop: &PropertyInfo) -> String {
        let optional = if prop.optional { "?" } else { "" };
        let readonly = if prop.readonly { "readonly " } else { "" };
        format!("{}{}{}: {}", readonly, prop.name, optional, self.format(prop.type_id))
    }

    fn format_object_with_index(&mut self, shape: &ObjectShape) -> String {
        let mut parts = Vec::new();

        if let Some(ref idx) = shape.string_index {
            parts.push(format!("[key: string]: {}", self.format(idx.value_type)));
        }
        if let Some(ref idx) = shape.number_index {
            parts.push(format!("[key: number]: {}", self.format(idx.value_type)));
        }
        for prop in &shape.properties {
            parts.push(self.format_property(prop));
        }

        format!("{{ {} }}", parts.join("; "))
    }

    fn format_union(&mut self, members: &[TypeId]) -> String {
        if members.len() > 5 {
            let first_five: Vec<String> = members.iter().take(5)
                .map(|&m| self.format(m))
                .collect();
            return format!("{} | ...", first_five.join(" | "));
        }
        let formatted: Vec<String> = members.iter()
            .map(|&m| self.format(m))
            .collect();
        formatted.join(" | ")
    }

    fn format_intersection(&mut self, members: &[TypeId]) -> String {
        let formatted: Vec<String> = members.iter()
            .map(|&m| self.format(m))
            .collect();
        formatted.join(" & ")
    }

    fn format_tuple(&mut self, elements: &[TupleElement]) -> String {
        let formatted: Vec<String> = elements.iter()
            .map(|e| {
                let rest = if e.rest { "..." } else { "" };
                let optional = if e.optional { "?" } else { "" };
                if let Some(ref name) = e.name {
                    format!("{}{}: {}{}", name, optional, rest, self.format(e.type_id))
                } else {
                    format!("{}{}{}", rest, self.format(e.type_id), optional)
                }
            })
            .collect();
        format!("[{}]", formatted.join(", "))
    }

    fn format_function(&mut self, shape: &FunctionShape) -> String {
        let params: Vec<String> = shape.params.iter()
            .map(|p| {
                let name = p.name.as_ref().map(|n| n.as_ref()).unwrap_or("_");
                let optional = if p.optional { "?" } else { "" };
                let rest = if p.rest { "..." } else { "" };
                format!("{}{}{}: {}", rest, name, optional, self.format(p.type_id))
            })
            .collect();
        let arrow = if shape.is_constructor { "new " } else { "" };
        format!("{}({}) => {}", arrow, params.join(", "), self.format(shape.return_type))
    }

    fn format_callable(&mut self, shape: &CallableShape) -> String {
        let mut parts = Vec::new();
        for sig in &shape.call_signatures {
            parts.push(self.format_call_signature(sig, false));
        }
        for sig in &shape.construct_signatures {
            parts.push(self.format_call_signature(sig, true));
        }
        for prop in &shape.properties {
            parts.push(self.format_property(prop));
        }
        format!("{{ {} }}", parts.join("; "))
    }

    fn format_call_signature(&mut self, sig: &CallSignature, is_construct: bool) -> String {
        let params: Vec<String> = sig.params.iter()
            .map(|p| {
                let name = p.name.as_ref().map(|n| n.as_ref()).unwrap_or("_");
                format!("{}: {}", name, self.format(p.type_id))
            })
            .collect();
        let prefix = if is_construct { "new " } else { "" };
        format!("{}({}): {}", prefix, params.join(", "), self.format(sig.return_type))
    }

    fn format_conditional(&mut self, cond: &ConditionalType) -> String {
        format!(
            "{} extends {} ? {} : {}",
            self.format(cond.check_type),
            self.format(cond.extends_type),
            self.format(cond.true_type),
            self.format(cond.false_type)
        )
    }

    fn format_mapped(&mut self, mapped: &MappedType) -> String {
        format!(
            "{{ [K in {}]: {} }}",
            self.format(mapped.constraint),
            self.format(mapped.template)
        )
    }

    fn format_template_literal(&mut self, spans: &[TemplateSpan]) -> String {
        let mut result = String::from("`");
        for span in spans {
            match span {
                TemplateSpan::Text(text) => result.push_str(text),
                TemplateSpan::Type(type_id) => {
                    result.push_str("${");
                    result.push_str(&self.format(*type_id));
                    result.push('}');
                }
            }
        }
        result.push('`');
        result
    }
}

// =============================================================================
// Diagnostic Builder
// =============================================================================

/// Builder for creating type error diagnostics.
pub struct DiagnosticBuilder<'a> {
    interner: &'a TypeInterner,
    formatter: TypeFormatter<'a>,
}

impl<'a> DiagnosticBuilder<'a> {
    pub fn new(interner: &'a TypeInterner) -> Self {
        DiagnosticBuilder {
            interner,
            formatter: TypeFormatter::new(interner),
        }
    }

    /// Create a "Type X is not assignable to type Y" diagnostic.
    pub fn type_not_assignable(&mut self, source: TypeId, target: TypeId) -> TypeDiagnostic {
        let source_str = self.formatter.format(source);
        let target_str = self.formatter.format(target);
        TypeDiagnostic::error(
            format!("Type '{}' is not assignable to type '{}'.", source_str, target_str),
            codes::TYPE_NOT_ASSIGNABLE,
        )
    }

    /// Create a "Property X is missing in type Y" diagnostic.
    pub fn property_missing(&mut self, prop_name: &str, source: TypeId, target: TypeId) -> TypeDiagnostic {
        let source_str = self.formatter.format(source);
        let target_str = self.formatter.format(target);
        TypeDiagnostic::error(
            format!(
                "Property '{}' is missing in type '{}' but required in type '{}'.",
                prop_name, source_str, target_str
            ),
            codes::PROPERTY_MISSING,
        )
    }

    /// Create a "Property X does not exist on type Y" diagnostic.
    pub fn property_not_exist(&mut self, prop_name: &str, type_id: TypeId) -> TypeDiagnostic {
        let type_str = self.formatter.format(type_id);
        TypeDiagnostic::error(
            format!("Property '{}' does not exist on type '{}'.", prop_name, type_str),
            codes::PROPERTY_NOT_EXIST,
        )
    }

    /// Create an "Argument not assignable" diagnostic.
    pub fn argument_not_assignable(&mut self, arg_type: TypeId, param_type: TypeId) -> TypeDiagnostic {
        let arg_str = self.formatter.format(arg_type);
        let param_str = self.formatter.format(param_type);
        TypeDiagnostic::error(
            format!(
                "Argument of type '{}' is not assignable to parameter of type '{}'.",
                arg_str, param_str
            ),
            codes::ARG_NOT_ASSIGNABLE,
        )
    }

    /// Create a "Cannot find name" diagnostic.
    pub fn cannot_find_name(&mut self, name: &str) -> TypeDiagnostic {
        TypeDiagnostic::error(
            format!("Cannot find name '{}'.", name),
            codes::CANNOT_FIND_NAME,
        )
    }

    /// Create a "Type X is not callable" diagnostic.
    pub fn not_callable(&mut self, type_id: TypeId) -> TypeDiagnostic {
        let type_str = self.formatter.format(type_id);
        TypeDiagnostic::error(
            format!("Type '{}' has no call signatures.", type_str),
            codes::NOT_CALLABLE,
        )
    }

    /// Create an "Expected N arguments but got M" diagnostic.
    pub fn argument_count_mismatch(&mut self, expected: usize, got: usize) -> TypeDiagnostic {
        TypeDiagnostic::error(
            format!("Expected {} arguments, but got {}.", expected, got),
            codes::ARG_COUNT_MISMATCH,
        )
    }

    /// Create a "Cannot assign to readonly property" diagnostic.
    pub fn readonly_property(&mut self, prop_name: &str) -> TypeDiagnostic {
        TypeDiagnostic::error(
            format!("Cannot assign to '{}' because it is a read-only property.", prop_name),
            codes::READONLY_PROPERTY,
        )
    }
}

// =============================================================================
// Spanned Diagnostic Builder
// =============================================================================

/// A diagnostic builder that automatically attaches source spans.
///
/// This builder wraps `DiagnosticBuilder` and requires a file name and
/// position information for each diagnostic.
pub struct SpannedDiagnosticBuilder<'a> {
    builder: DiagnosticBuilder<'a>,
    file: Arc<str>,
}

impl<'a> SpannedDiagnosticBuilder<'a> {
    pub fn new(interner: &'a TypeInterner, file: impl Into<Arc<str>>) -> Self {
        SpannedDiagnosticBuilder {
            builder: DiagnosticBuilder::new(interner),
            file: file.into(),
        }
    }

    /// Create a span for this file.
    pub fn span(&self, start: u32, length: u32) -> SourceSpan {
        SourceSpan::new(self.file.clone(), start, length)
    }

    /// Create a "Type X is not assignable to type Y" diagnostic with span.
    pub fn type_not_assignable(
        &mut self,
        source: TypeId,
        target: TypeId,
        start: u32,
        length: u32,
    ) -> TypeDiagnostic {
        self.builder.type_not_assignable(source, target)
            .with_span(self.span(start, length))
    }

    /// Create a "Property X is missing" diagnostic with span.
    pub fn property_missing(
        &mut self,
        prop_name: &str,
        source: TypeId,
        target: TypeId,
        start: u32,
        length: u32,
    ) -> TypeDiagnostic {
        self.builder.property_missing(prop_name, source, target)
            .with_span(self.span(start, length))
    }

    /// Create a "Property X does not exist" diagnostic with span.
    pub fn property_not_exist(
        &mut self,
        prop_name: &str,
        type_id: TypeId,
        start: u32,
        length: u32,
    ) -> TypeDiagnostic {
        self.builder.property_not_exist(prop_name, type_id)
            .with_span(self.span(start, length))
    }

    /// Create an "Argument not assignable" diagnostic with span.
    pub fn argument_not_assignable(
        &mut self,
        arg_type: TypeId,
        param_type: TypeId,
        start: u32,
        length: u32,
    ) -> TypeDiagnostic {
        self.builder.argument_not_assignable(arg_type, param_type)
            .with_span(self.span(start, length))
    }

    /// Create a "Cannot find name" diagnostic with span.
    pub fn cannot_find_name(
        &mut self,
        name: &str,
        start: u32,
        length: u32,
    ) -> TypeDiagnostic {
        self.builder.cannot_find_name(name)
            .with_span(self.span(start, length))
    }

    /// Create an "Expected N arguments" diagnostic with span.
    pub fn argument_count_mismatch(
        &mut self,
        expected: usize,
        got: usize,
        start: u32,
        length: u32,
    ) -> TypeDiagnostic {
        self.builder.argument_count_mismatch(expected, got)
            .with_span(self.span(start, length))
    }

    /// Create a "Type is not callable" diagnostic with span.
    pub fn not_callable(
        &mut self,
        type_id: TypeId,
        start: u32,
        length: u32,
    ) -> TypeDiagnostic {
        self.builder.not_callable(type_id)
            .with_span(self.span(start, length))
    }

    /// Add a related location to an existing diagnostic.
    pub fn add_related(
        &self,
        diag: TypeDiagnostic,
        message: impl Into<String>,
        start: u32,
        length: u32,
    ) -> TypeDiagnostic {
        diag.with_related(self.span(start, length), message)
    }
}

// =============================================================================
// Diagnostic Conversion
// =============================================================================

/// Convert a solver TypeDiagnostic to a checker Diagnostic.
///
/// This allows the solver's diagnostic infrastructure to integrate
/// with the existing checker diagnostic system.
impl TypeDiagnostic {
    /// Convert to a checker::Diagnostic.
    ///
    /// Uses the provided file_name if no span is present.
    pub fn to_checker_diagnostic(&self, default_file: &str) -> crate::checker::state::Diagnostic {
        use crate::checker::state::{Diagnostic, DiagnosticCategory, DiagnosticRelatedInformation};

        let (file, start, length) = if let Some(ref span) = self.span {
            (span.file.to_string(), span.start, span.length)
        } else {
            (default_file.to_string(), 0, 0)
        };

        let category = match self.severity {
            DiagnosticSeverity::Error => DiagnosticCategory::Error,
            DiagnosticSeverity::Warning => DiagnosticCategory::Warning,
            DiagnosticSeverity::Suggestion => DiagnosticCategory::Suggestion,
            DiagnosticSeverity::Message => DiagnosticCategory::Message,
        };

        let related_information: Vec<DiagnosticRelatedInformation> = self.related.iter()
            .map(|rel| DiagnosticRelatedInformation {
                file: rel.span.file.to_string(),
                start: rel.span.start,
                length: rel.span.length,
                message_text: rel.message.clone(),
                category: DiagnosticCategory::Message,
                code: 0,
            })
            .collect();

        Diagnostic {
            file,
            start,
            length,
            message_text: self.message.clone(),
            category,
            code: self.code,
            related_information,
        }
    }
}

// =============================================================================
// Source Location Tracker
// =============================================================================

/// Tracks source locations for AST nodes during type checking.
///
/// This struct provides a convenient way to associate type checking
/// operations with their source locations for diagnostic generation.
#[derive(Clone)]
pub struct SourceLocation {
    /// File name
    pub file: Arc<str>,
    /// Start position (byte offset)
    pub start: u32,
    /// End position (byte offset)
    pub end: u32,
}

impl SourceLocation {
    pub fn new(file: impl Into<Arc<str>>, start: u32, end: u32) -> Self {
        SourceLocation {
            file: file.into(),
            start,
            end,
        }
    }

    /// Get the length of this location.
    pub fn length(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Convert to a SourceSpan.
    pub fn to_span(&self) -> SourceSpan {
        SourceSpan::new(self.file.clone(), self.start, self.length())
    }
}

/// A diagnostic collector that accumulates diagnostics with source tracking.
pub struct DiagnosticCollector<'a> {
    interner: &'a TypeInterner,
    file: Arc<str>,
    diagnostics: Vec<TypeDiagnostic>,
}

impl<'a> DiagnosticCollector<'a> {
    pub fn new(interner: &'a TypeInterner, file: impl Into<Arc<str>>) -> Self {
        DiagnosticCollector {
            interner,
            file: file.into(),
            diagnostics: Vec::new(),
        }
    }

    /// Get the collected diagnostics.
    pub fn diagnostics(&self) -> &[TypeDiagnostic] {
        &self.diagnostics
    }

    /// Take the collected diagnostics.
    pub fn take_diagnostics(&mut self) -> Vec<TypeDiagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    /// Report a type not assignable error.
    pub fn type_not_assignable(
        &mut self,
        source: TypeId,
        target: TypeId,
        loc: &SourceLocation,
    ) {
        let mut builder = DiagnosticBuilder::new(self.interner);
        let diag = builder.type_not_assignable(source, target)
            .with_span(loc.to_span());
        self.diagnostics.push(diag);
    }

    /// Report a property missing error.
    pub fn property_missing(
        &mut self,
        prop_name: &str,
        source: TypeId,
        target: TypeId,
        loc: &SourceLocation,
    ) {
        let mut builder = DiagnosticBuilder::new(self.interner);
        let diag = builder.property_missing(prop_name, source, target)
            .with_span(loc.to_span());
        self.diagnostics.push(diag);
    }

    /// Report a property not exist error.
    pub fn property_not_exist(
        &mut self,
        prop_name: &str,
        type_id: TypeId,
        loc: &SourceLocation,
    ) {
        let mut builder = DiagnosticBuilder::new(self.interner);
        let diag = builder.property_not_exist(prop_name, type_id)
            .with_span(loc.to_span());
        self.diagnostics.push(diag);
    }

    /// Report an argument not assignable error.
    pub fn argument_not_assignable(
        &mut self,
        arg_type: TypeId,
        param_type: TypeId,
        loc: &SourceLocation,
    ) {
        let mut builder = DiagnosticBuilder::new(self.interner);
        let diag = builder.argument_not_assignable(arg_type, param_type)
            .with_span(loc.to_span());
        self.diagnostics.push(diag);
    }

    /// Report a cannot find name error.
    pub fn cannot_find_name(&mut self, name: &str, loc: &SourceLocation) {
        let mut builder = DiagnosticBuilder::new(self.interner);
        let diag = builder.cannot_find_name(name)
            .with_span(loc.to_span());
        self.diagnostics.push(diag);
    }

    /// Report an argument count mismatch error.
    pub fn argument_count_mismatch(
        &mut self,
        expected: usize,
        got: usize,
        loc: &SourceLocation,
    ) {
        let mut builder = DiagnosticBuilder::new(self.interner);
        let diag = builder.argument_count_mismatch(expected, got)
            .with_span(loc.to_span());
        self.diagnostics.push(diag);
    }

    /// Convert all collected diagnostics to checker diagnostics.
    pub fn to_checker_diagnostics(&self) -> Vec<crate::checker::state::Diagnostic> {
        self.diagnostics.iter()
            .map(|d| d.to_checker_diagnostic(&self.file))
            .collect()
    }
}

#[cfg(test)]
#[path = "diagnostics_tests.rs"]
mod tests;
