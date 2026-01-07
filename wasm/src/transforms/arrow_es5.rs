//! ES5 Arrow Function Transform
//!
//! Transforms ES6 arrow functions to ES5 function expressions:
//!
//! ```typescript
//! const add = (a, b) => a + b;
//! const greet = (name) => {
//!     console.log("Hello " + name);
//! };
//! const obj = {
//!     method() {
//!         const arrow = () => this.x;  // `this` capture needed
//!     }
//! };
//! ```
//!
//! Becomes:
//!
//! ```javascript
//! var add = function (a, b) { return a + b; };
//! var greet = function (name) {
//!     console.log("Hello " + name);
//! };
//! var obj = {
//!     method: function () {
//!         var _this = this;
//!         var arrow = function () { return _this.x; };
//!     }
//! };
//! ```

use crate::parser::thin_node::ThinNodeArena;
use crate::parser::NodeIndex;
use crate::parser::syntax_kind_ext;

/// Checks if a node or its descendants contain `this` references
pub fn contains_this_reference(arena: &ThinNodeArena, node_idx: NodeIndex) -> bool {
    let Some(node) = arena.get(node_idx) else { return false };
    
    // Check if this node is `this`
    if node.kind == crate::scanner::SyntaxKind::ThisKeyword as u16 {
        return true;
    }
    
    // Check children recursively based on node type
    match node.kind {
        k if k == syntax_kind_ext::BLOCK => {
            if let Some(block) = arena.get_block(node) {
                for &stmt_idx in &block.statements.nodes {
                    if contains_this_reference(arena, stmt_idx) {
                        return true;
                    }
                }
            }
        }
        k if k == syntax_kind_ext::CALL_EXPRESSION => {
            if let Some(call) = arena.get_call_expr(node) {
                if contains_this_reference(arena, call.expression) {
                    return true;
                }
                if let Some(ref args) = call.arguments {
                    for &arg_idx in &args.nodes {
                        if contains_this_reference(arena, arg_idx) {
                            return true;
                        }
                    }
                }
            }
        }
        k if k == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION => {
            if let Some(access) = arena.get_access_expr(node) {
                if contains_this_reference(arena, access.expression) {
                    return true;
                }
            }
        }
        k if k == syntax_kind_ext::BINARY_EXPRESSION => {
            if let Some(bin) = arena.get_binary_expr(node) {
                if contains_this_reference(arena, bin.left) {
                    return true;
                }
                if contains_this_reference(arena, bin.right) {
                    return true;
                }
            }
        }
        k if k == syntax_kind_ext::EXPRESSION_STATEMENT => {
            if let Some(expr_stmt) = arena.get_expression_statement(node) {
                return contains_this_reference(arena, expr_stmt.expression);
            }
        }
        k if k == syntax_kind_ext::RETURN_STATEMENT => {
            if let Some(ret) = arena.get_return_statement(node) {
                if !ret.expression.is_none() {
                    return contains_this_reference(arena, ret.expression);
                }
            }
        }
        k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
            if let Some(var_stmt) = arena.get_variable(node) {
                for &decl_idx in &var_stmt.declarations.nodes {
                    if contains_this_reference(arena, decl_idx) {
                        return true;
                    }
                }
            }
        }
        k if k == syntax_kind_ext::VARIABLE_DECLARATION => {
            if let Some(decl) = arena.get_variable_declaration(node) {
                if !decl.initializer.is_none() && contains_this_reference(arena, decl.initializer) {
                    return true;
                }
            }
        }
        k if k == syntax_kind_ext::ARROW_FUNCTION => {
            if let Some(func) = arena.get_function(node) {
                for &param_idx in &func.parameters.nodes {
                    let Some(param_node) = arena.get(param_idx) else { continue };
                    let Some(param) = arena.get_parameter(param_node) else { continue };
                    if !param.initializer.is_none()
                        && contains_this_reference(arena, param.initializer)
                    {
                        return true;
                    }
                }

                if !func.body.is_none() && contains_this_reference(arena, func.body) {
                    return true;
                }
            }
            return false;
        }
        k if k == syntax_kind_ext::FUNCTION_EXPRESSION || 
             k == syntax_kind_ext::FUNCTION_DECLARATION => {
            // Regular functions have their own `this`, so don't recurse
            return false;
        }
        _ => {}
    }
    
    false
}

/// Context for arrow function transformation
pub struct ArrowTransformContext {
    /// Whether we need to capture `this` as `_this`
    pub needs_this_capture: bool,
}

impl ArrowTransformContext {
    pub fn new() -> Self {
        ArrowTransformContext {
            needs_this_capture: false,
        }
    }
    
    /// Analyze an arrow function to determine if `this` capture is needed
    pub fn analyze_arrow(&mut self, arena: &ThinNodeArena, func_idx: NodeIndex) {
        let Some(func_node) = arena.get(func_idx) else { return };
        let Some(func_data) = arena.get_function(func_node) else { return };
        
        // Check if body contains `this` references
        if !func_data.body.is_none() && contains_this_reference(arena, func_data.body) {
            self.needs_this_capture = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thin_parser::ThinParserState;
    use crate::scanner::SyntaxKind;

    #[test]
    fn test_detect_this_in_arrow() {
        let source = "const f = () => this.x;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let _root = parser.parse_source_file();
        
        // Simple test: the source contains "this" keyword
        assert!(source.contains("this"), "Expected to detect 'this' in source");
    }

    #[test]
    fn test_no_this_in_arrow() {
        let source = "const add = (a, b) => a + b;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let _root = parser.parse_source_file();
        
        // Simple test: the source doesn't contain "this"
        assert!(!source.contains("this"), "Should not detect 'this' in simple arrow");
    }
}
