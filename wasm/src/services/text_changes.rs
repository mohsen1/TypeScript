//! Text changes system for code modifications.
//!
//! This module provides the `ChangeTracker` for accumulating text edits
//! in a source file. It's used by code fixes and refactorings.

use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::text_span::{TextSpan, TextChange, LineAndCharacter};

use std::collections::BTreeMap;

// =============================================================================
// Format Code Settings
// =============================================================================

/// Settings for code formatting.
/// Matches TypeScript's `FormatCodeSettings` interface.
#[derive(Debug, Clone)]
pub struct FormatCodeSettings {
    pub indent_size: u32,
    pub tab_size: u32,
    pub new_line_character: String,
    pub convert_tabs_to_spaces: bool,
    pub indent_style: IndentStyle,
    pub insert_space_after_comma_delimiter: bool,
    pub insert_space_after_semicolon_in_for_statements: bool,
    pub insert_space_before_and_after_binary_operators: bool,
    pub insert_space_after_constructor: bool,
    pub insert_space_after_keywords_in_control_flow_statements: bool,
    pub insert_space_after_function_keyword_for_anonymous_functions: bool,
    pub insert_space_after_opening_and_before_closing_nonempty_parenthesis: bool,
    pub insert_space_after_opening_and_before_closing_nonempty_brackets: bool,
    pub insert_space_after_opening_and_before_closing_nonempty_braces: bool,
    pub insert_space_after_opening_and_before_closing_template_string_braces: bool,
    pub insert_space_after_opening_and_before_closing_jsx_expression_braces: bool,
    pub insert_space_after_type_assertion: bool,
    pub insert_space_before_function_parenthesis: bool,
    pub place_open_brace_on_new_line_for_functions: bool,
    pub place_open_brace_on_new_line_for_control_blocks: bool,
    pub insert_space_before_type_annotation: bool,
    pub semicolons: SemicolonPreference,
}

impl Default for FormatCodeSettings {
    fn default() -> Self {
        Self {
            indent_size: 4,
            tab_size: 4,
            new_line_character: "\n".to_string(),
            convert_tabs_to_spaces: true,
            indent_style: IndentStyle::Smart,
            insert_space_after_comma_delimiter: true,
            insert_space_after_semicolon_in_for_statements: true,
            insert_space_before_and_after_binary_operators: true,
            insert_space_after_constructor: false,
            insert_space_after_keywords_in_control_flow_statements: true,
            insert_space_after_function_keyword_for_anonymous_functions: false,
            insert_space_after_opening_and_before_closing_nonempty_parenthesis: false,
            insert_space_after_opening_and_before_closing_nonempty_brackets: false,
            insert_space_after_opening_and_before_closing_nonempty_braces: true,
            insert_space_after_opening_and_before_closing_template_string_braces: false,
            insert_space_after_opening_and_before_closing_jsx_expression_braces: false,
            insert_space_after_type_assertion: false,
            insert_space_before_function_parenthesis: false,
            place_open_brace_on_new_line_for_functions: false,
            place_open_brace_on_new_line_for_control_blocks: false,
            insert_space_before_type_annotation: false,
            semicolons: SemicolonPreference::Ignore,
        }
    }
}

/// Indent style for formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentStyle {
    None,
    Block,
    Smart,
}

/// Semicolon preference for formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemicolonPreference {
    Ignore,
    Insert,
    Remove,
}

// =============================================================================
// Text Changes Context
// =============================================================================

/// Context for text changes operations.
#[derive(Debug, Clone)]
pub struct TextChangesContext {
    pub format_settings: FormatCodeSettings,
    pub source_text: String,
    pub file_name: String,
}

impl TextChangesContext {
    pub fn new(file_name: String, source_text: String) -> Self {
        Self {
            format_settings: FormatCodeSettings::default(),
            source_text,
            file_name,
        }
    }

    pub fn with_format_settings(mut self, settings: FormatCodeSettings) -> Self {
        self.format_settings = settings;
        self
    }
}

