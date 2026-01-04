//! Outlining (Code Folding) implementation.
//!
//! This module provides code folding regions for:
//! - Blocks (functions, classes, if statements, etc.)
//! - Import sections
//! - Comment blocks
//! - Region directives

use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::text_span::TextSpan;

// =============================================================================
// Outlining Types
// =============================================================================

/// A foldable region in the source code.
#[derive(Debug, Clone)]
pub struct OutliningSpan {
    /// The full span of the region (used for highlighting).
    pub text_span: TextSpan,
    /// The span that will be collapsed/hidden.
    pub hint_span: TextSpan,
    /// The text to show when collapsed.
    pub banner_text: String,
    /// Whether this region should be collapsed by default.
    pub auto_collapse: bool,
    /// The kind of outlining span.
    pub kind: OutliningSpanKind,
}

impl OutliningSpan {
    pub fn new(
        text_span: TextSpan,
        hint_span: TextSpan,
        banner_text: String,
        kind: OutliningSpanKind,
    ) -> Self {
        Self {
            text_span,
            hint_span,
            banner_text,
            auto_collapse: false,
            kind,
        }
    }

    pub fn auto_collapse(mut self) -> Self {
        self.auto_collapse = true;
        self
    }
}

/// The kind of outlining span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutliningSpanKind {
    /// A comment block.
    Comment,
    /// A region directive.
    Region,
    /// A code block.
    Code,
    /// An imports section.
    Imports,
}

impl OutliningSpanKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Comment => "comment",
            Self::Region => "region",
            Self::Code => "code",
            Self::Imports => "imports",
        }
    }
}

// =============================================================================
// Outlining Context
// =============================================================================

/// Context for outlining operations.
pub struct OutliningContext<'a> {
    pub arena: &'a NodeArena,
    pub source_text: &'a str,
}

// =============================================================================
// Get Outlining Spans
// =============================================================================

/// Get all outlining spans for a document.
pub fn get_outlining_spans(ctx: &OutliningContext) -> Vec<OutliningSpan> {
    let mut spans = Vec::new();

    let root = match ctx.arena.root() {
        Some(r) => r,
        None => return spans,
    };

    // Collect code folding regions from AST
    collect_outlining_spans(ctx, root, &mut spans);

    // Collect comment regions
    collect_comment_regions(ctx, &mut spans);

    // Collect region directives
    collect_region_directives(ctx, &mut spans);

    // Collect imports section
    collect_imports_section(ctx, &mut spans);

    // Sort by start position
    spans.sort_by_key(|s| s.text_span.start);

    spans
}

// =============================================================================
// AST-Based Folding Regions
// =============================================================================

fn collect_outlining_spans(
    ctx: &OutliningContext,
    node_id: NodeId,
    spans: &mut Vec<OutliningSpan>,
) {
    let node = match ctx.arena.get(node_id) {
        Some(n) => n,
        None => return,
    };

    // Check if this node creates a folding region
    if let Some(span) = get_folding_span_for_node(ctx, node_id) {
        spans.push(span);
    }

    // Recurse into children
    for child_id in node.children() {
        collect_outlining_spans(ctx, child_id, spans);
    }
}

fn get_folding_span_for_node(
    ctx: &OutliningContext,
    node_id: NodeId,
) -> Option<OutliningSpan> {
    let node = ctx.arena.get(node_id)?;
    let kind = node.kind();

    match kind {
        // Function-like declarations
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::Constructor
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor => {
            create_function_folding_span(ctx, node_id)
        }

        // Class declarations
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
            create_class_folding_span(ctx, node_id)
        }

        // Interface/type declarations
        SyntaxKind::InterfaceDeclaration | SyntaxKind::TypeLiteral => {
            create_type_folding_span(ctx, node_id)
        }

        // Enum declarations
        SyntaxKind::EnumDeclaration => {
            create_enum_folding_span(ctx, node_id)
        }

        // Module declarations
        SyntaxKind::ModuleDeclaration => {
            create_module_folding_span(ctx, node_id)
        }

        // Block statements
        SyntaxKind::Block => {
            // Only create folding for blocks that are not part of function bodies
            // (those are handled by the function case above)
            create_block_folding_span(ctx, node_id)
        }

        // Object literals
        SyntaxKind::ObjectLiteralExpression => {
            create_object_literal_folding_span(ctx, node_id)
        }

        // Array literals (multi-line)
        SyntaxKind::ArrayLiteralExpression => {
            create_array_literal_folding_span(ctx, node_id)
        }

        // Switch statements
        SyntaxKind::CaseBlock => {
            create_case_block_folding_span(ctx, node_id)
        }

        _ => None,
    }
}

