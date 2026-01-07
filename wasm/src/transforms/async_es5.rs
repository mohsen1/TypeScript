//! ES5 Async Function Transform
//!
//! Transforms async functions to ES5 generators wrapped in __awaiter.
//!
//! # Transform Patterns
//!
//! ## Simple async function (no await)
//! ```typescript
//! async function foo(): Promise<void> { }
//! ```
//! Becomes:
//! ```javascript
//! function foo() {
//!     return __awaiter(this, void 0, void 0, function () {
//!         return __generator(this, function (_a) {
//!             return [2 /*return*/];
//!         });
//!     });
//! }
//! ```
//!
//! ## Async function with await
//! ```typescript
//! async function foo() {
//!     await bar();
//!     return 1;
//! }
//! ```
//! Becomes:
//! ```javascript
//! function foo() {
//!     return __awaiter(this, void 0, void 0, function () {
//!         return __generator(this, function (_a) {
//!             switch (_a.label) {
//!                 case 0: return [4 /*yield*/, bar()];
//!                 case 1:
//!                     _a.sent();
//!                     return [2 /*return*/, 1];
//!             }
//!         });
//!     });
//! }
//! ```
//!
//! ## Async arrow function
//! ```typescript
//! var foo = async () => { };
//! ```
//! Becomes:
//! ```javascript
//! var _this = this;
//! var foo = function () { return __awaiter(_this, void 0, void 0, function () {
//!     return __generator(this, function (_a) {
//!         return [2 /*return*/];
//!     });
//! }); };
//! ```

use crate::parser::thin_node::ThinNodeArena;
use crate::parser::{syntax_kind_ext, NodeIndex, NodeList};
use crate::scanner::SyntaxKind;

/// State for tracking async function transformation
#[derive(Debug, Default)]
pub struct AsyncTransformState {
    /// Current label counter for generator switch/case
    pub label_counter: u32,
    /// Whether we're currently inside an async function body
    pub in_async_body: bool,
    /// Whether any await expressions were found (determines if we need switch/case)
    pub has_await: bool,
}

impl AsyncTransformState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset for a new async function
    pub fn reset(&mut self) {
        self.label_counter = 0;
        self.in_async_body = false;
        self.has_await = false;
    }

    /// Get the next label number
    pub fn next_label(&mut self) -> u32 {
        let label = self.label_counter;
        self.label_counter += 1;
        label
    }
}

/// Async ES5 emitter for transforming async functions
pub struct AsyncES5Emitter<'a> {
    arena: &'a ThinNodeArena,
    output: String,
    indent_level: u32,
    state: AsyncTransformState,
}

impl<'a> AsyncES5Emitter<'a> {
    pub fn new(arena: &'a ThinNodeArena) -> Self {
        Self {
            arena,
            output: String::with_capacity(1024),
            indent_level: 0,
            state: AsyncTransformState::new(),
        }
    }

    pub fn set_indent_level(&mut self, level: u32) {
        self.indent_level = level;
    }

    /// Check if a function body contains any await expressions
    pub fn body_contains_await(&self, body_idx: NodeIndex) -> bool {
        self.contains_await_recursive(body_idx)
    }

