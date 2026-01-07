//! Hover implementation for LSP.
//!
//! Displays type information and documentation for the symbol at the cursor.

use crate::parser::thin_node::ThinNodeArena;
use crate::parser::NodeIndex;
use crate::thin_binder::ThinBinderState;
use crate::solver::TypeInterner;
use crate::lsp::position::{Position, Range, LineMap};
use crate::lsp::utils::find_node_at_offset;
use crate::lsp::resolver::{ScopeCache, ScopeCacheStats, ScopeWalker};
use crate::thin_checker::ThinCheckerState;
use crate::comments::{get_comment_ranges, get_leading_comments, get_jsdoc_content, is_jsdoc_comment, get_leading_comments_from_cache};

/// Information returned for a hover request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HoverInfo {
    /// The contents of the hover (usually Markdown)
    pub contents: Vec<String>,
    /// The range of the symbol being hovered
    pub range: Option<Range>,
}

/// Hover provider.
pub struct HoverProvider<'a> {
    arena: &'a ThinNodeArena,
    binder: &'a ThinBinderState,
    line_map: &'a LineMap,
    interner: &'a TypeInterner,
    source_text: &'a str,
    file_name: String,
}

impl<'a> HoverProvider<'a> {
    /// Create a new Hover provider.
    pub fn new(
        arena: &'a ThinNodeArena,
        binder: &'a ThinBinderState,
        line_map: &'a LineMap,
        interner: &'a TypeInterner,
        source_text: &'a str,
        file_name: String,
    ) -> Self {
        Self {
            arena,
            binder,
            line_map,
            interner,
            source_text,
            file_name,
        }
    }

    /// Get hover information at the given position.
    ///
    /// # Arguments
    /// * `root` - The root node of the AST
    /// * `position` - The cursor position
    /// * `type_cache` - Mutable reference to the persistent type cache (for performance)
    pub fn get_hover(
        &self,
        root: NodeIndex,
        position: Position,
        type_cache: &mut Option<crate::checker::TypeCache>,
    ) -> Option<HoverInfo> {
        self.get_hover_internal(root, position, type_cache, None, None)
    }

    pub fn get_hover_with_scope_cache(
        &self,
        root: NodeIndex,
        position: Position,
        type_cache: &mut Option<crate::checker::TypeCache>,
        scope_cache: &mut ScopeCache,
        scope_stats: Option<&mut ScopeCacheStats>,
    ) -> Option<HoverInfo> {
        self.get_hover_internal(root, position, type_cache, Some(scope_cache), scope_stats)
    }

    fn get_hover_internal(
        &self,
        root: NodeIndex,
        position: Position,
        type_cache: &mut Option<crate::checker::TypeCache>,
        scope_cache: Option<&mut ScopeCache>,
        mut scope_stats: Option<&mut ScopeCacheStats>,
    ) -> Option<HoverInfo> {
        // 1. Find node at position
        let offset = self.line_map.position_to_offset(position, self.source_text)?;
        let node_idx = find_node_at_offset(self.arena, offset);

        if node_idx.is_none() {
            return None;
        }

        // 2. Resolve symbol using ScopeWalker
        // We use ScopeWalker to handle local scopes correctly
        let mut walker = ScopeWalker::new(self.arena, self.binder);
        let symbol_id = if let Some(scope_cache) = scope_cache {
            walker.resolve_node_cached(root, node_idx, scope_cache, scope_stats.as_deref_mut())?
        } else {
            walker.resolve_node(root, node_idx)?
        };
        let symbol = self.binder.symbols.get(symbol_id)?;

        // 3. Compute Type Information
        // Use persistent cache if available for O(1) lookups on repeated queries
        let mut checker = if let Some(cache) = type_cache.take() {
            ThinCheckerState::with_cache(
                self.arena,
                self.binder,
                self.interner,
                self.file_name.clone(),
                cache,
            )
        } else {
            ThinCheckerState::new(
                self.arena,
                self.binder,
                self.interner,
                self.file_name.clone(),
            )
        };

        let type_id = checker.get_type_of_symbol(symbol_id);
        let type_string = checker.format_type(type_id);

        // Extract and save the updated cache for future queries
        *type_cache = Some(checker.extract_cache());

        // 4. Construct the signature string
        // e.g. "(variable) x: number" or "(function) foo(): void"
        let kind_str = self.get_symbol_kind_string(symbol);
        let declaration_str = format!("({}) {}: {}", kind_str, symbol.escaped_name, type_string);

        // 5. Extract Documentation (JSDoc)
        // Look at the declaration node (value_declaration or first declaration)
        let decl_node_idx = if !symbol.value_declaration.is_none() {
            symbol.value_declaration
        } else if let Some(&first) = symbol.declarations.first() {
            first
        } else {
            NodeIndex::NONE
        };

        let documentation = if !decl_node_idx.is_none() {
            self.get_documentation(root, decl_node_idx)
        } else {
            String::new()
        };

        // 6. Build response
        let mut contents = Vec::new();

        // Code block for the signature
        contents.push(format!("```typescript\n{}\n```", declaration_str));

        // Documentation paragraph
        if !documentation.is_empty() {
            contents.push(documentation);
        }

        // Calculate range for the hovered identifier
        let node = self.arena.get(node_idx)?;
        let start = self.line_map.offset_to_position(node.pos, self.source_text);
        let end = self.line_map.offset_to_position(node.end, self.source_text);

        Some(HoverInfo {
            contents,
            range: Some(Range::new(start, end)),
        })
    }

