//! Navigation Bar implementation.
//!
//! This module provides the navigation bar (outline view) for IDE features,
//! showing a hierarchical view of declarations in a file.

use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::text_span::TextSpan;
use super::utilities::ScriptElementKind;

// =============================================================================
// Navigation Bar Item
// =============================================================================

/// An item in the navigation bar.
/// Matches TypeScript's `NavigationBarItem` interface.
#[derive(Debug, Clone)]
pub struct NavigationBarItem {
    /// The display text.
    pub text: String,
    /// The kind of item.
    pub kind: ScriptElementKind,
    /// Kind modifiers (e.g., "export", "static").
    pub kind_modifiers: String,
    /// Spans for this item (can have multiple for partial classes).
    pub spans: Vec<TextSpan>,
    /// Child items.
    pub child_items: Vec<NavigationBarItem>,
    /// Indentation level.
    pub indent: u32,
    /// Whether the item has a bold label.
    pub bold_label: bool,
    /// Whether the item is grayed out.
    pub grayed: bool,
}

impl NavigationBarItem {
    pub fn new(text: String, kind: ScriptElementKind, spans: Vec<TextSpan>) -> Self {
        Self {
            text,
            kind,
            kind_modifiers: String::new(),
            spans,
            child_items: Vec::new(),
            indent: 0,
            bold_label: false,
            grayed: false,
        }
    }

    pub fn with_children(mut self, children: Vec<NavigationBarItem>) -> Self {
        self.child_items = children;
        self
    }

    pub fn with_indent(mut self, indent: u32) -> Self {
        self.indent = indent;
        self
    }
}

/// Navigation tree (hierarchical structure).
/// Matches TypeScript's `NavigationTree` interface.
#[derive(Debug, Clone)]
pub struct NavigationTree {
    /// The display text.
    pub text: String,
    /// The kind of item.
    pub kind: ScriptElementKind,
    /// Kind modifiers.
    pub kind_modifiers: String,
    /// Spans for this item.
    pub spans: Vec<TextSpan>,
    /// The full span including leading/trailing trivia.
    pub name_span: Option<TextSpan>,
    /// Child items.
    pub child_items: Option<Vec<NavigationTree>>,
}

impl NavigationTree {
    pub fn new(text: String, kind: ScriptElementKind, spans: Vec<TextSpan>) -> Self {
        Self {
            text,
            kind,
            kind_modifiers: String::new(),
            spans,
            name_span: None,
            child_items: None,
        }
    }

    pub fn with_children(mut self, children: Vec<NavigationTree>) -> Self {
        self.child_items = Some(children);
        self
    }
}

// =============================================================================
// Navigation Bar Context
// =============================================================================

/// Context for navigation bar operations.
pub struct NavigationBarContext<'a> {
    pub arena: &'a NodeArena,
    pub source_text: &'a str,
    pub file_name: &'a str,
}

// =============================================================================
// Get Navigation Bar Items
// =============================================================================

/// Get navigation bar items for a source file.
pub fn get_navigation_bar_items(ctx: &NavigationBarContext) -> Vec<NavigationBarItem> {
    let root = match ctx.arena.root() {
        Some(id) => id,
        None => return Vec::new(),
    };

    let mut items = Vec::new();

    // Process top-level declarations
    if let Some(root_node) = ctx.arena.get(root) {
        for child_id in root_node.children() {
            if let Some(item) = get_navigation_item(ctx, child_id, 0) {
                items.push(item);
            }
        }
    }

    items
}

