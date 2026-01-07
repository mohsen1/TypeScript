//! Completions implementation for LSP.
//!
//! Given a position in the source, provides completion suggestions for
//! identifiers that are visible at that position.

use crate::parser::thin_node::ThinNodeArena;
use crate::parser::NodeIndex;
use crate::thin_binder::ThinBinderState;
use crate::lsp::position::{Position, LineMap};
use crate::lsp::resolver::ScopeWalker;
use crate::lsp::utils::find_node_at_offset;

/// The kind of completion item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CompletionItemKind {
    /// A variable or constant
    Variable,
    /// A function
    Function,
    /// A class
    Class,
    /// A method
    Method,
    /// A parameter
    Parameter,
    /// A property
    Property,
}

/// A completion item to be suggested to the user.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompletionItem {
    /// The label to display in the completion list
    pub label: String,
    /// The kind of completion item
    pub kind: CompletionItemKind,
    /// Optional detail text (e.g., type information)
    pub detail: Option<String>,
    /// Optional documentation
    pub documentation: Option<String>,
}

impl CompletionItem {
    /// Create a new completion item.
    pub fn new(label: String, kind: CompletionItemKind) -> Self {
        Self {
            label,
            kind,
            detail: None,
            documentation: None,
        }
    }

    /// Set the detail text.
    pub fn with_detail(mut self, detail: String) -> Self {
        self.detail = Some(detail);
        self
    }

    /// Set the documentation.
    pub fn with_documentation(mut self, documentation: String) -> Self {
        self.documentation = Some(documentation);
        self
    }
}

/// Completions provider.
///
/// This struct provides LSP "Completions" functionality by:
/// 1. Converting a position to a byte offset
/// 2. Finding the AST node at that offset
/// 3. Getting the active scope chain at that position
/// 4. Collecting all visible identifiers from the scope chain
/// 5. Returning them as completion items
pub struct Completions<'a> {
    arena: &'a ThinNodeArena,
    binder: &'a ThinBinderState,
    line_map: &'a LineMap,
    source_text: &'a str,
}

impl<'a> Completions<'a> {
    /// Create a new Completions provider.
    pub fn new(
        arena: &'a ThinNodeArena,
        binder: &'a ThinBinderState,
        line_map: &'a LineMap,
        source_text: &'a str,
    ) -> Self {
        Self {
            arena,
            binder,
            line_map,
            source_text,
        }
    }

    /// Get completion suggestions at the given position.
    ///
    /// Returns a list of completion items for identifiers visible at the cursor position.
    /// Returns None if no completions are available.
    pub fn get_completions(&self, root: NodeIndex, position: Position) -> Option<Vec<CompletionItem>> {
        // 1. Convert position to byte offset
        let offset = self.line_map.position_to_offset(position, self.source_text)?;

        // 2. Find the node at this offset (or use root if not found)
        let node_idx = find_node_at_offset(self.arena, offset);
        let node_idx = if node_idx.is_none() { root } else { node_idx };

        // 3. Get the scope chain at this position
        let mut walker = ScopeWalker::new(self.arena, self.binder);
        let scope_chain = walker.get_scope_chain(root, node_idx);

        // 4. Collect all visible identifiers from the scope chain
        let mut completions = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        // Walk scopes from innermost to outermost
        for scope in scope_chain.iter().rev() {
            for (name, symbol_id) in scope.iter() {
                // Skip if we've already seen this name (inner scopes shadow outer scopes)
                if seen_names.contains(name) {
                    continue;
                }
                seen_names.insert(name.clone());

                // Get the symbol to determine its kind
                if let Some(symbol) = self.binder.symbols.get(*symbol_id) {
                    let kind = self.determine_completion_kind(symbol);
                    let mut item = CompletionItem::new(name.clone(), kind);

                    // Add detail information if available
                    if let Some(detail) = self.get_symbol_detail(symbol) {
                        item = item.with_detail(detail);
                    }

                    completions.push(item);
                }
            }
        }

        if completions.is_empty() {
            None
        } else {
            // Sort completions alphabetically for better UX
            completions.sort_by(|a, b| a.label.cmp(&b.label));
            Some(completions)
        }
    }

