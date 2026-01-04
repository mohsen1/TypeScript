//! Signature Help implementation.
//!
//! This module provides signature help (parameter hints) for function calls,
//! constructor invocations, and other callable expressions.

use crate::binder::{Symbol, SymbolId, SymbolFlags};
use crate::checker::{Type, TypeId, CheckerState, Signature};
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::symbol_display::SymbolDisplayPart;
use super::text_span::TextSpan;

// =============================================================================
// Signature Help Types
// =============================================================================

/// Complete signature help information.
#[derive(Debug, Clone)]
pub struct SignatureHelpItems {
    /// The available signatures.
    pub items: Vec<SignatureHelpItem>,
    /// The span that the signature help applies to.
    pub applicable_span: TextSpan,
    /// The currently selected signature.
    pub selected_item_index: usize,
    /// The current argument index.
    pub argument_index: usize,
    /// The current argument count.
    pub argument_count: usize,
}

impl SignatureHelpItems {
    pub fn new(
        items: Vec<SignatureHelpItem>,
        applicable_span: TextSpan,
        selected_item_index: usize,
        argument_index: usize,
        argument_count: usize,
    ) -> Self {
        Self {
            items,
            applicable_span,
            selected_item_index,
            argument_index,
            argument_count,
        }
    }

    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            applicable_span: TextSpan::new(0, 0),
            selected_item_index: 0,
            argument_index: 0,
            argument_count: 0,
        }
    }
}

/// A single signature in the signature help.
#[derive(Debug, Clone)]
pub struct SignatureHelpItem {
    /// Whether this is a variadic signature.
    pub is_variadic: bool,
    /// The prefix of the signature (e.g., "function foo(").
    pub prefix_display_parts: Vec<SymbolDisplayPart>,
    /// The suffix of the signature (e.g., "): void").
    pub suffix_display_parts: Vec<SymbolDisplayPart>,
    /// The separator between parameters (e.g., ", ").
    pub separator_display_parts: Vec<SymbolDisplayPart>,
    /// The parameters.
    pub parameters: Vec<SignatureHelpParameter>,
    /// Documentation for this signature.
    pub documentation: Vec<SymbolDisplayPart>,
    /// JSDoc tags for this signature.
    pub tags: Vec<JSDocTagInfo>,
}

impl SignatureHelpItem {
    pub fn new() -> Self {
        Self {
            is_variadic: false,
            prefix_display_parts: Vec::new(),
            suffix_display_parts: Vec::new(),
            separator_display_parts: vec![
                SymbolDisplayPart::punctuation(","),
                SymbolDisplayPart::space(),
            ],
            parameters: Vec::new(),
            documentation: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_prefix(mut self, parts: Vec<SymbolDisplayPart>) -> Self {
        self.prefix_display_parts = parts;
        self
    }

    pub fn with_suffix(mut self, parts: Vec<SymbolDisplayPart>) -> Self {
        self.suffix_display_parts = parts;
        self
    }

    pub fn add_parameter(&mut self, param: SignatureHelpParameter) {
        self.parameters.push(param);
    }
}

impl Default for SignatureHelpItem {
    fn default() -> Self {
        Self::new()
    }
}

/// A parameter in a signature.
#[derive(Debug, Clone)]
pub struct SignatureHelpParameter {
    /// The name of the parameter.
    pub name: String,
    /// Documentation for this parameter.
    pub documentation: Vec<SymbolDisplayPart>,
    /// Display parts for the parameter.
    pub display_parts: Vec<SymbolDisplayPart>,
    /// Whether this parameter is optional.
    pub is_optional: bool,
    /// Whether this is a rest parameter.
    pub is_rest: bool,
}

impl SignatureHelpParameter {
    pub fn new(name: String) -> Self {
        Self {
            name,
            documentation: Vec::new(),
            display_parts: Vec::new(),
            is_optional: false,
            is_rest: false,
        }
    }

    pub fn with_display_parts(mut self, parts: Vec<SymbolDisplayPart>) -> Self {
        self.display_parts = parts;
        self
    }

    pub fn optional(mut self) -> Self {
        self.is_optional = true;
        self
    }

    pub fn rest(mut self) -> Self {
        self.is_rest = true;
        self
    }
}

/// JSDoc tag information.
#[derive(Debug, Clone)]
pub struct JSDocTagInfo {
    pub name: String,
    pub text: Option<Vec<SymbolDisplayPart>>,
}

// =============================================================================
// Signature Help Context
// =============================================================================

/// Trigger kind for signature help.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureHelpTriggerKind {
    /// Signature help was invoked manually (Ctrl+Shift+Space).
    Invoked,
    /// Signature help was triggered by a character (e.g., "(").
    TriggerCharacter,
    /// Signature help was re-triggered (e.g., by typing a comma).
    ContentChange,
}

/// Trigger reason for signature help.
#[derive(Debug, Clone)]
pub struct SignatureHelpTriggerReason {
    pub kind: SignatureHelpTriggerKind,
    pub trigger_character: Option<char>,
}

impl SignatureHelpTriggerReason {
    pub fn invoked() -> Self {
        Self {
            kind: SignatureHelpTriggerKind::Invoked,
            trigger_character: None,
        }
    }

