//! Call Hierarchy implementation.
//!
//! This module provides call hierarchy information for:
//! - Finding incoming calls (who calls this function)
//! - Finding outgoing calls (what functions this function calls)

use crate::binder::{Symbol, SymbolId, SymbolFlags};
use crate::checker::CheckerState;
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::symbol_display::SymbolDisplayPart;
use super::text_span::TextSpan;
use super::utilities::ScriptElementKind;

// =============================================================================
// Call Hierarchy Item
// =============================================================================

/// An item in the call hierarchy.
#[derive(Debug, Clone)]
pub struct CallHierarchyItem {
    /// The name of the callable.
    pub name: String,
    /// The kind of the callable.
    pub kind: ScriptElementKind,
    /// Kind modifiers (e.g., "async", "static").
    pub kind_modifiers: String,
    /// The file containing the callable.
    pub file_name: String,
    /// The span of the callable's body.
    pub span: TextSpan,
    /// The span of the callable's name.
    pub selection_span: TextSpan,
    /// The container name (e.g., class name).
    pub container_name: Option<String>,
}

impl CallHierarchyItem {
    pub fn new(
        name: String,
        kind: ScriptElementKind,
        file_name: String,
        span: TextSpan,
        selection_span: TextSpan,
    ) -> Self {
        Self {
            name,
            kind,
            kind_modifiers: String::new(),
            file_name,
            span,
            selection_span,
            container_name: None,
        }
    }

    pub fn with_modifiers(mut self, modifiers: String) -> Self {
        self.kind_modifiers = modifiers;
        self
    }

    pub fn with_container(mut self, container: String) -> Self {
        self.container_name = Some(container);
        self
    }
}

// =============================================================================
// Call Hierarchy Calls
// =============================================================================

/// An incoming call in the call hierarchy.
#[derive(Debug, Clone)]
pub struct CallHierarchyIncomingCall {
    /// The caller.
    pub from: CallHierarchyItem,
    /// The call sites within the caller.
    pub from_spans: Vec<TextSpan>,
}

impl CallHierarchyIncomingCall {
    pub fn new(from: CallHierarchyItem, spans: Vec<TextSpan>) -> Self {
        Self {
            from,
            from_spans: spans,
        }
    }
}

/// An outgoing call in the call hierarchy.
#[derive(Debug, Clone)]
pub struct CallHierarchyOutgoingCall {
    /// The callee.
    pub to: CallHierarchyItem,
    /// The call sites to the callee.
    pub from_spans: Vec<TextSpan>,
}

impl CallHierarchyOutgoingCall {
    pub fn new(to: CallHierarchyItem, spans: Vec<TextSpan>) -> Self {
        Self {
            to,
            from_spans: spans,
        }
    }
}

// =============================================================================
// Call Hierarchy Context
// =============================================================================

/// Context for call hierarchy operations.
pub struct CallHierarchyContext<'a> {
    pub arena: &'a NodeArena,
    pub checker: &'a CheckerState,
    pub source_text: &'a str,
    pub file_name: &'a str,
    /// All files to search for calls.
    pub files_to_search: &'a [String],
}

// =============================================================================
// Prepare Call Hierarchy
// =============================================================================

/// Prepare call hierarchy at a position.
/// Returns the items that can be expanded at this position.
pub fn prepare_call_hierarchy(
    ctx: &CallHierarchyContext,
    position: u32,
) -> Option<Vec<CallHierarchyItem>> {
    // Find the node at the position
    let node_id = find_node_at_position(ctx.arena, position)?;

    // Find the containing callable
    let callable_id = find_enclosing_callable(ctx.arena, node_id)?;

    // Create the call hierarchy item
    let item = create_call_hierarchy_item(ctx, callable_id)?;

    Some(vec![item])
}

