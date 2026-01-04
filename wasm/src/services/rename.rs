//! Rename implementation.
//!
//! This module provides functionality for renaming symbols across the codebase,
//! including validation and conflict detection.

use crate::binder::{Symbol, SymbolId, SymbolFlags};
use crate::checker::CheckerState;
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::text_span::TextSpan;
use super::find_all_references::{RenameLocation, find_rename_locations, FindReferencesContext, FindReferencesOptions};
use super::symbol_display::SymbolDisplayPart;

// =============================================================================
// Rename Info
// =============================================================================

/// Information about whether a symbol can be renamed.
#[derive(Debug, Clone)]
pub enum RenameInfo {
    /// The symbol can be renamed.
    CanRename {
        /// The kind of rename.
        kind: RenameInfoKind,
        /// Display name of the symbol.
        display_name: String,
        /// Full display name including container.
        full_display_name: String,
        /// The kind of the symbol.
        kind_modifiers: String,
        /// The span to rename.
        trigger_span: TextSpan,
        /// File name containing the symbol.
        file_name: String,
    },
    /// The symbol cannot be renamed.
    CannotRename {
        /// Reason why the symbol cannot be renamed.
        reason: String,
    },
}

impl RenameInfo {
    pub fn can_rename(
        display_name: String,
        trigger_span: TextSpan,
        file_name: String,
    ) -> Self {
        Self::CanRename {
            kind: RenameInfoKind::Variable,
            display_name: display_name.clone(),
            full_display_name: display_name,
            kind_modifiers: String::new(),
            trigger_span,
            file_name,
        }
    }

    pub fn cannot_rename(reason: &str) -> Self {
        Self::CannotRename {
            reason: reason.to_string(),
        }
    }

    pub fn is_renameable(&self) -> bool {
        matches!(self, Self::CanRename { .. })
    }
}

/// The kind of rename operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameInfoKind {
    Variable,
    Function,
    Class,
    Interface,
    TypeAlias,
    Enum,
    EnumMember,
    Method,
    Property,
    Parameter,
    TypeParameter,
    Module,
    Alias,
}

impl RenameInfoKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Variable => "variable",
            Self::Function => "function",
            Self::Class => "class",
            Self::Interface => "interface",
            Self::TypeAlias => "type alias",
            Self::Enum => "enum",
            Self::EnumMember => "enum member",
            Self::Method => "method",
            Self::Property => "property",
            Self::Parameter => "parameter",
            Self::TypeParameter => "type parameter",
            Self::Module => "module",
            Self::Alias => "alias",
        }
    }

    pub fn from_symbol_flags(flags: SymbolFlags) -> Self {
        if flags.contains(SymbolFlags::Class) {
            Self::Class
        } else if flags.contains(SymbolFlags::Interface) {
            Self::Interface
        } else if flags.contains(SymbolFlags::TypeAlias) {
            Self::TypeAlias
        } else if flags.contains(SymbolFlags::Enum) {
            Self::Enum
        } else if flags.contains(SymbolFlags::EnumMember) {
            Self::EnumMember
        } else if flags.contains(SymbolFlags::Function) {
            Self::Function
        } else if flags.contains(SymbolFlags::Method) {
            Self::Method
        } else if flags.contains(SymbolFlags::Property) {
            Self::Property
        } else if flags.contains(SymbolFlags::TypeParameter) {
            Self::TypeParameter
        } else if flags.contains(SymbolFlags::Module) || flags.contains(SymbolFlags::Namespace) {
            Self::Module
        } else if flags.contains(SymbolFlags::Alias) {
            Self::Alias
        } else {
            Self::Variable
        }
    }
}

// =============================================================================
// Rename Context
// =============================================================================

/// Options for rename operations.
#[derive(Debug, Clone, Default)]
pub struct RenameOptions {
    /// Whether to find in string literals.
    pub find_in_strings: bool,
    /// Whether to find in comments.
    pub find_in_comments: bool,
    /// Whether to provide prefix and suffix text.
    pub provide_prefix_and_suffix_text: bool,
}

