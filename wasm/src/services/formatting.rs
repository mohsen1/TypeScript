//! Formatting implementation.
//!
//! This module provides code formatting functionality including:
//! - Document formatting
//! - Range formatting
//! - On-type formatting (format on enter, semicolon, etc.)

use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::text_span::TextSpan;
use super::text_changes::{FormatCodeSettings, TextChange};

// =============================================================================
// Formatting Types
// =============================================================================

/// Options for formatting operations.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Base formatting settings.
    pub settings: FormatCodeSettings,
    /// Whether to insert final newline.
    pub insert_final_newline: bool,
    /// Whether to trim final newlines.
    pub trim_final_newlines: bool,
    /// Whether to trim trailing whitespace.
    pub trim_trailing_whitespace: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            settings: FormatCodeSettings::default(),
            insert_final_newline: true,
            trim_final_newlines: false,
            trim_trailing_whitespace: true,
        }
    }
}

/// Result of a formatting operation.
#[derive(Debug, Clone)]
pub struct FormatResult {
    /// The text changes to apply.
    pub changes: Vec<TextChange>,
}

impl FormatResult {
    pub fn new(changes: Vec<TextChange>) -> Self {
        Self { changes }
    }

    pub fn empty() -> Self {
        Self { changes: Vec::new() }
    }

    /// Apply the formatting changes to source text.
    pub fn apply(&self, source: &str) -> String {
        if self.changes.is_empty() {
            return source.to_string();
        }

        let mut result = source.to_string();

        // Apply changes in reverse order to preserve positions
        let mut sorted_changes = self.changes.clone();
        sorted_changes.sort_by(|a, b| b.span.start.cmp(&a.span.start));

        for change in sorted_changes {
            let start = change.span.start as usize;
            let end = start + change.span.length as usize;

            if end <= result.len() {
                result.replace_range(start..end, &change.new_text);
            }
        }

        result
    }
}

// =============================================================================
// Formatting Context
// =============================================================================

/// Context for formatting operations.
pub struct FormattingContext<'a> {
    pub arena: &'a NodeArena,
    pub source_text: &'a str,
    pub options: FormatOptions,
}

// =============================================================================
// Document Formatting
// =============================================================================

/// Format an entire document.
pub fn get_formatting_edits_for_document(
    ctx: &FormattingContext,
) -> Vec<TextChange> {
    let root = match ctx.arena.root() {
        Some(r) => r,
        None => return Vec::new(),
    };

    let source_file = match ctx.arena.get(root) {
        Some(sf) => sf,
        None => return Vec::new(),
    };

    let span = TextSpan::from_bounds(0, source_file.end());
    format_span(ctx, &span)
}

/// Format a range within a document.
pub fn get_formatting_edits_for_range(
    ctx: &FormattingContext,
    start: u32,
    end: u32,
) -> Vec<TextChange> {
    // Expand to full statements/declarations
    let (adjusted_start, adjusted_end) = adjust_range_to_statements(ctx, start, end);

    let span = TextSpan::from_bounds(adjusted_start, adjusted_end);
    format_span(ctx, &span)
}

/// Format after typing a specific character.
pub fn get_formatting_edits_after_keystroke(
    ctx: &FormattingContext,
    position: u32,
    key: char,
) -> Vec<TextChange> {
    match key {
        ';' => format_after_semicolon(ctx, position),
        '}' => format_after_closing_brace(ctx, position),
        '\n' => format_after_newline(ctx, position),
        _ => Vec::new(),
    }
}

// =============================================================================
// Core Formatting Logic
// =============================================================================

fn format_span(ctx: &FormattingContext, span: &TextSpan) -> Vec<TextChange> {
    let mut changes = Vec::new();

    // Get the text in the span
    let start = span.start as usize;
    let end = start + span.length as usize;

    if end > ctx.source_text.len() {
        return changes;
    }

    let text = &ctx.source_text[start..end];

    // In a full implementation, we would:
    // 1. Parse the formatting rules
    // 2. Walk the AST within the span
    // 3. Apply indentation rules
    // 4. Apply spacing rules
    // 5. Generate text changes

    // For now, apply basic whitespace normalization
    changes.extend(normalize_indentation(ctx, span));
    changes.extend(normalize_spacing(ctx, span));
    changes.extend(handle_trailing_whitespace(ctx, span));

    changes
}

