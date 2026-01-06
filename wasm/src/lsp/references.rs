//! Find References implementation for LSP.
//!
//! Given a position in the source, finds all references to the symbol at that position.

use crate::parser::thin_node::ThinNodeArena;
use crate::parser::NodeIndex;
use crate::thin_binder::ThinBinderState;
use crate::lsp::position::{Position, Location, LineMap, Range};
use crate::lsp::utils::find_node_at_offset;
use crate::lsp::resolver::ScopeWalker;

/// Find References provider.
///
/// This struct provides LSP "Find References" functionality by:
/// 1. Converting a position to a byte offset
/// 2. Finding the AST node at that offset
/// 3. Resolving the node to a symbol
/// 4. Finding all references to that symbol in the AST
/// 5. Returning their locations
pub struct FindReferences<'a> {
    arena: &'a ThinNodeArena,
    binder: &'a ThinBinderState,
    line_map: &'a LineMap,
    file_name: String,
}

impl<'a> FindReferences<'a> {
    /// Create a new Find References provider.
    pub fn new(
        arena: &'a ThinNodeArena,
        binder: &'a ThinBinderState,
        line_map: &'a LineMap,
        file_name: String,
    ) -> Self {
        Self {
            arena,
            binder,
            line_map,
            file_name,
        }
    }

    /// Find all references to the symbol at the given position.
    ///
    /// Returns a list of locations where the symbol is referenced.
    /// This includes both the declaration(s) and all usages.
    ///
    /// Returns None if no symbol is found at the position.
    pub fn find_references(&self, root: NodeIndex, position: Position) -> Option<Vec<Location>> {
        // 1. Convert position to byte offset
        let offset = self.line_map.position_to_offset(position);

        // 2. Find the most specific node at this offset
        let node_idx = find_node_at_offset(self.arena, offset);
        if node_idx.is_none() {
            return None;
        }

        // 3. Resolve the node to a symbol
        let mut walker = ScopeWalker::new(self.arena, self.binder);
        let symbol_id = walker.resolve_node(root, node_idx)?;

        // 4. Find all references to this symbol
        let ref_nodes = walker.find_references(root, symbol_id);

        // 5. Also include the declarations
        let symbol = self.binder.symbols.get(symbol_id)?;
        let mut all_nodes = ref_nodes.clone();
        all_nodes.extend(symbol.declarations.iter().copied());

        // Remove duplicates (a declaration might also be a reference)
        all_nodes.sort_by_key(|n| n.0);
        all_nodes.dedup();

        // 6. Convert to Locations
        let locations: Vec<Location> = all_nodes
            .iter()
            .filter_map(|&idx| {
                let node = self.arena.get(idx)?;
                let start_pos = self.line_map.offset_to_position(node.pos);
                let end_pos = self.line_map.offset_to_position(node.end);

                Some(Location {
                    file_path: self.file_name.clone(),
                    range: Range::new(start_pos, end_pos),
                })
            })
            .collect();

        if locations.is_empty() {
            None
        } else {
            Some(locations)
        }
    }

    /// Find references for a specific node (by NodeIndex).
    ///
    /// This is useful when you already have the node index from another operation.
    pub fn find_references_for_node(&self, root: NodeIndex, node_idx: NodeIndex) -> Option<Vec<Location>> {
        if node_idx.is_none() {
            return None;
        }

        // Resolve the node to a symbol
        let mut walker = ScopeWalker::new(self.arena, self.binder);
        let symbol_id = walker.resolve_node(root, node_idx)?;

        // Find all references to this symbol
        let ref_nodes = walker.find_references(root, symbol_id);

        // Also include the declarations
        let symbol = self.binder.symbols.get(symbol_id)?;
        let mut all_nodes = ref_nodes.clone();
        all_nodes.extend(symbol.declarations.iter().copied());

        // Remove duplicates
        all_nodes.sort_by_key(|n| n.0);
        all_nodes.dedup();

        // Convert to Locations
        let locations: Vec<Location> = all_nodes
            .iter()
            .filter_map(|&idx| {
                let node = self.arena.get(idx)?;
                let start_pos = self.line_map.offset_to_position(node.pos);
                let end_pos = self.line_map.offset_to_position(node.end);

                Some(Location {
                    file_path: self.file_name.clone(),
                    range: Range::new(start_pos, end_pos),
                })
            })
            .collect();

        if locations.is_empty() {
            None
        } else {
            Some(locations)
        }
    }

    /// Find only usages (excluding declarations) for the symbol at the given position.
    pub fn find_usages_only(&self, root: NodeIndex, position: Position) -> Option<Vec<Location>> {
        let offset = self.line_map.position_to_offset(position);
        let node_idx = find_node_at_offset(self.arena, offset);
        if node_idx.is_none() {
            return None;
        }

        let mut walker = ScopeWalker::new(self.arena, self.binder);
        let symbol_id = walker.resolve_node(root, node_idx)?;

        // Find all references (usages only, not declarations)
        let ref_nodes = walker.find_references(root, symbol_id);

        // Convert to Locations
        let locations: Vec<Location> = ref_nodes
            .iter()
            .filter_map(|&idx| {
                let node = self.arena.get(idx)?;
                let start_pos = self.line_map.offset_to_position(node.pos);
                let end_pos = self.line_map.offset_to_position(node.end);

                Some(Location {
                    file_path: self.file_name.clone(),
                    range: Range::new(start_pos, end_pos),
                })
            })
            .collect();

        if locations.is_empty() {
            None
        } else {
            Some(locations)
        }
    }
}

#[cfg(test)]
mod references_tests {
    use super::*;
    use crate::thin_parser::ThinParserState;
    use crate::thin_binder::ThinBinderState;
    use crate::lsp::position::LineMap;

    #[test]
    #[ignore] // TODO: Implement proper AST traversal in ScopeWalker
    fn test_find_references_simple() {
        // const x = 1;
        // x + x;
        let source = "const x = 1;\nx + x;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        let arena = parser.get_arena();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(arena, root);

        let line_map = LineMap::build(source);

        // Position at the first 'x' in "x + x" (line 1, column 0)
        let position = Position::new(1, 0);

        let find_refs = FindReferences::new(arena, &binder, &line_map, "test.ts".to_string());
        let references = find_refs.find_references(root, position);

        assert!(references.is_some(), "Should find references for x");

        if let Some(refs) = references {
            // Should find at least the declaration and two usages
            assert!(refs.len() >= 2, "Should find at least 2 references (declaration + usages)");
        }
    }

    #[test]
    fn test_find_references_not_found() {
        let source = "const x = 1;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        let arena = parser.get_arena();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(arena, root);

        let line_map = LineMap::build(source);

        // Position outside any identifier
        let position = Position::new(0, 11); // At the semicolon

        let find_refs = FindReferences::new(arena, &binder, &line_map, "test.ts".to_string());
        let references = find_refs.find_references(root, position);

        // Should not find references
        assert!(references.is_none(), "Should not find references at semicolon");
    }
}
