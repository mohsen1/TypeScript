//! Expression node definitions
//!
//! All TypeScript expression AST node types.

use super::arena::{AstArena, NodeId, Span};
use super::nodes::NodeKind;

/// Helper to create expression nodes in the arena
pub struct ExpressionBuilder<'a> {
    arena: &'a mut AstArena,
}

impl<'a> ExpressionBuilder<'a> {
    pub fn new(arena: &'a mut AstArena) -> Self {
        ExpressionBuilder { arena }
    }

    /// Create an Identifier
    pub fn identifier(&mut self, span: Span, name: &str) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::Identifier as u16, span);
        let string_id = self.arena.intern(name);
        self.arena.set_string(id, string_id);
        id
    }

    /// Create a PrivateIdentifier: #name
    pub fn private_identifier(&mut self, span: Span, name: &str) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::PrivateIdentifier as u16, span);
        let string_id = self.arena.intern(name);
        self.arena.set_string(id, string_id);
        id
    }

    /// Create a NumericLiteral
    pub fn numeric_literal(&mut self, span: Span, value: &str) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::NumericLiteral as u16, span);
        let string_id = self.arena.intern(value);
        self.arena.set_string(id, string_id);
        id
    }

    /// Create a BigIntLiteral
    pub fn bigint_literal(&mut self, span: Span, value: &str) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::BigIntLiteral as u16, span);
        let string_id = self.arena.intern(value);
        self.arena.set_string(id, string_id);
        id
    }

    /// Create a StringLiteral
    pub fn string_literal(&mut self, span: Span, value: &str) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::StringLiteral as u16, span);
        let string_id = self.arena.intern(value);
        self.arena.set_string(id, string_id);
        id
    }

    /// Create a RegexLiteral
    pub fn regex_literal(&mut self, span: Span, pattern: &str) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::RegexLiteral as u16, span);
        let string_id = self.arena.intern(pattern);
        self.arena.set_string(id, string_id);
        id
    }

    /// Create a NoSubstitutionTemplateLiteral
    pub fn no_substitution_template(&mut self, span: Span, text: &str) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::NoSubstitutionTemplate as u16, span);
        let string_id = self.arena.intern(text);
        self.arena.set_string(id, string_id);
        id
    }

    /// Create ThisKeyword
    pub fn this_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::ThisKeyword as u16, span)
    }

    /// Create SuperKeyword
    pub fn super_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::SuperKeyword as u16, span)
    }

    /// Create NullKeyword
    pub fn null_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::NullKeyword as u16, span)
    }

    /// Create TrueKeyword
    pub fn true_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::TrueKeyword as u16, span)
    }

    /// Create FalseKeyword
    pub fn false_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::FalseKeyword as u16, span)
    }

    /// Create an ArrayLiteralExpression: [elements]
    pub fn array_literal(&mut self, span: Span, elements: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ArrayLiteralExpression as u16, span);
        self.arena.add_children(id, elements);
        id
    }

    /// Create an ObjectLiteralExpression: { properties }
    pub fn object_literal(&mut self, span: Span, properties: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ObjectLiteralExpression as u16, span);
        self.arena.add_children(id, properties);
        id
    }

    /// Create a PropertyAccessExpression: expression.name
    pub fn property_access(&mut self, span: Span, expression: NodeId, name: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::PropertyAccessExpression as u16, span);
        self.arena.add_children(id, &[expression, name]);
        id
    }

    /// Create an ElementAccessExpression: expression[argument]
    pub fn element_access(&mut self, span: Span, expression: NodeId, argument: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ElementAccessExpression as u16, span);
        self.arena.add_children(id, &[expression, argument]);
        id
    }

    /// Create a CallExpression: expression(arguments)
    pub fn call_expression(
        &mut self,
        span: Span,
        expression: NodeId,
        type_arguments: Option<&[NodeId]>,
        arguments: &[NodeId],
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::CallExpression as u16, span);
        let mut children = vec![expression];
        if let Some(type_args) = type_arguments {
            // Store type argument count in extra
            self.arena.set_extra(id, type_args.len() as u32);
            children.extend_from_slice(type_args);
        }
        children.extend_from_slice(arguments);
        self.arena.add_children(id, &children);
        id
    }

    /// Create a NewExpression: new expression(arguments)
    pub fn new_expression(
        &mut self,
        span: Span,
        expression: NodeId,
        type_arguments: Option<&[NodeId]>,
        arguments: Option<&[NodeId]>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::NewExpression as u16, span);
        let mut children = vec![expression];
        if let Some(type_args) = type_arguments {
            self.arena.set_extra(id, type_args.len() as u32);
            children.extend_from_slice(type_args);
        }
        if let Some(args) = arguments {
            children.extend_from_slice(args);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a TaggedTemplateExpression: tag`template`
    pub fn tagged_template(&mut self, span: Span, tag: NodeId, template: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TaggedTemplateExpression as u16, span);
        self.arena.add_children(id, &[tag, template]);
        id
    }

    /// Create a TypeAssertionExpression: <type>expression
    pub fn type_assertion(&mut self, span: Span, type_node: NodeId, expression: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TypeAssertionExpression as u16, span);
        self.arena.add_children(id, &[type_node, expression]);
        id
    }

    /// Create a ParenthesizedExpression: (expression)
    pub fn parenthesized(&mut self, span: Span, expression: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ParenthesizedExpression as u16, span);
        self.arena.add_children(id, &[expression]);
        id
    }

    /// Create a FunctionExpression: function name?(params): type? { body }
    pub fn function_expression(
        &mut self,
        span: Span,
        name: Option<NodeId>,
        type_parameters: Option<&[NodeId]>,
        parameters: &[NodeId],
        return_type: Option<NodeId>,
        body: NodeId,
        modifiers: u32,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::FunctionExpression as u16, span);
        self.arena.set_modifiers(id, modifiers);

        let mut children = Vec::new();
        if let Some(n) = name {
            children.push(n);
        }
        if let Some(tp) = type_parameters {
            children.extend_from_slice(tp);
        }
        children.extend_from_slice(parameters);
        if let Some(rt) = return_type {
            children.push(rt);
        }
        children.push(body);
        self.arena.add_children(id, &children);
        id
    }

    /// Create an ArrowFunction: (params) => body
    pub fn arrow_function(
        &mut self,
        span: Span,
        type_parameters: Option<&[NodeId]>,
        parameters: &[NodeId],
        return_type: Option<NodeId>,
        body: NodeId,
        modifiers: u32,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ArrowFunction as u16, span);
        self.arena.set_modifiers(id, modifiers);

        let mut children = Vec::new();
        if let Some(tp) = type_parameters {
            self.arena.set_extra(id, tp.len() as u32);
            children.extend_from_slice(tp);
        }
        children.extend_from_slice(parameters);
        if let Some(rt) = return_type {
            children.push(rt);
        }
        children.push(body);
        self.arena.add_children(id, &children);
        id
    }

    /// Create a DeleteExpression: delete expression
    pub fn delete_expression(&mut self, span: Span, expression: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::DeleteExpression as u16, span);
        self.arena.add_children(id, &[expression]);
        id
    }

    /// Create a TypeOfExpression: typeof expression
    pub fn typeof_expression(&mut self, span: Span, expression: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TypeOfExpression as u16, span);
        self.arena.add_children(id, &[expression]);
        id
    }

    /// Create a VoidExpression: void expression
    pub fn void_expression(&mut self, span: Span, expression: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::VoidExpression as u16, span);
        self.arena.add_children(id, &[expression]);
        id
    }

    /// Create an AwaitExpression: await expression
    pub fn await_expression(&mut self, span: Span, expression: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::AwaitExpression as u16, span);
        self.arena.add_children(id, &[expression]);
        id
    }

    /// Create a PrefixUnaryExpression: operator operand
    pub fn prefix_unary(&mut self, span: Span, operator: u16, operand: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::PrefixUnaryExpression as u16, span);
        self.arena.set_extra(id, operator as u32);
        self.arena.add_children(id, &[operand]);
        id
    }

    /// Create a PostfixUnaryExpression: operand operator
    pub fn postfix_unary(&mut self, span: Span, operand: NodeId, operator: u16) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::PostfixUnaryExpression as u16, span);
        self.arena.set_extra(id, operator as u32);
        self.arena.add_children(id, &[operand]);
        id
    }

    /// Create a BinaryExpression: left operator right
    pub fn binary_expression(
        &mut self,
        span: Span,
        left: NodeId,
        operator: u16,
        right: NodeId,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::BinaryExpression as u16, span);
        self.arena.set_extra(id, operator as u32);
        self.arena.add_children(id, &[left, right]);
        id
    }

    /// Create a ConditionalExpression: condition ? whenTrue : whenFalse
    pub fn conditional_expression(
        &mut self,
        span: Span,
        condition: NodeId,
        when_true: NodeId,
        when_false: NodeId,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ConditionalExpression as u16, span);
        self.arena.add_children(id, &[condition, when_true, when_false]);
        id
    }

    /// Create a TemplateExpression: `head ${span} middle ${span} tail`
    pub fn template_expression(&mut self, span: Span, head: NodeId, spans: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TemplateExpression as u16, span);
        let mut children = vec![head];
        children.extend_from_slice(spans);
        self.arena.add_children(id, &children);
        id
    }

    /// Create a TemplateSpan: expression templateMiddle/templateTail
    pub fn template_span(&mut self, span: Span, expression: NodeId, literal: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TemplateSpan as u16, span);
        self.arena.add_children(id, &[expression, literal]);
        id
    }

    /// Create a YieldExpression: yield expression? or yield* expression
    pub fn yield_expression(
        &mut self,
        span: Span,
        expression: Option<NodeId>,
        is_delegate: bool,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::YieldExpression as u16, span);
        if is_delegate {
            self.arena.set_extra(id, 1);
        }
        if let Some(expr) = expression {
            self.arena.add_children(id, &[expr]);
        }
        id
    }

    /// Create a SpreadElement: ...expression
    pub fn spread_element(&mut self, span: Span, expression: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::SpreadElement as u16, span);
        self.arena.add_children(id, &[expression]);
        id
    }

    /// Create a ClassExpression
    pub fn class_expression(
        &mut self,
        span: Span,
        name: Option<NodeId>,
        type_parameters: Option<&[NodeId]>,
        heritage_clauses: Option<&[NodeId]>,
        members: &[NodeId],
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ClassExpression as u16, span);
        let mut children = Vec::new();
        if let Some(n) = name {
            children.push(n);
        }
        if let Some(tp) = type_parameters {
            children.extend_from_slice(tp);
        }
        if let Some(hc) = heritage_clauses {
            children.extend_from_slice(hc);
        }
        children.extend_from_slice(members);
        self.arena.add_children(id, &children);
        id
    }

    /// Create an OmittedExpression (for array holes)
    pub fn omitted_expression(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::OmittedExpression as u16, span)
    }

    /// Create an AsExpression: expression as type
    pub fn as_expression(&mut self, span: Span, expression: NodeId, type_node: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::AsExpression as u16, span);
        self.arena.add_children(id, &[expression, type_node]);
        id
    }

    /// Create a NonNullExpression: expression!
    pub fn non_null_expression(&mut self, span: Span, expression: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::NonNullExpression as u16, span);
        self.arena.add_children(id, &[expression]);
        id
    }

    /// Create a SatisfiesExpression: expression satisfies type
    pub fn satisfies_expression(&mut self, span: Span, expression: NodeId, type_node: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::SatisfiesExpression as u16, span);
        self.arena.add_children(id, &[expression, type_node]);
        id
    }

    /// Create a MetaProperty: new.target or import.meta
    pub fn meta_property(&mut self, span: Span, keyword: u16, name: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::MetaProperty as u16, span);
        self.arena.set_extra(id, keyword as u32);
        self.arena.add_children(id, &[name]);
        id
    }
}

