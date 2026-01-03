//! Emitter implementation for TypeScript AST.
//!
//! The emitter converts an AST back to source code (JavaScript or TypeScript).
//! This is Phase 6 of the Rust migration.

use crate::parser::{Node, NodeList};
use crate::parser::base::NodeBase;
use crate::scanner::SyntaxKind;

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
        }
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
        self.output.push_str(s);
    }

    /// Write indentation.
    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str(&self.indent_str);
        }
    }

    /// Write a new line.
    fn write_line(&mut self) {
        self.output.push_str(&self.new_line);
        self.at_line_start = true;
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

            // Declarations
            Node::FunctionDeclaration(decl) => self.emit_function_declaration(decl, arena),
            Node::ClassDeclaration(decl) => self.emit_class_declaration(decl, arena),
            Node::VariableDeclaration(decl) => self.emit_variable_declaration(decl, arena),
            Node::VariableDeclarationList(list) => self.emit_variable_declaration_list(list, arena),
            Node::ParameterDeclaration(param) => self.emit_parameter_declaration(param, arena),

            // Object literal members
            Node::PropertyAssignment(prop) => self.emit_property_assignment(prop, arena),
            Node::ShorthandPropertyAssignment(prop) => self.emit_shorthand_property(prop, arena),
            Node::SpreadAssignment(spread) => self.emit_spread_assignment(spread, arena),

            // Source file
            Node::SourceFile(sf) => self.emit_source_file(sf, arena),

            // Token (keywords, punctuation)
            Node::Token(base) => self.emit_token(base),

            // Default: emit nothing for unsupported nodes
            _ => {}
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
        self.write("class ");
        if let Some(name) = arena.get(decl.name) {
            self.emit_node(name, arena);
        }
        // TODO: heritage clauses
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
        // Safe because SyntaxKind is #[repr(u16)] and all values are valid tokens
        let kind: SyntaxKind = unsafe { std::mem::transmute(base.kind) };
        self.emit_token_kind(kind);
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
}
