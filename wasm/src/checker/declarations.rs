//! Declaration Type Checking
//!
//! Handles classes, interfaces, functions, and variable declarations.
//! This module separates declaration checking logic from the monolithic ThinCheckerState.

use crate::parser::NodeIndex;
use crate::parser::syntax_kind_ext;
use super::context::CheckerContext;

/// Declaration type checker that operates on the shared context.
///
/// This is a stateless checker that borrows the context mutably.
/// All declaration type checking goes through this checker.
pub struct DeclarationChecker<'a, 'ctx> {
    pub ctx: &'a mut CheckerContext<'ctx>,
}

impl<'a, 'ctx> DeclarationChecker<'a, 'ctx> {
    /// Create a new declaration checker with a mutable context reference.
    pub fn new(ctx: &'a mut CheckerContext<'ctx>) -> Self {
        Self { ctx }
    }

    /// Check a declaration node.
    ///
    /// This dispatches to specialized handlers based on declaration kind.
    /// Currently a skeleton - logic will be migrated incrementally from ThinCheckerState.
    pub fn check(&mut self, decl_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(decl_idx) else {
            return;
        };

        match node.kind {
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                self.check_variable_statement(decl_idx);
            }
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                self.check_function_declaration(decl_idx);
            }
            k if k == syntax_kind_ext::CLASS_DECLARATION => {
                self.check_class_declaration(decl_idx);
            }
            k if k == syntax_kind_ext::INTERFACE_DECLARATION => {
                self.check_interface_declaration(decl_idx);
            }
            k if k == syntax_kind_ext::TYPE_ALIAS_DECLARATION => {
                self.check_type_alias_declaration(decl_idx);
            }
            k if k == syntax_kind_ext::ENUM_DECLARATION => {
                self.check_enum_declaration(decl_idx);
            }
            k if k == syntax_kind_ext::MODULE_DECLARATION => {
                self.check_module_declaration(decl_idx);
            }
            _ => {
                // Unhandled declaration types - will be expanded incrementally
            }
        }
    }

    /// Check a variable statement.
    pub fn check_variable_statement(&mut self, stmt_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(stmt_idx) else {
            return;
        };

        if let Some(var_stmt) = self.ctx.arena.get_variable(node) {
            for &decl_idx in &var_stmt.declarations.nodes {
                self.check_variable_declaration(decl_idx);
            }
        }
    }

    /// Check a variable declaration list.
    pub fn check_variable_declaration_list(&mut self, list_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(list_idx) else {
            return;
        };

        if let Some(var_list) = self.ctx.arena.get_variable(node) {
            for &decl_idx in &var_list.declarations.nodes {
                self.check_variable_declaration(decl_idx);
            }
        }
    }

    /// Check a variable declaration.
    pub fn check_variable_declaration(&mut self, _decl_idx: NodeIndex) {
        // Variable declaration checking is handled by ThinCheckerState for now
        // Will be migrated incrementally
        // Key checks:
        // - Type annotation vs initializer type compatibility
        // - Adding variable to scope
    }

    /// Check a function declaration.
    pub fn check_function_declaration(&mut self, _func_idx: NodeIndex) {
        // Function declaration checking is handled by ThinCheckerState for now
        // Will be migrated incrementally
        // Key checks:
        // - Parameter types
        // - Return type vs actual returns
        // - Body statements
    }

    /// Check a class declaration.
    pub fn check_class_declaration(&mut self, _class_idx: NodeIndex) {
        // Class declaration checking is handled by ThinCheckerState for now
        // Will be migrated incrementally
        // Key checks:
        // - Heritage clauses (extends/implements)
        // - Member types and modifiers
        // - Abstract implementation requirements
        // - Constructor parameter properties
    }

    /// Check an interface declaration.
    pub fn check_interface_declaration(&mut self, _iface_idx: NodeIndex) {
        // Interface declaration checking is handled by ThinCheckerState for now
        // Will be migrated incrementally
        // Key checks:
        // - Heritage clauses
        // - Member signatures
    }

    /// Check a type alias declaration.
    pub fn check_type_alias_declaration(&mut self, _alias_idx: NodeIndex) {
        // Type alias checking is handled by ThinCheckerState for now
        // Will be migrated incrementally
        // Key checks:
        // - Type parameters
        // - Circular reference detection
    }

    /// Check an enum declaration.
    pub fn check_enum_declaration(&mut self, _enum_idx: NodeIndex) {
        // Enum declaration checking is handled by ThinCheckerState for now
        // Will be migrated incrementally
        // Key checks:
        // - Member types (numeric vs string)
        // - Computed members
    }

    /// Check a module/namespace declaration.
    pub fn check_module_declaration(&mut self, module_idx: NodeIndex) {
        use crate::scanner::SyntaxKind;
        use crate::checker::types::diagnostics::{diagnostic_codes, diagnostic_messages};

        let Some(node) = self.ctx.arena.get(module_idx) else {
            return;
        };

        if let Some(module) = self.ctx.arena.get_module(node) {
            // TS5061: Check for relative module names in ambient declarations
            // declare module "./foo" { } -> Error
            if self.ctx.has_modifier(&module.modifiers, SyntaxKind::DeclareKeyword as u16) {
                if let Some(name_node) = self.ctx.arena.get(module.name) {
                    if name_node.kind == SyntaxKind::StringLiteral as u16 {
                        if let Some(lit) = self.ctx.arena.get_literal(name_node) {
                            if self.is_relative_module_name(&lit.text) {
                                self.ctx.error(
                                    name_node.pos,
                                    name_node.end - name_node.pos,
                                    diagnostic_messages::AMBIENT_MODULE_DECLARATION_CANNOT_SPECIFY_RELATIVE_MODULE_NAME.to_string(),
                                    diagnostic_codes::AMBIENT_MODULE_DECLARATION_CANNOT_SPECIFY_RELATIVE_MODULE_NAME,
                                );
                            }
                        }
                    }
                }
            }

            if !module.body.is_none() {
                // Check module body (which can be a block or nested module)
                self.check_module_body(module.body);
            }
        }
    }

    /// Check if a module name is relative (starts with ./ or ../)
    fn is_relative_module_name(&self, name: &str) -> bool {
        name.starts_with("./") || name.starts_with("../") || name == "." || name == ".."
    }

    /// Check a module body (block or nested module).
    fn check_module_body(&mut self, body_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(body_idx) else {
            return;
        };

        if node.kind == syntax_kind_ext::MODULE_BLOCK {
            if let Some(block) = self.ctx.arena.get_module_block(node) {
                if let Some(ref stmts) = block.statements {
                    for &stmt_idx in &stmts.nodes {
                        // Dispatch to statement/declaration checking
                        // Currently a no-op - will call StatementChecker
                        let _ = stmt_idx;
                    }
                }
            }
        } else if node.kind == syntax_kind_ext::MODULE_DECLARATION {
            // Nested module
            self.check_module_declaration(body_idx);
        }
    }

    /// Check parameter properties (only valid in constructors).
    pub fn check_parameter_properties(&mut self, parameters: &[NodeIndex]) {
        use crate::checker::types::diagnostics::diagnostic_codes;

        for &param_idx in parameters {
            let Some(node) = self.ctx.arena.get(param_idx) else {
                continue;
            };

            if let Some(param) = self.ctx.arena.get_parameter(node) {
                // If parameter has accessibility modifiers (public/private/protected/readonly)
                // and we're not in a constructor, report error
                if param.modifiers.is_some() {
                    if let Some((pos, end)) = self.ctx.get_node_span(param_idx) {
                        self.ctx.error(
                            pos,
                            end - pos,
                            "A parameter property is only allowed in a constructor implementation."
                                .to_string(),
                            diagnostic_codes::PARAMETER_PROPERTY_NOT_ALLOWED,
                        );
                    }
                }
            }
        }
    }

    /// Check function implementations for overload sequences.
    pub fn check_function_implementations(&mut self, _nodes: &[NodeIndex]) {
        // Implementation of overload checking
        // Will be migrated from ThinCheckerState
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thin_parser::ThinParserState;
    use crate::thin_binder::ThinBinderState;
    use crate::solver::TypeInterner;

    #[test]
    fn test_declaration_checker_variable() {
        let source = "let x = 1;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let types = TypeInterner::new();
        let mut ctx = CheckerContext::new(
            parser.get_arena(),
            &binder,
            &types,
            "test.ts".to_string(),
        );

        // Get the variable statement
        if let Some(root_node) = parser.get_arena().get(root) {
            if let Some(sf_data) = parser.get_arena().get_source_file(root_node) {
                if let Some(&stmt_idx) = sf_data.statements.nodes.first() {
                    let mut checker = DeclarationChecker::new(&mut ctx);
                    checker.check(stmt_idx);
                    // Test passes if no panic
                }
            }
        }
    }
}