    pub fn character(c: char) -> Self {
        Self {
            kind: SignatureHelpTriggerKind::TriggerCharacter,
            trigger_character: Some(c),
        }
    }

    pub fn content_change() -> Self {
        Self {
            kind: SignatureHelpTriggerKind::ContentChange,
            trigger_character: None,
        }
    }
}

/// Context for signature help operations.
pub struct SignatureHelpContext<'a> {
    pub arena: &'a NodeArena,
    pub checker: &'a CheckerState,
    pub source_text: &'a str,
    pub file_name: &'a str,
    pub trigger_reason: SignatureHelpTriggerReason,
}

// =============================================================================
// Get Signature Help
// =============================================================================

/// Get signature help at a position.
pub fn get_signature_help_items(
    ctx: &SignatureHelpContext,
    position: u32,
) -> Option<SignatureHelpItems> {
    // Find the call-like expression containing the position
    let call_info = find_containing_call_expression(ctx.arena, position)?;

    // Get the signature(s) for this call
    let signatures = get_call_signatures(ctx, call_info.expression_id)?;

    if signatures.is_empty() {
        return None;
    }

    // Create signature help items
    let mut items = Vec::new();

    for signature in &signatures {
        let item = create_signature_help_item(ctx, signature);
        items.push(item);
    }

    // Determine the applicable span
    let applicable_span = TextSpan::from_bounds(
        call_info.arguments_start,
        call_info.arguments_end,
    );

    // Determine which signature is best
    let selected_index = find_best_signature(&signatures, call_info.argument_count);

    Some(SignatureHelpItems::new(
        items,
        applicable_span,
        selected_index,
        call_info.current_argument_index,
        call_info.argument_count,
    ))
}

/// Retrigger signature help with updated position.
pub fn get_signature_help_items_retrigger(
    ctx: &SignatureHelpContext,
    position: u32,
    previous_items: &SignatureHelpItems,
) -> Option<SignatureHelpItems> {
    // Get fresh signature help
    let mut items = get_signature_help_items(ctx, position)?;

    // Try to preserve the previously selected signature
    if items.selected_item_index == 0 && previous_items.selected_item_index < items.items.len() {
        items.selected_item_index = previous_items.selected_item_index;
    }

    Some(items)
}

// =============================================================================
// Call Expression Info
// =============================================================================

/// Information about a call expression for signature help.
struct CallExpressionInfo {
    /// The expression being called.
    expression_id: NodeId,
    /// Start of the arguments list.
    arguments_start: u32,
    /// End of the arguments list.
    arguments_end: u32,
    /// Index of the current argument.
    current_argument_index: usize,
    /// Total number of arguments.
    argument_count: usize,
}

fn find_containing_call_expression(
    arena: &NodeArena,
    position: u32,
) -> Option<CallExpressionInfo> {
    let node_id = find_node_at_position(arena, position)?;

    // Walk up to find a call expression
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = arena.get(id)?;

        match node.kind() {
            SyntaxKind::CallExpression => {
                return create_call_info(arena, id, position);
            }
            SyntaxKind::NewExpression => {
                return create_call_info(arena, id, position);
            }
            SyntaxKind::TaggedTemplateExpression => {
                return create_call_info(arena, id, position);
            }
            SyntaxKind::Decorator => {
                // Decorators can also trigger signature help
                return create_call_info(arena, id, position);
            }
            _ => {}
        }

        current = node.parent();
    }

    None
}

fn create_call_info(
    arena: &NodeArena,
    call_id: NodeId,
    position: u32,
) -> Option<CallExpressionInfo> {
    let call_node = arena.get(call_id)?;

    // In a full implementation, we would:
    // 1. Find the expression being called
    // 2. Find the arguments list
    // 3. Determine which argument the cursor is in
    // 4. Count the total arguments

    // For now, return a placeholder
    Some(CallExpressionInfo {
        expression_id: call_id,
        arguments_start: call_node.pos(),
        arguments_end: call_node.end(),
        current_argument_index: 0,
        argument_count: 0,
    })
}

