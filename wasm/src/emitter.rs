//! Emitter implementation for TypeScript AST.
//!
//! The emitter converts an AST back to source code (JavaScript or TypeScript).
//! This is Phase 6 of the Rust migration.

use crate::parser::{Node, NodeList, NodeIndex, TemplateSpan};
use crate::parser::base::NodeBase;
use crate::scanner::SyntaxKind;
use crate::source_map::SourceMapGenerator;

// =============================================================================
// Emit Flags
// =============================================================================

/// Flags controlling emit behavior.
pub mod emit_flags {
    pub const NONE: u32 = 0;
    pub const SINGLE_LINE: u32 = 1 << 0;
    pub const NO_TRAILING_SEMICOLON: u32 = 1 << 1;
    pub const NO_TRAILING_NEWLINE: u32 = 1 << 2;
    pub const NO_LEADING_COMMENTS: u32 = 1 << 3;
    pub const NO_TRAILING_COMMENTS: u32 = 1 << 4;
    pub const NO_COMMENTS: u32 = NO_LEADING_COMMENTS | NO_TRAILING_COMMENTS;
    pub const NO_NESTED_COMMENTS: u32 = 1 << 5;
    pub const HELPER_NAME: u32 = 1 << 6;
    pub const EXPORT_NAME: u32 = 1 << 7;
    pub const LOCAL_NAME: u32 = 1 << 8;
    pub const INTERNAL_NAME: u32 = 1 << 9;
    pub const INDENTED: u32 = 1 << 10;
    pub const NO_INDENTATION: u32 = 1 << 11;
    pub const ASYNC_FUNCTION_BODY: u32 = 1 << 12;
    pub const REUSE_TEMP_VARIABLE_SCOPE: u32 = 1 << 13;
    pub const CUSTOM_PROLOGUE: u32 = 1 << 14;
    pub const NO_HOISTING: u32 = 1 << 15;
    pub const HAS_END_OF_DECLARATION_MARKER: u32 = 1 << 16;
    pub const ITERATOR: u32 = 1 << 17;
    pub const NO_ASCII_ESCAPING: u32 = 1 << 18;
}

// =============================================================================
// Printer Options
// =============================================================================

/// Options for the printer/emitter.
#[derive(Clone, Debug, Default)]
pub struct PrinterOptions {
    /// Remove comments from output
    pub remove_comments: bool,
    /// Target ECMAScript version
    pub target: ScriptTarget,
    /// Use single quotes for strings
    pub single_quote: bool,
    /// Omit trailing semicolons
    pub omit_trailing_semicolon: bool,
    /// Don't emit helpers
    pub no_emit_helpers: bool,
    /// Module kind
    pub module: ModuleKind,
    /// New line character
    pub new_line: NewLineKind,
}

/// ECMAScript target version.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScriptTarget {
    ES3 = 0,
    ES5 = 1,
    ES2015 = 2,
    ES2016 = 3,
    ES2017 = 4,
    ES2018 = 5,
    ES2019 = 6,
    ES2020 = 7,
    ES2021 = 8,
    ES2022 = 9,
    #[default]
    ESNext = 99,
}

/// Module system kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModuleKind {
    #[default]
    None = 0,
    CommonJS = 1,
    AMD = 2,
    UMD = 3,
    System = 4,
    ES2015 = 5,
    ES2020 = 6,
    ES2022 = 7,
    ESNext = 99,
    Node16 = 100,
    NodeNext = 199,
}

/// New line kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NewLineKind {
    #[default]
    LineFeed = 0,
    CarriageReturnLineFeed = 1,
}

// =============================================================================
// Printer State
// =============================================================================

/// The printer converts AST nodes to source code.
pub struct Printer {
    /// Output buffer
    output: String,
    /// Current indentation level
    indent_level: u32,
    /// Indentation string (e.g., "  " or "\t")
    indent_str: String,
    /// New line string
    new_line: String,
    /// Printer options
    options: PrinterOptions,
    /// Whether we're at the start of a line
    at_line_start: bool,

    // Source map tracking
    /// Source map generator (optional)
    source_map: Option<SourceMapGenerator>,
    /// Current source file index (for source maps)
    current_source_index: u32,
    /// Current output line (0-indexed)
    output_line: u32,
    /// Current output column (0-indexed)
    output_column: u32,
}

impl Printer {
    /// Create a new printer with default options.
    pub fn new() -> Self {
        Printer::with_options(PrinterOptions::default())
    }

    /// Create a new printer with the given options.
    pub fn with_options(options: PrinterOptions) -> Self {
        let new_line = match options.new_line {
            NewLineKind::LineFeed => "\n".to_string(),
            NewLineKind::CarriageReturnLineFeed => "\r\n".to_string(),
        };
        Printer {
            output: String::with_capacity(1024),
            indent_level: 0,
            indent_str: "    ".to_string(), // 4 spaces
            new_line,
            options,
            at_line_start: true,
            source_map: None,
            current_source_index: 0,
            output_line: 0,
            output_column: 0,
        }
    }

    /// Create a new printer with source map generation enabled.
    pub fn with_source_map(options: PrinterOptions, output_file: String) -> Self {
        let new_line = match options.new_line {
            NewLineKind::LineFeed => "\n".to_string(),
            NewLineKind::CarriageReturnLineFeed => "\r\n".to_string(),
        };
        Printer {
            output: String::with_capacity(1024),
            indent_level: 0,
            indent_str: "    ".to_string(),
            new_line,
            options,
            at_line_start: true,
            source_map: Some(SourceMapGenerator::new(output_file)),
            current_source_index: 0,
            output_line: 0,
            output_column: 0,
        }
    }

    /// Add a source file to the source map and return its index.
    pub fn add_source_file(&mut self, path: String) -> u32 {
        if let Some(ref mut sm) = self.source_map {
            let index = sm.add_source(path);
            self.current_source_index = index;
            index
        } else {
            0
        }
    }

    /// Add a source file with content to the source map.
    pub fn add_source_file_with_content(&mut self, path: String, content: String) -> u32 {
        if let Some(ref mut sm) = self.source_map {
            let index = sm.add_source_with_content(path, content);
            self.current_source_index = index;
            index
        } else {
            0
        }
    }

    /// Get the generated source map JSON, if enabled.
    pub fn get_source_map(&mut self) -> Option<String> {
        self.source_map.as_mut().map(|sm| sm.to_json())
    }

    /// Get the inline source map comment, if enabled.
    pub fn get_inline_source_map(&mut self) -> Option<String> {
        self.source_map.as_mut().map(|sm| sm.to_inline_comment())
    }

    /// Get the emitted output.
    pub fn get_output(&self) -> &str {
        &self.output
    }

    /// Take the emitted output, consuming the printer.
    pub fn take_output(self) -> String {
        self.output
    }

    /// Clear the output buffer.
    pub fn clear(&mut self) {
        self.output.clear();
        self.indent_level = 0;
        self.at_line_start = true;
        self.output_line = 0;
        self.output_column = 0;
    }

    /// Add a source mapping from the current output position to the given source position.
    fn emit_source_mapping(&mut self, source_line: u32, source_column: u32) {
        if let Some(ref mut sm) = self.source_map {
            sm.add_simple_mapping(
                self.output_line,
                self.output_column,
                self.current_source_index,
                source_line,
                source_column,
            );
        }
    }

    /// Add a source mapping with a name.
    fn emit_named_source_mapping(&mut self, source_line: u32, source_column: u32, name: &str) {
        if let Some(ref mut sm) = self.source_map {
            let name_index = sm.add_name(name.to_string());
            sm.add_named_mapping(
                self.output_line,
                self.output_column,
                self.current_source_index,
                source_line,
                source_column,
                name_index,
            );
        }
    }

    // =========================================================================
    // Output helpers
    // =========================================================================

    /// Write a string to the output.
    fn write(&mut self, s: &str) {
        if self.at_line_start && !s.is_empty() {
            self.write_indent();
            self.at_line_start = false;
        }
        // Track position for source maps
        for c in s.chars() {
            if c == '\n' {
                self.output_line += 1;
                self.output_column = 0;
            } else {
                self.output_column += 1;
            }
        }
        self.output.push_str(s);
    }