/// Get navigation tree for a source file.
pub fn get_navigation_tree(ctx: &NavigationBarContext) -> NavigationTree {
    let root = match ctx.arena.root() {
        Some(id) => id,
        None => {
            return NavigationTree::new(
                ctx.file_name.to_string(),
                ScriptElementKind::ModuleElement,
                Vec::new(),
            );
        }
    };

    let root_node = match ctx.arena.get(root) {
        Some(n) => n,
        None => {
            return NavigationTree::new(
                ctx.file_name.to_string(),
                ScriptElementKind::ModuleElement,
                Vec::new(),
            );
        }
    };

    let span = TextSpan::from_bounds(root_node.pos(), root_node.end());
    let mut children = Vec::new();

    // Process top-level declarations
    for child_id in root_node.children() {
        if let Some(tree) = get_navigation_tree_item(ctx, child_id) {
            children.push(tree);
        }
    }

    NavigationTree::new(
        ctx.file_name.to_string(),
        ScriptElementKind::ModuleElement,
        vec![span],
    )
    .with_children(children)
}

// =============================================================================
// Get Navigation Item
// =============================================================================

fn get_navigation_item(
    ctx: &NavigationBarContext,
    node_id: NodeId,
    indent: u32,
) -> Option<NavigationBarItem> {
    let node = ctx.arena.get(node_id)?;
    let kind = node.kind();

    match kind {
        // Declarations that appear in navigation bar
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
            get_class_navigation_item(ctx, node_id, indent)
        }
        SyntaxKind::InterfaceDeclaration => {
            get_interface_navigation_item(ctx, node_id, indent)
        }
        SyntaxKind::TypeAliasDeclaration => {
            get_type_alias_navigation_item(ctx, node_id, indent)
        }
        SyntaxKind::EnumDeclaration => {
            get_enum_navigation_item(ctx, node_id, indent)
        }
        SyntaxKind::FunctionDeclaration => {
            get_function_navigation_item(ctx, node_id, indent)
        }
        SyntaxKind::VariableStatement => {
            get_variable_navigation_item(ctx, node_id, indent)
        }
        SyntaxKind::ModuleDeclaration => {
            get_module_navigation_item(ctx, node_id, indent)
        }
        SyntaxKind::ImportDeclaration | SyntaxKind::ExportDeclaration => {
            // Skip imports and exports in navigation bar
            None
        }
        _ => None,
    }
}

fn get_navigation_tree_item(
    ctx: &NavigationBarContext,
    node_id: NodeId,
) -> Option<NavigationTree> {
    let node = ctx.arena.get(node_id)?;
    let kind = node.kind();

    match kind {
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
            get_class_navigation_tree(ctx, node_id)
        }
        SyntaxKind::InterfaceDeclaration => {
            get_interface_navigation_tree(ctx, node_id)
        }
        SyntaxKind::TypeAliasDeclaration => {
            get_type_alias_navigation_tree(ctx, node_id)
        }
        SyntaxKind::EnumDeclaration => {
            get_enum_navigation_tree(ctx, node_id)
        }
        SyntaxKind::FunctionDeclaration => {
            get_function_navigation_tree(ctx, node_id)
        }
        SyntaxKind::VariableStatement => {
            get_variable_navigation_tree(ctx, node_id)
        }
        SyntaxKind::ModuleDeclaration => {
            get_module_navigation_tree(ctx, node_id)
        }
        _ => None,
    }
}

// =============================================================================
// Specific Navigation Items
// =============================================================================

fn get_class_navigation_item(
    ctx: &NavigationBarContext,
    node_id: NodeId,
    indent: u32,
) -> Option<NavigationBarItem> {
    let node = ctx.arena.get(node_id)?;
    let name = get_declaration_name(ctx, node_id).unwrap_or_else(|| "<anonymous>".to_string());
    let span = TextSpan::from_bounds(node.pos(), node.end());

    let mut item = NavigationBarItem::new(name, ScriptElementKind::ClassElement, vec![span])
        .with_indent(indent);

    // Get class members
    let mut children = Vec::new();
    for child_id in node.children() {
        if let Some(child_item) = get_class_member_navigation_item(ctx, child_id, indent + 1) {
            children.push(child_item);
        }
    }
    item = item.with_children(children);

    Some(item)
}

