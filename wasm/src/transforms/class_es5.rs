//! ES5 Class Transform
//!
//! Transforms ES6 classes to ES5 IIFE patterns:
//!
//! ```typescript
//! class Animal {
//!     constructor(name) { this.name = name; }
//!     speak() { console.log(this.name); }
//! }
//! ```
//!
//! Becomes:
//!
//! ```javascript
//! var Animal = /** @class */ (function () {
//!     function Animal(name) {
//!         this.name = name;
//!     }
//!     Animal.prototype.speak = function () {
//!         console.log(this.name);
//!     };
//!     return Animal;
//! }());
//! ```

use crate::parser::thin_node::{ThinNode, ThinNodeArena, FunctionData, ClassData, MethodDeclData, PropertyDeclData};
use crate::parser::{NodeIndex, NodeList};
use crate::parser::syntax_kind_ext;
use crate::scanner::SyntaxKind;

/// ES5 class emitter - emits ES5 IIFE pattern for classes
pub struct ClassES5Emitter<'a> {
    arena: &'a ThinNodeArena,
    output: String,
    indent_level: u32,
    source_text: Option<&'a str>,
}

impl<'a> ClassES5Emitter<'a> {
    pub fn new(arena: &'a ThinNodeArena) -> Self {
        ClassES5Emitter {
            arena,
            output: String::with_capacity(4096),
            indent_level: 0,
            source_text: None,
        }
    }

    /// Set the initial indentation level (to match the parent context)
    pub fn set_indent_level(&mut self, level: u32) {
        self.indent_level = level;
    }

    /// Set the source text (for single-line block detection)
    pub fn set_source_text(&mut self, source_text: &'a str) {
        self.source_text = Some(source_text);
    }

    pub fn emit_class(&mut self, class_idx: NodeIndex) -> String {
        self.output.clear();
        
        let Some(class_node) = self.arena.get(class_idx) else {
            return String::new();
        };
        
        let Some(class_data) = self.arena.get_class(class_node) else {
            return String::new();
        };
        
        // Get class name
        let class_name = self.get_identifier_text(class_data.name);

        // Check for extends clause and get base class name
        let base_class_name = self.get_extends_class_name(&class_data.heritage_clauses);
        let has_extends = base_class_name.is_some();

        // var ClassName = /** @class */ (function (_super) {
        self.write("var ");
        self.write(&class_name);
        self.write(" = /** @class */ (function (");
        if has_extends {
            self.write("_super");
        }
        self.write(") {");
        self.write_line();
        self.increase_indent();

        // __extends(ClassName, _super);
        if has_extends {
            self.write_indent();
            self.write("__extends(");
            self.write(&class_name);
            self.write(", _super);");
            self.write_line();
        }

        // Constructor function
        self.emit_constructor(&class_name, class_data, has_extends);

        // Prototype methods
        self.emit_methods(&class_name, class_data);

        // Static members
        self.emit_static_members(&class_name, class_data);

        // return ClassName;
        self.write_indent();
        self.write("return ");
        self.write(&class_name);
        self.write(";");
        self.write_line();

        self.decrease_indent();
        self.write_indent();
        self.write("}(");

        // Pass base class if extends
        if let Some(ref base_name) = base_class_name {
            self.write(base_name);
        }

        self.write("));");

        std::mem::take(&mut self.output)
    }
    
    fn emit_constructor(&mut self, class_name: &str, class_data: &ClassData, has_extends: bool) {
        // Collect instance property initializers
        let instance_props: Vec<NodeIndex> = class_data.members.nodes.iter()
            .filter_map(|&member_idx| {
                let member_node = self.arena.get(member_idx)?;
                if member_node.kind != syntax_kind_ext::PROPERTY_DECLARATION {
                    return None;
                }
                let prop_data = self.arena.get_property_decl(member_node)?;
                // Skip static properties
                if self.is_static(&prop_data.modifiers) {
                    return None;
                }
                // Include if has initializer
                if !prop_data.initializer.is_none() {
                    Some(member_idx)
                } else {
                    None
                }
            })
            .collect();

        // Find constructor implementation (the one with a body)
        // Skip declaration-only constructors (overload signatures)
        let mut found_constructor = false;

        for &member_idx in &class_data.members.nodes {
            let Some(member_node) = self.arena.get(member_idx) else { continue };

            if member_node.kind == syntax_kind_ext::CONSTRUCTOR {
                let Some(ctor_data) = self.arena.get_constructor(member_node) else { continue };

                // Only emit the constructor implementation (with a body), not overload signatures
                if ctor_data.body.is_none() {
                    continue;
                }

                found_constructor = true;

                self.write_indent();
                self.write("function ");
                self.write(class_name);
                self.write("(");
                self.emit_parameters(&ctor_data.parameters);
                self.write(") {");
                self.write_line();
                self.increase_indent();

                // For derived classes with explicit constructor:
                // 1. Transform super(args) to var _this = _super.call(this, args) || this;
                // 2. Use _this instead of this for property assignments
                // 3. Add return _this; at the end
                if has_extends {
                    self.emit_derived_constructor_body(ctor_data.body, &ctor_data.parameters, &instance_props);
                } else {
                    // Non-derived class: emit instance props and parameter props first
                    self.emit_instance_property_initializers(&instance_props);
                    self.emit_parameter_properties(&ctor_data.parameters);
                    self.emit_block_contents(ctor_data.body);
                }

                self.decrease_indent();
                self.write_indent();
                self.write("}");
                self.write_line();
                break;
            }
        }

        // Default constructor if none found
        if !found_constructor {
            self.write_indent();
            self.write("function ");
            self.write(class_name);
            self.write("(");

            // For derived classes without explicit constructor, accept variable args
            if has_extends {
                // No explicit params needed since we'll use arguments
            }

            self.write(") {");
            self.write_line();
            self.increase_indent();

            // For derived classes with no instance properties, just return _super.apply directly
            if has_extends && instance_props.is_empty() {
                self.write_indent();
                self.write("return _super !== null && _super.apply(this, arguments) || this;");
                self.write_line();
            } else if has_extends {
                // For derived classes with instance props, use _this variable
                self.write_indent();
                self.write("var _this = _super !== null && _super.apply(this, arguments) || this;");
                self.write_line();

                // Emit instance property initializers
                for &prop_idx in &instance_props {
                    let Some(prop_node) = self.arena.get(prop_idx) else { continue };
                    let Some(prop_data) = self.arena.get_property_decl(prop_node) else { continue };
                    let name = self.get_identifier_text(prop_data.name);
                    self.write_indent();
                    self.write("_this.");
                    self.write(&name);
                    self.write(" = ");
                    self.emit_expression(prop_data.initializer);
                    self.write(";");
                    self.write_line();
                }

                // Return _this
                self.write_indent();
                self.write("return _this;");
                self.write_line();
            } else {
                // Non-derived class - just emit instance property initializers
                for &prop_idx in &instance_props {
                    let Some(prop_node) = self.arena.get(prop_idx) else { continue };
                    let Some(prop_data) = self.arena.get_property_decl(prop_node) else { continue };
                    let name = self.get_identifier_text(prop_data.name);
                    self.write_indent();
                    self.write("this.");
                    self.write(&name);
                    self.write(" = ");
                    self.emit_expression(prop_data.initializer);
                    self.write(";");
                    self.write_line();
                }
            }

            self.decrease_indent();
            self.write_indent();
            self.write("}");
            self.write_line();
        }
    }

