//! Document Highlights implementation.
//!
//! This module provides functionality for highlighting all occurrences
//! of a symbol or keyword within a document.

use crate::binder::{Symbol, SymbolId, SymbolFlags};
use crate::checker::CheckerState;
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::text_span::TextSpan;
use super::utilities::HighlightSpanKind;

// =============================================================================
// Document Highlight
// =============================================================================

/// A highlighted span in a document.
#[derive(Debug, Clone)]
pub struct HighlightSpan {
    /// The file containing the highlight.
    pub file_name: String,
    /// The span of the highlight.
    pub text_span: TextSpan,
    /// The kind of highlight.
    pub kind: HighlightSpanKind,
    /// Whether this is in a string.
    pub is_in_string: Option<bool>,
}

impl HighlightSpan {
    pub fn new(file_name: String, text_span: TextSpan, kind: HighlightSpanKind) -> Self {
        Self {
            file_name,
            text_span,
            kind,
            is_in_string: None,
        }
    }

    pub fn definition(file_name: String, text_span: TextSpan) -> Self {
        Self::new(file_name, text_span, HighlightSpanKind::Definition)
    }

    pub fn reference(file_name: String, text_span: TextSpan) -> Self {
        Self::new(file_name, text_span, HighlightSpanKind::Reference)
    }

    pub fn write_reference(file_name: String, text_span: TextSpan) -> Self {
        Self::new(file_name, text_span, HighlightSpanKind::WrittenReference)
    }
}

/// Document highlights for a specific document.
#[derive(Debug, Clone)]
pub struct DocumentHighlights {
    /// The file name.
    pub file_name: String,
    /// The highlight spans in this file.
    pub highlight_spans: Vec<HighlightSpan>,
}

impl DocumentHighlights {
    pub fn new(file_name: String, spans: Vec<HighlightSpan>) -> Self {
        Self {
            file_name,
            highlight_spans: spans,
        }
    }
}

// =============================================================================
// Document Highlights Context
// =============================================================================

/// Context for document highlights operations.
pub struct DocumentHighlightsContext<'a> {
    pub arena: &'a NodeArena,
    pub checker: &'a CheckerState,
    pub source_text: &'a str,
    pub file_name: &'a str,
    /// Files to search for highlights.
    pub files_to_search: &'a [String],
}

// =============================================================================
// Get Document Highlights
// =============================================================================

/// Get document highlights at a position.
pub fn get_document_highlights(
    ctx: &DocumentHighlightsContext,
    position: u32,
) -> Option<Vec<DocumentHighlights>> {
    // Find the node at the position
    let node_id = find_node_at_position(ctx.arena, position)?;
    let node = ctx.arena.get(node_id)?;

    // Check if it's a keyword that needs special handling
    if let Some(highlights) = get_keyword_highlights(ctx, node_id) {
        return Some(highlights);
    }

    // Otherwise, get symbol-based highlights
    get_symbol_highlights(ctx, node_id)
}

// =============================================================================
// Keyword Highlights
// =============================================================================

/// Get highlights for keywords (if/else, try/catch, etc.).
fn get_keyword_highlights(
    ctx: &DocumentHighlightsContext,
    node_id: NodeId,
) -> Option<Vec<DocumentHighlights>> {
    let node = ctx.arena.get(node_id)?;
    let kind = node.kind();

    match kind {
        // if/else keywords
        SyntaxKind::IfKeyword | SyntaxKind::ElseKeyword => {
            get_if_else_highlights(ctx, node_id)
        }
        // try/catch/finally keywords
        SyntaxKind::TryKeyword | SyntaxKind::CatchKeyword | SyntaxKind::FinallyKeyword => {
            get_try_catch_highlights(ctx, node_id)
        }
        // switch/case/default keywords
        SyntaxKind::SwitchKeyword | SyntaxKind::CaseKeyword | SyntaxKind::DefaultKeyword => {
            get_switch_case_highlights(ctx, node_id)
        }
        // loop keywords
        SyntaxKind::ForKeyword
        | SyntaxKind::WhileKeyword
        | SyntaxKind::DoKeyword
        | SyntaxKind::BreakKeyword
        | SyntaxKind::ContinueKeyword => {
            get_loop_highlights(ctx, node_id)
        }
        // function/return keywords
        SyntaxKind::FunctionKeyword | SyntaxKind::ReturnKeyword => {
            get_function_return_highlights(ctx, node_id)
        }
        // async/await keywords
        SyntaxKind::AsyncKeyword | SyntaxKind::AwaitKeyword => {
            get_async_await_highlights(ctx, node_id)
        }
        // yield keyword
        SyntaxKind::YieldKeyword => {
            get_yield_highlights(ctx, node_id)
        }
        _ => None,
    }
}

