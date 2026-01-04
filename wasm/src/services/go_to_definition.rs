//! Go to Definition implementation.
//!
//! This module provides the ability to navigate to symbol definitions,
//! type definitions, and implementations.

use crate::binder::{Symbol, SymbolId, SymbolFlags, SymbolTable};
use crate::checker::{Type, TypeId, CheckerState};
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::text_span::TextSpan;
use super::utilities::{ScriptElementKind, create_text_span_from_bounds};

// =============================================================================
// Definition Info
// =============================================================================

/// Information about a definition location.
/// Matches TypeScript's `DefinitionInfo` interface.
#[derive(Debug, Clone)]
pub struct DefinitionInfo {
    /// The file containing the definition.
    pub file_name: String,
    /// The span of the definition.
    pub text_span: TextSpan,
    /// The kind of the definition.
    pub kind: ScriptElementKind,
    /// The name of the symbol.
    pub name: String,
    /// The container name (e.g., class name for a method).
    pub container_name: String,
    /// The container kind.
    pub container_kind: ScriptElementKind,
    /// Whether this is a local definition.
    pub is_local: bool,
    /// Whether this is an ambient declaration.
    pub is_ambient: bool,
    /// Whether this is a write access (for references).
    pub is_write_access: Option<bool>,
    /// Unverified - from go-to-source-definition.
    pub unverified: Option<bool>,
    /// Fail message if lookup failed.
    pub fail_message: Option<String>,
}

impl DefinitionInfo {
    pub fn new(
        file_name: String,
        text_span: TextSpan,
        kind: ScriptElementKind,
        name: String,
    ) -> Self {
        Self {
            file_name,
            text_span,
            kind,
            name,
            container_name: String::new(),
            container_kind: ScriptElementKind::Unknown,
            is_local: false,
            is_ambient: false,
            is_write_access: None,
            unverified: None,
            fail_message: None,
        }
    }

    pub fn with_container(mut self, name: String, kind: ScriptElementKind) -> Self {
        self.container_name = name;
        self.container_kind = kind;
        self
    }
}

/// Definition info with the triggering span.
/// Matches TypeScript's `DefinitionInfoAndBoundSpan` interface.
#[derive(Debug, Clone)]
pub struct DefinitionInfoAndBoundSpan {
    pub definitions: Vec<DefinitionInfo>,
    pub text_span: TextSpan,
}

impl DefinitionInfoAndBoundSpan {
    pub fn new(definitions: Vec<DefinitionInfo>, text_span: TextSpan) -> Self {
        Self { definitions, text_span }
    }

    pub fn empty() -> Self {
        Self {
            definitions: Vec::new(),
            text_span: TextSpan::new(0, 0),
        }
    }
}

// =============================================================================
// Go To Definition Context
// =============================================================================

/// Context for go-to-definition operations.
pub struct GoToDefinitionContext<'a> {
    pub arena: &'a NodeArena,
    pub checker: &'a CheckerState,
    pub source_text: &'a str,
    pub file_name: &'a str,
}

// =============================================================================
// Go To Definition Implementation
// =============================================================================

/// Get definition at a position.
pub fn get_definition_at_position(
    ctx: &GoToDefinitionContext,
    position: u32,
) -> Option<Vec<DefinitionInfo>> {
    // Find the node at the position
    let node_id = find_node_at_position(ctx.arena, position)?;

    // Get the symbol for this node
    let symbol = get_symbol_at_location(ctx, node_id)?;

    // Get definitions from the symbol's declarations
    let definitions = get_definitions_from_symbol(ctx, &symbol);

    if definitions.is_empty() {
        None
    } else {
        Some(definitions)
    }
}

/// Get definition and bound span at a position.
pub fn get_definition_and_bound_span(
    ctx: &GoToDefinitionContext,
    position: u32,
) -> DefinitionInfoAndBoundSpan {
    // Find the node at the position
    let node_id = match find_node_at_position(ctx.arena, position) {
        Some(id) => id,
        None => return DefinitionInfoAndBoundSpan::empty(),
    };

    // Get the span of the triggering node
    let text_span = match ctx.arena.get(node_id) {
        Some(node) => TextSpan::from_bounds(node.pos(), node.end()),
        None => TextSpan::new(position, 0),
    };

    // Get definitions
    let definitions = get_definition_at_position(ctx, position).unwrap_or_default();

    DefinitionInfoAndBoundSpan::new(definitions, text_span)
}