/// Context for rename operations.
pub struct RenameContext<'a> {
    pub arena: &'a NodeArena,
    pub checker: &'a CheckerState,
    pub source_text: &'a str,
    pub file_name: &'a str,
    pub files_to_search: &'a [String],
    pub options: RenameOptions,
}

// =============================================================================
// Get Rename Info
// =============================================================================

/// Get information about whether a symbol can be renamed.
pub fn get_rename_info(
    ctx: &RenameContext,
    position: u32,
) -> RenameInfo {
    // Find the node at the position
    let node_id = match find_node_at_position(ctx.arena, position) {
        Some(id) => id,
        None => return RenameInfo::cannot_rename("Cannot find node at position"),
    };

    let node = match ctx.arena.get(node_id) {
        Some(n) => n,
        None => return RenameInfo::cannot_rename("Cannot get node"),
    };

    // Check if the node is renameable
    if !is_renameable_node(node.kind()) {
        return RenameInfo::cannot_rename("This element cannot be renamed");
    }

    // Get the symbol
    let symbol = match ctx.checker.get_symbol_at_location(node_id) {
        Some(s) => s,
        None => return RenameInfo::cannot_rename("Cannot resolve symbol"),
    };

    // Check if the symbol can be renamed
    if let Some(reason) = check_symbol_rename_constraints(&symbol) {
        return RenameInfo::cannot_rename(&reason);
    }

    // Create rename info
    let trigger_span = TextSpan::from_bounds(node.pos(), node.end());
    let kind = RenameInfoKind::from_symbol_flags(symbol.flags);

    RenameInfo::CanRename {
        kind,
        display_name: symbol.escaped_name.clone(),
        full_display_name: get_full_display_name(ctx, &symbol),
        kind_modifiers: get_kind_modifiers(&symbol),
        trigger_span,
        file_name: ctx.file_name.to_string(),
    }
}

/// Find all locations that need to be renamed.
pub fn get_rename_locations(
    ctx: &RenameContext,
    position: u32,
) -> Option<Vec<RenameLocation>> {
    // First check if rename is possible
    let rename_info = get_rename_info(ctx, position);
    if !rename_info.is_renameable() {
        return None;
    }

    // Create find references context
    let find_ctx = FindReferencesContext {
        arena: ctx.arena,
        checker: ctx.checker,
        source_text: ctx.source_text,
        file_name: ctx.file_name,
        files_to_search: ctx.files_to_search,
        options: FindReferencesOptions {
            include_definition: true,
            search_in_strings: ctx.options.find_in_strings,
            search_in_comments: ctx.options.find_in_comments,
            ..Default::default()
        },
    };

    find_rename_locations(
        &find_ctx,
        position,
        ctx.options.find_in_strings,
        ctx.options.find_in_comments,
        ctx.options.provide_prefix_and_suffix_text,
    )
}

// =============================================================================
// Validation
// =============================================================================

/// Check if a new name is valid for a symbol.
pub fn validate_rename(
    ctx: &RenameContext,
    position: u32,
    new_name: &str,
) -> Result<(), String> {
    // Check if the name is a valid identifier
    if !is_valid_identifier(new_name) {
        return Err(format!("'{}' is not a valid identifier", new_name));
    }

    // Check for reserved words
    if is_reserved_word(new_name) {
        return Err(format!("'{}' is a reserved word", new_name));
    }

    // Get rename info to check if symbol is renameable
    let rename_info = get_rename_info(ctx, position);
    if let RenameInfo::CannotRename { reason } = rename_info {
        return Err(reason);
    }

    // In a full implementation, we would also check for:
    // - Name conflicts in the scope
    // - Breaking changes (e.g., exported API)
    // - TypeScript-specific constraints

    Ok(())
}

