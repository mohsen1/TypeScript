//! Individual Refactoring implementations.
//!
//! This module contains implementations of specific refactorings.

use crate::binder::{Symbol, SymbolFlags};
use crate::checker::CheckerState;
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::refactor_provider::{
    RefactorContext, RefactorRegistry, ApplicableRefactorInfo, RefactorActionInfo,
    RefactorEditInfo, refactor_kinds, find_node_at_position, find_nodes_in_range,
    is_expression_node, is_statement_node,
};
use super::code_fix_provider::FileTextChanges;
use super::text_span::TextSpan;
use super::text_changes::TextChange;

// =============================================================================
// Register All Refactors
// =============================================================================

/// Register all refactorings with the registry.
pub fn register_all_refactors(registry: &mut RefactorRegistry) {
    // Extract refactorings
    registry.register(
        "extractSymbol",
        "Extract to...",
        &[refactor_kinds::EXTRACT],
        check_extract_symbol,
        get_extract_symbol_edits,
    );

    // Inline refactorings
    registry.register(
        "inlineVariable",
        "Inline variable",
        &[refactor_kinds::INLINE_VARIABLE],
        check_inline_variable,
        get_inline_variable_edits,
    );

    // Convert refactorings
    registry.register(
        "convertFunction",
        "Convert function",
        &[refactor_kinds::REWRITE],
        check_convert_function,
        get_convert_function_edits,
    );

    registry.register(
        "convertToAsync",
        "Convert to async function",
        &[refactor_kinds::CONVERT_TO_ASYNC],
        check_convert_to_async,
        get_convert_to_async_edits,
    );

    // Move refactorings
    registry.register(
        "moveToNewFile",
        "Move to a new file",
        &[refactor_kinds::MOVE_TO_NEW_FILE],
        check_move_to_new_file,
        get_move_to_new_file_edits,
    );

    // Generate refactorings
    registry.register(
        "generateGetterSetter",
        "Generate getter/setter",
        &[refactor_kinds::REWRITE],
        check_generate_getter_setter,
        get_generate_getter_setter_edits,
    );

    // Convert string
    registry.register(
        "convertStringConcatenation",
        "Convert to template literal",
        &[refactor_kinds::REWRITE],
        check_convert_string,
        get_convert_string_edits,
    );

    // Add/remove braces
    registry.register(
        "addOrRemoveBraces",
        "Add/remove braces from arrow function",
        &[refactor_kinds::REWRITE],
        check_add_remove_braces,
        get_add_remove_braces_edits,
    );
}

// =============================================================================
// Extract Symbol Refactoring
// =============================================================================

fn check_extract_symbol(ctx: &RefactorContext) -> Option<ApplicableRefactorInfo> {
    let node_id = find_node_at_position(ctx.arena, ctx.start_position)?;
    let node = ctx.arena.get(node_id)?;

    // Check if we have an expression or statements selected
    if !is_expression_node(node.kind()) && !ctx.is_range_selection() {
        return None;
    }

    let mut info = ApplicableRefactorInfo::new(
        "extractSymbol".to_string(),
        "Extract to...".to_string(),
    );

    // Add actions based on context
    info.actions.push(
        RefactorActionInfo::new(
            "extractConstant_local".to_string(),
            "Extract to constant in enclosing scope".to_string(),
        )
        .with_kind(refactor_kinds::EXTRACT_CONSTANT.to_string()),
    );

    info.actions.push(
        RefactorActionInfo::new(
            "extractFunction_local".to_string(),
            "Extract to function in enclosing scope".to_string(),
        )
        .with_kind(refactor_kinds::EXTRACT_FUNCTION.to_string()),
    );

    // Check if in module scope
    if is_module_scope(ctx.arena, node_id) {
        info.actions.push(
            RefactorActionInfo::new(
                "extractFunction_global".to_string(),
                "Extract to function in module scope".to_string(),
            )
            .with_kind(refactor_kinds::EXTRACT_FUNCTION.to_string()),
        );
    }

    // Check if type extraction is applicable
    if is_type_context(ctx.arena, node_id) {
        info.actions.push(
            RefactorActionInfo::new(
                "extractType".to_string(),
                "Extract to type alias".to_string(),
            )
            .with_kind(refactor_kinds::EXTRACT_TYPE.to_string()),
        );
    }

    Some(info)
}

