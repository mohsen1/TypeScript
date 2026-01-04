//! Refactor Provider infrastructure.
//!
//! This module provides the core infrastructure for code refactorings,
//! including the registration system and base types.

use std::collections::HashMap;
use crate::binder::{Symbol, SymbolFlags};
use crate::checker::CheckerState;
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::code_fix_provider::{FileTextChanges, CodeActionCommand};
use super::text_span::TextSpan;

// =============================================================================
// Refactor Types
// =============================================================================

/// Information about available refactorings.
#[derive(Debug, Clone)]
pub struct ApplicableRefactorInfo {
    /// The unique identifier for this refactoring.
    pub name: String,
    /// A human-readable description.
    pub description: String,
    /// Whether interacting with this is likely to result in an error.
    pub is_potentially_unsafe: bool,
    /// The specific actions available.
    pub actions: Vec<RefactorActionInfo>,
}

impl ApplicableRefactorInfo {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            is_potentially_unsafe: false,
            actions: Vec::new(),
        }
    }

    pub fn with_action(mut self, action: RefactorActionInfo) -> Self {
        self.actions.push(action);
        self
    }

    pub fn unsafe_refactor(mut self) -> Self {
        self.is_potentially_unsafe = true;
        self
    }
}

/// Information about a specific refactor action.
#[derive(Debug, Clone)]
pub struct RefactorActionInfo {
    /// The unique identifier for this action.
    pub name: String,
    /// A human-readable description.
    pub description: String,
    /// Whether this action requires additional user input.
    pub not_applicable_reason: Option<String>,
    /// The kind of refactoring (for UI categorization).
    pub kind: Option<String>,
    /// Whether this is the preferred action.
    pub is_preferred: Option<bool>,
}

impl RefactorActionInfo {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            not_applicable_reason: None,
            kind: None,
            is_preferred: None,
        }
    }

    pub fn not_applicable(mut self, reason: String) -> Self {
        self.not_applicable_reason = Some(reason);
        self
    }

    pub fn with_kind(mut self, kind: String) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn preferred(mut self) -> Self {
        self.is_preferred = Some(true);
        self
    }
}

/// The result of applying a refactoring.
#[derive(Debug, Clone)]
pub struct RefactorEditInfo {
    /// The text changes to apply.
    pub edits: Vec<FileTextChanges>,
    /// Optional file to rename.
    pub rename_file_name: Option<String>,
    /// Optional new file name.
    pub rename_location: Option<u32>,
    /// Commands to execute after applying changes.
    pub commands: Option<Vec<CodeActionCommand>>,
}

impl RefactorEditInfo {
    pub fn new(edits: Vec<FileTextChanges>) -> Self {
        Self {
            edits,
            rename_file_name: None,
            rename_location: None,
            commands: None,
        }
    }

    pub fn with_rename(mut self, file_name: String, location: u32) -> Self {
        self.rename_file_name = Some(file_name);
        self.rename_location = Some(location);
        self
    }
}

// =============================================================================
// Refactor Context
// =============================================================================

/// Context for refactor operations.
pub struct RefactorContext<'a> {
    pub arena: &'a NodeArena,
    pub checker: &'a CheckerState,
    pub source_text: &'a str,
    pub file_name: &'a str,
    /// The start of the selection.
    pub start_position: u32,
    /// The end of the selection (if range-based).
    pub end_position: Option<u32>,
    /// Preferences for refactoring.
    pub preferences: RefactorPreferences,
    /// The trigger kind.
    pub trigger_kind: RefactorTriggerKind,
}

impl<'a> RefactorContext<'a> {
    /// Get the selection span.
    pub fn selection_span(&self) -> TextSpan {
        let length = self.end_position
            .map(|end| end - self.start_position)
            .unwrap_or(0);
        TextSpan::new(self.start_position, length)
    }

    /// Check if this is a range selection.
    pub fn is_range_selection(&self) -> bool {
        self.end_position.is_some()
    }
}

/// Preferences for refactoring.
#[derive(Debug, Clone, Default)]
pub struct RefactorPreferences {
    /// Whether to allow extract to type alias.
    pub allow_extract_to_type_alias: bool,
    /// Whether to allow move to new file.
    pub allow_move_to_new_file: bool,
    /// Whether to provide prefix for extracted variables.
    pub provide_prefix_and_suffix_text_for_rename: bool,
}

/// The trigger kind for refactoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RefactorTriggerKind {
    /// Implicit trigger (e.g., from lightbulb).
    #[default]
    Implicit,
    /// Explicit trigger (e.g., from menu).
    Invoked,
}

// =============================================================================
// Refactor Registry
// =============================================================================

/// Type for checking if a refactor is applicable.
pub type RefactorApplicabilityChecker = fn(&RefactorContext) -> Option<ApplicableRefactorInfo>;

/// Type for getting refactor edits.
pub type RefactorEditor = fn(&RefactorContext, &str) -> Option<RefactorEditInfo>;

/// A registered refactoring.
#[derive(Clone)]
struct RegisteredRefactor {
    name: String,
    description: String,
    kinds: Vec<String>,
    check_applicable: RefactorApplicabilityChecker,
    get_edits: RefactorEditor,
}

/// Registry of all available refactorings.
#[derive(Default)]
pub struct RefactorRegistry {
    refactors: Vec<RegisteredRefactor>,
    by_name: HashMap<String, usize>,
}