/// Check if there would be conflicts after renaming.
pub fn check_rename_conflicts(
    ctx: &RenameContext,
    position: u32,
    new_name: &str,
) -> Vec<RenameConflict> {
    let mut conflicts = Vec::new();

    // In a full implementation, this would:
    // 1. Find all scopes where the symbol is used
    // 2. Check if new_name already exists in those scopes
    // 3. Check for shadowing issues
    // 4. Check for module export conflicts

    conflicts
}

/// A conflict that would arise from a rename operation.
#[derive(Debug, Clone)]
pub struct RenameConflict {
    /// Description of the conflict.
    pub message: String,
    /// The file where the conflict occurs.
    pub file_name: String,
    /// The span of the conflicting symbol.
    pub text_span: TextSpan,
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

fn is_renameable_node(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Identifier
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
    )
}

fn check_symbol_rename_constraints(symbol: &Symbol) -> Option<String> {
    // Check for built-in symbols
    if symbol.escaped_name.starts_with("__") {
        return Some("Cannot rename built-in symbol".to_string());
    }

    // Check for globally-scoped symbols that shouldn't be renamed
    // (like 'undefined', 'NaN', 'Infinity')
    let reserved_globals = ["undefined", "NaN", "Infinity", "globalThis"];
    if reserved_globals.contains(&symbol.escaped_name.as_str()) {
        return Some("Cannot rename built-in global".to_string());
    }

    None
}

fn get_full_display_name(ctx: &RenameContext, symbol: &Symbol) -> String {
    // In a full implementation, this would include the container chain
    // e.g., "MyClass.myMethod"
    symbol.escaped_name.clone()
}

fn get_kind_modifiers(symbol: &Symbol) -> String {
    let mut modifiers = Vec::new();

    if symbol.flags.contains(SymbolFlags::Export) {
        modifiers.push("export");
    }
    if symbol.flags.contains(SymbolFlags::Const) {
        modifiers.push("const");
    }

    modifiers.join(",")
}

fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    // First character must be a letter, underscore, or dollar sign
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && first != '_' && first != '$' {
        return false;
    }

    // Rest can also include digits
    for c in chars {
        if !c.is_alphanumeric() && c != '_' && c != '$' {
            return false;
        }
    }

    true
}

fn is_reserved_word(name: &str) -> bool {
    const RESERVED_WORDS: &[&str] = &[
        "break", "case", "catch", "continue", "debugger", "default", "delete",
        "do", "else", "finally", "for", "function", "if", "in", "instanceof",
        "new", "return", "switch", "this", "throw", "try", "typeof", "var",
        "void", "while", "with", "class", "const", "enum", "export", "extends",
        "import", "super", "implements", "interface", "let", "package", "private",
        "protected", "public", "static", "yield", "null", "true", "false",
    ];

    RESERVED_WORDS.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rename_info_can_rename() {
        let info = RenameInfo::can_rename(
            "foo".to_string(),
            TextSpan::new(10, 3),
            "test.ts".to_string(),
        );

        assert!(info.is_renameable());
    }

    #[test]
    fn test_rename_info_cannot_rename() {
        let info = RenameInfo::cannot_rename("Cannot rename this symbol");
        assert!(!info.is_renameable());
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("_foo"));
        assert!(is_valid_identifier("$foo"));
        assert!(is_valid_identifier("foo123"));
        assert!(!is_valid_identifier("123foo"));
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("foo-bar"));
    }

    #[test]
    fn test_is_reserved_word() {
        assert!(is_reserved_word("if"));
        assert!(is_reserved_word("class"));
        assert!(is_reserved_word("const"));
        assert!(!is_reserved_word("foo"));
        assert!(!is_reserved_word("myVariable"));
    }

    #[test]
    fn test_rename_info_kind() {
        assert_eq!(RenameInfoKind::Variable.as_str(), "variable");
        assert_eq!(RenameInfoKind::Class.as_str(), "class");
        assert_eq!(RenameInfoKind::Interface.as_str(), "interface");
    }
}
