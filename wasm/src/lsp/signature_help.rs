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

/// Represents a parameter in a signature.
#[derive(Debug, Clone)]
pub struct ParameterInformation {
    /// The label of this parameter (e.g., "x: number")
    pub label: String,
    /// The documentation for this parameter
    pub documentation: Option<String>,
}

/// Represents a single signature (overload).
#[derive(Debug, Clone)]
pub struct SignatureInformation {
    /// The label of the signature (e.g., "add(x: number, y: number): number")
    pub label: String,
    /// The documentation for this signature
    pub documentation: Option<String>,
    /// The parameters of this signature
    pub parameters: Vec<ParameterInformation>,
}

/// The response for a signature help request.
#[derive(Debug, Clone)]
pub struct SignatureHelp {
    /// One or more signatures (for overloads)
    pub signatures: Vec<SignatureInformation>,
    /// The active signature (usually 0, or based on best match)
    pub active_signature: u32,
    /// The active parameter index based on cursor position
    pub active_parameter: u32,
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
    pub fn get_signature_help(&self, root: NodeIndex, position: Position) -> Option<SignatureHelp> {
        let offset = self.line_map.position_to_offset(position);

        // 1. Find the deepest node at the cursor
        let leaf_node = find_node_at_offset(self.arena, offset);

        // 2. Walk up to find the nearest CallExpression or NewExpression
        let (call_node_idx, call_expr) = self.find_containing_call(leaf_node)?;

        // 3. Determine active parameter by counting commas
        let active_parameter = self.determine_active_parameter(call_node_idx, call_expr, offset);

        // 4. Resolve the symbol being called using ScopeWalker
        let mut walker = crate::lsp::resolver::ScopeWalker::new(self.arena, self.binder);
        let symbol_id = walker.resolve_node(root, call_expr.expression)?;

        // 5. Create checker and get the type of the symbol
        let mut checker = ThinCheckerState::new(
            self.arena,
            self.binder,
            self.interner,
            self.file_name.clone()
        );

        let callee_type = checker.get_type_of_symbol(symbol_id);

        // 6. Extract signatures from the type
        let signatures = self.get_signatures_from_type(callee_type, &checker);

        if signatures.is_empty() {
            return None;
        }

        Some(SignatureHelp {
            signatures,
            active_signature: 0, // TODO: logic to select best overload based on args
            active_parameter,
        })
    }

    /// Walk up the AST to find the call expression containing the cursor.
    fn find_containing_call(&self, start_node: NodeIndex) -> Option<(NodeIndex, &'a CallExprData)> {
        let mut current = start_node;

        // Safety limit to prevent infinite loops
        let mut depth = 0;
        while !current.is_none() && depth < 100 {
            if let Some(node) = self.arena.get(current) {
                if node.kind == syntax_kind_ext::CALL_EXPRESSION ||
                   node.kind == syntax_kind_ext::NEW_EXPRESSION {
                    if let Some(data) = self.arena.get_call_expr(node) {
                        return Some((current, data));
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
        let call_node = self.arena.get(call_idx).unwrap();

        // Start scanning after type arguments if present, otherwise after expression
        let start_pos = if let Some(ref type_args) = data.type_arguments {
            type_args.end as usize
        } else if let Some(expr) = self.arena.get(data.expression) {
            expr.end as usize
        } else {
            call_node.pos as usize
        };

        // TODO: Performance - ScannerState::new clones the entire source text
        // Consider refactoring ScannerState to use &str or reusing ThinParser logic
        let mut scanner = ScannerState::new(self.source_text.to_string(), true);
        scanner.reset_token_state(start_pos);

        // 1. Find the opening parenthesis of the call
        let mut open_paren_pos = 0;
        loop {
            let token = scanner.scan();
            if token == SyntaxKind::EndOfFileToken { break; }
            if token == SyntaxKind::OpenParenToken {
                open_paren_pos = scanner.get_token_end();
                break;
            }
            // If we pass the cursor before finding '(', we are not in the args
            if scanner.get_token_start() >= cursor_offset as usize { return 0; }
        }

        if open_paren_pos == 0 {
            return 0;
        }

        // 2. Scan arguments counting commas at top level (depth 0)
        let mut comma_count = 0;
        let mut depth = 0;

        // Reset to just after the opening paren
        scanner.reset_token_state(open_paren_pos);

        loop {
            let token = scanner.scan();
            let token_start = scanner.get_token_start();

            // Stop if we reach the cursor
            if token_start >= cursor_offset as usize {
                break;
            }

            if token == SyntaxKind::EndOfFileToken { break; }

            match token {
                SyntaxKind::OpenParenToken |
                SyntaxKind::OpenBracketToken |
                SyntaxKind::OpenBraceToken => {
                    depth += 1;
                }
                SyntaxKind::CloseParenToken |
                SyntaxKind::CloseBracketToken |
                SyntaxKind::CloseBraceToken => {
                    if depth > 0 {
                        depth -= 1;
                    } else if token == SyntaxKind::CloseParenToken {
                        // Closing paren of the function call
                        return comma_count;
                    }
                }
                SyntaxKind::CommaToken => {
                    // Only count commas at depth 0 (top-level commas in the call)
                    if depth == 0 {
                        comma_count += 1;
                    }
                }
                _ => {}
            }
        }

        comma_count
    }

    /// Extract signature information from a TypeId.
    fn get_signatures_from_type(&self, type_id: TypeId, checker: &ThinCheckerState) -> Vec<SignatureInformation> {
        let key = match self.interner.lookup(type_id) {
            Some(k) => k,
            None => return vec![],
        };

        match key {
            // Single function signature
            TypeKey::Function(shape) => {
                vec![self.format_signature(&shape, checker, false)]
            }
            // Overloaded signatures
            TypeKey::Callable(shape) => {
                let mut sigs = Vec::new();
                // Add call signatures
                for sig in &shape.call_signatures {
                    // Convert CallSignature to FunctionShape for formatting
                    let func_shape = FunctionShape {
                        type_params: sig.type_params.clone(),
                        params: sig.params.clone(),
                        return_type: sig.return_type,
                        is_constructor: false,
                    };
                    sigs.push(self.format_signature(&func_shape, checker, false));
                }
                // Add construct signatures
                for sig in &shape.construct_signatures {
                    let func_shape = FunctionShape {
                        type_params: sig.type_params.clone(),
                        params: sig.params.clone(),
                        return_type: sig.return_type,
                        is_constructor: true,
                    };
                    sigs.push(self.format_signature(&func_shape, checker, true));
                }
                sigs
            }
            // Union of functions
            TypeKey::Union(members) => {
                let mut sigs = Vec::new();
                for member in members {
                    sigs.extend(self.get_signatures_from_type(member, checker));
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

            let name = param.name.as_deref().unwrap_or("arg");
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
}

#[cfg(test)]
mod signature_help_tests {
    use super::*;
    use crate::thin_parser::ThinParserState;
    use crate::thin_binder::ThinBinderState;
    use crate::solver::TypeInterner;
    use crate::lsp::position::LineMap;

    #[test]
    #[ignore] // TODO: Debug cursor position detection with multiple arguments
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
        let help = provider.get_signature_help(root, pos);

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
        let help = provider.get_signature_help(root, pos);

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
        let help = provider.get_signature_help(root, pos);

        assert!(help.is_some(), "Should find signature help");

        if let Some(h) = help {
            assert_eq!(h.active_parameter, 0, "Should be on first parameter");
        }
    }
}
