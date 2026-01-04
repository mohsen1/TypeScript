//! Breakpoint validation implementation.
//!
//! This module provides breakpoint location validation for debugging,
//! determining valid positions where breakpoints can be set.

use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::text_span::TextSpan;

// =============================================================================
// Breakpoint Types
// =============================================================================

/// A validated breakpoint location.
#[derive(Debug, Clone)]
pub struct BreakpointSpan {
    /// The text span where the breakpoint is valid.
    pub text_span: TextSpan,
    /// The kind of breakpoint location.
    pub kind: BreakpointSpanKind,
}

impl BreakpointSpan {
    pub fn new(text_span: TextSpan, kind: BreakpointSpanKind) -> Self {
        Self { text_span, kind }
    }

    pub fn statement(text_span: TextSpan) -> Self {
        Self::new(text_span, BreakpointSpanKind::Statement)
    }

    pub fn expression(text_span: TextSpan) -> Self {
        Self::new(text_span, BreakpointSpanKind::Expression)
    }
}

/// The kind of breakpoint location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointSpanKind {
    /// A statement breakpoint.
    Statement,
    /// An expression breakpoint.
    Expression,
    /// A function entry breakpoint.
    FunctionEntry,
    /// A function exit breakpoint.
    FunctionExit,
}

// =============================================================================
// Breakpoint Context
// =============================================================================

/// Context for breakpoint operations.
pub struct BreakpointContext<'a> {
    pub arena: &'a NodeArena,
    pub source_text: &'a str,
}

// =============================================================================
// Get Breakpoint Span
// =============================================================================

/// Get the valid breakpoint span at a position.
pub fn get_breakpoint_span_at_position(
    ctx: &BreakpointContext,
    position: u32,
) -> Option<BreakpointSpan> {
    // Find the node at the position
    let node_id = find_node_at_position(ctx.arena, position)?;

    // Find the nearest statement or breakable expression
    find_breakpoint_span(ctx.arena, node_id)
}

/// Get all breakpoint spans in a range.
pub fn get_breakpoint_spans_in_range(
    ctx: &BreakpointContext,
    start: u32,
    end: u32,
) -> Vec<BreakpointSpan> {
    let mut spans = Vec::new();

    let root = match ctx.arena.root() {
        Some(r) => r,
        None => return spans,
    };

    collect_breakpoint_spans(ctx.arena, root, start, end, &mut spans);

    spans
}

// =============================================================================
// Breakpoint Span Discovery
// =============================================================================

fn find_breakpoint_span(arena: &NodeArena, node_id: NodeId) -> Option<BreakpointSpan> {
    let mut current = Some(node_id);

    while let Some(id) = current {
        let node = arena.get(id)?;

        // Check if this is a breakable location
        if let Some(span) = get_breakpoint_for_node(arena, id) {
            return Some(span);
        }

        current = node.parent();
    }

    None
}

fn get_breakpoint_for_node(arena: &NodeArena, node_id: NodeId) -> Option<BreakpointSpan> {
    let node = arena.get(node_id)?;
    let kind = node.kind();

    // Statements are always breakable
    if is_statement(kind) {
        let span = TextSpan::from_bounds(node.pos(), node.end());
        return Some(BreakpointSpan::statement(span));
    }

    // Function declarations - break at entry
    if matches!(
        kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
    ) {
        let span = TextSpan::from_bounds(node.pos(), node.end());
        return Some(BreakpointSpan::new(span, BreakpointSpanKind::FunctionEntry));
    }

    // Arrow functions - the body is breakable
    if kind == SyntaxKind::ArrowFunction {
        let span = TextSpan::from_bounds(node.pos(), node.end());
        return Some(BreakpointSpan::expression(span));
    }

    // Call expressions in certain contexts
    if kind == SyntaxKind::CallExpression {
        // Check if this is part of a statement
        if let Some(parent_id) = node.parent() {
            if let Some(parent) = arena.get(parent_id) {
                if parent.kind() == SyntaxKind::ExpressionStatement {
                    let span = TextSpan::from_bounds(node.pos(), node.end());
                    return Some(BreakpointSpan::expression(span));
                }
            }
        }
    }

    None
}