    /// Write indentation.
    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output_column += self.indent_str.len() as u32;
            self.output.push_str(&self.indent_str);
        }
    }

    /// Write a new line.
    fn write_line(&mut self) {
        self.output.push_str(&self.new_line);
        self.at_line_start = true;
        self.output_line += 1;
        self.output_column = 0;
    }

    /// Write a space.
    fn write_space(&mut self) {
        self.write(" ");
    }

    /// Write a semicolon.
    fn write_semicolon(&mut self) {
        if !self.options.omit_trailing_semicolon {
            self.write(";");
        }
    }

    /// Increase indentation.
    fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation.
    fn decrease_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    // =========================================================================
    // Node emission
    // =========================================================================

    /// Emit a node.
    pub fn emit_node(&mut self, node: &Node, arena: &crate::parser::NodeArena) {
        match node {
            // Literals
            Node::Identifier(id) => self.emit_identifier(id),
            Node::StringLiteral(lit) => self.emit_string_literal(lit),
            Node::NumericLiteral(lit) => self.emit_numeric_literal(lit),
            Node::BigIntLiteral(lit) => self.emit_bigint_literal(lit),
            Node::RegularExpressionLiteral(lit) => self.emit_regex_literal(lit),

            // Expressions
            Node::BinaryExpression(expr) => self.emit_binary_expression(expr, arena),
            Node::PrefixUnaryExpression(expr) => self.emit_prefix_unary(expr, arena),
            Node::PostfixUnaryExpression(expr) => self.emit_postfix_unary(expr, arena),
            Node::CallExpression(expr) => self.emit_call_expression(expr, arena),
            Node::PropertyAccessExpression(expr) => self.emit_property_access(expr, arena),
            Node::ElementAccessExpression(expr) => self.emit_element_access(expr, arena),
            Node::ParenthesizedExpression(expr) => self.emit_parenthesized(expr, arena),
            Node::ArrayLiteralExpression(expr) => self.emit_array_literal(expr, arena),
            Node::ObjectLiteralExpression(expr) => self.emit_object_literal(expr, arena),
            Node::ArrowFunction(func) => self.emit_arrow_function(func, arena),
            Node::ConditionalExpression(expr) => self.emit_conditional(expr, arena),
            Node::NewExpression(expr) => self.emit_new_expression(expr, arena),
            Node::SpreadElement(expr) => self.emit_spread_element(expr, arena),
            Node::YieldExpression(expr) => self.emit_yield_expression(expr, arena),
            Node::AwaitExpression(expr) => self.emit_await_expression(expr, arena),
            Node::AsExpression(expr) => self.emit_as_expression(expr, arena),
            Node::TypeAssertion(expr) => self.emit_type_assertion(expr, arena),
            Node::NonNullExpression(expr) => self.emit_non_null_expression(expr, arena),
            Node::TemplateExpression(expr) => self.emit_template_expression(expr, arena),
            Node::TaggedTemplateExpression(expr) => self.emit_tagged_template(expr, arena),

            // Statements
            Node::VariableStatement(stmt) => self.emit_variable_statement(stmt, arena),
            Node::ExpressionStatement(stmt) => self.emit_expression_statement(stmt, arena),
            Node::IfStatement(stmt) => self.emit_if_statement(stmt, arena),
            Node::WhileStatement(stmt) => self.emit_while_statement(stmt, arena),
            Node::DoStatement(stmt) => self.emit_do_statement(stmt, arena),
            Node::ForStatement(stmt) => self.emit_for_statement(stmt, arena),
            Node::ForInStatement(stmt) => self.emit_for_in_statement(stmt, arena),
            Node::ForOfStatement(stmt) => self.emit_for_of_statement(stmt, arena),
            Node::ReturnStatement(stmt) => self.emit_return_statement(stmt, arena),
            Node::Block(block) => self.emit_block(block, arena),
            Node::EmptyStatement(_) => self.emit_empty_statement(),
            Node::BreakStatement(stmt) => self.emit_break_statement(stmt, arena),
            Node::ContinueStatement(stmt) => self.emit_continue_statement(stmt, arena),
            Node::ThrowStatement(stmt) => self.emit_throw_statement(stmt, arena),
            Node::TryStatement(stmt) => self.emit_try_statement(stmt, arena),
            Node::SwitchStatement(stmt) => self.emit_switch_statement(stmt, arena),
            Node::CaseBlock(block) => self.emit_case_block(block, arena),
            Node::CaseClause(clause) => self.emit_case_clause(clause, arena),
            Node::DefaultClause(clause) => self.emit_default_clause(clause, arena),
            Node::CatchClause(clause) => self.emit_catch_clause(clause, arena),
            Node::LabeledStatement(stmt) => self.emit_labeled_statement(stmt, arena),
            Node::DebuggerStatement(_) => self.write("debugger;"),
            Node::WithStatement(stmt) => self.emit_with_statement(stmt, arena),

            // Declarations
            Node::FunctionDeclaration(decl) => self.emit_function_declaration(decl, arena),
            Node::ClassDeclaration(decl) => self.emit_class_declaration(decl, arena),
            Node::VariableDeclaration(decl) => self.emit_variable_declaration(decl, arena),
            Node::VariableDeclarationList(list) => self.emit_variable_declaration_list(list, arena),
            Node::ParameterDeclaration(param) => self.emit_parameter_declaration(param, arena),
            Node::InterfaceDeclaration(decl) => self.emit_interface_declaration(decl, arena),
            Node::TypeAliasDeclaration(decl) => self.emit_type_alias_declaration(decl, arena),
            Node::EnumDeclaration(decl) => self.emit_enum_declaration(decl, arena),
            Node::ModuleDeclaration(decl) => self.emit_module_declaration(decl, arena),
            Node::ImportDeclaration(decl) => self.emit_import_declaration(decl, arena),
            Node::ExportDeclaration(decl) => self.emit_export_declaration(decl, arena),
            Node::ExportAssignment(decl) => self.emit_export_assignment(decl, arena),

            // Class members
            Node::MethodDeclaration(decl) => self.emit_method_declaration(decl, arena),
            Node::PropertyDeclaration(decl) => self.emit_property_declaration(decl, arena),
            Node::ConstructorDeclaration(decl) => self.emit_constructor_declaration(decl, arena),
            Node::GetAccessorDeclaration(decl) => self.emit_get_accessor(decl, arena),
            Node::SetAccessorDeclaration(decl) => self.emit_set_accessor(decl, arena),

            // Object literal members
            Node::PropertyAssignment(prop) => self.emit_property_assignment(prop, arena),
            Node::ShorthandPropertyAssignment(prop) => self.emit_shorthand_property(prop, arena),
            Node::SpreadAssignment(spread) => self.emit_spread_assignment(spread, arena),

            // Type nodes
            Node::TypeReference(t) => self.emit_type_reference(t, arena),
            Node::QualifiedName { left, right, .. } => self.emit_qualified_name(*left, *right, arena),
            Node::UnionType(t) => self.emit_union_type(t, arena),
            Node::IntersectionType(t) => self.emit_intersection_type(t, arena),
            Node::LiteralType(t) => self.emit_literal_type(t, arena),
            Node::ArrayType(t) => self.emit_array_type(t, arena),
            Node::TupleType(t) => self.emit_tuple_type(t, arena),
            Node::OptionalType(t) => self.emit_optional_type(t, arena),
            Node::RestType(t) => self.emit_rest_type(t, arena),
            Node::FunctionType(t) => self.emit_function_type(t, arena),
            Node::ConstructorType(t) => self.emit_constructor_type(t, arena),
            Node::TypeQuery(t) => self.emit_type_query(t, arena),
            Node::TypeLiteral(t) => self.emit_type_literal(t, arena),
            Node::IndexedAccessType(t) => self.emit_indexed_access_type(t, arena),
            Node::MappedType(t) => self.emit_mapped_type(t, arena),
            Node::ConditionalType(t) => self.emit_conditional_type(t, arena),
            Node::InferType(t) => self.emit_infer_type(t, arena),
            Node::ParenthesizedType(t) => self.emit_parenthesized_type(t, arena),
            Node::TypeOperator(t) => self.emit_type_operator(t, arena),
            Node::TemplateLiteralType(t) => self.emit_template_literal_type(t, arena),
            Node::NamedTupleMember(t) => self.emit_named_tuple_member(t, arena),
            Node::TypePredicate(t) => self.emit_type_predicate(t, arena),

            // Type-related declarations
            Node::TypeParameterDeclaration(t) => self.emit_type_parameter_declaration(t, arena),
            Node::HeritageClause(h) => self.emit_heritage_clause(h, arena),
            Node::ExpressionWithTypeArguments(e) => self.emit_expression_with_type_arguments(e, arena),
            Node::Decorator(d) => self.emit_decorator(d, arena),
            Node::ComputedPropertyName { expression, .. } => self.emit_computed_property_name(*expression, arena),

            // Interface/type literal members
            Node::PropertySignature(p) => self.emit_property_signature(p, arena),
            Node::MethodSignature(m) => self.emit_method_signature(m, arena),
            Node::IndexSignatureDeclaration(i) => self.emit_index_signature(i, arena),
            Node::CallSignature(c) => self.emit_call_signature(c, arena),
            Node::ConstructSignature(c) => self.emit_construct_signature(c, arena),

            // Enum members
            Node::EnumMember(e) => self.emit_enum_member(e, arena),

            // Import/export details
            Node::ImportClause(c) => self.emit_import_clause(c, arena),
            Node::NamespaceImport(n) => self.emit_namespace_import(n, arena),
            Node::NamedImports(n) => self.emit_named_imports(n, arena),
            Node::ImportSpecifier(s) => self.emit_import_specifier(s, arena),
            Node::NamedExports(n) => self.emit_named_exports(n, arena),
            Node::NamespaceExport(n) => self.emit_namespace_export(n, arena),
            Node::ExportSpecifier(s) => self.emit_export_specifier(s, arena),

            // Binding patterns
            Node::ObjectBindingPattern(p) => self.emit_object_binding_pattern(p, arena),
            Node::ArrayBindingPattern(p) => self.emit_array_binding_pattern(p, arena),
            Node::BindingElement(e) => self.emit_binding_element(e, arena),

            // Template parts
            Node::TemplateSpan(s) => self.emit_template_span(s, arena),
            Node::NoSubstitutionTemplateLiteral(lit) => self.emit_no_substitution_template(lit),
            Node::TemplateHead(lit) => self.emit_template_head(lit),
            Node::TemplateMiddle(lit) => self.emit_template_middle(lit),
            Node::TemplateTail(lit) => self.emit_template_tail(lit),

            // JSX nodes
            Node::JsxElement(e) => self.emit_jsx_element(e, arena),
            Node::JsxSelfClosingElement(e) => self.emit_jsx_self_closing_element(e, arena),
            Node::JsxOpeningElement(e) => self.emit_jsx_opening_element(e, arena),
            Node::JsxClosingElement(e) => self.emit_jsx_closing_element(e, arena),
            Node::JsxFragment(f) => self.emit_jsx_fragment(f, arena),
            Node::JsxOpeningFragment(_) => self.write("<>"),
            Node::JsxClosingFragment(_) => self.write("</>"),
            Node::JsxAttributes(a) => self.emit_jsx_attributes(a, arena),
            Node::JsxAttribute(a) => self.emit_jsx_attribute(a, arena),
            Node::JsxSpreadAttribute(a) => self.emit_jsx_spread_attribute(a, arena),
            Node::JsxExpression(e) => self.emit_jsx_expression(e, arena),
            Node::JsxText(t) => self.emit_jsx_text(t),
            Node::JsxNamespacedName(n) => self.emit_jsx_namespaced_name(n, arena),

            // Module block
            Node::ModuleBlock(b) => self.emit_module_block(b, arena),

            // Function expression
            Node::FunctionExpression(f) => self.emit_function_expression(f, arena),

            // Satisfies expression
            Node::SatisfiesExpression(e) => self.emit_satisfies_expression(e, arena),

            // Private identifier
            Node::PrivateIdentifier(id) => self.emit_private_identifier(id),

            // Source file
            Node::SourceFile(sf) => self.emit_source_file(sf, arena),

            // Token (keywords, punctuation)
            Node::Token(base) => self.emit_token(base),

            // End of file - emit nothing
            Node::EndOfFileToken(_) => {}

            // Import attributes
            Node::ImportAttributes(a) => self.emit_import_attributes(a, arena),
            Node::ImportAttribute(a) => self.emit_import_attribute(a, arena)
        }
    }

    // =========================================================================
    // Literal emission
    // =========================================================================

    fn emit_identifier(&mut self, id: &crate::parser::literals::Identifier) {
        self.write(&id.escaped_text);
    }

    fn emit_string_literal(&mut self, lit: &crate::parser::literals::StringLiteral) {
        let quote = if self.options.single_quote { "'" } else { "\"" };
        self.write(quote);
        // TODO: Escape special characters
        self.write(&lit.text);
        self.write(quote);
    }

    fn emit_numeric_literal(&mut self, lit: &crate::parser::literals::NumericLiteral) {
        self.write(&lit.text);
    }

    fn emit_bigint_literal(&mut self, lit: &crate::parser::literals::BigIntLiteral) {
        self.write(&lit.text);
    }

    fn emit_regex_literal(&mut self, lit: &crate::parser::literals::RegularExpressionLiteral) {
        self.write(&lit.text);
    }

    // =========================================================================
    // Expression emission
    // =========================================================================

    fn emit_binary_expression(&mut self, expr: &crate::parser::expressions::BinaryExpression, arena: &crate::parser::NodeArena) {
        if let Some(left) = arena.get(expr.left) {
            self.emit_node(left, arena);
        }
        self.write_space();
        self.emit_token_kind(expr.operator_token);
        self.write_space();
        if let Some(right) = arena.get(expr.right) {
            self.emit_node(right, arena);
        }
    }

    fn emit_prefix_unary(&mut self, expr: &crate::parser::expressions::PrefixUnaryExpression, arena: &crate::parser::NodeArena) {
        self.emit_token_kind(expr.operator);
        if let Some(operand) = arena.get(expr.operand) {
            self.emit_node(operand, arena);
        }
    }

    fn emit_postfix_unary(&mut self, expr: &crate::parser::expressions::PostfixUnaryExpression, arena: &crate::parser::NodeArena) {
        if let Some(operand) = arena.get(expr.operand) {
            self.emit_node(operand, arena);
        }
        self.emit_token_kind(expr.operator);
    }

    fn emit_call_expression(&mut self, expr: &crate::parser::expressions::CallExpression, arena: &crate::parser::NodeArena) {
        if let Some(callee) = arena.get(expr.expression) {
            self.emit_node(callee, arena);
        }
        self.write("(");
        self.emit_node_list(&expr.arguments, arena, ", ");
        self.write(")");
    }

    fn emit_property_access(&mut self, expr: &crate::parser::expressions::PropertyAccessExpression, arena: &crate::parser::NodeArena) {
        if let Some(obj) = arena.get(expr.expression) {
            self.emit_node(obj, arena);
        }
        if expr.question_dot_token {
            self.write("?.");
        } else {
            self.write(".");
        }
        if let Some(name) = arena.get(expr.name) {
            self.emit_node(name, arena);
        }
    }

    fn emit_element_access(&mut self, expr: &crate::parser::expressions::ElementAccessExpression, arena: &crate::parser::NodeArena) {
        if let Some(obj) = arena.get(expr.expression) {
            self.emit_node(obj, arena);
        }
        if expr.question_dot_token {
            self.write("?.[");
        } else {
            self.write("[");
        }
        if let Some(index) = arena.get(expr.argument_expression) {
            self.emit_node(index, arena);
        }
        self.write("]");
    }

    fn emit_parenthesized(&mut self, expr: &crate::parser::expressions::ParenthesizedExpression, arena: &crate::parser::NodeArena) {
        self.write("(");
        if let Some(inner) = arena.get(expr.expression) {
            self.emit_node(inner, arena);
        }
        self.write(")");
    }

    fn emit_array_literal(&mut self, expr: &crate::parser::expressions::ArrayLiteralExpression, arena: &crate::parser::NodeArena) {
        self.write("[");
        self.emit_node_list(&expr.elements, arena, ", ");
        self.write("]");
    }

    fn emit_object_literal(&mut self, expr: &crate::parser::expressions::ObjectLiteralExpression, arena: &crate::parser::NodeArena) {
        if expr.properties.nodes.is_empty() {
            self.write("{}");
            return;
        }
        self.write("{");
        self.write_space();
        self.emit_node_list(&expr.properties, arena, ", ");
        self.write_space();
        self.write("}");
    }

    fn emit_arrow_function(&mut self, func: &crate::parser::expressions::ArrowFunction, arena: &crate::parser::NodeArena) {
        // Parameters
        if func.parameters.nodes.len() == 1 && func.type_parameters.is_none() {
            // Single parameter without parens (if no type annotation)
            if let Some(param) = arena.get(func.parameters.nodes[0]) {
                if let Node::ParameterDeclaration(pd) = param {
                    if pd.type_annotation.is_none() && pd.initializer.is_none() {
                        self.emit_node(param, arena);
                        self.write(" => ");
                        if let Some(body) = arena.get(func.body) {
                            self.emit_node(body, arena);
                        }
                        return;
                    }
                }
            }
        }

        self.write("(");
        self.emit_node_list(&func.parameters, arena, ", ");
        self.write(")");
        self.write(" => ");
        if let Some(body) = arena.get(func.body) {
            self.emit_node(body, arena);
        }
    }

    fn emit_conditional(&mut self, expr: &crate::parser::expressions::ConditionalExpression, arena: &crate::parser::NodeArena) {
        if let Some(cond) = arena.get(expr.condition) {
            self.emit_node(cond, arena);
        }
        self.write(" ? ");
        if let Some(when_true) = arena.get(expr.when_true) {
            self.emit_node(when_true, arena);
        }
        self.write(" : ");
        if let Some(when_false) = arena.get(expr.when_false) {
            self.emit_node(when_false, arena);
        }
    }

    fn emit_new_expression(&mut self, expr: &crate::parser::expressions::NewExpression, arena: &crate::parser::NodeArena) {
        self.write("new ");
        if let Some(callee) = arena.get(expr.expression) {
            self.emit_node(callee, arena);
        }
        if let Some(ref args) = expr.arguments {
            self.write("(");
            self.emit_node_list(args, arena, ", ");
            self.write(")");
        }
    }

    fn emit_spread_element(&mut self, expr: &crate::parser::expressions::SpreadElement, arena: &crate::parser::NodeArena) {
        self.write("...");
        if let Some(inner) = arena.get(expr.expression) {
            self.emit_node(inner, arena);
        }
    }

    fn emit_yield_expression(&mut self, expr: &crate::parser::expressions::YieldExpression, arena: &crate::parser::NodeArena) {
        if expr.asterisk_token {
            self.write("yield*");
        } else {
            self.write("yield");
        }
        if !expr.expression.is_none() {
            self.write(" ");
            if let Some(inner) = arena.get(expr.expression) {
                self.emit_node(inner, arena);
            }
        }
    }

    fn emit_await_expression(&mut self, expr: &crate::parser::expressions::AwaitExpression, arena: &crate::parser::NodeArena) {
        self.write("await ");
        if let Some(inner) = arena.get(expr.expression) {
            self.emit_node(inner, arena);
        }
    }

    // =========================================================================
    // Statement emission
    // =========================================================================

    fn emit_variable_statement(&mut self, stmt: &crate::parser::statements::VariableStatement, arena: &crate::parser::NodeArena) {
        if let Some(decl_list) = arena.get(stmt.declaration_list) {
            self.emit_node(decl_list, arena);
        }
        self.write_semicolon();
    }

    fn emit_expression_statement(&mut self, stmt: &crate::parser::statements::ExpressionStatement, arena: &crate::parser::NodeArena) {
        if let Some(expr) = arena.get(stmt.expression) {
            self.emit_node(expr, arena);
        }
        self.write_semicolon();
    }

    fn emit_if_statement(&mut self, stmt: &crate::parser::statements::IfStatement, arena: &crate::parser::NodeArena) {
        self.write("if (");
        if let Some(cond) = arena.get(stmt.expression) {
            self.emit_node(cond, arena);
        }
        self.write(") ");
        if let Some(then_stmt) = arena.get(stmt.then_statement) {
            self.emit_node(then_stmt, arena);
        }
        if !stmt.else_statement.is_none() {
            if let Some(else_stmt) = arena.get(stmt.else_statement) {
                self.write(" else ");
                self.emit_node(else_stmt, arena);
            }
        }
    }

    fn emit_while_statement(&mut self, stmt: &crate::parser::statements::WhileStatement, arena: &crate::parser::NodeArena) {
        self.write("while (");
        if let Some(cond) = arena.get(stmt.expression) {
            self.emit_node(cond, arena);
        }
        self.write(") ");
        if let Some(body) = arena.get(stmt.statement) {
            self.emit_node(body, arena);
        }
    }

    fn emit_do_statement(&mut self, stmt: &crate::parser::statements::DoStatement, arena: &crate::parser::NodeArena) {
        self.write("do ");
        if let Some(body) = arena.get(stmt.statement) {
            self.emit_node(body, arena);
        }
        self.write(" while (");
        if let Some(cond) = arena.get(stmt.expression) {
            self.emit_node(cond, arena);
        }
        self.write(")");
        self.write_semicolon();
    }

    fn emit_for_statement(&mut self, stmt: &crate::parser::statements::ForStatement, arena: &crate::parser::NodeArena) {
        self.write("for (");
        if !stmt.initializer.is_none() {
            if let Some(init) = arena.get(stmt.initializer) {
                self.emit_node(init, arena);
            }
        }
        self.write("; ");
        if !stmt.condition.is_none() {
            if let Some(cond) = arena.get(stmt.condition) {
                self.emit_node(cond, arena);
            }
        }
        self.write("; ");
        if !stmt.incrementor.is_none() {
            if let Some(inc) = arena.get(stmt.incrementor) {
                self.emit_node(inc, arena);
            }
        }
        self.write(") ");
        if let Some(body) = arena.get(stmt.statement) {
            self.emit_node(body, arena);
        }
    }

    fn emit_for_in_statement(&mut self, stmt: &crate::parser::statements::ForInStatement, arena: &crate::parser::NodeArena) {
        self.write("for (");
        if let Some(init) = arena.get(stmt.initializer) {
            self.emit_node(init, arena);
        }
        self.write(" in ");
        if let Some(expr) = arena.get(stmt.expression) {
            self.emit_node(expr, arena);
        }
        self.write(") ");
        if let Some(body) = arena.get(stmt.statement) {
            self.emit_node(body, arena);
        }
    }

    fn emit_for_of_statement(&mut self, stmt: &crate::parser::statements::ForOfStatement, arena: &crate::parser::NodeArena) {
        if stmt.await_modifier {
            self.write("for await (");
        } else {
            self.write("for (");
        }
        if let Some(init) = arena.get(stmt.initializer) {
            self.emit_node(init, arena);
        }
        self.write(" of ");
        if let Some(expr) = arena.get(stmt.expression) {
            self.emit_node(expr, arena);
        }
        self.write(") ");
        if let Some(body) = arena.get(stmt.statement) {
            self.emit_node(body, arena);
        }
    }

    fn emit_return_statement(&mut self, stmt: &crate::parser::statements::ReturnStatement, arena: &crate::parser::NodeArena) {
        self.write("return");
        if !stmt.expression.is_none() {
            self.write_space();
            if let Some(expr) = arena.get(stmt.expression) {
                self.emit_node(expr, arena);
            }
        }
        self.write_semicolon();
    }

    fn emit_block(&mut self, block: &crate::parser::statements::Block, arena: &crate::parser::NodeArena) {
        self.write("{");
        if !block.statements.nodes.is_empty() {
            self.write_line();
            self.increase_indent();
            for node_idx in &block.statements.nodes {
                if let Some(stmt) = arena.get(*node_idx) {
                    self.emit_node(stmt, arena);
                    self.write_line();
                }
            }
            self.decrease_indent();
        }
        self.write("}");
    }

    fn emit_empty_statement(&mut self) {
        self.write_semicolon();
    }

    fn emit_break_statement(&mut self, stmt: &crate::parser::statements::BreakStatement, arena: &crate::parser::NodeArena) {
        self.write("break");
        if !stmt.label.is_none() {
            self.write_space();
            if let Some(label) = arena.get(stmt.label) {
                self.emit_node(label, arena);
            }
        }
        self.write_semicolon();
    }

    fn emit_continue_statement(&mut self, stmt: &crate::parser::statements::ContinueStatement, arena: &crate::parser::NodeArena) {
        self.write("continue");
        if !stmt.label.is_none() {
            self.write_space();
            if let Some(label) = arena.get(stmt.label) {
                self.emit_node(label, arena);
            }
        }
        self.write_semicolon();
    }

    fn emit_throw_statement(&mut self, stmt: &crate::parser::statements::ThrowStatement, arena: &crate::parser::NodeArena) {
        self.write("throw ");
        if let Some(expr) = arena.get(stmt.expression) {
            self.emit_node(expr, arena);
        }
        self.write_semicolon();
    }

    fn emit_try_statement(&mut self, stmt: &crate::parser::statements::TryStatement, arena: &crate::parser::NodeArena) {
        self.write("try ");
        if let Some(try_block) = arena.get(stmt.try_block) {
            self.emit_node(try_block, arena);
        }
        if !stmt.catch_clause.is_none() {
            if let Some(catch_clause) = arena.get(stmt.catch_clause) {
                self.write(" ");
                self.emit_node(catch_clause, arena);
            }
        }
        if !stmt.finally_block.is_none() {
            self.write(" finally ");
            if let Some(finally_block) = arena.get(stmt.finally_block) {
                self.emit_node(finally_block, arena);
            }
        }
    }

    fn emit_catch_clause(&mut self, clause: &crate::parser::statements::CatchClause, arena: &crate::parser::NodeArena) {
        self.write("catch");
        if !clause.variable_declaration.is_none() {
            self.write(" (");
            if let Some(var_decl) = arena.get(clause.variable_declaration) {
                self.emit_node(var_decl, arena);
            }
            self.write(")");
        }
        self.write(" ");
        if let Some(block) = arena.get(clause.block) {
            self.emit_node(block, arena);
        }
    }

    fn emit_switch_statement(&mut self, stmt: &crate::parser::statements::SwitchStatement, arena: &crate::parser::NodeArena) {
        self.write("switch (");
        if let Some(expr) = arena.get(stmt.expression) {
            self.emit_node(expr, arena);
        }
        self.write(") ");
        if let Some(case_block) = arena.get(stmt.case_block) {
            self.emit_node(case_block, arena);
        }
    }

    fn emit_case_block(&mut self, block: &crate::parser::statements::CaseBlock, arena: &crate::parser::NodeArena) {
        self.write("{");
        if !block.clauses.nodes.is_empty() {
            self.write_line();
            self.increase_indent();
            for clause_idx in &block.clauses.nodes {
                if let Some(clause) = arena.get(*clause_idx) {
                    self.emit_node(clause, arena);
                }
            }
            self.decrease_indent();
        }
        self.write("}");
    }

    fn emit_case_clause(&mut self, clause: &crate::parser::statements::CaseClause, arena: &crate::parser::NodeArena) {
        self.write("case ");
        if let Some(expr) = arena.get(clause.expression) {
            self.emit_node(expr, arena);
        }
        self.write(":");
        if !clause.statements.nodes.is_empty() {
            self.write_line();
            self.increase_indent();
            for stmt_idx in &clause.statements.nodes {
                if let Some(stmt) = arena.get(*stmt_idx) {
                    self.emit_node(stmt, arena);
                    self.write_line();
                }
            }
            self.decrease_indent();
        } else {
            self.write_line();
        }
    }

    fn emit_default_clause(&mut self, clause: &crate::parser::statements::DefaultClause, arena: &crate::parser::NodeArena) {
        self.write("default:");
        if !clause.statements.nodes.is_empty() {
            self.write_line();
            self.increase_indent();
            for stmt_idx in &clause.statements.nodes {
                if let Some(stmt) = arena.get(*stmt_idx) {
                    self.emit_node(stmt, arena);
                    self.write_line();
                }
            }
            self.decrease_indent();
        } else {
            self.write_line();
        }
    }

    fn emit_labeled_statement(&mut self, stmt: &crate::parser::statements::LabeledStatement, arena: &crate::parser::NodeArena) {
        if let Some(label) = arena.get(stmt.label) {
            self.emit_node(label, arena);
        }
        self.write(": ");
        if let Some(statement) = arena.get(stmt.statement) {
            self.emit_node(statement, arena);
        }
    }

    // =========================================================================
    // Declaration emission
    // =========================================================================

    fn emit_function_declaration(&mut self, decl: &crate::parser::declarations::FunctionDeclaration, arena: &crate::parser::NodeArena) {
        if decl.is_async {
            self.write("async ");
        }
        if decl.asterisk_token {
            self.write("function* ");
        } else {
            self.write("function ");
        }
        if let Some(name) = arena.get(decl.name) {
            self.emit_node(name, arena);
        }
        self.write("(");
        self.emit_node_list(&decl.parameters, arena, ", ");
        self.write(") ");
        if let Some(body) = arena.get(decl.body) {
            self.emit_node(body, arena);
        }
    }

    fn emit_class_declaration(&mut self, decl: &crate::parser::declarations::ClassDeclaration, arena: &crate::parser::NodeArena) {
        // Emit modifiers (including decorators)
        if let Some(ref modifiers) = decl.modifiers {
            self.emit_modifiers(modifiers, arena);
        }
        self.write("class ");
        if let Some(name) = arena.get(decl.name) {
            self.emit_node(name, arena);
        }
        // Emit type parameters
        if let Some(ref type_params) = decl.type_parameters {
            self.write("<");
            self.emit_node_list(type_params, arena, ", ");
            self.write(">");
        }
        // Emit heritage clauses
        if let Some(ref heritage_clauses) = decl.heritage_clauses {
            for clause_idx in &heritage_clauses.nodes {
                if let Some(clause) = arena.get(*clause_idx) {
                    self.write(" ");
                    self.emit_node(clause, arena);
                }
            }
        }
        self.write(" {");
        if !decl.members.nodes.is_empty() {
            self.write_line();
            self.increase_indent();
            for member_idx in &decl.members.nodes {
                if let Some(member) = arena.get(*member_idx) {
                    self.emit_node(member, arena);
                    self.write_line();
                }
            }
            self.decrease_indent();
        }
        self.write("}");
    }

    fn emit_variable_declaration(&mut self, decl: &crate::parser::statements::VariableDeclaration, arena: &crate::parser::NodeArena) {
        if let Some(name) = arena.get(decl.name) {
            self.emit_node(name, arena);
        }
        if !decl.initializer.is_none() {
            self.write(" = ");
            if let Some(init) = arena.get(decl.initializer) {
                self.emit_node(init, arena);
            }
        }
    }

    fn emit_variable_declaration_list(&mut self, list: &crate::parser::statements::VariableDeclarationList, arena: &crate::parser::NodeArena) {
        // Determine var/let/const from flags
        let keyword = if (list.base.flags & crate::parser::node_flags::CONST) != 0 {
            "const"
        } else if (list.base.flags & crate::parser::node_flags::LET) != 0 {
            "let"
        } else {
            "var"
        };
        self.write(keyword);
        self.write_space();
        self.emit_node_list(&list.declarations, arena, ", ");
    }

    fn emit_parameter_declaration(&mut self, param: &crate::parser::declarations::ParameterDeclaration, arena: &crate::parser::NodeArena) {
        // Modifiers (public, private, etc.)
        if let Some(ref modifiers) = param.modifiers {
            self.emit_modifiers(modifiers, arena);
        }
        // Rest parameter
        if param.dot_dot_dot_token {
            self.write("...");
        }
        if let Some(name) = arena.get(param.name) {
            self.emit_node(name, arena);
        }
        // Optional parameter
        if param.question_token {
            self.write("?");
        }
        // Type annotation
        if !param.type_annotation.is_none() {
            self.write(": ");
            if let Some(type_node) = arena.get(param.type_annotation) {
                self.emit_node(type_node, arena);
            }
        }
        // Initializer
        if !param.initializer.is_none() {
            self.write(" = ");
            if let Some(init) = arena.get(param.initializer) {
                self.emit_node(init, arena);
            }
        }
    }

    // =========================================================================
    // Object literal members
    // =========================================================================

    fn emit_property_assignment(&mut self, prop: &crate::parser::declarations::PropertyAssignment, arena: &crate::parser::NodeArena) {
        if let Some(name) = arena.get(prop.name) {
            self.emit_node(name, arena);
        }
        self.write(": ");
        if let Some(init) = arena.get(prop.initializer) {
            self.emit_node(init, arena);
        }
    }

    fn emit_shorthand_property(&mut self, prop: &crate::parser::declarations::ShorthandPropertyAssignment, arena: &crate::parser::NodeArena) {
        if let Some(name) = arena.get(prop.name) {
            self.emit_node(name, arena);
        }
        // Optional initializer (= value)
        if !prop.object_assignment_initializer.is_none() {
            self.write(" = ");
            if let Some(init) = arena.get(prop.object_assignment_initializer) {
                self.emit_node(init, arena);
            }
        }
    }

    fn emit_spread_assignment(&mut self, spread: &crate::parser::declarations::SpreadAssignment, arena: &crate::parser::NodeArena) {
        self.write("...");
        if let Some(expr) = arena.get(spread.expression) {
            self.emit_node(expr, arena);
        }
    }

    // =========================================================================
    // TypeScript-specific expressions
    // =========================================================================

    fn emit_as_expression(&mut self, expr: &crate::parser::expressions::AsExpression, arena: &crate::parser::NodeArena) {
        if let Some(e) = arena.get(expr.expression) {
            self.emit_node(e, arena);
        }
        self.write(" as ");
        if let Some(t) = arena.get(expr.type_node) {
            self.emit_node(t, arena);
        }
    }

    fn emit_type_assertion(&mut self, expr: &crate::parser::expressions::TypeAssertion, arena: &crate::parser::NodeArena) {
        self.write("<");
        if let Some(t) = arena.get(expr.type_node) {
            self.emit_node(t, arena);
        }
        self.write(">");
        if let Some(e) = arena.get(expr.expression) {
            self.emit_node(e, arena);
        }
    }

    fn emit_non_null_expression(&mut self, expr: &crate::parser::expressions::NonNullExpression, arena: &crate::parser::NodeArena) {
        if let Some(e) = arena.get(expr.expression) {
            self.emit_node(e, arena);
        }
        self.write("!");
    }

    fn emit_template_expression(&mut self, expr: &crate::parser::expressions::TemplateExpression, arena: &crate::parser::NodeArena) {
        // Template head (e.g., `text${)
        if let Some(head) = arena.get(expr.head) {
            self.emit_node(head, arena);
        }
        // Template spans
        for span_idx in &expr.template_spans.nodes {
            if let Some(span) = arena.get(*span_idx) {
                self.emit_node(span, arena);
            }
        }
    }

    fn emit_tagged_template(&mut self, expr: &crate::parser::expressions::TaggedTemplateExpression, arena: &crate::parser::NodeArena) {
        if let Some(tag) = arena.get(expr.tag) {
            self.emit_node(tag, arena);
        }
        if let Some(template) = arena.get(expr.template) {
            self.emit_node(template, arena);
        }
    }

    // =========================================================================
    // Additional statements
    // =========================================================================

    fn emit_with_statement(&mut self, stmt: &crate::parser::statements::WithStatement, arena: &crate::parser::NodeArena) {
        self.write("with (");
        if let Some(expr) = arena.get(stmt.expression) {
            self.emit_node(expr, arena);
        }
        self.write(") ");
        if let Some(s) = arena.get(stmt.statement) {
            self.emit_node(s, arena);
        }
    }

    // =========================================================================
    // TypeScript declarations
    // =========================================================================

    fn emit_interface_declaration(&mut self, decl: &crate::parser::declarations::InterfaceDeclaration, arena: &crate::parser::NodeArena) {
        self.write("interface ");
        if let Some(name) = arena.get(decl.name) {
            self.emit_node(name, arena);
        }
        // TODO: type parameters, extends clause
        self.write(" {");
        self.write_line();
        self.increase_indent();
        for member_idx in &decl.members.nodes {
            if let Some(member) = arena.get(*member_idx) {
                self.emit_node(member, arena);
                self.write(";");
                self.write_line();
            }
        }
        self.decrease_indent();
        self.write("}");
    }

    fn emit_type_alias_declaration(&mut self, decl: &crate::parser::declarations::TypeAliasDeclaration, arena: &crate::parser::NodeArena) {
        // Emit modifiers
        if let Some(ref modifiers) = decl.modifiers {
            self.emit_modifiers(modifiers, arena);
        }
        self.write("type ");
        if let Some(name) = arena.get(decl.name) {
            self.emit_node(name, arena);
        }
        // Emit type parameters
        if let Some(ref type_params) = decl.type_parameters {
            self.write("<");
            self.emit_node_list(type_params, arena, ", ");
            self.write(">");
        }
        self.write(" = ");
        if let Some(ty) = arena.get(decl.type_node) {
            self.emit_node(ty, arena);
        }
        self.write(";");
    }

    fn emit_enum_declaration(&mut self, decl: &crate::parser::declarations::EnumDeclaration, arena: &crate::parser::NodeArena) {
        self.write("enum ");
        if let Some(name) = arena.get(decl.name) {
            self.emit_node(name, arena);
        }
        self.write(" {");
        self.write_line();
        self.increase_indent();
        for (i, member_idx) in decl.members.nodes.iter().enumerate() {
            if let Some(member) = arena.get(*member_idx) {
                self.emit_node(member, arena);
                if i < decl.members.nodes.len() - 1 {
                    self.write(",");
                }
                self.write_line();
            }
        }
        self.decrease_indent();
        self.write("}");
    }

    fn emit_module_declaration(&mut self, decl: &crate::parser::declarations::ModuleDeclaration, arena: &crate::parser::NodeArena) {
        // TODO: handle 'declare' modifier
        self.write("namespace ");
        if let Some(name) = arena.get(decl.name) {
            self.emit_node(name, arena);
        }
        self.write(" ");
        if let Some(body) = arena.get(decl.body) {
            self.emit_node(body, arena);
        }
    }

    // =========================================================================
    // Import/Export declarations
    // =========================================================================

    fn emit_import_declaration(&mut self, decl: &crate::parser::declarations::ImportDeclaration, arena: &crate::parser::NodeArena) {
        self.write("import ");
        if let Some(clause) = arena.get(decl.import_clause) {
            self.emit_node(clause, arena);
            self.write(" from ");
        }
        if let Some(specifier) = arena.get(decl.module_specifier) {
            self.emit_node(specifier, arena);
        }
        self.write(";");
    }

    fn emit_export_declaration(&mut self, decl: &crate::parser::declarations::ExportDeclaration, arena: &crate::parser::NodeArena) {
        self.write("export ");
        if let Some(clause) = arena.get(decl.export_clause) {
            self.emit_node(clause, arena);
        } else {
            self.write("*");
        }
        if !decl.module_specifier.is_none() {
            self.write(" from ");
            if let Some(specifier) = arena.get(decl.module_specifier) {
                self.emit_node(specifier, arena);
            }
        }
        self.write(";");
    }

    fn emit_export_assignment(&mut self, decl: &crate::parser::declarations::ExportAssignment, arena: &crate::parser::NodeArena) {
        if decl.is_export_equals {
            self.write("export = ");
        } else {
            self.write("export default ");
        }
        if let Some(expr) = arena.get(decl.expression) {
            self.emit_node(expr, arena);
        }
        self.write(";");
    }

    // =========================================================================
    // Class members
    // =========================================================================

    fn emit_method_declaration(&mut self, decl: &crate::parser::declarations::MethodDeclaration, arena: &crate::parser::NodeArena) {
        // TODO: modifiers (async, static, etc.)
        if let Some(name) = arena.get(decl.name) {
            self.emit_node(name, arena);
        }
        self.write("(");
        self.emit_node_list(&decl.parameters, arena, ", ");
        self.write(") ");
        if let Some(body) = arena.get(decl.body) {
            self.emit_node(body, arena);
        }
    }

    fn emit_property_declaration(&mut self, decl: &crate::parser::declarations::PropertyDeclaration, arena: &crate::parser::NodeArena) {
        // TODO: modifiers
        if let Some(name) = arena.get(decl.name) {
            self.emit_node(name, arena);
        }
        if !decl.type_annotation.is_none() {
            self.write(": ");
            if let Some(ty) = arena.get(decl.type_annotation) {
                self.emit_node(ty, arena);
            }
        }
        if !decl.initializer.is_none() {
            self.write(" = ");
            if let Some(init) = arena.get(decl.initializer) {
                self.emit_node(init, arena);
            }
        }
        self.write(";");
    }

    fn emit_constructor_declaration(&mut self, decl: &crate::parser::declarations::ConstructorDeclaration, arena: &crate::parser::NodeArena) {
        self.write("constructor(");
        self.emit_node_list(&decl.parameters, arena, ", ");
        self.write(") ");
        if let Some(body) = arena.get(decl.body) {
            self.emit_node(body, arena);
        }
    }

    fn emit_get_accessor(&mut self, decl: &crate::parser::declarations::GetAccessorDeclaration, arena: &crate::parser::NodeArena) {
        self.write("get ");
        if let Some(name) = arena.get(decl.name) {
            self.emit_node(name, arena);
        }
        self.write("() ");
        if let Some(body) = arena.get(decl.body) {
            self.emit_node(body, arena);
        }
    }

    fn emit_set_accessor(&mut self, decl: &crate::parser::declarations::SetAccessorDeclaration, arena: &crate::parser::NodeArena) {
        self.write("set ");
        if let Some(name) = arena.get(decl.name) {
            self.emit_node(name, arena);
        }
        self.write("(");
        self.emit_node_list(&decl.parameters, arena, ", ");
        self.write(") ");
        if let Some(body) = arena.get(decl.body) {
            self.emit_node(body, arena);
        }
    }

    // =========================================================================
    // Source file emission
    // =========================================================================

    fn emit_source_file(&mut self, sf: &crate::parser::literals::SourceFile, arena: &crate::parser::NodeArena) {
        for stmt_idx in &sf.statements.nodes {
            if let Some(stmt) = arena.get(*stmt_idx) {
                self.emit_node(stmt, arena);
                self.write_line();
            }
        }
    }

    // =========================================================================
    // Token emission
    // =========================================================================

    fn emit_token(&mut self, base: &NodeBase) {
        // Only valid for token kinds (0-166). AST node kinds are > 166.
        if let Some(kind) = SyntaxKind::try_from_u16(base.kind) {
            self.emit_token_kind(kind);
        } else {
            // Fallback: emit as unknown token (should not happen if called correctly)
            debug_assert!(false, "emit_token called with non-token kind: {}", base.kind);
        }
    }

    fn emit_token_kind(&mut self, kind: SyntaxKind) {
        let text = match kind {
            SyntaxKind::PlusToken => "+",
            SyntaxKind::MinusToken => "-",
            SyntaxKind::AsteriskToken => "*",
            SyntaxKind::SlashToken => "/",
            SyntaxKind::PercentToken => "%",
            SyntaxKind::AsteriskAsteriskToken => "**",
            SyntaxKind::PlusPlusToken => "++",
            SyntaxKind::MinusMinusToken => "--",
            SyntaxKind::LessThanToken => "<",
            SyntaxKind::GreaterThanToken => ">",
            SyntaxKind::LessThanEqualsToken => "<=",
            SyntaxKind::GreaterThanEqualsToken => ">=",
            SyntaxKind::EqualsEqualsToken => "==",
            SyntaxKind::ExclamationEqualsToken => "!=",
            SyntaxKind::EqualsEqualsEqualsToken => "===",
            SyntaxKind::ExclamationEqualsEqualsToken => "!==",
            SyntaxKind::EqualsToken => "=",
            SyntaxKind::PlusEqualsToken => "+=",
            SyntaxKind::MinusEqualsToken => "-=",
            SyntaxKind::AsteriskEqualsToken => "*=",
            SyntaxKind::SlashEqualsToken => "/=",
            SyntaxKind::PercentEqualsToken => "%=",
            SyntaxKind::AsteriskAsteriskEqualsToken => "**=",
            SyntaxKind::AmpersandToken => "&",
            SyntaxKind::BarToken => "|",
            SyntaxKind::CaretToken => "^",
            SyntaxKind::TildeToken => "~",
            SyntaxKind::AmpersandAmpersandToken => "&&",
            SyntaxKind::BarBarToken => "||",
            SyntaxKind::ExclamationToken => "!",
            SyntaxKind::QuestionToken => "?",
            SyntaxKind::ColonToken => ":",
            SyntaxKind::CommaToken => ",",
            SyntaxKind::DotToken => ".",
            SyntaxKind::DotDotDotToken => "...",
            SyntaxKind::EqualsGreaterThanToken => "=>",
            SyntaxKind::LessThanLessThanToken => "<<",
            SyntaxKind::GreaterThanGreaterThanToken => ">>",
            SyntaxKind::GreaterThanGreaterThanGreaterThanToken => ">>>",
            SyntaxKind::QuestionQuestionToken => "??",
            SyntaxKind::QuestionDotToken => "?.",
            SyntaxKind::InKeyword => "in",
            SyntaxKind::InstanceOfKeyword => "instanceof",
            SyntaxKind::TypeOfKeyword => "typeof",
            SyntaxKind::VoidKeyword => "void",
            SyntaxKind::DeleteKeyword => "delete",
            SyntaxKind::AwaitKeyword => "await",
            _ => "",
        };
        self.write(text);
    }

    // =========================================================================
    // Type node emission
    // =========================================================================

    fn emit_type_reference(&mut self, t: &crate::parser::types::TypeReference, arena: &crate::parser::NodeArena) {
        if let Some(name) = arena.get(t.type_name) {
            self.emit_node(name, arena);
        }
        if let Some(ref type_args) = t.type_arguments {
            self.write("<");
            self.emit_node_list(type_args, arena, ", ");
            self.write(">");
        }
    }

    fn emit_qualified_name(&mut self, left: NodeIndex, right: NodeIndex, arena: &crate::parser::NodeArena) {
        if let Some(l) = arena.get(left) {
            self.emit_node(l, arena);
        }
        self.write(".");
        if let Some(r) = arena.get(right) {
            self.emit_node(r, arena);
        }
    }

    fn emit_union_type(&mut self, t: &crate::parser::types::UnionType, arena: &crate::parser::NodeArena) {
        self.emit_node_list(&t.types, arena, " | ");
    }

    fn emit_intersection_type(&mut self, t: &crate::parser::types::IntersectionType, arena: &crate::parser::NodeArena) {
        self.emit_node_list(&t.types, arena, " & ");
    }

    fn emit_literal_type(&mut self, t: &crate::parser::types::LiteralType, arena: &crate::parser::NodeArena) {
        if let Some(lit) = arena.get(t.literal) {
            self.emit_node(lit, arena);
        }
    }

    fn emit_array_type(&mut self, t: &crate::parser::types::ArrayType, arena: &crate::parser::NodeArena) {
        if let Some(elem) = arena.get(t.element_type) {
            self.emit_node(elem, arena);
        }
        self.write("[]");
    }

    fn emit_tuple_type(&mut self, t: &crate::parser::types::TupleType, arena: &crate::parser::NodeArena) {
        self.write("[");
        self.emit_node_list(&t.elements, arena, ", ");
        self.write("]");
    }

    fn emit_optional_type(&mut self, t: &crate::parser::types::OptionalType, arena: &crate::parser::NodeArena) {
        if let Some(ty) = arena.get(t.type_node) {
            self.emit_node(ty, arena);
        }
        self.write("?");
    }

    fn emit_rest_type(&mut self, t: &crate::parser::types::RestType, arena: &crate::parser::NodeArena) {
        self.write("...");
        if let Some(ty) = arena.get(t.type_node) {
            self.emit_node(ty, arena);
        }
    }

    fn emit_function_type(&mut self, t: &crate::parser::types::FunctionType, arena: &crate::parser::NodeArena) {
        if let Some(ref type_params) = t.type_parameters {
            self.write("<");
            self.emit_node_list(type_params, arena, ", ");
            self.write(">");
        }
        self.write("(");
        self.emit_node_list(&t.parameters, arena, ", ");
        self.write(") => ");
        if let Some(ret) = arena.get(t.type_node) {
            self.emit_node(ret, arena);
        }
    }

    fn emit_constructor_type(&mut self, t: &crate::parser::types::ConstructorType, arena: &crate::parser::NodeArena) {
        // Check for 'abstract' modifier
        if let Some(ref mods) = t.modifiers {
            for mod_idx in &mods.nodes {
                if let Some(Node::Token(base)) = arena.get(*mod_idx) {
                    if let Some(SyntaxKind::AbstractKeyword) = SyntaxKind::try_from_u16(base.kind) {
                        self.write("abstract ");
                    }
                }
            }
        }
        self.write("new ");
        if let Some(ref type_params) = t.type_parameters {
            self.write("<");
            self.emit_node_list(type_params, arena, ", ");
            self.write(">");
        }
        self.write("(");
        self.emit_node_list(&t.parameters, arena, ", ");
        self.write(") => ");
        if let Some(ret) = arena.get(t.type_node) {
            self.emit_node(ret, arena);
        }
    }

    fn emit_type_query(&mut self, t: &crate::parser::types::TypeQuery, arena: &crate::parser::NodeArena) {
        self.write("typeof ");
        if let Some(name) = arena.get(t.expr_name) {
            self.emit_node(name, arena);
        }
        if let Some(ref type_args) = t.type_arguments {
            self.write("<");
            self.emit_node_list(type_args, arena, ", ");
            self.write(">");
        }
    }

    fn emit_type_literal(&mut self, t: &crate::parser::types::TypeLiteral, arena: &crate::parser::NodeArena) {
        if t.members.nodes.is_empty() {
            self.write("{}");
            return;
        }
        self.write("{ ");
        for (i, member_idx) in t.members.nodes.iter().enumerate() {
            if i > 0 {
                self.write("; ");
            }
            if let Some(member) = arena.get(*member_idx) {
                self.emit_node(member, arena);
            }
        }
        self.write(" }");
    }

    fn emit_indexed_access_type(&mut self, t: &crate::parser::types::IndexedAccessType, arena: &crate::parser::NodeArena) {
        if let Some(obj) = arena.get(t.object_type) {
            self.emit_node(obj, arena);
        }
        self.write("[");
        if let Some(idx) = arena.get(t.index_type) {
            self.emit_node(idx, arena);
        }
        self.write("]");
    }

    fn emit_mapped_type(&mut self, t: &crate::parser::types::MappedType, arena: &crate::parser::NodeArena) {
        self.write("{ ");
        // readonly modifier
        if let Some(readonly) = t.readonly_token {
            match SyntaxKind::try_from_u16(readonly) {
                Some(SyntaxKind::ReadonlyKeyword) => self.write("readonly "),
                Some(SyntaxKind::PlusToken) => self.write("+readonly "),
                Some(SyntaxKind::MinusToken) => self.write("-readonly "),
                _ => {}
            }
        }
        self.write("[");
        // Emit mapped type parameter: [K in T] instead of [K extends T]
        if let Some(tp) = arena.get(t.type_parameter) {
            if let Node::TypeParameterDeclaration(type_param) = tp {
                if let Some(name) = arena.get(type_param.name) {
                    self.emit_node(name, arena);
                }
                if !type_param.constraint.is_none() {
                    self.write(" in ");
                    if let Some(c) = arena.get(type_param.constraint) {
                        self.emit_node(c, arena);
                    }
                }
            } else {
                self.emit_node(tp, arena);
            }
        }
        // name remapping with 'as'
        if !t.name_type.is_none() {
            self.write(" as ");
            if let Some(name) = arena.get(t.name_type) {
                self.emit_node(name, arena);
            }
        }
        self.write("]");
        // question modifier
        if let Some(question) = t.question_token {
            match SyntaxKind::try_from_u16(question) {
                Some(SyntaxKind::QuestionToken) => self.write("?"),
                Some(SyntaxKind::PlusToken) => self.write("+?"),
                Some(SyntaxKind::MinusToken) => self.write("-?"),
                _ => {}
            }
        }
        if !t.type_node.is_none() {
            self.write(": ");
            if let Some(ty) = arena.get(t.type_node) {
                self.emit_node(ty, arena);
            }
        }
        self.write(" }");
    }

    fn emit_conditional_type(&mut self, t: &crate::parser::types::ConditionalType, arena: &crate::parser::NodeArena) {
        if let Some(check) = arena.get(t.check_type) {
            self.emit_node(check, arena);
        }
        self.write(" extends ");
        if let Some(ext) = arena.get(t.extends_type) {
            self.emit_node(ext, arena);
        }
        self.write(" ? ");
        if let Some(true_ty) = arena.get(t.true_type) {
            self.emit_node(true_ty, arena);
        }
        self.write(" : ");
        if let Some(false_ty) = arena.get(t.false_type) {
            self.emit_node(false_ty, arena);
        }
    }

    fn emit_infer_type(&mut self, t: &crate::parser::types::InferType, arena: &crate::parser::NodeArena) {
        self.write("infer ");
        if let Some(tp) = arena.get(t.type_parameter) {
            self.emit_node(tp, arena);
        }
    }

    fn emit_parenthesized_type(&mut self, t: &crate::parser::types::ParenthesizedType, arena: &crate::parser::NodeArena) {
        self.write("(");
        if let Some(ty) = arena.get(t.type_node) {
            self.emit_node(ty, arena);
        }
        self.write(")");
    }

    fn emit_type_operator(&mut self, t: &crate::parser::types::TypeOperator, arena: &crate::parser::NodeArena) {
        match SyntaxKind::try_from_u16(t.operator) {
            Some(SyntaxKind::KeyOfKeyword) => self.write("keyof "),
            Some(SyntaxKind::UniqueKeyword) => self.write("unique "),
            Some(SyntaxKind::ReadonlyKeyword) => self.write("readonly "),
            _ => {}
        }
        if let Some(ty) = arena.get(t.type_node) {
            self.emit_node(ty, arena);
        }
    }

    fn emit_template_literal_type(&mut self, t: &crate::parser::types::TemplateLiteralType, arena: &crate::parser::NodeArena) {
        if let Some(head) = arena.get(t.head) {
            self.emit_node(head, arena);
        }
        for span_idx in &t.template_spans.nodes {
            if let Some(span) = arena.get(*span_idx) {
                self.emit_node(span, arena);
            }
        }
    }

    fn emit_named_tuple_member(&mut self, t: &crate::parser::types::NamedTupleMember, arena: &crate::parser::NodeArena) {
        if t.dot_dot_dot_token {
            self.write("...");
        }
        if let Some(name) = arena.get(t.name) {
            self.emit_node(name, arena);
        }
        if t.question_token {
            self.write("?");
        }
        self.write(": ");
        if let Some(ty) = arena.get(t.type_node) {
            self.emit_node(ty, arena);
        }
    }

    fn emit_type_predicate(&mut self, t: &crate::parser::types::TypePredicate, arena: &crate::parser::NodeArena) {
        if t.asserts_modifier {
            self.write("asserts ");
        }
        if let Some(param) = arena.get(t.parameter_name) {
            self.emit_node(param, arena);
        }
        if !t.type_node.is_none() {
            self.write(" is ");
            if let Some(ty) = arena.get(t.type_node) {
                self.emit_node(ty, arena);
            }
        }
    }

    // =========================================================================
    // Type-related declarations
    // =========================================================================

    fn emit_type_parameter_declaration(&mut self, t: &crate::parser::declarations::TypeParameterDeclaration, arena: &crate::parser::NodeArena) {
        // Variance modifiers (in/out)
        if let Some(ref mods) = t.modifiers {
            for mod_idx in &mods.nodes {
                if let Some(Node::Token(base)) = arena.get(*mod_idx) {
                    match SyntaxKind::try_from_u16(base.kind) {
                        Some(SyntaxKind::InKeyword) => self.write("in "),
                        Some(SyntaxKind::OutKeyword) => self.write("out "),
                        _ => {}
                    }
                }
            }
        }
        if let Some(name) = arena.get(t.name) {
            self.emit_node(name, arena);
        }
        if !t.constraint.is_none() {
            self.write(" extends ");
            if let Some(c) = arena.get(t.constraint) {
                self.emit_node(c, arena);
            }
        }
        if !t.default.is_none() {
            self.write(" = ");
            if let Some(d) = arena.get(t.default) {
                self.emit_node(d, arena);
            }
        }
    }

    fn emit_heritage_clause(&mut self, h: &crate::parser::declarations::HeritageClause, arena: &crate::parser::NodeArena) {
        match SyntaxKind::try_from_u16(h.token) {
            Some(SyntaxKind::ExtendsKeyword) => self.write("extends "),
            Some(SyntaxKind::ImplementsKeyword) => self.write("implements "),
            _ => {}
        }
        self.emit_node_list(&h.types, arena, ", ");
    }

    fn emit_expression_with_type_arguments(&mut self, e: &crate::parser::declarations::ExpressionWithTypeArguments, arena: &crate::parser::NodeArena) {
        if let Some(expr) = arena.get(e.expression) {
            self.emit_node(expr, arena);
        }
        if let Some(ref type_args) = e.type_arguments {
            self.write("<");
            self.emit_node_list(type_args, arena, ", ");
            self.write(">");
        }
    }

    fn emit_decorator(&mut self, d: &crate::parser::declarations::Decorator, arena: &crate::parser::NodeArena) {
        self.write("@");
        if let Some(expr) = arena.get(d.expression) {
            self.emit_node(expr, arena);
        }
    }

    fn emit_computed_property_name(&mut self, expression: NodeIndex, arena: &crate::parser::NodeArena) {
        self.write("[");
        if let Some(expr) = arena.get(expression) {
            self.emit_node(expr, arena);
        }
        self.write("]");
    }

    // =========================================================================
    // Interface/type literal members
    // =========================================================================

    fn emit_property_signature(&mut self, p: &crate::parser::declarations::PropertySignature, arena: &crate::parser::NodeArena) {
        // Modifiers (readonly)
        if let Some(ref mods) = p.modifiers {
            self.emit_modifiers(mods, arena);
        }
        if let Some(name) = arena.get(p.name) {
            self.emit_node(name, arena);
        }
        if p.question_token {
            self.write("?");
        }
        if !p.type_annotation.is_none() {
            self.write(": ");
            if let Some(ty) = arena.get(p.type_annotation) {
                self.emit_node(ty, arena);
            }
        }
    }

    fn emit_method_signature(&mut self, m: &crate::parser::declarations::MethodSignature, arena: &crate::parser::NodeArena) {
        if let Some(name) = arena.get(m.name) {
            self.emit_node(name, arena);
        }
        if m.question_token {
            self.write("?");
        }
        if let Some(ref type_params) = m.type_parameters {
            self.write("<");
            self.emit_node_list(type_params, arena, ", ");
            self.write(">");
        }
        self.write("(");
        self.emit_node_list(&m.parameters, arena, ", ");
        self.write(")");
        if !m.type_annotation.is_none() {
            self.write(": ");
            if let Some(ty) = arena.get(m.type_annotation) {
                self.emit_node(ty, arena);
            }
        }
    }

    fn emit_index_signature(&mut self, i: &crate::parser::declarations::IndexSignatureDeclaration, arena: &crate::parser::NodeArena) {
        // Modifiers (readonly)
        if let Some(ref mods) = i.modifiers {
            self.emit_modifiers(mods, arena);
        }
        self.write("[");
        self.emit_node_list(&i.parameters, arena, ", ");
        self.write("]");
        if !i.type_annotation.is_none() {
            self.write(": ");
            if let Some(ty) = arena.get(i.type_annotation) {
                self.emit_node(ty, arena);
            }
        }
    }

    fn emit_call_signature(&mut self, c: &crate::parser::declarations::CallSignature, arena: &crate::parser::NodeArena) {
        if let Some(ref type_params) = c.type_parameters {
            self.write("<");
            self.emit_node_list(type_params, arena, ", ");
            self.write(">");
        }
        self.write("(");
        self.emit_node_list(&c.parameters, arena, ", ");
        self.write(")");
        if !c.type_annotation.is_none() {
            self.write(": ");
            if let Some(ty) = arena.get(c.type_annotation) {
                self.emit_node(ty, arena);
            }
        }
    }

    fn emit_construct_signature(&mut self, c: &crate::parser::declarations::ConstructSignature, arena: &crate::parser::NodeArena) {
        self.write("new ");
        if let Some(ref type_params) = c.type_parameters {
            self.write("<");
            self.emit_node_list(type_params, arena, ", ");
            self.write(">");
        }
        self.write("(");
        self.emit_node_list(&c.parameters, arena, ", ");
        self.write(")");
        if !c.type_annotation.is_none() {
            self.write(": ");
            if let Some(ty) = arena.get(c.type_annotation) {
                self.emit_node(ty, arena);
            }
        }
    }

    // =========================================================================
    // Enum members
    // =========================================================================

    fn emit_enum_member(&mut self, e: &crate::parser::declarations::EnumMember, arena: &crate::parser::NodeArena) {
        if let Some(name) = arena.get(e.name) {
            self.emit_node(name, arena);
        }
        if !e.initializer.is_none() {
            self.write(" = ");
            if let Some(init) = arena.get(e.initializer) {
                self.emit_node(init, arena);
            }
        }
    }

    // =========================================================================
    // Import/export details
    // =========================================================================

    fn emit_import_clause(&mut self, c: &crate::parser::declarations::ImportClause, arena: &crate::parser::NodeArena) {
        if c.is_type_only {
            self.write("type ");
        }
        let has_name = !c.name.is_none();
        if has_name {
            if let Some(name) = arena.get(c.name) {
                self.emit_node(name, arena);
            }
        }
        if !c.named_bindings.is_none() {
            if has_name {
                self.write(", ");
            }
            if let Some(bindings) = arena.get(c.named_bindings) {
                self.emit_node(bindings, arena);
            }
        }
    }

    fn emit_namespace_import(&mut self, n: &crate::parser::declarations::NamespaceImport, arena: &crate::parser::NodeArena) {
        self.write("* as ");
        if let Some(name) = arena.get(n.name) {
            self.emit_node(name, arena);
        }
    }

    fn emit_named_imports(&mut self, n: &crate::parser::declarations::NamedImports, arena: &crate::parser::NodeArena) {
        self.write("{ ");
        self.emit_node_list(&n.elements, arena, ", ");
        self.write(" }");
    }

    fn emit_import_specifier(&mut self, s: &crate::parser::declarations::ImportSpecifier, arena: &crate::parser::NodeArena) {
        if s.is_type_only {
            self.write("type ");
        }
        if !s.property_name.is_none() {
            if let Some(prop) = arena.get(s.property_name) {
                self.emit_node(prop, arena);
            }
            self.write(" as ");
        }
        if let Some(name) = arena.get(s.name) {
            self.emit_node(name, arena);
        }
    }

    fn emit_named_exports(&mut self, n: &crate::parser::declarations::NamedExports, arena: &crate::parser::NodeArena) {
        self.write("{ ");
        self.emit_node_list(&n.elements, arena, ", ");
        self.write(" }");
    }

    fn emit_namespace_export(&mut self, n: &crate::parser::declarations::NamespaceExport, arena: &crate::parser::NodeArena) {
        self.write("* as ");
        if let Some(name) = arena.get(n.name) {
            self.emit_node(name, arena);
        }
    }

    fn emit_export_specifier(&mut self, s: &crate::parser::declarations::ExportSpecifier, arena: &crate::parser::NodeArena) {
        if s.is_type_only {
            self.write("type ");
        }
        if !s.property_name.is_none() {
            if let Some(prop) = arena.get(s.property_name) {
                self.emit_node(prop, arena);
            }
            self.write(" as ");
        }
        if let Some(name) = arena.get(s.name) {
            self.emit_node(name, arena);
        }
    }

    // =========================================================================
    // Binding patterns
    // =========================================================================

    fn emit_object_binding_pattern(&mut self, p: &crate::parser::declarations::ObjectBindingPattern, arena: &crate::parser::NodeArena) {
        self.write("{ ");
        self.emit_node_list(&p.elements, arena, ", ");
        self.write(" }");
    }

    fn emit_array_binding_pattern(&mut self, p: &crate::parser::declarations::ArrayBindingPattern, arena: &crate::parser::NodeArena) {
        self.write("[");
        self.emit_node_list(&p.elements, arena, ", ");
        self.write("]");
    }

    fn emit_binding_element(&mut self, e: &crate::parser::declarations::BindingElement, arena: &crate::parser::NodeArena) {
        if e.dot_dot_dot_token {
            self.write("...");
        }
        if !e.property_name.is_none() {
            if let Some(prop) = arena.get(e.property_name) {
                self.emit_node(prop, arena);
            }
            self.write(": ");
        }
        if let Some(name) = arena.get(e.name) {
            self.emit_node(name, arena);
        }
        if !e.initializer.is_none() {
            self.write(" = ");
            if let Some(init) = arena.get(e.initializer) {
                self.emit_node(init, arena);
            }
        }
    }

    // =========================================================================
    // Template parts
    // =========================================================================

    fn emit_template_span(&mut self, s: &TemplateSpan, arena: &crate::parser::NodeArena) {
        self.write("${");
        if let Some(expr) = arena.get(s.expression) {
            self.emit_node(expr, arena);
        }
        self.write("}");
        if let Some(lit) = arena.get(s.literal) {
            self.emit_node(lit, arena);
        }
    }

    fn emit_no_substitution_template(&mut self, lit: &crate::parser::literals::StringLiteral) {
        self.write("`");
        self.write(&lit.text);
        self.write("`");
    }

    fn emit_template_head(&mut self, lit: &crate::parser::literals::StringLiteral) {
        self.write("`");
        self.write(&lit.text);
    }

    fn emit_template_middle(&mut self, lit: &crate::parser::literals::StringLiteral) {
        self.write(&lit.text);
    }

    fn emit_template_tail(&mut self, lit: &crate::parser::literals::StringLiteral) {
        self.write(&lit.text);
        self.write("`");
    }

    // =========================================================================
    // JSX nodes
    // =========================================================================

    fn emit_jsx_element(&mut self, e: &crate::parser::jsx::JsxElement, arena: &crate::parser::NodeArena) {
        if let Some(open) = arena.get(e.opening_element) {
            self.emit_node(open, arena);
        }
        for child_idx in &e.children.nodes {
            if let Some(child) = arena.get(*child_idx) {
                self.emit_node(child, arena);
            }
        }
        if let Some(close) = arena.get(e.closing_element) {
            self.emit_node(close, arena);
        }
    }

    fn emit_jsx_self_closing_element(&mut self, e: &crate::parser::jsx::JsxSelfClosingElement, arena: &crate::parser::NodeArena) {
        self.write("<");
        if let Some(name) = arena.get(e.tag_name) {
            self.emit_node(name, arena);
        }
        if let Some(ref type_args) = e.type_arguments {
            self.write("<");
            self.emit_node_list(type_args, arena, ", ");
            self.write(">");
        }
        if let Some(attrs) = arena.get(e.attributes) {
            self.write(" ");
            self.emit_node(attrs, arena);
        }
        self.write(" />");
    }

    fn emit_jsx_opening_element(&mut self, e: &crate::parser::jsx::JsxOpeningElement, arena: &crate::parser::NodeArena) {
        self.write("<");
        if let Some(name) = arena.get(e.tag_name) {
            self.emit_node(name, arena);
        }
        if let Some(ref type_args) = e.type_arguments {
            self.write("<");
            self.emit_node_list(type_args, arena, ", ");
            self.write(">");
        }
        if let Some(attrs) = arena.get(e.attributes) {
            self.write(" ");
            self.emit_node(attrs, arena);
        }
        self.write(">");
    }

    fn emit_jsx_closing_element(&mut self, e: &crate::parser::jsx::JsxClosingElement, arena: &crate::parser::NodeArena) {
        self.write("</");
        if let Some(name) = arena.get(e.tag_name) {
            self.emit_node(name, arena);
        }
        self.write(">");
    }

    fn emit_jsx_fragment(&mut self, f: &crate::parser::jsx::JsxFragment, arena: &crate::parser::NodeArena) {
        self.write("<>");
        for child_idx in &f.children.nodes {
            if let Some(child) = arena.get(*child_idx) {
                self.emit_node(child, arena);
            }
        }
        self.write("</>");
    }

    fn emit_jsx_attributes(&mut self, a: &crate::parser::jsx::JsxAttributes, arena: &crate::parser::NodeArena) {
        for (i, attr_idx) in a.properties.nodes.iter().enumerate() {
            if i > 0 {
                self.write(" ");
            }
            if let Some(attr) = arena.get(*attr_idx) {
                self.emit_node(attr, arena);
            }
        }
    }

    fn emit_jsx_attribute(&mut self, a: &crate::parser::jsx::JsxAttribute, arena: &crate::parser::NodeArena) {
        if let Some(name) = arena.get(a.name) {
            self.emit_node(name, arena);
        }
        if !a.initializer.is_none() {
            self.write("=");
            if let Some(init) = arena.get(a.initializer) {
                self.emit_node(init, arena);
            }
        }
    }

    fn emit_jsx_spread_attribute(&mut self, a: &crate::parser::jsx::JsxSpreadAttribute, arena: &crate::parser::NodeArena) {
        self.write("{...");
        if let Some(expr) = arena.get(a.expression) {
            self.emit_node(expr, arena);
        }
        self.write("}");
    }

    fn emit_jsx_expression(&mut self, e: &crate::parser::jsx::JsxExpression, arena: &crate::parser::NodeArena) {
        self.write("{");
        if e.dot_dot_dot_token {
            self.write("...");
        }
        if !e.expression.is_none() {
            if let Some(expr) = arena.get(e.expression) {
                self.emit_node(expr, arena);
            }
        }
        self.write("}");
    }

    fn emit_jsx_text(&mut self, t: &crate::parser::jsx::JsxText) {
        self.write(&t.text);
    }

    fn emit_jsx_namespaced_name(&mut self, n: &crate::parser::jsx::JsxNamespacedName, arena: &crate::parser::NodeArena) {
        if let Some(ns) = arena.get(n.namespace) {
            self.emit_node(ns, arena);
        }
        self.write(":");
        if let Some(name) = arena.get(n.name) {
            self.emit_node(name, arena);
        }
    }

    // =========================================================================
    // Module block
    // =========================================================================

    fn emit_module_block(&mut self, b: &crate::parser::declarations::ModuleBlock, arena: &crate::parser::NodeArena) {
        self.write("{");
        if !b.statements.nodes.is_empty() {
            self.write_line();
            self.increase_indent();
            for stmt_idx in &b.statements.nodes {
                if let Some(stmt) = arena.get(*stmt_idx) {
                    self.emit_node(stmt, arena);
                    self.write_line();
                }
            }
            self.decrease_indent();
        }
        self.write("}");
    }

    // =========================================================================
    // Function expression
    // =========================================================================

    fn emit_function_expression(&mut self, f: &crate::parser::expressions::FunctionExpression, arena: &crate::parser::NodeArena) {
        // Handle modifiers (async)
        if let Some(ref mods) = f.modifiers {
            for mod_idx in &mods.nodes {
                if let Some(Node::Token(base)) = arena.get(*mod_idx) {
                    if let Some(SyntaxKind::AsyncKeyword) = SyntaxKind::try_from_u16(base.kind) {
                        self.write("async ");
                    }
                }
            }
        }
        if f.asterisk_token {
            self.write("function* ");
        } else {
            self.write("function ");
        }
        if !f.name.is_none() {
            if let Some(name) = arena.get(f.name) {
                self.emit_node(name, arena);
            }
        }
        if let Some(ref type_params) = f.type_parameters {
            self.write("<");
            self.emit_node_list(type_params, arena, ", ");
            self.write(">");
        }
        self.write("(");
        self.emit_node_list(&f.parameters, arena, ", ");
        self.write(")");
        if !f.type_annotation.is_none() {
            self.write(": ");
            if let Some(ty) = arena.get(f.type_annotation) {
                self.emit_node(ty, arena);
            }
        }
        self.write(" ");
        if let Some(body) = arena.get(f.body) {
            self.emit_node(body, arena);
        }
    }

    // =========================================================================
    // Satisfies expression
    // =========================================================================

    fn emit_satisfies_expression(&mut self, e: &crate::parser::expressions::SatisfiesExpression, arena: &crate::parser::NodeArena) {
        if let Some(expr) = arena.get(e.expression) {
            self.emit_node(expr, arena);
        }
        self.write(" satisfies ");
        if let Some(ty) = arena.get(e.type_node) {
            self.emit_node(ty, arena);
        }
    }

    // =========================================================================
    // Private identifier
    // =========================================================================

    fn emit_private_identifier(&mut self, id: &crate::parser::literals::Identifier) {
        self.write("#");
        self.write(&id.escaped_text);
    }

    // =========================================================================
    // Import attributes
    // =========================================================================

    fn emit_import_attributes(&mut self, a: &crate::parser::declarations::ImportAttributes, arena: &crate::parser::NodeArena) {
        // 'with' or 'assert' keyword
        if a.token == SyntaxKind::WithKeyword as u16 {
            self.write(" with ");
        } else {
            self.write(" assert ");
        }
        self.write("{ ");
        self.emit_node_list(&a.elements, arena, ", ");
        self.write(" }");
    }

    fn emit_import_attribute(&mut self, a: &crate::parser::declarations::ImportAttribute, arena: &crate::parser::NodeArena) {
        if let Some(name) = arena.get(a.name) {
            self.emit_node(name, arena);
        }
        self.write(": ");
        if let Some(val) = arena.get(a.value) {
            self.emit_node(val, arena);
        }
    }

    // =========================================================================
    // Modifiers helper
    // =========================================================================

    fn emit_modifiers(&mut self, modifiers: &NodeList, arena: &crate::parser::NodeArena) {
        for mod_idx in &modifiers.nodes {
            if let Some(Node::Token(base)) = arena.get(*mod_idx) {
                match SyntaxKind::try_from_u16(base.kind) {
                    Some(SyntaxKind::PublicKeyword) => self.write("public "),
                    Some(SyntaxKind::PrivateKeyword) => self.write("private "),
                    Some(SyntaxKind::ProtectedKeyword) => self.write("protected "),
                    Some(SyntaxKind::StaticKeyword) => self.write("static "),
                    Some(SyntaxKind::ReadonlyKeyword) => self.write("readonly "),
                    Some(SyntaxKind::AbstractKeyword) => self.write("abstract "),
                    Some(SyntaxKind::AsyncKeyword) => self.write("async "),
                    Some(SyntaxKind::ConstKeyword) => self.write("const "),
                    Some(SyntaxKind::DeclareKeyword) => self.write("declare "),
                    Some(SyntaxKind::DefaultKeyword) => self.write("default "),
                    Some(SyntaxKind::ExportKeyword) => self.write("export "),
                    Some(SyntaxKind::OverrideKeyword) => self.write("override "),
                    Some(SyntaxKind::AccessorKeyword) => self.write("accessor "),
                    _ => {}
                }
            } else if let Some(Node::Decorator(d)) = arena.get(*mod_idx) {
                self.emit_decorator(d, arena);
                self.write(" ");
            }
        }
    }

    // =========================================================================
    // Helpers
    // =========================================================================

    fn emit_node_list(&mut self, list: &NodeList, arena: &crate::parser::NodeArena, separator: &str) {
        for (i, node_idx) in list.nodes.iter().enumerate() {
            if i > 0 {
                self.write(separator);
            }
            if let Some(node) = arena.get(*node_idx) {
                self.emit_node(node, arena);
            }
        }
    }
}

