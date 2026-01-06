//! Hover implementation for LSP.
//!
//! Displays type information and documentation for the symbol at the cursor.

use crate::parser::thin_node::ThinNodeArena;
use crate::parser::NodeIndex;
use crate::thin_binder::ThinBinderState;
use crate::solver::TypeInterner;
use crate::lsp::position::{Position, Range, LineMap};
use crate::lsp::utils::find_node_at_offset;
use crate::lsp::resolver::ScopeWalker;
use crate::thin_checker::ThinCheckerState;
use crate::comments::{get_comment_ranges, get_leading_comments, get_jsdoc_content, is_jsdoc_comment};

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
    pub fn get_hover(&self, root: NodeIndex, position: Position) -> Option<HoverInfo> {
        // 1. Find node at position
        let offset = self.line_map.position_to_offset(position);
        let node_idx = find_node_at_offset(self.arena, offset);

        if node_idx.is_none() {
            return None;
        }

        // 2. Resolve symbol using ScopeWalker
        // We use ScopeWalker to handle local scopes correctly
        let mut walker = ScopeWalker::new(self.arena, self.binder);
        let symbol_id = walker.resolve_node(root, node_idx)?;
        let symbol = self.binder.symbols.get(symbol_id)?;

        // 3. Compute Type Information
        // We create a transient checker to compute the type of this specific symbol
        let mut checker = ThinCheckerState::new(
            self.arena,
            self.binder,
            self.interner,
            self.file_name.clone()
        );

        let type_id = checker.get_type_of_symbol(symbol_id);
        let type_string = checker.format_type(type_id);

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
            self.get_documentation(decl_node_idx)
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
        let start = self.line_map.offset_to_position(node.pos);
        let end = self.line_map.offset_to_position(node.end);

        Some(HoverInfo {
            contents,
            range: Some(Range::new(start, end)),
        })
    }

    /// Extract JSDoc comments preceding a node.
    fn get_documentation(&self, node_idx: NodeIndex) -> String {
        let Some(node) = self.arena.get(node_idx) else { return String::new() };

        // TODO: Cache comment ranges - this O(N) scan happens on every hover request
        // Ideally, ThinParser should cache comments and pass them to LSP providers
        let all_comments = get_comment_ranges(self.source_text);

        // Get comments before the node start position
        let leading_comments = get_leading_comments(self.source_text, node.pos, &all_comments);

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
        let info = provider.get_hover(root, pos);

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
        let info = provider.get_hover(root, pos);

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
        let info = provider.get_hover(root, pos);

        assert!(info.is_some(), "Should find hover info for function");

        if let Some(info) = info {
            assert!(info.contents[0].contains("foo"), "Should contain function name");
        }
    }
}