    fn contains_await_recursive(&self, idx: NodeIndex) -> bool {
        let Some(node) = self.arena.get(idx) else {
            return false;
        };

        // Check if this is an await expression
        if node.kind == syntax_kind_ext::AWAIT_EXPRESSION {
            return true;
        }

        // Don't recurse into nested functions (they have their own async context)
        if node.kind == syntax_kind_ext::FUNCTION_DECLARATION
            || node.kind == syntax_kind_ext::FUNCTION_EXPRESSION
            || node.kind == syntax_kind_ext::ARROW_FUNCTION
        {
            return false;
        }

        // Check block statements
        if node.kind == syntax_kind_ext::BLOCK {
            if let Some(block) = self.arena.get_block(node) {
                for &stmt_idx in &block.statements.nodes {
                    if self.contains_await_recursive(stmt_idx) {
                        return true;
                    }
                }
            }
            return false;
        }

        // Check expression statements
        if node.kind == syntax_kind_ext::EXPRESSION_STATEMENT {
            if let Some(expr_stmt) = self.arena.get_expression_statement(node) {
                return self.contains_await_recursive(expr_stmt.expression);
            }
        }

        // Check return statements
        if node.kind == syntax_kind_ext::RETURN_STATEMENT {
            if let Some(ret) = self.arena.get_return_statement(node) {
                return self.contains_await_recursive(ret.expression);
            }
        }

        // Check variable statements
        if node.kind == syntax_kind_ext::VARIABLE_STATEMENT {
            if let Some(var_data) = self.arena.get_variable(node) {
                for &decl_idx in &var_data.declarations.nodes {
                    if let Some(decl_node) = self.arena.get(decl_idx) {
                        if let Some(decl) = self.arena.get_variable_declaration(decl_node) {
                            if self.contains_await_recursive(decl.initializer) {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        // Check if statements
        if node.kind == syntax_kind_ext::IF_STATEMENT {
            if let Some(if_stmt) = self.arena.get_if_statement(node) {
                if self.contains_await_recursive(if_stmt.expression) {
                    return true;
                }
                if self.contains_await_recursive(if_stmt.then_statement) {
                    return true;
                }
                if self.contains_await_recursive(if_stmt.else_statement) {
                    return true;
                }
            }
        }

        // Check call expressions
        if node.kind == syntax_kind_ext::CALL_EXPRESSION {
            if let Some(call) = self.arena.get_call_expr(node) {
                if self.contains_await_recursive(call.expression) {
                    return true;
                }
                if let Some(args) = &call.arguments {
                    for &arg_idx in &args.nodes {
                        if self.contains_await_recursive(arg_idx) {
                            return true;
                        }
                    }
                }
            }
        }

        // Check binary expressions
        if node.kind == syntax_kind_ext::BINARY_EXPRESSION {
            if let Some(bin) = self.arena.get_binary_expr(node) {
                if self.contains_await_recursive(bin.left) {
                    return true;
                }
                if self.contains_await_recursive(bin.right) {
                    return true;
                }
            }
        }

        // Check prefix/postfix unary expressions
        if node.kind == syntax_kind_ext::PREFIX_UNARY_EXPRESSION
            || node.kind == syntax_kind_ext::POSTFIX_UNARY_EXPRESSION
        {
            if let Some(unary) = self.arena.get_unary_expr(node) {
                return self.contains_await_recursive(unary.operand);
            }
        }

        // Check parenthesized expressions
        if node.kind == syntax_kind_ext::PARENTHESIZED_EXPRESSION {
            if let Some(paren) = self.arena.get_parenthesized(node) {
                return self.contains_await_recursive(paren.expression);
            }
        }

        false
    }

    /// Emit a simple async body with no await (inline format)
    /// Returns: "return __generator(this, function (_a) { return [2 /*return*/]; })"
    /// or with return value: "return __generator(this, function (_a) { return [2 /*return*/, expr]; })"
    pub fn emit_simple_generator_body(&mut self, body_idx: NodeIndex) -> String {
        self.output.clear();

        self.write("return __generator(this, function (_a) {");

        // Check if body is a block with a single return statement or empty
        let Some(body_node) = self.arena.get(body_idx) else {
            self.write(" return [2 /*return*/]; });");
            return std::mem::take(&mut self.output);
        };

        if body_node.kind == syntax_kind_ext::BLOCK {
            if let Some(block) = self.arena.get_block(body_node) {
                // Check for single return statement or empty block
                if block.statements.nodes.is_empty() {
                    self.write(" return [2 /*return*/]; });");
                    return std::mem::take(&mut self.output);
                }

                if block.statements.nodes.len() == 1 {
                    let stmt_idx = block.statements.nodes[0];
                    if let Some(stmt_node) = self.arena.get(stmt_idx) {
                        if stmt_node.kind == syntax_kind_ext::RETURN_STATEMENT {
                            if let Some(ret) = self.arena.get_return_statement(stmt_node) {
                                if ret.expression.is_none() {
                                    self.write(" return [2 /*return*/]; });");
                                } else {
                                    self.write(" return [2 /*return*/, ");
                                    self.emit_expression(ret.expression);
                                    self.write("]; });");
                                }
                                return std::mem::take(&mut self.output);
                            }
                        }
                    }
                }

                // For non-trivial blocks, emit newlines
                self.write_line();
                self.increase_indent();
                self.write_indent();
                self.write("return [2 /*return*/];");
                self.write_line();
                self.decrease_indent();
                self.write_indent();
                self.write("});");
            }
        } else {
            // Concise arrow body - treat as return expression
            self.write(" return [2 /*return*/, ");
            self.emit_expression(body_idx);
            self.write("]; });");
        }

        std::mem::take(&mut self.output)
    }

    /// Emit a generator body with await (switch/case format)
    pub fn emit_generator_body_with_await(&mut self, body_idx: NodeIndex) -> String {
        self.output.clear();
        self.state.reset();
        self.state.has_await = true;

        self.write("return __generator(this, function (_a) {");
        self.write_line();
        self.increase_indent();

        // Start switch statement
        self.write_indent();
        self.write("switch (_a.label) {");
        self.write_line();
        self.increase_indent();

        // Emit case 0 (entry point)
        self.emit_case_label(0);

        // Process body statements
        self.emit_async_body_statements(body_idx);

        // Close switch
        self.decrease_indent();
        self.write_indent();
        self.write("}");
        self.write_line();

        // Close generator function
        self.decrease_indent();
        self.write_indent();
        self.write("});");

        std::mem::take(&mut self.output)
    }

    fn emit_async_body_statements(&mut self, body_idx: NodeIndex) {
        let Some(body_node) = self.arena.get(body_idx) else {
            // Empty body - just return
            self.write_indent();
            self.write("return [2 /*return*/];");
            self.write_line();
            return;
        };

        if body_node.kind == syntax_kind_ext::BLOCK {
            if let Some(block) = self.arena.get_block(body_node) {
                for &stmt_idx in &block.statements.nodes {
                    self.emit_async_statement(stmt_idx);
                }
            }
        } else {
            // Concise arrow - treat as return expression
            self.emit_return_with_possible_await(body_idx);
        }

        // Ensure we have a final return
        self.write_indent();
        self.write("return [2 /*return*/];");
        self.write_line();
    }

    fn emit_async_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(stmt_node) = self.arena.get(stmt_idx) else {
            return;
        };

        match stmt_node.kind {
            k if k == syntax_kind_ext::EXPRESSION_STATEMENT => {
                if let Some(expr_stmt) = self.arena.get_expression_statement(stmt_node) {
                    if self.is_await_expression(expr_stmt.expression) {
                        // await expr; -> return [4, expr]; case N: _a.sent();
                        self.emit_await_statement(expr_stmt.expression);
                    } else {
                        // Regular expression statement
                        self.write_indent();
                        self.emit_expression(expr_stmt.expression);
                        self.write(";");
                        self.write_line();
                    }
                }
            }
            k if k == syntax_kind_ext::RETURN_STATEMENT => {
                if let Some(ret) = self.arena.get_return_statement(stmt_node) {
                    self.emit_return_with_possible_await(ret.expression);
                }
            }
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                // Handle variable declarations with potential await
                self.emit_variable_statement_async(stmt_node);
            }
            _ => {
                // For other statements, just emit a placeholder for now
                // Full implementation would handle if/for/while/etc.
                self.write_indent();
                self.write("/* statement */;");
                self.write_line();
            }
        }
    }

    fn emit_await_statement(&mut self, await_idx: NodeIndex) {
        let Some(await_node) = self.arena.get(await_idx) else {
            return;
        };

        // Get the await operand (await uses UnaryExprDataEx, not UnaryExprData)
        let operand_idx = if await_node.has_data() {
            if let Some(unary_ex) = self.arena.unary_exprs_ex.get(await_node.data_index as usize) {
                unary_ex.expression
            } else {
                return;
            }
        } else {
            return;
        };

        // Emit: return [4 /*yield*/, operand];
        self.write_indent();
        self.write("return [4 /*yield*/, ");
        self.emit_expression(operand_idx);
        self.write("];");
        self.write_line();

        // Next case: _a.sent();
        self.state.label_counter += 1;
        self.emit_case_label(self.state.label_counter);
        self.write_indent();
        self.write("_a.sent();");
        self.write_line();
    }

    fn emit_return_with_possible_await(&mut self, expr_idx: NodeIndex) {
        if expr_idx.is_none() {
            self.write_indent();
            self.write("return [2 /*return*/];");
            self.write_line();
            return;
        }

        if self.is_await_expression(expr_idx) {
            // return await expr; -> return [4, expr]; case N: return [2, _a.sent()];
            let Some(await_node) = self.arena.get(expr_idx) else {
                return;
            };

            // await uses UnaryExprDataEx, not UnaryExprData
            let operand_idx = if await_node.has_data() {
                if let Some(unary_ex) = self.arena.unary_exprs_ex.get(await_node.data_index as usize) {
                    unary_ex.expression
                } else {
                    return;
                }
            } else {
                return;
            };

            self.write_indent();
            self.write("return [4 /*yield*/, ");
            self.emit_expression(operand_idx);
            self.write("];");
            self.write_line();

            self.state.label_counter += 1;
            self.emit_case_label(self.state.label_counter);
            self.write_indent();
            self.write("return [2 /*return*/, _a.sent()];");
            self.write_line();
        } else {
            self.write_indent();
            self.write("return [2 /*return*/, ");
            self.emit_expression(expr_idx);
            self.write("];");
            self.write_line();
        }
    }

    fn emit_variable_statement_async(&mut self, stmt_node: &crate::parser::thin_node::ThinNode) {
        let Some(var_data) = self.arena.get_variable(stmt_node) else {
            return;
        };

        for &decl_idx in &var_data.declarations.nodes {
            let Some(decl_node) = self.arena.get(decl_idx) else {
                continue;
            };

            let Some(decl) = self.arena.get_variable_declaration(decl_node) else {
                continue;
            };

            if self.contains_await_recursive(decl.initializer) {
                // Handle await in initializer
                // var x = await foo(); -> return [4, foo()]; case N: x = _a.sent();
                let name = self.get_binding_name(decl.name);

                if self.is_await_expression(decl.initializer) {
                    let Some(await_node) = self.arena.get(decl.initializer) else {
                        continue;
                    };

                    // await uses UnaryExprDataEx, not UnaryExprData
                    let operand_idx = if await_node.has_data() {
                        if let Some(unary_ex) = self.arena.unary_exprs_ex.get(await_node.data_index as usize) {
                            unary_ex.expression
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    };

                    self.write_indent();
                    self.write("return [4 /*yield*/, ");
                    self.emit_expression(operand_idx);
                    self.write("];");
                    self.write_line();

                    self.state.label_counter += 1;
                    self.emit_case_label(self.state.label_counter);
                    self.write_indent();
                    self.write(&name);
                    self.write(" = _a.sent();");
                    self.write_line();
                }
            } else {
                // Regular variable declaration
                self.write_indent();
                self.emit_expression(decl.name);
                if !decl.initializer.is_none() {
                    self.write(" = ");
                    self.emit_expression(decl.initializer);
                }
                self.write(";");
                self.write_line();
            }
        }
    }

    fn get_binding_name(&self, name_idx: NodeIndex) -> String {
        let Some(node) = self.arena.get(name_idx) else {
            return String::new();
        };

        if let Some(ident) = self.arena.get_identifier(node) {
            return ident.escaped_text.clone();
        }

        String::new()
    }

    fn is_await_expression(&self, idx: NodeIndex) -> bool {
        if let Some(node) = self.arena.get(idx) {
            return node.kind == syntax_kind_ext::AWAIT_EXPRESSION;
        }
        false
    }

    fn emit_case_label(&mut self, label: u32) {
        // Case labels are indented less than the case body
        self.decrease_indent();
        self.write_indent();
        self.write("case ");
        self.write(&label.to_string());
        self.write(":");
        if label > 0 {
            self.write_line();
        } else {
            self.write(" ");
        }
        self.increase_indent();
    }

    fn emit_expression(&mut self, idx: NodeIndex) {
        let Some(node) = self.arena.get(idx) else {
            return;
        };

        match node.kind {
            k if k == SyntaxKind::NumericLiteral as u16 => {
                if let Some(lit) = self.arena.get_literal(node) {
                    self.write(&lit.text);
                }
            }
            k if k == SyntaxKind::StringLiteral as u16 => {
                if let Some(lit) = self.arena.get_literal(node) {
                    self.write("\"");
                    self.write(&lit.text);
                    self.write("\"");
                }
            }
            k if k == SyntaxKind::Identifier as u16 => {
                if let Some(ident) = self.arena.get_identifier(node) {
                    self.write(&ident.escaped_text);
                }
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
            k if k == SyntaxKind::UndefinedKeyword as u16 => {
                self.write("undefined");
            }
            k if k == SyntaxKind::ThisKeyword as u16 => {
                self.write("this");
            }
            k if k == syntax_kind_ext::CALL_EXPRESSION => {
                if let Some(call) = self.arena.get_call_expr(node) {
                    self.emit_expression(call.expression);
                    self.write("(");
                    if let Some(args) = &call.arguments {
                        let mut first = true;
                        for &arg_idx in &args.nodes {
                            if !first {
                                self.write(", ");
                            }
                            first = false;
                            self.emit_expression(arg_idx);
                        }
                    }
                    self.write(")");
                }
            }
            k if k == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION => {
                if let Some(access) = self.arena.get_access_expr(node) {
                    self.emit_expression(access.expression);
                    self.write(".");
                    self.emit_expression(access.name_or_argument);
                }
            }
            k if k == syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION => {
                if let Some(access) = self.arena.get_access_expr(node) {
                    self.emit_expression(access.expression);
                    self.write("[");
                    self.emit_expression(access.name_or_argument);
                    self.write("]");
                }
            }
            k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                if let Some(bin) = self.arena.get_binary_expr(node) {
                    self.emit_expression(bin.left);
                    self.write(" ");
                    self.emit_operator(bin.operator_token);
                    self.write(" ");
                    self.emit_expression(bin.right);
                }
            }
            k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION => {
                if let Some(unary) = self.arena.get_unary_expr(node) {
                    self.emit_operator(unary.operator);
                    self.emit_expression(unary.operand);
                }
            }
            k if k == syntax_kind_ext::PARENTHESIZED_EXPRESSION => {
                if let Some(paren) = self.arena.get_parenthesized(node) {
                    self.write("(");
                    self.emit_expression(paren.expression);
                    self.write(")");
                }
            }
            k if k == syntax_kind_ext::AWAIT_EXPRESSION => {
                // For expressions like return await x, we emit just the operand
                // (the await is handled by the state machine)
                if let Some(unary) = self.arena.get_unary_expr_ex(node) {
                    if !unary.expression.is_none() {
                        self.emit_expression(unary.expression);
                    }
                }
            }
            _ => {
                // Fallback for unhandled expressions
                self.write("void 0");
            }
        }
    }

    fn emit_operator(&mut self, op: u16) {
        let op_str = match op {
            k if k == SyntaxKind::PlusToken as u16 => "+",
            k if k == SyntaxKind::MinusToken as u16 => "-",
            k if k == SyntaxKind::AsteriskToken as u16 => "*",
            k if k == SyntaxKind::SlashToken as u16 => "/",
            k if k == SyntaxKind::PercentToken as u16 => "%",
            k if k == SyntaxKind::PlusPlusToken as u16 => "++",
            k if k == SyntaxKind::MinusMinusToken as u16 => "--",
            k if k == SyntaxKind::EqualsToken as u16 => "=",
            k if k == SyntaxKind::PlusEqualsToken as u16 => "+=",
            k if k == SyntaxKind::MinusEqualsToken as u16 => "-=",
            k if k == SyntaxKind::EqualsEqualsToken as u16 => "==",
            k if k == SyntaxKind::EqualsEqualsEqualsToken as u16 => "===",
            k if k == SyntaxKind::ExclamationEqualsToken as u16 => "!=",
            k if k == SyntaxKind::ExclamationEqualsEqualsToken as u16 => "!==",
            k if k == SyntaxKind::LessThanToken as u16 => "<",
            k if k == SyntaxKind::GreaterThanToken as u16 => ">",
            k if k == SyntaxKind::LessThanEqualsToken as u16 => "<=",
            k if k == SyntaxKind::GreaterThanEqualsToken as u16 => ">=",
            k if k == SyntaxKind::AmpersandAmpersandToken as u16 => "&&",
            k if k == SyntaxKind::BarBarToken as u16 => "||",
            k if k == SyntaxKind::ExclamationToken as u16 => "!",
            k if k == SyntaxKind::TildeToken as u16 => "~",
            k if k == SyntaxKind::AmpersandToken as u16 => "&",
            k if k == SyntaxKind::BarToken as u16 => "|",
            k if k == SyntaxKind::CaretToken as u16 => "^",
            _ => "/* op */",
        };
        self.write(op_str);
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn write_line(&mut self) {
        self.output.push('\n');
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    fn decrease_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thin_parser::ThinParserState;

    fn parse_and_emit_async(source: &str) -> String {
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        if let Some(root_node) = parser.arena.get(root) {
            if let Some(source_file) = parser.arena.get_source_file(root_node) {
                if let Some(&func_idx) = source_file.statements.nodes.first() {
                    if let Some(func_node) = parser.arena.get(func_idx) {
                        if let Some(func) = parser.arena.get_function(func_node) {
                            let emitter = AsyncES5Emitter::new(&parser.arena);
                            let has_await = emitter.body_contains_await(func.body);
                            let mut emitter = AsyncES5Emitter::new(&parser.arena);
                            if has_await {
                                return emitter.emit_generator_body_with_await(func.body);
                            } else {
                                return emitter.emit_simple_generator_body(func.body);
                            }
                        }
                    }
                }
            }
        }
        String::new()
    }

    #[test]
    fn test_simple_async_empty() {
        let output = parse_and_emit_async("async function foo() { }");
        assert!(output.contains("return __generator"), "Should have generator wrapper");
        assert!(output.contains("[2 /*return*/]"), "Should have return instruction");
        assert!(!output.contains("switch"), "Empty body should not have switch");
    }

    #[test]
    fn test_simple_async_with_return() {
        let output = parse_and_emit_async("async function foo() { return 42; }");
        assert!(output.contains("[2 /*return*/, 42]"), "Should return 42");
    }

    #[test]
    fn test_async_with_await() {
        let output = parse_and_emit_async("async function foo() { await bar(); }");
        assert!(output.contains("switch (_a.label)"), "Should have switch statement");
        assert!(output.contains("[4 /*yield*/"), "Should have yield instruction");
        assert!(output.contains("_a.sent()"), "Should call _a.sent()");
    }

    #[test]
    fn test_body_contains_await_detection() {
        let mut parser =
            ThinParserState::new("test.ts".to_string(), "async function foo() { await x; }".to_string());
        let root = parser.parse_source_file();

        if let Some(root_node) = parser.arena.get(root) {
            if let Some(source_file) = parser.arena.get_source_file(root_node) {
                if let Some(&func_idx) = source_file.statements.nodes.first() {
                    if let Some(func_node) = parser.arena.get(func_idx) {
                        if let Some(func) = parser.arena.get_function(func_node) {
                            let emitter = AsyncES5Emitter::new(&parser.arena);
                            assert!(emitter.body_contains_await(func.body), "Should detect await");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_no_await_in_simple_function() {
        let mut parser =
            ThinParserState::new("test.ts".to_string(), "async function foo() { return 1; }".to_string());
        let root = parser.parse_source_file();

        if let Some(root_node) = parser.arena.get(root) {
            if let Some(source_file) = parser.arena.get_source_file(root_node) {
                if let Some(&func_idx) = source_file.statements.nodes.first() {
                    if let Some(func_node) = parser.arena.get(func_idx) {
                        if let Some(func) = parser.arena.get_function(func_node) {
                            let emitter = AsyncES5Emitter::new(&parser.arena);
                            assert!(!emitter.body_contains_await(func.body), "Should not detect await");
                        }
                    }
                }
            }
        }
    }
}