// =============================================================================
// Change Tracker
// =============================================================================

/// Accumulates text changes for a source file.
///
/// Changes are stored in a map keyed by position to ensure proper ordering
/// and to detect overlapping changes.
#[derive(Debug)]
pub struct ChangeTracker {
    /// File name this tracker is for.
    file_name: String,
    /// Source text of the file.
    source_text: String,
    /// Format settings to use.
    format_settings: FormatCodeSettings,
    /// Accumulated changes, keyed by start position for ordering.
    /// Using BTreeMap to keep changes sorted by position.
    changes: BTreeMap<u32, TextChange>,
    /// Whether to track changes for multiple files.
    changes_by_file: BTreeMap<String, Vec<TextChange>>,
}

impl ChangeTracker {
    /// Create a new change tracker.
    pub fn new(context: TextChangesContext) -> Self {
        Self {
            file_name: context.file_name,
            source_text: context.source_text,
            format_settings: context.format_settings,
            changes: BTreeMap::new(),
            changes_by_file: BTreeMap::new(),
        }
    }

    /// Create a change tracker and run a callback, returning the changes.
    pub fn with<F>(context: TextChangesContext, f: F) -> Vec<TextChange>
    where
        F: FnOnce(&mut ChangeTracker),
    {
        let mut tracker = ChangeTracker::new(context);
        f(&mut tracker);
        tracker.get_changes()
    }

    /// Get the accumulated changes.
    pub fn get_changes(&self) -> Vec<TextChange> {
        self.changes.values().cloned().collect()
    }

    /// Get changes for all files.
    pub fn get_changes_by_file(&self) -> &BTreeMap<String, Vec<TextChange>> {
        &self.changes_by_file
    }

    /// Get the new line character to use.
    fn get_new_line(&self) -> &str {
        &self.format_settings.new_line_character
    }

    /// Get indentation string for a level.
    fn get_indent(&self, level: u32) -> String {
        let size = self.format_settings.indent_size * level;
        if self.format_settings.convert_tabs_to_spaces {
            " ".repeat(size as usize)
        } else {
            "\t".repeat((size / self.format_settings.tab_size) as usize)
        }
    }

    // =========================================================================
    // Insert Operations
    // =========================================================================

    /// Insert text at a position.
    pub fn insert_text(&mut self, position: u32, text: String) {
        self.add_change(TextChange::insert(position, text));
    }

    /// Insert text at the start of the file.
    pub fn insert_at_start(&mut self, text: String) {
        self.insert_text(0, text);
    }

    /// Insert text at the end of the file.
    pub fn insert_at_end(&mut self, text: String) {
        let pos = self.source_text.len() as u32;
        self.insert_text(pos, text);
    }

    /// Insert a new line at a position.
    pub fn insert_newline(&mut self, position: u32) {
        self.insert_text(position, self.get_new_line().to_string());
    }

    /// Insert text before a node.
    pub fn insert_before(&mut self, arena: &NodeArena, node_id: NodeId, text: String) {
        if let Some(node) = arena.get(node_id) {
            self.insert_text(node.pos(), text);
        }
    }

    /// Insert text after a node.
    pub fn insert_after(&mut self, arena: &NodeArena, node_id: NodeId, text: String) {
        if let Some(node) = arena.get(node_id) {
            self.insert_text(node.end(), text);
        }
    }

    /// Insert a node before another node (as text).
    pub fn insert_node_before(
        &mut self,
        arena: &NodeArena,
        before_node: NodeId,
        new_node_text: String,
        with_newline: bool,
    ) {
        if let Some(node) = arena.get(before_node) {
            let pos = node.pos();
            let text = if with_newline {
                format!("{}{}", new_node_text, self.get_new_line())
            } else {
                new_node_text
            };
            self.insert_text(pos, text);
        }
    }

