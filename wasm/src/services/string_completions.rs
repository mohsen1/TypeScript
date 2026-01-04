//! String Literal Completions implementation.
//!
//! This module provides completions within string literals, including:
//! - Module specifier completions (import paths)
//! - Property name completions
//! - Object key completions

use crate::binder::{Symbol, SymbolId, SymbolFlags};
use crate::checker::{Type, TypeId, CheckerState};
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::completions::{CompletionInfo, CompletionEntry, CompletionsContext};
use super::text_span::TextSpan;
use super::utilities::ScriptElementKind;

// =============================================================================
// String Literal Completion Types
// =============================================================================

/// Context for string literal completions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringLiteralCompletionKind {
    /// Completing a module specifier (import/require path).
    ModuleSpecifier,
    /// Completing a property name access via bracket notation.
    PropertyName,
    /// Completing an object literal key.
    ObjectLiteralKey,
    /// Completing a type literal property.
    TypeLiteralProperty,
    /// Completing a discriminant in a discriminated union.
    Discriminant,
    /// Completing a path in a triple-slash reference.
    TripleSlashReference,
    /// Unknown string completion context.
    Unknown,
}

// =============================================================================
// Get String Literal Completions
// =============================================================================

/// Get completions for string literals.
pub fn get_string_literal_completions(
    ctx: &CompletionsContext,
    node_id: NodeId,
) -> Option<CompletionInfo> {
    let node = ctx.arena.get(node_id)?;

    // Determine the kind of string completion
    let completion_kind = determine_string_completion_kind(ctx.arena, node_id);

    match completion_kind {
        StringLiteralCompletionKind::ModuleSpecifier => {
            get_module_specifier_completions(ctx, node_id)
        }
        StringLiteralCompletionKind::PropertyName => {
            get_property_name_completions(ctx, node_id)
        }
        StringLiteralCompletionKind::ObjectLiteralKey => {
            get_object_literal_key_completions(ctx, node_id)
        }
        StringLiteralCompletionKind::TypeLiteralProperty => {
            get_type_literal_property_completions(ctx, node_id)
        }
        StringLiteralCompletionKind::Discriminant => {
            get_discriminant_completions(ctx, node_id)
        }
        StringLiteralCompletionKind::TripleSlashReference => {
            get_triple_slash_reference_completions(ctx, node_id)
        }
        StringLiteralCompletionKind::Unknown => None,
    }
}

// =============================================================================
// Determine String Completion Kind
// =============================================================================

fn determine_string_completion_kind(
    arena: &NodeArena,
    node_id: NodeId,
) -> StringLiteralCompletionKind {
    let node = match arena.get(node_id) {
        Some(n) => n,
        None => return StringLiteralCompletionKind::Unknown,
    };

    // Check the parent to determine context
    let parent_id = match node.parent() {
        Some(p) => p,
        None => return StringLiteralCompletionKind::Unknown,
    };

    let parent = match arena.get(parent_id) {
        Some(p) => p,
        None => return StringLiteralCompletionKind::Unknown,
    };

    match parent.kind() {
        // Import/require statements
        SyntaxKind::ImportDeclaration
        | SyntaxKind::ExportDeclaration
        | SyntaxKind::CallExpression => {
            // Check if this is a require() call
            if parent.kind() == SyntaxKind::CallExpression {
                if is_require_call(arena, parent_id) {
                    return StringLiteralCompletionKind::ModuleSpecifier;
                }
            } else {
                return StringLiteralCompletionKind::ModuleSpecifier;
            }
        }

        // Element access expression: obj["prop"]
        SyntaxKind::ElementAccessExpression => {
            return StringLiteralCompletionKind::PropertyName;
        }

        // Property assignment in object literal: { "key": value }
        SyntaxKind::PropertyAssignment => {
            return StringLiteralCompletionKind::ObjectLiteralKey;
        }

        // Type literal member: { "key": type }
        SyntaxKind::PropertySignature => {
            return StringLiteralCompletionKind::TypeLiteralProperty;
        }

        _ => {}
    }

    // Check for discriminant context
    if is_discriminant_context(arena, node_id) {
        return StringLiteralCompletionKind::Discriminant;
    }

    StringLiteralCompletionKind::Unknown
}

