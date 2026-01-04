//! Quick Info (Hover) implementation.
//!
//! This module provides hover/tooltip information for symbols,
//! including type information, documentation, and JSDoc tags.

use crate::binder::{Symbol, SymbolId, SymbolFlags};
use crate::checker::{Type, TypeId, CheckerState};
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::symbol_display::{
    SymbolDisplayPart, JSDocTagInfo, DisplayPartsBuilder,
    get_symbol_display_parts, get_symbol_documentation, get_symbol_jsdoc_tags,
};
use super::text_span::TextSpan;
use super::utilities::ScriptElementKind;

// =============================================================================
// Quick Info
// =============================================================================

/// Quick info (hover) information for a symbol.
#[derive(Debug, Clone)]
pub struct QuickInfo {
    /// The kind of the symbol.
    pub kind: ScriptElementKind,
    /// Kind modifiers (e.g., "export", "declare").
    pub kind_modifiers: String,
    /// The span of the symbol.
    pub text_span: TextSpan,
    /// Display parts showing the symbol definition.
    pub display_parts: Vec<SymbolDisplayPart>,
    /// Documentation for the symbol.
    pub documentation: Vec<SymbolDisplayPart>,
    /// JSDoc tags.
    pub tags: Vec<JSDocTagInfo>,
}

