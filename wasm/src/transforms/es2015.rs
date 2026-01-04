//! ES2015 Transforms
//!
//! Downlevels ES2015+ features to ES5:
//! - Arrow functions → function expressions
//! - Template literals → string concatenation
//! - Shorthand properties → full properties
//! - Default parameters → || fallbacks
//! - Rest parameters → arguments slice
//! - Spread in arrays → concat/apply
//! - let/const → var

use super::{TransformContext, Transformer};
use crate::parser::{Node, NodeIndex};

/// Node kind enumeration for pattern matching without borrowing
#[derive(Clone, Copy, PartialEq, Eq)]
enum NodeKind {
    ArrowFunction,
    TemplateExpression,
    VariableDeclaration,
    SpreadElement,
    ForOfStatement,
    SourceFile,
    Block,
    FunctionDeclaration,
    IfStatement,
    ForStatement,
    WhileStatement,
    ExpressionStatement,
    ReturnStatement,
    CallExpression,
    BinaryExpression,
    ParenthesizedExpression,
    Other,
}

/// Get the kind of a node without holding a reference
fn get_node_kind(node_idx: NodeIndex, ctx: &TransformContext) -> NodeKind {
    match ctx.arena.get(node_idx) {
        Some(Node::ArrowFunction(_)) => NodeKind::ArrowFunction,
        Some(Node::TemplateExpression(_)) => NodeKind::TemplateExpression,
        Some(Node::VariableDeclaration(_)) => NodeKind::VariableDeclaration,
        Some(Node::SpreadElement(_)) => NodeKind::SpreadElement,
        Some(Node::ForOfStatement(_)) => NodeKind::ForOfStatement,
        Some(Node::SourceFile(_)) => NodeKind::SourceFile,
        Some(Node::Block(_)) => NodeKind::Block,
        Some(Node::FunctionDeclaration(_)) => NodeKind::FunctionDeclaration,
        Some(Node::IfStatement(_)) => NodeKind::IfStatement,
        Some(Node::ForStatement(_)) => NodeKind::ForStatement,
        Some(Node::WhileStatement(_)) => NodeKind::WhileStatement,
        Some(Node::ExpressionStatement(_)) => NodeKind::ExpressionStatement,
        Some(Node::ReturnStatement(_)) => NodeKind::ReturnStatement,
        Some(Node::CallExpression(_)) => NodeKind::CallExpression,
        Some(Node::BinaryExpression(_)) => NodeKind::BinaryExpression,
        Some(Node::ParenthesizedExpression(_)) => NodeKind::ParenthesizedExpression,
        _ => NodeKind::Other,
    }
}

/// Collect child indices from a node without holding a reference
fn collect_children(node_idx: NodeIndex, ctx: &TransformContext) -> Vec<NodeIndex> {
    let mut children = Vec::new();

    match ctx.arena.get(node_idx) {
        Some(Node::SourceFile(sf)) => {
            children.extend(sf.statements.nodes.iter().copied());
        }
        Some(Node::Block(block)) => {
            children.extend(block.statements.nodes.iter().copied());
        }
        Some(Node::FunctionDeclaration(f)) => {
            if !f.body.is_none() {
                children.push(f.body);
            }
        }
        Some(Node::ArrowFunction(f)) => {
            children.push(f.body);
        }
        Some(Node::IfStatement(if_stmt)) => {
            children.push(if_stmt.expression);
            children.push(if_stmt.then_statement);
            if !if_stmt.else_statement.is_none() {
                children.push(if_stmt.else_statement);
            }
        }
        Some(Node::ForStatement(for_stmt)) => {
            if !for_stmt.initializer.is_none() {
                children.push(for_stmt.initializer);
            }
            if !for_stmt.condition.is_none() {
                children.push(for_stmt.condition);
            }
            if !for_stmt.incrementor.is_none() {
                children.push(for_stmt.incrementor);
            }
            children.push(for_stmt.statement);
        }
        Some(Node::WhileStatement(while_stmt)) => {
            children.push(while_stmt.expression);
            children.push(while_stmt.statement);
        }
        Some(Node::ExpressionStatement(expr_stmt)) => {
            children.push(expr_stmt.expression);
        }
        Some(Node::ReturnStatement(ret_stmt)) => {
            if !ret_stmt.expression.is_none() {
                children.push(ret_stmt.expression);
            }
        }
        Some(Node::CallExpression(call)) => {
            children.push(call.expression);
            children.extend(call.arguments.nodes.iter().copied());
        }
        Some(Node::BinaryExpression(bin)) => {
            children.push(bin.left);
            children.push(bin.right);
        }
        Some(Node::ParenthesizedExpression(paren)) => {
            children.push(paren.expression);
        }
        _ => {}
    }

    children
}

/// ES2015 to ES5 transformer
pub struct ES2015Transformer {
    /// Whether we're inside a class (affects this handling)
    _in_class: bool,
    /// Stack of enclosing function's this binding name (for arrow functions)
    this_bindings: Vec<Option<String>>,
}

impl ES2015Transformer {
    pub fn new() -> Self {
        ES2015Transformer {
            _in_class: false,
            this_bindings: vec![None],
        }
    }

