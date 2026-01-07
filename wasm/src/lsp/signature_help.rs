//! Signature Help implementation for LSP.
//!
//! Provides function signature information and active parameter highlighting
//! when typing arguments in a call expression.

use crate::parser::thin_node::{ThinNodeArena, CallExprData};
use crate::parser::{NodeIndex, syntax_kind_ext};
use crate::thin_binder::ThinBinderState;
use crate::solver::{TypeInterner, TypeId, TypeKey, FunctionShape, CallableShape};
use crate::lsp::position::{Position, LineMap};
use crate::lsp::utils::find_node_at_offset;
use crate::thin_checker::ThinCheckerState;
use crate::scanner_impl::ScannerState;
use crate::scanner::SyntaxKind;
use crate::comments::{get_jsdoc_content, get_leading_comments_from_cache, is_jsdoc_comment};
use std::collections::HashMap;

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
        let offset = self.line_map.position_to_offset(position, self.source_text)?;

        // 1. Find the deepest node at the cursor
        let leaf_node = find_node_at_offset(self.arena, offset);

        // 2. Walk up to find the nearest CallExpression or NewExpression
        let (call_node_idx, call_expr, call_kind) = self.find_containing_call(leaf_node)?;

        // 3. Determine active parameter by counting commas
        let active_parameter = self.determine_active_parameter(call_node_idx, call_expr, offset);

        // 4. Resolve the symbol being called using ScopeWalker
        let mut walker = crate::lsp::resolver::ScopeWalker::new(self.arena, self.binder);
        let symbol_id = walker.resolve_node(root, call_expr.expression)?;

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

        if let Some(parsed) = self.signature_documentation_for_symbol(root, symbol_id) {
            for sig in &mut signatures {
                sig.info.documentation = parsed.summary.clone();
                for (idx, name) in sig.param_names.iter().enumerate() {
                    let Some(name) = name else { continue; };
                    let Some(param_doc) = parsed.params.get(name) else { continue; };
                    if let Some(param_info) = sig.info.parameters.get_mut(idx) {
                        param_info.documentation = Some(param_doc.clone());
                    }
                }
            }
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
            TypeKey::Function(shape) => {
                vec![self.signature_candidate(&shape, checker, false)]
            }
            // Overloaded signatures
            TypeKey::Callable(shape) => {
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
                let mut sigs = Vec::new();
                for member in members {
                    sigs.extend(self.get_signatures_from_type(member, checker, call_kind));
                }
                sigs
            }
            _ => vec![],
        }
    }

    /// Format a FunctionShape into SignatureInformation
    fn format_signature(&self, shape: &FunctionShape, checker: &ThinCheckerState, is_constructor: bool) -> SignatureInformation {
        let mut label_parts = Vec::new();
        let mut parameters = Vec::new();

        // 1. Prefix
        if is_constructor {
            label_parts.push("new (".to_string());
        } else {
            label_parts.push("(".to_string());
        }

        // 2. Parameters
        for (i, param) in shape.params.iter().enumerate() {
            if i > 0 {
                label_parts.push(", ".to_string());
            }

            let name = param.name
                .map(|atom| checker.ctx.types.resolve_atom(atom))
                .unwrap_or_else(|| "arg".to_string());
            let type_str = checker.format_type(param.type_id);
            let optional = if param.optional { "?" } else { "" };
            let rest = if param.rest { "..." } else { "" };

            let param_label = format!("{}{}{}: {}", rest, name, optional, type_str);
            parameters.push(ParameterInformation {
                label: param_label.clone(),
                documentation: None, // TODO: Extract JSDoc for params
            });

            label_parts.push(param_label);
        }

        // 3. Return Type
        let return_type_str = checker.format_type(shape.return_type);
        label_parts.push(format!("): {}", return_type_str));

        SignatureInformation {
            label: label_parts.join(""),
            documentation: None, // TODO: Extract JSDoc for function
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

    fn signature_documentation_for_symbol(
        &self,
        root: NodeIndex,
        symbol_id: crate::binder::SymbolId,
    ) -> Option<ParsedJsdoc> {
        let symbol = self.binder.get_symbol(symbol_id)?;
        let mut decls = Vec::new();
        if !symbol.value_declaration.is_none() {
            decls.push(symbol.value_declaration);
        }
        decls.extend(symbol.declarations.iter().copied());

        for decl in decls {
            if decl.is_none() {
                continue;
            }
            let doc = self.get_documentation(root, decl);
            if doc.is_empty() {
                continue;
            }
            let parsed = self.parse_jsdoc(&doc);
            if parsed.is_empty() {
                continue;
            }
            return Some(parsed);
        }

        None
    }

    /// Extract JSDoc comments preceding a node.
    /// Uses cached comment ranges from SourceFileData for O(log N) performance.
    fn get_documentation(&self, root: NodeIndex, node_idx: NodeIndex) -> String {
        let Some(node) = self.arena.get(node_idx) else { return String::new() };

        let comments = if let Some(root_node) = self.arena.get(root) {
            if let Some(sf_data) = self.arena.get_source_file(root_node) {
                &sf_data.comments
            } else {
                return String::new();
            }
        } else {
            return String::new();
        };

        let leading_comments = get_leading_comments_from_cache(comments, node.pos, self.source_text);
        let mut docs = Vec::new();

        for comment in leading_comments.iter().rev() {
            if is_jsdoc_comment(comment, self.source_text) {
                docs.push(get_jsdoc_content(comment, self.source_text));
            } else if !docs.is_empty() {
                break;
            }
        }

        docs.reverse();
        docs.join("\n\n")
    }

    fn parse_jsdoc(&self, doc: &str) -> ParsedJsdoc {
        let mut summary_lines = Vec::new();
        let mut params = HashMap::new();
        let mut current_param: Option<String> = None;
        let mut current_desc = String::new();
        let mut in_tags = false;

        for line in doc.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if !in_tags {
                    summary_lines.push(String::new());
                }
                continue;
            }

            if trimmed.starts_with('@') {
                in_tags = true;
                if let Some(name) = current_param.take() {
                    let desc = current_desc.trim().to_string();
                    if !desc.is_empty() {
                        params.insert(name, desc);
                    }
                    current_desc.clear();
                }

                if let Some((name, desc)) = self.parse_param_tag(trimmed) {
                    current_param = Some(name);
                    current_desc = desc;
                }
                continue;
            }

            if let Some(_) = current_param {
                if !current_desc.is_empty() {
                    current_desc.push(' ');
                }
                current_desc.push_str(trimmed);
            } else if !in_tags {
                summary_lines.push(trimmed.to_string());
            }
        }

        if let Some(name) = current_param {
            let desc = current_desc.trim().to_string();
            if !desc.is_empty() {
                params.insert(name, desc);
            }
        }

        let summary = summary_lines
            .join("\n")
            .trim()
            .to_string();

        ParsedJsdoc {
            summary: if summary.is_empty() { None } else { Some(summary) },
            params,
        }
    }

    fn parse_param_tag(&self, line: &str) -> Option<(String, String)> {
        let rest = line.strip_prefix("@param")?.trim();
        if rest.is_empty() {
            return None;
        }

        let rest = if rest.starts_with('{') {
            if let Some(end) = rest.find('}') {
                rest[end + 1..].trim()
            } else {
                rest
            }
        } else {
            rest
        };

        let mut parts = rest.splitn(2, char::is_whitespace);
        let name_raw = parts.next()?.trim();
        if name_raw.is_empty() {
            return None;
        }
        let desc = parts.next().unwrap_or("").trim().to_string();
        let name = self.normalize_param_name(name_raw);
        if name.is_empty() {
            return None;
        }
        Some((name, desc))
    }

    fn normalize_param_name(&self, name: &str) -> String {
        let trimmed = name.trim();
        let mut name = if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() > 2 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };
        if let Some(eq) = name.find('=') {
            name = &name[..eq];
        }
        name.trim().to_string()
    }
}

#[derive(Clone, Debug)]
struct ParsedJsdoc {
    summary: Option<String>,
    params: HashMap<String, String>,
}

impl ParsedJsdoc {
    fn is_empty(&self) -> bool {
        self.summary.is_none() && self.params.is_empty()
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
}