// =============================================================================
// Module Specifier Completions
// =============================================================================

fn get_module_specifier_completions(
    ctx: &CompletionsContext,
    node_id: NodeId,
) -> Option<CompletionInfo> {
    let node = ctx.arena.get(node_id)?;

    // Get the current text in the string
    let string_text = get_string_text(ctx.source_text, node.pos(), node.end());

    let mut entries = Vec::new();

    // Check if this is a relative path
    if string_text.starts_with('.') || string_text.starts_with('/') {
        // Get file system completions
        let path_completions = get_path_completions(ctx, &string_text);
        entries.extend(path_completions);
    } else {
        // Get node_modules completions
        let module_completions = get_node_modules_completions(ctx, &string_text);
        entries.extend(module_completions);
    }

    if entries.is_empty() {
        None
    } else {
        let info = CompletionInfo::new(entries);
        Some(info)
    }
}

fn get_path_completions(
    ctx: &CompletionsContext,
    current_path: &str,
) -> Vec<CompletionEntry> {
    let mut entries = Vec::new();

    // In a full implementation, this would:
    // 1. Resolve the current directory
    // 2. List files and directories
    // 3. Filter based on the current input
    // 4. Consider tsconfig path mappings

    // For now, return empty
    entries
}

fn get_node_modules_completions(
    ctx: &CompletionsContext,
    prefix: &str,
) -> Vec<CompletionEntry> {
    let mut entries = Vec::new();

    // In a full implementation, this would:
    // 1. Scan node_modules directories
    // 2. Look at package.json files
    // 3. Consider scoped packages (@org/package)
    // 4. Filter based on prefix

    entries
}

// =============================================================================
// Property Name Completions
// =============================================================================

fn get_property_name_completions(
    ctx: &CompletionsContext,
    node_id: NodeId,
) -> Option<CompletionInfo> {
    let node = ctx.arena.get(node_id)?;

    // Find the element access expression
    let parent_id = node.parent()?;
    let element_access = ctx.arena.get(parent_id)?;

    // Get the expression being accessed
    let expression_id = get_element_access_expression(ctx.arena, parent_id)?;

    // Get the type of the expression
    let type_id = ctx.checker.get_type_at_location(expression_id)?;

    // Get string index type or property names
    let mut entries = Vec::new();

    // Get properties of the type
    let properties = ctx.checker.get_properties_of_type(type_id);

    for prop in properties {
        let entry = CompletionEntry::new(
            prop.escaped_name.clone(),
            ScriptElementKind::MemberVariableElement,
        );
        entries.push(entry);
    }

    // Also add index signatures if applicable
    if let Some(string_index_type) = ctx.checker.get_index_type_of_type(type_id, false) {
        // The type has a string index signature
        // In a full implementation, we might show this differently
    }

    if entries.is_empty() {
        None
    } else {
        Some(CompletionInfo::new(entries))
    }
}

// =============================================================================
// Object Literal Key Completions
// =============================================================================

fn get_object_literal_key_completions(
    ctx: &CompletionsContext,
    node_id: NodeId,
) -> Option<CompletionInfo> {
    let node = ctx.arena.get(node_id)?;

    // Find the object literal
    let object_literal_id = find_containing_object_literal(ctx.arena, node_id)?;

    // Get the contextual type for the object literal
    let contextual_type = ctx.checker.get_contextual_type(object_literal_id)?;

    // Get properties that haven't been defined yet
    let mut entries = Vec::new();
    let existing_keys = get_existing_object_keys(ctx.arena, object_literal_id);

    let properties = ctx.checker.get_properties_of_type(contextual_type);

    for prop in properties {
        if !existing_keys.contains(&prop.escaped_name) {
            let entry = CompletionEntry::new(
                prop.escaped_name.clone(),
                ScriptElementKind::MemberVariableElement,
            );
            entries.push(entry);
        }
    }

    if entries.is_empty() {
        None
    } else {
        Some(CompletionInfo::new(entries))
    }
}

