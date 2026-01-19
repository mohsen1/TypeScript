//! Type node definitions
//!
//! All TypeScript type AST node types.

use super::arena::{AstArena, NodeId, Span};
use super::nodes::NodeKind;

/// Helper to create type nodes in the arena
pub struct TypeBuilder<'a> {
    arena: &'a mut AstArena,
}

impl<'a> TypeBuilder<'a> {
    pub fn new(arena: &'a mut AstArena) -> Self {
        TypeBuilder { arena }
    }

    // === Keyword Types ===

    /// Create AnyKeyword type
    pub fn any_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::AnyKeyword as u16, span)
    }

    /// Create UnknownKeyword type
    pub fn unknown_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::UnknownKeyword as u16, span)
    }

    /// Create NumberKeyword type
    pub fn number_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::NumberKeyword as u16, span)
    }

    /// Create BigIntKeyword type
    pub fn bigint_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::BigIntKeyword as u16, span)
    }

    /// Create StringKeyword type
    pub fn string_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::StringKeyword as u16, span)
    }

    /// Create BooleanKeyword type
    pub fn boolean_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::BooleanKeyword as u16, span)
    }

    /// Create SymbolKeyword type
    pub fn symbol_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::SymbolKeyword as u16, span)
    }

    /// Create ObjectKeyword type
    pub fn object_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::ObjectKeyword as u16, span)
    }

    /// Create VoidKeyword type
    pub fn void_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::VoidKeyword as u16, span)
    }

    /// Create UndefinedKeyword type
    pub fn undefined_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::UndefinedKeyword as u16, span)
    }

    /// Create NullKeyword type
    pub fn null_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::NullKeyword as u16, span)
    }

    /// Create NeverKeyword type
    pub fn never_keyword(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::NeverKeyword as u16, span)
    }

    // === Complex Type Nodes ===

    /// Create a TypeReference: TypeName<TypeArguments>
    pub fn type_reference(
        &mut self,
        span: Span,
        type_name: NodeId,
        type_arguments: Option<&[NodeId]>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TypeReference as u16, span);
        let mut children = vec![type_name];
        if let Some(args) = type_arguments {
            self.arena.set_extra(id, args.len() as u32);
            children.extend_from_slice(args);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a QualifiedName: left.right
    pub fn qualified_name(&mut self, span: Span, left: NodeId, right: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::QualifiedName as u16, span);
        self.arena.add_children(id, &[left, right]);
        id
    }

    /// Create a TypePredicate: parameterName is Type
    pub fn type_predicate(
        &mut self,
        span: Span,
        asserts: bool,
        parameter_name: NodeId,
        type_node: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TypePredicate as u16, span);
        if asserts {
            self.arena.set_extra(id, 1);
        }
        let mut children = vec![parameter_name];
        if let Some(t) = type_node {
            children.push(t);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a FunctionType: (params) => returnType
    pub fn function_type(
        &mut self,
        span: Span,
        type_parameters: Option<&[NodeId]>,
        parameters: &[NodeId],
        return_type: NodeId,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::FunctionType as u16, span);
        let mut children = Vec::new();
        if let Some(tp) = type_parameters {
            self.arena.set_extra(id, tp.len() as u32);
            children.extend_from_slice(tp);
        }
        children.extend_from_slice(parameters);
        children.push(return_type);
        self.arena.add_children(id, &children);
        id
    }

    /// Create a ConstructorType: new (params) => returnType
    pub fn constructor_type(
        &mut self,
        span: Span,
        type_parameters: Option<&[NodeId]>,
        parameters: &[NodeId],
        return_type: NodeId,
        is_abstract: bool,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ConstructorType as u16, span);
        let mut extra = 0u32;
        if let Some(tp) = type_parameters {
            extra |= (tp.len() as u32) << 8;
        }
        if is_abstract {
            extra |= 1;
        }
        self.arena.set_extra(id, extra);

        let mut children = Vec::new();
        if let Some(tp) = type_parameters {
            children.extend_from_slice(tp);
        }
        children.extend_from_slice(parameters);
        children.push(return_type);
        self.arena.add_children(id, &children);
        id
    }

    /// Create a TypeQuery: typeof expression
    pub fn type_query(&mut self, span: Span, expr_name: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TypeQuery as u16, span);
        self.arena.add_children(id, &[expr_name]);
        id
    }

    /// Create a TypeLiteral: { members }
    pub fn type_literal(&mut self, span: Span, members: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TypeLiteral as u16, span);
        self.arena.add_children(id, members);
        id
    }

    /// Create an ArrayType: elementType[]
    pub fn array_type(&mut self, span: Span, element_type: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ArrayType as u16, span);
        self.arena.add_children(id, &[element_type]);
        id
    }

    /// Create a TupleType: [types]
    pub fn tuple_type(&mut self, span: Span, elements: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TupleType as u16, span);
        self.arena.add_children(id, elements);
        id
    }

    /// Create a NamedTupleMember: name: type
    pub fn named_tuple_member(
        &mut self,
        span: Span,
        name: NodeId,
        type_node: NodeId,
        is_optional: bool,
        is_rest: bool,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::NamedTupleMember as u16, span);
        let extra = (is_optional as u32) | ((is_rest as u32) << 1);
        self.arena.set_extra(id, extra);
        self.arena.add_children(id, &[name, type_node]);
        id
    }

    /// Create an OptionalType: type?
    pub fn optional_type(&mut self, span: Span, type_node: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::OptionalType as u16, span);
        self.arena.add_children(id, &[type_node]);
        id
    }

    /// Create a RestType: ...type
    pub fn rest_type(&mut self, span: Span, type_node: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::RestType as u16, span);
        self.arena.add_children(id, &[type_node]);
        id
    }

    /// Create a UnionType: type | type | ...
    pub fn union_type(&mut self, span: Span, types: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::UnionType as u16, span);
        self.arena.add_children(id, types);
        id
    }

    /// Create an IntersectionType: type & type & ...
    pub fn intersection_type(&mut self, span: Span, types: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::IntersectionType as u16, span);
        self.arena.add_children(id, types);
        id
    }

    /// Create a ConditionalType: checkType extends extendsType ? trueType : falseType
    pub fn conditional_type(
        &mut self,
        span: Span,
        check_type: NodeId,
        extends_type: NodeId,
        true_type: NodeId,
        false_type: NodeId,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ConditionalType as u16, span);
        self.arena.add_children(id, &[check_type, extends_type, true_type, false_type]);
        id
    }

    /// Create an InferType: infer T
    pub fn infer_type(&mut self, span: Span, type_parameter: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::InferType as u16, span);
        self.arena.add_children(id, &[type_parameter]);
        id
    }

    /// Create a ParenthesizedType: (type)
    pub fn parenthesized_type(&mut self, span: Span, type_node: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ParenthesizedType as u16, span);
        self.arena.add_children(id, &[type_node]);
        id
    }

    /// Create ThisType: this
    pub fn this_type(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::ThisType as u16, span)
    }

    /// Create a TypeOperator: keyof/unique/readonly type
    pub fn type_operator(&mut self, span: Span, operator: u16, type_node: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TypeOperator as u16, span);
        self.arena.set_extra(id, operator as u32);
        self.arena.add_children(id, &[type_node]);
        id
    }

    /// Create an IndexedAccessType: objectType[indexType]
    pub fn indexed_access_type(
        &mut self,
        span: Span,
        object_type: NodeId,
        index_type: NodeId,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::IndexedAccessType as u16, span);
        self.arena.add_children(id, &[object_type, index_type]);
        id
    }

    /// Create a MappedType: { [P in K]: T }
    pub fn mapped_type(
        &mut self,
        span: Span,
        readonly_token: Option<u16>,
        type_parameter: NodeId,
        name_type: Option<NodeId>,
        question_token: Option<u16>,
        type_node: Option<NodeId>,
        members: Option<&[NodeId]>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::MappedType as u16, span);

        // Pack readonly and question modifiers into extra
        let readonly = readonly_token.map_or(0u32, |t| (t as u32) << 16);
        let question = question_token.map_or(0u32, |t| t as u32);
        self.arena.set_extra(id, readonly | question);

        let mut children = vec![type_parameter];
        if let Some(nt) = name_type {
            children.push(nt);
        }
        if let Some(t) = type_node {
            children.push(t);
        }
        if let Some(m) = members {
            children.extend_from_slice(m);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a LiteralType: "string" | 123 | true | false | null
    pub fn literal_type(&mut self, span: Span, literal: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::LiteralType as u16, span);
        self.arena.add_children(id, &[literal]);
        id
    }

    /// Create a TemplateLiteralType: `head${type}middle${type}tail`
    pub fn template_literal_type(&mut self, span: Span, head: NodeId, spans: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TemplateLiteralType as u16, span);
        let mut children = vec![head];
        children.extend_from_slice(spans);
        self.arena.add_children(id, &children);
        id
    }

    /// Create a TemplateLiteralTypeSpan: type literal
    pub fn template_literal_type_span(
        &mut self,
        span: Span,
        type_node: NodeId,
        literal: NodeId,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TemplateLiteralTypeSpan as u16, span);
        self.arena.add_children(id, &[type_node, literal]);
        id
    }

    /// Create an ImportType: import("module").Member
    pub fn import_type(
        &mut self,
        span: Span,
        argument: NodeId,
        assertions: Option<NodeId>,
        qualifier: Option<NodeId>,
        type_arguments: Option<&[NodeId]>,
        is_typeof: bool,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ImportType as u16, span);
        if is_typeof {
            self.arena.set_extra(id, 1);
        }
        let mut children = vec![argument];
        if let Some(a) = assertions {
            children.push(a);
        }
        if let Some(q) = qualifier {
            children.push(q);
        }
        if let Some(ta) = type_arguments {
            children.extend_from_slice(ta);
        }
        self.arena.add_children(id, &children);
        id
    }

    // === Type Member Nodes ===

    /// Create a TypeParameter: T extends Constraint = Default
    pub fn type_parameter(
        &mut self,
        span: Span,
        name: NodeId,
        constraint: Option<NodeId>,
        default: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TypeParameter as u16, span);
        let mut children = vec![name];
        if let Some(c) = constraint {
            children.push(c);
        }
        if let Some(d) = default {
            children.push(d);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a Parameter: name?: type = initializer
    pub fn parameter(
        &mut self,
        span: Span,
        decorators: Option<&[NodeId]>,
        modifiers: u32,
        dotdotdot: bool,
        name: NodeId,
        question: bool,
        type_annotation: Option<NodeId>,
        initializer: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::Parameter as u16, span);
        self.arena.set_modifiers(id, modifiers);

        let extra = (dotdotdot as u32) | ((question as u32) << 1);
        self.arena.set_extra(id, extra);

        let mut children = Vec::new();
        if let Some(d) = decorators {
            children.extend_from_slice(d);
        }
        children.push(name);
        if let Some(t) = type_annotation {
            children.push(t);
        }
        if let Some(i) = initializer {
            children.push(i);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a PropertySignature: name?: type
    pub fn property_signature(
        &mut self,
        span: Span,
        modifiers: u32,
        name: NodeId,
        question: bool,
        type_annotation: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::PropertySignature as u16, span);
        self.arena.set_modifiers(id, modifiers);
        if question {
            self.arena.set_extra(id, 1);
        }
        let mut children = vec![name];
        if let Some(t) = type_annotation {
            children.push(t);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a MethodSignature: name(params): returnType
    pub fn method_signature(
        &mut self,
        span: Span,
        modifiers: u32,
        name: NodeId,
        question: bool,
        type_parameters: Option<&[NodeId]>,
        parameters: &[NodeId],
        return_type: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::MethodSignature as u16, span);
        self.arena.set_modifiers(id, modifiers);
        if question {
            self.arena.set_extra(id, 1);
        }
        let mut children = vec![name];
        if let Some(tp) = type_parameters {
            children.extend_from_slice(tp);
        }
        children.extend_from_slice(parameters);
        if let Some(rt) = return_type {
            children.push(rt);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a CallSignature: (params): returnType
    pub fn call_signature(
        &mut self,
        span: Span,
        type_parameters: Option<&[NodeId]>,
        parameters: &[NodeId],
        return_type: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::CallSignature as u16, span);
        let mut children = Vec::new();
        if let Some(tp) = type_parameters {
            self.arena.set_extra(id, tp.len() as u32);
            children.extend_from_slice(tp);
        }
        children.extend_from_slice(parameters);
        if let Some(rt) = return_type {
            children.push(rt);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a ConstructSignature: new (params): returnType
    pub fn construct_signature(
        &mut self,
        span: Span,
        type_parameters: Option<&[NodeId]>,
        parameters: &[NodeId],
        return_type: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ConstructSignature as u16, span);
        let mut children = Vec::new();
        if let Some(tp) = type_parameters {
            self.arena.set_extra(id, tp.len() as u32);
            children.extend_from_slice(tp);
        }
        children.extend_from_slice(parameters);
        if let Some(rt) = return_type {
            children.push(rt);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create an IndexSignature: [param: keyType]: valueType
    pub fn index_signature(
        &mut self,
        span: Span,
        modifiers: u32,
        parameters: &[NodeId],
        type_annotation: NodeId,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::IndexSignature as u16, span);
        self.arena.set_modifiers(id, modifiers);
        let mut children = Vec::new();
        children.extend_from_slice(parameters);
        children.push(type_annotation);
        self.arena.add_children(id, &children);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::expressions::ExpressionBuilder;

    #[test]
    fn test_keyword_types() {
        let mut arena = AstArena::new();
        let mut builder = TypeBuilder::new(&mut arena);

        let string_type = builder.string_keyword(Span::new(0, 6));
        let number_type = builder.number_keyword(Span::new(0, 6));
        let any_type = builder.any_keyword(Span::new(0, 3));

        assert_eq!(arena.kind(string_type), Some(NodeKind::StringKeyword as u16));
        assert_eq!(arena.kind(number_type), Some(NodeKind::NumberKeyword as u16));
        assert_eq!(arena.kind(any_type), Some(NodeKind::AnyKeyword as u16));
    }

    #[test]
    fn test_type_reference() {
        let mut arena = AstArena::new();

        let name = {
            let mut eb = ExpressionBuilder::new(&mut arena);
            eb.identifier(Span::new(0, 5), "Array")
        };

        let element_type = {
            let mut tb = TypeBuilder::new(&mut arena);
            tb.string_keyword(Span::new(6, 12))
        };

        let type_ref = {
            let mut tb = TypeBuilder::new(&mut arena);
            tb.type_reference(Span::new(0, 13), name, Some(&[element_type]))
        };

        assert_eq!(arena.kind(type_ref), Some(NodeKind::TypeReference as u16));
        assert_eq!(arena.get_children(type_ref).len(), 2);
    }

    #[test]
    fn test_union_type() {
        let mut arena = AstArena::new();
        let mut builder = TypeBuilder::new(&mut arena);

        let string_type = builder.string_keyword(Span::new(0, 6));
        let number_type = builder.number_keyword(Span::new(9, 15));

        let union = builder.union_type(Span::new(0, 15), &[string_type, number_type]);

        assert_eq!(arena.kind(union), Some(NodeKind::UnionType as u16));
        assert_eq!(arena.get_children(union).len(), 2);
    }

    #[test]
    fn test_function_type() {
        let mut arena = AstArena::new();

        let param_type = {
            let mut tb = TypeBuilder::new(&mut arena);
            tb.string_keyword(Span::new(1, 7))
        };

        let param_name = {
            let mut eb = ExpressionBuilder::new(&mut arena);
            eb.identifier(Span::new(1, 2), "x")
        };

        let param = {
            let mut tb = TypeBuilder::new(&mut arena);
            tb.parameter(Span::new(1, 7), None, 0, false, param_name, false, Some(param_type), None)
        };

        let return_type = {
            let mut tb = TypeBuilder::new(&mut arena);
            tb.number_keyword(Span::new(12, 18))
        };

        let func_type = {
            let mut tb = TypeBuilder::new(&mut arena);
            tb.function_type(Span::new(0, 18), None, &[param], return_type)
        };

        assert_eq!(arena.kind(func_type), Some(NodeKind::FunctionType as u16));
    }

    #[test]
    fn test_conditional_type() {
        let mut arena = AstArena::new();
        let mut builder = TypeBuilder::new(&mut arena);

        let check = builder.string_keyword(Span::new(0, 6));
        let extends = builder.any_keyword(Span::new(15, 18));
        let true_type = builder.string_keyword(Span::new(21, 27));
        let false_type = builder.never_keyword(Span::new(30, 35));

        let conditional = builder.conditional_type(
            Span::new(0, 35),
            check,
            extends,
            true_type,
            false_type,
        );

        assert_eq!(arena.kind(conditional), Some(NodeKind::ConditionalType as u16));
        assert_eq!(arena.get_children(conditional).len(), 4);
    }
}