    /// Emit instance property initializers as this.prop = value;
    fn emit_instance_property_initializers(&mut self, props: &[NodeIndex]) {
        for &prop_idx in props {
            let Some(prop_node) = self.arena.get(prop_idx) else { continue };
            let Some(prop_data) = self.arena.get_property_decl(prop_node) else { continue };

            let prop_name = self.get_identifier_text(prop_data.name);

            self.write_indent();
            self.write("this.");
            self.write(&prop_name);
            self.write(" = ");
            self.emit_expression(prop_data.initializer);
            self.write(";");
            self.write_line();
        }
    }

    /// Emit parameter properties as this.param = param;
    /// For constructor parameters with public, private, protected, or readonly modifiers
    fn emit_parameter_properties(&mut self, params: &NodeList) {
        for &param_idx in &params.nodes {
            let Some(param_node) = self.arena.get(param_idx) else { continue };
            let Some(param) = self.arena.get_parameter(param_node) else { continue };

            // Check for modifiers that trigger property creation
            if self.has_parameter_property_modifier(&param.modifiers) {
                let name = self.get_identifier_text(param.name);

                if !name.is_empty() {
                    self.write_indent();
                    self.write("this.");
                    self.write(&name);
                    self.write(" = ");
                    self.write(&name);
                    self.write(";");
                    self.write_line();
                }
            }
        }
    }

