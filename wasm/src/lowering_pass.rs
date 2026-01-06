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
use crate::transform_context::{TransformContext, TransformDirective};

/// Lowering pass - Phase 1 of emission
///
/// Walks the AST and produces transform directives based on compiler options.
pub struct LoweringPass<'a> {
    arena: &'a ThinNodeArena,
    ctx: &'a EmitContext,
    transforms: TransformContext,
}

impl<'a> LoweringPass<'a> {
    /// Create a new lowering pass
    pub fn new(arena: &'a ThinNodeArena, ctx: &'a EmitContext) -> Self {
        LoweringPass {
            arena,
            ctx,
            transforms: TransformContext::new(),
        }
    }

    /// Run the lowering pass on a source file and return the transform context
    pub fn run(mut self, source_file: NodeIndex) -> TransformContext {
        self.visit(source_file);
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
            k if k == syntax_kind_ext::ARROW_FUNCTION => self.visit_arrow_function(node, idx),
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => self.visit_variable_statement(node, idx),
            k if k == syntax_kind_ext::EXPORT_DECLARATION => self.visit_export_declaration(node, idx),
            _ => self.visit_children(idx),
        }
    }

    /// Visit all children of a node
    fn visit_children(&mut self, idx: NodeIndex) {
        let Some(node) = self.arena.get(idx) else {
            return;
        };

        // For now, we'll implement a generic child visitor
        // In the full implementation, we'd need to handle all node types
        // This is a simplified version for the architecture refactor
        match node.kind {
            k if k == syntax_kind_ext::SOURCE_FILE => {
                if let Some(sf) = self.arena.get_source_file(node) {
                    for &stmt in &sf.statements.nodes {
                        self.visit(stmt);
                    }
                }
            }
            _ => {
                // Generic traversal - would need to be expanded for all node types
            }
        }
    }

    /// Visit a class declaration
    fn visit_class_declaration(&mut self, node: &ThinNode, idx: NodeIndex) {
        self.lower_class_declaration(node, idx, false, false);
    }

    fn visit_export_declaration(&mut self, node: &ThinNode, _idx: NodeIndex) {
        let Some(export_decl) = self.arena.get_export_decl(node) else {
            return;
        };

        if export_decl.export_clause.is_none() {
            return;
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

        // Skip ambient declarations (declare class)
        if self.has_declare_modifier(&class.modifiers) {
            return;
        }

        let mut is_exported = self.ctx.is_commonjs()
            && !self.ctx.module_state.has_export_assignment
            && (force_export || self.has_export_modifier(&class.modifiers));

        if force_export && self.ctx.is_commonjs() && !self.ctx.module_state.has_export_assignment {
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
            TransformDirective::CommonJSExport {
                name: class_name.unwrap(),
                is_default,
                inner: Box::new(base_directive),
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

    /// Visit a function declaration
    fn visit_function_declaration(&mut self, node: &ThinNode, idx: NodeIndex) {
        let Some(func) = self.arena.get_function(node) else {
            return;
        };

        // Check if this is an async function targeting ES5
        if self.ctx.target_es5 && self.has_async_modifier(idx) {
            self.transforms.insert(
                idx,
                TransformDirective::ES5AsyncFunction {
                    function_node: idx,
                },
            );
        }

        // TODO: Handle CommonJS exports for functions

        // Visit children
        if !func.body.is_none() {
            self.visit(func.body);
        }
    }

    /// Visit an arrow function
    fn visit_arrow_function(&mut self, node: &ThinNode, idx: NodeIndex) {
        let Some(_arrow) = self.arena.get_function(node) else {
            return;
        };

        if self.ctx.target_es5 {
            // TODO: Analyze if this arrow function captures 'this'
            let captures_this = false; // Simplified for now

            self.transforms.insert(
                idx,
                TransformDirective::ES5ArrowFunction {
                    arrow_node: idx,
                    captures_this,
                },
            );
        }

        // TODO: Visit children
    }

    /// Visit a variable statement
    fn visit_variable_statement(&mut self, node: &ThinNode, idx: NodeIndex) {
        let Some(var_stmt) = self.arena.get_variable(node) else {
            return;
        };

        // TODO: Handle exported variable statements in CommonJS
        // TODO: Handle const/let -> var transformation for ES5

        // Visit each declaration
        for &decl in &var_stmt.declarations.nodes {
            self.visit(decl);
        }
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

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
}