fn get_extract_symbol_edits(
    ctx: &RefactorContext,
    action_name: &str,
) -> Option<RefactorEditInfo> {
    let node_id = find_node_at_position(ctx.arena, ctx.start_position)?;
    let node = ctx.arena.get(node_id)?;

    let span = TextSpan::from_bounds(node.pos(), node.end());
    let expression_text = get_text_at_span(ctx.source_text, &span);

    match action_name {
        "extractConstant_local" => {
            extract_to_constant(ctx, node_id, &expression_text)
        }
        "extractFunction_local" | "extractFunction_global" => {
            extract_to_function(ctx, node_id, &expression_text, action_name.contains("global"))
        }
        "extractType" => {
            extract_to_type(ctx, node_id, &expression_text)
        }
        _ => None,
    }
}

fn extract_to_constant(
    ctx: &RefactorContext,
    node_id: NodeId,
    expression: &str,
) -> Option<RefactorEditInfo> {
    let node = ctx.arena.get(node_id)?;
    let span = TextSpan::from_bounds(node.pos(), node.end());

    // Find the enclosing statement to insert before
    let insert_pos = find_statement_start(ctx.arena, node_id)?;

    // Generate the constant declaration
    let const_decl = format!("const newLocal = {};\n", expression);

    let changes = vec![
        // Insert constant declaration
        TextChange::new(TextSpan::new(insert_pos, 0), const_decl),
        // Replace expression with variable reference
        TextChange::new(span, "newLocal".to_string()),
    ];

    Some(RefactorEditInfo::new(vec![FileTextChanges::new(
        ctx.file_name.to_string(),
        changes,
    )]).with_rename(ctx.file_name.to_string(), insert_pos + 6)) // Position of "newLocal"
}

fn extract_to_function(
    ctx: &RefactorContext,
    node_id: NodeId,
    expression: &str,
    global: bool,
) -> Option<RefactorEditInfo> {
    let node = ctx.arena.get(node_id)?;
    let span = TextSpan::from_bounds(node.pos(), node.end());

    // Analyze free variables in the expression
    let free_vars = analyze_free_variables(ctx, node_id);

    // Generate parameter list
    let params = if free_vars.is_empty() {
        String::new()
    } else {
        free_vars.join(", ")
    };

    // Generate function call
    let call = if free_vars.is_empty() {
        "newFunction()".to_string()
    } else {
        format!("newFunction({})", params)
    };

    // Generate function declaration
    let func_decl = format!(
        "\nfunction newFunction({}) {{\n    return {};\n}}\n",
        params, expression
    );

    // Find insert position
    let insert_pos = if global {
        find_module_scope_position(ctx.arena)?
    } else {
        find_function_scope_position(ctx.arena, node_id)?
    };

    let changes = vec![
        // Replace expression with function call
        TextChange::new(span, call),
        // Insert function declaration
        TextChange::new(TextSpan::new(insert_pos, 0), func_decl),
    ];

    Some(RefactorEditInfo::new(vec![FileTextChanges::new(
        ctx.file_name.to_string(),
        changes,
    )]).with_rename(ctx.file_name.to_string(), insert_pos + 10)) // Position of "newFunction"
}

fn extract_to_type(
    ctx: &RefactorContext,
    node_id: NodeId,
    type_text: &str,
) -> Option<RefactorEditInfo> {
    let node = ctx.arena.get(node_id)?;
    let span = TextSpan::from_bounds(node.pos(), node.end());

    // Find position to insert type alias
    let insert_pos = find_type_insert_position(ctx.arena, node_id)?;

    // Generate type alias
    let type_decl = format!("type NewType = {};\n", type_text);

    let changes = vec![
        // Insert type alias
        TextChange::new(TextSpan::new(insert_pos, 0), type_decl),
        // Replace type with alias reference
        TextChange::new(span, "NewType".to_string()),
    ];

    Some(RefactorEditInfo::new(vec![FileTextChanges::new(
        ctx.file_name.to_string(),
        changes,
    )]).with_rename(ctx.file_name.to_string(), insert_pos + 5)) // Position of "NewType"
}

// =============================================================================
// Inline Variable Refactoring
// =============================================================================

fn check_inline_variable(ctx: &RefactorContext) -> Option<ApplicableRefactorInfo> {
    let node_id = find_node_at_position(ctx.arena, ctx.start_position)?;
    let node = ctx.arena.get(node_id)?;

    // Must be an identifier
    if node.kind() != SyntaxKind::Identifier {
        return None;
    }

    // Check if it's a variable declaration or reference
    let symbol = ctx.checker.get_symbol_at_location(node_id)?;

    // Check if the variable has a single assignment
    if !can_inline_variable(ctx, &symbol) {
        return None;
    }

    let mut info = ApplicableRefactorInfo::new(
        "inlineVariable".to_string(),
        "Inline variable".to_string(),
    );

    info.actions.push(
        RefactorActionInfo::new(
            "inlineVariable".to_string(),
            format!("Inline '{}'", symbol.escaped_name),
        )
        .with_kind(refactor_kinds::INLINE_VARIABLE.to_string()),
    );

    Some(info)
}

