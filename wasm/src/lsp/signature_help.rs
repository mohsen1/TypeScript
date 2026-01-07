//! Signature Help implementation for LSP.
//!
//! Provides function signature information and active parameter highlighting
//! when typing arguments in a call expression.

use crate::parser::thin_node::{ThinNodeArena, CallExprData};
use crate::parser::{NodeIndex, NodeList, syntax_kind_ext};
use crate::thin_binder::ThinBinderState;
use crate::solver::{TypeInterner, TypeId, TypeKey, FunctionShape};
use crate::lsp::position::{Position, LineMap};
use crate::lsp::utils::find_node_at_offset;
use crate::lsp::resolver::{ScopeCache, ScopeCacheStats};
use crate::lsp::jsdoc::{jsdoc_for_node, parse_jsdoc, ParsedJsdoc};
use crate::thin_checker::ThinCheckerState;
use crate::scanner::SyntaxKind;

/// Represents a parameter in a signature.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParameterInformation {
    /// The label of this parameter (e.g., "x: number")
    pub label: String,
    /// The documentation for this parameter
    pub documentation: Option<String>,
}

/// Represents a single signature (overload).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignatureInformation {
    /// The label of the signature (e.g., "add(x: number, y: number): number")
    pub label: String,
    /// The documentation for this signature
    pub documentation: Option<String>,
    /// The parameters of this signature
    pub parameters: Vec<ParameterInformation>,
}

/// The response for a signature help request.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignatureHelp {
    /// One or more signatures (for overloads)
    pub signatures: Vec<SignatureInformation>,
    /// The active signature (usually 0, or based on best match)
    pub active_signature: u32,
    /// The active parameter index based on cursor position
    pub active_parameter: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CallKind {
    Call,
    New,
}

struct SignatureCandidate {
    info: SignatureInformation,
    required_params: usize,
    total_params: usize,
    has_rest: bool,
    param_names: Vec<Option<String>>,
}

struct SignatureDocCandidate {
    doc: ParsedJsdoc,
    required_params: usize,
    total_params: usize,
    has_rest: bool,
}

struct SignatureDocs {
    candidates: Vec<SignatureDocCandidate>,
    fallback: Option<ParsedJsdoc>,
}

impl SignatureDocs {
    fn is_empty(&self) -> bool {
        self.candidates.is_empty() && self.fallback.is_none()
    }
}

pub struct SignatureHelpProvider<'a> {
    arena: &'a ThinNodeArena,
    binder: &'a ThinBinderState,
    line_map: &'a LineMap,
    interner: &'a TypeInterner,
    source_text: &'a str,
    file_name: String,
}

impl<'a> SignatureHelpProvider<'a> {
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

    /// Get signature help at the given position.
    ///
    /// # Arguments
    /// * `root` - The root node of the AST
    /// * `position` - The cursor position
    /// * `type_cache` - Mutable reference to the persistent type cache (for performance)
    pub fn get_signature_help(
        &self,
        root: NodeIndex,
        position: Position,
        type_cache: &mut Option<crate::checker::TypeCache>,
    ) -> Option<SignatureHelp> {
        self.get_signature_help_internal(root, position, type_cache, None, None)
    }

    pub fn get_signature_help_with_scope_cache(
        &self,
        root: NodeIndex,
        position: Position,
        type_cache: &mut Option<crate::checker::TypeCache>,
        scope_cache: &mut ScopeCache,
        scope_stats: Option<&mut ScopeCacheStats>,
    ) -> Option<SignatureHelp> {
        self.get_signature_help_internal(root, position, type_cache, Some(scope_cache), scope_stats)
    }