    /// Determine the completion kind from a symbol.
    fn determine_completion_kind(&self, symbol: &crate::binder::Symbol) -> CompletionItemKind {
        use crate::binder::symbol_flags;

        if symbol.flags & symbol_flags::FUNCTION != 0 {
            CompletionItemKind::Function
        } else if symbol.flags & symbol_flags::CLASS != 0 {
            CompletionItemKind::Class
        } else if symbol.flags & symbol_flags::METHOD != 0 {
            CompletionItemKind::Method
        } else if symbol.flags & symbol_flags::PROPERTY != 0 {
            CompletionItemKind::Property
        } else if symbol.flags & symbol_flags::VALUE_MODULE != 0 {
            // Parameters are value modules in the binder
            CompletionItemKind::Parameter
        } else {
            // Default to variable for const, let, var
            CompletionItemKind::Variable
        }
    }

    /// Get detail information for a symbol (e.g., "const", "function", "class").
    fn get_symbol_detail(&self, symbol: &crate::binder::Symbol) -> Option<String> {
        use crate::binder::symbol_flags;

        if symbol.flags & symbol_flags::FUNCTION != 0 {
            Some("function".to_string())
        } else if symbol.flags & symbol_flags::CLASS != 0 {
            Some("class".to_string())
        } else if symbol.flags & symbol_flags::INTERFACE != 0 {
            Some("interface".to_string())
        } else if symbol.flags & symbol_flags::REGULAR_ENUM != 0 {
            Some("enum".to_string())
        } else if symbol.flags & symbol_flags::TYPE_ALIAS != 0 {
            Some("type".to_string())
        } else if symbol.flags & symbol_flags::METHOD != 0 {
            Some("method".to_string())
        } else if symbol.flags & symbol_flags::PROPERTY != 0 {
            Some("property".to_string())
        } else if symbol.flags & symbol_flags::BLOCK_SCOPED_VARIABLE != 0 {
            Some("let/const".to_string())
        } else if symbol.flags & symbol_flags::FUNCTION_SCOPED_VARIABLE != 0 {
            Some("var".to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod completions_tests {
    use super::*;
    use crate::thin_parser::ThinParserState;
    use crate::thin_binder::ThinBinderState;
    use crate::lsp::position::LineMap;

    #[test]
    fn test_completions_simple() {
        // const x = 1;
        // const y = 2;
        // |  <- cursor here
        let source = "const x = 1;\nconst y = 2;\n";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        let arena = parser.get_arena();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(arena, root);

        let line_map = LineMap::build(source);

        // Position at the end (line 2, column 0)
        let position = Position::new(2, 0);

        let completions = Completions::new(arena, &binder, &line_map, source);
        let items = completions.get_completions(root, position);

        assert!(items.is_some(), "Should have completions");

        if let Some(items) = items {
            // Should suggest both x and y
            assert!(items.len() >= 2, "Should have at least 2 completions");

            let names: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
            assert!(names.contains(&"x"), "Should suggest 'x'");
            assert!(names.contains(&"y"), "Should suggest 'y'");
        }
    }

    #[test]
    fn test_completions_with_scope() {
        // const x = 1;
        // function foo() {
        //   const y = 2;
        //   |  <- cursor here (should see both x and y)
        // }
        let source = "const x = 1;\nfunction foo() {\n  const y = 2;\n  \n}";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        let arena = parser.get_arena();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(arena, root);

        let line_map = LineMap::build(source);

        // Position inside the function (line 3, column 2)
        let position = Position::new(3, 2);

        let completions = Completions::new(arena, &binder, &line_map, source);
        let items = completions.get_completions(root, position);

        assert!(items.is_some(), "Should have completions");

        if let Some(items) = items {
            let names: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

            // Should see both x (outer scope) and y (inner scope)
            assert!(names.contains(&"x"), "Should suggest 'x' from outer scope");
            assert!(names.contains(&"y"), "Should suggest 'y' from inner scope");
            assert!(names.contains(&"foo"), "Should suggest 'foo' (the function itself)");
        }
    }

    #[test]
    fn test_completions_shadowing() {
        // const x = 1;
        // function foo() {
        //   const x = 2;
        //   |  <- cursor here (should see inner x, not outer x)
        // }
        let source = "const x = 1;\nfunction foo() {\n  const x = 2;\n  \n}";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        let arena = parser.get_arena();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(arena, root);

        let line_map = LineMap::build(source);

        // Position inside the function (line 3, column 2)
        let position = Position::new(3, 2);

        let completions = Completions::new(arena, &binder, &line_map, source);
        let items = completions.get_completions(root, position);

        assert!(items.is_some(), "Should have completions");

        if let Some(items) = items {
            let names: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

            // Should only suggest 'x' once (the inner one shadows the outer one)
            let x_count = names.iter().filter(|&&n| n == "x").count();
            assert_eq!(x_count, 1, "Should suggest 'x' only once (inner shadows outer)");
        }
    }
}