impl QuickInfo {
    pub fn new(kind: ScriptElementKind, text_span: TextSpan) -> Self {
        Self {
            kind,
            kind_modifiers: String::new(),
            text_span,
            display_parts: Vec::new(),
            documentation: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_display_parts(mut self, parts: Vec<SymbolDisplayPart>) -> Self {
        self.display_parts = parts;
        self
    }

    pub fn with_documentation(mut self, docs: Vec<SymbolDisplayPart>) -> Self {
        self.documentation = docs;
        self
    }

    pub fn with_tags(mut self, tags: Vec<JSDocTagInfo>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_modifiers(mut self, modifiers: String) -> Self {
        self.kind_modifiers = modifiers;
        self
    }

    /// Convert display parts to a single string.
    pub fn display_string(&self) -> String {
        self.display_parts
            .iter()
            .map(|p| p.text.as_str())
            .collect()
    }

    /// Convert documentation to a single string.
    pub fn documentation_string(&self) -> String {
        self.documentation
            .iter()
            .map(|p| p.text.as_str())
            .collect()
    }
}

// =============================================================================
// Quick Info Context
// =============================================================================

/// Context for quick info operations.
pub struct QuickInfoContext<'a> {
    pub arena: &'a NodeArena,
    pub checker: &'a CheckerState,
    pub source_text: &'a str,
    pub file_name: &'a str,
}

// =============================================================================
// Get Quick Info
// =============================================================================

/// Get quick info at a position.
pub fn get_quick_info_at_position(
    ctx: &QuickInfoContext,
    position: u32,
) -> Option<QuickInfo> {
    // Find the node at the position
    let node_id = find_node_at_position(ctx.arena, position)?;
    let node = ctx.arena.get(node_id)?;

    // Get the span for the node
    let text_span = TextSpan::from_bounds(node.pos(), node.end());

    // Check for special cases first
    if let Some(quick_info) = get_quick_info_for_keyword(ctx, node_id) {
        return Some(quick_info);
    }

    // Get the symbol at this location
    let symbol = ctx.checker.get_symbol_at_location(node_id)?;

    // Build quick info from the symbol
    let quick_info = build_quick_info_for_symbol(ctx, &symbol, text_span);

    Some(quick_info)
}

/// Build quick info for a symbol.
fn build_quick_info_for_symbol(
    ctx: &QuickInfoContext,
    symbol: &Symbol,
    text_span: TextSpan,
) -> QuickInfo {
    let kind = super::utilities::get_symbol_kind(symbol);
    let kind_modifiers = get_symbol_modifiers(symbol);

    // Get display parts
    let display_parts = get_symbol_display_parts(symbol, true);

    // Get type information if applicable
    let type_parts = get_type_display_parts(ctx, symbol);

    // Combine display parts
    let mut all_display_parts = display_parts;
    if !type_parts.is_empty() {
        all_display_parts.push(SymbolDisplayPart::punctuation(":"));
        all_display_parts.push(SymbolDisplayPart::space());
        all_display_parts.extend(type_parts);
    }

    // Get documentation
    let documentation = get_symbol_documentation(symbol);

    // Get JSDoc tags
    let tags = get_symbol_jsdoc_tags(symbol);

    QuickInfo::new(kind, text_span)
        .with_display_parts(all_display_parts)
        .with_documentation(documentation)
        .with_tags(tags)
        .with_modifiers(kind_modifiers)
}

/// Get type display parts for a symbol.
fn get_type_display_parts(
    ctx: &QuickInfoContext,
    symbol: &Symbol,
) -> Vec<SymbolDisplayPart> {
    // Get the type from a declaration
    if let Some(decl_id) = symbol.declarations.first() {
        if let Some(type_id) = ctx.checker.get_type_at_location(*decl_id) {
            let type_string = ctx.checker.type_to_string(type_id);
            return vec![SymbolDisplayPart::text(&type_string)];
        }
    }

    Vec::new()
}

/// Get quick info for keywords.
fn get_quick_info_for_keyword(
    ctx: &QuickInfoContext,
    node_id: NodeId,
) -> Option<QuickInfo> {
    let node = ctx.arena.get(node_id)?;
    let kind = node.kind();

    // Only handle keyword tokens
    if !is_keyword(kind) {
        return None;
    }

    let text_span = TextSpan::from_bounds(node.pos(), node.end());
    let keyword_text = kind_to_keyword_string(kind)?;

    let mut builder = DisplayPartsBuilder::new();
    builder.push_keyword(keyword_text);

    let documentation = get_keyword_documentation(kind);

    let mut quick_info = QuickInfo::new(ScriptElementKind::Keyword, text_span)
        .with_display_parts(builder.build());

    if !documentation.is_empty() {
        quick_info.documentation = vec![SymbolDisplayPart::text(&documentation)];
    }

    Some(quick_info)
}

/// Get documentation for a keyword.
fn get_keyword_documentation(kind: SyntaxKind) -> String {
    match kind {
        SyntaxKind::ConstKeyword => "Declares a block-scoped, read-only constant.".to_string(),
        SyntaxKind::LetKeyword => "Declares a block-scoped variable.".to_string(),
        SyntaxKind::VarKeyword => "Declares a function-scoped variable.".to_string(),
        SyntaxKind::FunctionKeyword => "Declares a function.".to_string(),
        SyntaxKind::ClassKeyword => "Declares a class.".to_string(),
        SyntaxKind::InterfaceKeyword => "Declares an interface (TypeScript only).".to_string(),
        SyntaxKind::TypeKeyword => "Declares a type alias (TypeScript only).".to_string(),
        SyntaxKind::EnumKeyword => "Declares an enum (TypeScript only).".to_string(),
        SyntaxKind::AsyncKeyword => "Marks a function as asynchronous.".to_string(),
        SyntaxKind::AwaitKeyword => "Pauses async function execution until a Promise is settled.".to_string(),
        SyntaxKind::YieldKeyword => "Pauses and resumes a generator function.".to_string(),
        SyntaxKind::ReturnKeyword => "Returns a value from a function.".to_string(),
        SyntaxKind::IfKeyword => "Conditional statement.".to_string(),
        SyntaxKind::ElseKeyword => "Alternative branch in an if statement.".to_string(),
        SyntaxKind::SwitchKeyword => "Multi-way branch statement.".to_string(),
        SyntaxKind::CaseKeyword => "Defines a case in a switch statement.".to_string(),
        SyntaxKind::DefaultKeyword => "Default case in a switch statement.".to_string(),
        SyntaxKind::ForKeyword => "Loop statement.".to_string(),
        SyntaxKind::WhileKeyword => "Loop statement with condition at the start.".to_string(),
        SyntaxKind::DoKeyword => "Loop statement with condition at the end.".to_string(),
        SyntaxKind::BreakKeyword => "Exits a loop or switch statement.".to_string(),
        SyntaxKind::ContinueKeyword => "Skips to the next iteration of a loop.".to_string(),
        SyntaxKind::TryKeyword => "Defines a try block for exception handling.".to_string(),
        SyntaxKind::CatchKeyword => "Catches exceptions from a try block.".to_string(),
        SyntaxKind::FinallyKeyword => "Executes after try/catch, regardless of outcome.".to_string(),
        SyntaxKind::ThrowKeyword => "Throws an exception.".to_string(),
        SyntaxKind::ImportKeyword => "Imports from a module.".to_string(),
        SyntaxKind::ExportKeyword => "Exports from a module.".to_string(),
        SyntaxKind::ExtendsKeyword => "Indicates class inheritance or type constraint.".to_string(),
        SyntaxKind::ImplementsKeyword => "Indicates that a class implements an interface.".to_string(),
        SyntaxKind::PublicKeyword => "Marks a member as publicly accessible.".to_string(),
        SyntaxKind::PrivateKeyword => "Marks a member as privately accessible.".to_string(),
        SyntaxKind::ProtectedKeyword => "Marks a member as accessible within class and subclasses.".to_string(),
        SyntaxKind::StaticKeyword => "Marks a member as belonging to the class, not instances.".to_string(),
        SyntaxKind::ReadonlyKeyword => "Marks a property as read-only.".to_string(),
        SyntaxKind::AbstractKeyword => "Marks a class or member as abstract.".to_string(),
        SyntaxKind::ThisKeyword => "Reference to the current object.".to_string(),
        SyntaxKind::SuperKeyword => "Reference to the parent class.".to_string(),
        SyntaxKind::NewKeyword => "Creates a new instance of a class.".to_string(),
        SyntaxKind::TypeOfKeyword => "Returns the type of a value at runtime.".to_string(),
        SyntaxKind::InstanceOfKeyword => "Tests if an object is an instance of a class.".to_string(),
        SyntaxKind::InKeyword => "Tests if a property exists in an object.".to_string(),
        SyntaxKind::DeleteKeyword => "Deletes a property from an object.".to_string(),
        SyntaxKind::VoidKeyword => "Evaluates an expression and returns undefined.".to_string(),
        SyntaxKind::NullKeyword => "Represents the intentional absence of a value.".to_string(),
        SyntaxKind::TrueKeyword => "Boolean true value.".to_string(),
        SyntaxKind::FalseKeyword => "Boolean false value.".to_string(),
        _ => String::new(),
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

fn get_symbol_modifiers(symbol: &Symbol) -> String {
    let mut modifiers = Vec::new();

    if symbol.flags.contains(SymbolFlags::Export) {
        modifiers.push("export");
    }
    if symbol.flags.contains(SymbolFlags::Const) {
        modifiers.push("const");
    }
    if symbol.flags.contains(SymbolFlags::Optional) {
        modifiers.push("optional");
    }

    modifiers.join(",")
}

fn is_keyword(kind: SyntaxKind) -> bool {
    kind >= SyntaxKind::BreakKeyword && kind <= SyntaxKind::OfKeyword
}

fn kind_to_keyword_string(kind: SyntaxKind) -> Option<&'static str> {
    match kind {
        SyntaxKind::BreakKeyword => Some("break"),
        SyntaxKind::CaseKeyword => Some("case"),
        SyntaxKind::CatchKeyword => Some("catch"),
        SyntaxKind::ClassKeyword => Some("class"),
        SyntaxKind::ConstKeyword => Some("const"),
        SyntaxKind::ContinueKeyword => Some("continue"),
        SyntaxKind::DebuggerKeyword => Some("debugger"),
        SyntaxKind::DefaultKeyword => Some("default"),
        SyntaxKind::DeleteKeyword => Some("delete"),
        SyntaxKind::DoKeyword => Some("do"),
        SyntaxKind::ElseKeyword => Some("else"),
        SyntaxKind::EnumKeyword => Some("enum"),
        SyntaxKind::ExportKeyword => Some("export"),
        SyntaxKind::ExtendsKeyword => Some("extends"),
        SyntaxKind::FalseKeyword => Some("false"),
        SyntaxKind::FinallyKeyword => Some("finally"),
        SyntaxKind::ForKeyword => Some("for"),
        SyntaxKind::FunctionKeyword => Some("function"),
        SyntaxKind::IfKeyword => Some("if"),
        SyntaxKind::ImportKeyword => Some("import"),
        SyntaxKind::InKeyword => Some("in"),
        SyntaxKind::InstanceOfKeyword => Some("instanceof"),
        SyntaxKind::NewKeyword => Some("new"),
        SyntaxKind::NullKeyword => Some("null"),
        SyntaxKind::ReturnKeyword => Some("return"),
        SyntaxKind::SuperKeyword => Some("super"),
        SyntaxKind::SwitchKeyword => Some("switch"),
        SyntaxKind::ThisKeyword => Some("this"),
        SyntaxKind::ThrowKeyword => Some("throw"),
        SyntaxKind::TrueKeyword => Some("true"),
        SyntaxKind::TryKeyword => Some("try"),
        SyntaxKind::TypeOfKeyword => Some("typeof"),
        SyntaxKind::VarKeyword => Some("var"),
        SyntaxKind::VoidKeyword => Some("void"),
        SyntaxKind::WhileKeyword => Some("while"),
        SyntaxKind::WithKeyword => Some("with"),
        SyntaxKind::ImplementsKeyword => Some("implements"),
        SyntaxKind::InterfaceKeyword => Some("interface"),
        SyntaxKind::LetKeyword => Some("let"),
        SyntaxKind::PackageKeyword => Some("package"),
        SyntaxKind::PrivateKeyword => Some("private"),
        SyntaxKind::ProtectedKeyword => Some("protected"),
        SyntaxKind::PublicKeyword => Some("public"),
        SyntaxKind::StaticKeyword => Some("static"),
        SyntaxKind::YieldKeyword => Some("yield"),
        SyntaxKind::AbstractKeyword => Some("abstract"),
        SyntaxKind::AsKeyword => Some("as"),
        SyntaxKind::AsyncKeyword => Some("async"),
        SyntaxKind::AwaitKeyword => Some("await"),
        SyntaxKind::ConstructorKeyword => Some("constructor"),
        SyntaxKind::DeclareKeyword => Some("declare"),
        SyntaxKind::GetKeyword => Some("get"),
        SyntaxKind::InferKeyword => Some("infer"),
        SyntaxKind::IsKeyword => Some("is"),
        SyntaxKind::KeyOfKeyword => Some("keyof"),
        SyntaxKind::ModuleKeyword => Some("module"),
        SyntaxKind::NamespaceKeyword => Some("namespace"),
        SyntaxKind::NeverKeyword => Some("never"),
        SyntaxKind::ReadonlyKeyword => Some("readonly"),
        SyntaxKind::RequireKeyword => Some("require"),
        SyntaxKind::SetKeyword => Some("set"),
        SyntaxKind::TypeKeyword => Some("type"),
        SyntaxKind::UniqueKeyword => Some("unique"),
        SyntaxKind::UnknownKeyword => Some("unknown"),
        SyntaxKind::FromKeyword => Some("from"),
        SyntaxKind::GlobalKeyword => Some("global"),
        SyntaxKind::OfKeyword => Some("of"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_info() {
        let quick_info = QuickInfo::new(
            ScriptElementKind::VariableElement,
            TextSpan::new(10, 5),
        )
        .with_display_parts(vec![
            SymbolDisplayPart::keyword("const"),
            SymbolDisplayPart::space(),
            SymbolDisplayPart::local_name("foo"),
        ])
        .with_modifiers("const".to_string());

        assert_eq!(quick_info.kind, ScriptElementKind::VariableElement);
        assert_eq!(quick_info.display_string(), "const foo");
        assert_eq!(quick_info.kind_modifiers, "const");
    }

    #[test]
    fn test_keyword_documentation() {
        let doc = get_keyword_documentation(SyntaxKind::ConstKeyword);
        assert!(doc.contains("read-only"));

        let doc = get_keyword_documentation(SyntaxKind::AsyncKeyword);
        assert!(doc.contains("asynchronous"));
    }

    #[test]
    fn test_kind_to_keyword_string() {
        assert_eq!(kind_to_keyword_string(SyntaxKind::ConstKeyword), Some("const"));
        assert_eq!(kind_to_keyword_string(SyntaxKind::ClassKeyword), Some("class"));
        assert_eq!(kind_to_keyword_string(SyntaxKind::AsyncKeyword), Some("async"));
    }
}