/// Get type definition at a position.
pub fn get_type_definition_at_position(
    ctx: &GoToDefinitionContext,
    position: u32,
) -> Option<Vec<DefinitionInfo>> {
    // Find the node at the position
    let node_id = find_node_at_position(ctx.arena, position)?;

    // Get the type of this node
    let type_id = ctx.checker.get_type_at_location(node_id)?;

    // Get the symbol of the type
    let type_symbol = ctx.checker.get_type_symbol(type_id)?;

    // Get definitions from the type's symbol
    let definitions = get_definitions_from_symbol_id(ctx, type_symbol);

    if definitions.is_empty() {
        None
    } else {
        Some(definitions)
    }
}

/// Get implementation at a position (for interfaces/abstract classes).
pub fn get_implementation_at_position(
    ctx: &GoToDefinitionContext,
    position: u32,
) -> Option<Vec<DefinitionInfo>> {
    // Find the node at the position
    let node_id = find_node_at_position(ctx.arena, position)?;

    // Get the symbol for this node
    let symbol = get_symbol_at_location(ctx, node_id)?;

    // If it's an interface or abstract member, find implementations
    if symbol.flags.contains(SymbolFlags::Interface) {
        // In a full implementation, this would search for classes
        // that implement this interface
        return None;
    }

    // Otherwise, fall back to definition
    get_definition_at_position(ctx, position)
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Find the node at a position in the AST.
fn find_node_at_position(arena: &NodeArena, position: u32) -> Option<NodeId> {
    // This would do a proper AST walk to find the deepest node
    // containing the position. For now, simplified implementation.

    // Start from the root and walk down
    let root = arena.root()?;
    find_deepest_node_at_position(arena, root, position)
}

fn find_deepest_node_at_position(
    arena: &NodeArena,
    node_id: NodeId,
    position: u32,
) -> Option<NodeId> {
    let node = arena.get(node_id)?;

    // Check if position is within this node
    if position < node.pos() || position >= node.end() {
        return None;
    }

    // Check children
    for child_id in node.children() {
        if let Some(deeper) = find_deepest_node_at_position(arena, child_id, position) {
            return Some(deeper);
        }
    }

    // No child contains the position, return this node
    Some(node_id)
}

/// Get the symbol at a node location.
fn get_symbol_at_location(ctx: &GoToDefinitionContext, node_id: NodeId) -> Option<Symbol> {
    // In a full implementation, this would use the type checker
    // to resolve the symbol at this location
    ctx.checker.get_symbol_at_location(node_id)
}

/// Get definitions from a symbol.
fn get_definitions_from_symbol(ctx: &GoToDefinitionContext, symbol: &Symbol) -> Vec<DefinitionInfo> {
    let mut definitions = Vec::new();

    for decl_id in &symbol.declarations {
        if let Some(node) = ctx.arena.get(*decl_id) {
            let span = TextSpan::from_bounds(node.pos(), node.end());
            let kind = super::utilities::get_symbol_kind(symbol);

            let def = DefinitionInfo::new(
                ctx.file_name.to_string(),
                span,
                kind,
                symbol.escaped_name.clone(),
            );

            definitions.push(def);
        }
    }

    definitions
}

/// Get definitions from a symbol ID.
fn get_definitions_from_symbol_id(ctx: &GoToDefinitionContext, symbol_id: SymbolId) -> Vec<DefinitionInfo> {
    // In a full implementation, this would look up the symbol
    // and get its definitions
    Vec::new()
}

// =============================================================================
// Special Cases
// =============================================================================

/// Handle go-to-definition for module specifiers.
pub fn get_definition_for_module_specifier(
    file_name: &str,
    module_specifier: &str,
) -> Option<DefinitionInfo> {
    // In a full implementation, this would resolve the module
    // and return the definition of the module file
    None
}

/// Handle go-to-definition for string literals (in certain contexts).
pub fn get_definition_for_string_literal(
    ctx: &GoToDefinitionContext,
    literal_text: &str,
    position: u32,
) -> Option<Vec<DefinitionInfo>> {
    // In a full implementation, this would handle:
    // - Module specifiers in imports
    // - Property names in object literals
    // - Type members accessed via string
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition_info() {
        let def = DefinitionInfo::new(
            "test.ts".to_string(),
            TextSpan::new(10, 5),
            ScriptElementKind::VariableElement,
            "foo".to_string(),
        );

        assert_eq!(def.file_name, "test.ts");
        assert_eq!(def.name, "foo");
        assert_eq!(def.text_span.start, 10);
        assert_eq!(def.text_span.length, 5);
    }

    #[test]
    fn test_definition_info_with_container() {
        let def = DefinitionInfo::new(
            "test.ts".to_string(),
            TextSpan::new(10, 5),
            ScriptElementKind::MemberFunctionElement,
            "doSomething".to_string(),
        )
        .with_container("MyClass".to_string(), ScriptElementKind::ClassElement);

        assert_eq!(def.container_name, "MyClass");
        assert_eq!(def.container_kind, ScriptElementKind::ClassElement);
    }
}
