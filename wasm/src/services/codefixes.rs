//! Individual Code Fix implementations.
//!
//! This module contains implementations of specific code fixes.
//! Each fix handles one or more error codes.

use crate::binder::{Symbol, SymbolFlags};
use crate::checker::CheckerState;
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::code_fix_provider::{
    CodeFixAction, CodeFixContext, CodeFixRegistry, DiagnosticCode,
    FileTextChanges, create_insertion, create_replacement, get_text_at_span,
};
use super::text_span::TextSpan;
use super::text_changes::TextChange;
use super::export_info_map::ExportInfoMap;

// =============================================================================
// Register All Fixes
// =============================================================================

/// Register all code fixes with the registry.
pub fn register_all_fixes(registry: &mut CodeFixRegistry) {
    // Import fixes
    registry.register(
        "fixMissingImport",
        &[DiagnosticCode::CANNOT_FIND_NAME, DiagnosticCode::CANNOT_FIND_MODULE],
        fix_missing_import,
    );

    // Property fixes
    registry.register(
        "fixMissingProperty",
        &[DiagnosticCode::PROPERTY_DOES_NOT_EXIST],
        fix_missing_property,
    );
    registry.register(
        "fixSpellingError",
        &[DiagnosticCode::PROPERTY_DOES_NOT_EXIST, DiagnosticCode::CANNOT_FIND_NAME],
        fix_spelling_error,
    );

    // Type fixes
    registry.register(
        "fixAddMissingMember",
        &[DiagnosticCode::MISSING_PROPERTY],
        fix_add_missing_member,
    );
    registry.register(
        "fixReturnType",
        &[DiagnosticCode::MISSING_RETURN_TYPE],
        fix_return_type,
    );
    registry.register(
        "fixImplicitAny",
        &[DiagnosticCode::IMPLICIT_ANY],
        fix_implicit_any,
    );

    // Unused code fixes
    registry.register(
        "fixUnusedVariable",
        &[DiagnosticCode::UNUSED_VARIABLE],
        fix_unused_variable,
    );
    registry.register(
        "fixUnusedParameter",
        &[DiagnosticCode::UNUSED_PARAMETER],
        fix_unused_parameter,
    );

    // Type conversion fixes
    registry.register(
        "fixAwaitInSyncFunction",
        &[DiagnosticCode(1308)], // Await only valid in async function
        fix_await_in_sync_function,
    );

    // Accessibility fixes
    registry.register(
        "fixAddMissingOverride",
        &[DiagnosticCode(4114)], // Must use override modifier
        fix_add_missing_override,
    );

    // Class fixes
    registry.register(
        "fixImplementInterface",
        &[DiagnosticCode(2420)], // Class incorrectly implements interface
        fix_implement_interface,
    );
    registry.register(
        "fixExtendAbstract",
        &[DiagnosticCode(2515)], // Non-abstract class must implement abstract method
        fix_extend_abstract,
    );

    // Strict mode fixes
    registry.register(
        "fixStrictClassInitialization",
        &[DiagnosticCode(2564)], // Property has no initializer
        fix_strict_class_initialization,
    );
}

// =============================================================================
// Import Fixes
// =============================================================================

fn fix_missing_import(ctx: &CodeFixContext) -> Vec<CodeFixAction> {
    let mut fixes = Vec::new();

    // Get the identifier that couldn't be found
    let identifier = get_text_at_span(ctx.source_text, &ctx.span);

    // In a full implementation, we would:
    // 1. Search the export map for matching exports
    // 2. Generate import statements for each match
    // 3. Consider different import styles (named, default, namespace)

    // For now, create a placeholder fix
    let import_statement = format!("import {{ {} }} from 'module';\n", identifier);

    let changes = vec![FileTextChanges::new(
        ctx.file_name.to_string(),
        vec![create_insertion(0, import_statement)],
    )];

    fixes.push(
        CodeFixAction::new(
            "fixMissingImport".to_string(),
            format!("Import '{}' from module", identifier),
            changes,
        )
        .with_fix_all(
            "fixMissingImport".to_string(),
            "Add all missing imports".to_string(),
        ),
    );

    fixes
}

// =============================================================================
// Property Fixes
// =============================================================================

fn fix_missing_property(ctx: &CodeFixContext) -> Vec<CodeFixAction> {
    let mut fixes = Vec::new();

    let property_name = get_text_at_span(ctx.source_text, &ctx.span);

    // In a full implementation, we would:
    // 1. Find the type being accessed
    // 2. Determine where to add the property
    // 3. Generate appropriate declaration

    fixes
}