// =============================================================================
// Signature Creation
// =============================================================================

fn get_call_signatures(
    ctx: &SignatureHelpContext,
    expression_id: NodeId,
) -> Option<Vec<Signature>> {
    // Get the type of the expression
    let type_id = ctx.checker.get_type_at_location(expression_id)?;

    // Get call signatures from the type
    let signatures = ctx.checker.get_signatures_of_type(type_id);

    if signatures.is_empty() {
        None
    } else {
        Some(signatures)
    }
}

fn create_signature_help_item(
    ctx: &SignatureHelpContext,
    signature: &Signature,
) -> SignatureHelpItem {
    let mut item = SignatureHelpItem::new();

    // Build the prefix (function name and opening paren)
    let mut prefix = Vec::new();
    if let Some(name) = &signature.declaration_name {
        prefix.push(SymbolDisplayPart::function_name(name));
    }
    prefix.push(SymbolDisplayPart::punctuation("("));
    item.prefix_display_parts = prefix;

    // Build parameters
    for param in &signature.parameters {
        let mut display_parts = Vec::new();

        if param.is_rest {
            display_parts.push(SymbolDisplayPart::punctuation("..."));
        }

        display_parts.push(SymbolDisplayPart::parameter_name(&param.name));

        if param.is_optional {
            display_parts.push(SymbolDisplayPart::punctuation("?"));
        }

        display_parts.push(SymbolDisplayPart::punctuation(":"));
        display_parts.push(SymbolDisplayPart::space());

        // Add the type
        let type_string = ctx.checker.type_to_string(param.type_id);
        display_parts.push(SymbolDisplayPart::text(&type_string));

        let sig_param = SignatureHelpParameter::new(param.name.clone())
            .with_display_parts(display_parts);

        item.add_parameter(sig_param);
    }

    // Build the suffix (closing paren and return type)
    let mut suffix = Vec::new();
    suffix.push(SymbolDisplayPart::punctuation(")"));
    suffix.push(SymbolDisplayPart::punctuation(":"));
    suffix.push(SymbolDisplayPart::space());

    let return_type = ctx.checker.type_to_string(signature.return_type);
    suffix.push(SymbolDisplayPart::text(&return_type));

    item.suffix_display_parts = suffix;

    // Check if variadic
    if signature.has_rest_parameter {
        item.is_variadic = true;
    }

    item
}

fn find_best_signature(
    signatures: &[Signature],
    argument_count: usize,
) -> usize {
    // Find the signature that best matches the current argument count
    for (index, sig) in signatures.iter().enumerate() {
        let min_args = sig.min_argument_count;
        let max_args = if sig.has_rest_parameter {
            usize::MAX
        } else {
            sig.parameters.len()
        };

        if argument_count >= min_args && argument_count <= max_args {
            return index;
        }
    }

    // Default to first signature
    0
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

/// Characters that trigger signature help.
pub fn get_signature_help_trigger_characters() -> &'static [char] {
    &['(', ',', '<']
}

/// Characters that retrigger signature help.
pub fn get_signature_help_retrigger_characters() -> &'static [char] {
    &[')']
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_help_item() {
        let mut item = SignatureHelpItem::new();

        item.add_parameter(SignatureHelpParameter::new("x".to_string()));
        item.add_parameter(SignatureHelpParameter::new("y".to_string()).optional());

        assert_eq!(item.parameters.len(), 2);
        assert!(!item.parameters[0].is_optional);
        assert!(item.parameters[1].is_optional);
    }

    #[test]
    fn test_signature_help_parameter() {
        let param = SignatureHelpParameter::new("items".to_string())
            .rest()
            .with_display_parts(vec![
                SymbolDisplayPart::punctuation("..."),
                SymbolDisplayPart::parameter_name("items"),
            ]);

        assert_eq!(param.name, "items");
        assert!(param.is_rest);
        assert!(!param.is_optional);
    }

    #[test]
    fn test_trigger_reason() {
        let invoked = SignatureHelpTriggerReason::invoked();
        assert_eq!(invoked.kind, SignatureHelpTriggerKind::Invoked);
        assert!(invoked.trigger_character.is_none());

        let character = SignatureHelpTriggerReason::character('(');
        assert_eq!(character.kind, SignatureHelpTriggerKind::TriggerCharacter);
        assert_eq!(character.trigger_character, Some('('));
    }

    #[test]
    fn test_trigger_characters() {
        let triggers = get_signature_help_trigger_characters();
        assert!(triggers.contains(&'('));
        assert!(triggers.contains(&','));
    }
}