    /// Extract JSDoc comments preceding a node.
    /// Uses cached comment ranges from SourceFileData for O(log N) performance
    /// instead of O(N) rescanning on every hover.
    fn get_documentation(&self, root: NodeIndex, node_idx: NodeIndex) -> String {
        let Some(node) = self.arena.get(node_idx) else { return String::new() };

        // OPTIMIZATION: Use cached comments from SourceFileData instead of rescanning
        let comments = if let Some(root_node) = self.arena.get(root) {
            if let Some(sf_data) = self.arena.get_source_file(root_node) {
                &sf_data.comments
            } else {
                // Fallback: if root is not a source file, rescan (shouldn't happen in LSP)
                return String::new();
            }
        } else {
            return String::new();
        };

        // Get comments immediately before the node start position
        let leading_comments = get_leading_comments_from_cache(comments, node.pos, self.source_text);

        // Collect only the JSDoc comments immediately preceding the node
        // (not all JSDoc comments in the file up to this point)
        let mut relevant_docs = Vec::new();

        // Iterate backwards from the node
        for comment in leading_comments.iter().rev() {
            if is_jsdoc_comment(comment, self.source_text) {
                relevant_docs.push(get_jsdoc_content(comment, self.source_text));
            } else {
                // If we hit a non-JSDoc comment, stop collecting
                // (once we have docs and encounter non-doc content, that's a break)
                if !relevant_docs.is_empty() {
                    break;
                }
            }
        }

        // Restore order (we collected backwards)
        relevant_docs.reverse();
        relevant_docs.join("\n\n")
    }

    /// Helper to get a human-readable kind string for the symbol.
    fn get_symbol_kind_string(&self, symbol: &crate::binder::Symbol) -> &'static str {
        use crate::binder::symbol_flags;
        let f = symbol.flags;

        if f & symbol_flags::FUNCTION != 0 { "function" }
        else if f & symbol_flags::CLASS != 0 { "class" }
        else if f & symbol_flags::INTERFACE != 0 { "interface" }
        else if f & symbol_flags::REGULAR_ENUM != 0 { "enum" }
        else if f & symbol_flags::TYPE_ALIAS != 0 { "type" }
        else if f & (symbol_flags::VALUE_MODULE | symbol_flags::NAMESPACE_MODULE) != 0 { "module" }
        else if f & symbol_flags::METHOD != 0 { "method" }
        else if f & symbol_flags::PROPERTY != 0 { "property" }
        else if f & symbol_flags::BLOCK_SCOPED_VARIABLE != 0 { "let/const" }
        else if f & symbol_flags::FUNCTION_SCOPED_VARIABLE != 0 { "var" }
        else { "variable" }
    }
}

#[cfg(test)]
mod hover_tests {
    use super::*;
    use crate::thin_parser::ThinParserState;
    use crate::thin_binder::ThinBinderState;
    use crate::solver::TypeInterner;
    use crate::lsp::position::LineMap;

    #[test]
    fn test_hover_variable_type() {
        // /** The answer */
        // const x = 42;
        // x;
        let source = "/** The answer */\nconst x = 42;\nx;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let interner = TypeInterner::new();
        let line_map = LineMap::build(source);

        let provider = HoverProvider::new(
            parser.get_arena(),
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string()
        );

        // Hover over 'x' in the last line (line 2, column 0)
        let pos = Position::new(2, 0);
        let mut cache = None;
        let info = provider.get_hover(root, pos, &mut cache);

        assert!(info.is_some(), "Should find hover info");

        if let Some(info) = info {
            // Check that we have contents
            assert!(!info.contents.is_empty(), "Should have contents");

            // First content should be the type signature
            assert!(info.contents[0].contains("x"), "Should contain variable name");

            // Check that we have a range
            assert!(info.range.is_some(), "Should have range");
        }
    }

    #[test]
    fn test_hover_no_symbol() {
        let source = "const x = 42;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let interner = TypeInterner::new();
        let line_map = LineMap::build(source);

        let provider = HoverProvider::new(
            parser.get_arena(),
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string()
        );

        // Hover over semicolon (no symbol)
        let pos = Position::new(0, 13);
        let mut cache = None;
        let info = provider.get_hover(root, pos, &mut cache);

        assert!(info.is_none(), "Should not find hover info at semicolon");
    }

    #[test]
    fn test_hover_function() {
        let source = "function foo() { return 1; }\nfoo();";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let interner = TypeInterner::new();
        let line_map = LineMap::build(source);

        let provider = HoverProvider::new(
            parser.get_arena(),
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string()
        );

        // Hover over 'foo' in the call
        let pos = Position::new(1, 0);
        let mut cache = None;
        let info = provider.get_hover(root, pos, &mut cache);

        assert!(info.is_some(), "Should find hover info for function");

        if let Some(info) = info {
            assert!(info.contents[0].contains("foo"), "Should contain function name");
        }
    }
}