fn normalize_indentation(ctx: &FormattingContext, span: &TextSpan) -> Vec<TextChange> {
    let mut changes = Vec::new();

    let indent_size = ctx.options.settings.indent_size;
    let use_tabs = !ctx.options.settings.convert_tabs_to_spaces;
    let indent_char = if use_tabs { "\t" } else { &" ".repeat(indent_size as usize) };

    // In a full implementation:
    // 1. Track nesting level through braces, parentheses
    // 2. Handle special cases (switch statements, chained methods)
    // 3. Generate changes for incorrect indentation

    changes
}

fn normalize_spacing(ctx: &FormattingContext, span: &TextSpan) -> Vec<TextChange> {
    let mut changes = Vec::new();

    // In a full implementation:
    // 1. Ensure proper spacing around operators
    // 2. Handle spacing after keywords
    // 3. Handle spacing in function calls
    // 4. Apply comma and colon spacing rules

    changes
}

fn handle_trailing_whitespace(ctx: &FormattingContext, span: &TextSpan) -> Vec<TextChange> {
    let mut changes = Vec::new();

    if !ctx.options.trim_trailing_whitespace {
        return changes;
    }

    let start = span.start as usize;
    let end = start + span.length as usize;

    if end > ctx.source_text.len() {
        return changes;
    }

    let text = &ctx.source_text[start..end];

    // Find lines with trailing whitespace
    let mut line_start = 0;
    for (i, c) in text.char_indices() {
        if c == '\n' {
            // Check for trailing whitespace before newline
            let line_end = i;
            if let Some(ws_start) = find_trailing_whitespace(&text[line_start..line_end]) {
                let abs_start = start + line_start + ws_start;
                let ws_length = line_end - line_start - ws_start;

                changes.push(TextChange::new(
                    TextSpan::new(abs_start as u32, ws_length as u32),
                    String::new(),
                ));
            }
            line_start = i + 1;
        }
    }

    changes
}

fn find_trailing_whitespace(line: &str) -> Option<usize> {
    let trimmed_len = line.trim_end().len();
    if trimmed_len < line.len() {
        Some(trimmed_len)
    } else {
        None
    }
}

// =============================================================================
// On-Type Formatting
// =============================================================================

fn format_after_semicolon(ctx: &FormattingContext, position: u32) -> Vec<TextChange> {
    let mut changes = Vec::new();

    // Find the statement containing this semicolon
    // and format just that statement

    changes
}

fn format_after_closing_brace(ctx: &FormattingContext, position: u32) -> Vec<TextChange> {
    let mut changes = Vec::new();

    // Find the block that was just closed
    // and format the entire block

    // Also handle else/catch/finally that might follow

    changes
}

fn format_after_newline(ctx: &FormattingContext, position: u32) -> Vec<TextChange> {
    let mut changes = Vec::new();

    // Calculate the expected indentation for the new line
    // based on the previous line's content

    let expected_indent = calculate_indent_at_position(ctx, position);

    // If the cursor is at a position with incorrect indentation,
    // generate a change to fix it

    changes
}

fn calculate_indent_at_position(ctx: &FormattingContext, position: u32) -> u32 {
    // Find the current nesting level
    let nesting_level = calculate_nesting_level(ctx, position);

    nesting_level * ctx.options.settings.indent_size as u32
}

fn calculate_nesting_level(ctx: &FormattingContext, position: u32) -> u32 {
    let pos = position as usize;
    let text = ctx.source_text;

    if pos > text.len() {
        return 0;
    }

    let mut level: i32 = 0;

    for (i, c) in text[..pos].char_indices() {
        match c {
            '{' | '(' | '[' => level += 1,
            '}' | ')' | ']' => level = (level - 1).max(0),
            _ => {}
        }
    }

    level.max(0) as u32
}

// =============================================================================
// Range Adjustment
// =============================================================================

fn adjust_range_to_statements(
    ctx: &FormattingContext,
    start: u32,
    end: u32,
) -> (u32, u32) {
    // In a full implementation, expand the range to include:
    // - Full statements/declarations
    // - Opening/closing braces of blocks

    // For now, just return the original range
    (start, end)
}

// =============================================================================
// Formatting Rules
// =============================================================================

