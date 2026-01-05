//! Module System Transforms
//!
//! Transforms ES modules to various module systems:
//! - CommonJS (Node.js require/exports)
//! - AMD (RequireJS define)
//! - UMD (Universal - works in both AMD and CommonJS)
//! - SystemJS (System.register)
//!
//! Also handles:
//! - Import/export elision for type-only
//! - __esModule marker
//! - Interop helpers (__importDefault, __importStar)

use super::{TransformContext, Transformer, HelpersNeeded};
use crate::emitter::ModuleKind;
use crate::parser::{Node, NodeIndex, NodeBase, NodeList, syntax_kind_ext};
use crate::parser::ast::{
    Identifier, VariableStatement, VariableDeclarationList, VariableDeclaration,
    CallExpression, PropertyAccessExpression, ExpressionStatement, StringLiteral,
};
use crate::scanner::SyntaxKind;

/// Module transformation state
pub struct ModuleTransformer {
    /// Target module system
    module_kind: ModuleKind,
    /// Whether we've emitted the __esModule marker
    _has_es_module_marker: bool,
    /// Collected imports for AMD/UMD wrapper
    _imports: Vec<ImportInfo>,
    /// Collected exports for AMD/UMD wrapper
    _exports: Vec<ExportInfo>,
}

/// Information about an import
#[derive(Clone)]
struct ImportInfo {
    /// The module specifier (e.g., "./foo")
    _module_specifier: String,
    /// Whether it's a namespace import (* as foo)
    _is_namespace: bool,
    /// Whether it's a default import
    _is_default: bool,
    /// Import bindings
    _bindings: Vec<String>,
}

/// Information about an export
#[derive(Clone)]
struct ExportInfo {
    /// The local name
    _local_name: String,
    /// The exported name (if different)
    _exported_name: Option<String>,
    /// Whether it's a re-export from another module
    _is_re_export: bool,
}

impl ModuleTransformer {
    pub fn new(module_kind: ModuleKind) -> Self {
        ModuleTransformer {
            module_kind,
            _has_es_module_marker: false,
            _imports: Vec::new(),
            _exports: Vec::new(),
        }
    }

    /// Transform import declaration
    ///
    /// ```ts
    /// import { foo } from './bar';
    /// import * as baz from './qux';
    /// import quz from './quux';
    /// ```
    ///
    /// To CommonJS:
    /// ```js
    /// const bar_1 = require('./bar');
    /// const baz = require('./qux');
    /// const quux_1 = __importDefault(require('./quux'));
    /// ```
    pub fn transform_import_declaration(
        &mut self,
        node_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        match self.module_kind {
            ModuleKind::CommonJS => {
                return self.transform_import_to_commonjs(node_idx, ctx);
            }
            ModuleKind::AMD | ModuleKind::UMD => {
                // Collect import for the define() wrapper
            }
            ModuleKind::System => {
                // Transform to System.register format
            }
            _ => {}
        }
        None
    }

    /// Transform an import to CommonJS require()
    ///
    /// import foo from './bar'  → const bar_1 = __importDefault(require('./bar'))
    /// import { x } from './bar' → const bar_1 = require('./bar')
    /// import * as bar from './bar' → const bar = __importStar(require('./bar'))
    fn transform_import_to_commonjs(
        &mut self,
        node_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        let import_decl = match ctx.arena.get(node_idx) {
            Some(Node::ImportDeclaration(decl)) => decl.clone(),
            _ => return None,
        };

        // Skip type-only imports entirely
        if let Some(Node::ImportClause(clause)) = ctx.arena.get(import_decl.import_clause) {
            if clause.is_type_only {
                // Return an empty statement to elide the import
                return Some(self.create_empty_statement(ctx));
            }
        }

        // Get the module specifier string
        let module_specifier = match ctx.arena.get(import_decl.module_specifier) {
            Some(Node::StringLiteral(lit)) => lit.text.clone(),
            _ => return None,
        };

        // Generate a unique binding name for the module
        let binding_name = self.generate_import_binding_name(&module_specifier, ctx);

        // Determine if we need __importDefault or __importStar
        let (needs_default, needs_star) = self.analyze_import_needs(import_decl.import_clause, ctx);

        if needs_default {
            ctx.helpers_needed.import_default = true;
        }
        if needs_star {
            ctx.helpers_needed.import_star = true;
        }

        // Create: const binding_name = require('./module')
        let require_call = self.create_require_call(&module_specifier, ctx);

        // Wrap with helper if needed
        let initializer = if needs_default {
            self.create_import_default_call(require_call, ctx)
        } else if needs_star {
            self.create_import_star_call(require_call, ctx)
        } else {
            require_call
        };

        // Create the variable declaration
        Some(self.create_const_declaration(&binding_name, initializer, ctx))
    }