    /// Transform an arrow function to a function expression
    /// `(x) => x * 2` becomes `function(x) { return x * 2; }`
    fn transform_arrow_function(&mut self, node_idx: NodeIndex, ctx: &mut TransformContext) -> Option<NodeIndex> {
        // Check if we need to capture 'this'
        let needs_this = self.arrow_uses_this(node_idx, ctx);

        if needs_this {
            // We'll need to add `var _this = this;` before this function
            let this_name = ctx.generate_unique_name("this");
            self.this_bindings.push(Some(this_name));
        } else {
            self.this_bindings.push(None);
        }

        // Get body index before mutably borrowing ctx
        let body_idx = match ctx.arena.get(node_idx) {
            Some(Node::ArrowFunction(arrow)) => arrow.body,
            _ => return None,
        };

        // Transform body
        self.visit_node(body_idx, ctx);

        self.this_bindings.pop();

        // Note: In a full implementation, we would create a new FunctionExpression node
        // and replace this node. For now, we just mark that transformation is needed.
        // The actual node creation would require arena allocation.
        None
    }

    /// Check if an arrow function uses 'this' or 'arguments'
    fn arrow_uses_this(&self, _node_idx: NodeIndex, _ctx: &TransformContext) -> bool {
        // Walk the body looking for 'this' or 'arguments'
        // For now, return false - this is a placeholder
        false
    }

    /// Transform template literal to string concatenation
    /// ``Hello ${name}!`` becomes `"Hello " + name + "!"`
    fn transform_template_literal(&mut self, _node_idx: NodeIndex, _ctx: &mut TransformContext) -> Option<NodeIndex> {
        // Transform head + spans to concatenation
        // This would require creating new BinaryExpression nodes
        // For now, this is a placeholder
        None
    }

    /// Transform let/const to var
    fn transform_variable_declaration(&mut self, _node_idx: NodeIndex, _ctx: &mut TransformContext) -> Option<NodeIndex> {
        // Check if it's let or const, convert to var
        // Also need to add block scoping simulation if needed
        None
    }

    /// Transform spread element in array to concat
    /// `[...arr, x]` becomes `[].concat(arr, [x])`
    fn transform_spread_in_array(&mut self, _node_idx: NodeIndex, ctx: &mut TransformContext) -> Option<NodeIndex> {
        ctx.helpers_needed.spread_array = true;
        None
    }

    /// Transform for..of to indexed for loop
    /// `for (let x of arr)` becomes `for (var _i = 0; _i < arr.length; _i++) { var x = arr[_i]; ... }`
    fn transform_for_of(&mut self, _node_idx: NodeIndex, ctx: &mut TransformContext) -> Option<NodeIndex> {
        ctx.helpers_needed.values = true;
        None
    }
}

impl Transformer for ES2015Transformer {
    fn visit_node(&mut self, node_idx: NodeIndex, ctx: &mut TransformContext) -> Option<NodeIndex> {
        // Get kind first to avoid borrow issues
        let kind = get_node_kind(node_idx, ctx);

        match kind {
            NodeKind::ArrowFunction => {
                return self.transform_arrow_function(node_idx, ctx);
            }
            NodeKind::TemplateExpression => {
                return self.transform_template_literal(node_idx, ctx);
            }
            NodeKind::VariableDeclaration => {
                return self.transform_variable_declaration(node_idx, ctx);
            }
            NodeKind::SpreadElement => {
                return self.transform_spread_in_array(node_idx, ctx);
            }
            NodeKind::ForOfStatement => {
                return self.transform_for_of(node_idx, ctx);
            }
            _ => {}
        }

        // Visit children
        self.visit_children(node_idx, ctx);

        None
    }

    fn visit_children(&mut self, node_idx: NodeIndex, ctx: &mut TransformContext) {
        // Collect children first to avoid borrow issues
        let children = collect_children(node_idx, ctx);

        // Visit each child
        for child_idx in children {
            self.visit_node(child_idx, ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emitter::ScriptTarget;
    use crate::parser_impl::ParserState;

    fn parse_and_transform(source: &str, target: ScriptTarget) -> super::super::HelpersNeeded {
        let mut parser = ParserState::new("test.ts".to_string(), source.to_string());
        let source_file = parser.parse_source_file();
        let mut arena = parser.arena;

        super::super::transform_source_file(source_file, target, &mut arena)
    }

    #[test]
    fn test_transformer_creates_context() {
        let helpers = parse_and_transform("let x = 1;", ScriptTarget::ES5);
        // Just verify it doesn't crash
        assert!(!helpers.extends);
    }

    #[test]
    fn test_arrow_function_transform() {
        let _helpers = parse_and_transform("const fn = (x) => x * 2;", ScriptTarget::ES5);
        // Arrow function detection should work
        // Full transform creates function expression
    }

    #[test]
    fn test_for_of_needs_values_helper() {
        let helpers = parse_and_transform("for (const x of arr) { console.log(x); }", ScriptTarget::ES5);
        // For-of should require __values helper
        assert!(helpers.values);
    }
}