fn create_function_folding_span(
    ctx: &OutliningContext,
    node_id: NodeId,
) -> Option<OutliningSpan> {
    let node = ctx.arena.get(node_id)?;

    // Find the function body's opening brace
    let body_start = find_opening_brace_position(ctx, node_id)?;
    let body_end = node.end();

    // The hint span starts after the opening brace
    let hint_span = TextSpan::from_bounds(body_start + 1, body_end - 1);

    // The full span includes the braces
    let text_span = TextSpan::from_bounds(body_start, body_end);

    Some(OutliningSpan::new(
        text_span,
        hint_span,
        "...".to_string(),
        OutliningSpanKind::Code,
    ))
}

fn create_class_folding_span(
    ctx: &OutliningContext,
    node_id: NodeId,
) -> Option<OutliningSpan> {
    let node = ctx.arena.get(node_id)?;

    let body_start = find_opening_brace_position(ctx, node_id)?;
    let body_end = node.end();

    let hint_span = TextSpan::from_bounds(body_start + 1, body_end - 1);
    let text_span = TextSpan::from_bounds(body_start, body_end);

    Some(OutliningSpan::new(
        text_span,
        hint_span,
        "...".to_string(),
        OutliningSpanKind::Code,
    ))
}

fn create_type_folding_span(
    ctx: &OutliningContext,
    node_id: NodeId,
) -> Option<OutliningSpan> {
    let node = ctx.arena.get(node_id)?;

    // Only fold if multi-line
    if !is_multiline(ctx.source_text, node.pos(), node.end()) {
        return None;
    }

    let body_start = find_opening_brace_position(ctx, node_id)?;
    let body_end = node.end();

    let hint_span = TextSpan::from_bounds(body_start + 1, body_end - 1);
    let text_span = TextSpan::from_bounds(body_start, body_end);

    Some(OutliningSpan::new(
        text_span,
        hint_span,
        "...".to_string(),
        OutliningSpanKind::Code,
    ))
}

fn create_enum_folding_span(
    ctx: &OutliningContext,
    node_id: NodeId,
) -> Option<OutliningSpan> {
    create_type_folding_span(ctx, node_id)
}

fn create_module_folding_span(
    ctx: &OutliningContext,
    node_id: NodeId,
) -> Option<OutliningSpan> {
    create_type_folding_span(ctx, node_id)
}

fn create_block_folding_span(
    ctx: &OutliningContext,
    node_id: NodeId,
) -> Option<OutliningSpan> {
    let node = ctx.arena.get(node_id)?;

    // Only fold if multi-line
    if !is_multiline(ctx.source_text, node.pos(), node.end()) {
        return None;
    }

    let text_span = TextSpan::from_bounds(node.pos(), node.end());
    let hint_span = TextSpan::from_bounds(node.pos() + 1, node.end() - 1);

    Some(OutliningSpan::new(
        text_span,
        hint_span,
        "...".to_string(),
        OutliningSpanKind::Code,
    ))
}