fn fix_spelling_error(ctx: &CodeFixContext) -> Vec<CodeFixAction> {
    let mut fixes = Vec::new();

    let misspelled = get_text_at_span(ctx.source_text, &ctx.span);

    // In a full implementation, we would:
    // 1. Get available symbols in scope
    // 2. Find similar names using edit distance
    // 3. Suggest corrections

    // For demonstration, find symbols with similar names
    if let Some(suggestions) = find_spelling_suggestions(ctx, misspelled) {
        for suggestion in suggestions {
            let changes = vec![FileTextChanges::new(
                ctx.file_name.to_string(),
                vec![create_replacement(ctx.span.clone(), suggestion.clone())],
            )];

            fixes.push(CodeFixAction::new(
                "fixSpellingError".to_string(),
                format!("Change '{}' to '{}'", misspelled, suggestion),
                changes,
            ));
        }
    }

    fixes
}

fn find_spelling_suggestions(ctx: &CodeFixContext, name: &str) -> Option<Vec<String>> {
    // In a full implementation, this would:
    // 1. Get all symbols in scope
    // 2. Calculate edit distance to each
    // 3. Return the closest matches

    None
}

// =============================================================================
// Type Fixes
// =============================================================================

fn fix_add_missing_member(ctx: &CodeFixContext) -> Vec<CodeFixAction> {
    let mut fixes = Vec::new();

    // In a full implementation:
    // 1. Determine what member is missing
    // 2. Find the class/interface to add it to
    // 3. Generate appropriate member declaration

    fixes
}

fn fix_return_type(ctx: &CodeFixContext) -> Vec<CodeFixAction> {
    let mut fixes = Vec::new();

    // In a full implementation:
    // 1. Infer the return type from the function body
    // 2. Find the position to insert the type annotation
    // 3. Generate the fix

    fixes
}

fn fix_implicit_any(ctx: &CodeFixContext) -> Vec<CodeFixAction> {
    let mut fixes = Vec::new();

    // Find the node with implicit any
    let node_id = find_node_at_span(ctx.arena, &ctx.span);

    if let Some(id) = node_id {
        // Get the inferred type
        if let Some(type_id) = ctx.checker.get_type_at_location(id) {
            let type_string = ctx.checker.type_to_string(type_id);

            // Find position to insert type annotation
            let insert_pos = ctx.span.start + ctx.span.length;

            let changes = vec![FileTextChanges::new(
                ctx.file_name.to_string(),
                vec![create_insertion(insert_pos, format!(": {}", type_string))],
            )];

            fixes.push(
                CodeFixAction::new(
                    "fixImplicitAny".to_string(),
                    format!("Infer type '{}'", type_string),
                    changes,
                )
                .with_fix_all(
                    "fixImplicitAny".to_string(),
                    "Infer all implicit any types".to_string(),
                ),
            );
        }
    }

    fixes
}

// =============================================================================
// Unused Code Fixes
// =============================================================================

fn fix_unused_variable(ctx: &CodeFixContext) -> Vec<CodeFixAction> {
    let mut fixes = Vec::new();

    let var_name = get_text_at_span(ctx.source_text, &ctx.span);

    // Option 1: Prefix with underscore
    let prefixed_name = format!("_{}", var_name);
    let prefix_changes = vec![FileTextChanges::new(
        ctx.file_name.to_string(),
        vec![create_replacement(ctx.span.clone(), prefixed_name.clone())],
    )];

    fixes.push(CodeFixAction::new(
        "fixUnusedVariable".to_string(),
        format!("Prefix '{}' with '_'", var_name),
        prefix_changes,
    ));

    // Option 2: Remove the variable (if possible)
    // This would require more context to implement properly

    fixes
}

fn fix_unused_parameter(ctx: &CodeFixContext) -> Vec<CodeFixAction> {
    let mut fixes = Vec::new();

    let param_name = get_text_at_span(ctx.source_text, &ctx.span);

    // Prefix with underscore
    let prefixed_name = format!("_{}", param_name);
    let changes = vec![FileTextChanges::new(
        ctx.file_name.to_string(),
        vec![create_replacement(ctx.span.clone(), prefixed_name.clone())],
    )];

    fixes.push(
        CodeFixAction::new(
            "fixUnusedParameter".to_string(),
            format!("Prefix '{}' with '_'", param_name),
            changes,
        )
        .with_fix_all(
            "fixUnusedParameter".to_string(),
            "Prefix all unused parameters with '_'".to_string(),
        ),
    );

    fixes
}

// =============================================================================
// Async/Await Fixes
// =============================================================================