    /// Analyze what kind of import helpers are needed
    fn analyze_import_needs(&self, import_clause_idx: NodeIndex, ctx: &TransformContext) -> (bool, bool) {
        let mut needs_default = false;
        let mut needs_star = false;

        if let Some(Node::ImportClause(clause)) = ctx.arena.get(import_clause_idx) {
            // Default import (import foo from './bar')
            if !clause.name.is_none() {
                needs_default = true;
            }

            // Check named bindings
            if let Some(Node::NamespaceImport(_)) = ctx.arena.get(clause.named_bindings) {
                // Namespace import (import * as foo from './bar')
                needs_star = true;
            }
            // Named imports don't need helpers, just destructuring
        }

        (needs_default, needs_star)
    }

    /// Generate a unique binding name for the imported module
    fn generate_import_binding_name(&self, module_specifier: &str, ctx: &mut TransformContext) -> String {
        // Extract base name from module specifier
        let base_name = module_specifier
            .rsplit('/')
            .next()
            .unwrap_or("module")
            .replace(['.', '-'], "_");
        ctx.generate_unique_name(&base_name)
    }

    /// Create a require() call expression
    fn create_require_call(&self, module_specifier: &str, ctx: &mut TransformContext) -> NodeIndex {
        // Create the 'require' identifier
        let require_id = Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, 0, 0),
            escaped_text: "require".to_string(),
            original_text: None,
            type_arguments: None,
        };
        let require_idx = ctx.arena.add(Node::Identifier(require_id));

        // Create the module specifier string literal
        let spec_lit = StringLiteral {
            base: NodeBase::new(SyntaxKind::StringLiteral, 0, 0),
            text: module_specifier.to_string(),
            is_unterminated: false,
            has_extended_unicode_escape: false,
        };
        let spec_idx = ctx.arena.add(Node::StringLiteral(spec_lit));

        // Create the call expression: require('./module')
        let mut args = NodeList::new();
        args.push(spec_idx);

        let call_expr = CallExpression {
            base: NodeBase::new_ext(syntax_kind_ext::CALL_EXPRESSION, 0, 0),
            expression: require_idx,
            type_arguments: None,
            arguments: args,
        };
        ctx.arena.add(Node::CallExpression(call_expr))
    }

    /// Create __importDefault(x) call
    fn create_import_default_call(&self, arg_idx: NodeIndex, ctx: &mut TransformContext) -> NodeIndex {
        let helper_id = Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, 0, 0),
            escaped_text: "__importDefault".to_string(),
            original_text: None,
            type_arguments: None,
        };
        let helper_idx = ctx.arena.add(Node::Identifier(helper_id));

        let mut args = NodeList::new();
        args.push(arg_idx);

        let call_expr = CallExpression {
            base: NodeBase::new_ext(syntax_kind_ext::CALL_EXPRESSION, 0, 0),
            expression: helper_idx,
            type_arguments: None,
            arguments: args,
        };
        ctx.arena.add(Node::CallExpression(call_expr))
    }

    /// Create __importStar(x) call
    fn create_import_star_call(&self, arg_idx: NodeIndex, ctx: &mut TransformContext) -> NodeIndex {
        let helper_id = Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, 0, 0),
            escaped_text: "__importStar".to_string(),
            original_text: None,
            type_arguments: None,
        };
        let helper_idx = ctx.arena.add(Node::Identifier(helper_id));

        let mut args = NodeList::new();
        args.push(arg_idx);

        let call_expr = CallExpression {
            base: NodeBase::new_ext(syntax_kind_ext::CALL_EXPRESSION, 0, 0),
            expression: helper_idx,
            type_arguments: None,
            arguments: args,
        };
        ctx.arena.add(Node::CallExpression(call_expr))
    }

    /// Create a const declaration: const name = initializer
    fn create_const_declaration(&self, name: &str, initializer: NodeIndex, ctx: &mut TransformContext) -> NodeIndex {
        // Create the identifier for the binding
        let name_id = Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, 0, 0),
            escaped_text: name.to_string(),
            original_text: None,
            type_arguments: None,
        };
        let name_idx = ctx.arena.add(Node::Identifier(name_id));

        // Create the variable declaration
        let var_decl = VariableDeclaration {
            base: NodeBase::new_ext(syntax_kind_ext::VARIABLE_DECLARATION, 0, 0),
            name: name_idx,
            exclamation_token: false,
            type_annotation: NodeIndex::NONE,
            initializer,
        };
        let var_decl_idx = ctx.arena.add(Node::VariableDeclaration(var_decl));

        // Create the declaration list (const)
        let mut declarations = NodeList::new();
        declarations.push(var_decl_idx);

        let var_decl_list = VariableDeclarationList {
            base: NodeBase::new_ext(syntax_kind_ext::VARIABLE_DECLARATION_LIST, 0, 0),
            declarations,
            // Const is indicated by flags
        };
        let var_decl_list_idx = ctx.arena.add(Node::VariableDeclarationList(var_decl_list));

        // Create the variable statement
        let var_stmt = VariableStatement {
            base: NodeBase::new_ext(syntax_kind_ext::VARIABLE_STATEMENT, 0, 0),
            modifiers: None,
            declaration_list: var_decl_list_idx,
        };
        ctx.arena.add(Node::VariableStatement(var_stmt))
    }

    /// Create an empty statement (for eliding type-only imports)
    fn create_empty_statement(&self, ctx: &mut TransformContext) -> NodeIndex {
        use crate::parser::ast::EmptyStatement;
        let empty = EmptyStatement {
            base: NodeBase::new_ext(syntax_kind_ext::EMPTY_STATEMENT, 0, 0),
        };
        ctx.arena.add(Node::EmptyStatement(empty))
    }

    /// Transform export declaration
    ///
    /// ```ts
    /// export { foo, bar as baz };
    /// export * from './qux';
    /// export default function() {}
    /// ```
    ///
    /// To CommonJS:
    /// ```js
    /// exports.foo = foo;
    /// exports.baz = bar;
    /// __exportStar(require('./qux'), exports);
    /// exports.default = function() {};
    /// ```
    pub fn transform_export_declaration(
        &mut self,
        node_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        match self.module_kind {
            ModuleKind::CommonJS => {
                return self.transform_export_to_commonjs(node_idx, ctx);
            }
            ModuleKind::AMD | ModuleKind::UMD => {
                // Collect export for the define() wrapper
            }
            ModuleKind::System => {
                // Transform to System.register format
            }
            _ => {}
        }
        None
    }

    /// Transform an export to CommonJS
    ///
    /// export { foo }  → exports.foo = foo
    /// export { foo as bar }  → exports.bar = foo
    /// export * from './bar'  → __exportStar(require('./bar'), exports)
    fn transform_export_to_commonjs(
        &mut self,
        node_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        let export_decl = match ctx.arena.get(node_idx) {
            Some(Node::ExportDeclaration(decl)) => decl.clone(),
            _ => return None,
        };

        // Skip type-only exports
        if export_decl.is_type_only {
            return Some(self.create_empty_statement(ctx));
        }

        // Check for re-export (export * from './foo' or export { x } from './foo')
        if !export_decl.module_specifier.is_none() {
            // Get the module specifier
            let module_specifier = match ctx.arena.get(export_decl.module_specifier) {
                Some(Node::StringLiteral(lit)) => lit.text.clone(),
                _ => return None,
            };

            // Check if it's export * or named re-export
            if export_decl.export_clause.is_none() {
                // export * from './foo' → __exportStar(require('./foo'), exports)
                ctx.helpers_needed.export_star = true;
                return Some(self.create_export_star_call(&module_specifier, ctx));
            } else {
                // export { x } from './foo' → re-export with require
                return self.transform_named_reexport(export_decl.export_clause, &module_specifier, ctx);
            }
        }

        // Named exports: export { foo, bar as baz }
        if let Some(Node::NamedExports(named)) = ctx.arena.get(export_decl.export_clause) {
            return self.transform_named_exports(&named.elements.nodes.clone(), ctx);
        }

        None
    }

    /// Transform named exports to exports.x = x statements
    fn transform_named_exports(
        &mut self,
        specifiers: &[NodeIndex],
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        // For now, just transform the first specifier
        // A full implementation would create a block with all assignments
        for spec_idx in specifiers {
            if let Some(Node::ExportSpecifier(spec)) = ctx.arena.get(*spec_idx) {
                // Skip type-only specifiers
                if spec.is_type_only {
                    continue;
                }

                // Get the local name (property_name or name)
                let local_name = if !spec.property_name.is_none() {
                    self.get_identifier_text(spec.property_name, ctx)
                } else {
                    self.get_identifier_text(spec.name, ctx)
                };

                // Get the exported name
                let exported_name = self.get_identifier_text(spec.name, ctx);

                if let (Some(local), Some(exported)) = (local_name, exported_name) {
                    // Create: exports.exported = local
                    return Some(self.create_export_assignment(&exported, &local, ctx));
                }
            }
        }

        Some(self.create_empty_statement(ctx))
    }

    /// Transform named re-exports: export { x } from './foo'
    fn transform_named_reexport(
        &mut self,
        _export_clause_idx: NodeIndex,
        module_specifier: &str,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        // First, create require call
        let require_call = self.create_require_call(module_specifier, ctx);

        // Store in a temp variable
        let temp_name = ctx.generate_unique_name("tmp");
        let var_stmt = self.create_const_declaration(&temp_name, require_call, ctx);

        // For full implementation, we'd create exports.x = temp.x for each specifier
        // For now, just return the require as a demonstration
        Some(var_stmt)
    }

    /// Create __exportStar(require('./module'), exports) call
    fn create_export_star_call(&self, module_specifier: &str, ctx: &mut TransformContext) -> NodeIndex {
        // Create require('./module')
        let require_call = self.create_require_call(module_specifier, ctx);

        // Create exports identifier
        let exports_id = Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, 0, 0),
            escaped_text: "exports".to_string(),
            original_text: None,
            type_arguments: None,
        };
        let exports_idx = ctx.arena.add(Node::Identifier(exports_id));

        // Create __exportStar identifier
        let helper_id = Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, 0, 0),
            escaped_text: "__exportStar".to_string(),
            original_text: None,
            type_arguments: None,
        };
        let helper_idx = ctx.arena.add(Node::Identifier(helper_id));

        // Create call: __exportStar(require('./module'), exports)
        let mut args = NodeList::new();
        args.push(require_call);
        args.push(exports_idx);

        let call_expr = CallExpression {
            base: NodeBase::new_ext(syntax_kind_ext::CALL_EXPRESSION, 0, 0),
            expression: helper_idx,
            type_arguments: None,
            arguments: args,
        };
        let call_idx = ctx.arena.add(Node::CallExpression(call_expr));

        // Wrap in expression statement
        let expr_stmt = ExpressionStatement {
            base: NodeBase::new_ext(syntax_kind_ext::EXPRESSION_STATEMENT, 0, 0),
            expression: call_idx,
        };
        ctx.arena.add(Node::ExpressionStatement(expr_stmt))
    }

    /// Create exports.name = value expression statement
    fn create_export_assignment(&self, name: &str, value: &str, ctx: &mut TransformContext) -> NodeIndex {
        use crate::parser::ast::BinaryExpression;

        // Create exports.name
        let exports_id = Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, 0, 0),
            escaped_text: "exports".to_string(),
            original_text: None,
            type_arguments: None,
        };
        let exports_idx = ctx.arena.add(Node::Identifier(exports_id));

        let name_id = Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, 0, 0),
            escaped_text: name.to_string(),
            original_text: None,
            type_arguments: None,
        };
        let name_idx = ctx.arena.add(Node::Identifier(name_id));

        let prop_access = PropertyAccessExpression {
            base: NodeBase::new_ext(syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION, 0, 0),
            expression: exports_idx,
            question_dot_token: false,
            name: name_idx,
        };
        let prop_access_idx = ctx.arena.add(Node::PropertyAccessExpression(prop_access));

        // Create value identifier
        let value_id = Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, 0, 0),
            escaped_text: value.to_string(),
            original_text: None,
            type_arguments: None,
        };
        let value_idx = ctx.arena.add(Node::Identifier(value_id));

        // Create assignment: exports.name = value
        let binary = BinaryExpression {
            base: NodeBase::new_ext(syntax_kind_ext::BINARY_EXPRESSION, 0, 0),
            left: prop_access_idx,
            operator_token: SyntaxKind::EqualsToken,
            right: value_idx,
        };
        let binary_idx = ctx.arena.add(Node::BinaryExpression(binary));

        // Wrap in expression statement
        let expr_stmt = ExpressionStatement {
            base: NodeBase::new_ext(syntax_kind_ext::EXPRESSION_STATEMENT, 0, 0),
            expression: binary_idx,
        };
        ctx.arena.add(Node::ExpressionStatement(expr_stmt))
    }

    /// Get identifier text from a node
    fn get_identifier_text(&self, idx: NodeIndex, ctx: &TransformContext) -> Option<String> {
        match ctx.arena.get(idx) {
            Some(Node::Identifier(id)) => Some(id.escaped_text.clone()),
            _ => None,
        }
    }

    /// Transform export assignment (export default or export =)
    ///
    /// ```ts
    /// export default class Foo {}
    /// export = foo;
    /// ```
    ///
    /// To CommonJS:
    /// ```js
    /// exports.default = Foo;
    /// // or for export =
    /// module.exports = foo;
    /// ```
    pub fn transform_export_assignment(
        &mut self,
        _node_idx: NodeIndex,
        _ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        // Transform to exports.default or module.exports
        None
    }

    /// Add __esModule marker to output
    ///
    /// ```js
    /// Object.defineProperty(exports, "__esModule", { value: true });
    /// ```
    pub fn emit_es_module_marker(&self) -> String {
        r#"Object.defineProperty(exports, "__esModule", { value: true });"#.to_string()
    }

    /// Generate AMD define wrapper
    ///
    /// ```js
    /// define(["require", "exports", "./foo"], function (require, exports, foo_1) {
    ///     "use strict";
    ///     Object.defineProperty(exports, "__esModule", { value: true });
    ///     // ... module body ...
    /// });
    /// ```
    pub fn emit_amd_wrapper(&self, _dependencies: &[String], _body: &str) -> String {
        // Generate AMD define() wrapper
        "define([/* deps */], function(/* params */) {\n});\n".to_string()
    }

    /// Generate UMD wrapper
    ///
    /// ```js
    /// (function (factory) {
    ///     if (typeof module === "object" && typeof module.exports === "object") {
    ///         var v = factory(require, exports);
    ///         if (v !== undefined) module.exports = v;
    ///     }
    ///     else if (typeof define === "function" && define.amd) {
    ///         define(["require", "exports"], factory);
    ///     }
    /// })(function (require, exports) {
    ///     // ... module body ...
    /// });
    /// ```
    pub fn emit_umd_wrapper(&self, _dependencies: &[String], _body: &str) -> String {
        // Generate UMD wrapper
        "(function (factory) {\n});\n".to_string()
    }

    /// Generate SystemJS register wrapper
    ///
    /// ```js
    /// System.register(["./foo"], function (exports_1, context_1) {
    ///     "use strict";
    ///     var foo_1;
    ///     var __moduleName = context_1 && context_1.id;
    ///     return {
    ///         setters: [
    ///             function (foo_1_1) { foo_1 = foo_1_1; }
    ///         ],
    ///         execute: function () {
    ///             // ... module body ...
    ///         }
    ///     };
    /// });
    /// ```
    pub fn emit_system_wrapper(&self, _dependencies: &[String], _body: &str) -> String {
        // Generate System.register wrapper
        "System.register([], function() {\n});\n".to_string()
    }
}