fn get_inline_variable_edits(
    ctx: &RefactorContext,
    _action_name: &str,
) -> Option<RefactorEditInfo> {
    let node_id = find_node_at_position(ctx.arena, ctx.start_position)?;
    let symbol = ctx.checker.get_symbol_at_location(node_id)?;

    // Get the initializer value
    let initializer = get_variable_initializer(ctx, &symbol)?;

    // Find all references to inline
    let references = find_all_references(ctx, &symbol);

    let mut changes = Vec::new();

    // Replace each reference with the initializer
    for ref_id in references {
        let ref_node = ctx.arena.get(ref_id)?;
        let span = TextSpan::from_bounds(ref_node.pos(), ref_node.end());
        changes.push(TextChange::new(span, initializer.clone()));
    }

    // Remove the variable declaration
    if let Some(decl_span) = get_declaration_span(ctx, &symbol) {
        changes.push(TextChange::new(decl_span, String::new()));
    }

    Some(RefactorEditInfo::new(vec![FileTextChanges::new(
        ctx.file_name.to_string(),
        changes,
    )]))
}

// =============================================================================
// Convert Function Refactoring
// =============================================================================

fn check_convert_function(ctx: &RefactorContext) -> Option<ApplicableRefactorInfo> {
    let node_id = find_node_at_position(ctx.arena, ctx.start_position)?;
    let node = ctx.arena.get(node_id)?;

    let mut info = ApplicableRefactorInfo::new(
        "convertFunction".to_string(),
        "Convert function".to_string(),
    );

    match node.kind() {
        SyntaxKind::FunctionDeclaration | SyntaxKind::FunctionExpression => {
            info.actions.push(
                RefactorActionInfo::new(
                    "convertToArrow".to_string(),
                    "Convert to arrow function".to_string(),
                )
                .with_kind(refactor_kinds::CONVERT_TO_ARROW.to_string()),
            );
        }
        SyntaxKind::ArrowFunction => {
            info.actions.push(
                RefactorActionInfo::new(
                    "convertToNamedFunction".to_string(),
                    "Convert to named function".to_string(),
                )
                .with_kind(refactor_kinds::CONVERT_TO_FUNCTION.to_string()),
            );
        }
        _ => return None,
    }

    Some(info)
}

fn get_convert_function_edits(
    ctx: &RefactorContext,
    action_name: &str,
) -> Option<RefactorEditInfo> {
    let node_id = find_node_at_position(ctx.arena, ctx.start_position)?;

    match action_name {
        "convertToArrow" => convert_to_arrow_function(ctx, node_id),
        "convertToNamedFunction" => convert_to_named_function(ctx, node_id),
        _ => None,
    }
}

fn convert_to_arrow_function(
    ctx: &RefactorContext,
    func_id: NodeId,
) -> Option<RefactorEditInfo> {
    let func = ctx.arena.get(func_id)?;
    let span = TextSpan::from_bounds(func.pos(), func.end());

    // In a full implementation, we would:
    // 1. Parse the function components
    // 2. Check for 'this' usage
    // 3. Generate arrow function syntax

    // Placeholder implementation
    None
}

fn convert_to_named_function(
    ctx: &RefactorContext,
    arrow_id: NodeId,
) -> Option<RefactorEditInfo> {
    let arrow = ctx.arena.get(arrow_id)?;
    let span = TextSpan::from_bounds(arrow.pos(), arrow.end());

    // In a full implementation, we would convert arrow to function declaration
    None
}

// =============================================================================
// Convert to Async Refactoring
// =============================================================================

fn check_convert_to_async(ctx: &RefactorContext) -> Option<ApplicableRefactorInfo> {
    let node_id = find_node_at_position(ctx.arena, ctx.start_position)?;
    let node = ctx.arena.get(node_id)?;

    // Check if it's a function that returns a Promise
    if !is_function_returning_promise(ctx, node_id) {
        return None;
    }

    let mut info = ApplicableRefactorInfo::new(
        "convertToAsync".to_string(),
        "Convert to async function".to_string(),
    );

    info.actions.push(
        RefactorActionInfo::new(
            "convertToAsyncFunction".to_string(),
            "Convert to async/await".to_string(),
        )
        .with_kind(refactor_kinds::CONVERT_TO_ASYNC.to_string())
        .preferred(),
    );

    Some(info)
}