fn create_object_literal_folding_span(
    ctx: &OutliningContext,
    node_id: NodeId,
) -> Option<OutliningSpan> {
    let node = ctx.arena.get(node_id)?;

    // Only fold if multi-line
    if !is_multiline(ctx.source_text, node.pos(), node.end()) {
        return None;
    }

    let text_span = TextSpan::from_bounds(node.pos(), node.end());
    let hint_span = TextSpan::from_bounds(node.pos() + 1, node.end() - 1);

    Some(OutliningSpan::new(
        text_span,
        hint_span,
        "{...}".to_string(),
        OutliningSpanKind::Code,
    ))
}

fn create_array_literal_folding_span(
    ctx: &OutliningContext,
    node_id: NodeId,
) -> Option<OutliningSpan> {
    let node = ctx.arena.get(node_id)?;

    // Only fold if multi-line
    if !is_multiline(ctx.source_text, node.pos(), node.end()) {
        return None;
    }

    let text_span = TextSpan::from_bounds(node.pos(), node.end());
    let hint_span = TextSpan::from_bounds(node.pos() + 1, node.end() - 1);

    Some(OutliningSpan::new(
        text_span,
        hint_span,
        "[...]".to_string(),
        OutliningSpanKind::Code,
    ))
}

fn create_case_block_folding_span(
    ctx: &OutliningContext,
    node_id: NodeId,
) -> Option<OutliningSpan> {
    create_block_folding_span(ctx, node_id)
}

// =============================================================================
// Comment Regions
// =============================================================================

fn collect_comment_regions(ctx: &OutliningContext, spans: &mut Vec<OutliningSpan>) {
    let source = ctx.source_text;
    let mut i = 0;

    while i < source.len() {
        // Look for block comments
        if source[i..].starts_with("/*") {
            if let Some(end) = source[i + 2..].find("*/") {
                let comment_end = i + 2 + end + 2;
                let comment_text = &source[i..comment_end];

                // Only fold multi-line comments
                if comment_text.contains('\n') {
                    let text_span = TextSpan::new(i as u32, (comment_end - i) as u32);

                    // First line as banner
                    let banner = comment_text
                        .lines()
                        .next()
                        .unwrap_or("/*...")
                        .to_string();

                    spans.push(
                        OutliningSpan::new(
                            text_span.clone(),
                            text_span,
                            banner,
                            OutliningSpanKind::Comment,
                        )
                        .auto_collapse(),
                    );
                }

                i = comment_end;
                continue;
            }
        }

        // Look for consecutive single-line comments
        if source[i..].starts_with("//") {
            let comment_start = i;
            let mut comment_end = i;
            let mut line_count = 0;

            while i < source.len() && source[i..].starts_with("//") {
                // Find end of line
                if let Some(newline) = source[i..].find('\n') {
                    comment_end = i + newline + 1;
                    i = comment_end;
                    line_count += 1;

                    // Skip whitespace at start of next line
                    while i < source.len() && source[i..].starts_with(|c: char| c.is_whitespace() && c != '\n') {
                        i += 1;
                    }
                } else {
                    comment_end = source.len();
                    break;
                }
            }

            // Only fold if 3+ consecutive comment lines
            if line_count >= 3 {
                let text_span = TextSpan::new(
                    comment_start as u32,
                    (comment_end - comment_start) as u32,
                );

                let banner = source[comment_start..]
                    .lines()
                    .next()
                    .unwrap_or("//...")
                    .to_string();

                spans.push(OutliningSpan::new(
                    text_span.clone(),
                    text_span,
                    banner,
                    OutliningSpanKind::Comment,
                ));
            }

            continue;
        }

        i += 1;
    }
}

// =============================================================================
// Region Directives
// =============================================================================

