//! Inlay Hints implementation.
//!
//! This module provides inlay hints for:
//! - Parameter names in function calls
//! - Return types for functions
//! - Variable types
//! - Property declaration types
//! - Enum member values

use crate::binder::{Symbol, SymbolId, SymbolFlags};
use crate::checker::{Type, TypeId, CheckerState};
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::symbol_display::SymbolDisplayPart;
use super::text_span::TextSpan;

// =============================================================================
// Inlay Hint Types
// =============================================================================

/// An inlay hint to display in the editor.
#[derive(Debug, Clone)]
pub struct InlayHint {
    /// The position where the hint should be displayed.
    pub position: u32,
    /// The text of the hint.
    pub text: String,
    /// The kind of hint.
    pub kind: InlayHintKind,
    /// Whether there should be whitespace before the hint.
    pub whitespace_before: Option<bool>,
    /// Whether there should be whitespace after the hint.
    pub whitespace_after: Option<bool>,
    /// Display parts for rich rendering.
    pub display_parts: Option<Vec<InlayHintDisplayPart>>,
}

impl InlayHint {
    pub fn new(position: u32, text: String, kind: InlayHintKind) -> Self {
        Self {
            position,
            text,
            kind,
            whitespace_before: None,
            whitespace_after: None,
            display_parts: None,
        }
    }

    pub fn parameter(position: u32, name: &str) -> Self {
        Self::new(position, format!("{}:", name), InlayHintKind::Parameter)
            .with_whitespace_after()
    }

    pub fn type_hint(position: u32, type_text: &str) -> Self {
        Self::new(position, format!(": {}", type_text), InlayHintKind::Type)
    }

    pub fn enum_member(position: u32, value: &str) -> Self {
        Self::new(position, format!("= {}", value), InlayHintKind::EnumMember)
            .with_whitespace_before()
    }

    pub fn with_whitespace_before(mut self) -> Self {
        self.whitespace_before = Some(true);
        self
    }

    pub fn with_whitespace_after(mut self) -> Self {
        self.whitespace_after = Some(true);
        self
    }

    pub fn with_display_parts(mut self, parts: Vec<InlayHintDisplayPart>) -> Self {
        self.display_parts = Some(parts);
        self
    }
}

/// The kind of inlay hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlayHintKind {
    /// A parameter name hint.
    Parameter,
    /// A type annotation hint.
    Type,
    /// An enum member value hint.
    EnumMember,
}

impl InlayHintKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Parameter => "parameter",
            Self::Type => "type",
            Self::EnumMember => "enumMember",
        }
    }
}

/// A display part for an inlay hint.
#[derive(Debug, Clone)]
pub struct InlayHintDisplayPart {
    /// The text to display.
    pub text: String,
    /// The span for navigation (if clickable).
    pub span: Option<InlayHintSpan>,
}

impl InlayHintDisplayPart {
    pub fn text(text: &str) -> Self {
        Self {
            text: text.to_string(),
            span: None,
        }
    }

    pub fn with_span(mut self, file: String, start: u32, length: u32) -> Self {
        self.span = Some(InlayHintSpan {
            file_name: file,
            text_span: TextSpan::new(start, length),
        });
        self
    }
}

/// Span information for clickable inlay hint parts.
#[derive(Debug, Clone)]
pub struct InlayHintSpan {
    pub file_name: String,
    pub text_span: TextSpan,
}

// =============================================================================
// Inlay Hints Preferences
// =============================================================================

/// Preferences for inlay hints.
#[derive(Debug, Clone, Default)]
pub struct InlayHintsPreferences {
    /// Show parameter name hints.
    pub include_inlay_parameter_name_hints: ParameterNameHintsOption,
    /// Show parameter name hints only for literals.
    pub include_inlay_parameter_name_hints_when_argument_matches_name: bool,
    /// Show return type hints for functions.
    pub include_inlay_function_like_return_type_hints: bool,
    /// Show type hints for variables.
    pub include_inlay_variable_type_hints: bool,
    /// Don't show type hints for variables with obvious types.
    pub include_inlay_variable_type_hints_when_type_matches_name: bool,
    /// Show type hints for properties.
    pub include_inlay_property_declaration_type_hints: bool,
    /// Show type hints for function parameters.
    pub include_inlay_function_parameter_type_hints: bool,
    /// Show enum member value hints.
    pub include_inlay_enum_member_value_hints: bool,
}

