//! Statement node definitions
//!
//! All TypeScript statement AST node types.

use super::arena::{AstArena, NodeId, Span, StringId};
use super::nodes::NodeKind;

/// Helper to create statement nodes in the arena
pub struct StatementBuilder<'a> {
    arena: &'a mut AstArena,
}

impl<'a> StatementBuilder<'a> {
    pub fn new(arena: &'a mut AstArena) -> Self {
        StatementBuilder { arena }
    }

    /// Create a Block statement: { statements }
    pub fn block(&mut self, span: Span, statements: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::Block as u16, span);
        self.arena.add_children(id, statements);
        id
    }

    /// Create a VariableStatement: var/let/const declarations;
    pub fn variable_statement(&mut self, span: Span, declaration_list: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::VariableStatement as u16, span);
        self.arena.add_children(id, &[declaration_list]);
        id
    }

    /// Create an EmptyStatement: ;
    pub fn empty_statement(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::EmptyStatement as u16, span)
    }

    /// Create an ExpressionStatement: expression;
    pub fn expression_statement(&mut self, span: Span, expression: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ExpressionStatement as u16, span);
        self.arena.add_children(id, &[expression]);
        id
    }

    /// Create an IfStatement: if (condition) thenStmt else? elseStmt?
    pub fn if_statement(
        &mut self,
        span: Span,
        condition: NodeId,
        then_stmt: NodeId,
        else_stmt: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::IfStatement as u16, span);
        let mut children = vec![condition, then_stmt];
        if let Some(else_s) = else_stmt {
            children.push(else_s);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a DoStatement: do statement while (condition);
    pub fn do_statement(&mut self, span: Span, statement: NodeId, condition: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::DoStatement as u16, span);
        self.arena.add_children(id, &[statement, condition]);
        id
    }

    /// Create a WhileStatement: while (condition) statement
    pub fn while_statement(&mut self, span: Span, condition: NodeId, statement: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::WhileStatement as u16, span);
        self.arena.add_children(id, &[condition, statement]);
        id
    }

    /// Create a ForStatement: for (init?; cond?; incr?) statement
    pub fn for_statement(
        &mut self,
        span: Span,
        initializer: Option<NodeId>,
        condition: Option<NodeId>,
        incrementor: Option<NodeId>,
        statement: NodeId,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ForStatement as u16, span);
        let mut children = Vec::new();
        // Use NodeId::NONE as placeholder for missing parts
        children.push(initializer.unwrap_or(NodeId::NONE));
        children.push(condition.unwrap_or(NodeId::NONE));
        children.push(incrementor.unwrap_or(NodeId::NONE));
        children.push(statement);
        self.arena.add_children(id, &children);
        id
    }

    /// Create a ForInStatement: for (variable in expression) statement
    pub fn for_in_statement(
        &mut self,
        span: Span,
        initializer: NodeId,
        expression: NodeId,
        statement: NodeId,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ForInStatement as u16, span);
        self.arena.add_children(id, &[initializer, expression, statement]);
        id
    }

    /// Create a ForOfStatement: for (variable of expression) statement
    pub fn for_of_statement(
        &mut self,
        span: Span,
        initializer: NodeId,
        expression: NodeId,
        statement: NodeId,
        is_await: bool,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ForOfStatement as u16, span);
        if is_await {
            self.arena.set_extra(id, 1); // Mark as await for-of
        }
        self.arena.add_children(id, &[initializer, expression, statement]);
        id
    }

    /// Create a ContinueStatement: continue label?;
    pub fn continue_statement(&mut self, span: Span, label: Option<StringId>) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ContinueStatement as u16, span);
        if let Some(l) = label {
            self.arena.set_string(id, l);
        }
        id
    }

    /// Create a BreakStatement: break label?;
    pub fn break_statement(&mut self, span: Span, label: Option<StringId>) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::BreakStatement as u16, span);
        if let Some(l) = label {
            self.arena.set_string(id, l);
        }
        id
    }

    /// Create a ReturnStatement: return expression?;
    pub fn return_statement(&mut self, span: Span, expression: Option<NodeId>) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ReturnStatement as u16, span);
        if let Some(expr) = expression {
            self.arena.add_children(id, &[expr]);
        }
        id
    }

    /// Create a WithStatement: with (expression) statement
    pub fn with_statement(&mut self, span: Span, expression: NodeId, statement: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::WithStatement as u16, span);
        self.arena.add_children(id, &[expression, statement]);
        id
    }

    /// Create a SwitchStatement: switch (expression) { caseBlock }
    pub fn switch_statement(&mut self, span: Span, expression: NodeId, case_block: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::SwitchStatement as u16, span);
        self.arena.add_children(id, &[expression, case_block]);
        id
    }

    /// Create a LabeledStatement: label: statement
    pub fn labeled_statement(&mut self, span: Span, label: StringId, statement: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::LabeledStatement as u16, span);
        self.arena.set_string(id, label);
        self.arena.add_children(id, &[statement]);
        id
    }

    /// Create a ThrowStatement: throw expression;
    pub fn throw_statement(&mut self, span: Span, expression: NodeId) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::ThrowStatement as u16, span);
        self.arena.add_children(id, &[expression]);
        id
    }

    /// Create a TryStatement: try block catch? finally?
    pub fn try_statement(
        &mut self,
        span: Span,
        try_block: NodeId,
        catch_clause: Option<NodeId>,
        finally_block: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::TryStatement as u16, span);
        let mut children = vec![try_block];
        if let Some(catch) = catch_clause {
            children.push(catch);
        }
        if let Some(finally) = finally_block {
            children.push(finally);
        }
        self.arena.add_children(id, &children);
        id
    }

    /// Create a DebuggerStatement: debugger;
    pub fn debugger_statement(&mut self, span: Span) -> NodeId {
        self.arena.alloc_node(NodeKind::DebuggerStatement as u16, span)
    }

    /// Create a CaseBlock: { clauses }
    pub fn case_block(&mut self, span: Span, clauses: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::CaseBlock as u16, span);
        self.arena.add_children(id, clauses);
        id
    }

    /// Create a CaseClause: case expression: statements
    pub fn case_clause(&mut self, span: Span, expression: NodeId, statements: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::CaseClause as u16, span);
        let mut children = vec![expression];
        children.extend_from_slice(statements);
        self.arena.add_children(id, &children);
        id
    }

    /// Create a DefaultClause: default: statements
    pub fn default_clause(&mut self, span: Span, statements: &[NodeId]) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::DefaultClause as u16, span);
        self.arena.add_children(id, statements);
        id
    }

    /// Create a CatchClause: catch (variable?) block
    pub fn catch_clause(
        &mut self,
        span: Span,
        variable: Option<NodeId>,
        block: NodeId,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::CatchClause as u16, span);
        let mut children = Vec::new();
        if let Some(var) = variable {
            children.push(var);
        }
        children.push(block);
        self.arena.add_children(id, &children);
        id
    }
}