fn collect_region_directives(ctx: &OutliningContext, spans: &mut Vec<OutliningSpan>) {
    let source = ctx.source_text;
    let mut region_stack: Vec<(u32, String)> = Vec::new();

    for (i, line) in source.lines().enumerate() {
        let line_start = source[..source.len()]
            .lines()
            .take(i)
            .map(|l| l.len() + 1)
            .sum::<usize>() as u32;

        let trimmed = line.trim();

        // Check for #region
        if trimmed.starts_with("//#region") || trimmed.starts_with("// #region") {
            let name = trimmed
                .trim_start_matches("//#region")
                .trim_start_matches("// #region")
                .trim()
                .to_string();

            region_stack.push((line_start, name));
        }
        // Check for #endregion
        else if trimmed.starts_with("//#endregion") || trimmed.starts_with("// #endregion") {
            if let Some((start, name)) = region_stack.pop() {
                let end = line_start + line.len() as u32;
                let text_span = TextSpan::from_bounds(start, end);

                let banner = if name.is_empty() {
                    "#region".to_string()
                } else {
                    format!("#region {}", name)
                };

                spans.push(OutliningSpan::new(
                    text_span.clone(),
                    text_span,
                    banner,
                    OutliningSpanKind::Region,
                ));
            }
        }
    }
}

// =============================================================================
// Imports Section
// =============================================================================

fn collect_imports_section(ctx: &OutliningContext, spans: &mut Vec<OutliningSpan>) {
    let root = match ctx.arena.root() {
        Some(r) => r,
        None => return,
    };

    let source_file = match ctx.arena.get(root) {
        Some(sf) => sf,
        None => return,
    };

    // Find consecutive import statements
    let mut import_start: Option<u32> = None;
    let mut import_end: u32 = 0;
    let mut import_count = 0;

    for child_id in source_file.children() {
        let child = match ctx.arena.get(child_id) {
            Some(c) => c,
            None => continue,
        };

        if child.kind() == SyntaxKind::ImportDeclaration {
            if import_start.is_none() {
                import_start = Some(child.pos());
            }
            import_end = child.end();
            import_count += 1;
        } else if import_start.is_some() {
            // End of import section
            break;
        }
    }

    // Only fold if 3+ imports
    if let Some(start) = import_start {
        if import_count >= 3 {
            let text_span = TextSpan::from_bounds(start, import_end);

            spans.push(
                OutliningSpan::new(
                    text_span.clone(),
                    text_span,
                    format!("import ... ({} imports)", import_count),
                    OutliningSpanKind::Imports,
                )
                .auto_collapse(),
            );
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn find_opening_brace_position(ctx: &OutliningContext, node_id: NodeId) -> Option<u32> {
    let node = ctx.arena.get(node_id)?;
    let start = node.pos() as usize;
    let end = node.end() as usize;

    let text = &ctx.source_text[start..end];
    text.find('{').map(|offset| (start + offset) as u32)
}

fn is_multiline(source: &str, start: u32, end: u32) -> bool {
    let start = start as usize;
    let end = end as usize;

    if end > source.len() {
        return false;
    }

    source[start..end].contains('\n')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outlining_span() {
        let span = OutliningSpan::new(
            TextSpan::new(0, 100),
            TextSpan::new(1, 98),
            "...".to_string(),
            OutliningSpanKind::Code,
        );

        assert_eq!(span.kind, OutliningSpanKind::Code);
        assert!(!span.auto_collapse);
    }

    #[test]
    fn test_outlining_span_auto_collapse() {
        let span = OutliningSpan::new(
            TextSpan::new(0, 100),
            TextSpan::new(1, 98),
            "...".to_string(),
            OutliningSpanKind::Comment,
        )
        .auto_collapse();

        assert!(span.auto_collapse);
    }

    #[test]
    fn test_is_multiline() {
        assert!(is_multiline("hello\nworld", 0, 11));
        assert!(!is_multiline("hello world", 0, 11));
    }

    #[test]
    fn test_outlining_span_kind() {
        assert_eq!(OutliningSpanKind::Code.as_str(), "code");
        assert_eq!(OutliningSpanKind::Comment.as_str(), "comment");
        assert_eq!(OutliningSpanKind::Region.as_str(), "region");
        assert_eq!(OutliningSpanKind::Imports.as_str(), "imports");
    }
}