/// Options for parameter name hints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParameterNameHintsOption {
    /// Don't show parameter name hints.
    #[default]
    None,
    /// Show hints for literals only.
    Literals,
    /// Show all parameter name hints.
    All,
}

// =============================================================================
// Inlay Hints Context
// =============================================================================

/// Context for inlay hints operations.
pub struct InlayHintsContext<'a> {
    pub arena: &'a NodeArena,
    pub checker: &'a CheckerState,
    pub source_text: &'a str,
    pub file_name: &'a str,
    pub preferences: InlayHintsPreferences,
    /// Start of the range to provide hints for.
    pub start: u32,
    /// End of the range to provide hints for.
    pub end: u32,
}

// =============================================================================
// Get Inlay Hints
// =============================================================================

/// Get inlay hints for a range.
pub fn provide_inlay_hints(
    ctx: &InlayHintsContext,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();

    // Get the root node
    let root = match ctx.arena.root() {
        Some(r) => r,
        None => return hints,
    };

    // Walk the AST in the range
    collect_inlay_hints(ctx, root, &mut hints);

    // Sort by position
    hints.sort_by_key(|h| h.position);

    hints
}

fn collect_inlay_hints(
    ctx: &InlayHintsContext,
    node_id: NodeId,
    hints: &mut Vec<InlayHint>,
) {
    let node = match ctx.arena.get(node_id) {
        Some(n) => n,
        None => return,
    };

    // Skip nodes outside the range
    if node.end() < ctx.start || node.pos() > ctx.end {
        return;
    }

    // Check for hints on this node
    match node.kind() {
        SyntaxKind::CallExpression | SyntaxKind::NewExpression => {
            if ctx.preferences.include_inlay_parameter_name_hints != ParameterNameHintsOption::None {
                collect_parameter_name_hints(ctx, node_id, hints);
            }
        }

        SyntaxKind::VariableDeclaration => {
            if ctx.preferences.include_inlay_variable_type_hints {
                collect_variable_type_hints(ctx, node_id, hints);
            }
        }

        SyntaxKind::PropertyDeclaration => {
            if ctx.preferences.include_inlay_property_declaration_type_hints {
                collect_property_type_hints(ctx, node_id, hints);
            }
        }

        SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::MethodDeclaration => {
            if ctx.preferences.include_inlay_function_like_return_type_hints {
                collect_return_type_hints(ctx, node_id, hints);
            }
            if ctx.preferences.include_inlay_function_parameter_type_hints {
                collect_parameter_type_hints(ctx, node_id, hints);
            }
        }

        SyntaxKind::EnumMember => {
            if ctx.preferences.include_inlay_enum_member_value_hints {
                collect_enum_member_hints(ctx, node_id, hints);
            }
        }

        _ => {}
    }

    // Recurse into children
    for child_id in node.children() {
        collect_inlay_hints(ctx, child_id, hints);
    }
}

// =============================================================================
// Parameter Name Hints
// =============================================================================

fn collect_parameter_name_hints(
    ctx: &InlayHintsContext,
    call_id: NodeId,
    hints: &mut Vec<InlayHint>,
) {
    let call = match ctx.arena.get(call_id) {
        Some(n) => n,
        None => return,
    };

    // Get the signature being called
    let signature = match ctx.checker.get_resolved_signature(call_id) {
        Some(s) => s,
        None => return,
    };

    // Get the arguments
    let arguments = get_call_arguments(ctx.arena, call_id);

    for (index, arg_id) in arguments.iter().enumerate() {
        // Check if we should show a hint for this argument
        if !should_show_parameter_hint(ctx, *arg_id) {
            continue;
        }

        // Get the parameter name
        if let Some(param) = signature.parameters.get(index) {
            let arg = match ctx.arena.get(*arg_id) {
                Some(n) => n,
                None => continue,
            };

            // Don't show hints for spread arguments
            if arg.kind() == SyntaxKind::SpreadElement {
                continue;
            }

            // Don't show if argument is already named (object shorthand)
            if is_argument_name_matching(&param.name, ctx.arena, *arg_id) {
                if ctx.preferences.include_inlay_parameter_name_hints_when_argument_matches_name {
                    continue;
                }
            }

            let hint = InlayHint::parameter(arg.pos(), &param.name);
            hints.push(hint);
        }
    }
}