fn get_convert_to_async_edits(
    ctx: &RefactorContext,
    _action_name: &str,
) -> Option<RefactorEditInfo> {
    // In a full implementation:
    // 1. Add 'async' keyword
    // 2. Convert .then() chains to await
    // 3. Convert .catch() to try/catch
    None
}

// =============================================================================
// Move to New File Refactoring
// =============================================================================

fn check_move_to_new_file(ctx: &RefactorContext) -> Option<ApplicableRefactorInfo> {
    let node_id = find_node_at_position(ctx.arena, ctx.start_position)?;
    let node = ctx.arena.get(node_id)?;

    // Check if it's a top-level declaration
    if !is_top_level_declaration(ctx.arena, node_id) {
        return None;
    }

    let mut info = ApplicableRefactorInfo::new(
        "moveToNewFile".to_string(),
        "Move to a new file".to_string(),
    );

    info.actions.push(
        RefactorActionInfo::new(
            "moveToNewFile".to_string(),
            "Move to a new file".to_string(),
        )
        .with_kind(refactor_kinds::MOVE_TO_NEW_FILE.to_string()),
    );

    Some(info)
}

fn get_move_to_new_file_edits(
    ctx: &RefactorContext,
    _action_name: &str,
) -> Option<RefactorEditInfo> {
    // In a full implementation:
    // 1. Extract the declaration
    // 2. Create new file with the declaration
    // 3. Add import in original file
    // 4. Update all references
    None
}

// =============================================================================
// Generate Getter/Setter Refactoring
// =============================================================================

fn check_generate_getter_setter(ctx: &RefactorContext) -> Option<ApplicableRefactorInfo> {
    let node_id = find_node_at_position(ctx.arena, ctx.start_position)?;
    let node = ctx.arena.get(node_id)?;

    // Must be a property declaration
    if node.kind() != SyntaxKind::PropertyDeclaration {
        return None;
    }

    let mut info = ApplicableRefactorInfo::new(
        "generateGetterSetter".to_string(),
        "Generate getter/setter".to_string(),
    );

    info.actions.push(RefactorActionInfo::new(
        "generateGetter".to_string(),
        "Generate getter".to_string(),
    ));

    info.actions.push(RefactorActionInfo::new(
        "generateSetter".to_string(),
        "Generate setter".to_string(),
    ));

    info.actions.push(RefactorActionInfo::new(
        "generateGetterAndSetter".to_string(),
        "Generate getter and setter".to_string(),
    ));

    Some(info)
}

fn get_generate_getter_setter_edits(
    ctx: &RefactorContext,
    action_name: &str,
) -> Option<RefactorEditInfo> {
    // In a full implementation:
    // 1. Make property private
    // 2. Generate getter/setter methods
    None
}

// =============================================================================
// Convert String Refactoring
// =============================================================================

fn check_convert_string(ctx: &RefactorContext) -> Option<ApplicableRefactorInfo> {
    let node_id = find_node_at_position(ctx.arena, ctx.start_position)?;
    let node = ctx.arena.get(node_id)?;

    // Check for string concatenation
    if node.kind() != SyntaxKind::BinaryExpression {
        return None;
    }

    // Check if it's a string concatenation
    if !is_string_concatenation(ctx.arena, node_id) {
        return None;
    }

    let mut info = ApplicableRefactorInfo::new(
        "convertStringConcatenation".to_string(),
        "Convert to template literal".to_string(),
    );

    info.actions.push(RefactorActionInfo::new(
        "convertToTemplateLiteral".to_string(),
        "Convert to template literal".to_string(),
    ));

    Some(info)
}

fn get_convert_string_edits(
    ctx: &RefactorContext,
    _action_name: &str,
) -> Option<RefactorEditInfo> {
    // In a full implementation:
    // 1. Parse the concatenation
    // 2. Convert to template literal with interpolations
    None
}

// =============================================================================
// Add/Remove Braces Refactoring
// =============================================================================

fn check_add_remove_braces(ctx: &RefactorContext) -> Option<ApplicableRefactorInfo> {
    let node_id = find_node_at_position(ctx.arena, ctx.start_position)?;
    let node = ctx.arena.get(node_id)?;

    // Must be an arrow function
    if node.kind() != SyntaxKind::ArrowFunction {
        return None;
    }

    let mut info = ApplicableRefactorInfo::new(
        "addOrRemoveBraces".to_string(),
        "Add/remove braces from arrow function".to_string(),
    );

    // Check current state
    if has_braces(ctx.arena, node_id) {
        info.actions.push(RefactorActionInfo::new(
            "removeBraces".to_string(),
            "Remove braces from arrow function".to_string(),
        ));
    } else {
        info.actions.push(RefactorActionInfo::new(
            "addBraces".to_string(),
            "Add braces to arrow function".to_string(),
        ));
    }

    Some(info)
}