fn collect_breakpoint_spans(
    arena: &NodeArena,
    node_id: NodeId,
    start: u32,
    end: u32,
    spans: &mut Vec<BreakpointSpan>,
) {
    let node = match arena.get(node_id) {
        Some(n) => n,
        None => return,
    };

    // Check if node is in range
    if node.end() < start || node.pos() > end {
        return;
    }

    // Check if this node is breakable
    if let Some(span) = get_breakpoint_for_node(arena, node_id) {
        // Only add if the span is within the range
        if span.text_span.start >= start && span.text_span.start + span.text_span.length <= end {
            spans.push(span);
        }
    }

    // Recurse into children
    for child_id in node.children() {
        collect_breakpoint_spans(arena, child_id, start, end, spans);
    }
}

// =============================================================================
// Statement Detection
// =============================================================================

fn is_statement(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::VariableStatement
            | SyntaxKind::ExpressionStatement
            | SyntaxKind::IfStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::BreakStatement
            | SyntaxKind::ContinueStatement
            | SyntaxKind::ReturnStatement
            | SyntaxKind::WithStatement
            | SyntaxKind::SwitchStatement
            | SyntaxKind::LabeledStatement
            | SyntaxKind::ThrowStatement
            | SyntaxKind::TryStatement
            | SyntaxKind::DebuggerStatement
    )
}

// =============================================================================
// Line-Based Breakpoints
// =============================================================================

/// Get breakpoint span for a line number.
pub fn get_breakpoint_span_for_line(
    ctx: &BreakpointContext,
    line: u32,
) -> Option<BreakpointSpan> {
    // Convert line number to position
    let position = line_to_position(ctx.source_text, line)?;

    // Find the first breakable span at or after this position
    get_breakpoint_span_at_position(ctx, position)
}

fn line_to_position(source: &str, line: u32) -> Option<u32> {
    let mut current_line = 0u32;
    let mut position = 0u32;

    for (i, c) in source.char_indices() {
        if current_line == line {
            // Skip leading whitespace
            let remaining = &source[i..];
            let trimmed_pos = remaining.len() - remaining.trim_start().len();
            return Some((i + trimmed_pos) as u32);
        }

        if c == '\n' {
            current_line += 1;
        }
        position = (i + c.len_utf8()) as u32;
    }

    if current_line == line {
        Some(position)
    } else {
        None
    }
}

// =============================================================================
// Inline Breakpoints
// =============================================================================

/// Get all possible inline breakpoint positions for a line.
pub fn get_inline_breakpoint_positions(
    ctx: &BreakpointContext,
    line: u32,
) -> Vec<BreakpointSpan> {
    let mut positions = Vec::new();

    // Get the line's start and end positions
    let (line_start, line_end) = match get_line_bounds(ctx.source_text, line) {
        Some(bounds) => bounds,
        None => return positions,
    };

    // Find all breakable expressions on this line
    get_breakpoint_spans_in_range(ctx, line_start, line_end)
}

fn get_line_bounds(source: &str, line: u32) -> Option<(u32, u32)> {
    let mut current_line = 0u32;
    let mut line_start = 0usize;

    for (i, c) in source.char_indices() {
        if current_line == line {
            // Find end of line
            let line_end = source[i..]
                .find('\n')
                .map(|offset| i + offset)
                .unwrap_or(source.len());
            return Some((line_start as u32, line_end as u32));
        }

        if c == '\n' {
            current_line += 1;
            line_start = i + 1;
        }
    }

    if current_line == line {
        Some((line_start as u32, source.len() as u32))
    } else {
        None
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_span() {
        let span = BreakpointSpan::statement(TextSpan::new(10, 20));
        assert_eq!(span.kind, BreakpointSpanKind::Statement);
        assert_eq!(span.text_span.start, 10);
    }

    #[test]
    fn test_is_statement() {
        assert!(is_statement(SyntaxKind::VariableStatement));
        assert!(is_statement(SyntaxKind::IfStatement));
        assert!(is_statement(SyntaxKind::ReturnStatement));
        assert!(!is_statement(SyntaxKind::Identifier));
        assert!(!is_statement(SyntaxKind::CallExpression));
    }

    #[test]
    fn test_line_to_position() {
        let source = "line0\nline1\nline2";

        assert_eq!(line_to_position(source, 0), Some(0));
        assert_eq!(line_to_position(source, 1), Some(6));
        assert_eq!(line_to_position(source, 2), Some(12));
        assert_eq!(line_to_position(source, 3), None);
    }

    #[test]
    fn test_get_line_bounds() {
        let source = "line0\nline1\nline2";

        assert_eq!(get_line_bounds(source, 0), Some((0, 5)));
        assert_eq!(get_line_bounds(source, 1), Some((6, 11)));
        assert_eq!(get_line_bounds(source, 2), Some((12, 17)));
    }
}