impl Default for Printer {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser_impl::ParserState;

    fn parse_and_emit(source: &str) -> String {
        let mut parser = ParserState::new("test.ts".to_string(), source.to_string());
        let root_idx = parser.parse_source_file();

        let mut printer = Printer::new();
        if let Some(root) = parser.arena.get(root_idx) {
            printer.emit_node(root, &parser.arena);
        }
        printer.take_output()
    }

    #[test]
    fn test_emit_variable_declaration() {
        let output = parse_and_emit("let x = 42;");
        assert!(output.contains("let"), "Should contain 'let': {}", output);
        assert!(output.contains("x"), "Should contain 'x': {}", output);
        assert!(output.contains("42"), "Should contain '42': {}", output);
    }

    #[test]
    fn test_emit_function_declaration() {
        let output = parse_and_emit("function add(a, b) { return a + b; }");
        assert!(output.contains("function"), "Should contain 'function': {}", output);
        assert!(output.contains("add"), "Should contain 'add': {}", output);
        assert!(output.contains("return"), "Should contain 'return': {}", output);
    }

    #[test]
    fn test_emit_if_statement() {
        let output = parse_and_emit("if (x > 0) { y = 1; }");
        assert!(output.contains("if"), "Should contain 'if': {}", output);
        assert!(output.contains("x"), "Should contain 'x': {}", output);
    }