    /// Check if parameter has a modifier that makes it a property (public, private, protected, readonly)
    fn has_parameter_property_modifier(&self, modifiers: &Option<NodeList>) -> bool {
        let Some(mods) = modifiers else {
            return false;
        };
        for &mod_idx in &mods.nodes {
            let Some(mod_node) = self.arena.get(mod_idx) else { continue };
            match mod_node.kind {
                k if k == SyntaxKind::PublicKeyword as u16
                    || k == SyntaxKind::PrivateKeyword as u16
                    || k == SyntaxKind::ProtectedKeyword as u16
                    || k == SyntaxKind::ReadonlyKeyword as u16 =>
                {
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    /// Emit derived class constructor body with super() transformation
    /// - Transform super(args) to var _this = _super.call(this, args) || this;
    /// - Use _this for parameter properties
    /// - Add return _this; at the end
    fn emit_derived_constructor_body(
        &mut self,
        body_idx: NodeIndex,
        params: &NodeList,
        instance_props: &[NodeIndex],
    ) {
        let Some(body_node) = self.arena.get(body_idx) else { return };
        let Some(block) = self.arena.get_block(body_node) else { return };

        // First, find and emit the super() call as _super.call(this, ...)
        let mut found_super = false;
        for &stmt_idx in &block.statements.nodes {
            if self.is_super_call_statement(stmt_idx) {
                self.emit_super_call_as_this_assignment(stmt_idx);
                found_super = true;
                break;
            }
        }

        // Emit parameter properties using _this
        for &param_idx in &params.nodes {
            let Some(param_node) = self.arena.get(param_idx) else { continue };
            let Some(param) = self.arena.get_parameter(param_node) else { continue };

            if self.has_parameter_property_modifier(&param.modifiers) {
                let name = self.get_identifier_text(param.name);
                if !name.is_empty() {
                    self.write_indent();
                    self.write("_this.");
                    self.write(&name);
                    self.write(" = ");
                    self.write(&name);
                    self.write(";");
                    self.write_line();
                }
            }
        }

        // Emit remaining statements (after super call), transforming this to _this
        let mut past_super = false;
        for &stmt_idx in &block.statements.nodes {
            if !past_super && self.is_super_call_statement(stmt_idx) {
                past_super = true;
                continue; // Skip the super call, already emitted
            }
            if past_super {
                self.write_indent();
                self.emit_statement_with_this_transform(stmt_idx);
                self.write_line();
            }
        }

        // Add return _this;
        if found_super {
            self.write_indent();
            self.write("return _this;");
            self.write_line();
        }
    }

    /// Check if a statement is a super() call expression
    fn is_super_call_statement(&self, stmt_idx: NodeIndex) -> bool {
        let Some(stmt_node) = self.arena.get(stmt_idx) else { return false };

        if stmt_node.kind != syntax_kind_ext::EXPRESSION_STATEMENT {
            return false;
        }

        let Some(expr_stmt) = self.arena.get_expression_statement(stmt_node) else { return false };
        let Some(call_node) = self.arena.get(expr_stmt.expression) else { return false };

        if call_node.kind != syntax_kind_ext::CALL_EXPRESSION {
            return false;
        }

        let Some(call) = self.arena.get_call_expr(call_node) else { return false };
        let Some(callee) = self.arena.get(call.expression) else { return false };

        callee.kind == SyntaxKind::SuperKeyword as u16
    }

    /// Emit super(args) as var _this = _super.call(this, args) || this;
    fn emit_super_call_as_this_assignment(&mut self, stmt_idx: NodeIndex) {
        let Some(stmt_node) = self.arena.get(stmt_idx) else { return };
        let Some(expr_stmt) = self.arena.get_expression_statement(stmt_node) else { return };
        let Some(call_node) = self.arena.get(expr_stmt.expression) else { return };
        let Some(call) = self.arena.get_call_expr(call_node) else { return };

        self.write_indent();
        self.write("var _this = _super.call(this");

        // Emit arguments
        if let Some(ref args) = call.arguments {
            for &arg_idx in &args.nodes {
                self.write(", ");
                self.emit_expression(arg_idx);
            }
        }

        self.write(") || this;");
        self.write_line();
    }

    /// Emit a statement, but transform `this` references to `_this`
    fn emit_statement_with_this_transform(&mut self, stmt_idx: NodeIndex) {
        // For now, just delegate to regular emit
        // TODO: Implement proper this->_this transformation
        self.emit_statement(stmt_idx);
    }

    fn emit_methods(&mut self, class_name: &str, class_data: &ClassData) {
        // First, collect accessors by name for combining getter/setter pairs
        let mut accessor_map: std::collections::HashMap<String, (Option<NodeIndex>, Option<NodeIndex>, bool)> =
            std::collections::HashMap::new();

        for &member_idx in &class_data.members.nodes {
            let Some(member_node) = self.arena.get(member_idx) else { continue };

            if member_node.kind == syntax_kind_ext::GET_ACCESSOR {
                if let Some(accessor_data) = self.arena.get_accessor(member_node) {
                    let is_static = self.is_static(&accessor_data.modifiers);
                    // Skip static accessors (handled in emit_static_members)
                    if is_static {
                        continue;
                    }
                    let name = self.get_identifier_text(accessor_data.name);
                    let entry = accessor_map.entry(name).or_insert((None, None, is_static));
                    entry.0 = Some(member_idx);
                }
            } else if member_node.kind == syntax_kind_ext::SET_ACCESSOR {
                if let Some(accessor_data) = self.arena.get_accessor(member_node) {
                    let is_static = self.is_static(&accessor_data.modifiers);
                    // Skip static accessors (handled in emit_static_members)
                    if is_static {
                        continue;
                    }
                    let name = self.get_identifier_text(accessor_data.name);
                    let entry = accessor_map.entry(name).or_insert((None, None, is_static));
                    entry.1 = Some(member_idx);
                }
            }
        }

        // Now emit methods (non-accessors)
        for &member_idx in &class_data.members.nodes {
            let Some(member_node) = self.arena.get(member_idx) else { continue };

            if member_node.kind == syntax_kind_ext::METHOD_DECLARATION {
                let Some(method_data) = self.arena.get_method_decl(member_node) else { continue };

                // Skip static methods (handled separately)
                if self.is_static(&method_data.modifiers) {
                    continue;
                }

                // Skip if no body (declaration only)
                if method_data.body.is_none() {
                    continue;
                }

                let use_bracket = !self.is_valid_identifier_name(method_data.name);
                let method_name = if use_bracket {
                    self.get_computed_property_name(method_data.name)
                } else {
                    self.get_identifier_text(method_data.name)
                };

                // ClassName.prototype.methodName = function () { ... };
                // or ClassName.prototype[1] = function () { ... };
                // or ClassName.prototype["bar"] = function () { ... };
                self.write_indent();
                self.write(class_name);
                self.write(".prototype");
                if use_bracket {
                    self.write("[");
                    self.write(&method_name);
                    self.write("]");
                } else {
                    self.write(".");
                    self.write(&method_name);
                }
                self.write(" = function (");
                self.emit_parameters(&method_data.parameters);
                self.write(") ");

                // Check if body is empty - only empty bodies go on single line
                let body_node = self.arena.get(method_data.body);
                let is_empty_body = if let Some(block_node) = body_node {
                    if let Some(block) = self.arena.get_block(block_node) {
                        block.statements.nodes.is_empty()
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_empty_body {
                    self.write("{ }");
                } else {
                    self.write("{");
                    self.write_line();
                    self.increase_indent();
                    self.emit_block_contents(method_data.body);
                    self.decrease_indent();
                    self.write_indent();
                    self.write("}");
                }

                self.write(";");
                self.write_line();
            }
        }

        // Now emit combined accessors
        for (name, (getter_idx, setter_idx, is_static)) in accessor_map {
            self.emit_combined_accessor(class_name, &name, getter_idx, setter_idx, is_static);
        }
    }
    
    /// Emit a combined Object.defineProperty for getter/setter pairs
    fn emit_combined_accessor(
        &mut self,
        class_name: &str,
        name: &str,
        getter_idx: Option<NodeIndex>,
        setter_idx: Option<NodeIndex>,
        is_static: bool,
    ) {
        // Object.defineProperty(ClassName.prototype, "name", { get: ..., set: ..., ... })
        self.write_indent();
        self.write("Object.defineProperty(");
        self.write(class_name);
        if !is_static {
            self.write(".prototype");
        }
        self.write(", \"");
        self.write(name);
        self.write("\", {");
        self.write_line();
        self.increase_indent();

        // Emit getter if present
        if let Some(getter_idx) = getter_idx {
            self.emit_accessor_function(getter_idx, true);
        }

        // Emit setter if present
        if let Some(setter_idx) = setter_idx {
            self.emit_accessor_function(setter_idx, false);
        }

        self.write_indent();
        self.write("enumerable: false,");
        self.write_line();
        self.write_indent();
        self.write("configurable: true");
        self.write_line();

        self.decrease_indent();
        self.write_indent();
        self.write("});");
        self.write_line();
    }

    /// Emit just the function part of an accessor (get: function () {...}, or set: function (v) {...},)
    fn emit_accessor_function(&mut self, accessor_idx: NodeIndex, is_getter: bool) {
        let Some(accessor_node) = self.arena.get(accessor_idx) else { return };
        let Some(accessor_data) = self.arena.get_accessor(accessor_node) else { return };

        // Check if accessor body is empty
        let (body_is_empty, body_is_single_line) = if !accessor_data.body.is_none() {
            let body_node = self.arena.get(accessor_data.body);
            let is_empty = body_node.map_or(true, |n| {
                self.arena.get_block(n).map_or(true, |b| b.statements.nodes.is_empty())
            });
            let is_single_line = body_node.map_or(false, |n| self.is_single_line_block(n));
            (is_empty, is_single_line)
        } else {
            (true, false)
        };

        self.write_indent();
        if is_getter {
            self.write("get: function () ");
        } else {
            self.write("set: function (");
            self.emit_parameters(&accessor_data.parameters);
            self.write(") ");
        }

        if body_is_empty {
            // Inline empty body: { },
            self.write("{ },");
        } else if body_is_single_line {
            // Single-line body: { return 1; },
            self.write("{ ");
            self.emit_block_contents_inline(accessor_data.body);
            self.write(" },");
        } else {
            // Multi-line body
            self.write("{");
            self.write_line();
            self.increase_indent();
            self.emit_block_contents(accessor_data.body);
            self.decrease_indent();
            self.write_indent();
            self.write("},");
        }
        self.write_line();
    }

    /// Check if a block was on a single line in the source
    fn is_single_line_block(&self, block_node: &ThinNode) -> bool {
        if let Some(source_text) = self.source_text {
            let start = block_node.pos as usize;
            let end = block_node.end as usize;
            if start < end && end <= source_text.len() {
                // The block end position may be incorrect, so find the matching }
                // from the start position
                let block_text = &source_text[start..end];
                // Find the first } which closes the block
                if let Some(close_brace_pos) = block_text.find('}') {
                    let actual_block = &block_text[..=close_brace_pos];
                    // A single-line block has no newlines between { and }
                    !actual_block.contains('\n')
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Emit block contents inline (for single-line blocks)
    fn emit_block_contents_inline(&mut self, body_idx: NodeIndex) {
        let Some(body_node) = self.arena.get(body_idx) else { return };
        let Some(block) = self.arena.get_block(body_node) else { return };

        for (i, &stmt_idx) in block.statements.nodes.iter().enumerate() {
            if i > 0 {
                self.write(" ");
            }
            self.emit_statement_inline(stmt_idx);
        }
    }

    /// Emit a statement inline (without newlines/indentation)
    fn emit_statement_inline(&mut self, stmt_idx: NodeIndex) {
        let Some(stmt_node) = self.arena.get(stmt_idx) else { return };

        if stmt_node.kind == syntax_kind_ext::RETURN_STATEMENT {
            self.write("return");
            if let Some(ret_data) = self.arena.get_return_statement(stmt_node) {
                if !ret_data.expression.is_none() {
                    self.write(" ");
                    self.emit_expression(ret_data.expression);
                }
            }
            self.write(";");
        } else {
            // Fallback: emit statement normally but it might not look right
            self.emit_statement(stmt_idx);
        }
    }

    fn emit_accessor(&mut self, class_name: &str, accessor_idx: NodeIndex, is_getter: bool) {
        let Some(accessor_node) = self.arena.get(accessor_idx) else { return };
        let Some(accessor_data) = self.arena.get_accessor(accessor_node) else { return };

        let is_static = self.is_static(&accessor_data.modifiers);
        let name = self.get_identifier_text(accessor_data.name);

        // Use combined accessor for single getter or setter
        if is_getter {
            self.emit_combined_accessor(class_name, &name, Some(accessor_idx), None, is_static);
        } else {
            self.emit_combined_accessor(class_name, &name, None, Some(accessor_idx), is_static);
        }
    }
    
    fn emit_static_members(&mut self, class_name: &str, class_data: &ClassData) {
        // First, collect static accessors by name for combining getter/setter pairs
        let mut static_accessor_map: std::collections::HashMap<String, (Option<NodeIndex>, Option<NodeIndex>)> =
            std::collections::HashMap::new();

        for &member_idx in &class_data.members.nodes {
            let Some(member_node) = self.arena.get(member_idx) else { continue };

            if member_node.kind == syntax_kind_ext::GET_ACCESSOR {
                if let Some(accessor_data) = self.arena.get_accessor(member_node) {
                    if self.is_static(&accessor_data.modifiers) {
                        let name = self.get_identifier_text(accessor_data.name);
                        let entry = static_accessor_map.entry(name).or_insert((None, None));
                        entry.0 = Some(member_idx);
                    }
                }
            } else if member_node.kind == syntax_kind_ext::SET_ACCESSOR {
                if let Some(accessor_data) = self.arena.get_accessor(member_node) {
                    if self.is_static(&accessor_data.modifiers) {
                        let name = self.get_identifier_text(accessor_data.name);
                        let entry = static_accessor_map.entry(name).or_insert((None, None));
                        entry.1 = Some(member_idx);
                    }
                }
            }
        }

        // Emit static methods and properties
        for &member_idx in &class_data.members.nodes {
            let Some(member_node) = self.arena.get(member_idx) else { continue };

            if member_node.kind == syntax_kind_ext::METHOD_DECLARATION {
                let Some(method_data) = self.arena.get_method_decl(member_node) else { continue };

                if !self.is_static(&method_data.modifiers) {
                    continue;
                }

                if method_data.body.is_none() {
                    continue;
                }

                let method_name = self.get_identifier_text(method_data.name);

                // ClassName.staticMethod = function () { ... };
                self.write_indent();
                self.write(class_name);
                self.write(".");
                self.write(&method_name);
                self.write(" = function (");
                self.emit_parameters(&method_data.parameters);
                self.write(") {");
                self.write_line();
                self.increase_indent();

                self.emit_block_contents(method_data.body);

                self.decrease_indent();
                self.write_indent();
                self.write("};");
                self.write_line();
            } else if member_node.kind == syntax_kind_ext::PROPERTY_DECLARATION {
                let Some(prop_data) = self.arena.get_property_decl(member_node) else { continue };

                if !self.is_static(&prop_data.modifiers) {
                    continue;
                }

                if prop_data.initializer.is_none() {
                    continue;
                }

                let prop_name = self.get_identifier_text(prop_data.name);

                // ClassName.staticProp = value;
                self.write_indent();
                self.write(class_name);
                self.write(".");
                self.write(&prop_name);
                self.write(" = ");
                self.emit_expression(prop_data.initializer);
                self.write(";");
                self.write_line();
            }
        }

        // Emit combined static accessors
        for (name, (getter_idx, setter_idx)) in static_accessor_map {
            self.emit_combined_accessor(class_name, &name, getter_idx, setter_idx, true);
        }
    }
    
    fn emit_parameters(&mut self, params: &NodeList) {
        let mut first = true;
        for &param_idx in &params.nodes {
            if !first {
                self.write(", ");
            }
            first = false;
            
            if let Some(param_node) = self.arena.get(param_idx) {
                if let Some(param_data) = self.arena.get_parameter(param_node) {
                    if param_data.dot_dot_dot_token {
                        // Rest parameter - we'd need to transform this for ES5
                        // For now, just emit the name
                    }
                    self.emit_binding_name(param_data.name);
                }
            }
        }
    }
    
    fn emit_binding_name(&mut self, name_idx: NodeIndex) {
        let Some(name_node) = self.arena.get(name_idx) else { return };
        
        if let Some(ident) = self.arena.get_identifier(name_node) {
            self.write(&ident.escaped_text);
        }
        // TODO: Handle destructuring patterns
    }
    
    fn emit_block_contents(&mut self, block_idx: NodeIndex) {
        let Some(block_node) = self.arena.get(block_idx) else { return };
        
        if let Some(block_data) = self.arena.get_block(block_node) {
            for &stmt_idx in &block_data.statements.nodes {
                self.write_indent();
                self.emit_statement(stmt_idx);
                self.write_line();
            }
        }
    }
    
    fn emit_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(stmt_node) = self.arena.get(stmt_idx) else { return };
        
        match stmt_node.kind {
            k if k == syntax_kind_ext::EXPRESSION_STATEMENT => {
                if let Some(expr_stmt) = self.arena.get_expression_statement(stmt_node) {
                    self.emit_expression(expr_stmt.expression);
                    self.write(";");
                }
            }
            k if k == syntax_kind_ext::RETURN_STATEMENT => {
                if let Some(ret) = self.arena.get_return_statement(stmt_node) {
                    self.write("return");
                    if !ret.expression.is_none() {
                        self.write(" ");
                        self.emit_expression(ret.expression);
                    }
                    self.write(";");
                }
            }
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                self.emit_variable_statement(stmt_idx);
            }
            k if k == syntax_kind_ext::IF_STATEMENT => {
                self.emit_if_statement(stmt_idx);
            }
            k if k == syntax_kind_ext::BLOCK => {
                self.write("{");
                self.write_line();
                self.increase_indent();
                self.emit_block_contents(stmt_idx);
                self.decrease_indent();
                self.write_indent();
                self.write("}");
            }
            k if k == syntax_kind_ext::FOR_STATEMENT => {
                self.emit_for_statement(stmt_idx);
            }
            k if k == syntax_kind_ext::WHILE_STATEMENT => {
                self.emit_while_statement(stmt_idx);
            }
            k if k == syntax_kind_ext::THROW_STATEMENT => {
                // TODO: Implement throw statement when API available
                self.write("throw /* TODO */;");
            }
            k if k == syntax_kind_ext::TRY_STATEMENT => {
                // TODO: Implement try statement
                self.write("try { /* TODO */ }");
            }
            _ => {
                // Fallback: emit expression if possible
                self.emit_expression(stmt_idx);
                self.write(";");
            }
        }
    }
    
    fn emit_variable_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(stmt_node) = self.arena.get(stmt_idx) else { return };
        let Some(var_stmt) = self.arena.get_variable(stmt_node) else { return };
        
        self.write("var ");
        
        let mut first = true;
        for &decl_list_idx in &var_stmt.declarations.nodes {
            let Some(decl_list_node) = self.arena.get(decl_list_idx) else { continue };
            
            if decl_list_node.kind == syntax_kind_ext::VARIABLE_DECLARATION_LIST {
                if let Some(decl_list) = self.arena.get_variable(decl_list_node) {
                    for &decl_idx in &decl_list.declarations.nodes {
                        if !first {
                            self.write(", ");
                        }
                        first = false;
                        self.emit_variable_declaration(decl_idx);
                    }
                }
            } else {
                // Single declaration
                if !first {
                    self.write(", ");
                }
                first = false;
                self.emit_variable_declaration(decl_list_idx);
            }
        }
        self.write(";");
    }
    
    fn emit_variable_declaration(&mut self, decl_idx: NodeIndex) {
        let Some(decl_node) = self.arena.get(decl_idx) else { return };
        let Some(decl) = self.arena.get_variable_declaration(decl_node) else { return };
        
        self.emit_binding_name(decl.name);
        
        if !decl.initializer.is_none() {
            self.write(" = ");
            self.emit_expression(decl.initializer);
        }
    }
    
    fn emit_if_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(stmt_node) = self.arena.get(stmt_idx) else { return };
        let Some(if_stmt) = self.arena.get_if_statement(stmt_node) else { return };
        
        self.write("if (");
        self.emit_expression(if_stmt.expression);
        self.write(") ");
        self.emit_statement(if_stmt.then_statement);
        
        if !if_stmt.else_statement.is_none() {
            self.write_line();
            self.write_indent();
            self.write("else ");
            self.emit_statement(if_stmt.else_statement);
        }
    }
    
    fn emit_for_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(stmt_node) = self.arena.get(stmt_idx) else { return };
        let Some(for_stmt) = self.arena.get_loop(stmt_node) else { return };
        
        self.write("for (");
        if !for_stmt.initializer.is_none() {
            // Check if it's a variable declaration list
            if let Some(init_node) = self.arena.get(for_stmt.initializer) {
                if init_node.kind == syntax_kind_ext::VARIABLE_DECLARATION_LIST {
                    self.write("var ");
                    if let Some(decl_list) = self.arena.get_variable(init_node) {
                        let mut first = true;
                        for &decl_idx in &decl_list.declarations.nodes {
                            if !first { self.write(", "); }
                            first = false;
                            self.emit_variable_declaration(decl_idx);
                        }
                    }
                } else {
                    self.emit_expression(for_stmt.initializer);
                }
            }
        }
        self.write("; ");
        if !for_stmt.condition.is_none() {
            self.emit_expression(for_stmt.condition);
        }
        self.write("; ");
        if !for_stmt.incrementor.is_none() {
            self.emit_expression(for_stmt.incrementor);
        }
        self.write(") ");
        self.emit_statement(for_stmt.statement);
    }
    
    fn emit_while_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(stmt_node) = self.arena.get(stmt_idx) else { return };
        let Some(while_stmt) = self.arena.get_loop(stmt_node) else { return };
        
        self.write("while (");
        self.emit_expression(while_stmt.condition);
        self.write(") ");
        self.emit_statement(while_stmt.statement);
    }
    
    fn emit_try_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(stmt_node) = self.arena.get(stmt_idx) else { return };
        let Some(try_stmt) = self.arena.get_try(stmt_node) else { return };
        
        self.write("try ");
        self.emit_statement(try_stmt.try_block);
        
        if !try_stmt.catch_clause.is_none() {
            self.write_line();
            self.write_indent();
            if let Some(catch_node) = self.arena.get(try_stmt.catch_clause) {
                if let Some(catch_data) = self.arena.get_catch_clause(catch_node) {
                    self.write("catch (");
                    self.emit_binding_name(catch_data.variable_declaration);
                    self.write(") ");
                    self.emit_statement(catch_data.block);
                }
            }
        }
        
        if !try_stmt.finally_block.is_none() {
            self.write_line();
            self.write_indent();
            self.write("finally ");
            self.emit_statement(try_stmt.finally_block);
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
            k if k == SyntaxKind::TrueKeyword as u16 => self.write("true"),
            k if k == SyntaxKind::FalseKeyword as u16 => self.write("false"),
            k if k == SyntaxKind::NullKeyword as u16 => self.write("null"),
            k if k == SyntaxKind::ThisKeyword as u16 => self.write("this"),
            k if k == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION => {
                if let Some(access) = self.arena.get_access_expr(expr_node) {
                    self.emit_expression(access.expression);
                    self.write(".");
                    self.emit_expression(access.name_or_argument);
                }
            }
            k if k == syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION => {
                if let Some(access) = self.arena.get_access_expr(expr_node) {
                    self.emit_expression(access.expression);
                    self.write("[");
                    self.emit_expression(access.name_or_argument);
                    self.write("]");
                }
            }
            k if k == syntax_kind_ext::CALL_EXPRESSION => {
                if let Some(call) = self.arena.get_call_expr(expr_node) {
                    // Check if this is super.method(args) - transform to _super.prototype.method.call(this, args)
                    if self.is_super_method_call(call.expression) {
                        self.emit_super_method_call(call.expression, &call.arguments);
                    } else {
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
            }
            k if k == syntax_kind_ext::NEW_EXPRESSION => {
                if let Some(call) = self.arena.get_call_expr(expr_node) {
                    self.write("new ");
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
            k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                if let Some(bin) = self.arena.get_binary_expr(expr_node) {
                    self.emit_expression(bin.left);
                    self.write(" ");
                    self.emit_binary_operator(bin.operator_token);
                    self.write(" ");
                    self.emit_expression(bin.right);
                }
            }
            k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION => {
                if let Some(unary) = self.arena.get_unary_expr(expr_node) {
                    self.emit_prefix_operator(unary.operator);
                    self.emit_expression(unary.operand);
                }
            }
            k if k == syntax_kind_ext::POSTFIX_UNARY_EXPRESSION => {
                if let Some(unary) = self.arena.get_unary_expr(expr_node) {
                    self.emit_expression(unary.operand);
                    self.emit_postfix_operator(unary.operator);
                }
            }
            k if k == syntax_kind_ext::PARENTHESIZED_EXPRESSION => {
                // TODO: Implement parenthesized expression
                self.write("(/* TODO */)");
            }
            k if k == syntax_kind_ext::CONDITIONAL_EXPRESSION => {
                if let Some(cond) = self.arena.get_conditional_expr(expr_node) {
                    self.emit_expression(cond.condition);
                    self.write(" ? ");
                    self.emit_expression(cond.when_true);
                    self.write(" : ");
                    self.emit_expression(cond.when_false);
                }
            }
            k if k == syntax_kind_ext::ARRAY_LITERAL_EXPRESSION => {
                if let Some(arr) = self.arena.get_literal_expr(expr_node) {
                    self.write("[");
                    let mut first = true;
                    for &elem_idx in &arr.elements.nodes {
                        if !first { self.write(", "); }
                        first = false;
                        self.emit_expression(elem_idx);
                    }
                    self.write("]");
                }
            }
            k if k == syntax_kind_ext::OBJECT_LITERAL_EXPRESSION => {
                if let Some(obj) = self.arena.get_literal_expr(expr_node) {
                    self.write("{ ");
                    let mut first = true;
                    for &prop_idx in &obj.elements.nodes {
                        if !first { self.write(", "); }
                        first = false;
                        self.emit_object_property(prop_idx);
                    }
                    self.write(" }");
                }
            }
            k if k == syntax_kind_ext::ARROW_FUNCTION => {
                // Transform arrow to function expression
                if let Some(func) = self.arena.get_function(expr_node) {
                    self.write("function (");
                    self.emit_parameters(&func.parameters);
                    self.write(") ");
                    
                    // Check if body is an expression or block
                    if let Some(body_node) = self.arena.get(func.body) {
                        if body_node.kind == syntax_kind_ext::BLOCK {
                            self.emit_statement(func.body);
                        } else {
                            // Expression body - wrap in return
                            self.write("{ return ");
                            self.emit_expression(func.body);
                            self.write("; }");
                        }
                    }
                }
            }
            k if k == syntax_kind_ext::FUNCTION_EXPRESSION => {
                if let Some(func) = self.arena.get_function(expr_node) {
                    self.write("function");
                    if !func.name.is_none() {
                        self.write(" ");
                        self.emit_expression(func.name);
                    }
                    // Space before ( for TypeScript compatibility
                    self.write(" (");
                    self.emit_parameters(&func.parameters);
                    self.write(") ");
                    
                    // Check if body is a single return statement - emit on one line
                    let body_node = self.arena.get(func.body);
                    let is_simple_body = if let Some(block) = body_node.and_then(|n| self.arena.get_block(n)) {
                        block.statements.nodes.len() == 1 && {
                            let stmt_node = self.arena.get(block.statements.nodes[0]);
                            stmt_node.map(|s| s.kind == syntax_kind_ext::RETURN_STATEMENT).unwrap_or(false)
                        }
                    } else {
                        false
                    };
                    
                    if is_simple_body {
                        // Single-line: { return expr; }
                        if let Some(block_node) = body_node {
                            if let Some(block) = self.arena.get_block(block_node) {
                                self.write("{ ");
                                for &stmt_idx in &block.statements.nodes {
                                    self.emit_statement(stmt_idx);
                                }
                                self.write(" }");
                            }
                        }
                    } else {
                        self.emit_statement(func.body);
                    }
                }
            }
            _ => {
                // Unknown expression - try to get text from source
            }
        }
    }
    
    fn emit_object_property(&mut self, prop_idx: NodeIndex) {
        let Some(prop_node) = self.arena.get(prop_idx) else { return };
        
        if prop_node.kind == syntax_kind_ext::PROPERTY_ASSIGNMENT {
            if let Some(prop_data) = self.arena.get_property_assignment(prop_node) {
                self.emit_expression(prop_data.name);
                self.write(": ");
                self.emit_expression(prop_data.initializer);
            }
        } else if prop_node.kind == syntax_kind_ext::SHORTHAND_PROPERTY_ASSIGNMENT {
            if let Some(shorthand) = self.arena.get_shorthand_property(prop_node) {
                self.emit_expression(shorthand.name);
            } else if let Some(ident) = self.arena.get_identifier(prop_node) {
                self.write(&ident.escaped_text);
            }
        }
    }
    
    fn emit_binary_operator(&mut self, op: u16) {
        let op_str = match op {
            x if x == SyntaxKind::PlusToken as u16 => "+",
            x if x == SyntaxKind::MinusToken as u16 => "-",
            x if x == SyntaxKind::AsteriskToken as u16 => "*",
            x if x == SyntaxKind::SlashToken as u16 => "/",
            x if x == SyntaxKind::PercentToken as u16 => "%",
            x if x == SyntaxKind::EqualsToken as u16 => "=",
            x if x == SyntaxKind::EqualsEqualsToken as u16 => "==",
            x if x == SyntaxKind::EqualsEqualsEqualsToken as u16 => "===",
            x if x == SyntaxKind::ExclamationEqualsToken as u16 => "!=",
            x if x == SyntaxKind::ExclamationEqualsEqualsToken as u16 => "!==",
            x if x == SyntaxKind::LessThanToken as u16 => "<",
            x if x == SyntaxKind::LessThanEqualsToken as u16 => "<=",
            x if x == SyntaxKind::GreaterThanToken as u16 => ">",
            x if x == SyntaxKind::GreaterThanEqualsToken as u16 => ">=",
            x if x == SyntaxKind::AmpersandAmpersandToken as u16 => "&&",
            x if x == SyntaxKind::BarBarToken as u16 => "||",
            x if x == SyntaxKind::PlusEqualsToken as u16 => "+=",
            x if x == SyntaxKind::MinusEqualsToken as u16 => "-=",
            x if x == SyntaxKind::AsteriskEqualsToken as u16 => "*=",
            x if x == SyntaxKind::SlashEqualsToken as u16 => "/=",
            x if x == SyntaxKind::AmpersandToken as u16 => "&",
            x if x == SyntaxKind::BarToken as u16 => "|",
            x if x == SyntaxKind::CaretToken as u16 => "^",
            x if x == SyntaxKind::LessThanLessThanToken as u16 => "<<",
            x if x == SyntaxKind::GreaterThanGreaterThanToken as u16 => ">>",
            x if x == SyntaxKind::GreaterThanGreaterThanGreaterThanToken as u16 => ">>>",
            x if x == SyntaxKind::InKeyword as u16 => "in",
            x if x == SyntaxKind::InstanceOfKeyword as u16 => "instanceof",
            _ => "?",
        };
        self.write(op_str);
    }
    
    fn emit_prefix_operator(&mut self, op: u16) {
        let op_str = match op {
            x if x == SyntaxKind::PlusPlusToken as u16 => "++",
            x if x == SyntaxKind::MinusMinusToken as u16 => "--",
            x if x == SyntaxKind::ExclamationToken as u16 => "!",
            x if x == SyntaxKind::TildeToken as u16 => "~",
            x if x == SyntaxKind::PlusToken as u16 => "+",
            x if x == SyntaxKind::MinusToken as u16 => "-",
            x if x == SyntaxKind::TypeOfKeyword as u16 => "typeof ",
            x if x == SyntaxKind::VoidKeyword as u16 => "void ",
            x if x == SyntaxKind::DeleteKeyword as u16 => "delete ",
            _ => "",
        };
        self.write(op_str);
    }
    
    fn emit_postfix_operator(&mut self, op: u16) {
        let op_str = match op {
            x if x == SyntaxKind::PlusPlusToken as u16 => "++",
            x if x == SyntaxKind::MinusMinusToken as u16 => "--",
            _ => "",
        };
        self.write(op_str);
    }
    
    fn get_identifier_text(&self, idx: NodeIndex) -> String {
        if let Some(node) = self.arena.get(idx) {
            if let Some(ident) = self.arena.get_identifier(node) {
                return ident.escaped_text.clone();
            }
            // Handle numeric literals as property names
            if let Some(lit) = self.arena.get_literal(node) {
                return lit.text.clone();
            }
        }
        String::new()
    }
    
    /// Check if a name is a valid identifier (can use dot notation) or needs bracket notation
    fn is_valid_identifier_name(&self, idx: NodeIndex) -> bool {
        if let Some(node) = self.arena.get(idx) {
            // Identifiers use dot notation
            if self.arena.get_identifier(node).is_some() {
                return true;
            }
            // Numeric and string literals need bracket notation
            if node.kind == SyntaxKind::NumericLiteral as u16 {
                return false;
            }
            if node.kind == SyntaxKind::StringLiteral as u16 {
                return false;
            }
        }
        true // Default to dot notation
    }
    
    /// Get the property name for bracket notation (with quotes for strings)
    fn get_computed_property_name(&self, idx: NodeIndex) -> String {
        if let Some(node) = self.arena.get(idx) {
            // String literals need quotes in bracket notation
            if node.kind == SyntaxKind::StringLiteral as u16 {
                if let Some(lit) = self.arena.get_literal(node) {
                    return format!("\"{}\"", lit.text);
                }
            }
            // Other literals (numbers) are used as-is
            if let Some(lit) = self.arena.get_literal(node) {
                return lit.text.clone();
            }
            // Identifiers
            if let Some(ident) = self.arena.get_identifier(node) {
                return ident.escaped_text.clone();
            }
        }
        String::new()
    }
    
    fn is_static(&self, modifiers: &Option<NodeList>) -> bool {
        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = self.arena.get(mod_idx) {
                    if mod_node.kind == SyntaxKind::StaticKeyword as u16 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if heritage clauses contain an `extends` clause (not just `implements`)
    fn has_extends_clause(&self, heritage_clauses: &Option<NodeList>) -> bool {
        self.get_extends_class_name(heritage_clauses).is_some()
    }

    /// Get the base class name from the extends clause
    fn get_extends_class_name(&self, heritage_clauses: &Option<NodeList>) -> Option<String> {
        let clauses = heritage_clauses.as_ref()?;

        for &clause_idx in &clauses.nodes {
            let clause_node = self.arena.get(clause_idx)?;
            let heritage_data = self.arena.get_heritage(clause_node)?;

            // Check if this is an extends clause (not implements)
            if heritage_data.token != SyntaxKind::ExtendsKeyword as u16 {
                continue;
            }

            // Get the first type in the extends clause (the base class)
            let first_type_idx = heritage_data.types.nodes.first()?;
            let type_node = self.arena.get(*first_type_idx)?;

            // The type could be:
            // 1. A simple identifier (B in `extends B`)
            // 2. An ExpressionWithTypeArguments (B<T> in `extends B<T>`)
            // 3. A PropertyAccessExpression (A.B in `extends A.B`)

            // Try as simple identifier first
            if let Some(ident) = self.arena.get_identifier(type_node) {
                return Some(ident.escaped_text.clone());
            }

            // Try as ExpressionWithTypeArguments (for generics)
            if let Some(expr_data) = self.arena.get_expr_type_args(type_node) {
                return Some(self.get_identifier_text(expr_data.expression));
            }

            // For property access, just get the text (simplified - not handling A.B yet)
        }
        None
    }
    
    /// Check if expression is super.method (property access on super)
    fn is_super_method_call(&self, expr_idx: NodeIndex) -> bool {
        let Some(expr_node) = self.arena.get(expr_idx) else { return false };

        if expr_node.kind != syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION {
            return false;
        }

        let Some(access) = self.arena.get_access_expr(expr_node) else { return false };
        let Some(base_node) = self.arena.get(access.expression) else { return false };

        base_node.kind == SyntaxKind::SuperKeyword as u16
    }

    /// Emit super.method(args) as _super.prototype.method.call(this, args)
    fn emit_super_method_call(&mut self, callee_idx: NodeIndex, args: &Option<NodeList>) {
        let Some(callee_node) = self.arena.get(callee_idx) else { return };
        let Some(access) = self.arena.get_access_expr(callee_node) else { return };

        // Get method name
        let method_name = self.get_identifier_text(access.name_or_argument);

        // Emit _super.prototype.method.call(this, args)
        self.write("_super.prototype.");
        self.write(&method_name);
        self.write(".call(this");

        if let Some(arg_list) = args {
            for &arg_idx in &arg_list.nodes {
                self.write(", ");
                self.emit_expression(arg_idx);
            }
        }

        self.write(")");
    }

    // Helper methods
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

    #[test]
    fn test_simple_class_to_iife() {
        let source = r#"class Animal {
            constructor(name) {
                this.name = name;
            }
        }"#;
        
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        
        // Find the class declaration
        if let Some(root_node) = parser.arena.get(root) {
            if let Some(source_file) = parser.arena.get_source_file(root_node) {
                if let Some(&class_idx) = source_file.statements.nodes.first() {
                    let mut emitter = ClassES5Emitter::new(&parser.arena);
                    let output = emitter.emit_class(class_idx);
                    
                    assert!(output.contains("var Animal = /** @class */"), 
                            "Expected IIFE pattern: {}", output);
                    assert!(output.contains("function Animal(name)"),
                            "Expected constructor function: {}", output);
                    assert!(output.contains("return Animal;"),
                            "Expected return statement: {}", output);
                }
            }
        }
    }

    #[test]
    fn test_class_with_method() {
        let source = r#"class Animal {
            speak() {
                console.log("Hello");
            }
        }"#;
        
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        
        if let Some(root_node) = parser.arena.get(root) {
            if let Some(source_file) = parser.arena.get_source_file(root_node) {
                if let Some(&class_idx) = source_file.statements.nodes.first() {
                    let mut emitter = ClassES5Emitter::new(&parser.arena);
                    let output = emitter.emit_class(class_idx);
                    
                    assert!(output.contains("Animal.prototype.speak = function"),
                            "Expected prototype method: {}", output);
                }
            }
        }
    }

    #[test]
    fn test_class_with_static_method() {
        let source = r#"class Counter {
            static count = 0;
            static increment() {
                Counter.count++;
            }
        }"#;
        
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        
        if let Some(root_node) = parser.arena.get(root) {
            if let Some(source_file) = parser.arena.get_source_file(root_node) {
                if let Some(&class_idx) = source_file.statements.nodes.first() {
                    let mut emitter = ClassES5Emitter::new(&parser.arena);
                    let output = emitter.emit_class(class_idx);
                    
                    assert!(output.contains("Counter.count = 0"),
                            "Expected static property: {}", output);
                    assert!(output.contains("Counter.increment = function"),
                            "Expected static method: {}", output);
                }
            }
        }
    }
}
