//! Lowering Pass - Phase 1 of the Transform/Print Architecture
//!
//! This module implements the first phase of emission: analyzing the AST and
//! producing transform directives. The lowering pass walks the ThinNode AST
//! and determines which nodes need transformation based on compiler options
//! (ES5 target, module format, etc.).
//!
//! # Architecture
//!
//! The lowering pass is a **read-only** traversal of the AST that produces
//! a `TransformContext` containing `TransformDirective`s for nodes that need
//! special handling during emission.
//!
//! ## Examples
//!
//! ### ES5 Class Transform
//!
//! When `target: ES5`, a ClassDeclaration needs transformation:
//!
//! ```typescript
//! class Point {
//!     constructor(x, y) { this.x = x; this.y = y; }
//! }
//! ```
//!
//! The lowering pass creates a `TransformDirective::ES5Class` for this node,
//! which the printer will use to emit an IIFE pattern instead of `class`.
//!
//! ### CommonJS Export
//!
//! When `module: CommonJS`, exported declarations need wrapping:
//!
//! ```typescript
//! export class Foo {}
//! ```
//!
//! The lowering pass creates a `TransformDirective::CommonJSExport` that
//! chains with any other transforms (like ES5Class).

use crate::emit_context::EmitContext;
use crate::parser::syntax_kind_ext;
use crate::parser::thin_node::{ThinNode, ThinNodeArena};
use crate::parser::{NodeIndex, NodeList};
use crate::scanner::SyntaxKind;
use crate::transform_context::{ModuleFormat, TransformContext, TransformDirective};
use crate::thin_emitter::ModuleKind;
use crate::transforms::arrow_es5::contains_this_reference;

/// Lowering pass - Phase 1 of emission
///
/// Walks the AST and produces transform directives based on compiler options.
pub struct LoweringPass<'a> {
    arena: &'a ThinNodeArena,
    ctx: &'a EmitContext,
    transforms: TransformContext,
    commonjs_mode: bool,
    has_export_assignment: bool,
}

impl<'a> LoweringPass<'a> {
    /// Create a new lowering pass
    pub fn new(arena: &'a ThinNodeArena, ctx: &'a EmitContext) -> Self {
        LoweringPass {
            arena,
            ctx,
            transforms: TransformContext::new(),
            commonjs_mode: false,
            has_export_assignment: false,
        }
    }

    /// Run the lowering pass on a source file and return the transform context
    pub fn run(mut self, source_file: NodeIndex) -> TransformContext {
        self.init_module_state(source_file);
        self.visit(source_file);
        self.maybe_wrap_module(source_file);
        self.transforms
    }

    /// Visit a node and its children
    fn visit(&mut self, idx: NodeIndex) {
        let Some(node) = self.arena.get(idx) else {
            return;
        };

        match node.kind {
            k if k == syntax_kind_ext::CLASS_DECLARATION => self.visit_class_declaration(node, idx),
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => self.visit_function_declaration(node, idx),
            k if k == syntax_kind_ext::FUNCTION_EXPRESSION => self.visit_function_expression(node, idx),
            k if k == syntax_kind_ext::ARROW_FUNCTION => self.visit_arrow_function(node, idx),
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => self.visit_variable_statement(node, idx),
            k if k == syntax_kind_ext::ENUM_DECLARATION => self.visit_enum_declaration(node, idx),
            k if k == syntax_kind_ext::MODULE_DECLARATION => self.visit_module_declaration(node, idx),
            k if k == syntax_kind_ext::EXPORT_DECLARATION => self.visit_export_declaration(node, idx),
            k if k == syntax_kind_ext::FOR_IN_STATEMENT => self.visit_for_in_statement(node),
            k if k == syntax_kind_ext::FOR_OF_STATEMENT => self.visit_for_of_statement(node, idx),
            _ => self.visit_children(idx),
        }
    }