    fn get_signature_help_internal(
        &self,
        root: NodeIndex,
        position: Position,
        type_cache: &mut Option<crate::checker::TypeCache>,
        scope_cache: Option<&mut ScopeCache>,
        mut scope_stats: Option<&mut ScopeCacheStats>,
    ) -> Option<SignatureHelp> {
        let offset = self.line_map.position_to_offset(position, self.source_text)?;

        // 1. Find the deepest node at the cursor
        let leaf_node = find_node_at_offset(self.arena, offset);

        // 2. Walk up to find the nearest CallExpression or NewExpression
        let (call_node_idx, call_expr, call_kind) = self.find_containing_call(leaf_node)?;

        // 3. Determine active parameter by counting commas
        let active_parameter = self.determine_active_parameter(call_node_idx, call_expr, offset);

        // 4. Resolve the symbol being called using ScopeWalker
        let mut walker = crate::lsp::resolver::ScopeWalker::new(self.arena, self.binder);
        let symbol_id = if let Some(scope_cache) = scope_cache {
            walker.resolve_node_cached(root, call_expr.expression, scope_cache, scope_stats.as_deref_mut())?
        } else {
            walker.resolve_node(root, call_expr.expression)?
        };

        // 5. Create checker with persistent cache if available
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

        let callee_type = checker.get_type_of_symbol(symbol_id);

        // 6. Extract signatures from the type
        let mut signatures = self.get_signatures_from_type(callee_type, &checker, call_kind);

        if let Some(docs) = self.signature_documentation_for_symbol(root, symbol_id) {
            self.apply_signature_docs(&mut signatures, &docs);
        }

        // Extract and save the updated cache for future queries
        *type_cache = Some(checker.extract_cache());

        if signatures.is_empty() {
            return None;
        }

        let arg_count = call_expr
            .arguments
            .as_ref()
            .map(|args| args.nodes.len())
            .unwrap_or(0);
        let active_signature = self.select_active_signature(&signatures, arg_count, active_parameter);

        Some(SignatureHelp {
            signatures: signatures.into_iter().map(|sig| sig.info).collect(),
            active_signature,
            active_parameter,
        })
    }

    /// Walk up the AST to find the call expression containing the cursor.
    fn find_containing_call(&self, start_node: NodeIndex) -> Option<(NodeIndex, &'a CallExprData, CallKind)> {
        let mut current = start_node;

        // Safety limit to prevent infinite loops
        let mut depth = 0;
        while !current.is_none() && depth < 100 {
            if let Some(node) = self.arena.get(current) {
                if node.kind == syntax_kind_ext::CALL_EXPRESSION
                    || node.kind == syntax_kind_ext::NEW_EXPRESSION {
                    if let Some(data) = self.arena.get_call_expr(node) {
                        let kind = if node.kind == syntax_kind_ext::NEW_EXPRESSION {
                            CallKind::New
                        } else {
                            CallKind::Call
                        };
                        return Some((current, data, kind));
                    }
                }

                // Move up to parent
                if let Some(extended) = self.arena.get_extended(current) {
                    current = extended.parent;
                } else {
                    break;
                }
            } else {
                break;
            }
            depth += 1;
        }

        None
    }

    /// Determine active parameter by scanning for commas, respecting nesting.
    /// This is more robust than AST analysis for incomplete code.
    fn determine_active_parameter(&self, call_idx: NodeIndex, data: &CallExprData, cursor_offset: u32) -> u32 {
        // Use AST-based approach instead of token scanning to handle edge cases:
        // - Generic type arguments with angle brackets: Set<string, number>
        // - Nested calls: foo(bar(x, y), z)
        // - Complex expressions with comparison operators: a < b

        // If there are no arguments, return 0
        let Some(ref args) = data.arguments else {
            return 0;
        };

        // Check if cursor is before the first argument
        if args.nodes.is_empty() {
            return 0;
        }

        // Find which argument contains or precedes the cursor
        for (index, &arg_idx) in args.nodes.iter().enumerate() {
            let Some(arg_node) = self.arena.get(arg_idx) else {
                continue;
            };

            // If cursor is before this argument's start, we're between args
            // Treat it as the next argument.
            if cursor_offset < arg_node.pos {
                return index as u32;
            }

            // If cursor is within this argument's range, return this index
            if cursor_offset >= arg_node.pos && cursor_offset < arg_node.end {
                return index as u32;
            }
        }

        // Cursor is after all arguments - return the last argument index
        (args.nodes.len().saturating_sub(1)) as u32
    }