fn fix_await_in_sync_function(ctx: &CodeFixContext) -> Vec<CodeFixAction> {
    let mut fixes = Vec::new();

    // Find the containing function
    if let Some(func_id) = find_containing_function(ctx.arena, &ctx.span) {
        // Get position to insert 'async'
        if let Some(async_insert_pos) = get_async_keyword_position(ctx.arena, func_id) {
            let changes = vec![FileTextChanges::new(
                ctx.file_name.to_string(),
                vec![create_insertion(async_insert_pos, "async ".to_string())],
            )];

            fixes.push(CodeFixAction::new(
                "fixAwaitInSyncFunction".to_string(),
                "Add 'async' modifier to containing function".to_string(),
                changes,
            ));
        }
    }

    fixes
}

// =============================================================================
// Override Fixes
// =============================================================================

fn fix_add_missing_override(ctx: &CodeFixContext) -> Vec<CodeFixAction> {
    let mut fixes = Vec::new();

    // Find position to insert 'override'
    let changes = vec![FileTextChanges::new(
        ctx.file_name.to_string(),
        vec![create_insertion(ctx.span.start, "override ".to_string())],
    )];

    fixes.push(
        CodeFixAction::new(
            "fixAddMissingOverride".to_string(),
            "Add 'override' modifier".to_string(),
            changes,
        )
        .with_fix_all(
            "fixAddMissingOverride".to_string(),
            "Add all missing 'override' modifiers".to_string(),
        ),
    );

    fixes
}

// =============================================================================
// Class Fixes
// =============================================================================

fn fix_implement_interface(ctx: &CodeFixContext) -> Vec<CodeFixAction> {
    let mut fixes = Vec::new();

    // In a full implementation:
    // 1. Find the interface being implemented
    // 2. Get all members that need to be implemented
    // 3. Generate stubs for each member
    // 4. Insert at appropriate position in class

    fixes
}

fn fix_extend_abstract(ctx: &CodeFixContext) -> Vec<CodeFixAction> {
    let mut fixes = Vec::new();

    // Similar to fix_implement_interface but for abstract members

    fixes
}

fn fix_strict_class_initialization(ctx: &CodeFixContext) -> Vec<CodeFixAction> {
    let mut fixes = Vec::new();

    let property_name = get_text_at_span(ctx.source_text, &ctx.span);

    // Option 1: Add definite assignment assertion
    let assertion_changes = vec![FileTextChanges::new(
        ctx.file_name.to_string(),
        vec![create_insertion(ctx.span.start + ctx.span.length, "!".to_string())],
    )];

    fixes.push(CodeFixAction::new(
        "fixStrictClassInitialization".to_string(),
        "Add definite assignment assertion".to_string(),
        assertion_changes,
    ));

    // Option 2: Add undefined to the type
    // This would require finding the type annotation and modifying it

    fixes
}

// =============================================================================
// Helper Functions
// =============================================================================

fn find_node_at_span(arena: &NodeArena, span: &TextSpan) -> Option<NodeId> {
    let root = arena.root()?;
    find_node_at_span_recursive(arena, root, span)
}

fn find_node_at_span_recursive(
    arena: &NodeArena,
    node_id: NodeId,
    span: &TextSpan,
) -> Option<NodeId> {
    let node = arena.get(node_id)?;

    let node_start = node.pos();
    let node_end = node.end();

    if span.start >= node_start && span.start + span.length <= node_end {
        // Check children for more specific match
        for child_id in node.children() {
            if let Some(found) = find_node_at_span_recursive(arena, child_id, span) {
                return Some(found);
            }
        }
        return Some(node_id);
    }

    None
}

fn find_containing_function(arena: &NodeArena, span: &TextSpan) -> Option<NodeId> {
    let node_id = find_node_at_span(arena, span)?;

    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = arena.get(id)?;
        if is_function_like(node.kind()) {
            return Some(id);
        }
        current = node.parent();
    }

    None
}

fn is_function_like(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
    )
}

fn get_async_keyword_position(arena: &NodeArena, func_id: NodeId) -> Option<u32> {
    let func = arena.get(func_id)?;

    // In a full implementation, this would find the exact position
    // to insert 'async' based on existing modifiers

    Some(func.pos())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_all_fixes() {
        let mut registry = CodeFixRegistry::new();
        register_all_fixes(&mut registry);

        // Check that some fixes are registered
        assert!(registry.get_fix_by_name("fixMissingImport").is_some());
        assert!(registry.get_fix_by_name("fixUnusedVariable").is_some());
        assert!(registry.get_fix_by_name("fixSpellingError").is_some());
    }

    #[test]
    fn test_is_function_like() {
        assert!(is_function_like(SyntaxKind::FunctionDeclaration));
        assert!(is_function_like(SyntaxKind::ArrowFunction));
        assert!(is_function_like(SyntaxKind::MethodDeclaration));
        assert!(!is_function_like(SyntaxKind::ClassDeclaration));
        assert!(!is_function_like(SyntaxKind::Identifier));
    }
}