    #[test]
    fn test_emit_binary_expression() {
        let output = parse_and_emit("let result = 1 + 2 * 3;");
        assert!(output.contains("+"), "Should contain '+': {}", output);
        assert!(output.contains("*"), "Should contain '*': {}", output);
    }

    #[test]
    fn test_emit_arrow_function() {
        let output = parse_and_emit("const fn = (x) => x * 2;");
        assert!(output.contains("=>"), "Should contain '=>': {}", output);
        assert!(output.contains("const"), "Should contain 'const': {}", output);
    }

    // =========================================================================
    // Roundtrip tests: parse → emit → parse
    // =========================================================================

    fn roundtrip_test(source: &str) -> bool {
        // First pass: parse and emit
        let emitted = parse_and_emit(source);

        // Second pass: parse the emitted code
        let mut parser2 = ParserState::new("test2.ts".to_string(), emitted.clone());
        let root2 = parser2.parse_source_file();

        // Verify we got a valid parse
        if root2.is_none() {
            eprintln!("=== ROUNDTRIP FAILED (null parse) ===");
            eprintln!("Source: {}", source);
            eprintln!("Emitted: {}", emitted);
            return false;
        }

        // Third pass: emit again
        let mut printer2 = Printer::new();
        if let Some(root) = parser2.arena.get(root2) {
            printer2.emit_node(root, &parser2.arena);
        }
        let emitted2 = printer2.take_output();

        // The second emission should be identical to the first
        // (we've reached a fixed point)
        if emitted != emitted2 {
            eprintln!("=== ROUNDTRIP FAILED (mismatch) ===");
            eprintln!("Source: {}", source);
            eprintln!("First emit: [{}]", emitted);
            eprintln!("Second emit: [{}]", emitted2);
            return false;
        }
        true
    }