    /// Insert a node after another node (as text).
    pub fn insert_node_after(
        &mut self,
        arena: &NodeArena,
        after_node: NodeId,
        new_node_text: String,
        with_newline: bool,
    ) {
        if let Some(node) = arena.get(after_node) {
            let pos = node.end();
            let text = if with_newline {
                format!("{}{}", self.get_new_line(), new_node_text)
            } else {
                new_node_text
            };
            self.insert_text(pos, text);
        }
    }

    // =========================================================================
    // Delete Operations
    // =========================================================================

    /// Delete a span of text.
    pub fn delete_range(&mut self, span: TextSpan) {
        self.add_change(TextChange::delete(span));
    }

    /// Delete a node.
    pub fn delete_node(&mut self, arena: &NodeArena, node_id: NodeId) {
        if let Some(node) = arena.get(node_id) {
            let span = TextSpan::from_bounds(node.pos(), node.end());
            self.delete_range(span);
        }
    }

    /// Delete a node and its surrounding trivia.
    pub fn delete_node_with_trivia(
        &mut self,
        arena: &NodeArena,
        node_id: NodeId,
        include_leading: bool,
        include_trailing: bool,
    ) {
        if let Some(node) = arena.get(node_id) {
            let mut start = node.pos();
            let mut end = node.end();

            // Extend to include leading trivia if requested
            if include_leading && start > 0 {
                // Skip back over whitespace
                let chars: Vec<char> = self.source_text.chars().collect();
                while start > 0 && chars.get((start - 1) as usize).map_or(false, |c| c.is_whitespace()) {
                    start -= 1;
                }
            }

            // Extend to include trailing trivia if requested
            if include_trailing {
                let chars: Vec<char> = self.source_text.chars().collect();
                while (end as usize) < chars.len() && chars.get(end as usize).map_or(false, |c| c.is_whitespace()) {
                    end += 1;
                }
            }

            let span = TextSpan::from_bounds(start, end);
            self.delete_range(span);
        }
    }

    /// Delete a range of nodes.
    pub fn delete_node_range(
        &mut self,
        arena: &NodeArena,
        start_node: NodeId,
        end_node: NodeId,
    ) {
        let start = arena.get(start_node).map(|n| n.pos());
        let end = arena.get(end_node).map(|n| n.end());
        if let (Some(start), Some(end)) = (start, end) {
            self.delete_range(TextSpan::from_bounds(start, end));
        }
    }

    // =========================================================================
    // Replace Operations
    // =========================================================================

    /// Replace a span with new text.
    pub fn replace_range(&mut self, span: TextSpan, new_text: String) {
        self.add_change(TextChange::replace(span, new_text));
    }

    /// Replace a node with new text.
    pub fn replace_node(&mut self, arena: &NodeArena, node_id: NodeId, new_text: String) {
        if let Some(node) = arena.get(node_id) {
            let span = TextSpan::from_bounds(node.pos(), node.end());
            self.replace_range(span, new_text);
        }
    }

    /// Replace a node with another node (as text).
    pub fn replace_node_with_node(
        &mut self,
        arena: &NodeArena,
        old_node: NodeId,
        new_node_text: String,
    ) {
        self.replace_node(arena, old_node, new_node_text);
    }

    /// Replace a range of nodes with new text.
    pub fn replace_node_range(
        &mut self,
        arena: &NodeArena,
        start_node: NodeId,
        end_node: NodeId,
        new_text: String,
    ) {
        let start = arena.get(start_node).map(|n| n.pos());
        let end = arena.get(end_node).map(|n| n.end());
        if let (Some(start), Some(end)) = (start, end) {
            self.replace_range(TextSpan::from_bounds(start, end), new_text);
        }
    }

    // =========================================================================
    // Internal Methods
    // =========================================================================

