//! ThinEmitter - Emitter using ThinNodeArena
//!
//! This emitter uses the ThinNode architecture for cache-optimized AST access.
//! It works directly with ThinNodeArena instead of the old Node enum.
//!
//! # Architecture
//!
//! - Uses ThinNodeArena for AST access (16-byte nodes, 13x cache improvement)
//! - Dispatches based on ThinNode.kind (u16)
//! - Uses accessor methods to get typed node data

// Allow dead code for emitter infrastructure methods that will be used in future phases
#![allow(dead_code)]
//!
//! # Status
//!
//! This is an initial implementation with core emit methods.
//! More emit methods will be added as needed.

use crate::parser::{NodeIndex, NodeList};
use crate::parser::thin_node::{ThinNode, ThinNodeArena};
use crate::parser::syntax_kind_ext;
use crate::scanner::SyntaxKind;
use crate::emitter::{PrinterOptions, NewLineKind};

// =============================================================================
// ThinPrinter
// =============================================================================

/// Printer that works with ThinNodeArena.
pub struct ThinPrinter<'a> {
    /// The ThinNodeArena containing the AST.
    arena: &'a ThinNodeArena,

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

    /// Current output line (0-indexed)
    output_line: u32,

    /// Current output column (0-indexed)
    output_column: u32,
}

impl<'a> ThinPrinter<'a> {
    /// Create a new ThinPrinter.
    pub fn new(arena: &'a ThinNodeArena) -> Self {
        Self::with_options(arena, PrinterOptions::default())
    }

    /// Create a new ThinPrinter with options.
    pub fn with_options(arena: &'a ThinNodeArena, options: PrinterOptions) -> Self {
        let new_line = match options.new_line {
            NewLineKind::LineFeed => "\n".to_string(),
            NewLineKind::CarriageReturnLineFeed => "\r\n".to_string(),
        };
        ThinPrinter {
            arena,
            output: String::with_capacity(1024),
            indent_level: 0,
            indent_str: "    ".to_string(),
            new_line,
            options,
            at_line_start: true,
            output_line: 0,
            output_column: 0,
        }
    }

    /// Get the output.
    pub fn get_output(&self) -> &str {
        &self.output
    }

    /// Take the output.
    pub fn take_output(self) -> String {
        self.output
    }

    // =========================================================================
    // Output Helpers
    // =========================================================================

    /// Write text to output.
    fn write(&mut self, text: &str) {
        if self.at_line_start && self.indent_level > 0 {
            for _ in 0..self.indent_level {
                self.output.push_str(&self.indent_str);
                self.output_column += self.indent_str.len() as u32;
            }
            self.at_line_start = false;
        }

        for ch in text.chars() {
            if ch == '\n' {
                self.output_line += 1;
                self.output_column = 0;
            } else {
                self.output_column += 1;
            }
        }
        self.output.push_str(text);
    }

    /// Write a single character.
    fn write_char(&mut self, ch: char) {
        if self.at_line_start && self.indent_level > 0 {
            for _ in 0..self.indent_level {
                self.output.push_str(&self.indent_str);
                self.output_column += self.indent_str.len() as u32;
            }
            self.at_line_start = false;
        }

        if ch == '\n' {
            self.output_line += 1;
            self.output_column = 0;
        } else {
            self.output_column += 1;
        }
        self.output.push(ch);
    }

    /// Write a newline.
    fn write_line(&mut self) {
        self.output.push_str(&self.new_line);
        self.output_line += 1;
        self.output_column = 0;
        self.at_line_start = true;
    }

    /// Write a space.
    fn write_space(&mut self) {
        self.write(" ");
    }

    /// Write a semicolon (respecting options).
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
    // Main Emit Method
    // =========================================================================

    /// Emit a node by index.
    pub fn emit(&mut self, idx: NodeIndex) {
        if idx.is_none() {
            return;
        }

        let Some(node) = self.arena.get(idx) else {
            return;
        };

        self.emit_node(node, idx);
    }

    /// Emit a node.
    fn emit_node(&mut self, node: &ThinNode, idx: NodeIndex) {
        let kind = node.kind;

        match kind {
            // Identifiers
            k if k == SyntaxKind::Identifier as u16 => {
                self.emit_identifier(node);
            }

            // Literals
            k if k == SyntaxKind::NumericLiteral as u16 => {
                self.emit_numeric_literal(node);
            }
            k if k == SyntaxKind::StringLiteral as u16 => {
                self.emit_string_literal(node);
            }
            k if k == SyntaxKind::TrueKeyword as u16 => {
                self.write("true");
            }
            k if k == SyntaxKind::FalseKeyword as u16 => {
                self.write("false");
            }
            k if k == SyntaxKind::NullKeyword as u16 => {
                self.write("null");
            }

            // Binary expression
            k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                self.emit_binary_expression(node);
            }

            // Unary expressions
            k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION => {
                self.emit_prefix_unary(node);
            }
            k if k == syntax_kind_ext::POSTFIX_UNARY_EXPRESSION => {
                self.emit_postfix_unary(node);
            }

            // Call expression
            k if k == syntax_kind_ext::CALL_EXPRESSION => {
                self.emit_call_expression(node);
            }

            // New expression
            k if k == syntax_kind_ext::NEW_EXPRESSION => {
                self.emit_new_expression(node);
            }

            // Property access
            k if k == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION => {
                self.emit_property_access(node);
            }

            // Element access
            k if k == syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION => {
                self.emit_element_access(node);
            }

            // Parenthesized expression
            k if k == syntax_kind_ext::PARENTHESIZED_EXPRESSION => {
                self.emit_parenthesized(node);
            }

            // Conditional expression
            k if k == syntax_kind_ext::CONDITIONAL_EXPRESSION => {
                self.emit_conditional(node);
            }

            // Array literal
            k if k == syntax_kind_ext::ARRAY_LITERAL_EXPRESSION => {
                self.emit_array_literal(node);
            }

            // Object literal
            k if k == syntax_kind_ext::OBJECT_LITERAL_EXPRESSION => {
                self.emit_object_literal(node);
            }

            // Arrow function
            k if k == syntax_kind_ext::ARROW_FUNCTION => {
                self.emit_arrow_function(node, idx);
            }

            // Function expression
            k if k == syntax_kind_ext::FUNCTION_EXPRESSION => {
                self.emit_function_expression(node, idx);
            }

            // Function declaration
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                self.emit_function_declaration(node, idx);
            }

            // Variable declaration
            k if k == syntax_kind_ext::VARIABLE_DECLARATION => {
                self.emit_variable_declaration(node);
            }

            // Variable declaration list
            k if k == syntax_kind_ext::VARIABLE_DECLARATION_LIST => {
                self.emit_variable_declaration_list(node);
            }