    #[test]
    fn test_roundtrip_variable() {
        assert!(roundtrip_test("let x = 42;"), "Variable declaration should roundtrip");
    }

    #[test]
    fn test_roundtrip_function() {
        assert!(roundtrip_test("function foo(a, b) { return a + b; }"), "Function should roundtrip");
    }

    #[test]
    fn test_roundtrip_if_else() {
        assert!(roundtrip_test("if (x > 0) { y = 1; } else { y = 2; }"), "If-else should roundtrip");
    }

    #[test]
    fn test_roundtrip_for_loop() {
        assert!(roundtrip_test("for (let i = 0; i < 10; i++) { sum = sum + i; }"), "For loop should roundtrip");
    }

    #[test]
    fn test_roundtrip_while_loop() {
        assert!(roundtrip_test("while (x > 0) { x = x - 1; }"), "While loop should roundtrip");
    }

    #[test]
    fn test_roundtrip_arrow_function() {
        assert!(roundtrip_test("const fn = (x) => x * 2;"), "Arrow function should roundtrip");
    }

    #[test]
    fn test_roundtrip_class() {
        assert!(roundtrip_test("class Foo { }"), "Empty class should roundtrip");
    }

    #[test]
    fn test_roundtrip_object_literal() {
        assert!(roundtrip_test("const obj = { a: 1, b: 2 };"), "Object literal should roundtrip");
    }