impl RefactorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a refactoring.
    pub fn register(
        &mut self,
        name: &str,
        description: &str,
        kinds: &[&str],
        check_applicable: RefactorApplicabilityChecker,
        get_edits: RefactorEditor,
    ) {
        let index = self.refactors.len();
        let refactor = RegisteredRefactor {
            name: name.to_string(),
            description: description.to_string(),
            kinds: kinds.iter().map(|s| s.to_string()).collect(),
            check_applicable,
            get_edits,
        };

        self.refactors.push(refactor);
        self.by_name.insert(name.to_string(), index);
    }

    /// Get all applicable refactorings.
    pub fn get_applicable_refactors(
        &self,
        ctx: &RefactorContext,
        kind_filter: Option<&str>,
    ) -> Vec<ApplicableRefactorInfo> {
        let mut applicable = Vec::new();

        for refactor in &self.refactors {
            // Filter by kind if specified
            if let Some(filter) = kind_filter {
                if !refactor.kinds.iter().any(|k| k.starts_with(filter)) {
                    continue;
                }
            }

            // Check if applicable
            if let Some(info) = (refactor.check_applicable)(ctx) {
                applicable.push(info);
            }
        }

        applicable
    }

    /// Get edits for a specific refactoring.
    pub fn get_edits_for_refactor(
        &self,
        ctx: &RefactorContext,
        refactor_name: &str,
        action_name: &str,
    ) -> Option<RefactorEditInfo> {
        let index = self.by_name.get(refactor_name)?;
        let refactor = &self.refactors[*index];
        (refactor.get_edits)(ctx, action_name)
    }
}

// =============================================================================
// Refactor Action Kinds
// =============================================================================

/// Standard refactoring kinds (for organization in UI).
pub mod refactor_kinds {
    pub const EXTRACT: &str = "refactor.extract";
    pub const EXTRACT_FUNCTION: &str = "refactor.extract.function";
    pub const EXTRACT_CONSTANT: &str = "refactor.extract.constant";
    pub const EXTRACT_TYPE: &str = "refactor.extract.type";

    pub const INLINE: &str = "refactor.inline";
    pub const INLINE_VARIABLE: &str = "refactor.inline.variable";
    pub const INLINE_FUNCTION: &str = "refactor.inline.function";

    pub const MOVE: &str = "refactor.move";
    pub const MOVE_TO_NEW_FILE: &str = "refactor.move.newFile";

    pub const REWRITE: &str = "refactor.rewrite";
    pub const CONVERT_TO_ARROW: &str = "refactor.rewrite.convertToArrowFunction";
    pub const CONVERT_TO_FUNCTION: &str = "refactor.rewrite.convertToNamedFunction";
    pub const CONVERT_TO_ASYNC: &str = "refactor.rewrite.convertToAsyncFunction";
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Find the node at a position.
pub fn find_node_at_position(arena: &NodeArena, position: u32) -> Option<NodeId> {
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

/// Find nodes within a range.
pub fn find_nodes_in_range(
    arena: &NodeArena,
    start: u32,
    end: u32,
) -> Vec<NodeId> {
    let mut nodes = Vec::new();

    if let Some(root) = arena.root() {
        collect_nodes_in_range(arena, root, start, end, &mut nodes);
    }

    nodes
}

fn collect_nodes_in_range(
    arena: &NodeArena,
    node_id: NodeId,
    start: u32,
    end: u32,
    nodes: &mut Vec<NodeId>,
) {
    let node = match arena.get(node_id) {
        Some(n) => n,
        None => return,
    };

    // Check if node overlaps with range
    if node.pos() >= end || node.end() <= start {
        return;
    }

    // If fully contained, add it
    if node.pos() >= start && node.end() <= end {
        nodes.push(node_id);
    }

    // Check children
    for child_id in node.children() {
        collect_nodes_in_range(arena, child_id, start, end, nodes);
    }
}

/// Check if a selection is a valid expression.
pub fn is_expression_node(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Identifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BinaryExpression
            | SyntaxKind::CallExpression
            | SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::ConditionalExpression
            | SyntaxKind::TemplateExpression
            | SyntaxKind::ParenthesizedExpression
    )
}

/// Check if a selection is a statement.
pub fn is_statement_node(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::VariableStatement
            | SyntaxKind::ExpressionStatement
            | SyntaxKind::IfStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::ReturnStatement
            | SyntaxKind::ThrowStatement
            | SyntaxKind::TryStatement
            | SyntaxKind::SwitchStatement
            | SyntaxKind::Block
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_applicable_refactor_info() {
        let info = ApplicableRefactorInfo::new(
            "extractFunction".to_string(),
            "Extract function".to_string(),
        )
        .with_action(RefactorActionInfo::new(
            "extractFunction_global".to_string(),
            "Extract to function in global scope".to_string(),
        ));

        assert_eq!(info.name, "extractFunction");
        assert_eq!(info.actions.len(), 1);
    }

    #[test]
    fn test_refactor_action_info() {
        let action = RefactorActionInfo::new(
            "extractConstant".to_string(),
            "Extract to constant".to_string(),
        )
        .with_kind(refactor_kinds::EXTRACT_CONSTANT.to_string())
        .preferred();

        assert_eq!(action.name, "extractConstant");
        assert_eq!(action.is_preferred, Some(true));
    }

    #[test]
    fn test_refactor_edit_info() {
        let edits = RefactorEditInfo::new(Vec::new())
            .with_rename("newName".to_string(), 10);

        assert_eq!(edits.rename_file_name, Some("newName".to_string()));
        assert_eq!(edits.rename_location, Some(10));
    }

    #[test]
    fn test_is_expression_node() {
        assert!(is_expression_node(SyntaxKind::Identifier));
        assert!(is_expression_node(SyntaxKind::CallExpression));
        assert!(!is_expression_node(SyntaxKind::IfStatement));
    }

    #[test]
    fn test_is_statement_node() {
        assert!(is_statement_node(SyntaxKind::IfStatement));
        assert!(is_statement_node(SyntaxKind::ReturnStatement));
        assert!(!is_statement_node(SyntaxKind::Identifier));
    }
}