/// Rules for spacing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpacingRule {
    /// No space.
    None,
    /// Single space.
    Space,
    /// Newline.
    Newline,
    /// Preserve existing.
    Preserve,
}

/// Get the spacing rule between two tokens.
pub fn get_spacing_rule(
    options: &FormatCodeSettings,
    left_kind: SyntaxKind,
    right_kind: SyntaxKind,
) -> SpacingRule {
    // Binary operators
    if is_binary_operator(right_kind) {
        if options.insert_space_before_and_after_binary_operators {
            return SpacingRule::Space;
        }
    }

    // Function call parentheses
    if right_kind == SyntaxKind::OpenParenToken {
        if left_kind == SyntaxKind::Identifier {
            if options.insert_space_before_function_parenthesis {
                return SpacingRule::Space;
            } else {
                return SpacingRule::None;
            }
        }
    }

    // Comma
    if left_kind == SyntaxKind::CommaToken {
        return SpacingRule::Space;
    }

    // Colon in type annotations
    if left_kind == SyntaxKind::ColonToken {
        return SpacingRule::Space;
    }

    // Semicolon
    if left_kind == SyntaxKind::SemicolonToken {
        return SpacingRule::Preserve;
    }

    // Default: preserve
    SpacingRule::Preserve
}

fn is_binary_operator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::PlusToken
            | SyntaxKind::MinusToken
            | SyntaxKind::AsteriskToken
            | SyntaxKind::SlashToken
            | SyntaxKind::PercentToken
            | SyntaxKind::EqualsToken
            | SyntaxKind::EqualsEqualsToken
            | SyntaxKind::EqualsEqualsEqualsToken
            | SyntaxKind::ExclamationEqualsToken
            | SyntaxKind::ExclamationEqualsEqualsToken
            | SyntaxKind::LessThanToken
            | SyntaxKind::GreaterThanToken
            | SyntaxKind::LessThanEqualsToken
            | SyntaxKind::GreaterThanEqualsToken
            | SyntaxKind::AmpersandAmpersandToken
            | SyntaxKind::BarBarToken
            | SyntaxKind::AmpersandToken
            | SyntaxKind::BarToken
            | SyntaxKind::CaretToken
    )
}

// =============================================================================
// Semicolon Handling
// =============================================================================

/// Options for semicolon insertion/removal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SemicolonPreference {
    #[default]
    Ignore,
    Insert,
    Remove,
}

/// Get edits to conform to semicolon preference.
pub fn get_semicolon_edits(
    ctx: &FormattingContext,
    preference: SemicolonPreference,
) -> Vec<TextChange> {
    let mut changes = Vec::new();

    match preference {
        SemicolonPreference::Ignore => {}
        SemicolonPreference::Insert => {
            // Find statements missing semicolons
            // and insert them
        }
        SemicolonPreference::Remove => {
            // Find unnecessary semicolons
            // and remove them
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_result_apply() {
        let result = FormatResult::new(vec![
            TextChange::new(TextSpan::new(5, 1), "  ".to_string()),
        ]);

        let formatted = result.apply("hello world");
        assert_eq!(formatted, "hello  world");
    }

    #[test]
    fn test_format_result_apply_multiple() {
        let result = FormatResult::new(vec![
            TextChange::new(TextSpan::new(0, 5), "hi".to_string()),
            TextChange::new(TextSpan::new(6, 5), "there".to_string()),
        ]);

        let formatted = result.apply("hello world");
        assert_eq!(formatted, "hi there");
    }

    #[test]
    fn test_find_trailing_whitespace() {
        assert_eq!(find_trailing_whitespace("hello  "), Some(5));
        assert_eq!(find_trailing_whitespace("hello"), None);
        assert_eq!(find_trailing_whitespace("  "), Some(0));
    }

    #[test]
    fn test_spacing_rule() {
        let options = FormatCodeSettings::default();

        // Comma should be followed by space
        assert_eq!(
            get_spacing_rule(&options, SyntaxKind::CommaToken, SyntaxKind::Identifier),
            SpacingRule::Space
        );
    }

    #[test]
    fn test_is_binary_operator() {
        assert!(is_binary_operator(SyntaxKind::PlusToken));
        assert!(is_binary_operator(SyntaxKind::EqualsEqualsEqualsToken));
        assert!(!is_binary_operator(SyntaxKind::OpenParenToken));
    }
}