// =============================================================================
// Type Literal Property Completions
// =============================================================================

fn get_type_literal_property_completions(
    ctx: &CompletionsContext,
    node_id: NodeId,
) -> Option<CompletionInfo> {
    // Similar to object literal key completions but for type literals
    None
}

// =============================================================================
// Discriminant Completions
// =============================================================================

fn get_discriminant_completions(
    ctx: &CompletionsContext,
    node_id: NodeId,
) -> Option<CompletionInfo> {
    let node = ctx.arena.get(node_id)?;

    // Find the discriminant property
    let parent_id = node.parent()?;

    // Get the contextual type which should be a union
    let contextual_type = ctx.checker.get_contextual_type(parent_id)?;

    // Get the discriminant values from the union
    let discriminant_values = get_discriminant_values(ctx.checker, contextual_type);

    let mut entries = Vec::new();

    for value in discriminant_values {
        let entry = CompletionEntry::new(
            value,
            ScriptElementKind::StringElement,
        );
        entries.push(entry);
    }

    if entries.is_empty() {
        None
    } else {
        Some(CompletionInfo::new(entries))
    }
}

fn get_discriminant_values(
    checker: &CheckerState,
    type_id: TypeId,
) -> Vec<String> {
    let mut values = Vec::new();

    // In a full implementation, this would:
    // 1. Check if the type is a union
    // 2. Find common discriminant properties
    // 3. Extract the literal values

    values
}

// =============================================================================
// Triple Slash Reference Completions
// =============================================================================

fn get_triple_slash_reference_completions(
    ctx: &CompletionsContext,
    node_id: NodeId,
) -> Option<CompletionInfo> {
    // Completions for /// <reference path="..." /> and similar
    None
}

// =============================================================================
// Helper Functions
// =============================================================================

fn is_require_call(arena: &NodeArena, call_id: NodeId) -> bool {
    let call = match arena.get(call_id) {
        Some(n) => n,
        None => return false,
    };

    // In a full implementation, check if the expression is 'require'
    false
}

fn is_discriminant_context(arena: &NodeArena, node_id: NodeId) -> bool {
    // Check if we're in a context where discriminant values are expected
    // (e.g., switch case on a discriminated union)
    false
}

fn get_string_text(source: &str, start: u32, end: u32) -> String {
    let start = start as usize;
    let end = end as usize;

    if start >= source.len() || end > source.len() || start >= end {
        return String::new();
    }

    let text = &source[start..end];

    // Remove quotes
    let text = text.trim_start_matches(|c| c == '"' || c == '\'' || c == '`');
    let text = text.trim_end_matches(|c| c == '"' || c == '\'' || c == '`');

    text.to_string()
}

fn get_element_access_expression(
    arena: &NodeArena,
    element_access_id: NodeId,
) -> Option<NodeId> {
    // Get the expression part of obj["prop"]
    // In a full implementation, this would navigate to the 'obj' part
    None
}

fn find_containing_object_literal(
    arena: &NodeArena,
    node_id: NodeId,
) -> Option<NodeId> {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = arena.get(id)?;
        if node.kind() == SyntaxKind::ObjectLiteralExpression {
            return Some(id);
        }
        current = node.parent();
    }
    None
}

fn get_existing_object_keys(
    arena: &NodeArena,
    object_literal_id: NodeId,
) -> Vec<String> {
    let mut keys = Vec::new();

    // In a full implementation, walk the object literal properties
    // and collect existing key names

    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_string_text() {
        assert_eq!(get_string_text("\"hello\"", 0, 7), "hello");
        assert_eq!(get_string_text("'world'", 0, 7), "world");
        assert_eq!(get_string_text("`test`", 0, 6), "test");
    }

    #[test]
    fn test_string_literal_completion_kind() {
        // These would require a real arena to test properly
        // Just testing that the enum values exist
        assert_eq!(
            StringLiteralCompletionKind::ModuleSpecifier,
            StringLiteralCompletionKind::ModuleSpecifier
        );
        assert_ne!(
            StringLiteralCompletionKind::ModuleSpecifier,
            StringLiteralCompletionKind::PropertyName
        );
    }
}