fn should_show_parameter_hint(ctx: &InlayHintsContext, arg_id: NodeId) -> bool {
    let arg = match ctx.arena.get(arg_id) {
        Some(n) => n,
        None => return false,
    };

    match ctx.preferences.include_inlay_parameter_name_hints {
        ParameterNameHintsOption::None => false,
        ParameterNameHintsOption::All => true,
        ParameterNameHintsOption::Literals => {
            // Only show for literal arguments
            is_literal_expression(arg.kind())
        }
    }
}

fn is_literal_expression(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ObjectLiteralExpression
    )
}

fn is_argument_name_matching(param_name: &str, arena: &NodeArena, arg_id: NodeId) -> bool {
    let arg = match arena.get(arg_id) {
        Some(n) => n,
        None => return false,
    };

    // Check if the argument is an identifier with the same name
    if arg.kind() == SyntaxKind::Identifier {
        // In a full implementation, we'd get the identifier text
        return false;
    }

    false
}

// =============================================================================
// Variable Type Hints
// =============================================================================

fn collect_variable_type_hints(
    ctx: &InlayHintsContext,
    var_id: NodeId,
    hints: &mut Vec<InlayHint>,
) {
    let var_decl = match ctx.arena.get(var_id) {
        Some(n) => n,
        None => return,
    };

    // Don't show if there's already an explicit type annotation
    if has_explicit_type_annotation(ctx.arena, var_id) {
        return;
    }

    // Get the type
    let type_id = match ctx.checker.get_type_at_location(var_id) {
        Some(t) => t,
        None => return,
    };

    let type_string = ctx.checker.type_to_string(type_id);

    // Don't show obvious types
    if ctx.preferences.include_inlay_variable_type_hints_when_type_matches_name {
        if is_type_obvious(ctx.arena, var_id, &type_string) {
            return;
        }
    }

    // Truncate long types
    let type_string = truncate_type_string(&type_string, 30);

    // Position after the variable name
    let position = get_name_end_position(ctx.arena, var_id);

    let hint = InlayHint::type_hint(position, &type_string);
    hints.push(hint);
}

// =============================================================================
// Property Type Hints
// =============================================================================

fn collect_property_type_hints(
    ctx: &InlayHintsContext,
    prop_id: NodeId,
    hints: &mut Vec<InlayHint>,
) {
    let prop_decl = match ctx.arena.get(prop_id) {
        Some(n) => n,
        None => return,
    };

    // Don't show if there's already an explicit type annotation
    if has_explicit_type_annotation(ctx.arena, prop_id) {
        return;
    }

    // Get the type
    let type_id = match ctx.checker.get_type_at_location(prop_id) {
        Some(t) => t,
        None => return,
    };

    let type_string = ctx.checker.type_to_string(type_id);
    let type_string = truncate_type_string(&type_string, 30);

    let position = get_name_end_position(ctx.arena, prop_id);

    let hint = InlayHint::type_hint(position, &type_string);
    hints.push(hint);
}

// =============================================================================
// Return Type Hints
// =============================================================================

fn collect_return_type_hints(
    ctx: &InlayHintsContext,
    func_id: NodeId,
    hints: &mut Vec<InlayHint>,
) {
    let func = match ctx.arena.get(func_id) {
        Some(n) => n,
        None => return,
    };

    // Don't show if there's already an explicit return type
    if has_explicit_return_type(ctx.arena, func_id) {
        return;
    }

    // Get the function type and extract return type
    let type_id = match ctx.checker.get_type_at_location(func_id) {
        Some(t) => t,
        None => return,
    };

    let return_type = match ctx.checker.get_return_type_of_signature(type_id) {
        Some(t) => t,
        None => return,
    };

    let type_string = ctx.checker.type_to_string(return_type);
    let type_string = truncate_type_string(&type_string, 30);

    // Position after the parameter list
    let position = get_return_type_position(ctx.arena, func_id);

    let hint = InlayHint::type_hint(position, &type_string);
    hints.push(hint);
}

// =============================================================================
// Parameter Type Hints
// =============================================================================

fn collect_parameter_type_hints(
    ctx: &InlayHintsContext,
    func_id: NodeId,
    hints: &mut Vec<InlayHint>,
) {
    // Get parameters
    let params = get_function_parameters(ctx.arena, func_id);

    for param_id in params {
        // Don't show if there's already an explicit type annotation
        if has_explicit_type_annotation(ctx.arena, param_id) {
            continue;
        }

        // Get the type
        let type_id = match ctx.checker.get_type_at_location(param_id) {
            Some(t) => t,
            None => continue,
        };

        let type_string = ctx.checker.type_to_string(type_id);
        let type_string = truncate_type_string(&type_string, 30);

        let position = get_name_end_position(ctx.arena, param_id);

        let hint = InlayHint::type_hint(position, &type_string);
        hints.push(hint);
    }
}

