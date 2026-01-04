//! Find All References implementation.
//!
//! This module provides functionality for finding all references to a symbol
//! across the codebase, as well as finding implementations of interfaces/types.

use std::collections::{HashMap, HashSet};
use crate::binder::{Symbol, SymbolId, SymbolFlags};
use crate::checker::CheckerState;
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;

use super::text_span::TextSpan;
use super::utilities::{HighlightSpanKind, ScriptElementKind};
use super::symbol_display::SymbolDisplayPart;

// =============================================================================
// Reference Entry
// =============================================================================

/// A single reference to a symbol.
#[derive(Debug, Clone)]
pub struct ReferenceEntry {
    /// The text span of the reference.
    pub text_span: TextSpan,
    /// The file containing the reference.
    pub file_name: String,
    /// Whether this is a write access.
    pub is_write_access: bool,
    /// Whether this is a definition.
    pub is_definition: bool,
    /// Whether this is in a string literal.
    pub is_in_string: Option<bool>,
}

impl ReferenceEntry {
    pub fn new(file_name: String, text_span: TextSpan) -> Self {
        Self {
            file_name,
            text_span,
            is_write_access: false,
            is_definition: false,
            is_in_string: None,
        }
    }

    pub fn definition(file_name: String, text_span: TextSpan) -> Self {
        Self {
            file_name,
            text_span,
            is_write_access: true,
            is_definition: true,
            is_in_string: None,
        }
    }

    pub fn write_access(file_name: String, text_span: TextSpan) -> Self {
        Self {
            file_name,
            text_span,
            is_write_access: true,
            is_definition: false,
            is_in_string: None,
        }
    }
}

// =============================================================================
// Referenced Symbol
// =============================================================================

/// Information about a referenced symbol.
#[derive(Debug, Clone)]
pub struct ReferencedSymbol {
    /// Information about the symbol definition.
    pub definition: ReferencedSymbolDefinitionInfo,
    /// All references to this symbol.
    pub references: Vec<ReferenceEntry>,
}

impl ReferencedSymbol {
    pub fn new(definition: ReferencedSymbolDefinitionInfo) -> Self {
        Self {
            definition,
            references: Vec::new(),
        }
    }

    pub fn add_reference(&mut self, reference: ReferenceEntry) {
        self.references.push(reference);
    }
}

/// Definition information for a referenced symbol.
#[derive(Debug, Clone)]
pub struct ReferencedSymbolDefinitionInfo {
    /// The name of the symbol.
    pub name: String,
    /// The file containing the definition.
    pub file_name: String,
    /// The span of the definition.
    pub text_span: TextSpan,
    /// The kind of the symbol.
    pub kind: ScriptElementKind,
    /// The container name.
    pub container_name: String,
    /// The container kind.
    pub container_kind: ScriptElementKind,
    /// Display parts for the symbol.
    pub display_parts: Vec<SymbolDisplayPart>,
}

impl ReferencedSymbolDefinitionInfo {
    pub fn new(
        name: String,
        file_name: String,
        text_span: TextSpan,
        kind: ScriptElementKind,
    ) -> Self {
        Self {
            name,
            file_name,
            text_span,
            kind,
            container_name: String::new(),
            container_kind: ScriptElementKind::Unknown,
            display_parts: Vec::new(),
        }
    }
}

// =============================================================================
// Implementation Location
// =============================================================================

/// Location of an implementation.
#[derive(Debug, Clone)]
pub struct ImplementationLocation {
    /// The text span of the implementation.
    pub text_span: TextSpan,
    /// The file containing the implementation.
    pub file_name: String,
    /// The kind of implementation.
    pub kind: ScriptElementKind,
    /// Display parts for the implementation.
    pub display_parts: Vec<SymbolDisplayPart>,
}

// =============================================================================
// Find References Context
// =============================================================================

/// Options for finding references.
#[derive(Debug, Clone, Default)]
pub struct FindReferencesOptions {
    /// Whether to find implementations instead of references.
    pub find_implementations: bool,
    /// Whether to include the definition.
    pub include_definition: bool,
    /// Whether to search in strings.
    pub search_in_strings: bool,
    /// Whether to search in comments.
    pub search_in_comments: bool,
    /// Limit the number of results.
    pub limit: Option<usize>,
}