    /// Add a change, checking for overlaps.
    fn add_change(&mut self, change: TextChange) {
        // Check for overlapping changes
        let start = change.span.start;
        let end = change.span.end();

        // Remove any changes that would overlap
        let overlapping: Vec<u32> = self
            .changes
            .iter()
            .filter(|(_, c)| {
                let c_start = c.span.start;
                let c_end = c.span.end();
                // Check if ranges overlap
                start < c_end && c_start < end
            })
            .map(|(k, _)| *k)
            .collect();

        for key in overlapping {
            self.changes.remove(&key);
        }

        self.changes.insert(start, change);
    }
}

// =============================================================================
// File Text Changes
// =============================================================================

/// Text changes for a file.
/// Matches TypeScript's `FileTextChanges` interface.
#[derive(Debug, Clone)]
pub struct FileTextChanges {
    pub file_name: String,
    pub text_changes: Vec<TextChange>,
    pub is_new_file: Option<bool>,
}

impl FileTextChanges {
    pub fn new(file_name: String, changes: Vec<TextChange>) -> Self {
        Self {
            file_name,
            text_changes: changes,
            is_new_file: None,
        }
    }

    pub fn new_file(file_name: String, content: String) -> Self {
        Self {
            file_name,
            text_changes: vec![TextChange::insert(0, content)],
            is_new_file: Some(true),
        }
    }
}

// =============================================================================
// Apply Changes
// =============================================================================

/// Apply a list of text changes to source text.
/// Changes must be sorted and non-overlapping.
pub fn apply_text_changes(source: &str, changes: &[TextChange]) -> String {
    if changes.is_empty() {
        return source.to_string();
    }

    // Sort changes by position (descending) to apply from end to start
    let mut sorted_changes: Vec<&TextChange> = changes.iter().collect();
    sorted_changes.sort_by(|a, b| b.span.start.cmp(&a.span.start));

    let mut result = source.to_string();

    for change in sorted_changes {
        let start = change.span.start as usize;
        let end = change.span.end() as usize;

        if start <= result.len() && end <= result.len() {
            result = format!(
                "{}{}{}",
                &result[..start],
                &change.new_text,
                &result[end..]
            );
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_text() {
        let context = TextChangesContext::new("test.ts".to_string(), "let x = 1;".to_string());
        let changes = ChangeTracker::with(context, |tracker| {
            tracker.insert_text(4, "y, ".to_string());
        });

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].span.start, 4);
        assert_eq!(changes[0].new_text, "y, ");
    }

    #[test]
    fn test_delete_range() {
        let context = TextChangesContext::new("test.ts".to_string(), "let x = 1;".to_string());
        let changes = ChangeTracker::with(context, |tracker| {
            tracker.delete_range(TextSpan::new(4, 2)); // delete "x "
        });

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].span.start, 4);
        assert_eq!(changes[0].span.length, 2);
        assert!(changes[0].new_text.is_empty());
    }

    #[test]
    fn test_replace_range() {
        let context = TextChangesContext::new("test.ts".to_string(), "let x = 1;".to_string());
        let changes = ChangeTracker::with(context, |tracker| {
            tracker.replace_range(TextSpan::new(4, 1), "y".to_string()); // replace "x" with "y"
        });

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].span.start, 4);
        assert_eq!(changes[0].new_text, "y");
    }

    #[test]
    fn test_apply_changes() {
        let source = "let x = 1;";
        let changes = vec![TextChange::replace(TextSpan::new(4, 1), "y".to_string())];

        let result = apply_text_changes(source, &changes);
        assert_eq!(result, "let y = 1;");
    }

    #[test]
    fn test_apply_multiple_changes() {
        let source = "let x = 1;";
        let changes = vec![
            TextChange::replace(TextSpan::new(4, 1), "y".to_string()),
            TextChange::replace(TextSpan::new(8, 1), "2".to_string()),
        ];

        let result = apply_text_changes(source, &changes);
        assert_eq!(result, "let y = 2;");
    }

    #[test]
    fn test_apply_insert() {
        let source = "let x = 1;";
        let changes = vec![TextChange::insert(4, "foo, ".to_string())];

        let result = apply_text_changes(source, &changes);
        assert_eq!(result, "let foo, x = 1;");
    }
}