    /// Extract signature information from a TypeId.
    fn get_signatures_from_type(
        &self,
        type_id: TypeId,
        checker: &ThinCheckerState,
        call_kind: CallKind,
    ) -> Vec<SignatureCandidate> {
        let key = match self.interner.lookup(type_id) {
            Some(k) => k,
            None => return vec![],
        };

        match key {
            // Single function signature
            TypeKey::Function(shape_id) => {
                let shape = self.interner.function_shape(shape_id);
                vec![self.signature_candidate(&shape, checker, false)]
            }
            // Overloaded signatures
            TypeKey::Callable(shape_id) => {
                let shape = self.interner.callable_shape(shape_id);
                let mut sigs = Vec::new();
                let include_call = call_kind == CallKind::Call || shape.construct_signatures.is_empty();
                let include_construct = call_kind == CallKind::New || shape.call_signatures.is_empty();

                if include_call {
                    // Add call signatures
                    for sig in &shape.call_signatures {
                        // Convert CallSignature to FunctionShape for formatting
                        let func_shape = FunctionShape {
                            type_params: sig.type_params.clone(),
                            params: sig.params.clone(),
                            this_type: sig.this_type,
                            return_type: sig.return_type,
                            type_predicate: sig.type_predicate.clone(),
                            is_constructor: false,
                        };
                        sigs.push(self.signature_candidate(&func_shape, checker, false));
                    }
                }
                if include_construct {
                    // Add construct signatures
                    for sig in &shape.construct_signatures {
                        let func_shape = FunctionShape {
                            type_params: sig.type_params.clone(),
                            params: sig.params.clone(),
                            this_type: sig.this_type,
                            return_type: sig.return_type,
                            type_predicate: sig.type_predicate.clone(),
                            is_constructor: true,
                        };
                        sigs.push(self.signature_candidate(&func_shape, checker, true));
                    }
                }
                sigs
            }
            // Union of functions
            TypeKey::Union(members) => {
                let members = self.interner.type_list(members);
                let mut sigs = Vec::new();
                for &member in members.iter() {
                    sigs.extend(self.get_signatures_from_type(member, checker, call_kind));
                }
                sigs
            }
            _ => vec![],
        }
    }

    /// Format a FunctionShape into SignatureInformation
    fn format_signature(&self, shape: &FunctionShape, checker: &ThinCheckerState, is_constructor: bool) -> SignatureInformation {
        let mut parameters = Vec::new();

        // 1. Parameters
        let mut param_labels = Vec::new();
        if let Some(this_type) = shape.this_type {
            param_labels.push(format!("this: {}", checker.format_type(this_type)));
        }

        for param in &shape.params {

            let name = param.name
                .map(|atom| checker.ctx.types.resolve_atom(atom))
                .unwrap_or_else(|| "arg".to_string());
            let type_str = checker.format_type(param.type_id);
            let optional = if param.optional { "?" } else { "" };
            let rest = if param.rest { "..." } else { "" };

            let param_label = format!("{}{}{}: {}", rest, name, optional, type_str);
            parameters.push(ParameterInformation {
                label: param_label.clone(),
                documentation: None,
            });

            param_labels.push(param_label);
        }

        // 3. Return Type
        let return_type_str = checker.format_type(shape.return_type);
        let prefix = if is_constructor { "new (" } else { "(" };
        let label = format!("{}{}): {}", prefix, param_labels.join(", "), return_type_str);

        SignatureInformation {
            label,
            documentation: None,
            parameters,
        }
    }

    fn signature_candidate(
        &self,
        shape: &FunctionShape,
        checker: &ThinCheckerState,
        is_constructor: bool,
    ) -> SignatureCandidate {
        let (required_params, total_params, has_rest) = self.signature_meta(&shape.params);
        let param_names = shape
            .params
            .iter()
            .map(|param| param.name.map(|atom| checker.ctx.types.resolve_atom(atom)))
            .collect();
        SignatureCandidate {
            info: self.format_signature(shape, checker, is_constructor),
            required_params,
            total_params,
            has_rest,
            param_names,
        }
    }