// =============================================================================
// Enum Member Hints
// =============================================================================

fn collect_enum_member_hints(
    ctx: &InlayHintsContext,
    member_id: NodeId,
    hints: &mut Vec<InlayHint>,
) {
    let member = match ctx.arena.get(member_id) {
        Some(n) => n,
        None => return,
    };

    // Don't show if there's already an explicit initializer
    if has_explicit_initializer(ctx.arena, member_id) {
        return;
    }

    // Get the enum member value
    let value = match ctx.checker.get_constant_value(member_id) {
        Some(v) => v,
        None => return,
    };

    let hint = InlayHint::enum_member(member.end(), &value.to_string());
    hints.push(hint);
}

// =============================================================================
// Helper Functions
// =============================================================================

fn get_call_arguments(arena: &NodeArena, call_id: NodeId) -> Vec<NodeId> {
    // In a full implementation, get the arguments from the call expression
    Vec::new()
}

fn get_function_parameters(arena: &NodeArena, func_id: NodeId) -> Vec<NodeId> {
    // In a full implementation, get the parameters from the function
    Vec::new()
}

fn has_explicit_type_annotation(arena: &NodeArena, node_id: NodeId) -> bool {
    // In a full implementation, check if the node has a type annotation
    false
}

fn has_explicit_return_type(arena: &NodeArena, func_id: NodeId) -> bool {
    // In a full implementation, check if the function has a return type annotation
    false
}

fn has_explicit_initializer(arena: &NodeArena, node_id: NodeId) -> bool {
    // In a full implementation, check if the node has an initializer
    false
}

fn is_type_obvious(arena: &NodeArena, var_id: NodeId, type_string: &str) -> bool {
    // Check if the type is obvious from the initializer
    // e.g., `const x = 5` -> type is `number`
    // e.g., `const foo = new Foo()` -> type is `Foo`
    false
}

fn get_name_end_position(arena: &NodeArena, node_id: NodeId) -> u32 {
    // In a full implementation, get the end of the name/identifier
    let node = match arena.get(node_id) {
        Some(n) => n,
        None => return 0,
    };
    node.end()
}

fn get_return_type_position(arena: &NodeArena, func_id: NodeId) -> u32 {
    // In a full implementation, get the position after the parameter list
    let node = match arena.get(func_id) {
        Some(n) => n,
        None => return 0,
    };
    node.end()
}

fn truncate_type_string(type_string: &str, max_length: usize) -> String {
    if type_string.len() <= max_length {
        type_string.to_string()
    } else {
        format!("{}...", &type_string[..max_length - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inlay_hint_parameter() {
        let hint = InlayHint::parameter(10, "name");

        assert_eq!(hint.position, 10);
        assert_eq!(hint.text, "name:");
        assert_eq!(hint.kind, InlayHintKind::Parameter);
        assert_eq!(hint.whitespace_after, Some(true));
    }

    #[test]
    fn test_inlay_hint_type() {
        let hint = InlayHint::type_hint(20, "string");

        assert_eq!(hint.position, 20);
        assert_eq!(hint.text, ": string");
        assert_eq!(hint.kind, InlayHintKind::Type);
    }

    #[test]
    fn test_inlay_hint_enum_member() {
        let hint = InlayHint::enum_member(30, "0");

        assert_eq!(hint.position, 30);
        assert_eq!(hint.text, "= 0");
        assert_eq!(hint.kind, InlayHintKind::EnumMember);
    }

    #[test]
    fn test_truncate_type_string() {
        assert_eq!(truncate_type_string("string", 30), "string");
        assert_eq!(
            truncate_type_string("a very long type string here", 15),
            "a very long ..."
        );
    }

    #[test]
    fn test_is_literal_expression() {
        assert!(is_literal_expression(SyntaxKind::StringLiteral));
        assert!(is_literal_expression(SyntaxKind::NumericLiteral));
        assert!(is_literal_expression(SyntaxKind::TrueKeyword));
        assert!(!is_literal_expression(SyntaxKind::Identifier));
    }
}