/// Create a call hierarchy item for a callable node.
fn create_call_hierarchy_item(
    ctx: &CallHierarchyContext,
    callable_id: NodeId,
) -> Option<CallHierarchyItem> {
    let node = ctx.arena.get(callable_id)?;

    // Get the name
    let name = get_callable_name(ctx.arena, callable_id)?;

    // Determine the kind
    let kind = match node.kind() {
        SyntaxKind::FunctionDeclaration => ScriptElementKind::FunctionElement,
        SyntaxKind::FunctionExpression => ScriptElementKind::FunctionElement,
        SyntaxKind::ArrowFunction => ScriptElementKind::FunctionElement,
        SyntaxKind::MethodDeclaration => ScriptElementKind::MemberFunctionElement,
        SyntaxKind::Constructor => ScriptElementKind::ConstructorImplementationElement,
        SyntaxKind::GetAccessor => ScriptElementKind::MemberGetAccessorElement,
        SyntaxKind::SetAccessor => ScriptElementKind::MemberSetAccessorElement,
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => ScriptElementKind::ClassElement,
        _ => return None,
    };

    // Get spans
    let span = TextSpan::from_bounds(node.pos(), node.end());
    let selection_span = get_selection_span(ctx.arena, callable_id);

    let mut item = CallHierarchyItem::new(
        name,
        kind,
        ctx.file_name.to_string(),
        span,
        selection_span,
    );

    // Get modifiers
    let modifiers = get_callable_modifiers(ctx.arena, callable_id);
    if !modifiers.is_empty() {
        item = item.with_modifiers(modifiers);
    }

    // Get container
    if let Some(container_name) = get_container_name(ctx.arena, callable_id) {
        item = item.with_container(container_name);
    }

    Some(item)
}

// =============================================================================
// Get Incoming Calls
// =============================================================================

/// Get incoming calls to a callable.
pub fn provide_call_hierarchy_incoming_calls(
    ctx: &CallHierarchyContext,
    item: &CallHierarchyItem,
) -> Vec<CallHierarchyIncomingCall> {
    let mut incoming_calls = Vec::new();

    // Find the callable node
    let callable_id = match find_callable_at_span(ctx.arena, &item.selection_span) {
        Some(id) => id,
        None => return incoming_calls,
    };

    // Get the symbol for this callable
    let symbol = match ctx.checker.get_symbol_at_location(callable_id) {
        Some(s) => s,
        None => return incoming_calls,
    };

    // Search for references to this symbol
    for file_name in ctx.files_to_search {
        let calls = find_incoming_calls_in_file(ctx, &symbol, file_name);

        for (caller_id, call_sites) in calls {
            if let Some(caller_item) = create_call_hierarchy_item(ctx, caller_id) {
                incoming_calls.push(CallHierarchyIncomingCall::new(
                    caller_item,
                    call_sites,
                ));
            }
        }
    }

    incoming_calls
}

fn find_incoming_calls_in_file(
    ctx: &CallHierarchyContext,
    target_symbol: &Symbol,
    file_name: &str,
) -> Vec<(NodeId, Vec<TextSpan>)> {
    let mut results = Vec::new();

    // In a full implementation, this would:
    // 1. Load/get the source file for the given file name
    // 2. Walk the AST looking for call expressions
    // 3. Resolve each call expression to see if it calls the target
    // 4. Group calls by their containing callable
    // 5. Return the results

    results
}

// =============================================================================
// Get Outgoing Calls
// =============================================================================

/// Get outgoing calls from a callable.
pub fn provide_call_hierarchy_outgoing_calls(
    ctx: &CallHierarchyContext,
    item: &CallHierarchyItem,
) -> Vec<CallHierarchyOutgoingCall> {
    let mut outgoing_calls = Vec::new();

    // Find the callable node
    let callable_id = match find_callable_at_span(ctx.arena, &item.span) {
        Some(id) => id,
        None => return outgoing_calls,
    };

    // Find all call expressions within this callable
    let calls = find_outgoing_calls_in_callable(ctx, callable_id);

    // Group calls by target
    let grouped = group_calls_by_target(ctx, calls);

    for (target_id, call_sites) in grouped {
        if let Some(target_item) = create_call_hierarchy_item(ctx, target_id) {
            outgoing_calls.push(CallHierarchyOutgoingCall::new(
                target_item,
                call_sites,
            ));
        }
    }

    outgoing_calls
}