    /// Visit all children of a node
    fn visit_children(&mut self, idx: NodeIndex) {
        let Some(node) = self.arena.get(idx) else {
            return;
        };

        match node.kind {
            k if k == syntax_kind_ext::SOURCE_FILE => {
                if let Some(sf) = self.arena.get_source_file(node) {
                    for &stmt in &sf.statements.nodes {
                        self.visit(stmt);
                    }
                }
            }
            k if k == syntax_kind_ext::BLOCK || k == syntax_kind_ext::CASE_BLOCK => {
                if let Some(block) = self.get_block_like(node) {
                    let statements = block.statements.nodes.clone();
                    for stmt in statements {
                        self.visit(stmt);
                    }
                }
            }
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                if let Some(var_stmt) = self.arena.get_variable(node) {
                    for &decl_list in &var_stmt.declarations.nodes {
                        self.visit(decl_list);
                    }
                }
            }
            k if k == syntax_kind_ext::VARIABLE_DECLARATION_LIST => {
                if let Some(decl_list) = self.arena.get_variable(node) {
                    if self.ctx.target_es5
                        && self.decl_list_needs_es5_destructuring(decl_list)
                    {
                        self.transforms.insert(
                            idx,
                            TransformDirective::ES5VariableDeclarationList { decl_list: idx },
                        );
                    }
                    for &decl in &decl_list.declarations.nodes {
                        self.visit(decl);
                    }
                }
            }
            k if k == syntax_kind_ext::VARIABLE_DECLARATION => {
                if let Some(decl) = self.arena.get_variable_declaration(node) {
                    self.visit(decl.name);
                    if !decl.initializer.is_none() {
                        self.visit(decl.initializer);
                    }
                }
            }
            k if k == syntax_kind_ext::EXPRESSION_STATEMENT => {
                if let Some(expr_stmt) = self.arena.get_expression_statement(node) {
                    self.visit(expr_stmt.expression);
                }
            }
            k if k == syntax_kind_ext::EXPORT_ASSIGNMENT => {
                if let Some(export_assign) = self.arena.get_export_assignment(node) {
                    self.visit(export_assign.expression);
                }
            }
            k if k == syntax_kind_ext::CALL_EXPRESSION => {
                if let Some(call) = self.arena.get_call_expr(node) {
                    self.visit(call.expression);
                    if let Some(ref args) = call.arguments {
                        for &arg_idx in &args.nodes {
                            self.visit(arg_idx);
                        }
                    }
                }
            }
            k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                if let Some(bin) = self.arena.get_binary_expr(node) {
                    self.visit(bin.left);
                    self.visit(bin.right);
                }
            }
            k if k == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION
                || k == syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION =>
            {
                if let Some(access) = self.arena.get_access_expr(node) {
                    self.visit(access.expression);
                    self.visit(access.name_or_argument);
                }
            }
            k if k == syntax_kind_ext::PROPERTY_ASSIGNMENT => {
                if let Some(prop) = self.arena.get_property_assignment(node) {
                    self.visit(prop.name);
                    self.visit(prop.initializer);
                }
            }
            k if k == syntax_kind_ext::PROPERTY_DECLARATION => {
                if let Some(prop) = self.arena.get_property_decl(node) {
                    if let Some(mods) = &prop.modifiers {
                        for &mod_idx in &mods.nodes {
                            self.visit(mod_idx);
                        }
                    }
                    self.visit(prop.name);
                    if !prop.initializer.is_none() {
                        self.visit(prop.initializer);
                    }
                }
            }
            k if k == syntax_kind_ext::METHOD_DECLARATION => {
                if let Some(method) = self.arena.get_method_decl(node) {
                    if let Some(mods) = &method.modifiers {
                        for &mod_idx in &mods.nodes {
                            self.visit(mod_idx);
                        }
                    }
                    self.visit(method.name);
                    for &param_idx in &method.parameters.nodes {
                        self.visit(param_idx);
                    }
                    if !method.body.is_none() {
                        self.visit(method.body);
                    }
                }
            }
            k if k == syntax_kind_ext::CONSTRUCTOR => {
                if let Some(ctor) = self.arena.get_constructor(node) {
                    if let Some(mods) = &ctor.modifiers {
                        for &mod_idx in &mods.nodes {
                            self.visit(mod_idx);
                        }
                    }
                    for &param_idx in &ctor.parameters.nodes {
                        self.visit(param_idx);
                    }
                    if !ctor.body.is_none() {
                        self.visit(ctor.body);
                    }
                }
            }
            k if k == syntax_kind_ext::GET_ACCESSOR
                || k == syntax_kind_ext::SET_ACCESSOR =>
            {
                if let Some(accessor) = self.arena.get_accessor(node) {
                    if let Some(mods) = &accessor.modifiers {
                        for &mod_idx in &mods.nodes {
                            self.visit(mod_idx);
                        }
                    }
                    self.visit(accessor.name);
                    for &param_idx in &accessor.parameters.nodes {
                        self.visit(param_idx);
                    }
                    if !accessor.body.is_none() {
                        self.visit(accessor.body);
                    }
                }
            }
            k if k == syntax_kind_ext::FUNCTION_EXPRESSION => {
                if let Some(func) = self.arena.get_function(node) {
                    for &param_idx in &func.parameters.nodes {
                        self.visit(param_idx);
                    }
                    if !func.body.is_none() {
                        self.visit(func.body);
                    }
                }
            }
            k if k == syntax_kind_ext::CLASS_EXPRESSION => {
                if let Some(class_data) = self.arena.get_class(node) {
                    if let Some(mods) = &class_data.modifiers {
                        for &mod_idx in &mods.nodes {
                            self.visit(mod_idx);
                        }
                    }
                    for &member in &class_data.members.nodes {
                        self.visit(member);
                    }
                }
            }
            k if k == syntax_kind_ext::PARAMETER => {
                if let Some(param) = self.arena.get_parameter(node) {
                    if let Some(mods) = &param.modifiers {
                        for &mod_idx in &mods.nodes {
                            self.visit(mod_idx);
                        }
                    }
                    self.visit(param.name);
                    if !param.initializer.is_none() {
                        self.visit(param.initializer);
                    }
                }
            }
            k if k == syntax_kind_ext::OBJECT_BINDING_PATTERN
                || k == syntax_kind_ext::ARRAY_BINDING_PATTERN =>
            {
                if let Some(pattern) = self.arena.get_binding_pattern(node) {
                    for &elem in &pattern.elements.nodes {
                        self.visit(elem);
                    }
                }
            }
            k if k == syntax_kind_ext::BINDING_ELEMENT => {
                if let Some(elem) = self.arena.get_binding_element(node) {
                    if !elem.property_name.is_none() {
                        self.visit(elem.property_name);
                    }
                    self.visit(elem.name);
                    if !elem.initializer.is_none() {
                        self.visit(elem.initializer);
                    }
                }
            }
            k if k == syntax_kind_ext::COMPUTED_PROPERTY_NAME => {
                if let Some(computed) = self.arena.get_computed_property(node) {
                    self.visit(computed.expression);
                }
            }
            k if k == syntax_kind_ext::DECORATOR => {
                if let Some(decorator) = self.arena.get_decorator(node) {
                    self.visit(decorator.expression);
                }
            }
            k if k == SyntaxKind::NoSubstitutionTemplateLiteral as u16 => {
                if self.ctx.target_es5 {
                    self.transforms.insert(
                        idx,
                        TransformDirective::ES5TemplateLiteral {
                            template_node: idx,
                        },
                    );
                }
            }
            k if k == syntax_kind_ext::TAGGED_TEMPLATE_EXPRESSION => {
                if self.ctx.target_es5 {
                    self.transforms.insert(
                        idx,
                        TransformDirective::ES5TemplateLiteral {
                            template_node: idx,
                        },
                    );
                }
                if let Some(tagged) = self.arena.get_tagged_template(node) {
                    self.visit(tagged.tag);
                    self.visit(tagged.template);
                }
            }
            k if k == syntax_kind_ext::TEMPLATE_EXPRESSION => {
                if self.ctx.target_es5 {
                    self.transforms.insert(
                        idx,
                        TransformDirective::ES5TemplateLiteral {
                            template_node: idx,
                        },
                    );
                }
                if let Some(template) = self.arena.get_template_expr(node) {
                    self.visit(template.head);
                    for &span_idx in &template.template_spans.nodes {
                        self.visit(span_idx);
                    }
                }
            }
            k if k == syntax_kind_ext::TEMPLATE_SPAN => {
                if let Some(span) = self.arena.get_template_span(node) {
                    self.visit(span.expression);
                    self.visit(span.literal);
                }
            }
            k if k == syntax_kind_ext::SPREAD_ELEMENT
                || k == syntax_kind_ext::SPREAD_ASSIGNMENT =>
            {
                if let Some(spread) = self.arena.get_spread(node) {
                    self.visit(spread.expression);
                }
            }
            k if k == syntax_kind_ext::PARENTHESIZED_EXPRESSION => {
                if let Some(paren) = self.arena.get_parenthesized(node) {
                    self.visit(paren.expression);
                }
            }
            k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION
                || k == syntax_kind_ext::POSTFIX_UNARY_EXPRESSION =>
            {
                if let Some(unary) = self.arena.get_unary_expr(node) {
                    self.visit(unary.operand);
                }
            }
            k if k == syntax_kind_ext::CONDITIONAL_EXPRESSION => {
                if let Some(cond) = self.arena.get_conditional_expr(node) {
                    self.visit(cond.condition);
                    self.visit(cond.when_true);
                    self.visit(cond.when_false);
                }
            }
            k if k == syntax_kind_ext::OBJECT_LITERAL_EXPRESSION => {
                if let Some(lit) = self.arena.get_literal_expr(node) {
                    if self.ctx.target_es5
                        && self.needs_es5_object_literal_transform(&lit.elements.nodes)
                    {
                        self.transforms.insert(
                            idx,
                            TransformDirective::ES5ObjectLiteral {
                                object_literal: idx,
                            },
                        );
                    }

                    for &elem in &lit.elements.nodes {
                        self.visit(elem);
                    }
                }
            }
            k if k == syntax_kind_ext::ARRAY_LITERAL_EXPRESSION => {
                if let Some(lit) = self.arena.get_literal_expr(node) {
                    for &elem in &lit.elements.nodes {
                        self.visit(elem);
                    }
                }
            }
            k if k == syntax_kind_ext::IF_STATEMENT => {
                if let Some(if_stmt) = self.arena.get_if_statement(node) {
                    self.visit(if_stmt.expression);
                    self.visit(if_stmt.then_statement);
                    if !if_stmt.else_statement.is_none() {
                        self.visit(if_stmt.else_statement);
                    }
                }
            }
            k if k == syntax_kind_ext::FOR_STATEMENT
                || k == syntax_kind_ext::WHILE_STATEMENT
                || k == syntax_kind_ext::DO_STATEMENT =>
            {
                if let Some(loop_data) = self.arena.get_loop(node) {
                    self.visit(loop_data.initializer);
                    self.visit(loop_data.condition);
                    self.visit(loop_data.incrementor);
                    self.visit(loop_data.statement);
                }
            }
            k if k == syntax_kind_ext::RETURN_STATEMENT => {
                if let Some(ret) = self.arena.get_return_statement(node) {
                    if !ret.expression.is_none() {
                        self.visit(ret.expression);
                    }
                }
            }
            k if k == syntax_kind_ext::THROW_STATEMENT => {
                if let Some(thr) = self.arena.get_return_statement(node) {
                    if !thr.expression.is_none() {
                        self.visit(thr.expression);
                    }
                }
            }
            k if k == syntax_kind_ext::SWITCH_STATEMENT => {
                if let Some(switch) = self.arena.get_switch(node) {
                    self.visit(switch.expression);
                    self.visit(switch.case_block);
                }
            }
            k if k == syntax_kind_ext::CASE_CLAUSE
                || k == syntax_kind_ext::DEFAULT_CLAUSE =>
            {
                if let Some(clause) = self.arena.get_case_clause(node) {
                    if !clause.expression.is_none() {
                        self.visit(clause.expression);
                    }
                    for &stmt in &clause.statements.nodes {
                        self.visit(stmt);
                    }
                }
            }
            k if k == syntax_kind_ext::TRY_STATEMENT => {
                if let Some(try_stmt) = self.arena.get_try(node) {
                    self.visit(try_stmt.try_block);
                    if !try_stmt.catch_clause.is_none() {
                        self.visit(try_stmt.catch_clause);
                    }
                    if !try_stmt.finally_block.is_none() {
                        self.visit(try_stmt.finally_block);
                    }
                }
            }
            k if k == syntax_kind_ext::CATCH_CLAUSE => {
                if let Some(catch) = self.arena.get_catch_clause(node) {
                    if !catch.variable_declaration.is_none() {
                        self.visit(catch.variable_declaration);
                    }
                    self.visit(catch.block);
                }
            }
            _ => {
            }
        }
    }

    fn visit_for_in_statement(&mut self, node: &ThinNode) {
        let Some(for_in_of) = self.arena.get_for_in_of(node) else {
            return;
        };

        self.visit(for_in_of.initializer);
        self.visit(for_in_of.expression);
        self.visit(for_in_of.statement);
    }

    fn visit_for_of_statement(&mut self, node: &ThinNode, idx: NodeIndex) {
        let Some(for_in_of) = self.arena.get_for_in_of(node) else {
            return;
        };

        if self.ctx.target_es5 && !for_in_of.await_modifier {
            self.transforms.insert(
                idx,
                TransformDirective::ES5ForOf { for_of_node: idx },
            );
        }

        self.visit(for_in_of.initializer);
        self.visit(for_in_of.expression);
        self.visit(for_in_of.statement);
    }

    /// Visit a class declaration
    fn visit_class_declaration(&mut self, node: &ThinNode, idx: NodeIndex) {
        self.lower_class_declaration(node, idx, false, false);
    }

    fn visit_enum_declaration(&mut self, node: &ThinNode, idx: NodeIndex) {
        self.lower_enum_declaration(node, idx, false);
    }

    fn visit_module_declaration(&mut self, node: &ThinNode, idx: NodeIndex) {
        self.lower_module_declaration(node, idx, false);
    }

    fn visit_export_declaration(&mut self, node: &ThinNode, _idx: NodeIndex) {
        let Some(export_decl) = self.arena.get_export_decl(node) else {
            return;
        };

        if export_decl.export_clause.is_none() {
            return;
        }

        if export_decl.is_default_export && self.is_commonjs() {
            if let Some(export_node) = self.arena.get(export_decl.export_clause) {
                if export_node.kind == syntax_kind_ext::FUNCTION_DECLARATION {
                    if let Some(func) = self.arena.get_function(export_node) {
                        let func_name = self.get_identifier_text(func.name);
                        let is_anonymous = func_name == "function"
                            || !Self::is_valid_identifier_name(&func_name);
                        if is_anonymous {
                            self.transforms.insert(
                                export_decl.export_clause,
                                TransformDirective::CommonJSExportDefaultExpr,
                            );

                            if let Some(mods) = &func.modifiers {
                                for &mod_idx in &mods.nodes {
                                    self.visit(mod_idx);
                                }
                            }

                            for &param_idx in &func.parameters.nodes {
                                self.visit(param_idx);
                            }

                            if !func.body.is_none() {
                                self.visit(func.body);
                            }

                            return;
                        }
                    }
                }

                if export_node.kind == syntax_kind_ext::CLASS_DECLARATION {
                    if let Some(class) = self.arena.get_class(export_node) {
                        let class_name = self.get_identifier_text(class.name);
                        if !Self::is_valid_identifier_name(&class_name) {
                            self.transforms.insert(
                                export_decl.export_clause,
                                TransformDirective::CommonJSExportDefaultExpr,
                            );

                            if let Some(mods) = &class.modifiers {
                                for &mod_idx in &mods.nodes {
                                    self.visit(mod_idx);
                                }
                            }

                            for &member_idx in &class.members.nodes {
                                self.visit(member_idx);
                            }

                            return;
                        }
                    }
                }
            }
        }

        if let Some(export_node) = self.arena.get(export_decl.export_clause) {
            if export_node.kind == syntax_kind_ext::CLASS_DECLARATION {
                self.lower_class_declaration(
                    export_node,
                    export_decl.export_clause,
                    true,
                    export_decl.is_default_export,
                );
                return;
            }

            if export_node.kind == syntax_kind_ext::FUNCTION_DECLARATION {
                self.lower_function_declaration(
                    export_node,
                    export_decl.export_clause,
                    true,
                    export_decl.is_default_export,
                );
                return;
            }

            if export_node.kind == syntax_kind_ext::VARIABLE_STATEMENT {
                self.lower_variable_statement(export_node, export_decl.export_clause, true);
                return;
            }

            if export_node.kind == syntax_kind_ext::ENUM_DECLARATION {
                self.lower_enum_declaration(export_node, export_decl.export_clause, true);
                return;
            }

            if export_node.kind == syntax_kind_ext::MODULE_DECLARATION {
                self.lower_module_declaration(export_node, export_decl.export_clause, true);
                return;
            }
        }

        self.visit(export_decl.export_clause);
    }

    fn lower_class_declaration(
        &mut self,
        node: &ThinNode,
        idx: NodeIndex,
        force_export: bool,
        force_default: bool,
    ) {
        let Some(class) = self.arena.get_class(node) else {
            return;
        };

        if let Some(mods) = &class.modifiers {
            for &mod_idx in &mods.nodes {
                self.visit(mod_idx);
            }
        }

        // Skip ambient declarations (declare class)
        if self.has_declare_modifier(&class.modifiers) {
            return;
        }

        let mut is_exported = self.is_commonjs()
            && !self.has_export_assignment
            && (force_export || self.has_export_modifier(&class.modifiers));

        if force_export && self.is_commonjs() && !self.has_export_assignment {
            is_exported = true;
        }

        let is_default = if force_export {
            force_default
        } else {
            self.has_default_modifier(&class.modifiers)
        };

        // Get class name for export
        let class_name = if !class.name.is_none() {
            Some(self.get_identifier_text(class.name))
        } else {
            None
        };

        // Determine the base transform
        let base_directive = if self.ctx.target_es5 {
            // ES5 class transform
            TransformDirective::ES5Class {
                class_node: idx,
                class_name: class_name.clone(),
                heritage: None, // TODO: Handle heritage clauses
                members: class.members.nodes.clone(),
            }
        } else {
            // No transform needed for ES6+ targets
            TransformDirective::Identity
        };

        // Wrap with CommonJS export if needed
        let final_directive = if is_exported && class_name.is_some() {
            let export_directive = TransformDirective::CommonJSExport {
                names: vec![class_name.unwrap()],
                is_default,
                inner: Box::new(TransformDirective::Identity),
            };

            match base_directive {
                TransformDirective::Identity => export_directive,
                other => TransformDirective::Chain(vec![other, export_directive]),
            }
        } else {
            base_directive
        };

        // Only register non-identity transforms
        if !matches!(final_directive, TransformDirective::Identity) {
            self.transforms.insert(idx, final_directive);
        }

        // Visit children (members)
        for &member_idx in &class.members.nodes {
            self.visit(member_idx);
        }
    }

    fn lower_function_declaration(
        &mut self,
        node: &ThinNode,
        idx: NodeIndex,
        force_export: bool,
        force_default: bool,
    ) {
        let Some(func) = self.arena.get_function(node) else {
            return;
        };

        if let Some(mods) = &func.modifiers {
            for &mod_idx in &mods.nodes {
                self.visit(mod_idx);
            }
        }

        let mut is_exported = self.is_commonjs()
            && !self.has_export_assignment
            && (force_export || self.has_export_modifier(&func.modifiers));
        if force_export && self.is_commonjs() && !self.has_export_assignment {
            is_exported = true;
        }

        let is_default = if force_export {
            force_default
        } else {
            self.has_default_modifier(&func.modifiers)
        };

        let func_name = if !func.name.is_none() {
            Some(self.get_identifier_text(func.name))
        } else {
            None
        };

        // Check if this is an async function targeting ES5
        let base_directive = if self.ctx.target_es5 && self.has_async_modifier(idx) {
            TransformDirective::ES5AsyncFunction { function_node: idx }
        } else if self.ctx.target_es5 && self.function_parameters_need_es5_transform(&func.parameters) {
            TransformDirective::ES5FunctionParameters { function_node: idx }
        } else {
            TransformDirective::Identity
        };

        let final_directive = if is_exported && func_name.is_some() {
            let export_directive = TransformDirective::CommonJSExport {
                names: vec![func_name.unwrap()],
                is_default,
                inner: Box::new(TransformDirective::Identity),
            };

            match base_directive {
                TransformDirective::Identity => export_directive,
                other => TransformDirective::Chain(vec![other, export_directive]),
            }
        } else {
            base_directive
        };

        if !matches!(final_directive, TransformDirective::Identity) {
            self.transforms.insert(idx, final_directive);
        }

        for &param_idx in &func.parameters.nodes {
            self.visit(param_idx);
        }

        if !func.body.is_none() {
            self.visit(func.body);
        }
    }

    fn lower_enum_declaration(
        &mut self,
        node: &ThinNode,
        idx: NodeIndex,
        force_export: bool,
    ) {
        let Some(enum_decl) = self.arena.get_enum(node) else {
            return;
        };

        // Skip ambient and const enums (declare/const enums are erased)
        if self.has_declare_modifier(&enum_decl.modifiers)
            || self.has_const_modifier(&enum_decl.modifiers)
        {
            return;
        }

        let mut is_exported = self.is_commonjs()
            && !self.has_export_assignment
            && (force_export || self.has_export_modifier(&enum_decl.modifiers));
        if force_export && self.is_commonjs() && !self.has_export_assignment {
            is_exported = true;
        }

        let enum_name = if !enum_decl.name.is_none() {
            Some(self.get_identifier_text(enum_decl.name))
        } else {
            None
        };

        let base_directive = if self.ctx.target_es5 {
            TransformDirective::ES5Enum { enum_node: idx }
        } else {
            TransformDirective::Identity
        };

        let final_directive = if is_exported && enum_name.is_some() {
            let export_directive = TransformDirective::CommonJSExport {
                names: vec![enum_name.unwrap()],
                is_default: false,
                inner: Box::new(TransformDirective::Identity),
            };

            match base_directive {
                TransformDirective::Identity => export_directive,
                other => TransformDirective::Chain(vec![other, export_directive]),
            }
        } else {
            base_directive
        };

        if !matches!(final_directive, TransformDirective::Identity) {
            self.transforms.insert(idx, final_directive);
        }

        for &member_idx in &enum_decl.members.nodes {
            if let Some(member_node) = self.arena.get(member_idx) {
                if let Some(member) = self.arena.get_enum_member(member_node) {
                    self.visit(member.name);
                    if !member.initializer.is_none() {
                        self.visit(member.initializer);
                    }
                }
            }
        }
    }

    fn lower_module_declaration(
        &mut self,
        node: &ThinNode,
        idx: NodeIndex,
        force_export: bool,
    ) {
        let Some(module_decl) = self.arena.get_module(node) else {
            return;
        };

        // Skip ambient declarations (declare namespace/module)
        if self.has_declare_modifier(&module_decl.modifiers) {
            return;
        }

        let mut is_exported = self.is_commonjs()
            && !self.has_export_assignment
            && (force_export || self.has_export_modifier(&module_decl.modifiers));
        if force_export && self.is_commonjs() && !self.has_export_assignment {
            is_exported = true;
        }

        let module_name = self.get_module_root_name(module_decl.name);

        let base_directive = if self.ctx.target_es5 {
            TransformDirective::ES5Namespace { namespace_node: idx }
        } else {
            TransformDirective::Identity
        };

        let final_directive = if is_exported && module_name.is_some() {
            let export_directive = TransformDirective::CommonJSExport {
                names: vec![module_name.unwrap()],
                is_default: false,
                inner: Box::new(TransformDirective::Identity),
            };

            match base_directive {
                TransformDirective::Identity => export_directive,
                other => TransformDirective::Chain(vec![other, export_directive]),
            }
        } else {
            base_directive
        };

        if !matches!(final_directive, TransformDirective::Identity) {
            self.transforms.insert(idx, final_directive);
        }
    }

    /// Visit a function declaration
    fn visit_function_declaration(&mut self, node: &ThinNode, idx: NodeIndex) {
        self.lower_function_declaration(node, idx, false, false);
    }

    /// Visit an arrow function
    fn visit_arrow_function(&mut self, node: &ThinNode, idx: NodeIndex) {
        let Some(arrow) = self.arena.get_function(node) else {
            return;
        };

        if self.ctx.target_es5 {
            let captures_this = contains_this_reference(self.arena, idx);

            self.transforms.insert(
                idx,
                TransformDirective::ES5ArrowFunction {
                    arrow_node: idx,
                    captures_this,
                },
            );
        }

        for &param_idx in &arrow.parameters.nodes {
            self.visit(param_idx);
        }

        if !arrow.body.is_none() {
            self.visit(arrow.body);
        }
    }

    /// Visit a variable statement
    fn visit_variable_statement(&mut self, node: &ThinNode, idx: NodeIndex) {
        self.lower_variable_statement(node, idx, false);
    }

    fn lower_variable_statement(
        &mut self,
        node: &ThinNode,
        idx: NodeIndex,
        force_export: bool,
    ) {
        let Some(var_stmt) = self.arena.get_variable(node) else {
            return;
        };

        let is_exported = self.is_commonjs()
            && !self.has_export_assignment
            && (force_export || self.has_export_modifier(&var_stmt.modifiers));

        if is_exported {
            let export_names = self.collect_variable_names(&var_stmt.declarations);
            if !export_names.is_empty() {
                self.transforms.insert(
                    idx,
                    TransformDirective::CommonJSExport {
                        names: export_names,
                        is_default: false,
                        inner: Box::new(TransformDirective::Identity),
                    },
                );
            }
        }

        // Visit each declaration
        for &decl in &var_stmt.declarations.nodes {
            self.visit(decl);
        }
    }

    fn visit_function_expression(&mut self, node: &ThinNode, idx: NodeIndex) {
        let Some(func) = self.arena.get_function(node) else {
            return;
        };

        if self.ctx.target_es5 {
            if func.is_async {
                self.transforms.insert(
                    idx,
                    TransformDirective::ES5AsyncFunction { function_node: idx },
                );
            } else if self.function_parameters_need_es5_transform(&func.parameters) {
                self.transforms.insert(
                    idx,
                    TransformDirective::ES5FunctionParameters { function_node: idx },
                );
            }
        }

        for &param_idx in &func.parameters.nodes {
            self.visit(param_idx);
        }

        if !func.body.is_none() {
            self.visit(func.body);
        }
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    fn init_module_state(&mut self, source_file: NodeIndex) {
        let Some(node) = self.arena.get(source_file) else {
            return;
        };
        let Some(source) = self.arena.get_source_file(node) else {
            return;
        };

        self.has_export_assignment = self.contains_export_assignment(&source.statements);
        self.commonjs_mode = if self.ctx.is_commonjs() {
            true
        } else if self.ctx.auto_detect_module {
            self.file_is_module(&source.statements)
        } else {
            false
        };
    }

    fn is_commonjs(&self) -> bool {
        self.commonjs_mode
    }

    /// Check if a modifier list contains the 'declare' keyword
    fn has_declare_modifier(&self, modifiers: &Option<NodeList>) -> bool {
        let Some(mods) = modifiers else {
            return false;
        };

        mods.nodes.iter().any(|&mod_idx| {
            self.arena
                .get(mod_idx)
                .map(|n| n.kind == SyntaxKind::DeclareKeyword as u16)
                .unwrap_or(false)
        })
    }

    /// Check if a modifier list contains the 'const' keyword
    fn has_const_modifier(&self, modifiers: &Option<NodeList>) -> bool {
        let Some(mods) = modifiers else {
            return false;
        };

        mods.nodes.iter().any(|&mod_idx| {
            self.arena
                .get(mod_idx)
                .map(|n| n.kind == SyntaxKind::ConstKeyword as u16)
                .unwrap_or(false)
        })
    }

    /// Check if a modifier list contains the 'export' keyword
    fn has_export_modifier(&self, modifiers: &Option<NodeList>) -> bool {
        let Some(mods) = modifiers else {
            return false;
        };

        mods.nodes.iter().any(|&mod_idx| {
            self.arena
                .get(mod_idx)
                .map(|n| n.kind == SyntaxKind::ExportKeyword as u16)
                .unwrap_or(false)
        })
    }

    /// Check if a modifier list contains the 'default' keyword
    fn has_default_modifier(&self, modifiers: &Option<NodeList>) -> bool {
        let Some(mods) = modifiers else {
            return false;
        };

        mods.nodes.iter().any(|&mod_idx| {
            self.arena
                .get(mod_idx)
                .map(|n| n.kind == SyntaxKind::DefaultKeyword as u16)
                .unwrap_or(false)
        })
    }

    /// Check if a function has the 'async' modifier
    fn has_async_modifier(&self, func_idx: NodeIndex) -> bool {
        let Some(func_node) = self.arena.get(func_idx) else {
            return false;
        };

        let Some(func) = self.arena.get_function(func_node) else {
            return false;
        };

        if func.is_async {
            return true;
        }

        let Some(mods) = &func.modifiers else {
            return false;
        };

        mods.nodes.iter().any(|&mod_idx| {
            self.arena
                .get(mod_idx)
                .map(|n| n.kind == SyntaxKind::AsyncKeyword as u16)
                .unwrap_or(false)
        })
    }

    fn needs_es5_object_literal_transform(&self, elements: &[NodeIndex]) -> bool {
        elements.iter().any(|&idx| {
            if self.is_computed_property_member(idx) || self.is_spread_element(idx) {
                return true;
            }

            let Some(node) = self.arena.get(idx) else {
                return false;
            };

            node.kind == syntax_kind_ext::METHOD_DECLARATION
                || node.kind == syntax_kind_ext::SHORTHAND_PROPERTY_ASSIGNMENT
        })
    }

    fn decl_list_needs_es5_destructuring(
        &self,
        decl_list: &crate::parser::thin_node::VariableData,
    ) -> bool {
        decl_list.declarations.nodes.iter().any(|&decl_idx| {
            let Some(decl_node) = self.arena.get(decl_idx) else {
                return false;
            };
            let Some(decl) = self.arena.get_variable_declaration(decl_node) else {
                return false;
            };

            !decl.initializer.is_none() && self.is_binding_pattern_idx(decl.name)
        })
    }

    fn function_parameters_need_es5_transform(&self, params: &NodeList) -> bool {
        params.nodes.iter().any(|&param_idx| {
            let Some(param_node) = self.arena.get(param_idx) else {
                return false;
            };
            let Some(param) = self.arena.get_parameter(param_node) else {
                return false;
            };

            param.dot_dot_dot_token
                || !param.initializer.is_none()
                || self.is_binding_pattern_idx(param.name)
        })
    }

    fn is_binding_pattern_idx(&self, idx: NodeIndex) -> bool {
        self.arena.get(idx).map(|node| {
            node.kind == syntax_kind_ext::OBJECT_BINDING_PATTERN
                || node.kind == syntax_kind_ext::ARRAY_BINDING_PATTERN
        }).unwrap_or(false)
    }

    fn is_computed_property_member(&self, idx: NodeIndex) -> bool {
        let Some(node) = self.arena.get(idx) else {
            return false;
        };

        let name_idx = match node.kind {
            k if k == syntax_kind_ext::PROPERTY_ASSIGNMENT => {
                self.arena.get_property_assignment(node).map(|p| p.name)
            }
            k if k == syntax_kind_ext::METHOD_DECLARATION => {
                self.arena.get_method_decl(node).map(|m| m.name)
            }
            k if k == syntax_kind_ext::GET_ACCESSOR || k == syntax_kind_ext::SET_ACCESSOR => {
                self.arena.get_accessor(node).map(|a| a.name)
            }
            _ => None,
        };

        if let Some(name_idx) = name_idx {
            if let Some(name_node) = self.arena.get(name_idx) {
                return name_node.kind == syntax_kind_ext::COMPUTED_PROPERTY_NAME;
            }
        }

        false
    }

    fn is_spread_element(&self, idx: NodeIndex) -> bool {
        let Some(node) = self.arena.get(idx) else {
            return false;
        };

        node.kind == syntax_kind_ext::SPREAD_ASSIGNMENT
            || node.kind == syntax_kind_ext::SPREAD_ELEMENT
    }

    /// Get identifier text from a node index
    fn get_identifier_text(&self, idx: NodeIndex) -> String {
        if idx.is_none() {
            return String::new();
        }

        let Some(node) = self.arena.get(idx) else {
            return String::new();
        };

        if node.kind != SyntaxKind::Identifier as u16 {
            return String::new();
        }

        let Some(ident) = self.arena.get_identifier(node) else {
            return String::new();
        };

        ident.escaped_text.clone()
    }

    fn is_valid_identifier_name(name: &str) -> bool {
        let mut chars = name.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !(first == '_' || first == '$' || first.is_alphabetic()) {
            return false;
        }
        chars.all(|ch| ch == '_' || ch == '$' || ch.is_alphanumeric())
    }

    fn get_module_root_name(&self, name_idx: NodeIndex) -> Option<String> {
        if name_idx.is_none() {
            return None;
        }

        let node = self.arena.get(name_idx)?;
        if node.kind == SyntaxKind::Identifier as u16 {
            return self.arena.get_identifier(node).map(|id| id.escaped_text.clone());
        }

        if node.kind == syntax_kind_ext::QUALIFIED_NAME {
            if let Some(qn) = self.arena.qualified_names.get(node.data_index as usize) {
                return self.get_module_root_name(qn.left);
            }
        }

        None
    }

    fn get_block_like(&self, node: &ThinNode) -> Option<&crate::parser::thin_node::BlockData> {
        if node.kind == syntax_kind_ext::BLOCK || node.kind == syntax_kind_ext::CASE_BLOCK {
            self.arena.blocks.get(node.data_index as usize)
        } else {
            None
        }
    }

    fn collect_variable_names(&self, declarations: &NodeList) -> Vec<String> {
        let mut names = Vec::new();
        for &decl_list_idx in &declarations.nodes {
            let Some(decl_list_node) = self.arena.get(decl_list_idx) else {
                continue;
            };
            let Some(decl_list) = self.arena.get_variable(decl_list_node) else {
                continue;
            };

            for &decl_idx in &decl_list.declarations.nodes {
                let Some(decl_node) = self.arena.get(decl_idx) else {
                    continue;
                };
                let Some(decl) = self.arena.get_variable_declaration(decl_node) else {
                    continue;
                };
                self.collect_binding_names(decl.name, &mut names);
            }
        }
        names
    }

    fn collect_binding_names(&self, name_idx: NodeIndex, names: &mut Vec<String>) {
        if name_idx.is_none() {
            return;
        }

        let Some(node) = self.arena.get(name_idx) else {
            return;
        };

        if node.kind == SyntaxKind::Identifier as u16 {
            if let Some(id) = self.arena.get_identifier(node) {
                names.push(id.escaped_text.clone());
            }
            return;
        }

        match node.kind {
            k if k == syntax_kind_ext::OBJECT_BINDING_PATTERN
                || k == syntax_kind_ext::ARRAY_BINDING_PATTERN =>
            {
                if let Some(pattern) = self.arena.get_binding_pattern(node) {
                    for &elem_idx in &pattern.elements.nodes {
                        self.collect_binding_names_from_element(elem_idx, names);
                    }
                }
            }
            k if k == syntax_kind_ext::BINDING_ELEMENT => {
                if let Some(elem) = self.arena.get_binding_element(node) {
                    self.collect_binding_names(elem.name, names);
                }
            }
            _ => {}
        }
    }

    fn collect_binding_names_from_element(&self, elem_idx: NodeIndex, names: &mut Vec<String>) {
        if elem_idx.is_none() {
            return;
        }

        let Some(elem_node) = self.arena.get(elem_idx) else {
            return;
        };

        if let Some(elem) = self.arena.get_binding_element(elem_node) {
            self.collect_binding_names(elem.name, names);
        }
    }

    fn maybe_wrap_module(&mut self, source_file: NodeIndex) {
        let format = match self.ctx.options.module {
            ModuleKind::AMD => ModuleFormat::AMD,
            ModuleKind::System => ModuleFormat::System,
            ModuleKind::UMD => ModuleFormat::UMD,
            _ => return,
        };

        let Some(node) = self.arena.get(source_file) else {
            return;
        };
        let Some(source) = self.arena.get_source_file(node) else {
            return;
        };

        if !self.file_is_module(&source.statements) {
            return;
        }

        let dependencies = self.collect_module_dependencies(&source.statements.nodes);
        self.transforms.insert(
            source_file,
            TransformDirective::ModuleWrapper {
                format,
                dependencies,
                body: source.statements.nodes.clone(),
            },
        );
    }

    fn file_is_module(&self, statements: &NodeList) -> bool {
        for &stmt_idx in &statements.nodes {
            if let Some(node) = self.arena.get(stmt_idx) {
                match node.kind {
                    k if k == syntax_kind_ext::IMPORT_DECLARATION
                        || k == syntax_kind_ext::IMPORT_EQUALS_DECLARATION =>
                    {
                        if let Some(import_decl) = self.arena.get_import_decl(node) {
                            if self.import_has_runtime_dependency(import_decl) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::EXPORT_DECLARATION => {
                        if let Some(export_decl) = self.arena.get_export_decl(node) {
                            if self.export_decl_has_runtime_value(export_decl) {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::EXPORT_ASSIGNMENT => return true,
                    k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                        if let Some(var_stmt) = self.arena.get_variable(node) {
                            if self.has_export_modifier(&var_stmt.modifiers)
                                && !self.has_declare_modifier(&var_stmt.modifiers)
                            {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                        if let Some(func) = self.arena.get_function(node) {
                            if self.has_export_modifier(&func.modifiers)
                                && !self.has_declare_modifier(&func.modifiers)
                            {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::CLASS_DECLARATION => {
                        if let Some(class) = self.arena.get_class(node) {
                            if self.has_export_modifier(&class.modifiers)
                                && !self.has_declare_modifier(&class.modifiers)
                            {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::ENUM_DECLARATION => {
                        if let Some(enum_decl) = self.arena.get_enum(node) {
                            if self.has_export_modifier(&enum_decl.modifiers)
                                && !self.has_declare_modifier(&enum_decl.modifiers)
                                && !self.has_const_modifier(&enum_decl.modifiers)
                            {
                                return true;
                            }
                        }
                    }
                    k if k == syntax_kind_ext::MODULE_DECLARATION => {
                        if let Some(module) = self.arena.get_module(node) {
                            if self.has_export_modifier(&module.modifiers)
                                && !self.has_declare_modifier(&module.modifiers)
                            {
                                return true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    fn contains_export_assignment(&self, statements: &NodeList) -> bool {
        for &stmt_idx in &statements.nodes {
            if let Some(node) = self.arena.get(stmt_idx) {
                if node.kind == syntax_kind_ext::EXPORT_ASSIGNMENT {
                    return true;
                }
            }
        }
        false
    }

    fn collect_module_dependencies(&self, statements: &[NodeIndex]) -> Vec<String> {
        let mut deps = Vec::new();
        for &stmt_idx in statements {
            let Some(node) = self.arena.get(stmt_idx) else {
                continue;
            };

            if node.kind == syntax_kind_ext::IMPORT_DECLARATION
                || node.kind == syntax_kind_ext::IMPORT_EQUALS_DECLARATION
            {
                if let Some(import_decl) = self.arena.get_import_decl(node) {
                    if !self.import_has_runtime_dependency(import_decl) {
                        continue;
                    }
                    if let Some(text) = self.get_module_specifier_text(import_decl.module_specifier) {
                        if !deps.contains(&text) {
                            deps.push(text);
                        }
                    }
                }
                continue;
            }

            if node.kind == syntax_kind_ext::EXPORT_DECLARATION {
                if let Some(export_decl) = self.arena.get_export_decl(node) {
                    if !self.export_has_runtime_dependency(export_decl) {
                        continue;
                    }
                    if let Some(text) = self.get_module_specifier_text(export_decl.module_specifier) {
                        if !deps.contains(&text) {
                            deps.push(text);
                        }
                    }
                }
            }
        }

        deps
    }

    fn import_has_runtime_dependency(
        &self,
        import_decl: &crate::parser::thin_node::ImportDeclData,
    ) -> bool {
        if import_decl.import_clause.is_none() {
            return true;
        }

        let Some(clause_node) = self.arena.get(import_decl.import_clause) else {
            return true;
        };

        if clause_node.kind != syntax_kind_ext::IMPORT_CLAUSE {
            return self.import_equals_has_external_module(import_decl.module_specifier);
        }

        let Some(clause) = self.arena.get_import_clause(clause_node) else {
            return true;
        };

        if clause.is_type_only {
            return false;
        }

        if !clause.name.is_none() {
            return true;
        }

        if clause.named_bindings.is_none() {
            return false;
        }

        let Some(bindings_node) = self.arena.get(clause.named_bindings) else {
            return false;
        };

        let Some(named) = self.arena.get_named_imports(bindings_node) else {
            return true;
        };

        if !named.name.is_none() {
            return true;
        }

        if named.elements.nodes.is_empty() {
            return true;
        }

        for &spec_idx in &named.elements.nodes {
            let Some(spec_node) = self.arena.get(spec_idx) else {
                continue;
            };
            if let Some(spec) = self.arena.get_specifier(spec_node) {
                if !spec.is_type_only {
                    return true;
                }
            }
        }

        false
    }

    fn import_equals_has_external_module(&self, module_specifier: NodeIndex) -> bool {
        if module_specifier.is_none() {
            return false;
        }

        let Some(node) = self.arena.get(module_specifier) else {
            return false;
        };

        node.kind == SyntaxKind::StringLiteral as u16
    }

    fn export_decl_has_runtime_value(
        &self,
        export_decl: &crate::parser::thin_node::ExportDeclData,
    ) -> bool {
        if export_decl.is_type_only {
            return false;
        }

        if export_decl.is_default_export {
            return true;
        }

        if export_decl.export_clause.is_none() {
            return true;
        }

        let Some(clause_node) = self.arena.get(export_decl.export_clause) else {
            return false;
        };

        if let Some(named) = self.arena.get_named_imports(clause_node) {
            if !named.name.is_none() {
                return true;
            }

            if named.elements.nodes.is_empty() {
                return true;
            }

            for &spec_idx in &named.elements.nodes {
                let Some(spec_node) = self.arena.get(spec_idx) else {
                    continue;
                };
                if let Some(spec) = self.arena.get_specifier(spec_node) {
                    if !spec.is_type_only {
                        return true;
                    }
                }
            }

            return false;
        }

        if self.export_clause_is_type_only(clause_node) {
            return false;
        }

        true
    }

    fn export_clause_is_type_only(&self, clause_node: &ThinNode) -> bool {
        match clause_node.kind {
            k if k == syntax_kind_ext::INTERFACE_DECLARATION => true,
            k if k == syntax_kind_ext::TYPE_ALIAS_DECLARATION => true,
            k if k == syntax_kind_ext::ENUM_DECLARATION => {
                let Some(enum_decl) = self.arena.get_enum(clause_node) else {
                    return false;
                };
                self.has_declare_modifier(&enum_decl.modifiers)
                    || self.has_const_modifier(&enum_decl.modifiers)
            }
            k if k == syntax_kind_ext::CLASS_DECLARATION => {
                let Some(class_decl) = self.arena.get_class(clause_node) else {
                    return false;
                };
                self.has_declare_modifier(&class_decl.modifiers)
            }
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                let Some(func_decl) = self.arena.get_function(clause_node) else {
                    return false;
                };
                self.has_declare_modifier(&func_decl.modifiers)
            }
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                let Some(var_decl) = self.arena.get_variable(clause_node) else {
                    return false;
                };
                self.has_declare_modifier(&var_decl.modifiers)
            }
            k if k == syntax_kind_ext::MODULE_DECLARATION => {
                let Some(module_decl) = self.arena.get_module(clause_node) else {
                    return false;
                };
                self.has_declare_modifier(&module_decl.modifiers)
            }
            _ => false,
        }
    }

    fn export_has_runtime_dependency(
        &self,
        export_decl: &crate::parser::thin_node::ExportDeclData,
    ) -> bool {
        if export_decl.is_type_only {
            return false;
        }

        if export_decl.module_specifier.is_none() {
            return false;
        }

        if export_decl.export_clause.is_none() {
            return true;
        }

        let Some(clause_node) = self.arena.get(export_decl.export_clause) else {
            return true;
        };

        let Some(named) = self.arena.get_named_imports(clause_node) else {
            return true;
        };

        if !named.name.is_none() {
            return true;
        }

        if named.elements.nodes.is_empty() {
            return true;
        }

        for &spec_idx in &named.elements.nodes {
            let Some(spec_node) = self.arena.get(spec_idx) else {
                continue;
            };
            if let Some(spec) = self.arena.get_specifier(spec_node) {
                if !spec.is_type_only {
                    return true;
                }
            }
        }

        false
    }

    fn get_module_specifier_text(&self, specifier: NodeIndex) -> Option<String> {
        if specifier.is_none() {
            return None;
        }

        let Some(node) = self.arena.get(specifier) else {
            return None;
        };
        let Some(literal) = self.arena.get_literal(node) else {
            return None;
        };

        Some(literal.text.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::thin_node::ThinNodeArena;
    use crate::thin_parser::ThinParserState;

    fn parse(source: &str) -> (ThinNodeArena, NodeIndex) {
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        (parser.arena, root)
    }

    #[test]
    fn test_lowering_pass_es6_no_transforms() {
        let (arena, root) = parse("class Foo {}");
        let ctx = EmitContext::default();
        let lowering = LoweringPass::new(&arena, &ctx);
        let transforms = lowering.run(root);

        // ES6 target should not add transforms for classes
        assert!(transforms.is_empty());
    }

    #[test]
    fn test_lowering_pass_es5_class() {
        let (arena, root) = parse("class Foo { constructor(x) { this.x = x; } }");
        let mut ctx = EmitContext::default();
        ctx.target_es5 = true;

        let lowering = LoweringPass::new(&arena, &ctx);
        let transforms = lowering.run(root);

        // ES5 target should add ES5Class transform
        // The actual class node index depends on parser implementation
        // This test validates the architecture, not specific indices
        assert!(!transforms.is_empty(), "Expected ES5 class transform");
    }

    #[test]
    fn test_lowering_pass_commonjs_export() {
        let (arena, root) = parse("export class Foo {}");
        let mut ctx = EmitContext::default();
        ctx.options.module = crate::thin_emitter::ModuleKind::CommonJS;

        let lowering = LoweringPass::new(&arena, &ctx);
        let transforms = lowering.run(root);

        // CommonJS module should add export transform
        assert!(!transforms.is_empty(), "Expected CommonJS export transform");
    }

    #[test]
    fn test_lowering_pass_commonjs_export_vars() {
        let (arena, root) = parse("export const a = 1, b = 2;");
        let mut ctx = EmitContext::default();
        ctx.options.module = crate::thin_emitter::ModuleKind::CommonJS;

        let lowering = LoweringPass::new(&arena, &ctx);
        let transforms = lowering.run(root);

        assert!(
            !transforms.is_empty(),
            "Expected CommonJS export transform for variables"
        );
    }

    #[test]
    fn test_lowering_pass_nested_arrow_in_class() {
        let (arena, root) = parse("class C { m() { const f = () => this; } }");
        let mut ctx = EmitContext::default();
        ctx.target_es5 = true;

        let lowering = LoweringPass::new(&arena, &ctx);
        let transforms = lowering.run(root);

        assert!(
            transforms.len() >= 2,
            "Expected transforms for class and nested arrow function"
        );
    }
}