/// Object literal property builders
impl<'a> ExpressionBuilder<'a> {
    /// Create a PropertyAssignment: name: initializer
    pub fn property_assignment(
        &mut self,
        span: Span,
        name: NodeId,
        initializer: NodeId,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::PropertyAssignment as u16, span);
        self.arena.add_children(id, &[name, initializer]);
        id
    }

    /// Create a ShorthandPropertyAssignment: name
    pub fn shorthand_property_assignment(
        &mut self,
        span: Span,
        name: NodeId,
        initializer: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ShorthandPropertyAssignment as u16, span);
        let mut children = vec![name];
        if let Some(init) = initializer {
            children.push(init);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a SpreadAssignment: ...expression
    pub fn spread_assignment(&mut self, span: Span, expression: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::SpreadAssignment as u16, span);
        self.arena.add_children(id, &[expression]);
        id
    }

    /// Create a ComputedPropertyName: [expression]
    pub fn computed_property_name(&mut self, span: Span, expression: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ComputedPropertyName as u16, span);
        self.arena.add_children(id, &[expression]);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier() {
        let mut arena = AstArena::new();
        let mut builder = ExpressionBuilder::new(&mut arena);

        let id = builder.identifier(Span::new(0, 3), "foo");

        assert_eq!(arena.kind(id), Some(NodeKind::Identifier as u16));
        let string_id = arena.string_ids[id.index()];
        assert_eq!(arena.get_string(string_id), Some("foo"));
    }

    #[test]
    fn test_binary_expression() {
        let mut arena = AstArena::new();
        let mut builder = ExpressionBuilder::new(&mut arena);

        let left = builder.numeric_literal(Span::new(0, 1), "1");
        let right = builder.numeric_literal(Span::new(4, 5), "2");
        let expr = builder.binary_expression(
            Span::new(0, 5),
            left,
            NodeKind::Plus as u16,
            right,
        );

        assert_eq!(arena.kind(expr), Some(NodeKind::BinaryExpression as u16));
        assert_eq!(arena.get_children(expr).len(), 2);
        assert_eq!(arena.extra[expr.index()], NodeKind::Plus as u32);
    }

    #[test]
    fn test_call_expression() {
        let mut arena = AstArena::new();
        let mut builder = ExpressionBuilder::new(&mut arena);

        let callee = builder.identifier(Span::new(0, 3), "foo");
        let arg1 = builder.numeric_literal(Span::new(4, 5), "1");
        let arg2 = builder.string_literal(Span::new(7, 12), "hello");

        let call = builder.call_expression(
            Span::new(0, 13),
            callee,
            None,
            &[arg1, arg2],
        );

        assert_eq!(arena.kind(call), Some(NodeKind::CallExpression as u16));
        assert_eq!(arena.get_children(call).len(), 3); // callee + 2 args
    }

    #[test]
    fn test_arrow_function() {
        let mut arena = AstArena::new();
        let mut builder = ExpressionBuilder::new(&mut arena);

        let param = builder.identifier(Span::new(1, 2), "x");
        let body = builder.identifier(Span::new(7, 8), "x");

        let arrow = builder.arrow_function(
            Span::new(0, 8),
            None,
            &[param],
            None,
            body,
            0,
        );

        assert_eq!(arena.kind(arrow), Some(NodeKind::ArrowFunction as u16));
    }

    #[test]
    fn test_object_literal() {
        let mut arena = AstArena::new();
        let mut builder = ExpressionBuilder::new(&mut arena);

        let key = builder.identifier(Span::new(2, 5), "foo");
        let value = builder.numeric_literal(Span::new(7, 8), "1");
        let prop = builder.property_assignment(Span::new(2, 8), key, value);

        let obj = builder.object_literal(Span::new(0, 10), &[prop]);

        assert_eq!(arena.kind(obj), Some(NodeKind::ObjectLiteralExpression as u16));
        assert_eq!(arena.get_children(obj).len(), 1);
    }
}