fn find_outgoing_calls_in_callable(
    ctx: &CallHierarchyContext,
    callable_id: NodeId,
) -> Vec<(NodeId, TextSpan)> {
    let mut calls = Vec::new();

    // Walk the callable's body looking for call expressions
    collect_call_expressions(ctx.arena, callable_id, &mut calls);

    calls
}

fn collect_call_expressions(
    arena: &NodeArena,
    node_id: NodeId,
    calls: &mut Vec<(NodeId, TextSpan)>,
) {
    let node = match arena.get(node_id) {
        Some(n) => n,
        None => return,
    };

    match node.kind() {
        SyntaxKind::CallExpression | SyntaxKind::NewExpression => {
            let span = TextSpan::from_bounds(node.pos(), node.end());
            calls.push((node_id, span));
        }
        // Don't recurse into nested functions
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::ClassDeclaration
        | SyntaxKind::ClassExpression => {
            return;
        }
        _ => {}
    }

    // Recurse into children
    for child_id in node.children() {
        collect_call_expressions(arena, child_id, calls);
    }
}

fn group_calls_by_target(
    ctx: &CallHierarchyContext,
    calls: Vec<(NodeId, TextSpan)>,
) -> Vec<(NodeId, Vec<TextSpan>)> {
    use std::collections::HashMap;

    let mut grouped: HashMap<SymbolId, (NodeId, Vec<TextSpan>)> = HashMap::new();

    for (call_id, span) in calls {
        // Resolve the call to find the target
        if let Some(target_symbol) = resolve_call_target(ctx, call_id) {
            if let Some(target_decl) = target_symbol.declarations.first() {
                let entry = grouped
                    .entry(target_symbol.id)
                    .or_insert((*target_decl, Vec::new()));
                entry.1.push(span);
            }
        }
    }

    grouped.into_values().collect()
}

fn resolve_call_target(
    ctx: &CallHierarchyContext,
    call_id: NodeId,
) -> Option<Symbol> {
    // Get the expression being called
    let call = ctx.arena.get(call_id)?;

    // In a full implementation, this would navigate to the callee expression
    // and resolve the symbol
    ctx.checker.get_symbol_at_location(call_id)
}

// =============================================================================
// Helper Functions
// =============================================================================

fn find_node_at_position(arena: &NodeArena, position: u32) -> Option<NodeId> {
    let root = arena.root()?;
    find_deepest_node_at_position(arena, root, position)
}

fn find_deepest_node_at_position(
    arena: &NodeArena,
    node_id: NodeId,
    position: u32,
) -> Option<NodeId> {
    let node = arena.get(node_id)?;

    if position < node.pos() || position >= node.end() {
        return None;
    }

    for child_id in node.children() {
        if let Some(deeper) = find_deepest_node_at_position(arena, child_id, position) {
            return Some(deeper);
        }
    }

    Some(node_id)
}

fn find_enclosing_callable(arena: &NodeArena, node_id: NodeId) -> Option<NodeId> {
    let mut current = Some(node_id);

    while let Some(id) = current {
        let node = arena.get(id)?;

        if is_callable(node.kind()) {
            return Some(id);
        }

        current = node.parent();
    }

    None
}

fn is_callable(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
    )
}

fn find_callable_at_span(arena: &NodeArena, span: &TextSpan) -> Option<NodeId> {
    // Find a callable that matches the given span
    let root = arena.root()?;
    find_callable_at_span_recursive(arena, root, span)
}

fn find_callable_at_span_recursive(
    arena: &NodeArena,
    node_id: NodeId,
    span: &TextSpan,
) -> Option<NodeId> {
    let node = arena.get(node_id)?;

    // Check if this node matches the span
    let node_span = TextSpan::from_bounds(node.pos(), node.end());
    if node_span.start == span.start && node_span.length == span.length && is_callable(node.kind()) {
        return Some(node_id);
    }

    // Check children
    for child_id in node.children() {
        if let Some(found) = find_callable_at_span_recursive(arena, child_id, span) {
            return Some(found);
        }
    }

    None
}