fn get_interface_navigation_item(
    ctx: &NavigationBarContext,
    node_id: NodeId,
    indent: u32,
) -> Option<NavigationBarItem> {
    let node = ctx.arena.get(node_id)?;
    let name = get_declaration_name(ctx, node_id).unwrap_or_else(|| "<anonymous>".to_string());
    let span = TextSpan::from_bounds(node.pos(), node.end());

    Some(
        NavigationBarItem::new(name, ScriptElementKind::InterfaceElement, vec![span])
            .with_indent(indent),
    )
}

fn get_type_alias_navigation_item(
    ctx: &NavigationBarContext,
    node_id: NodeId,
    indent: u32,
) -> Option<NavigationBarItem> {
    let node = ctx.arena.get(node_id)?;
    let name = get_declaration_name(ctx, node_id).unwrap_or_else(|| "<anonymous>".to_string());
    let span = TextSpan::from_bounds(node.pos(), node.end());

    Some(
        NavigationBarItem::new(name, ScriptElementKind::TypeElement, vec![span])
            .with_indent(indent),
    )
}

fn get_enum_navigation_item(
    ctx: &NavigationBarContext,
    node_id: NodeId,
    indent: u32,
) -> Option<NavigationBarItem> {
    let node = ctx.arena.get(node_id)?;
    let name = get_declaration_name(ctx, node_id).unwrap_or_else(|| "<anonymous>".to_string());
    let span = TextSpan::from_bounds(node.pos(), node.end());

    let mut item = NavigationBarItem::new(name, ScriptElementKind::EnumElement, vec![span])
        .with_indent(indent);

    // Get enum members
    let mut children = Vec::new();
    for child_id in node.children() {
        if let Some(child) = ctx.arena.get(child_id) {
            if child.kind() == SyntaxKind::EnumMember {
                if let Some(member_name) = get_declaration_name(ctx, child_id) {
                    let member_span = TextSpan::from_bounds(child.pos(), child.end());
                    children.push(
                        NavigationBarItem::new(
                            member_name,
                            ScriptElementKind::EnumMemberElement,
                            vec![member_span],
                        )
                        .with_indent(indent + 1),
                    );
                }
            }
        }
    }
    item = item.with_children(children);

    Some(item)
}

fn get_function_navigation_item(
    ctx: &NavigationBarContext,
    node_id: NodeId,
    indent: u32,
) -> Option<NavigationBarItem> {
    let node = ctx.arena.get(node_id)?;
    let name = get_declaration_name(ctx, node_id).unwrap_or_else(|| "<anonymous>".to_string());
    let span = TextSpan::from_bounds(node.pos(), node.end());

    Some(
        NavigationBarItem::new(name, ScriptElementKind::FunctionElement, vec![span])
            .with_indent(indent),
    )
}

fn get_variable_navigation_item(
    ctx: &NavigationBarContext,
    node_id: NodeId,
    indent: u32,
) -> Option<NavigationBarItem> {
    let node = ctx.arena.get(node_id)?;

    // Get the first variable name
    for child_id in node.children() {
        if let Some(child) = ctx.arena.get(child_id) {
            if child.kind() == SyntaxKind::VariableDeclarationList {
                for decl_id in child.children() {
                    if let Some(name) = get_declaration_name(ctx, decl_id) {
                        let span = TextSpan::from_bounds(node.pos(), node.end());
                        return Some(
                            NavigationBarItem::new(
                                name,
                                ScriptElementKind::VariableElement,
                                vec![span],
                            )
                            .with_indent(indent),
                        );
                    }
                }
            }
        }
    }

    None
}

fn get_module_navigation_item(
    ctx: &NavigationBarContext,
    node_id: NodeId,
    indent: u32,
) -> Option<NavigationBarItem> {
    let node = ctx.arena.get(node_id)?;
    let name = get_declaration_name(ctx, node_id).unwrap_or_else(|| "<module>".to_string());
    let span = TextSpan::from_bounds(node.pos(), node.end());

    let mut item = NavigationBarItem::new(name, ScriptElementKind::ModuleElement, vec![span])
        .with_indent(indent);

    // Get module members
    let mut children = Vec::new();
    for child_id in node.children() {
        if let Some(child_item) = get_navigation_item(ctx, child_id, indent + 1) {
            children.push(child_item);
        }
    }
    item = item.with_children(children);

    Some(item)
}

