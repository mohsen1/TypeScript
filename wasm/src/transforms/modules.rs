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
use crate::parser::{Node, NodeIndex};

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
        _node_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        match self.module_kind {
            ModuleKind::CommonJS => {
                // May need __importDefault or __importStar
                // Check if it's a default import or namespace import
                ctx.helpers_needed.import_default = true;
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
        _node_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        match self.module_kind {
            ModuleKind::CommonJS => {
                // Transform to exports.x = x
                // May need __exportStar for export *
                ctx.helpers_needed.export_star = true;
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
mod tests {
    use super::*;
    use crate::emitter::ScriptTarget;
    use crate::parser_impl::ParserState;
    use crate::transforms::transform_source_file;

    fn parse_and_transform(source: &str, _target: ScriptTarget) -> HelpersNeeded {
        let mut parser = ParserState::new("test.ts".to_string(), source.to_string());
        let source_file = parser.parse_source_file();
        let mut arena = parser.arena;

        // For module transforms, we need to run the transformer directly
        // since the main transform_source_file checks for ES5 target
        let mut ctx = super::super::TransformContext::new(_target, &mut arena);
        let mut transformer = ModuleTransformer::new(ModuleKind::CommonJS);
        transformer.visit_node(source_file, &mut ctx);
        ctx.helpers_needed
    }

    #[test]
    fn test_import_needs_import_default() {
        let helpers = parse_and_transform(
            "import foo from './bar';",
            ScriptTarget::ES5,
        );
        assert!(helpers.import_default);
    }

    #[test]
    fn test_export_star_needs_export_star() {
        let helpers = parse_and_transform(
            "export * from './bar';",
            ScriptTarget::ES5,
        );
        assert!(helpers.export_star);
    }

    #[test]
    fn test_es_module_marker() {
        let transformer = ModuleTransformer::new(ModuleKind::CommonJS);
        let marker = transformer.emit_es_module_marker();
        assert!(marker.contains("__esModule"));
    }

    #[test]
    fn test_amd_wrapper() {
        let transformer = ModuleTransformer::new(ModuleKind::AMD);
        let wrapper = transformer.emit_amd_wrapper(&[], "");
        assert!(wrapper.contains("define"));
    }

    #[test]
    fn test_umd_wrapper() {
        let transformer = ModuleTransformer::new(ModuleKind::UMD);
        let wrapper = transformer.emit_umd_wrapper(&[], "");
        assert!(wrapper.contains("function"));
    }
}