/// Context for find references operations.
pub struct FindReferencesContext<'a> {
    pub arena: &'a NodeArena,
    pub checker: &'a CheckerState,
    pub source_text: &'a str,
    pub file_name: &'a str,
    /// All files to search in.
    pub files_to_search: &'a [String],
    pub options: FindReferencesOptions,
}

// =============================================================================
// Find All References
// =============================================================================

/// Entry point for find all references.
pub fn find_references_at_position(
    ctx: &FindReferencesContext,
    position: u32,
) -> Option<Vec<ReferencedSymbol>> {
    // Find the node at the position
    let node_id = find_node_at_position(ctx.arena, position)?;

    // Get the symbol at this location
    let symbol = ctx.checker.get_symbol_at_location(node_id)?;

    // Create the search set - in a real implementation this would
    // handle aliased symbols, merged declarations, etc.
    let search_symbols = create_search_symbol_set(ctx.checker, &symbol);

    // Find all references
    let mut result = Vec::new();

    for search_symbol in search_symbols {
        let definition_info = create_definition_info(ctx, &search_symbol)?;
        let mut referenced_symbol = ReferencedSymbol::new(definition_info);

        // Search in each file
        for file_name in ctx.files_to_search {
            let references = find_references_in_file(ctx, &search_symbol, file_name);
            for reference in references {
                referenced_symbol.add_reference(reference);

                // Check limit
                if let Some(limit) = ctx.options.limit {
                    if referenced_symbol.references.len() >= limit {
                        result.push(referenced_symbol);
                        return Some(result);
                    }
                }
            }
        }

        result.push(referenced_symbol);
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Get references (flattened list without symbol grouping).
pub fn get_references_at_position(
    ctx: &FindReferencesContext,
    position: u32,
) -> Option<Vec<ReferenceEntry>> {
    let referenced_symbols = find_references_at_position(ctx, position)?;

    let mut all_references = Vec::new();
    for symbol in referenced_symbols {
        for reference in symbol.references {
            all_references.push(reference);
        }
    }

    if all_references.is_empty() {
        None
    } else {
        Some(all_references)
    }
}

/// Find implementations of an interface or abstract class.
pub fn find_implementations_at_position(
    ctx: &FindReferencesContext,
    position: u32,
) -> Option<Vec<ImplementationLocation>> {
    // Find the node at the position
    let node_id = find_node_at_position(ctx.arena, position)?;

    // Get the symbol at this location
    let symbol = ctx.checker.get_symbol_at_location(node_id)?;

    // Check if this is an interface or abstract class
    if !symbol.flags.contains(SymbolFlags::Interface)
        && !symbol.flags.contains(SymbolFlags::Class) {
        // Fall back to regular references for non-interface/class symbols
        return None;
    }

    // Find all implementations
    let implementations = find_implementations(ctx, &symbol);

    if implementations.is_empty() {
        None
    } else {
        Some(implementations)
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

fn create_search_symbol_set(checker: &CheckerState, symbol: &Symbol) -> Vec<Symbol> {
    // In a full implementation, this would:
    // - Handle aliased symbols (import aliases)
    // - Handle merged declarations
    // - Handle overloaded functions
    // - Handle property symbols with shorthand assignments
    vec![symbol.clone()]
}

fn create_definition_info(
    ctx: &FindReferencesContext,
    symbol: &Symbol,
) -> Option<ReferencedSymbolDefinitionInfo> {
    let decl_id = symbol.declarations.first()?;
    let node = ctx.arena.get(*decl_id)?;

    let text_span = TextSpan::from_bounds(node.pos(), node.end());
    let kind = super::utilities::get_symbol_kind(symbol);

    Some(ReferencedSymbolDefinitionInfo::new(
        symbol.escaped_name.clone(),
        ctx.file_name.to_string(),
        text_span,
        kind,
    ))
}

fn find_references_in_file(
    ctx: &FindReferencesContext,
    symbol: &Symbol,
    file_name: &str,
) -> Vec<ReferenceEntry> {
    let mut references = Vec::new();

    // In a full implementation, this would:
    // 1. Get the source file for the given file name
    // 2. Walk the AST looking for identifiers
    // 3. For each identifier, check if it references the target symbol
    // 4. Determine if it's a read or write access
    // 5. Handle special cases like imports, exports, etc.

    // Add definitions
    if ctx.options.include_definition {
        for decl_id in &symbol.declarations {
            if let Some(node) = ctx.arena.get(*decl_id) {
                let span = TextSpan::from_bounds(node.pos(), node.end());
                references.push(ReferenceEntry::definition(
                    file_name.to_string(),
                    span,
                ));
            }
        }
    }

    references
}

fn find_implementations(
    ctx: &FindReferencesContext,
    symbol: &Symbol,
) -> Vec<ImplementationLocation> {
    let mut implementations = Vec::new();

    // In a full implementation, this would:
    // 1. For interfaces: find all classes that implement the interface
    // 2. For abstract classes: find all concrete subclasses
    // 3. For abstract methods: find all concrete implementations
    // 4. Handle complex inheritance chains

    implementations
}

// =============================================================================
// Rename Locations
// =============================================================================

/// A location that can be renamed.
#[derive(Debug, Clone)]
pub struct RenameLocation {
    /// The text span of the location.
    pub text_span: TextSpan,
    /// The file containing the location.
    pub file_name: String,
    /// The original text at this location.
    pub original_text: String,
    /// Prefix text before the rename span (e.g., for shorthand properties).
    pub prefix_text: Option<String>,
    /// Suffix text after the rename span.
    pub suffix_text: Option<String>,
}

impl RenameLocation {
    pub fn new(file_name: String, text_span: TextSpan, original_text: String) -> Self {
        Self {
            file_name,
            text_span,
            original_text,
            prefix_text: None,
            suffix_text: None,
        }
    }
}

/// Find all rename locations for a symbol.
pub fn find_rename_locations(
    ctx: &FindReferencesContext,
    position: u32,
    find_in_strings: bool,
    find_in_comments: bool,
    provide_prefix_and_suffix_text: bool,
) -> Option<Vec<RenameLocation>> {
    // Find references first
    let references = get_references_at_position(ctx, position)?;

    // Convert to rename locations
    let mut rename_locations = Vec::new();

    for reference in references {
        // In a full implementation, we would:
        // - Get the actual text at the reference location
        // - Handle shorthand property assignments
        // - Handle import/export specifiers
        // - Handle string literal property names

        let rename_location = RenameLocation::new(
            reference.file_name.clone(),
            reference.text_span.clone(),
            String::new(), // Would be extracted from source
        );

        rename_locations.push(rename_location);
    }

    if rename_locations.is_empty() {
        None
    } else {
        Some(rename_locations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_entry() {
        let entry = ReferenceEntry::new(
            "test.ts".to_string(),
            TextSpan::new(10, 5),
        );

        assert_eq!(entry.file_name, "test.ts");
        assert!(!entry.is_write_access);
        assert!(!entry.is_definition);
    }

    #[test]
    fn test_reference_entry_definition() {
        let entry = ReferenceEntry::definition(
            "test.ts".to_string(),
            TextSpan::new(10, 5),
        );

        assert!(entry.is_write_access);
        assert!(entry.is_definition);
    }

    #[test]
    fn test_referenced_symbol() {
        let def_info = ReferencedSymbolDefinitionInfo::new(
            "foo".to_string(),
            "test.ts".to_string(),
            TextSpan::new(0, 3),
            ScriptElementKind::FunctionElement,
        );

        let mut symbol = ReferencedSymbol::new(def_info);
        symbol.add_reference(ReferenceEntry::new(
            "test.ts".to_string(),
            TextSpan::new(50, 3),
        ));

        assert_eq!(symbol.definition.name, "foo");
        assert_eq!(symbol.references.len(), 1);
    }

    #[test]
    fn test_rename_location() {
        let location = RenameLocation::new(
            "test.ts".to_string(),
            TextSpan::new(10, 5),
            "oldName".to_string(),
        );

        assert_eq!(location.file_name, "test.ts");
        assert_eq!(location.original_text, "oldName");
        assert!(location.prefix_text.is_none());
    }
}
