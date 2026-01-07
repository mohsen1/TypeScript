//! Completions implementation for LSP.
//!
//! Given a position in the source, provides completion suggestions for
//! identifiers that are visible at that position.

use rustc_hash::{FxHashMap, FxHashSet};
use std::borrow::Cow;

use crate::binder::SymbolId;
use crate::checker::TypeCache;
use crate::parser::thin_node::ThinNodeArena;
use crate::parser::NodeIndex;
use crate::solver::{
    apparent_primitive_members, ApparentMemberKind, IntrinsicKind, TypeId, TypeInterner, TypeKey,
};
use crate::thin_binder::ThinBinderState;
use crate::thin_checker::ThinCheckerState;
use crate::lsp::position::{Position, LineMap};
use crate::lsp::resolver::{ScopeCache, ScopeCacheStats, ScopeWalker};
use crate::lsp::utils::find_node_at_offset;
use crate::parser::syntax_kind_ext;

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
    interner: Option<&'a TypeInterner>,
    file_name: Option<String>,
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
            interner: None,
            file_name: None,
        }
    }

    /// Create a completions provider with type-aware member completion support.
    pub fn new_with_types(
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
            source_text,
            interner: Some(interner),
            file_name: Some(file_name),
        }
    }

    /// Get completion suggestions at the given position.
    ///
    /// Returns a list of completion items for identifiers visible at the cursor position.
    /// Returns None if no completions are available.
    pub fn get_completions(&self, root: NodeIndex, position: Position) -> Option<Vec<CompletionItem>> {
        self.get_completions_internal(root, position, None, None, None)
    }

    /// Get completion suggestions at the given position with a persistent type cache.
    pub fn get_completions_with_cache(
        &self,
        root: NodeIndex,
        position: Position,
        type_cache: &mut Option<TypeCache>,
    ) -> Option<Vec<CompletionItem>> {
        self.get_completions_internal(root, position, Some(type_cache), None, None)
    }

    pub fn get_completions_with_caches(
        &self,
        root: NodeIndex,
        position: Position,
        type_cache: &mut Option<TypeCache>,
        scope_cache: &mut ScopeCache,
        scope_stats: Option<&mut ScopeCacheStats>,
    ) -> Option<Vec<CompletionItem>> {
        self.get_completions_internal(root, position, Some(type_cache), Some(scope_cache), scope_stats)
    }

    fn get_completions_internal(
        &self,
        root: NodeIndex,
        position: Position,
        type_cache: Option<&mut Option<TypeCache>>,
        scope_cache: Option<&mut ScopeCache>,
        scope_stats: Option<&mut ScopeCacheStats>,
    ) -> Option<Vec<CompletionItem>> {
        // 1. Convert position to byte offset
        let offset = self.line_map.position_to_offset(position, self.source_text)?;

        // 2. Find the node at this offset (or use root if not found)
        let mut node_idx = find_node_at_offset(self.arena, offset);
        if node_idx.is_none() && offset > 0 {
            node_idx = find_node_at_offset(self.arena, offset - 1);
        }
        let node_idx = if node_idx.is_none() { root } else { node_idx };

        if let Some(expr_idx) = self.member_completion_target(node_idx, offset) {
            if let Some(items) = self.get_member_completions(expr_idx, type_cache) {
                return if items.is_empty() { None } else { Some(items) };
            }
        }

        // 3. Get the scope chain at this position
        let mut walker = ScopeWalker::new(self.arena, self.binder);
        let scope_chain = if let Some(scope_cache) = scope_cache {
            Cow::Borrowed(walker.get_scope_chain_cached(root, node_idx, scope_cache, scope_stats))
        } else {
            Cow::Owned(walker.get_scope_chain(root, node_idx))
        };

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

    fn member_completion_target(&self, node_idx: NodeIndex, offset: u32) -> Option<NodeIndex> {
        let mut current = node_idx;

        while !current.is_none() {
            let node = self.arena.get(current)?;
            if node.kind == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION {
                let access = self.arena.get_access_expr(node)?;
                let expr_node = self.arena.get(access.expression)?;
                if offset >= expr_node.end && offset <= node.end {
                    return Some(access.expression);
                }
            }

            let ext = self.arena.get_extended(current)?;
            current = ext.parent;
        }

        None
    }

    fn get_member_completions(
        &self,
        expr_idx: NodeIndex,
        type_cache: Option<&mut Option<TypeCache>>,
    ) -> Option<Vec<CompletionItem>> {
        let interner = self.interner?;
        let file_name = self.file_name.as_ref()?;

        let mut cache_ref = type_cache;
        let mut checker = if let Some(cache) = cache_ref.as_deref_mut() {
            if let Some(cache_value) = cache.take() {
                ThinCheckerState::with_cache(
                    self.arena,
                    self.binder,
                    interner,
                    file_name.clone(),
                    cache_value,
                )
            } else {
                ThinCheckerState::new(
                    self.arena,
                    self.binder,
                    interner,
                    file_name.clone(),
                )
            }
        } else {
            ThinCheckerState::new(
                self.arena,
                self.binder,
                interner,
                file_name.clone(),
            )
        };

        let type_id = checker.get_type_of_node(expr_idx);
        let mut visited = FxHashSet::default();
        let mut props: FxHashMap<String, PropertyCompletion> = FxHashMap::default();
        self.collect_properties_for_type(type_id, interner, &mut checker, &mut visited, &mut props);

        let mut items = Vec::new();
        for (name, info) in props {
            let kind = if info.is_method {
                CompletionItemKind::Method
            } else {
                CompletionItemKind::Property
            };
            let mut item = CompletionItem::new(name, kind);
            item = item.with_detail(checker.format_type(info.type_id));
            items.push(item);
        }

        items.sort_by(|a, b| a.label.cmp(&b.label));
        if let Some(cache) = cache_ref {
            *cache = Some(checker.extract_cache());
        }
        Some(items)
    }

    fn collect_properties_for_type(
        &self,
        type_id: TypeId,
        interner: &TypeInterner,
        checker: &mut ThinCheckerState,
        visited: &mut FxHashSet<TypeId>,
        props: &mut FxHashMap<String, PropertyCompletion>,
    ) {
        if !visited.insert(type_id) {
            return;
        }

        let key = match interner.lookup(type_id) {
            Some(key) => key,
            None => return,
        };

        match key {
            TypeKey::Object(properties) => {
                for prop in properties {
                    let name = interner.resolve_atom(prop.name);
                    self.add_property_completion(props, interner, name, prop.type_id, prop.is_method);
                }
            }
            TypeKey::ObjectWithIndex(shape) => {
                for prop in shape.properties {
                    let name = interner.resolve_atom(prop.name);
                    self.add_property_completion(props, interner, name, prop.type_id, prop.is_method);
                }
            }
            TypeKey::Union(members) | TypeKey::Intersection(members) => {
                for member in members {
                    self.collect_properties_for_type(member, interner, checker, visited, props);
                }
            }
            TypeKey::Ref(symbol_ref) => {
                let type_id = checker.get_type_of_symbol(SymbolId(symbol_ref.0));
                self.collect_properties_for_type(type_id, interner, checker, visited, props);
            }
            TypeKey::Application(app) => {
                self.collect_properties_for_type(app.base, interner, checker, visited, props);
            }
            TypeKey::Literal(literal) => {
                if let Some(kind) = self.literal_intrinsic_kind(&literal) {
                    self.collect_intrinsic_members(kind, interner, props);
                }
            }
            TypeKey::TemplateLiteral(_) => {
                self.collect_intrinsic_members(IntrinsicKind::String, interner, props);
            }
            TypeKey::Intrinsic(kind) => {
                self.collect_intrinsic_members(kind, interner, props);
            }
            _ => {}
        }
    }

    fn collect_intrinsic_members(
        &self,
        kind: IntrinsicKind,
        interner: &TypeInterner,
        props: &mut FxHashMap<String, PropertyCompletion>,
    ) {
        let members = apparent_primitive_members(interner, kind);
        for member in members {
            let type_id = match member.kind {
                ApparentMemberKind::Value(type_id) => type_id,
                ApparentMemberKind::Method(type_id) => type_id,
            };
            let is_method = matches!(member.kind, ApparentMemberKind::Method(_));
            self.add_property_completion(
                props,
                interner,
                member.name.to_string(),
                type_id,
                is_method,
            );
        }
    }

    fn literal_intrinsic_kind(&self, literal: &crate::solver::LiteralValue) -> Option<IntrinsicKind> {
        match literal {
            crate::solver::LiteralValue::String(_) => Some(IntrinsicKind::String),
            crate::solver::LiteralValue::Number(_) => Some(IntrinsicKind::Number),
            crate::solver::LiteralValue::Boolean(_) => Some(IntrinsicKind::Boolean),
            crate::solver::LiteralValue::BigInt(_) => Some(IntrinsicKind::Bigint),
        }
    }

    fn add_property_completion(
        &self,
        props: &mut FxHashMap<String, PropertyCompletion>,
        interner: &TypeInterner,
        name: String,
        type_id: TypeId,
        is_method: bool,
    ) {
        if let Some(existing) = props.get_mut(&name) {
            if existing.type_id != type_id {
                existing.type_id = interner.union(vec![existing.type_id, type_id]);
            }
            existing.is_method |= is_method;
        } else {
            props.insert(name, PropertyCompletion { type_id, is_method });
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PropertyCompletion {
    type_id: TypeId,
    is_method: bool,
}

#[cfg(test)]
mod completions_tests {
    use super::*;
    use crate::thin_parser::ThinParserState;
    use crate::thin_binder::ThinBinderState;
    use crate::lsp::position::LineMap;
    use crate::solver::TypeInterner;

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

    #[test]
    fn test_completions_member_object_literal() {
        let source = "const obj = { foo: 1, bar: \"hi\" };\nobj.";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        let arena = parser.get_arena();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(arena, root);

        let line_map = LineMap::build(source);
        let interner = TypeInterner::new();
        let completions = Completions::new_with_types(
            arena,
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string(),
        );

        let position = Position::new(1, 4);
        let mut cache = None;
        let items = completions.get_completions_with_cache(root, position, &mut cache);

        assert!(items.is_some(), "Should have member completions");
        let items = items.unwrap();
        let names: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

        assert!(names.contains(&"foo"), "Should suggest object member 'foo'");
        assert!(names.contains(&"bar"), "Should suggest object member 'bar'");
    }

    #[test]
    fn test_completions_member_string_literal() {
        let source = "const s = \"hello\";\ns.";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        let arena = parser.get_arena();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(arena, root);

        let line_map = LineMap::build(source);
        let interner = TypeInterner::new();
        let completions = Completions::new_with_types(
            arena,
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string(),
        );

        let position = Position::new(1, 2);
        let mut cache = None;
        let items = completions.get_completions_with_cache(root, position, &mut cache);

        assert!(items.is_some(), "Should have member completions");
        let items = items.unwrap();
        let names: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

        assert!(names.contains(&"length"), "Should suggest string member 'length'");
    }
}