    fn signature_meta(&self, params: &[crate::solver::ParamInfo]) -> (usize, usize, bool) {
        let required_params = params.iter().filter(|param| !param.optional && !param.rest).count();
        let total_params = params.len();
        let has_rest = params.iter().any(|param| param.rest);
        (required_params, total_params, has_rest)
    }

    fn select_active_signature(
        &self,
        signatures: &[SignatureCandidate],
        arg_count: usize,
        active_parameter: u32,
    ) -> u32 {
        if signatures.is_empty() {
            return 0;
        }

        let desired = if arg_count == 0 {
            0
        } else {
            arg_count.max(active_parameter as usize + 1)
        };

        let mut best_idx = 0usize;
        let mut best_score = usize::MAX;
        let mut best_rest_penalty = usize::MAX;
        let mut best_total_params = usize::MAX;

        for (idx, sig) in signatures.iter().enumerate() {
            let min_params = sig.required_params;
            let max_params = if sig.has_rest { usize::MAX } else { sig.total_params };
            let score = if desired < min_params {
                min_params - desired
            } else if desired > max_params {
                desired - max_params
            } else {
                0
            };
            let rest_penalty = if sig.has_rest { 1 } else { 0 };

            if score < best_score
                || (score == best_score && rest_penalty < best_rest_penalty)
                || (score == best_score
                    && rest_penalty == best_rest_penalty
                    && sig.total_params < best_total_params)
            {
                best_idx = idx;
                best_score = score;
                best_rest_penalty = rest_penalty;
                best_total_params = sig.total_params;
            }
        }

        best_idx as u32
    }

    fn apply_signature_docs(&self, signatures: &mut [SignatureCandidate], docs: &SignatureDocs) {
        if signatures.is_empty() || docs.is_empty() {
            return;
        }

        if docs.candidates.len() == 1 {
            let doc = &docs.candidates[0].doc;
            for sig in signatures {
                self.apply_jsdoc_to_signature(sig, doc, true);
            }
            return;
        }

        if docs.candidates.is_empty() {
            if let Some(fallback) = docs.fallback.as_ref() {
                for sig in signatures {
                    self.apply_jsdoc_to_signature(sig, fallback, true);
                }
            }
            return;
        }

        let mut used = vec![false; docs.candidates.len()];
        for sig in signatures {
            if let Some(idx) = Self::match_doc_candidate(sig, &docs.candidates, &mut used) {
                let doc = &docs.candidates[idx].doc;
                self.apply_jsdoc_to_signature(sig, doc, true);
            } else if let Some(fallback) = docs.fallback.as_ref() {
                self.apply_jsdoc_to_signature(sig, fallback, false);
            }
        }
    }

    fn apply_jsdoc_to_signature(&self, sig: &mut SignatureCandidate, parsed: &ParsedJsdoc, overwrite: bool) {
        if overwrite || sig.info.documentation.is_none() {
            sig.info.documentation = parsed.summary.clone();
        }

        for (idx, name) in sig.param_names.iter().enumerate() {
            let Some(name) = name else { continue; };
            let Some(param_doc) = parsed.params.get(name) else { continue; };
            if let Some(param_info) = sig.info.parameters.get_mut(idx) {
                if overwrite || param_info.documentation.is_none() {
                    param_info.documentation = Some(param_doc.clone());
                }
            }
        }
    }

    fn match_doc_candidate(
        sig: &SignatureCandidate,
        candidates: &[SignatureDocCandidate],
        used: &mut [bool],
    ) -> Option<usize> {
        for (idx, candidate) in candidates.iter().enumerate() {
            if used[idx] {
                continue;
            }
            if candidate.required_params == sig.required_params
                && candidate.total_params == sig.total_params
                && candidate.has_rest == sig.has_rest
            {
                used[idx] = true;
                return Some(idx);
            }
        }
        None
    }