fn get_callable_name(arena: &NodeArena, callable_id: NodeId) -> Option<String> {
    let node = arena.get(callable_id)?;

    match node.kind() {
        SyntaxKind::Constructor => Some("constructor".to_string()),
        SyntaxKind::ArrowFunction => Some("<anonymous>".to_string()),
        _ => {
            // In a full implementation, get the name from the node's children
            Some("<function>".to_string())
        }
    }
}

fn get_selection_span(arena: &NodeArena, callable_id: NodeId) -> TextSpan {
    let node = match arena.get(callable_id) {
        Some(n) => n,
        None => return TextSpan::new(0, 0),
    };

    // In a full implementation, this would find the name token
    TextSpan::from_bounds(node.pos(), node.end())
}

fn get_callable_modifiers(arena: &NodeArena, callable_id: NodeId) -> String {
    // In a full implementation, check for async, static, etc.
    String::new()
}

fn get_container_name(arena: &NodeArena, callable_id: NodeId) -> Option<String> {
    let node = arena.get(callable_id)?;
    let parent_id = node.parent()?;
    let parent = arena.get(parent_id)?;

    match parent.kind() {
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
            // Get the class name
            Some("<class>".to_string())
        }
        SyntaxKind::ObjectLiteralExpression => {
            Some("<object>".to_string())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_hierarchy_item() {
        let item = CallHierarchyItem::new(
            "foo".to_string(),
            ScriptElementKind::FunctionElement,
            "test.ts".to_string(),
            TextSpan::new(0, 100),
            TextSpan::new(9, 3),
        );

        assert_eq!(item.name, "foo");
        assert_eq!(item.kind, ScriptElementKind::FunctionElement);
    }

    #[test]
    fn test_call_hierarchy_item_with_container() {
        let item = CallHierarchyItem::new(
            "bar".to_string(),
            ScriptElementKind::MemberFunctionElement,
            "test.ts".to_string(),
            TextSpan::new(50, 50),
            TextSpan::new(55, 3),
        )
        .with_container("MyClass".to_string())
        .with_modifiers("async".to_string());

        assert_eq!(item.container_name, Some("MyClass".to_string()));
        assert_eq!(item.kind_modifiers, "async");
    }

    #[test]
    fn test_incoming_call() {
        let caller = CallHierarchyItem::new(
            "caller".to_string(),
            ScriptElementKind::FunctionElement,
            "test.ts".to_string(),
            TextSpan::new(0, 50),
            TextSpan::new(9, 6),
        );

        let call = CallHierarchyIncomingCall::new(
            caller,
            vec![TextSpan::new(25, 5)],
        );

        assert_eq!(call.from.name, "caller");
        assert_eq!(call.from_spans.len(), 1);
    }

    #[test]
    fn test_outgoing_call() {
        let callee = CallHierarchyItem::new(
            "callee".to_string(),
            ScriptElementKind::FunctionElement,
            "test.ts".to_string(),
            TextSpan::new(100, 50),
            TextSpan::new(109, 6),
        );

        let call = CallHierarchyOutgoingCall::new(
            callee,
            vec![TextSpan::new(25, 8), TextSpan::new(35, 8)],
        );

        assert_eq!(call.to.name, "callee");
        assert_eq!(call.from_spans.len(), 2);
    }

    #[test]
    fn test_is_callable() {
        assert!(is_callable(SyntaxKind::FunctionDeclaration));
        assert!(is_callable(SyntaxKind::MethodDeclaration));
        assert!(is_callable(SyntaxKind::ArrowFunction));
        assert!(!is_callable(SyntaxKind::Identifier));
        assert!(!is_callable(SyntaxKind::CallExpression));
    }
}