/// Variable declaration related builders
impl<'a> StatementBuilder<'a> {
    /// Create a VariableDeclarationList: var/let/const declarations
    pub fn variable_declaration_list(
        &mut self,
        span: Span,
        declarations: &[NodeId],
        flags: u16,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::VariableDeclarationList as u16, span);
        self.arena.set_flags(id, flags);
        self.arena.add_children(id, declarations);
        id
    }

    /// Create a VariableDeclaration: name: type? = initializer?
    pub fn variable_declaration(
        &mut self,
        span: Span,
        name: NodeId,
        type_annotation: Option<NodeId>,
        initializer: Option<NodeId>,
    ) -> NodeId {
        let id = self.arena.alloc_node(NodeKind::VariableDeclaration as u16, span);
        let mut children = vec![name];
        if let Some(ty) = type_annotation {
            children.push(ty);
        }
        if let Some(init) = initializer {
            children.push(init);
        }
        self.arena.add_children(id, &children);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_statement() {
        let mut arena = AstArena::new();
        let mut builder = StatementBuilder::new(&mut arena);

        let stmt1 = builder.empty_statement(Span::new(1, 2));
        let stmt2 = builder.debugger_statement(Span::new(3, 12));
        let block = builder.block(Span::new(0, 13), &[stmt1, stmt2]);

        assert_eq!(arena.kind(block), Some(NodeKind::Block as u16));
        assert_eq!(arena.get_children(block).len(), 2);
    }

    #[test]
    fn test_if_statement() {
        let mut arena = AstArena::new();

        let condition = arena.alloc_node(NodeKind::TrueKeyword as u16, Span::new(4, 8));
        let then_stmt;
        let else_stmt;
        let if_stmt;
        {
            let mut builder = StatementBuilder::new(&mut arena);
            then_stmt = builder.empty_statement(Span::new(10, 11));
            else_stmt = builder.empty_statement(Span::new(17, 18));
            if_stmt = builder.if_statement(
                Span::new(0, 18),
                condition,
                then_stmt,
                Some(else_stmt),
            );
        }

        assert_eq!(arena.kind(if_stmt), Some(NodeKind::IfStatement as u16));
        assert_eq!(arena.get_children(if_stmt).len(), 3);
    }

    #[test]
    fn test_for_statement() {
        let mut arena = AstArena::new();
        let mut builder = StatementBuilder::new(&mut arena);

        let body = builder.empty_statement(Span::new(10, 11));
        let for_stmt = builder.for_statement(
            Span::new(0, 11),
            None,
            None,
            None,
            body,
        );

        assert_eq!(arena.kind(for_stmt), Some(NodeKind::ForStatement as u16));
    }

    #[test]
    fn test_try_statement() {
        let mut arena = AstArena::new();
        let mut builder = StatementBuilder::new(&mut arena);

        let try_block = builder.block(Span::new(4, 6), &[]);
        let catch_block = builder.block(Span::new(15, 17), &[]);
        let catch_clause = builder.catch_clause(Span::new(7, 17), None, catch_block);

        let try_stmt = builder.try_statement(
            Span::new(0, 17),
            try_block,
            Some(catch_clause),
            None,
        );

        assert_eq!(arena.kind(try_stmt), Some(NodeKind::TryStatement as u16));
        assert_eq!(arena.get_children(try_stmt).len(), 2);
    }
}