fn get_add_remove_braces_edits(
    ctx: &RefactorContext,
    action_name: &str,
) -> Option<RefactorEditInfo> {
    // In a full implementation:
    // 1. Parse arrow function body
    // 2. Add/remove braces and return statement
    None
}

// =============================================================================
// Helper Functions
// =============================================================================

fn get_text_at_span(source: &str, span: &TextSpan) -> String {
    let start = span.start as usize;
    let end = start + span.length as usize;

    if end <= source.len() {
        source[start..end].to_string()
    } else {
        String::new()
    }
}

fn is_module_scope(arena: &NodeArena, node_id: NodeId) -> bool {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = match arena.get(id) {
            Some(n) => n,
            None => return false,
        };

        if matches!(
            node.kind(),
            SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::ClassDeclaration
        ) {
            return false;
        }

        if node.kind() == SyntaxKind::SourceFile {
            return true;
        }

        current = node.parent();
    }

    false
}

fn is_type_context(arena: &NodeArena, node_id: NodeId) -> bool {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = match arena.get(id) {
            Some(n) => n,
            None => return false,
        };

        if matches!(
            node.kind(),
            SyntaxKind::TypeReference
                | SyntaxKind::TypeLiteral
                | SyntaxKind::UnionType
                | SyntaxKind::IntersectionType
        ) {
            return true;
        }

        current = node.parent();
    }

    false
}

fn find_statement_start(arena: &NodeArena, node_id: NodeId) -> Option<u32> {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = arena.get(id)?;

        if is_statement_node(node.kind()) {
            return Some(node.pos());
        }

        current = node.parent();
    }

    None
}

fn find_module_scope_position(arena: &NodeArena) -> Option<u32> {
    let root = arena.root()?;
    let source_file = arena.get(root)?;
    Some(source_file.end())
}

fn find_function_scope_position(arena: &NodeArena, node_id: NodeId) -> Option<u32> {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = arena.get(id)?;

        if matches!(
            node.kind(),
            SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
        ) {
            return Some(node.end());
        }

        current = node.parent();
    }

    find_module_scope_position(arena)
}

fn find_type_insert_position(arena: &NodeArena, node_id: NodeId) -> Option<u32> {
    find_statement_start(arena, node_id)
}

fn analyze_free_variables(ctx: &RefactorContext, node_id: NodeId) -> Vec<String> {
    // In a full implementation, analyze the expression to find
    // variables that need to be passed as parameters
    Vec::new()
}

fn can_inline_variable(ctx: &RefactorContext, symbol: &Symbol) -> bool {
    // Check if variable can be inlined (single assignment, etc.)
    true
}

fn get_variable_initializer(ctx: &RefactorContext, symbol: &Symbol) -> Option<String> {
    None
}

fn find_all_references(ctx: &RefactorContext, symbol: &Symbol) -> Vec<NodeId> {
    Vec::new()
}

fn get_declaration_span(ctx: &RefactorContext, symbol: &Symbol) -> Option<TextSpan> {
    None
}

fn is_function_returning_promise(ctx: &RefactorContext, node_id: NodeId) -> bool {
    false
}

fn is_top_level_declaration(arena: &NodeArena, node_id: NodeId) -> bool {
    let node = match arena.get(node_id) {
        Some(n) => n,
        None => return false,
    };

    let parent_id = match node.parent() {
        Some(p) => p,
        None => return false,
    };

    let parent = match arena.get(parent_id) {
        Some(p) => p,
        None => return false,
    };

    parent.kind() == SyntaxKind::SourceFile
}

fn is_string_concatenation(arena: &NodeArena, node_id: NodeId) -> bool {
    // Check if binary expression is string concatenation
    false
}

fn has_braces(arena: &NodeArena, arrow_id: NodeId) -> bool {
    // Check if arrow function has braces
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_all_refactors() {
        let mut registry = RefactorRegistry::new();
        register_all_refactors(&mut registry);

        // The registry should now have refactors registered
        // (We can't easily test this without reflection)
    }

    #[test]
    fn test_is_module_scope() {
        // Would need a real arena to test
    }
}
