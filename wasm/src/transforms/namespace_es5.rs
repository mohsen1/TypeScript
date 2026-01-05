//! ES5 Namespace Transform
//!
//! Transforms TypeScript namespaces to ES5 IIFE patterns:
//!
//! ```typescript
//! namespace foo {
//!     export class Provide { }
//! }
//! ```
//!
//! Becomes:
//!
//! ```javascript
//! var foo;
//! (function (foo) {
//!     var Provide = /** @class */ (function () {
//!         function Provide() { }
//!         return Provide;
//!     }());
//!     foo.Provide = Provide;
//! })(foo || (foo = {}));
//! ```

use crate::parser::thin_node::ThinNodeArena;
use crate::parser::{NodeIndex, NodeList};
use crate::parser::syntax_kind_ext;
use crate::scanner::SyntaxKind;
use crate::transforms::class_es5::ClassES5Emitter;

/// Namespace ES5 emitter
pub struct NamespaceES5Emitter<'a> {
    arena: &'a ThinNodeArena,
    output: String,
    indent_level: u32,
}

impl<'a> NamespaceES5Emitter<'a> {
    pub fn new(arena: &'a ThinNodeArena) -> Self {
        NamespaceES5Emitter {
            arena,
            output: String::with_capacity(4096),
            indent_level: 0,
        }
    }

    /// Emit a namespace declaration
    pub fn emit_namespace(&mut self, ns_idx: NodeIndex) -> String {
        self.output.clear();
        
        let Some(ns_node) = self.arena.get(ns_idx) else {
            return String::new();
        };
        
        let Some(ns_data) = self.arena.get_module(ns_node) else {
            return String::new();
        };
        
        let ns_name = self.get_identifier_text(ns_data.name);
        
        // var foo;
        self.write("var ");
        self.write(&ns_name);
        self.write(";");
        self.write_line();
        
        // (function (foo) { ... })(foo || (foo = {}));
        self.emit_namespace_iife(&ns_name, ns_data.body, false);
        
        std::mem::take(&mut self.output)
    }
    
    /// Emit namespace IIFE
    /// `is_nested` is true for nested namespaces like `foo.bar`
    fn emit_namespace_iife(&mut self, ns_name: &str, body_idx: NodeIndex, is_nested: bool) {
        self.write("(function (");
        self.write(ns_name);
        self.write(") {");
        self.write_line();
        self.increase_indent();
        
        // Emit body contents
        self.emit_namespace_body(ns_name, body_idx);
        
        self.decrease_indent();
        self.write_indent();
        self.write("})(");
        self.write(ns_name);
        self.write(" || (");
        self.write(ns_name);
        self.write(" = {}));");
        self.write_line();
    }
    
    /// Emit namespace body contents
    fn emit_namespace_body(&mut self, ns_name: &str, body_idx: NodeIndex) {
        let Some(body_node) = self.arena.get(body_idx) else { return };
        
        // Check if it's a module block
        if let Some(block_data) = self.arena.get_module_block(body_node) {
            if let Some(ref stmts) = block_data.statements {
                for &stmt_idx in &stmts.nodes {
                    self.emit_namespace_member(ns_name, stmt_idx);
                }
            }
        }
    }
    