fn get_class_member_navigation_item(
    ctx: &NavigationBarContext,
    node_id: NodeId,
    indent: u32,
) -> Option<NavigationBarItem> {
    let node = ctx.arena.get(node_id)?;
    let kind = node.kind();

    let (element_kind, name) = match kind {
        SyntaxKind::MethodDeclaration => {
            let name = get_declaration_name(ctx, node_id)?;
            (ScriptElementKind::MemberFunctionElement, name)
        }
        SyntaxKind::PropertyDeclaration => {
            let name = get_declaration_name(ctx, node_id)?;
            (ScriptElementKind::MemberVariableElement, name)
        }
        SyntaxKind::GetAccessor => {
            let name = get_declaration_name(ctx, node_id)?;
            (ScriptElementKind::MemberGetAccessorElement, name)
        }
        SyntaxKind::SetAccessor => {
            let name = get_declaration_name(ctx, node_id)?;
            (ScriptElementKind::MemberSetAccessorElement, name)
        }
        SyntaxKind::Constructor => {
            (ScriptElementKind::ConstructorImplementationElement, "constructor".to_string())
        }
        _ => return None,
    };

    let span = TextSpan::from_bounds(node.pos(), node.end());
    Some(NavigationBarItem::new(name, element_kind, vec![span]).with_indent(indent))
}

// Tree versions
fn get_class_navigation_tree(ctx: &NavigationBarContext, node_id: NodeId) -> Option<NavigationTree> {
    let node = ctx.arena.get(node_id)?;
    let name = get_declaration_name(ctx, node_id).unwrap_or_else(|| "<anonymous>".to_string());
    let span = TextSpan::from_bounds(node.pos(), node.end());

    let mut children = Vec::new();
    for child_id in node.children() {
        if let Some(child) = get_class_member_navigation_tree(ctx, child_id) {
            children.push(child);
        }
    }

    Some(
        NavigationTree::new(name, ScriptElementKind::ClassElement, vec![span])
            .with_children(children),
    )
}

fn get_interface_navigation_tree(ctx: &NavigationBarContext, node_id: NodeId) -> Option<NavigationTree> {
    let node = ctx.arena.get(node_id)?;
    let name = get_declaration_name(ctx, node_id).unwrap_or_else(|| "<anonymous>".to_string());
    let span = TextSpan::from_bounds(node.pos(), node.end());

    Some(NavigationTree::new(name, ScriptElementKind::InterfaceElement, vec![span]))
}

fn get_type_alias_navigation_tree(ctx: &NavigationBarContext, node_id: NodeId) -> Option<NavigationTree> {
    let node = ctx.arena.get(node_id)?;
    let name = get_declaration_name(ctx, node_id).unwrap_or_else(|| "<anonymous>".to_string());
    let span = TextSpan::from_bounds(node.pos(), node.end());

    Some(NavigationTree::new(name, ScriptElementKind::TypeElement, vec![span]))
}

fn get_enum_navigation_tree(ctx: &NavigationBarContext, node_id: NodeId) -> Option<NavigationTree> {
    let node = ctx.arena.get(node_id)?;
    let name = get_declaration_name(ctx, node_id).unwrap_or_else(|| "<anonymous>".to_string());
    let span = TextSpan::from_bounds(node.pos(), node.end());

    Some(NavigationTree::new(name, ScriptElementKind::EnumElement, vec![span]))
}

fn get_function_navigation_tree(ctx: &NavigationBarContext, node_id: NodeId) -> Option<NavigationTree> {
    let node = ctx.arena.get(node_id)?;
    let name = get_declaration_name(ctx, node_id).unwrap_or_else(|| "<anonymous>".to_string());
    let span = TextSpan::from_bounds(node.pos(), node.end());

    Some(NavigationTree::new(name, ScriptElementKind::FunctionElement, vec![span]))
}