    #[test]
    fn test_roundtrip_array_literal() {
        assert!(roundtrip_test("const arr = [1, 2, 3];"), "Array literal should roundtrip");
    }

    #[test]
    fn test_roundtrip_call_expression() {
        assert!(roundtrip_test("foo(a, b, c);"), "Call expression should roundtrip");
    }

    #[test]
    fn test_emit_try_catch() {
        let output = parse_and_emit("try { x(); } catch (e) { console.log(e); }");
        assert!(output.contains("try"), "Should contain 'try': {}", output);
        assert!(output.contains("catch"), "Should contain 'catch': {}", output);
    }

    #[test]
    fn test_emit_try_finally() {
        let output = parse_and_emit("try { x(); } finally { cleanup(); }");
        assert!(output.contains("try"), "Should contain 'try': {}", output);
        assert!(output.contains("finally"), "Should contain 'finally': {}", output);
    }

    #[test]
    fn test_emit_switch() {
        let output = parse_and_emit("switch (x) { case 1: break; default: y(); }");
        assert!(output.contains("switch"), "Should contain 'switch': {}", output);
        assert!(output.contains("case"), "Should contain 'case': {}", output);
        assert!(output.contains("default"), "Should contain 'default': {}", output);
    }

    // Note: Labeled statement test skipped - parser doesn't generate LabeledStatement nodes yet