fn get_if_else_highlights(
    ctx: &DocumentHighlightsContext,
    node_id: NodeId,
) -> Option<Vec<DocumentHighlights>> {
    // Find the containing if statement and highlight all if/else keywords
    let if_stmt = find_containing_if_statement(ctx.arena, node_id)?;
    let mut spans = Vec::new();

    collect_if_else_keywords(ctx.arena, if_stmt, &mut spans, ctx.file_name);

    if spans.is_empty() {
        None
    } else {
        Some(vec![DocumentHighlights::new(ctx.file_name.to_string(), spans)])
    }
}

fn get_try_catch_highlights(
    ctx: &DocumentHighlightsContext,
    node_id: NodeId,
) -> Option<Vec<DocumentHighlights>> {
    // Find the containing try statement
    let try_stmt = find_containing_try_statement(ctx.arena, node_id)?;
    let mut spans = Vec::new();

    collect_try_catch_keywords(ctx.arena, try_stmt, &mut spans, ctx.file_name);

    if spans.is_empty() {
        None
    } else {
        Some(vec![DocumentHighlights::new(ctx.file_name.to_string(), spans)])
    }
}

fn get_switch_case_highlights(
    ctx: &DocumentHighlightsContext,
    node_id: NodeId,
) -> Option<Vec<DocumentHighlights>> {
    // Find the containing switch statement
    let switch_stmt = find_containing_switch_statement(ctx.arena, node_id)?;
    let mut spans = Vec::new();

    collect_switch_case_keywords(ctx.arena, switch_stmt, &mut spans, ctx.file_name);

    if spans.is_empty() {
        None
    } else {
        Some(vec![DocumentHighlights::new(ctx.file_name.to_string(), spans)])
    }
}

fn get_loop_highlights(
    ctx: &DocumentHighlightsContext,
    node_id: NodeId,
) -> Option<Vec<DocumentHighlights>> {
    // Find the containing loop and highlight break/continue
    let loop_stmt = find_containing_loop(ctx.arena, node_id)?;
    let mut spans = Vec::new();

    collect_loop_keywords(ctx.arena, loop_stmt, &mut spans, ctx.file_name);

    if spans.is_empty() {
        None
    } else {
        Some(vec![DocumentHighlights::new(ctx.file_name.to_string(), spans)])
    }
}

fn get_function_return_highlights(
    ctx: &DocumentHighlightsContext,
    node_id: NodeId,
) -> Option<Vec<DocumentHighlights>> {
    // Find the containing function and highlight all return statements
    let func = find_containing_function(ctx.arena, node_id)?;
    let mut spans = Vec::new();

    collect_return_keywords(ctx.arena, func, &mut spans, ctx.file_name);

    if spans.is_empty() {
        None
    } else {
        Some(vec![DocumentHighlights::new(ctx.file_name.to_string(), spans)])
    }
}

fn get_async_await_highlights(
    ctx: &DocumentHighlightsContext,
    node_id: NodeId,
) -> Option<Vec<DocumentHighlights>> {
    // Find the containing async function and highlight all await expressions
    let async_func = find_containing_async_function(ctx.arena, node_id)?;
    let mut spans = Vec::new();

    collect_async_await_keywords(ctx.arena, async_func, &mut spans, ctx.file_name);

    if spans.is_empty() {
        None
    } else {
        Some(vec![DocumentHighlights::new(ctx.file_name.to_string(), spans)])
    }
}

fn get_yield_highlights(
    ctx: &DocumentHighlightsContext,
    node_id: NodeId,
) -> Option<Vec<DocumentHighlights>> {
    // Find the containing generator and highlight all yield expressions
    let generator = find_containing_generator(ctx.arena, node_id)?;
    let mut spans = Vec::new();

    collect_yield_keywords(ctx.arena, generator, &mut spans, ctx.file_name);

    if spans.is_empty() {
        None
    } else {
        Some(vec![DocumentHighlights::new(ctx.file_name.to_string(), spans)])
    }
}

// =============================================================================
// Symbol Highlights
// =============================================================================

/// Get highlights for a symbol.
fn get_symbol_highlights(
    ctx: &DocumentHighlightsContext,
    node_id: NodeId,
) -> Option<Vec<DocumentHighlights>> {
    // Get the symbol at this location
    let symbol = ctx.checker.get_symbol_at_location(node_id)?;

    let mut all_highlights = Vec::new();

    // For each file to search
    for file_name in ctx.files_to_search {
        let spans = find_symbol_references_in_file(ctx, &symbol, file_name);
        if !spans.is_empty() {
            all_highlights.push(DocumentHighlights::new(file_name.clone(), spans));
        }
    }

    if all_highlights.is_empty() {
        None
    } else {
        Some(all_highlights)
    }
}