fn get_variable_navigation_tree(ctx: &NavigationBarContext, node_id: NodeId) -> Option<NavigationTree> {
    let node = ctx.arena.get(node_id)?;

    for child_id in node.children() {
        if let Some(child) = ctx.arena.get(child_id) {
            if child.kind() == SyntaxKind::VariableDeclarationList {
                for decl_id in child.children() {
                    if let Some(name) = get_declaration_name(ctx, decl_id) {
                        let span = TextSpan::from_bounds(node.pos(), node.end());
                        return Some(NavigationTree::new(
                            name,
                            ScriptElementKind::VariableElement,
                            vec![span],
                        ));
                    }
                }
            }
        }
    }

    None
}

fn get_module_navigation_tree(ctx: &NavigationBarContext, node_id: NodeId) -> Option<NavigationTree> {
    let node = ctx.arena.get(node_id)?;
    let name = get_declaration_name(ctx, node_id).unwrap_or_else(|| "<module>".to_string());
    let span = TextSpan::from_bounds(node.pos(), node.end());

    let mut children = Vec::new();
    for child_id in node.children() {
        if let Some(child) = get_navigation_tree_item(ctx, child_id) {
            children.push(child);
        }
    }

    Some(
        NavigationTree::new(name, ScriptElementKind::ModuleElement, vec![span])
            .with_children(children),
    )
}

fn get_class_member_navigation_tree(ctx: &NavigationBarContext, node_id: NodeId) -> Option<NavigationTree> {
    let node = ctx.arena.get(node_id)?;
    let kind = node.kind();

    let (element_kind, name) = match kind {
        SyntaxKind::MethodDeclaration => {
            let name = get_declaration_name(ctx, node_id)?;
            (ScriptElementKind::MemberFunctionElement, name)
        }
        SyntaxKind::PropertyDeclaration => {
            let name = get_declaration_name(ctx, node_id)?;
            (ScriptElementKind::MemberVariableElement, name)
        }
        SyntaxKind::GetAccessor => {
            let name = get_declaration_name(ctx, node_id)?;
            (ScriptElementKind::MemberGetAccessorElement, name)
        }
        SyntaxKind::SetAccessor => {
            let name = get_declaration_name(ctx, node_id)?;
            (ScriptElementKind::MemberSetAccessorElement, name)
        }
        SyntaxKind::Constructor => {
            (ScriptElementKind::ConstructorImplementationElement, "constructor".to_string())
        }
        _ => return None,
    };

    let span = TextSpan::from_bounds(node.pos(), node.end());
    Some(NavigationTree::new(name, element_kind, vec![span]))
}

// =============================================================================
// Helper Functions
// =============================================================================

fn get_declaration_name(ctx: &NavigationBarContext, node_id: NodeId) -> Option<String> {
    let node = ctx.arena.get(node_id)?;

    // Look for a name child
    for child_id in node.children() {
        if let Some(child) = ctx.arena.get(child_id) {
            if child.kind() == SyntaxKind::Identifier {
                let start = child.pos() as usize;
                let end = child.end() as usize;
                if end <= ctx.source_text.len() {
                    return Some(ctx.source_text[start..end].to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_bar_item() {
        let item = NavigationBarItem::new(
            "MyClass".to_string(),
            ScriptElementKind::ClassElement,
            vec![TextSpan::new(0, 100)],
        );

        assert_eq!(item.text, "MyClass");
        assert_eq!(item.kind, ScriptElementKind::ClassElement);
        assert_eq!(item.indent, 0);
    }

    #[test]
    fn test_navigation_tree() {
        let tree = NavigationTree::new(
            "test.ts".to_string(),
            ScriptElementKind::ModuleElement,
            vec![TextSpan::new(0, 100)],
        );

        assert_eq!(tree.text, "test.ts");
        assert!(tree.child_items.is_none());
    }
}