    /// Emit a namespace member and its export assignment if needed
    fn emit_namespace_member(&mut self, ns_name: &str, member_idx: NodeIndex) {
        let Some(member_node) = self.arena.get(member_idx) else { return };
        
        match member_node.kind {
            k if k == syntax_kind_ext::EXPORT_DECLARATION => {
                // Handle export declarations by extracting the inner declaration
                if let Some(export_data) = self.arena.get_export_decl(member_node) {
                    let inner_decl_idx = export_data.export_clause;
                    self.emit_namespace_member_exported(ns_name, inner_decl_idx);
                }
            }
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                self.emit_function_in_namespace(ns_name, member_idx);
            }
            k if k == syntax_kind_ext::CLASS_DECLARATION => {
                self.emit_class_in_namespace(ns_name, member_idx);
            }
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                self.emit_variable_in_namespace(ns_name, member_idx);
            }
            k if k == syntax_kind_ext::MODULE_DECLARATION => {
                self.emit_nested_namespace(ns_name, member_idx);
            }
            k if k == syntax_kind_ext::ENUM_DECLARATION => {
                self.emit_enum_in_namespace(ns_name, member_idx);
            }
            k if k == syntax_kind_ext::INTERFACE_DECLARATION => {}
            k if k == syntax_kind_ext::TYPE_ALIAS_DECLARATION => {
                // TypeScript-only: skip in JS emit
            }
            _ => {
                // Other statements - emit directly
                self.emit_statement(member_idx);
            }
        }
    }
    
    /// Emit an exported namespace member (extracted from EXPORT_DECLARATION)
    fn emit_namespace_member_exported(&mut self, ns_name: &str, decl_idx: NodeIndex) {
        let Some(decl_node) = self.arena.get(decl_idx) else { return };
        
        match decl_node.kind {
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                self.emit_function_in_namespace_exported(ns_name, decl_idx);
            }
            k if k == syntax_kind_ext::CLASS_DECLARATION => {
                self.emit_class_in_namespace_exported(ns_name, decl_idx);
            }
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                self.emit_variable_in_namespace_exported(ns_name, decl_idx);
            }
            k if k == syntax_kind_ext::ENUM_DECLARATION => {
                self.emit_enum_in_namespace_exported(ns_name, decl_idx);
            }
            k if k == syntax_kind_ext::MODULE_DECLARATION => {
                // Nested namespace export
                self.emit_nested_namespace_exported(ns_name, decl_idx);
            }
            k if k == syntax_kind_ext::INTERFACE_DECLARATION => {}
            k if k == syntax_kind_ext::TYPE_ALIAS_DECLARATION => {}
            _ => {}
        }
    }
    
    /// Emit a function declaration in namespace context
    fn emit_function_in_namespace(&mut self, ns_name: &str, func_idx: NodeIndex) {
        let Some(func_node) = self.arena.get(func_idx) else { return };
        let Some(func_data) = self.arena.get_function(func_node) else { return };
        
        // Skip declaration-only functions
        if func_data.body.is_none() {
            return;
        }
        
        let func_name = self.get_identifier_text(func_data.name);
        let is_exported = self.has_export_modifier(&func_data.modifiers);
        
        // function funcName(...) { ... }
        self.write_indent();
        self.write("function ");
        self.write(&func_name);
        self.write("(");
        self.emit_parameters(&func_data.parameters);
        self.write(") ");
        self.emit_block(func_data.body);
        self.write_line();
        
        // Export assignment: ns.funcName = funcName;
        if is_exported {
            self.write_indent();
            self.write(ns_name);
            self.write(".");
            self.write(&func_name);
            self.write(" = ");
            self.write(&func_name);
            self.write(";");
            self.write_line();
        }
    }
    
    /// Emit an exported function in namespace context
    fn emit_function_in_namespace_exported(&mut self, ns_name: &str, func_idx: NodeIndex) {
        let Some(func_node) = self.arena.get(func_idx) else { return };
        let Some(func_data) = self.arena.get_function(func_node) else { return };
        
        // Skip declaration-only functions
        if func_data.body.is_none() {
            return;
        }
        
        let func_name = self.get_identifier_text(func_data.name);
        
        // function funcName(...) { ... }
        self.write_indent();
        self.write("function ");
        self.write(&func_name);
        self.write("(");
        self.emit_parameters(&func_data.parameters);
        self.write(") ");
        self.emit_block(func_data.body);
        self.write_line();
        
        // Always export: ns.funcName = funcName;
        self.write_indent();
        self.write(ns_name);
        self.write(".");
        self.write(&func_name);
        self.write(" = ");
        self.write(&func_name);
        self.write(";");
        self.write_line();
    }
    
    /// Emit a class declaration in namespace context
    fn emit_class_in_namespace(&mut self, ns_name: &str, class_idx: NodeIndex) {
        let Some(class_node) = self.arena.get(class_idx) else { return };
        let Some(class_data) = self.arena.get_class(class_node) else { return };
        
        let class_name = self.get_identifier_text(class_data.name);
        let is_exported = self.has_export_modifier(&class_data.modifiers);
        
        // Use ES5 class emitter
        let mut class_emitter = ClassES5Emitter::new(self.arena);
        let class_output = class_emitter.emit_class(class_idx);
        
        // Write indented class output
        self.write_indent();
        self.write(&class_output);
        
        // Export assignment: ns.ClassName = ClassName;
        if is_exported {
            self.write_indent();
            self.write(ns_name);
            self.write(".");
            self.write(&class_name);
            self.write(" = ");
            self.write(&class_name);
            self.write(";");
            self.write_line();
        }
    }
    
    /// Emit an exported class in namespace context
    fn emit_class_in_namespace_exported(&mut self, ns_name: &str, class_idx: NodeIndex) {
        let Some(class_node) = self.arena.get(class_idx) else { return };
        let Some(class_data) = self.arena.get_class(class_node) else { return };
        
        let class_name = self.get_identifier_text(class_data.name);
        
        // Use ES5 class emitter
        let mut class_emitter = ClassES5Emitter::new(self.arena);
        let class_output = class_emitter.emit_class(class_idx);
        
        // Write indented class output
        self.write_indent();
        self.write(&class_output);
        
        // Always export: ns.ClassName = ClassName;
        self.write_indent();
        self.write(ns_name);
        self.write(".");
        self.write(&class_name);
        self.write(" = ");
        self.write(&class_name);
        self.write(";");
        self.write_line();
    }
    
    /// Emit a variable statement in namespace context
    fn emit_variable_in_namespace(&mut self, ns_name: &str, var_idx: NodeIndex) {
        let Some(var_node) = self.arena.get(var_idx) else { return };
        let Some(var_data) = self.arena.get_variable(var_node) else { return };
        
        let is_exported = self.has_export_modifier(&var_data.modifiers);
        
        // Emit variable declarations
        self.write_indent();
        self.write("var ");
        
        let mut var_names = Vec::new();
        let mut first = true;
        
        for &decl_list_idx in &var_data.declarations.nodes {
            if let Some(decl_list_node) = self.arena.get(decl_list_idx) {
                if let Some(decl_list) = self.arena.get_variable(decl_list_node) {
                    for &decl_idx in &decl_list.declarations.nodes {
                        if let Some(decl_node) = self.arena.get(decl_idx) {
                            if let Some(decl) = self.arena.get_variable_declaration(decl_node) {
                                if !first {
                                    self.write(", ");
                                }
                                first = false;
                                
                                let var_name = self.get_identifier_text(decl.name);
                                self.write(&var_name);
                                
                                if !decl.initializer.is_none() {
                                    self.write(" = ");
                                    self.emit_expression(decl.initializer);
                                }
                                
                                if is_exported {
                                    var_names.push(var_name);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        self.write(";");
        self.write_line();
        
        // Export assignments
        for var_name in var_names {
            self.write_indent();
            self.write(ns_name);
            self.write(".");
            self.write(&var_name);
            self.write(" = ");
            self.write(&var_name);
            self.write(";");
            self.write_line();
        }
    }
    
    /// Emit an exported variable statement in namespace context
    fn emit_variable_in_namespace_exported(&mut self, ns_name: &str, var_idx: NodeIndex) {
        let Some(var_node) = self.arena.get(var_idx) else { return };
        let Some(var_data) = self.arena.get_variable(var_node) else { return };
        
        // Emit variable declarations
        self.write_indent();
        self.write("var ");
        
        let mut var_names = Vec::new();
        let mut first = true;
        
        for &decl_list_idx in &var_data.declarations.nodes {
            if let Some(decl_list_node) = self.arena.get(decl_list_idx) {
                if let Some(decl_list) = self.arena.get_variable(decl_list_node) {
                    for &decl_idx in &decl_list.declarations.nodes {
                        if let Some(decl_node) = self.arena.get(decl_idx) {
                            if let Some(decl) = self.arena.get_variable_declaration(decl_node) {
                                if !first {
                                    self.write(", ");
                                }
                                first = false;
                                
                                let var_name = self.get_identifier_text(decl.name);
                                self.write(&var_name);
                                
                                if !decl.initializer.is_none() {
                                    self.write(" = ");
                                    self.emit_expression(decl.initializer);
                                }
                                
                                var_names.push(var_name);
                            }
                        }
                    }
                }
            }
        }
        
        self.write(";");
        self.write_line();
        
        // Always export
        for var_name in var_names {
            self.write_indent();
            self.write(ns_name);
            self.write(".");
            self.write(&var_name);
            self.write(" = ");
            self.write(&var_name);
            self.write(";");
            self.write_line();
        }
    }
    
    /// Emit an exported enum in namespace context
    fn emit_enum_in_namespace_exported(&mut self, ns_name: &str, enum_idx: NodeIndex) {
        let Some(enum_node) = self.arena.get(enum_idx) else { return };
        let Some(enum_data) = self.arena.get_enum(enum_node) else { return };
        
        let enum_name = self.get_identifier_text(enum_data.name);
        
        // var EnumName;
        self.write_indent();
        self.write("var ");
        self.write(&enum_name);
        self.write(";");
        self.write_line();
        
        // (function (EnumName) { ... })(EnumName || (EnumName = {}));
        self.write_indent();
        self.write("(function (");
        self.write(&enum_name);
        self.write(") {");
        self.write_line();
        self.increase_indent();
        
        // Emit enum members
        let mut value = 0i64;
        for &member_idx in &enum_data.members.nodes {
            if let Some(member_node) = self.arena.get(member_idx) {
                if let Some(member_data) = self.arena.get_enum_member(member_node) {
                    let member_name = self.get_identifier_text(member_data.name);
                    
                    if !member_data.initializer.is_none() {
                        self.write_indent();
                        self.write(&enum_name);
                        self.write("[");
                        self.write(&enum_name);
                        self.write("[\"");
                        self.write(&member_name);
                        self.write("\"] = ");
                        self.emit_expression(member_data.initializer);
                        self.write("] = \"");
                        self.write(&member_name);
                        self.write("\";");
                        self.write_line();
                    } else {
                        self.write_indent();
                        self.write(&enum_name);
                        self.write("[");
                        self.write(&enum_name);
                        self.write("[\"");
                        self.write(&member_name);
                        self.write("\"] = ");
                        self.write(&value.to_string());
                        self.write("] = \"");
                        self.write(&member_name);
                        self.write("\";");
                        self.write_line();
                        value += 1;
                    }
                }
            }
        }
        
        self.decrease_indent();
        self.write_indent();
        self.write("})(");
        self.write(&enum_name);
        self.write(" || (");
        self.write(&enum_name);
        self.write(" = {}));");
        self.write_line();
        
        // Always export
        self.write_indent();
        self.write(ns_name);
        self.write(".");
        self.write(&enum_name);
        self.write(" = ");
        self.write(&enum_name);
        self.write(";");
        self.write_line();
    }
    
    /// Emit an exported nested namespace
    fn emit_nested_namespace_exported(&mut self, parent_ns: &str, ns_idx: NodeIndex) {
        let Some(ns_node) = self.arena.get(ns_idx) else { return };
        let Some(ns_data) = self.arena.get_module(ns_node) else { return };
        
        let nested_name = self.get_identifier_text(ns_data.name);
        
        // var bar;
        self.write_indent();
        self.write("var ");
        self.write(&nested_name);
        self.write(";");
        self.write_line();
        
        // (function (bar) { ... })(bar = foo.bar || (foo.bar = {}));
        self.write_indent();
        self.write("(function (");
        self.write(&nested_name);
        self.write(") {");
        self.write_line();
        self.increase_indent();
        
        self.emit_namespace_body(&nested_name, ns_data.body);
        
        self.decrease_indent();
        self.write_indent();
        self.write("})(");
        self.write(&nested_name);
        self.write(" = ");
        self.write(parent_ns);
        self.write(".");
        self.write(&nested_name);
        self.write(" || (");
        self.write(parent_ns);
        self.write(".");
        self.write(&nested_name);
        self.write(" = {}));");
        self.write_line();
    }
    
    /// Emit a nested namespace
    fn emit_nested_namespace(&mut self, parent_ns: &str, ns_idx: NodeIndex) {
        let Some(ns_node) = self.arena.get(ns_idx) else { return };
        let Some(ns_data) = self.arena.get_module(ns_node) else { return };
        
        let nested_name = self.get_identifier_text(ns_data.name);
        let is_exported = self.has_export_modifier(&ns_data.modifiers);
        
        // var bar;
        self.write_indent();
        self.write("var ");
        self.write(&nested_name);
        self.write(";");
        self.write_line();
        
        // (function (bar) { ... })(bar = foo.bar || (foo.bar = {}));
        self.write_indent();
        self.write("(function (");
        self.write(&nested_name);
        self.write(") {");
        self.write_line();
        self.increase_indent();
        
        self.emit_namespace_body(&nested_name, ns_data.body);
        
        self.decrease_indent();
        self.write_indent();
        self.write("})(");
        self.write(&nested_name);
        self.write(" = ");
        self.write(parent_ns);
        self.write(".");
        self.write(&nested_name);
        self.write(" || (");
        self.write(parent_ns);
        self.write(".");
        self.write(&nested_name);
        self.write(" = {}));");
        self.write_line();
    }
    
    /// Emit enum in namespace
    fn emit_enum_in_namespace(&mut self, ns_name: &str, enum_idx: NodeIndex) {
        let Some(enum_node) = self.arena.get(enum_idx) else { return };
        let Some(enum_data) = self.arena.get_enum(enum_node) else { return };
        
        let enum_name = self.get_identifier_text(enum_data.name);
        let is_exported = self.has_export_modifier(&enum_data.modifiers);
        
        // var EnumName;
        self.write_indent();
        self.write("var ");
        self.write(&enum_name);
        self.write(";");
        self.write_line();
        
        // (function (EnumName) { ... })(EnumName || (EnumName = {}));
        self.write_indent();
        self.write("(function (");
        self.write(&enum_name);
        self.write(") {");
        self.write_line();
        self.increase_indent();
        
        // Emit enum members
        let mut value = 0i64;
        for &member_idx in &enum_data.members.nodes {
            if let Some(member_node) = self.arena.get(member_idx) {
                if let Some(member_data) = self.arena.get_enum_member(member_node) {
                    let member_name = self.get_identifier_text(member_data.name);
                    
                    // Check for initializer
                    if !member_data.initializer.is_none() {
                        // EnumName[EnumName["Name"] = value] = "Name";
                        self.write_indent();
                        self.write(&enum_name);
                        self.write("[");
                        self.write(&enum_name);
                        self.write("[\"");
                        self.write(&member_name);
                        self.write("\"] = ");
                        self.emit_expression(member_data.initializer);
                        self.write("] = \"");
                        self.write(&member_name);
                        self.write("\";");
                        self.write_line();
                    } else {
                        self.write_indent();
                        self.write(&enum_name);
                        self.write("[");
                        self.write(&enum_name);
                        self.write("[\"");
                        self.write(&member_name);
                        self.write("\"] = ");
                        self.write(&value.to_string());
                        self.write("] = \"");
                        self.write(&member_name);
                        self.write("\";");
                        self.write_line();
                        value += 1;
                    }
                }
            }
        }
        
        self.decrease_indent();
        self.write_indent();
        self.write("})(");
        self.write(&enum_name);
        self.write(" || (");
        self.write(&enum_name);
        self.write(" = {}));");
        self.write_line();
        
        // Export assignment
        if is_exported {
            self.write_indent();
            self.write(ns_name);
            self.write(".");
            self.write(&enum_name);
            self.write(" = ");
            self.write(&enum_name);
            self.write(";");
            self.write_line();
        }
    }
    
    // =========================================================================
    // Helper Methods
    // =========================================================================
    
    fn emit_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(stmt_node) = self.arena.get(stmt_idx) else { return };
        
        match stmt_node.kind {
            k if k == syntax_kind_ext::EXPRESSION_STATEMENT => {
                if let Some(expr_stmt) = self.arena.get_expression_statement(stmt_node) {
                    self.write_indent();
                    self.emit_expression(expr_stmt.expression);
                    self.write(";");
                    self.write_line();
                }
            }
            k if k == syntax_kind_ext::RETURN_STATEMENT => {
                if let Some(ret) = self.arena.get_return_statement(stmt_node) {
                    self.write_indent();
                    self.write("return");
                    if !ret.expression.is_none() {
                        self.write(" ");
                        self.emit_expression(ret.expression);
                    }
                    self.write(";");
                    self.write_line();
                }
            }
            _ => {}
        }
    }
    
    fn emit_block(&mut self, block_idx: NodeIndex) {
        let Some(block_node) = self.arena.get(block_idx) else { return };
        let Some(block) = self.arena.get_block(block_node) else { return };
        
        if block.statements.nodes.is_empty() {
            self.write("{ }");
            return;
        }
        
        self.write("{");
        self.write_line();
        self.increase_indent();
        
        for &stmt_idx in &block.statements.nodes {
            self.emit_statement(stmt_idx);
        }
        
        self.decrease_indent();
        self.write_indent();
        self.write("}");
    }
    
    fn emit_parameters(&mut self, params: &NodeList) {
        let mut first = true;
        for &param_idx in &params.nodes {
            if !first {
                self.write(", ");
            }
            first = false;
            
            if let Some(param_node) = self.arena.get(param_idx) {
                if let Some(param) = self.arena.get_parameter(param_node) {
                    let name = self.get_identifier_text(param.name);
                    self.write(&name);
                }
            }
        }
    }
    
    fn emit_expression(&mut self, expr_idx: NodeIndex) {
        let Some(expr_node) = self.arena.get(expr_idx) else { return };
        
        match expr_node.kind {
            k if k == SyntaxKind::Identifier as u16 => {
                if let Some(ident) = self.arena.get_identifier(expr_node) {
                    self.write(&ident.escaped_text);
                }
            }
            k if k == SyntaxKind::NumericLiteral as u16 => {
                if let Some(lit) = self.arena.get_literal(expr_node) {
                    self.write(&lit.text);
                }
            }
            k if k == SyntaxKind::StringLiteral as u16 => {
                if let Some(lit) = self.arena.get_literal(expr_node) {
                    self.write("\"");
                    self.write(&lit.text);
                    self.write("\"");
                }
            }
            k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                if let Some(bin) = self.arena.get_binary_expr(expr_node) {
                    self.emit_expression(bin.left);
                    self.write(" ");
                    self.emit_operator_token(bin.operator_token);
                    self.write(" ");
                    self.emit_expression(bin.right);
                }
            }
            k if k == syntax_kind_ext::CALL_EXPRESSION => {
                if let Some(call) = self.arena.get_call_expr(expr_node) {
                    self.emit_expression(call.expression);
                    self.write("(");
                    if let Some(ref args) = call.arguments {
                        let mut first = true;
                        for &arg_idx in &args.nodes {
                            if !first { self.write(", "); }
                            first = false;
                            self.emit_expression(arg_idx);
                        }
                    }
                    self.write(")");
                }
            }
            k if k == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION => {
                if let Some(access) = self.arena.get_access_expr(expr_node) {
                    self.emit_expression(access.expression);
                    self.write(".");
                    self.emit_expression(access.name_or_argument);
                }
            }
            _ => {}
        }
    }
    
    fn emit_operator_token(&mut self, op: u16) {
        let op_str = match op {
            k if k == SyntaxKind::PlusToken as u16 => "+",
            k if k == SyntaxKind::MinusToken as u16 => "-",
            k if k == SyntaxKind::AsteriskToken as u16 => "*",
            k if k == SyntaxKind::SlashToken as u16 => "/",
            k if k == SyntaxKind::EqualsToken as u16 => "=",
            k if k == SyntaxKind::EqualsEqualsToken as u16 => "==",
            k if k == SyntaxKind::EqualsEqualsEqualsToken as u16 => "===",
            k if k == SyntaxKind::ExclamationEqualsToken as u16 => "!=",
            k if k == SyntaxKind::ExclamationEqualsEqualsToken as u16 => "!==",
            k if k == SyntaxKind::LessThanToken as u16 => "<",
            k if k == SyntaxKind::GreaterThanToken as u16 => ">",
            k if k == SyntaxKind::LessThanEqualsToken as u16 => "<=",
            k if k == SyntaxKind::GreaterThanEqualsToken as u16 => ">=",
            k if k == SyntaxKind::PlusEqualsToken as u16 => "+=",
            k if k == SyntaxKind::MinusEqualsToken as u16 => "-=",
            k if k == SyntaxKind::AsteriskEqualsToken as u16 => "*=",
            k if k == SyntaxKind::SlashEqualsToken as u16 => "/=",
            k if k == SyntaxKind::AmpersandAmpersandToken as u16 => "&&",
            k if k == SyntaxKind::BarBarToken as u16 => "||",
            _ => "?"
        };
        self.write(op_str);
    }
    
    fn get_identifier_text(&self, idx: NodeIndex) -> String {
        if let Some(node) = self.arena.get(idx) {
            if let Some(ident) = self.arena.get_identifier(node) {
                return ident.escaped_text.clone();
            }
        }
        String::new()
    }
    
    fn has_export_modifier(&self, modifiers: &Option<NodeList>) -> bool {
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.arena.get(mod_idx) {
                    if mod_node.kind == SyntaxKind::ExportKeyword as u16 {
                        return true;
                    }
                }
            }
        }
        false
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
    
    fn emit_namespace(source: &str) -> String {
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        
        // Find the namespace declaration
        if let Some(root_node) = parser.arena.get(root) {
            if let Some(source_file) = parser.arena.get_source_file(root_node) {
                if let Some(&ns_idx) = source_file.statements.nodes.first() {
                    let mut emitter = NamespaceES5Emitter::new(&parser.arena);
                    return emitter.emit_namespace(ns_idx);
                }
            }
        }
        String::new()
    }
    
    #[test]
    fn test_empty_namespace() {
        let output = emit_namespace("namespace M { }");
        assert!(output.contains("var M;"), "Should declare var M");
        assert!(output.contains("(function (M)"), "Should have IIFE");
        assert!(output.contains("(M || (M = {}))"), "Should have M || (M = {{}})");
    }
    
    #[test]
    fn test_namespace_with_function() {
        let output = emit_namespace("namespace M { export function foo() { return 1; } }");
        assert!(output.contains("var M;"), "Should declare var M");
        assert!(output.contains("function foo()"), "Should have function foo");
        assert!(output.contains("M.foo = foo;"), "Should export foo");
    }
}