/// Node kind for module-related transforms
#[derive(Clone, Copy, PartialEq, Eq)]
enum ModuleNodeKind {
    SourceFile,
    ImportDeclaration,
    ExportDeclaration,
    ExportAssignment,
    Other,
}

fn get_module_node_kind(node_idx: NodeIndex, ctx: &TransformContext) -> ModuleNodeKind {
    match ctx.arena.get(node_idx) {
        Some(Node::SourceFile(_)) => ModuleNodeKind::SourceFile,
        Some(Node::ImportDeclaration(_)) => ModuleNodeKind::ImportDeclaration,
        Some(Node::ExportDeclaration(_)) => ModuleNodeKind::ExportDeclaration,
        Some(Node::ExportAssignment(_)) => ModuleNodeKind::ExportAssignment,
        _ => ModuleNodeKind::Other,
    }
}

impl Transformer for ModuleTransformer {
    fn visit_node(&mut self, node_idx: NodeIndex, ctx: &mut TransformContext) -> Option<NodeIndex> {
        let kind = get_module_node_kind(node_idx, ctx);

        match kind {
            ModuleNodeKind::ImportDeclaration => {
                return self.transform_import_declaration(node_idx, ctx);
            }
            ModuleNodeKind::ExportDeclaration => {
                return self.transform_export_declaration(node_idx, ctx);
            }
            ModuleNodeKind::ExportAssignment => {
                return self.transform_export_assignment(node_idx, ctx);
            }
            _ => {}
        }

        self.visit_children(node_idx, ctx);
        None
    }

    fn visit_children(&mut self, node_idx: NodeIndex, ctx: &mut TransformContext) {
        let children = collect_module_children(node_idx, ctx);

        for child_idx in children {
            self.visit_node(child_idx, ctx);
        }
    }
}

fn collect_module_children(node_idx: NodeIndex, ctx: &TransformContext) -> Vec<NodeIndex> {
    let mut children = Vec::new();

    match ctx.arena.get(node_idx) {
        Some(Node::SourceFile(sf)) => {
            for stmt in &sf.statements.nodes {
                children.push(*stmt);
            }
        }
        Some(Node::Block(block)) => {
            for stmt in &block.statements.nodes {
                children.push(*stmt);
            }
        }
        _ => {}
    }

    children
}

#[cfg(test)]
#[path = "modules_tests.rs"]
mod modules_tests;