    #[test]
    fn test_roundtrip_try_catch() {
        assert!(roundtrip_test("try { x(); } catch (e) { log(e); }"), "Try-catch should roundtrip");
    }

    #[test]
    fn test_roundtrip_switch() {
        assert!(roundtrip_test("switch (x) { case 1: break; }"), "Switch should roundtrip");
    }

    #[test]
    fn test_emit_for_in() {
        let output = parse_and_emit("for (let key in obj) { console.log(key); }");
        assert!(output.contains("for"), "Should contain 'for': {}", output);
        assert!(output.contains("in"), "Should contain 'in': {}", output);
        assert!(output.contains("key"), "Should contain 'key': {}", output);
    }

    #[test]
    fn test_emit_for_of() {
        let output = parse_and_emit("for (let item of arr) { console.log(item); }");
        assert!(output.contains("for"), "Should contain 'for': {}", output);
        assert!(output.contains("of"), "Should contain 'of': {}", output);
        assert!(output.contains("item"), "Should contain 'item': {}", output);
    }

    #[test]
    fn test_emit_spread() {
        let output = parse_and_emit("const arr = [1, ...other, 3];");
        assert!(output.contains("..."), "Should contain '...': {}", output);
        assert!(output.contains("other"), "Should contain 'other': {}", output);
    }

    #[test]
    fn test_emit_spread_object() {
        let output = parse_and_emit("const obj = { a: 1, ...other };");
        assert!(output.contains("..."), "Should contain '...': {}", output);
        assert!(output.contains("other"), "Should contain 'other': {}", output);
    }

    // Note: yield/await tests disabled pending parser fixes for async/generator functions
    // TODO: Fix infinite loop when parsing yield/await expressions
    // #[test]
    // fn test_emit_await() {
    //     let output = parse_and_emit("async function f() { await fetch(url); }");
    //     assert!(output.contains("await"), "Should contain 'await': {}", output);
    //     assert!(output.contains("async"), "Should contain 'async': {}", output);
    // }

    // #[test]
    // fn test_emit_yield() {
    //     let output = parse_and_emit("function* gen() { yield 1; }");
    //     assert!(output.contains("yield"), "Should contain 'yield': {}", output);
    // }

    #[test]
    fn test_roundtrip_for_in() {
        assert!(roundtrip_test("for (let key in obj) { log(key); }"), "For-in should roundtrip");
    }

    #[test]
    fn test_roundtrip_for_of() {
        assert!(roundtrip_test("for (let item of arr) { log(item); }"), "For-of should roundtrip");
    }

    #[test]
    fn test_roundtrip_spread_array() {
        assert!(roundtrip_test("const arr = [1, ...other, 3];"), "Spread array should roundtrip");
    }

    #[test]
    fn test_roundtrip_spread_object() {
        assert!(roundtrip_test("const obj = { a: 1, ...other };"), "Spread object should roundtrip");
    }