    fn signature_documentation_for_symbol(
        &self,
        root: NodeIndex,
        symbol_id: crate::binder::SymbolId,
    ) -> Option<SignatureDocs> {
        let symbol = self.binder.get_symbol(symbol_id)?;
        let mut decls = symbol.declarations.clone();
        if !symbol.value_declaration.is_none() && !decls.contains(&symbol.value_declaration) {
            decls.insert(0, symbol.value_declaration);
        }

        let mut candidates = Vec::new();
        let mut fallback = None;

        for decl in decls {
            if decl.is_none() {
                continue;
            }
            let doc = jsdoc_for_node(self.arena, root, decl, self.source_text);
            if doc.is_empty() {
                continue;
            }
            let parsed = parse_jsdoc(&doc);
            if parsed.is_empty() {
                continue;
            }

            if let Some((required_params, total_params, has_rest)) = self.signature_meta_from_decl(decl) {
                candidates.push(SignatureDocCandidate {
                    doc: parsed,
                    required_params,
                    total_params,
                    has_rest,
                });
            } else if fallback.is_none() {
                fallback = Some(parsed);
            }
        }

        let docs = SignatureDocs { candidates, fallback };
        if docs.is_empty() {
            None
        } else {
            Some(docs)
        }
    }

    fn signature_meta_from_decl(&self, decl: NodeIndex) -> Option<(usize, usize, bool)> {
        let node = self.arena.get(decl)?;
        if let Some(func) = self.arena.get_function(node) {
            return self.signature_meta_from_params(&func.parameters);
        }
        if let Some(method) = self.arena.get_method_decl(node) {
            return self.signature_meta_from_params(&method.parameters);
        }
        if let Some(ctor) = self.arena.get_constructor(node) {
            return self.signature_meta_from_params(&ctor.parameters);
        }
        None
    }

    fn signature_meta_from_params(&self, params: &NodeList) -> Option<(usize, usize, bool)> {
        let mut required_params = 0;
        let mut total_params = 0;
        let mut has_rest = false;

        for &param_idx in params.nodes.iter() {
            let Some(param_node) = self.arena.get(param_idx) else { continue; };
            let Some(param_data) = self.arena.get_parameter(param_node) else { continue; };
            if let Some(name_node) = self.arena.get(param_data.name) {
                if name_node.kind == SyntaxKind::ThisKeyword as u16 {
                    continue;
                }
            }

            total_params += 1;
            if param_data.dot_dot_dot_token {
                has_rest = true;
                continue;
            }
            if !param_data.question_token && param_data.initializer.is_none() {
                required_params += 1;
            }
        }

        Some((required_params, total_params, has_rest))
    }

}

#[cfg(test)]
mod signature_help_tests {
    use super::*;
    use crate::thin_parser::ThinParserState;
    use crate::thin_binder::ThinBinderState;
    use crate::solver::TypeInterner;
    use crate::lsp::position::LineMap;

    #[test]
    fn test_signature_help_simple() {
        // function add(x: number, y: number): number { return x + y; }
        // add(1, 2|);
        let source = "function add(x: number, y: number): number { return x + y; }\nadd(1, 2);";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let interner = TypeInterner::new();
        let line_map = LineMap::build(source);

        let provider = SignatureHelpProvider::new(
            parser.get_arena(),
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string()
        );

        // Position at the second argument '2' (line 1, column 7)
        let pos = Position::new(1, 7);
        let mut cache = None;
        let help = provider.get_signature_help(root, pos, &mut cache);

        assert!(help.is_some(), "Should find signature help");

        if let Some(h) = help {
            assert_eq!(h.active_parameter, 1, "Should be on second parameter");
            assert!(!h.signatures.is_empty(), "Should have signatures");
            // Note: The label format depends on how ThinChecker resolves types
            // For a simple function it may not include the full signature
        }
    }

    #[test]
    fn test_signature_help_no_call() {
        let source = "const x = 42;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let interner = TypeInterner::new();
        let line_map = LineMap::build(source);

        let provider = SignatureHelpProvider::new(
            parser.get_arena(),
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string()
        );

        // Position not in a call
        let pos = Position::new(0, 5);
        let mut cache = None;
        let help = provider.get_signature_help(root, pos, &mut cache);

        assert!(help.is_none(), "Should not find signature help outside call");
    }