fn find_symbol_references_in_file(
    ctx: &DocumentHighlightsContext,
    symbol: &Symbol,
    file_name: &str,
) -> Vec<HighlightSpan> {
    let mut spans = Vec::new();

    // Add the definition
    for decl_id in &symbol.declarations {
        if let Some(node) = ctx.arena.get(*decl_id) {
            let span = TextSpan::from_bounds(node.pos(), node.end());
            spans.push(HighlightSpan::definition(file_name.to_string(), span));
        }
    }

    // In a full implementation, we would walk the AST to find all references
    // to this symbol in the file

    spans
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

fn find_containing_if_statement(arena: &NodeArena, node_id: NodeId) -> Option<NodeId> {
    find_ancestor_of_kind(arena, node_id, SyntaxKind::IfStatement)
}

fn find_containing_try_statement(arena: &NodeArena, node_id: NodeId) -> Option<NodeId> {
    find_ancestor_of_kind(arena, node_id, SyntaxKind::TryStatement)
}

fn find_containing_switch_statement(arena: &NodeArena, node_id: NodeId) -> Option<NodeId> {
    find_ancestor_of_kind(arena, node_id, SyntaxKind::SwitchStatement)
}

fn find_containing_loop(arena: &NodeArena, node_id: NodeId) -> Option<NodeId> {
    // Find any loop: for, while, do
    let mut current = Some(node_id);
    while let Some(id) = current {
        if let Some(node) = arena.get(id) {
            match node.kind() {
                SyntaxKind::ForStatement
                | SyntaxKind::ForInStatement
                | SyntaxKind::ForOfStatement
                | SyntaxKind::WhileStatement
                | SyntaxKind::DoStatement => {
                    return Some(id);
                }
                _ => {}
            }
            current = node.parent();
        } else {
            break;
        }
    }
    None
}

fn find_containing_function(arena: &NodeArena, node_id: NodeId) -> Option<NodeId> {
    let mut current = Some(node_id);
    while let Some(id) = current {
        if let Some(node) = arena.get(id) {
            match node.kind() {
                SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::Constructor => {
                    return Some(id);
                }
                _ => {}
            }
            current = node.parent();
        } else {
            break;
        }
    }
    None
}

fn find_containing_async_function(arena: &NodeArena, node_id: NodeId) -> Option<NodeId> {
    // Same as find_containing_function but check for async modifier
    find_containing_function(arena, node_id)
    // In a full implementation, we'd check if the function has the async modifier
}

fn find_containing_generator(arena: &NodeArena, node_id: NodeId) -> Option<NodeId> {
    // Same as find_containing_function but check for generator (asterisk)
    find_containing_function(arena, node_id)
    // In a full implementation, we'd check if the function is a generator
}

fn find_ancestor_of_kind(arena: &NodeArena, node_id: NodeId, kind: SyntaxKind) -> Option<NodeId> {
    let mut current = Some(node_id);
    while let Some(id) = current {
        if let Some(node) = arena.get(id) {
            if node.kind() == kind {
                return Some(id);
            }
            current = node.parent();
        } else {
            break;
        }
    }
    None
}

// Stub functions for collecting keywords
fn collect_if_else_keywords(
    arena: &NodeArena,
    node_id: NodeId,
    spans: &mut Vec<HighlightSpan>,
    file_name: &str,
) {
    // In a full implementation, walk the if statement and collect if/else keywords
}

fn collect_try_catch_keywords(
    arena: &NodeArena,
    node_id: NodeId,
    spans: &mut Vec<HighlightSpan>,
    file_name: &str,
) {
    // In a full implementation, walk the try statement and collect try/catch/finally keywords
}

fn collect_switch_case_keywords(
    arena: &NodeArena,
    node_id: NodeId,
    spans: &mut Vec<HighlightSpan>,
    file_name: &str,
) {
    // In a full implementation, walk the switch statement and collect keywords
}

fn collect_loop_keywords(
    arena: &NodeArena,
    node_id: NodeId,
    spans: &mut Vec<HighlightSpan>,
    file_name: &str,
) {
    // In a full implementation, walk the loop and collect break/continue
}

fn collect_return_keywords(
    arena: &NodeArena,
    node_id: NodeId,
    spans: &mut Vec<HighlightSpan>,
    file_name: &str,
) {
    // In a full implementation, walk the function and collect return statements
}

fn collect_async_await_keywords(
    arena: &NodeArena,
    node_id: NodeId,
    spans: &mut Vec<HighlightSpan>,
    file_name: &str,
) {
    // In a full implementation, walk the async function and collect await
}

fn collect_yield_keywords(
    arena: &NodeArena,
    node_id: NodeId,
    spans: &mut Vec<HighlightSpan>,
    file_name: &str,
) {
    // In a full implementation, walk the generator and collect yield
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_span() {
        let span = HighlightSpan::definition(
            "test.ts".to_string(),
            TextSpan::new(10, 5),
        );

        assert_eq!(span.file_name, "test.ts");
        assert_eq!(span.kind, HighlightSpanKind::Definition);
    }

    #[test]
    fn test_document_highlights() {
        let spans = vec![
            HighlightSpan::definition("test.ts".to_string(), TextSpan::new(10, 5)),
            HighlightSpan::reference("test.ts".to_string(), TextSpan::new(50, 5)),
        ];

        let highlights = DocumentHighlights::new("test.ts".to_string(), spans);

        assert_eq!(highlights.file_name, "test.ts");
        assert_eq!(highlights.highlight_spans.len(), 2);
    }
}
