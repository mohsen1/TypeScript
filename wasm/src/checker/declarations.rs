//! Declaration Type Checking
//!
//! Handles classes, interfaces, functions, and variable declarations.
//! This module separates declaration checking logic from the monolithic ThinCheckerState.

use super::context::CheckerContext;
use crate::checker::types::diagnostics::{diagnostic_codes, diagnostic_messages};
use crate::parser::NodeIndex;
use crate::parser::syntax_kind_ext;
use crate::scanner::SyntaxKind;

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
    pub fn check_class_declaration(&mut self, class_idx: NodeIndex) {
        let Some(node) = self.ctx.arena.get(class_idx) else {
            return;
        };

        // Get the class data
        let Some(class_decl) = self.ctx.arena.get_class(node) else {
            return;
        };

        // Skip ambient classes (declare keyword)
        if self
            .ctx
            .has_modifier(&class_decl.modifiers, SyntaxKind::DeclareKeyword as u16)
        {
            return;
        }

        // Check property initialization if strictPropertyInitialization is enabled
        if self.ctx.strict_property_initialization {
            self.check_property_initialization(class_idx, class_decl);
        }

        // Additional class checks will be added here:
        // - Heritage clauses (extends/implements)
        // - Member types and modifiers
        // - Abstract implementation requirements
        // - Constructor parameter properties
    }

    /// Check property initialization for TS2564.
    ///
    /// Reports errors for class properties that:
    /// - Don't have initializers
    /// - Don't have definite assignment assertions (!)
    /// - Are not assigned in all constructor code paths
    fn check_property_initialization(
        &mut self,
        _class_idx: NodeIndex,
        class_decl: &crate::parser::thin_node::ClassData,
    ) {
        // Iterate over class members
        for &member_idx in &class_decl.members.nodes {
            let Some(member_node) = self.ctx.arena.get(member_idx) else {
                continue;
            };

            // Only check PropertyDeclaration nodes
            if member_node.kind != syntax_kind_ext::PROPERTY_DECLARATION {
                continue;
            }

            let Some(prop) = self.ctx.arena.get_property_decl(member_node) else {
                continue;
            };

            // Skip if property has definite assignment assertion (!)
            if prop.exclamation_token {
                continue;
            }

            // Skip if property has an initializer
            if !prop.initializer.is_none() {
                continue;
            }

            // Skip static properties
            if self
                .ctx
                .has_modifier(&prop.modifiers, SyntaxKind::StaticKeyword as u16)
            {
                continue;
            }

            // Skip abstract properties
            if self
                .ctx
                .has_modifier(&prop.modifiers, SyntaxKind::AbstractKeyword as u16)
            {
                continue;
            }

            // Skip ambient properties (declare keyword)
            if self
                .ctx
                .has_modifier(&prop.modifiers, SyntaxKind::DeclareKeyword as u16)
            {
                continue;
            }

            // Get property name for error message
            let prop_name = self.get_property_name(prop.name);

            // Report TS2564 error
            // TODO: Add control flow analysis to check if property is initialized in constructor
            // For now, we report on all properties without initializers
            let message =
                diagnostic_messages::PROPERTY_HAS_NO_INITIALIZER.replace("{0}", &prop_name);

            // Get the span for the property name
            if let Some((pos, end)) = self.ctx.get_node_span(prop.name) {
                self.ctx.error(
                    pos,
                    end - pos,
                    message,
                    diagnostic_codes::PROPERTY_HAS_NO_INITIALIZER,
                );
            }
        }
    }

    /// Get the name of a property as a string.
    fn get_property_name(&self, name_idx: NodeIndex) -> String {
        if let Some(name_node) = self.ctx.arena.get(name_idx) {
            if let Some(ident) = self.ctx.arena.get_identifier(name_node) {
                return ident.escaped_text.clone();
            }
            // Handle computed properties and private identifiers
            "[computed]".to_string()
        } else {
            "[unknown]".to_string()
        }
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
        use crate::checker::types::diagnostics::{diagnostic_codes, diagnostic_messages};
        use crate::scanner::SyntaxKind;

        let Some(node) = self.ctx.arena.get(module_idx) else {
            return;
        };

        if let Some(module) = self.ctx.arena.get_module(node) {
            // TS5061: Check for relative module names in ambient declarations
            // declare module "./foo" { } -> Error
            if self
                .ctx
                .has_modifier(&module.modifiers, SyntaxKind::DeclareKeyword as u16)
            {
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
    use crate::checker::types::diagnostics::diagnostic_codes;
    use crate::solver::TypeInterner;
    use crate::thin_binder::ThinBinderState;
    use crate::thin_parser::ThinParserState;

    #[test]
    fn test_declaration_checker_variable() {
        let source = "let x = 1;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let types = TypeInterner::new();
        let mut ctx =
            CheckerContext::new(parser.get_arena(), &binder, &types, "test.ts".to_string(), false);

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

    #[test]
    fn test_ts2564_property_without_initializer() {
        // Test that TS2564 is reported for properties without initializers
        let source = r#"
class Foo {
    x: number;  // Should report TS2564
    y: string = "hello";  // Should NOT report (has initializer)
}
"#;
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
            true, // strict = true
        );

        // Get the class declaration
        if let Some(root_node) = parser.get_arena().get(root) {
            if let Some(sf_data) = parser.get_arena().get_source_file(root_node) {
                if let Some(&stmt_idx) = sf_data.statements.nodes.first() {
                    let mut checker = DeclarationChecker::new(&mut ctx);
                    checker.check(stmt_idx);

                    // Should have one TS2564 error for property 'x'
                    let ts2564_errors: Vec<_> = ctx
                        .diagnostics
                        .iter()
                        .filter(|d| d.code == diagnostic_codes::PROPERTY_HAS_NO_INITIALIZER)
                        .collect();

                    assert_eq!(
                        ts2564_errors.len(),
                        1,
                        "Expected 1 TS2564 error, got {}",
                        ts2564_errors.len()
                    );

                    // Verify the error message contains 'x'
                    if let Some(err) = ts2564_errors.first() {
                        assert!(
                            err.message_text.contains("x"),
                            "Error message should contain 'x', got: {}",
                            err.message_text
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_ts2564_with_definite_assignment_assertion() {
        // Test that TS2564 is NOT reported for properties with definite assignment assertion (!)
        let source = r#"
class Foo {
    x!: number;  // Should NOT report (has definite assignment assertion)
}
"#;
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
            true, // strict = true
        );

        // Get the class declaration
        if let Some(root_node) = parser.get_arena().get(root) {
            if let Some(sf_data) = parser.get_arena().get_source_file(root_node) {
                if let Some(&stmt_idx) = sf_data.statements.nodes.first() {
                    let mut checker = DeclarationChecker::new(&mut ctx);
                    checker.check(stmt_idx);

                    // Should have NO TS2564 errors
                    let ts2564_errors: Vec<_> = ctx
                        .diagnostics
                        .iter()
                        .filter(|d| d.code == diagnostic_codes::PROPERTY_HAS_NO_INITIALIZER)
                        .collect();

                    assert_eq!(
                        ts2564_errors.len(),
                        0,
                        "Expected 0 TS2564 errors, got {}",
                        ts2564_errors.len()
                    );
                }
            }
        }
    }

    #[test]
    fn test_ts2564_skips_static_properties() {
        // Test that TS2564 is NOT reported for static properties
        let source = r#"
class Foo {
    static x: number;  // Should NOT report (static property)
}
"#;
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
            true, // strict = true
        );

        // Get the class declaration
        if let Some(root_node) = parser.get_arena().get(root) {
            if let Some(sf_data) = parser.get_arena().get_source_file(root_node) {
                if let Some(&stmt_idx) = sf_data.statements.nodes.first() {
                    let mut checker = DeclarationChecker::new(&mut ctx);
                    checker.check(stmt_idx);

                    // Should have NO TS2564 errors
                    let ts2564_errors: Vec<_> = ctx
                        .diagnostics
                        .iter()
                        .filter(|d| d.code == diagnostic_codes::PROPERTY_HAS_NO_INITIALIZER)
                        .collect();

                    assert_eq!(
                        ts2564_errors.len(),
                        0,
                        "Expected 0 TS2564 errors, got {}",
                        ts2564_errors.len()
                    );
                }
            }
        }
    }

    #[test]
    fn test_ts2564_disabled_when_strict_false() {
        // Test that TS2564 is NOT reported when strict mode is disabled
        let source = r#"
class Foo {
    x: number;  // Should NOT report (strict mode disabled)
}
"#;
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
            false, // strict = false
        );

        // Get the class declaration
        if let Some(root_node) = parser.get_arena().get(root) {
            if let Some(sf_data) = parser.get_arena().get_source_file(root_node) {
                if let Some(&stmt_idx) = sf_data.statements.nodes.first() {
                    let mut checker = DeclarationChecker::new(&mut ctx);
                    checker.check(stmt_idx);

                    // Should have NO TS2564 errors (strict mode disabled)
                    let ts2564_errors: Vec<_> = ctx
                        .diagnostics
                        .iter()
                        .filter(|d| d.code == diagnostic_codes::PROPERTY_HAS_NO_INITIALIZER)
                        .collect();

                    assert_eq!(
                        ts2564_errors.len(),
                        0,
                        "Expected 0 TS2564 errors when strict mode disabled, got {}",
                        ts2564_errors.len()
                    );
                }
            }
        }
    }
}