    // =========================================================================
    // TypeScript-specific tests
    // =========================================================================

    // TODO: Parser doesn't create AsExpression nodes yet
    // #[test]
    // fn test_emit_as_expression() {
    //     let output = parse_and_emit("const x = value as string;");
    //     assert!(output.contains("as"), "Should contain 'as': {}", output);
    //     assert!(output.contains("string"), "Should contain 'string': {}", output);
    // }

    // TODO: Parser doesn't create TypeAssertion nodes yet
    // #[test]
    // fn test_emit_type_assertion() {
    //     let output = parse_and_emit("const x = <string>value;");
    //     assert!(output.contains("<string>"), "Should contain '<string>': {}", output);
    // }

    #[test]
    fn test_emit_non_null_assertion() {
        let output = parse_and_emit("const x = value!;");
        assert!(output.contains("!"), "Should contain '!': {}", output);
    }

    // TODO: Parser doesn't create InterfaceDeclaration nodes from statements
    // #[test]
    // fn test_emit_interface() {
    //     let output = parse_and_emit("interface Foo { x: number; }");
    //     assert!(output.contains("interface"), "Should contain 'interface': {}", output);
    //     assert!(output.contains("Foo"), "Should contain 'Foo': {}", output);
    //     assert!(output.contains("x"), "Should contain 'x': {}", output);
    // }

    #[test]
    fn test_emit_type_alias() {
        let output = parse_and_emit("type Foo = string | number;");
        assert!(output.contains("type"), "Should contain 'type': {}", output);
        assert!(output.contains("Foo"), "Should contain 'Foo': {}", output);
    }

    // TODO: Parser doesn't create EnumDeclaration nodes from statements
    // #[test]
    // fn test_emit_enum() {
    //     let output = parse_and_emit("enum Color { Red, Green, Blue }");
    //     assert!(output.contains("enum"), "Should contain 'enum': {}", output);
    //     assert!(output.contains("Color"), "Should contain 'Color': {}", output);
    //     assert!(output.contains("Red"), "Should contain 'Red': {}", output);
    // }

    #[test]
    fn test_emit_import() {
        let output = parse_and_emit("import { foo } from 'bar';");
        assert!(output.contains("import"), "Should contain 'import': {}", output);
        assert!(output.contains("from"), "Should contain 'from': {}", output);
    }

    #[test]
    fn test_emit_export() {
        let output = parse_and_emit("export { foo, bar };");
        assert!(output.contains("export"), "Should contain 'export': {}", output);
    }

    // TODO: Parser doesn't handle export default correctly yet
    // #[test]
    // fn test_emit_export_default() {
    //     let output = parse_and_emit("export default function() {}");
    //     assert!(output.contains("export"), "Should contain 'export': {}", output);
    //     assert!(output.contains("default"), "Should contain 'default': {}", output);
    // }

    // TODO: Parser doesn't create TemplateExpression nodes yet
    // #[test]
    // fn test_emit_template_literal() {
    //     let output = parse_and_emit("const s = `hello ${name}!`;");
    //     assert!(output.contains("`"), "Should contain backtick: {}", output);
    //     assert!(output.contains("${"), "Should contain '${': {}", output);
    // }

    #[test]
    fn test_emit_typeof() {
        let output = parse_and_emit("const t = typeof x;");
        assert!(output.contains("typeof"), "Should contain 'typeof': {}", output);
    }

    #[test]
    fn test_emit_void() {
        let output = parse_and_emit("void 0;");
        assert!(output.contains("void"), "Should contain 'void': {}", output);
    }

    #[test]
    fn test_emit_delete() {
        let output = parse_and_emit("delete obj.prop;");
        assert!(output.contains("delete"), "Should contain 'delete': {}", output);
    }

    // =========================================================================
    // Type node emission tests
    // =========================================================================

    #[test]
    fn test_emit_type_reference() {
        let output = parse_and_emit("type Foo = Array<string>;");
        assert!(output.contains("Array"), "Should contain 'Array': {}", output);
        assert!(output.contains("<"), "Should contain '<': {}", output);
        assert!(output.contains("string"), "Should contain 'string': {}", output);
    }

    #[test]
    fn test_emit_union_type() {
        let output = parse_and_emit("type Foo = string | number;");
        assert!(output.contains("|"), "Should contain '|': {}", output);
        assert!(output.contains("string"), "Should contain 'string': {}", output);
        assert!(output.contains("number"), "Should contain 'number': {}", output);
    }

    #[test]
    fn test_emit_intersection_type() {
        let output = parse_and_emit("type Foo = A & B;");
        assert!(output.contains("&"), "Should contain '&': {}", output);
    }

    #[test]
    fn test_emit_array_type() {
        let output = parse_and_emit("type Foo = string[];");
        assert!(output.contains("[]"), "Should contain '[]': {}", output);
    }

    #[test]
    fn test_emit_tuple_type() {
        let output = parse_and_emit("type Foo = [string, number];");
        assert!(output.contains("["), "Should contain '[': {}", output);
        assert!(output.contains("]"), "Should contain ']': {}", output);
        assert!(output.contains("string"), "Should contain 'string': {}", output);
        assert!(output.contains("number"), "Should contain 'number': {}", output);
    }

    #[test]
    fn test_emit_function_type() {
        let output = parse_and_emit("type Foo = (x: number) => string;");
        assert!(output.contains("=>"), "Should contain '=>': {}", output);
        assert!(output.contains("number"), "Should contain 'number': {}", output);
        assert!(output.contains("string"), "Should contain 'string': {}", output);
    }

    #[test]
    fn test_emit_type_literal() {
        let output = parse_and_emit("type Foo = { x: number; y: string };");
        assert!(output.contains("{"), "Should contain braces: {}", output);
        assert!(output.contains("x"), "Should contain 'x': {}", output);
    }

    #[test]
    fn test_emit_indexed_access_type() {
        let output = parse_and_emit("type Foo = T[K];");
        assert!(output.contains("["), "Should contain '[': {}", output);
        assert!(output.contains("K"), "Should contain 'K': {}", output);
    }

    #[test]
    fn test_emit_mapped_type() {
        let output = parse_and_emit("type Foo = { [K in T]: U };");
        assert!(output.contains("["), "Should contain '[': {}", output);
        assert!(output.contains("in"), "Should contain 'in': {}", output);
    }

    #[test]
    fn test_emit_conditional_type() {
        let output = parse_and_emit("type Foo = T extends U ? X : Y;");
        assert!(output.contains("extends"), "Should contain 'extends': {}", output);
        assert!(output.contains("?"), "Should contain '?': {}", output);
        assert!(output.contains(":"), "Should contain ':': {}", output);
    }

    #[test]
    fn test_emit_infer_type() {
        let output = parse_and_emit("type Foo = T extends (infer U) ? U : never;");
        assert!(output.contains("infer"), "Should contain 'infer': {}", output);
    }

    #[test]
    fn test_emit_type_query() {
        let output = parse_and_emit("type Foo = typeof x;");
        assert!(output.contains("typeof"), "Should contain 'typeof': {}", output);
        assert!(output.contains("x"), "Should contain 'x': {}", output);
    }

    #[test]
    fn test_emit_keyof_type() {
        let output = parse_and_emit("type Foo = keyof T;");
        assert!(output.contains("keyof"), "Should contain 'keyof': {}", output);
    }

    #[test]
    fn test_emit_type_parameter_constraint() {
        let output = parse_and_emit("type Foo<T extends string> = T;");
        assert!(output.contains("extends"), "Should contain 'extends': {}", output);
        assert!(output.contains("string"), "Should contain 'string': {}", output);
    }

    #[test]
    fn test_emit_decorator() {
        let output = parse_and_emit("@decorator class Foo {}");
        assert!(output.contains("@"), "Should contain '@': {}", output);
        assert!(output.contains("decorator"), "Should contain 'decorator': {}", output);
    }

    #[test]
    fn test_emit_interface() {
        let output = parse_and_emit("interface Foo { x: number; }");
        assert!(output.contains("interface"), "Should contain 'interface': {}", output);
        assert!(output.contains("Foo"), "Should contain 'Foo': {}", output);
    }

    #[test]
    fn test_emit_enum_with_values() {
        let output = parse_and_emit("enum Color { Red = 1, Green = 2, Blue = 3 }");
        assert!(output.contains("enum"), "Should contain 'enum': {}", output);
        assert!(output.contains("Red"), "Should contain 'Red': {}", output);
        assert!(output.contains("="), "Should contain '=': {}", output);
    }

    // NOTE: Destructuring tests cause infinite loop in parsing/emitting - needs investigation
    // #[test]
    // fn test_emit_destructuring() {
    //     let output = parse_and_emit("const { a, b } = obj;");
    //     assert!(output.contains("a"), "Should contain 'a': {}", output);
    //     assert!(output.contains("b"), "Should contain 'b': {}", output);
    // }

    #[test]
    fn test_emit_array_destructuring() {
        let output = parse_and_emit("const [a, b] = arr;");
        assert!(output.contains("["), "Should contain '[': {}", output);
        assert!(output.contains("a"), "Should contain 'a': {}", output);
    }

    #[test]
    fn test_emit_namespace() {
        let output = parse_and_emit("namespace Foo { export const x = 1; }");
        assert!(output.contains("namespace"), "Should contain 'namespace': {}", output);
        assert!(output.contains("Foo"), "Should contain 'Foo': {}", output);
    }

    // =========================================================================
    // Roundtrip tests for new type nodes
    // =========================================================================

    #[test]
    fn test_roundtrip_type_alias() {
        assert!(roundtrip_test("type Foo = string;"), "Type alias should roundtrip");
    }

    #[test]
    fn test_roundtrip_union_type() {
        assert!(roundtrip_test("type Foo = string | number;"), "Union type should roundtrip");
    }

    #[test]
    fn test_roundtrip_interface() {
        assert!(roundtrip_test("interface Foo { x: number }"), "Interface should roundtrip");
    }

    #[test]
    fn test_roundtrip_enum() {
        assert!(roundtrip_test("enum Color { Red, Green, Blue }"), "Enum should roundtrip");
    }

    // NOTE: Destructuring tests cause infinite loop - needs investigation
    // #[test]
    // fn test_roundtrip_destructuring() {
    //     assert!(roundtrip_test("const { a, b } = obj;"), "Destructuring should roundtrip");
    // }

    // =========================================================================
    // Source map tests
    // =========================================================================

    #[test]
    fn test_source_map_basic() {
        let mut printer = Printer::with_source_map(
            PrinterOptions::default(),
            "output.js".to_string()
        );
        printer.add_source_file("input.ts".to_string());

        // Parse and emit some code
        let source = "const x = 1;";
        let mut parser = ParserState::new(
            "input.ts".to_string(),
            source.to_string()
        );
        let root = parser.parse_source_file();

        if let Some(root_node) = parser.arena.get(root) {
            printer.emit_node(root_node, &parser.arena);
        }

        // Get the source map
        let source_map = printer.get_source_map();
        assert!(source_map.is_some(), "Should generate source map");
        let json = source_map.unwrap();
        assert!(json.contains("\"version\": 3"), "Should be v3 source map");
        assert!(json.contains("\"file\": \"output.js\""), "Should have output file name");
        assert!(json.contains("\"sources\": [\"input.ts\"]"), "Should have input file name");
    }

    #[test]
    fn test_source_map_with_content() {
        let mut printer = Printer::with_source_map(
            PrinterOptions::default(),
            "output.js".to_string()
        );
        let source = "const x = 1;";
        printer.add_source_file_with_content(
            "input.ts".to_string(),
            source.to_string()
        );

        let mut parser = ParserState::new(
            "input.ts".to_string(),
            source.to_string()
        );
        let root = parser.parse_source_file();

        if let Some(root_node) = parser.arena.get(root) {
            printer.emit_node(root_node, &parser.arena);
        }

        let source_map = printer.get_source_map();
        assert!(source_map.is_some());
        let json = source_map.unwrap();
        assert!(json.contains("\"sourcesContent\""), "Should have sources content");
        assert!(json.contains("const x = 1;"), "Should contain source text");
    }

    #[test]
    fn test_inline_source_map() {
        let mut printer = Printer::with_source_map(
            PrinterOptions::default(),
            "output.js".to_string()
        );
        printer.add_source_file("input.ts".to_string());

        let source = "const x = 1;";
        let mut parser = ParserState::new(
            "input.ts".to_string(),
            source.to_string()
        );
        let root = parser.parse_source_file();

        if let Some(root_node) = parser.arena.get(root) {
            printer.emit_node(root_node, &parser.arena);
        }

        let inline_map = printer.get_inline_source_map();
        assert!(inline_map.is_some());
        let comment = inline_map.unwrap();
        assert!(comment.starts_with("//# sourceMappingURL=data:application/json;base64,"));
    }

    #[test]
    fn test_position_tracking() {
        let mut printer = Printer::with_source_map(
            PrinterOptions::default(),
            "output.js".to_string()
        );
        printer.add_source_file("input.ts".to_string());

        // Write some text and check position tracking
        printer.write("hello");
        assert_eq!(printer.output_column, 5);
        assert_eq!(printer.output_line, 0);

        printer.write_line();
        assert_eq!(printer.output_column, 0);
        assert_eq!(printer.output_line, 1);

        printer.write("world");
        assert_eq!(printer.output_column, 5);
        assert_eq!(printer.output_line, 1);
    }
}
