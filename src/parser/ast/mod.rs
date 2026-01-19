//! AST Module - TypeScript Abstract Syntax Tree
//!
//! This module provides a cache-efficient AST representation using:
//! - Structure of Arrays (SoA) layout for cache locality
//! - Arena allocation with bumpalo
//! - NodeId indices instead of Box pointers
//! - ThinNode 32-byte compact representation

pub mod arena;
pub mod nodes;
pub mod statements;
pub mod expressions;
pub mod types;

// Re-export main types
pub use arena::{AstArena, NodeId, Span, StringId, ChildList};
pub use nodes::{NodeKind, NodeFlags, ModifierFlags, ThinNode};
pub use statements::StatementBuilder;
pub use expressions::ExpressionBuilder;
pub use types::TypeBuilder;

/// A unified builder for creating AST nodes
pub struct AstBuilder<'a> {
    arena: &'a mut AstArena,
}

impl<'a> AstBuilder<'a> {
    pub fn new(arena: &'a mut AstArena) -> Self {
        AstBuilder { arena }
    }

    /// Get a statement builder
    pub fn statements(&mut self) -> StatementBuilder<'_> {
        StatementBuilder::new(self.arena)
    }

    /// Get an expression builder
    pub fn expressions(&mut self) -> ExpressionBuilder<'_> {
        ExpressionBuilder::new(self.arena)
    }

    /// Get a type builder
    pub fn types(&mut self) -> TypeBuilder<'_> {
        TypeBuilder::new(self.arena)
    }

    /// Direct access to arena for low-level operations
    pub fn arena(&mut self) -> &mut AstArena {
        self.arena
    }

    /// Create a SourceFile node
    pub fn source_file(&mut self, span: Span, statements: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::SourceFile as u16, span);
        self.arena.add_children(id, statements);
        id
    }

    /// Create a FunctionDeclaration
    pub fn function_declaration(
        &mut self,
        span: Span,
        modifiers: u32,
        name: Option<NodeId>,
        type_parameters: Option<&[NodeId]>,
        parameters: &[NodeId],
        return_type: Option<NodeId>,
        body: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::FunctionDeclaration as u16, span);
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
        if let Some(b) = body {
            children.push(b);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a ClassDeclaration
    pub fn class_declaration(
        &mut self,
        span: Span,
        modifiers: u32,
        name: Option<NodeId>,
        type_parameters: Option<&[NodeId]>,
        heritage_clauses: Option<&[NodeId]>,
        members: &[NodeId],
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ClassDeclaration as u16, span);
        self.arena.set_modifiers(id, modifiers);

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

    /// Create an InterfaceDeclaration
    pub fn interface_declaration(
        &mut self,
        span: Span,
        modifiers: u32,
        name: NodeId,
        type_parameters: Option<&[NodeId]>,
        heritage_clauses: Option<&[NodeId]>,
        members: &[NodeId],
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::InterfaceDeclaration as u16, span);
        self.arena.set_modifiers(id, modifiers);

        let mut children = vec![name];
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

    /// Create a TypeAliasDeclaration
    pub fn type_alias_declaration(
        &mut self,
        span: Span,
        modifiers: u32,
        name: NodeId,
        type_parameters: Option<&[NodeId]>,
        type_node: NodeId,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TypeAliasDeclaration as u16, span);
        self.arena.set_modifiers(id, modifiers);

        let mut children = vec![name];
        if let Some(tp) = type_parameters {
            children.extend_from_slice(tp);
        }
        children.push(type_node);
        self.arena.add_children(id, &children);
        id
    }

    /// Create an EnumDeclaration
    pub fn enum_declaration(
        &mut self,
        span: Span,
        modifiers: u32,
        name: NodeId,
        members: &[NodeId],
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::EnumDeclaration as u16, span);
        self.arena.set_modifiers(id, modifiers);

        let mut children = vec![name];
        children.extend_from_slice(members);
        self.arena.add_children(id, &children);
        id
    }

    /// Create an EnumMember
    pub fn enum_member(
        &mut self,
        span: Span,
        name: NodeId,
        initializer: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::EnumMember as u16, span);
        let mut children = vec![name];
        if let Some(init) = initializer {
            children.push(init);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a ModuleDeclaration (namespace/module)
    pub fn module_declaration(
        &mut self,
        span: Span,
        modifiers: u32,
        name: NodeId,
        body: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ModuleDeclaration as u16, span);
        self.arena.set_modifiers(id, modifiers);

        let mut children = vec![name];
        if let Some(b) = body {
            children.push(b);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create an ImportDeclaration
    pub fn import_declaration(
        &mut self,
        span: Span,
        import_clause: Option<NodeId>,
        module_specifier: NodeId,
        assert_clause: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ImportDeclaration as u16, span);
        let mut children = Vec::new();
        if let Some(ic) = import_clause {
            children.push(ic);
        }
        children.push(module_specifier);
        if let Some(ac) = assert_clause {
            children.push(ac);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create an ExportDeclaration
    pub fn export_declaration(
        &mut self,
        span: Span,
        is_type_only: bool,
        export_clause: Option<NodeId>,
        module_specifier: Option<NodeId>,
        assert_clause: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ExportDeclaration as u16, span);
        if is_type_only {
            self.arena.set_extra(id, 1);
        }
        let mut children = Vec::new();
        if let Some(ec) = export_clause {
            children.push(ec);
        }
        if let Some(ms) = module_specifier {
            children.push(ms);
        }
        if let Some(ac) = assert_clause {
            children.push(ac);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a HeritageClause (extends/implements)
    pub fn heritage_clause(&mut self, span: Span, token: u16, types: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::HeritageClause as u16, span);
        self.arena.set_extra(id, token as u32);
        self.arena.add_children(id, types);
        id
    }

    /// Create a PropertyDeclaration (class member)
    pub fn property_declaration(
        &mut self,
        span: Span,
        modifiers: u32,
        name: NodeId,
        question_or_exclamation: Option<u16>,
        type_annotation: Option<NodeId>,
        initializer: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::PropertyDeclaration as u16, span);
        self.arena.set_modifiers(id, modifiers);
        if let Some(tok) = question_or_exclamation {
            self.arena.set_extra(id, tok as u32);
        }
        let mut children = vec![name];
        if let Some(t) = type_annotation {
            children.push(t);
        }
        if let Some(i) = initializer {
            children.push(i);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a MethodDeclaration (class member)
    pub fn method_declaration(
        &mut self,
        span: Span,
        modifiers: u32,
        name: NodeId,
        question: bool,
        type_parameters: Option<&[NodeId]>,
        parameters: &[NodeId],
        return_type: Option<NodeId>,
        body: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::MethodDeclaration as u16, span);
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
        if let Some(b) = body {
            children.push(b);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a ConstructorDeclaration
    pub fn constructor_declaration(
        &mut self,
        span: Span,
        modifiers: u32,
        parameters: &[NodeId],
        body: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ConstructorDeclaration as u16, span);
        self.arena.set_modifiers(id, modifiers);
        let mut children = Vec::new();
        children.extend_from_slice(parameters);
        if let Some(b) = body {
            children.push(b);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a GetAccessor
    pub fn get_accessor(
        &mut self,
        span: Span,
        modifiers: u32,
        name: NodeId,
        parameters: &[NodeId],
        return_type: Option<NodeId>,
        body: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::GetAccessor as u16, span);
        self.arena.set_modifiers(id, modifiers);
        let mut children = vec![name];
        children.extend_from_slice(parameters);
        if let Some(rt) = return_type {
            children.push(rt);
        }
        if let Some(b) = body {
            children.push(b);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a SetAccessor
    pub fn set_accessor(
        &mut self,
        span: Span,
        modifiers: u32,
        name: NodeId,
        parameters: &[NodeId],
        body: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::SetAccessor as u16, span);
        self.arena.set_modifiers(id, modifiers);
        let mut children = vec![name];
        children.extend_from_slice(parameters);
        if let Some(b) = body {
            children.push(b);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a Decorator
    pub fn decorator(&mut self, span: Span, expression: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::Decorator as u16, span);
        self.arena.add_children(id, &[expression]);
        id
    }
}

/// AST traversal utilities
pub mod traversal {
    use super::{AstArena, NodeId};

    /// Walk AST in pre-order (parent before children)
    pub fn walk_preorder<F>(arena: &AstArena, start: NodeId, mut visitor: F)
    where
        F: FnMut(NodeId) -> bool,
    {
        let mut stack = vec![start];
        while let Some(id) = stack.pop() {
            if !id.is_some() {
                continue;
            }
            if visitor(id) {
                let children = arena.get_children(id);
                for &child in children.iter().rev() {
                    stack.push(child);
                }
            }
        }
    }

    /// Walk AST in post-order (children before parent)
    pub fn walk_postorder<F>(arena: &AstArena, start: NodeId, mut visitor: F)
    where
        F: FnMut(NodeId),
    {
        enum State { Enter(NodeId), Exit(NodeId) }

        let mut stack = vec![State::Enter(start)];
        while let Some(state) = stack.pop() {
            match state {
                State::Enter(id) => {
                    if !id.is_some() {
                        continue;
                    }
                    stack.push(State::Exit(id));
                    let children = arena.get_children(id);
                    for &child in children.iter().rev() {
                        stack.push(State::Enter(child));
                    }
                }
                State::Exit(id) => {
                    visitor(id);
                }
            }
        }
    }

    /// Find first ancestor matching predicate
    pub fn find_ancestor<F>(arena: &AstArena, start: NodeId, predicate: F) -> Option<NodeId>
    where
        F: Fn(NodeId) -> bool,
    {
        let mut current = arena.parent(start)?;
        while current.is_some() {
            if predicate(current) {
                return Some(current);
            }
            current = arena.parent(current)?;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_file_creation() {
        let mut arena = AstArena::new();
        let mut builder = AstBuilder::new(&mut arena);

        // Create: const x = 1;
        let x = builder.expressions().identifier(Span::new(6, 7), "x");
        let one = builder.expressions().numeric_literal(Span::new(10, 11), "1");

        let decl = builder.statements().variable_declaration(
            Span::new(6, 11),
            x,
            None,
            Some(one),
        );

        let decl_list = builder.statements().variable_declaration_list(
            Span::new(0, 11),
            &[decl],
            NodeFlags::CONST.0,
        );

        let stmt = builder.statements().variable_statement(Span::new(0, 12), decl_list);
        let source = builder.source_file(Span::new(0, 12), &[stmt]);

        assert_eq!(arena.kind(source), Some(NodeKind::SourceFile as u16));
        assert_eq!(arena.get_children(source).len(), 1);
    }

    #[test]
    fn test_function_declaration() {
        let mut arena = AstArena::new();
        let mut builder = AstBuilder::new(&mut arena);

        let name = builder.expressions().identifier(Span::new(9, 12), "foo");
        let body = builder.statements().block(Span::new(15, 17), &[]);

        let func = builder.function_declaration(
            Span::new(0, 17),
            0,
            Some(name),
            None,
            &[],
            None,
            Some(body),
        );

        assert_eq!(arena.kind(func), Some(NodeKind::FunctionDeclaration as u16));
    }

    #[test]
    fn test_class_declaration() {
        let mut arena = AstArena::new();
        let mut builder = AstBuilder::new(&mut arena);

        let name = builder.expressions().identifier(Span::new(6, 9), "Foo");

        let class = builder.class_declaration(
            Span::new(0, 12),
            0,
            Some(name),
            None,
            None,
            &[],
        );

        assert_eq!(arena.kind(class), Some(NodeKind::ClassDeclaration as u16));
    }

    #[test]
    fn test_traversal() {
        let mut arena = AstArena::new();
        let mut builder = AstBuilder::new(&mut arena);

        let stmt1 = builder.statements().empty_statement(Span::new(0, 1));
        let stmt2 = builder.statements().debugger_statement(Span::new(2, 11));
        let block = builder.statements().block(Span::new(0, 12), &[stmt1, stmt2]);

        let mut visited = Vec::new();
        traversal::walk_preorder(&arena, block, |id| {
            visited.push(arena.kind(id).unwrap());
            true
        });

        assert_eq!(visited.len(), 3);
        assert_eq!(visited[0], NodeKind::Block as u16);
    }
}