    #[test]
    fn test_signature_help_first_arg() {
        // function foo(a: string): void {}
        // foo(|);
        let source = "function foo(a: string): void {}\nfoo();";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let interner = TypeInterner::new();
        let line_map = LineMap::build(source);

        let provider = SignatureHelpProvider::new(
            parser.get_arena(),
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string()
        );

        // Position inside the call (line 1, column 4)
        let pos = Position::new(1, 4);
        let mut cache = None;
        let help = provider.get_signature_help(root, pos, &mut cache);

        assert!(help.is_some(), "Should find signature help");

        if let Some(h) = help {
            assert_eq!(h.active_parameter, 0, "Should be on first parameter");
        }
    }

    #[test]
    fn test_signature_help_between_arguments() {
        // Test edge case: cursor between arguments (after comma, before next arg)
        // function process(a: any, b: number, c: string): void {}
        // process(1, |2, 3);
        //          ^ cursor here should be on parameter 1
        let source = "function process(a: any, b: number, c: string): void {}\nprocess(1, 2, 3);";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let interner = TypeInterner::new();
        let line_map = LineMap::build(source);

        let provider = SignatureHelpProvider::new(
            parser.get_arena(),
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string()
        );

        // Test cursor at first argument
        let pos1 = Position::new(1, 8); // At "1"
        let mut cache = None;
        let help1 = provider.get_signature_help(root, pos1, &mut cache);
        if let Some(h) = help1 {
            assert_eq!(h.active_parameter, 0, "Should be on first parameter");
        }

        // Test cursor at second argument
        let pos2 = Position::new(1, 11); // At "2"
        let help2 = provider.get_signature_help(root, pos2, &mut cache);
        if let Some(h) = help2 {
            assert_eq!(h.active_parameter, 1, "Should be on second parameter");
        }

        // Test cursor between comma and second argument
        let pos_between = Position::new(1, 10); // Between "," and "2"
        let help_between = provider.get_signature_help(root, pos_between, &mut cache);
        if let Some(h) = help_between {
            assert_eq!(h.active_parameter, 1, "Should be on second parameter");
        }

        // Test cursor at third argument
        let pos3 = Position::new(1, 14); // At "3"
        let help3 = provider.get_signature_help(root, pos3, &mut cache);
        if let Some(h) = help3 {
            assert_eq!(h.active_parameter, 2, "Should be on third parameter");
        }
    }

    #[test]
    fn test_signature_help_overload_selection() {
        let source = "interface Fn {\n  (a: number): void;\n  (a: number, b: string): void;\n}\ndeclare const fn: Fn;\nfn(1);\nfn(1, \"x\");";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let interner = TypeInterner::new();
        let line_map = LineMap::build(source);

        let provider = SignatureHelpProvider::new(
            parser.get_arena(),
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string(),
        );

        let mut cache = None;
        let pos_first = Position::new(5, 3); // At "1"
        let help_first = provider.get_signature_help(root, pos_first, &mut cache);
        assert!(help_first.is_some(), "Should find signature help for first call");
        let first = help_first.unwrap();
        assert!(first.signatures.len() >= 2, "Expected overload signatures");
        let first_active = &first.signatures[first.active_signature as usize];
        assert!(
            !first_active.label.contains("b: string"),
            "First call should select single-arg overload"
        );

        let pos_second = Position::new(6, 6); // At "\"x\""
        let help_second = provider.get_signature_help(root, pos_second, &mut cache);
        assert!(help_second.is_some(), "Should find signature help for second call");
        let second = help_second.unwrap();
        assert!(second.signatures.len() >= 2, "Expected overload signatures");
        let second_active = &second.signatures[second.active_signature as usize];
        assert!(
            second_active.label.contains("b: string"),
            "Second call should select two-arg overload"
        );
    }