            // Variable statement
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                self.emit_variable_statement(node);
            }

            // Expression statement
            k if k == syntax_kind_ext::EXPRESSION_STATEMENT => {
                self.emit_expression_statement(node);
            }

            // Block
            k if k == syntax_kind_ext::BLOCK => {
                self.emit_block(node);
            }

            // If statement
            k if k == syntax_kind_ext::IF_STATEMENT => {
                self.emit_if_statement(node);
            }

            // While statement
            k if k == syntax_kind_ext::WHILE_STATEMENT => {
                self.emit_while_statement(node);
            }

            // For statement
            k if k == syntax_kind_ext::FOR_STATEMENT => {
                self.emit_for_statement(node);
            }

            // For-in statement
            k if k == syntax_kind_ext::FOR_IN_STATEMENT => {
                self.emit_for_in_statement(node);
            }

            // For-of statement
            k if k == syntax_kind_ext::FOR_OF_STATEMENT => {
                self.emit_for_of_statement(node);
            }

            // Return statement
            k if k == syntax_kind_ext::RETURN_STATEMENT => {
                self.emit_return_statement(node);
            }

            // Class declaration
            k if k == syntax_kind_ext::CLASS_DECLARATION => {
                self.emit_class_declaration(node, idx);
            }

            // Property assignment
            k if k == syntax_kind_ext::PROPERTY_ASSIGNMENT => {
                self.emit_property_assignment(node);
            }

            // Shorthand property assignment
            k if k == syntax_kind_ext::SHORTHAND_PROPERTY_ASSIGNMENT => {
                self.emit_shorthand_property(node);
            }

            // Parameter declaration
            k if k == syntax_kind_ext::PARAMETER => {
                self.emit_parameter(node);
            }

            // Type keywords (for type annotations)
            k if k == SyntaxKind::NumberKeyword as u16 => self.write("number"),
            k if k == SyntaxKind::StringKeyword as u16 => self.write("string"),
            k if k == SyntaxKind::BooleanKeyword as u16 => self.write("boolean"),
            k if k == SyntaxKind::VoidKeyword as u16 => self.write("void"),
            k if k == SyntaxKind::AnyKeyword as u16 => self.write("any"),
            k if k == SyntaxKind::NeverKeyword as u16 => self.write("never"),
            k if k == SyntaxKind::UnknownKeyword as u16 => self.write("unknown"),
            k if k == SyntaxKind::UndefinedKeyword as u16 => self.write("undefined"),
            k if k == SyntaxKind::ObjectKeyword as u16 => self.write("object"),
            k if k == SyntaxKind::SymbolKeyword as u16 => self.write("symbol"),
            k if k == SyntaxKind::BigIntKeyword as u16 => self.write("bigint"),

            // Type reference
            k if k == syntax_kind_ext::TYPE_REFERENCE => {
                self.emit_type_reference(node);
            }

            // Array type
            k if k == syntax_kind_ext::ARRAY_TYPE => {
                self.emit_array_type(node);
            }

            // Union type
            k if k == syntax_kind_ext::UNION_TYPE => {
                self.emit_union_type(node);
            }

            // Intersection type
            k if k == syntax_kind_ext::INTERSECTION_TYPE => {
                self.emit_intersection_type(node);
            }

            // Tuple type
            k if k == syntax_kind_ext::TUPLE_TYPE => {
                self.emit_tuple_type(node);
            }

            // Function type
            k if k == syntax_kind_ext::FUNCTION_TYPE => {
                self.emit_function_type(node);
            }

            // Type literal
            k if k == syntax_kind_ext::TYPE_LITERAL => {
                self.emit_type_literal(node);
            }

            // Parenthesized type
            k if k == syntax_kind_ext::PARENTHESIZED_TYPE => {
                self.emit_parenthesized_type(node);
            }

            // Empty statement
            k if k == syntax_kind_ext::EMPTY_STATEMENT => {
                self.write_semicolon();
            }

            // JSX
            k if k == syntax_kind_ext::JSX_ELEMENT => {
                self.emit_jsx_element(node);
            }
            k if k == syntax_kind_ext::JSX_SELF_CLOSING_ELEMENT => {
                self.emit_jsx_self_closing_element(node);
            }
            k if k == syntax_kind_ext::JSX_OPENING_ELEMENT => {
                self.emit_jsx_opening_element(node);
            }
            k if k == syntax_kind_ext::JSX_CLOSING_ELEMENT => {
                self.emit_jsx_closing_element(node);
            }
            k if k == syntax_kind_ext::JSX_FRAGMENT => {
                self.emit_jsx_fragment(node);
            }
            k if k == syntax_kind_ext::JSX_OPENING_FRAGMENT => {
                self.write("<>");
            }
            k if k == syntax_kind_ext::JSX_CLOSING_FRAGMENT => {
                self.write("</>");
            }
            k if k == syntax_kind_ext::JSX_ATTRIBUTES => {
                self.emit_jsx_attributes(node);
            }
            k if k == syntax_kind_ext::JSX_ATTRIBUTE => {
                self.emit_jsx_attribute(node);
            }
            k if k == syntax_kind_ext::JSX_SPREAD_ATTRIBUTE => {
                self.emit_jsx_spread_attribute(node);
            }
            k if k == syntax_kind_ext::JSX_EXPRESSION => {
                self.emit_jsx_expression(node);
            }
            k if k == SyntaxKind::JsxText as u16 => {
                self.emit_jsx_text(node);
            }
            k if k == syntax_kind_ext::JSX_NAMESPACED_NAME => {
                self.emit_jsx_namespaced_name(node);
            }

            // Imports/Exports
            k if k == syntax_kind_ext::IMPORT_DECLARATION => {
                self.emit_import_declaration(node);
            }
            k if k == syntax_kind_ext::IMPORT_CLAUSE => {
                self.emit_import_clause(node);
            }
            k if k == syntax_kind_ext::NAMED_IMPORTS => {
                self.emit_named_imports(node);
            }
            k if k == syntax_kind_ext::IMPORT_SPECIFIER => {
                self.emit_import_specifier(node);
            }
            k if k == syntax_kind_ext::EXPORT_DECLARATION => {
                self.emit_export_declaration(node);
            }
            k if k == syntax_kind_ext::NAMED_EXPORTS => {
                self.emit_named_exports(node);
            }
            k if k == syntax_kind_ext::EXPORT_SPECIFIER => {
                self.emit_export_specifier(node);
            }

            // Additional statements
            k if k == syntax_kind_ext::THROW_STATEMENT => {
                self.emit_throw_statement(node);
            }
            k if k == syntax_kind_ext::TRY_STATEMENT => {
                self.emit_try_statement(node);
            }
            k if k == syntax_kind_ext::CATCH_CLAUSE => {
                self.emit_catch_clause(node);
            }
            k if k == syntax_kind_ext::SWITCH_STATEMENT => {
                self.emit_switch_statement(node);
            }
            k if k == syntax_kind_ext::CASE_CLAUSE => {
                self.emit_case_clause(node);
            }
            k if k == syntax_kind_ext::DEFAULT_CLAUSE => {
                self.emit_default_clause(node);
            }
            k if k == syntax_kind_ext::BREAK_STATEMENT => {
                self.emit_break_statement();
            }
            k if k == syntax_kind_ext::CONTINUE_STATEMENT => {
                self.emit_continue_statement();
            }
            k if k == syntax_kind_ext::DO_STATEMENT => {
                self.emit_do_statement(node);
            }
            k if k == syntax_kind_ext::DEBUGGER_STATEMENT => {
                self.emit_debugger_statement();
            }

            // Declarations
            k if k == syntax_kind_ext::ENUM_DECLARATION => {
                self.emit_enum_declaration(node);
            }
            k if k == syntax_kind_ext::ENUM_MEMBER => {
                self.emit_enum_member(node);
            }
            k if k == syntax_kind_ext::INTERFACE_DECLARATION => {
                // Interface declarations are TypeScript-only - skip for JavaScript
                // self.emit_interface_declaration(node);
            }
            k if k == syntax_kind_ext::TYPE_ALIAS_DECLARATION => {
                // Type alias declarations are TypeScript-only - skip for JavaScript
                // self.emit_type_alias_declaration(node);
            }
            k if k == syntax_kind_ext::MODULE_DECLARATION => {
                self.emit_module_declaration(node);
            }

            // Class members
            k if k == syntax_kind_ext::METHOD_DECLARATION => {
                self.emit_method_declaration(node);
            }
            k if k == syntax_kind_ext::PROPERTY_DECLARATION => {
                self.emit_property_declaration(node);
            }
            k if k == syntax_kind_ext::CONSTRUCTOR => {
                self.emit_constructor_declaration(node);
            }
            k if k == syntax_kind_ext::GET_ACCESSOR => {
                self.emit_get_accessor(node);
            }
            k if k == syntax_kind_ext::SET_ACCESSOR => {
                self.emit_set_accessor(node);
            }
            k if k == syntax_kind_ext::DECORATOR => {
                self.emit_decorator(node);
            }

            // Interface/type members (signatures)
            k if k == syntax_kind_ext::PROPERTY_SIGNATURE => {
                self.emit_property_signature(node);
            }
            k if k == syntax_kind_ext::METHOD_SIGNATURE => {
                self.emit_method_signature(node);
            }
            k if k == syntax_kind_ext::CALL_SIGNATURE => {
                self.emit_call_signature(node);
            }
            k if k == syntax_kind_ext::CONSTRUCT_SIGNATURE => {
                self.emit_construct_signature(node);
            }
            k if k == syntax_kind_ext::INDEX_SIGNATURE => {
                self.emit_index_signature(node);
            }

            // Template literals
            k if k == syntax_kind_ext::TEMPLATE_EXPRESSION => {
                self.emit_template_expression(node);
            }
            k if k == SyntaxKind::NoSubstitutionTemplateLiteral as u16 => {
                self.emit_no_substitution_template(node);
            }
            k if k == syntax_kind_ext::TEMPLATE_SPAN => {
                self.emit_template_span(node);
            }
            k if k == SyntaxKind::TemplateHead as u16 => {
                self.emit_template_head(node);
            }
            k if k == SyntaxKind::TemplateMiddle as u16 => {
                self.emit_template_middle(node);
            }
            k if k == SyntaxKind::TemplateTail as u16 => {
                self.emit_template_tail(node);
            }

            // Yield/Await/Spread
            k if k == syntax_kind_ext::YIELD_EXPRESSION => {
                self.emit_yield_expression(node);
            }
            k if k == syntax_kind_ext::AWAIT_EXPRESSION => {
                self.emit_await_expression(node);
            }
            k if k == syntax_kind_ext::SPREAD_ELEMENT => {
                self.emit_spread_element(node);
            }

            // Source file
            k if k == syntax_kind_ext::SOURCE_FILE => {
                self.emit_source_file(node);
            }

            // Other tokens and keywords - emit their text
            k if k == SyntaxKind::ThisKeyword as u16 => self.write("this"),
            k if k == SyntaxKind::SuperKeyword as u16 => self.write("super"),

            // Default: do nothing (or handle other cases as needed)
            _ => {}
        }
    }

    // =========================================================================
    // Literals
    // =========================================================================

    fn emit_identifier(&mut self, node: &ThinNode) {
        if let Some(ident) = self.arena.get_identifier(node) {
            self.write(&ident.escaped_text);
        }
    }

    fn emit_numeric_literal(&mut self, node: &ThinNode) {
        if let Some(lit) = self.arena.get_literal(node) {
            self.write(&lit.text);
        }
    }

    fn emit_string_literal(&mut self, node: &ThinNode) {
        if let Some(lit) = self.arena.get_literal(node) {
            let quote = if self.options.single_quote { '\'' } else { '"' };
            self.write_char(quote);
            self.emit_escaped_string(&lit.text, quote);
            self.write_char(quote);
        }
    }

    fn emit_escaped_string(&mut self, s: &str, quote_char: char) {
        for ch in s.chars() {
            match ch {
                '\n' => self.write("\\n"),
                '\r' => self.write("\\r"),
                '\t' => self.write("\\t"),
                '\\' => self.write("\\\\"),
                c if c == quote_char => {
                    self.write_char('\\');
                    self.write_char(c);
                }
                c => self.write_char(c),
            }
        }
    }

    // =========================================================================
    // Expressions
    // =========================================================================

    fn emit_binary_expression(&mut self, node: &ThinNode) {
        let Some(binary) = self.arena.get_binary_expr(node) else {
            return;
        };

        self.emit(binary.left);
        self.write_space();
        self.write(get_operator_text(binary.operator_token));
        self.write_space();
        self.emit(binary.right);
    }

    fn emit_prefix_unary(&mut self, node: &ThinNode) {
        let Some(unary) = self.arena.get_unary_expr(node) else {
            return;
        };

        self.write(get_operator_text(unary.operator));
        self.emit(unary.operand);
    }

    fn emit_postfix_unary(&mut self, node: &ThinNode) {
        let Some(unary) = self.arena.get_unary_expr(node) else {
            return;
        };

        self.emit(unary.operand);
        self.write(get_operator_text(unary.operator));
    }

    fn emit_call_expression(&mut self, node: &ThinNode) {
        let Some(call) = self.arena.get_call_expr(node) else {
            return;
        };

        self.emit(call.expression);
        self.write("(");
        if let Some(ref args) = call.arguments {
            self.emit_comma_separated(&args.nodes);
        }
        self.write(")");
    }

    fn emit_new_expression(&mut self, node: &ThinNode) {
        let Some(call) = self.arena.get_call_expr(node) else {
            return;
        };

        self.write("new ");
        self.emit(call.expression);
        self.write("(");
        if let Some(ref args) = call.arguments {
            self.emit_comma_separated(&args.nodes);
        }
        self.write(")");
    }

    fn emit_property_access(&mut self, node: &ThinNode) {
        let Some(access) = self.arena.get_access_expr(node) else {
            return;
        };

        self.emit(access.expression);
        self.write(".");
        self.emit(access.name_or_argument);
    }

    fn emit_element_access(&mut self, node: &ThinNode) {
        let Some(access) = self.arena.get_access_expr(node) else {
            return;
        };

        self.emit(access.expression);
        self.write("[");
        self.emit(access.name_or_argument);
        self.write("]");
    }

    fn emit_parenthesized(&mut self, node: &ThinNode) {
        let Some(paren) = self.arena.get_parenthesized(node) else {
            return;
        };

        self.write("(");
        self.emit(paren.expression);
        self.write(")");
    }

    fn emit_conditional(&mut self, node: &ThinNode) {
        let Some(cond) = self.arena.get_conditional_expr(node) else {
            return;
        };

        self.emit(cond.condition);
        self.write(" ? ");
        self.emit(cond.when_true);
        self.write(" : ");
        self.emit(cond.when_false);
    }

    fn emit_array_literal(&mut self, node: &ThinNode) {
        let Some(array) = self.arena.get_literal_expr(node) else {
            return;
        };

        self.write("[");
        self.emit_comma_separated(&array.elements.nodes);
        self.write("]");
    }

    fn emit_object_literal(&mut self, node: &ThinNode) {
        let Some(obj) = self.arena.get_literal_expr(node) else {
            return;
        };

        if obj.elements.nodes.is_empty() {
            self.write("{}");
            return;
        }

        // Multi-line format for object literals with multiple properties
        if obj.elements.nodes.len() > 1 {
            self.write("{");
            self.write_line();
            self.increase_indent();
            for (i, &prop) in obj.elements.nodes.iter().enumerate() {
                self.emit(prop);
                if i < obj.elements.nodes.len() - 1 {
                    self.write(",");
                }
                self.write_line();
            }
            self.decrease_indent();
            self.write("}");
        } else {
            // Single property: { key: value }
            self.write("{ ");
            self.emit(obj.elements.nodes[0]);
            self.write(" }");
        }
    }

    fn emit_property_assignment(&mut self, node: &ThinNode) {
        let Some(prop) = self.arena.get_property_assignment(node) else {
            return;
        };

        self.emit(prop.name);
        self.write(": ");
        self.emit(prop.initializer);
    }

    fn emit_shorthand_property(&mut self, node: &ThinNode) {
        let Some(shorthand) = self.arena.get_shorthand_property(node) else {
            // Fallback: try to get identifier data directly
            if let Some(ident) = self.arena.get_identifier(node) {
                self.write(&ident.escaped_text);
            }
            return;
        };

        self.emit(shorthand.name);
        if shorthand.equals_token {
            self.write(" = ");
            // Object assignment pattern default value would go here
        }
    }

    // =========================================================================
    // Functions
    // =========================================================================

    fn emit_arrow_function(&mut self, node: &ThinNode, _idx: NodeIndex) {
        let Some(func) = self.arena.get_function(node) else {
            return;
        };

        if func.is_async {
            self.write("async ");
        }

        // Parameters (without types for JavaScript)
        self.write("(");
        self.emit_function_parameters_js(&func.parameters.nodes);
        self.write(")");

        // Skip return type for JavaScript

        self.write(" => ");

        // Body
        self.emit(func.body);
    }

    fn emit_function_expression(&mut self, node: &ThinNode, _idx: NodeIndex) {
        let Some(func) = self.arena.get_function(node) else {
            return;
        };

        if func.is_async {
            self.write("async ");
        }

        self.write("function");

        if func.asterisk_token {
            self.write("*");
        }

        // Name (if any)
        if !func.name.is_none() {
            self.write_space();
            self.emit(func.name);
        }

        // Parameters (without types for JavaScript)
        self.write("(");
        self.emit_function_parameters_js(&func.parameters.nodes);
        self.write(")");

        // Skip return type for JavaScript

        self.write_space();
        self.emit(func.body);
    }

    fn emit_function_declaration(&mut self, node: &ThinNode, _idx: NodeIndex) {
        let Some(func) = self.arena.get_function(node) else {
            return;
        };

        // For JavaScript emit: skip declaration-only functions (no body)
        // These are just type information in TypeScript
        if func.body.is_none() {
            return;
        }

        if func.is_async {
            self.write("async ");
        }

        self.write("function");

        if func.asterisk_token {
            self.write("*");
        }

        // Name
        if !func.name.is_none() {
            self.write_space();
            self.emit(func.name);
        }

        // Parameters - only emit names, not types for JavaScript
        self.write("(");
        self.emit_function_parameters_js(&func.parameters.nodes);
        self.write(")");

        // No return type for JavaScript

        self.write_space();
        self.emit(func.body);
    }

    /// Emit function parameters for JavaScript (no types)
    fn emit_function_parameters_js(&mut self, params: &[NodeIndex]) {
        let mut first = true;
        for &param_idx in params {
            if !first {
                self.write(", ");
            }
            first = false;

            if let Some(param_node) = self.arena.get(param_idx) {
                if let Some(param) = self.arena.get_parameter(param_node) {
                    if param.dot_dot_dot_token {
                        self.write("...");
                    }
                    self.emit(param.name);
                    // Skip type annotations and defaults for JS emit
                    if !param.initializer.is_none() {
                        self.write(" = ");
                        self.emit(param.initializer);
                    }
                }
            }
        }
    }

    fn emit_parameter(&mut self, node: &ThinNode) {
        let Some(param) = self.arena.get_parameter(node) else {
            return;
        };

        if param.dot_dot_dot_token {
            self.write("...");
        }

        self.emit(param.name);

        if param.question_token {
            self.write("?");
        }

        if !param.type_annotation.is_none() {
            self.write(": ");
            self.emit(param.type_annotation);
        }

        if !param.initializer.is_none() {
            self.write(" = ");
            self.emit(param.initializer);
        }
    }

    // =========================================================================
    // Statements
    // =========================================================================

    fn emit_block(&mut self, node: &ThinNode) {
        let Some(block) = self.arena.get_block(node) else {
            return;
        };

        self.write("{");
        self.write_line();
        self.increase_indent();

        for &stmt_idx in &block.statements.nodes {
            self.emit(stmt_idx);
            self.write_line();
        }

        self.decrease_indent();
        self.write("}");
    }

    fn emit_variable_statement(&mut self, node: &ThinNode) {
        let Some(var_stmt) = self.arena.get_variable(node) else {
            return;
        };

        // VariableStatement.declarations contains a VARIABLE_DECLARATION_LIST
        // Emit the declaration list (which handles the let/const/var keyword)
        for &decl_list_idx in &var_stmt.declarations.nodes {
            self.emit(decl_list_idx);
        }
        self.write_semicolon();
    }

    fn emit_variable_declaration_list(&mut self, node: &ThinNode) {
        // Variable declaration list is stored as VariableData
        let Some(decl_list) = self.arena.get_variable(node) else {
            return;
        };

        // Emit keyword based on node flags
        let flags = node.flags as u32;
        let keyword = if flags & crate::parser::node_flags::CONST != 0 {
            "const"
        } else if flags & crate::parser::node_flags::LET != 0 {
            "let"
        } else {
            "var"
        };
        self.write(keyword);
        self.write(" ");

        self.emit_comma_separated(&decl_list.declarations.nodes);
    }

    fn emit_variable_declaration(&mut self, node: &ThinNode) {
        let Some(decl) = self.arena.get_variable_declaration(node) else {
            return;
        };

        self.emit(decl.name);

        // Skip type annotation for JavaScript emit

        if !decl.initializer.is_none() {
            self.write(" = ");
            self.emit(decl.initializer);
        }
    }

    fn emit_expression_statement(&mut self, node: &ThinNode) {
        let Some(expr_stmt) = self.arena.get_expression_statement(node) else {
            return;
        };

        self.emit(expr_stmt.expression);
        self.write_semicolon();
    }

    fn emit_if_statement(&mut self, node: &ThinNode) {
        let Some(if_stmt) = self.arena.get_if_statement(node) else {
            return;
        };

        self.write("if (");
        self.emit(if_stmt.expression);
        self.write(") ");
        self.emit(if_stmt.then_statement);

        if !if_stmt.else_statement.is_none() {
            self.write(" else ");
            self.emit(if_stmt.else_statement);
        }
    }

    fn emit_while_statement(&mut self, node: &ThinNode) {
        let Some(loop_stmt) = self.arena.get_loop(node) else {
            return;
        };

        self.write("while (");
        self.emit(loop_stmt.condition);
        self.write(") ");
        self.emit(loop_stmt.statement);
    }

    fn emit_for_statement(&mut self, node: &ThinNode) {
        let Some(loop_stmt) = self.arena.get_loop(node) else {
            return;
        };

        self.write("for (");
        self.emit(loop_stmt.initializer);
        self.write("; ");
        self.emit(loop_stmt.condition);
        self.write("; ");
        self.emit(loop_stmt.incrementor);
        self.write(") ");
        self.emit(loop_stmt.statement);
    }

    fn emit_for_in_statement(&mut self, node: &ThinNode) {
        let Some(for_in_of) = self.arena.get_for_in_of(node) else {
            return;
        };

        self.write("for (");
        self.emit(for_in_of.initializer);
        self.write(" in ");
        self.emit(for_in_of.expression);
        self.write(") ");
        self.emit(for_in_of.statement);
    }

    fn emit_for_of_statement(&mut self, node: &ThinNode) {
        let Some(for_in_of) = self.arena.get_for_in_of(node) else {
            return;
        };

        self.write("for ");
        if for_in_of.await_modifier {
            self.write("await ");
        }
        self.write("(");
        self.emit(for_in_of.initializer);
        self.write(" of ");
        self.emit(for_in_of.expression);
        self.write(") ");
        self.emit(for_in_of.statement);
    }

    fn emit_return_statement(&mut self, node: &ThinNode) {
        let Some(ret) = self.arena.get_return_statement(node) else {
            self.write("return");
            self.write_semicolon();
            return;
        };

        self.write("return");
        if !ret.expression.is_none() {
            self.write(" ");
            self.emit(ret.expression);
        }
        self.write_semicolon();
    }

    // =========================================================================
    // Classes
    // =========================================================================

    fn emit_class_declaration(&mut self, node: &ThinNode, _idx: NodeIndex) {
        let Some(class) = self.arena.get_class(node) else {
            return;
        };

        // Emit modifiers (including decorators)
        if let Some(ref modifiers) = class.modifiers {
            for &mod_idx in &modifiers.nodes {
                self.emit(mod_idx);
                // Add space or newline after decorator
                if let Some(mod_node) = self.arena.get(mod_idx) {
                    if mod_node.kind == syntax_kind_ext::DECORATOR {
                        self.write_line();
                    } else {
                        self.write_space();
                    }
                }
            }
        }

        self.write("class");

        if !class.name.is_none() {
            self.write_space();
            self.emit(class.name);
        }

        // TODO: Emit heritage clause

        self.write(" {");
        self.write_line();
        self.increase_indent();

        for &member_idx in &class.members.nodes {
            self.emit(member_idx);
            self.write_line();
        }

        self.decrease_indent();
        self.write("}");
    }

    // =========================================================================
    // Types
    // =========================================================================

    fn emit_type_reference(&mut self, node: &ThinNode) {
        let Some(type_ref) = self.arena.get_type_ref(node) else {
            return;
        };

        self.emit(type_ref.type_name);

        if let Some(ref type_args) = type_ref.type_arguments {
            if !type_args.nodes.is_empty() {
                self.write("<");
                self.emit_comma_separated(&type_args.nodes);
                self.write(">");
            }
        }
    }

    // =========================================================================
    // Helpers
    // =========================================================================

    fn emit_comma_separated(&mut self, nodes: &[NodeIndex]) {
        let mut first = true;
        for &idx in nodes {
            if !first {
                self.write(", ");
            }
            first = false;
            self.emit(idx);
        }
    }

    // =========================================================================
    // JSX
    // =========================================================================

    fn emit_jsx_element(&mut self, node: &ThinNode) {
        let Some(jsx) = self.arena.get_jsx_element(node) else {
            return;
        };

        self.emit(jsx.opening_element);
        for &child in &jsx.children.nodes {
            self.emit(child);
        }
        self.emit(jsx.closing_element);
    }

    fn emit_jsx_self_closing_element(&mut self, node: &ThinNode) {
        let Some(jsx) = self.arena.get_jsx_opening(node) else {
            return;
        };

        self.write("<");
        self.emit(jsx.tag_name);
        self.emit(jsx.attributes);
        self.write(" />");
    }

    fn emit_jsx_opening_element(&mut self, node: &ThinNode) {
        let Some(jsx) = self.arena.get_jsx_opening(node) else {
            return;
        };

        self.write("<");
        self.emit(jsx.tag_name);
        self.emit(jsx.attributes);
        self.write(">");
    }

    fn emit_jsx_closing_element(&mut self, node: &ThinNode) {
        let Some(jsx) = self.arena.get_jsx_closing(node) else {
            return;
        };

        self.write("</");
        self.emit(jsx.tag_name);
        self.write(">");
    }

    fn emit_jsx_fragment(&mut self, node: &ThinNode) {
        let Some(jsx) = self.arena.get_jsx_fragment(node) else {
            return;
        };

        self.write("<>");
        for &child in &jsx.children.nodes {
            self.emit(child);
        }
        self.write("</>");
    }

    fn emit_jsx_attributes(&mut self, node: &ThinNode) {
        let Some(attrs) = self.arena.get_jsx_attributes(node) else {
            return;
        };

        for &attr in &attrs.properties.nodes {
            self.write_space();
            self.emit(attr);
        }
    }

    fn emit_jsx_attribute(&mut self, node: &ThinNode) {
        let Some(attr) = self.arena.get_jsx_attribute(node) else {
            return;
        };

        self.emit(attr.name);
        if !attr.initializer.is_none() {
            self.write("=");
            self.emit(attr.initializer);
        }
    }

    fn emit_jsx_spread_attribute(&mut self, node: &ThinNode) {
        let Some(spread) = self.arena.get_jsx_spread_attribute(node) else {
            return;
        };

        self.write("{...");
        self.emit(spread.expression);
        self.write("}");
    }

    fn emit_jsx_expression(&mut self, node: &ThinNode) {
        let Some(expr) = self.arena.get_jsx_expression(node) else {
            return;
        };

        self.write("{");
        if expr.dot_dot_dot_token {
            self.write("...");
        }
        self.emit(expr.expression);
        self.write("}");
    }

    fn emit_jsx_text(&mut self, node: &ThinNode) {
        let Some(text) = self.arena.get_jsx_text(node) else {
            return;
        };

        self.write(&text.text);
    }

    fn emit_jsx_namespaced_name(&mut self, node: &ThinNode) {
        let Some(ns) = self.arena.get_jsx_namespaced_name(node) else {
            return;
        };

        self.emit(ns.namespace);
        self.write(":");
        self.emit(ns.name);
    }

    // =========================================================================
    // Imports/Exports
    // =========================================================================

    fn emit_import_declaration(&mut self, node: &ThinNode) {
        let Some(import) = self.arena.get_import_decl(node) else {
            return;
        };

        self.write("import ");

        if !import.import_clause.is_none() {
            self.emit(import.import_clause);
            self.write(" from ");
        }

        self.emit(import.module_specifier);
        self.write_semicolon();
    }

    fn emit_import_clause(&mut self, node: &ThinNode) {
        let Some(clause) = self.arena.get_import_clause(node) else {
            return;
        };

        let mut has_default = false;

        // Default import
        if !clause.name.is_none() {
            self.emit(clause.name);
            has_default = true;
        }

        // Named bindings
        if !clause.named_bindings.is_none() {
            if has_default {
                self.write(", ");
            }
            self.emit(clause.named_bindings);
        }
    }

    fn emit_named_imports(&mut self, node: &ThinNode) {
        let Some(imports) = self.arena.get_named_imports(node) else {
            return;
        };

        self.write("{ ");
        self.emit_comma_separated(&imports.elements.nodes);
        self.write(" }");
    }

    fn emit_import_specifier(&mut self, node: &ThinNode) {
        let Some(spec) = self.arena.get_specifier(node) else {
            return;
        };

        if !spec.property_name.is_none() {
            self.emit(spec.property_name);
            self.write(" as ");
        }
        self.emit(spec.name);
    }

    fn emit_export_declaration(&mut self, node: &ThinNode) {
        let Some(export) = self.arena.get_export_decl(node) else {
            return;
        };

        self.write("export ");

        if !export.export_clause.is_none() {
            self.emit(export.export_clause);
        } else {
            self.write("*");
        }

        if !export.module_specifier.is_none() {
            self.write(" from ");
            self.emit(export.module_specifier);
        }

        self.write_semicolon();
    }

    fn emit_named_exports(&mut self, node: &ThinNode) {
        // Named exports uses the same data structure as named imports
        let Some(exports) = self.arena.get_named_imports(node) else {
            self.write("{ }");
            return;
        };

        self.write("{ ");
        self.emit_comma_separated(&exports.elements.nodes);
        self.write(" }");
    }

    fn emit_export_specifier(&mut self, node: &ThinNode) {
        let Some(spec) = self.arena.get_specifier(node) else {
            return;
        };

        if !spec.property_name.is_none() {
            self.emit(spec.property_name);
            self.write(" as ");
        }
        self.emit(spec.name);
    }

    // =========================================================================
    // Additional Statements
    // =========================================================================

    fn emit_throw_statement(&mut self, node: &ThinNode) {
        // ThrowStatement uses ReturnData (same structure)
        let Some(throw_data) = self.arena.get_return_statement(node) else {
            self.write("throw");
            self.write_semicolon();
            return;
        };

        self.write("throw ");
        self.emit(throw_data.expression);
        self.write_semicolon();
    }

    fn emit_try_statement(&mut self, node: &ThinNode) {
        let Some(try_stmt) = self.arena.get_try(node) else {
            return;
        };

        self.write("try ");
        self.emit(try_stmt.try_block);

        if !try_stmt.catch_clause.is_none() {
            self.write(" ");
            self.emit(try_stmt.catch_clause);
        }

        if !try_stmt.finally_block.is_none() {
            self.write(" finally ");
            self.emit(try_stmt.finally_block);
        }
    }

    fn emit_catch_clause(&mut self, node: &ThinNode) {
        let Some(catch) = self.arena.get_catch_clause(node) else {
            return;
        };

        self.write("catch");

        if !catch.variable_declaration.is_none() {
            self.write(" (");
            self.emit(catch.variable_declaration);
            self.write(")");
        }

        self.write(" ");
        self.emit(catch.block);
    }

    fn emit_switch_statement(&mut self, node: &ThinNode) {
        let Some(switch) = self.arena.get_switch(node) else {
            return;
        };

        self.write("switch (");
        self.emit(switch.expression);
        self.write(") ");
        // case_block is a NodeIndex pointing to a CaseBlock node
        self.emit(switch.case_block);
    }

    fn emit_case_clause(&mut self, node: &ThinNode) {
        let Some(clause) = self.arena.get_case_clause(node) else {
            return;
        };

        self.write("case ");
        self.emit(clause.expression);
        self.write(":");
        self.write_line();
        self.increase_indent();

        for &stmt in &clause.statements.nodes {
            self.emit(stmt);
            self.write_line();
        }

        self.decrease_indent();
    }

    fn emit_default_clause(&mut self, node: &ThinNode) {
        let Some(clause) = self.arena.get_case_clause(node) else {
            return;
        };

        self.write("default:");
        self.write_line();
        self.increase_indent();

        for &stmt in &clause.statements.nodes {
            self.emit(stmt);
            self.write_line();
        }

        self.decrease_indent();
    }

    fn emit_break_statement(&mut self) {
        self.write("break");
        self.write_semicolon();
    }

    fn emit_continue_statement(&mut self) {
        self.write("continue");
        self.write_semicolon();
    }

    fn emit_do_statement(&mut self, node: &ThinNode) {
        let Some(loop_stmt) = self.arena.get_loop(node) else {
            return;
        };

        self.write("do ");
        self.emit(loop_stmt.statement);
        self.write(" while (");
        self.emit(loop_stmt.condition);
        self.write(")");
        self.write_semicolon();
    }

    fn emit_debugger_statement(&mut self) {
        self.write("debugger");
        self.write_semicolon();
    }

    // =========================================================================
    // Type Emit Methods
    // =========================================================================

    fn emit_union_type(&mut self, node: &ThinNode) {
        let Some(union) = self.arena.get_composite_type(node) else {
            return;
        };

        let mut first = true;
        for &type_idx in &union.types.nodes {
            if !first {
                self.write(" | ");
            }
            first = false;
            self.emit(type_idx);
        }
    }

    fn emit_intersection_type(&mut self, node: &ThinNode) {
        let Some(intersection) = self.arena.get_composite_type(node) else {
            return;
        };

        let mut first = true;
        for &type_idx in &intersection.types.nodes {
            if !first {
                self.write(" & ");
            }
            first = false;
            self.emit(type_idx);
        }
    }

    fn emit_array_type(&mut self, node: &ThinNode) {
        let Some(array) = self.arena.get_array_type(node) else {
            return;
        };

        self.emit(array.element_type);
        self.write("[]");
    }

    fn emit_tuple_type(&mut self, node: &ThinNode) {
        let Some(tuple) = self.arena.get_tuple_type(node) else {
            self.write("[]");
            return;
        };

        self.write("[");
        self.emit_comma_separated(&tuple.elements.nodes);
        self.write("]");
    }

    fn emit_function_type(&mut self, node: &ThinNode) {
        let Some(func_type) = self.arena.get_function_type(node) else {
            return;
        };

        // Type parameters
        if let Some(ref type_params) = func_type.type_parameters {
            if !type_params.nodes.is_empty() {
                self.write("<");
                self.emit_comma_separated(&type_params.nodes);
                self.write(">");
            }
        }

        // Parameters
        self.write("(");
        self.emit_comma_separated(&func_type.parameters.nodes);
        self.write(") => ");

        // Return type
        self.emit(func_type.type_annotation);
    }

    fn emit_type_literal(&mut self, node: &ThinNode) {
        let Some(type_lit) = self.arena.get_type_literal(node) else {
            self.write("{}");
            return;
        };

        if type_lit.members.nodes.is_empty() {
            self.write("{}");
            return;
        }

        self.write("{");
        self.write_line();
        self.increase_indent();

        for &member_idx in &type_lit.members.nodes {
            self.emit(member_idx);
            self.write_semicolon();
            self.write_line();
        }

        self.decrease_indent();
        self.write("}");
    }

    fn emit_parenthesized_type(&mut self, node: &ThinNode) {
        let Some(paren_type) = self.arena.get_wrapped_type(node) else {
            return;
        };

        self.write("(");
        self.emit(paren_type.type_node);
        self.write(")");
    }

    // =========================================================================
    // Declarations - Enum, Interface, Type Alias
    // =========================================================================

    fn emit_enum_declaration(&mut self, node: &ThinNode) {
        let Some(enum_decl) = self.arena.get_enum(node) else {
            return;
        };

        self.write("enum ");
        self.emit(enum_decl.name);
        self.write(" {");
        self.write_line();
        self.increase_indent();

        for &member_idx in &enum_decl.members.nodes {
            self.emit(member_idx);
            self.write(",");
            self.write_line();
        }

        self.decrease_indent();
        self.write("}");
    }

    fn emit_enum_member(&mut self, node: &ThinNode) {
        let Some(member) = self.arena.get_enum_member(node) else {
            return;
        };

        self.emit(member.name);

        if !member.initializer.is_none() {
            self.write(" = ");
            self.emit(member.initializer);
        }
    }

    fn emit_interface_declaration(&mut self, node: &ThinNode) {
        let Some(interface) = self.arena.get_interface(node) else {
            return;
        };

        self.write("interface ");
        self.emit(interface.name);

        // Type parameters
        if let Some(ref type_params) = interface.type_parameters {
            if !type_params.nodes.is_empty() {
                self.write("<");
                self.emit_comma_separated(&type_params.nodes);
                self.write(">");
            }
        }

        // Heritage clauses
        if let Some(ref heritage) = interface.heritage_clauses {
            if !heritage.nodes.is_empty() {
                self.write(" extends ");
                self.emit_comma_separated(&heritage.nodes);
            }
        }

        self.write(" {");
        self.write_line();
        self.increase_indent();

        for &member_idx in &interface.members.nodes {
            self.emit(member_idx);
            self.write_semicolon();
            self.write_line();
        }

        self.decrease_indent();
        self.write("}");
    }

    fn emit_type_alias_declaration(&mut self, node: &ThinNode) {
        let Some(type_alias) = self.arena.get_type_alias(node) else {
            return;
        };

        self.write("type ");
        self.emit(type_alias.name);

        // Type parameters
        if let Some(ref type_params) = type_alias.type_parameters {
            if !type_params.nodes.is_empty() {
                self.write("<");
                self.emit_comma_separated(&type_params.nodes);
                self.write(">");
            }
        }

        self.write(" = ");
        self.emit(type_alias.type_node);
        self.write_semicolon();
    }

    fn emit_module_declaration(&mut self, node: &ThinNode) {
        let Some(module) = self.arena.get_module(node) else {
            return;
        };

        self.write("namespace ");
        self.emit(module.name);
        self.write(" ");
        self.emit(module.body);
    }

    // =========================================================================
    // Template Literals
    // =========================================================================

    fn emit_template_expression(&mut self, node: &ThinNode) {
        let Some(tpl) = self.arena.get_template_expr(node) else {
            self.write("``");
            return;
        };

        // Emit the template head (opening backtick and initial text)
        self.emit(tpl.head);

        // Emit each template span (expression + middle/tail)
        for &span_idx in &tpl.template_spans.nodes {
            self.emit(span_idx);
        }
    }

    fn emit_no_substitution_template(&mut self, node: &ThinNode) {
        if let Some(lit) = self.arena.get_literal(node) {
            self.write("`");
            self.write(&lit.text);
            self.write("`");
        }
    }

    fn emit_template_span(&mut self, node: &ThinNode) {
        let Some(span) = self.arena.get_template_span(node) else {
            return;
        };

        // Emit ${expression}
        self.write("${");
        self.emit(span.expression);
        self.write("}");
        // Emit the literal part (middle or tail)
        self.emit(span.literal);
    }

    fn emit_template_head(&mut self, node: &ThinNode) {
        if let Some(lit) = self.arena.get_literal(node) {
            // Template head starts with ` and ends with ${
            self.write("`");
            self.write(&lit.text);
        }
    }

    fn emit_template_middle(&mut self, node: &ThinNode) {
        if let Some(lit) = self.arena.get_literal(node) {
            // Template middle is between } and ${
            self.write(&lit.text);
        }
    }

    fn emit_template_tail(&mut self, node: &ThinNode) {
        if let Some(lit) = self.arena.get_literal(node) {
            // Template tail ends with `
            self.write(&lit.text);
            self.write("`");
        }
    }

    // =========================================================================
    // Class Members
    // =========================================================================

    /// Emit class member modifiers (static, public, private, etc.)
    fn emit_class_member_modifiers(&mut self, modifiers: &Option<NodeList>) {
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.arena.get(mod_idx) {
                    // Emit the modifier keyword based on its kind
                    let keyword = match mod_node.kind as u32 {
                        k if k == SyntaxKind::StaticKeyword as u32 => "static",
                        k if k == SyntaxKind::PublicKeyword as u32 => "public",
                        k if k == SyntaxKind::PrivateKeyword as u32 => "private",
                        k if k == SyntaxKind::ProtectedKeyword as u32 => "protected",
                        k if k == SyntaxKind::ReadonlyKeyword as u32 => "readonly",
                        k if k == SyntaxKind::AbstractKeyword as u32 => "abstract",
                        k if k == SyntaxKind::OverrideKeyword as u32 => "override",
                        k if k == SyntaxKind::AsyncKeyword as u32 => "async",
                        k if k == SyntaxKind::DeclareKeyword as u32 => "declare",
                        _ => continue,
                    };
                    self.write(keyword);
                    self.write_space();
                }
            }
        }
    }

    fn emit_method_declaration(&mut self, node: &ThinNode) {
        let Some(method) = self.arena.get_method_decl(node) else {
            return;
        };

        // Skip method declarations without bodies (TypeScript-only overloads)
        if method.body.is_none() {
            return;
        }

        // Emit modifiers (static, async only for JavaScript)
        self.emit_method_modifiers_js(&method.modifiers);

        self.emit(method.name);
        self.write("(");
        self.emit_function_parameters_js(&method.parameters.nodes);
        self.write(")");

        // Skip return type for JavaScript emit

        self.write(" ");
        self.emit(method.body);
    }

    /// Emit method modifiers for JavaScript (static, async only)
    fn emit_method_modifiers_js(&mut self, modifiers: &Option<NodeList>) {
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.arena.get(mod_idx) {
                    match mod_node.kind {
                        k if k == SyntaxKind::StaticKeyword as u16 => self.write("static "),
                        k if k == SyntaxKind::AsyncKeyword as u16 => self.write("async "),
                        _ => {} // Skip private/protected/public/readonly/abstract
                    }
                }
            }
        }
    }

    fn emit_property_declaration(&mut self, node: &ThinNode) {
        let Some(prop) = self.arena.get_property_decl(node) else {
            return;
        };

        // For JavaScript: Skip property declarations that are TypeScript-only
        // (declarations with type annotation but no initializer)
        if prop.initializer.is_none() && !prop.type_annotation.is_none() {
            return;
        }

        // Emit modifiers (static only for JavaScript)
        self.emit_class_member_modifiers_js(&prop.modifiers);

        self.emit(prop.name);

        // Skip type annotations for JavaScript emit

        if !prop.initializer.is_none() {
            self.write(" = ");
            self.emit(prop.initializer);
        }

        self.write_semicolon();
    }

    /// Emit class member modifiers for JavaScript (only static is valid)
    fn emit_class_member_modifiers_js(&mut self, modifiers: &Option<NodeList>) {
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.arena.get(mod_idx) {
                    // Only emit 'static' for JavaScript - skip private/readonly/public/protected
                    if mod_node.kind == SyntaxKind::StaticKeyword as u16 {
                        self.write("static ");
                    }
                }
            }
        }
    }

    fn emit_constructor_declaration(&mut self, node: &ThinNode) {
        let Some(ctor) = self.arena.get_constructor(node) else {
            return;
        };

        // Emit modifiers (public, protected, private)
        self.emit_class_member_modifiers(&ctor.modifiers);

        self.write("constructor(");
        self.emit_comma_separated(&ctor.parameters.nodes);
        self.write(")");

        if !ctor.body.is_none() {
            self.write(" ");
            self.emit(ctor.body);
        }
    }

    fn emit_get_accessor(&mut self, node: &ThinNode) {
        let Some(accessor) = self.arena.get_accessor(node) else {
            return;
        };

        // Emit modifiers (static, private, etc.)
        self.emit_class_member_modifiers(&accessor.modifiers);

        self.write("get ");
        self.emit(accessor.name);
        self.write("()");

        if !accessor.type_annotation.is_none() {
            self.write(": ");
            self.emit(accessor.type_annotation);
        }

        if !accessor.body.is_none() {
            self.write(" ");
            self.emit(accessor.body);
        }
    }

    fn emit_set_accessor(&mut self, node: &ThinNode) {
        let Some(accessor) = self.arena.get_accessor(node) else {
            return;
        };

        // Emit modifiers (static, private, etc.)
        self.emit_class_member_modifiers(&accessor.modifiers);

        self.write("set ");
        self.emit(accessor.name);
        self.write("(");
        self.emit_comma_separated(&accessor.parameters.nodes);
        self.write(")");

        if !accessor.body.is_none() {
            self.write(" ");
            self.emit(accessor.body);
        }
    }

    // =========================================================================
    // Interface/Type Members (Signatures)
    // =========================================================================

    fn emit_property_signature(&mut self, node: &ThinNode) {
        let Some(sig) = self.arena.get_signature(node) else {
            return;
        };

        // Emit modifiers (readonly)
        self.emit_class_member_modifiers(&sig.modifiers);

        if !sig.name.is_none() {
            self.emit(sig.name);
        }

        if sig.question_token {
            self.write("?");
        }

        if !sig.type_annotation.is_none() {
            self.write(": ");
            self.emit(sig.type_annotation);
        }
    }

    fn emit_method_signature(&mut self, node: &ThinNode) {
        let Some(sig) = self.arena.get_signature(node) else {
            return;
        };

        if !sig.name.is_none() {
            self.emit(sig.name);
        }

        if sig.question_token {
            self.write("?");
        }

        self.write("(");
        if let Some(ref params) = sig.parameters {
            self.emit_comma_separated(&params.nodes);
        }
        self.write(")");

        if !sig.type_annotation.is_none() {
            self.write(": ");
            self.emit(sig.type_annotation);
        }
    }

    fn emit_call_signature(&mut self, node: &ThinNode) {
        let Some(sig) = self.arena.get_signature(node) else {
            return;
        };

        // TODO: type parameters

        self.write("(");
        if let Some(ref params) = sig.parameters {
            self.emit_comma_separated(&params.nodes);
        }
        self.write(")");

        if !sig.type_annotation.is_none() {
            self.write(": ");
            self.emit(sig.type_annotation);
        }
    }

    fn emit_construct_signature(&mut self, node: &ThinNode) {
        let Some(sig) = self.arena.get_signature(node) else {
            return;
        };

        self.write("new ");

        // TODO: type parameters

        self.write("(");
        if let Some(ref params) = sig.parameters {
            self.emit_comma_separated(&params.nodes);
        }
        self.write(")");

        if !sig.type_annotation.is_none() {
            self.write(": ");
            self.emit(sig.type_annotation);
        }
    }

    fn emit_index_signature(&mut self, node: &ThinNode) {
        let Some(sig) = self.arena.get_index_signature(node) else {
            return;
        };

        // Emit modifiers (readonly)
        self.emit_class_member_modifiers(&sig.modifiers);

        self.write("[");
        self.emit_comma_separated(&sig.parameters.nodes);
        self.write("]");

        if !sig.type_annotation.is_none() {
            self.write(": ");
            self.emit(sig.type_annotation);
        }
    }

    // =========================================================================
    // Yield and Await
    // =========================================================================

    fn emit_yield_expression(&mut self, node: &ThinNode) {
        // YieldExpression is stored with UnaryExprData (operand = expression, operator = asterisk flag)
        let Some(unary) = self.arena.get_unary_expr(node) else {
            self.write("yield");
            return;
        };

        self.write("yield");
        // Check if this is yield* (operator stores asterisk flag as SyntaxKind)
        if unary.operator == crate::scanner::SyntaxKind::AsteriskToken as u16 {
            self.write("*");
        }
        if !unary.operand.is_none() {
            self.write(" ");
            self.emit(unary.operand);
        }
    }

    fn emit_await_expression(&mut self, node: &ThinNode) {
        // AwaitExpression is stored with UnaryExprData
        let Some(unary) = self.arena.get_unary_expr(node) else {
            self.write("await");
            return;
        };

        self.write("await ");
        self.emit(unary.operand);
    }

    fn emit_spread_element(&mut self, node: &ThinNode) {
        let Some(spread) = self.arena.get_spread(node) else {
            self.write("...");
            return;
        };

        self.write("...");
        self.emit(spread.expression);
    }

    // =========================================================================
    // Decorators
    // =========================================================================

    fn emit_decorator(&mut self, node: &ThinNode) {
        let Some(decorator) = self.arena.get_decorator(node) else {
            return;
        };

        self.write("@");
        self.emit(decorator.expression);
    }

    // =========================================================================
    // Source File
    // =========================================================================

    fn emit_source_file(&mut self, node: &ThinNode) {
        let Some(source) = self.arena.get_source_file(node) else {
            return;
        };

        for &stmt_idx in &source.statements.nodes {
            self.emit(stmt_idx);
            self.write_line();
        }
    }
}