    #[test]
    fn test_signature_help_new_overload_selection() {
        let source = "interface Ctor {\n  new (a: number): Foo;\n  new (a: number, b: string): Foo;\n}\nclass Foo {}\ndeclare const Ctor: Ctor;\nnew Ctor(1);\nnew Ctor(1, \"x\");";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let interner = TypeInterner::new();
        let line_map = LineMap::build(source);

        let provider = SignatureHelpProvider::new(
            parser.get_arena(),
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string(),
        );

        let mut cache = None;
        let pos_first = Position::new(6, 9); // At "1"
        let help_first = provider.get_signature_help(root, pos_first, &mut cache);
        assert!(help_first.is_some(), "Should find signature help for first new");
        let first = help_first.unwrap();
        assert!(!first.signatures.is_empty(), "Expected constructor signatures");
        let first_active = &first.signatures[first.active_signature as usize];
        assert!(
            first_active.label.starts_with("new ("),
            "Constructor signatures should use new() label"
        );
        assert!(
            !first_active.label.contains("b: string"),
            "First new should select single-arg overload"
        );

        let pos_second = Position::new(7, 13); // At "x"
        let help_second = provider.get_signature_help(root, pos_second, &mut cache);
        assert!(help_second.is_some(), "Should find signature help for second new");
        let second = help_second.unwrap();
        assert!(!second.signatures.is_empty(), "Expected constructor signatures");
        let second_active = &second.signatures[second.active_signature as usize];
        assert!(
            second_active.label.contains("b: string"),
            "Second new should select two-arg overload"
        );
    }

    #[test]
    fn test_signature_help_includes_jsdoc() {
        let source = "/** Adds two numbers. */\nfunction add(a: number, b: number): number { return a + b; }\nadd(1, 2);";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let interner = TypeInterner::new();
        let line_map = LineMap::build(source);

        let provider = SignatureHelpProvider::new(
            parser.get_arena(),
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string(),
        );

        let pos = Position::new(2, 6); // At "1"
        let mut cache = None;
        let help = provider.get_signature_help(root, pos, &mut cache);
        assert!(help.is_some(), "Should find signature help");

        let help = help.unwrap();
        assert!(!help.signatures.is_empty(), "Should have signatures");
        let doc = help.signatures[help.active_signature as usize]
            .documentation
            .clone()
            .unwrap_or_default();
        assert_eq!(doc, "Adds two numbers.");
    }

    #[test]
    fn test_signature_help_param_docs() {
        let source = "/**\n * Adds two numbers.\n * @param a First number.\n * @param b Second number.\n */\nfunction add(a: number, b: number): number { return a + b; }\nadd(1, 2);";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let interner = TypeInterner::new();
        let line_map = LineMap::build(source);

        let provider = SignatureHelpProvider::new(
            parser.get_arena(),
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string(),
        );

        let pos = Position::new(6, 6); // At "1"
        let mut cache = None;
        let help = provider.get_signature_help(root, pos, &mut cache);
        assert!(help.is_some(), "Should find signature help");

        let help = help.unwrap();
        let sig = &help.signatures[help.active_signature as usize];
        assert_eq!(sig.parameters.len(), 2);
        assert_eq!(
            sig.parameters[0].documentation.as_deref(),
            Some("First number.")
        );
        assert_eq!(
            sig.parameters[1].documentation.as_deref(),
            Some("Second number.")
        );
    }

    #[test]
    fn test_signature_help_overload_jsdoc() {
        let source = "/** One arg */\nfunction foo(a: number): void;\n/** Two args */\nfunction foo(a: number, b: string): void;\nfunction foo(a: number, b?: string): void {}\nfoo(1);\nfoo(1, \"x\");";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let interner = TypeInterner::new();
        let line_map = LineMap::build(source);

        let provider = SignatureHelpProvider::new(
            parser.get_arena(),
            &binder,
            &line_map,
            &interner,
            source,
            "test.ts".to_string(),
        );

        let mut cache = None;
        let pos_first = Position::new(5, 4); // At "1"
        let help_first = provider.get_signature_help(root, pos_first, &mut cache)
            .expect("Expected signature help for first call");
        let doc_first = help_first.signatures[help_first.active_signature as usize]
            .documentation
            .clone()
            .unwrap_or_default();
        assert_eq!(doc_first, "One arg");

        let pos_second = Position::new(6, 8); // At "x"
        let help_second = provider.get_signature_help(root, pos_second, &mut cache)
            .expect("Expected signature help for second call");
        let doc_second = help_second.signatures[help_second.active_signature as usize]
            .documentation
            .clone()
            .unwrap_or_default();
        assert_eq!(doc_second, "Two args");
    }
}