// =============================================================================
// Operator Text Helper
// =============================================================================

fn get_operator_text(op: u16) -> &'static str {
    match op {
        k if k == SyntaxKind::PlusToken as u16 => "+",
        k if k == SyntaxKind::MinusToken as u16 => "-",
        k if k == SyntaxKind::AsteriskToken as u16 => "*",
        k if k == SyntaxKind::SlashToken as u16 => "/",
        k if k == SyntaxKind::PercentToken as u16 => "%",
        k if k == SyntaxKind::AsteriskAsteriskToken as u16 => "**",
        k if k == SyntaxKind::PlusPlusToken as u16 => "++",
        k if k == SyntaxKind::MinusMinusToken as u16 => "--",
        k if k == SyntaxKind::LessThanToken as u16 => "<",
        k if k == SyntaxKind::GreaterThanToken as u16 => ">",
        k if k == SyntaxKind::LessThanEqualsToken as u16 => "<=",
        k if k == SyntaxKind::GreaterThanEqualsToken as u16 => ">=",
        k if k == SyntaxKind::EqualsEqualsToken as u16 => "==",
        k if k == SyntaxKind::ExclamationEqualsToken as u16 => "!=",
        k if k == SyntaxKind::EqualsEqualsEqualsToken as u16 => "===",
        k if k == SyntaxKind::ExclamationEqualsEqualsToken as u16 => "!==",
        k if k == SyntaxKind::EqualsToken as u16 => "=",
        k if k == SyntaxKind::PlusEqualsToken as u16 => "+=",
        k if k == SyntaxKind::MinusEqualsToken as u16 => "-=",
        k if k == SyntaxKind::AsteriskEqualsToken as u16 => "*=",
        k if k == SyntaxKind::SlashEqualsToken as u16 => "/=",
        k if k == SyntaxKind::PercentEqualsToken as u16 => "%=",
        k if k == SyntaxKind::AmpersandToken as u16 => "&",
        k if k == SyntaxKind::BarToken as u16 => "|",
        k if k == SyntaxKind::CaretToken as u16 => "^",
        k if k == SyntaxKind::TildeToken as u16 => "~",
        k if k == SyntaxKind::AmpersandAmpersandToken as u16 => "&&",
        k if k == SyntaxKind::BarBarToken as u16 => "||",
        k if k == SyntaxKind::ExclamationToken as u16 => "!",
        k if k == SyntaxKind::QuestionQuestionToken as u16 => "??",
        k if k == SyntaxKind::LessThanLessThanToken as u16 => "<<",
        k if k == SyntaxKind::GreaterThanGreaterThanToken as u16 => ">>",
        k if k == SyntaxKind::GreaterThanGreaterThanGreaterThanToken as u16 => ">>>",
        _ => "",
    }
}

