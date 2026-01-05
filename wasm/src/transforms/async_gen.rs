//! Async/Await and Generator Transforms
//!
//! Transforms async functions and generators to ES5:
//! - async function → __awaiter helper
//! - await expression → Promise chain
//! - generator function → __generator state machine
//! - yield expression → state transitions
//! - async generator → combined transform
//! - for-await-of → async iteration protocol

use super::{TransformContext, Transformer, HelpersNeeded};
use crate::parser::{Node, NodeIndex, NodeBase, NodeList, syntax_kind_ext};
use crate::parser::ast::{
    Identifier, CallExpression, FunctionExpression, Block, ReturnStatement,
    YieldExpression,
};
use crate::scanner::SyntaxKind;

/// Async/generator transformation state
pub struct AsyncTransformer {
    /// Stack of enclosing async function depths
    _async_depth: u32,
    /// Stack of enclosing generator depths
    _generator_depth: u32,
}

impl AsyncTransformer {
    pub fn new() -> Self {
        AsyncTransformer {
            _async_depth: 0,
            _generator_depth: 0,
        }
    }

    /// Check if a function declaration is async
    fn is_async_function(&self, node_idx: NodeIndex, ctx: &TransformContext) -> bool {
        match ctx.arena.get(node_idx) {
            Some(Node::FunctionDeclaration(f)) => f.is_async,
            Some(Node::ArrowFunction(f)) => {
                // Check modifiers for async
                if let Some(ref mods) = f.modifiers {
                    for mod_idx in &mods.nodes {
                        if let Some(Node::Token(base)) = ctx.arena.get(*mod_idx) {
                            if base.kind == crate::scanner::SyntaxKind::AsyncKeyword as u16 {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            Some(Node::FunctionExpression(f)) => {
                // Check modifiers for async
                if let Some(ref mods) = f.modifiers {
                    for mod_idx in &mods.nodes {
                        if let Some(Node::Token(base)) = ctx.arena.get(*mod_idx) {
                            if base.kind == crate::scanner::SyntaxKind::AsyncKeyword as u16 {
                                return true;
                            }
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Check if a function declaration is a generator
    fn is_generator_function(&self, node_idx: NodeIndex, ctx: &TransformContext) -> bool {
        match ctx.arena.get(node_idx) {
            Some(Node::FunctionDeclaration(f)) => f.asterisk_token,
            Some(Node::FunctionExpression(f)) => f.asterisk_token,
            _ => false,
        }
    }

    /// Transform async function to __awaiter pattern
    ///
    /// ```ts
    /// async function foo() {
    ///     const x = await bar();
    ///     return x + 1;
    /// }
    /// ```
    ///
    /// Becomes:
    ///
    /// ```js
    /// function foo() {
    ///     return __awaiter(this, void 0, void 0, function* () {
    ///         const x = yield bar();
    ///         return x + 1;
    ///     });
    /// }
    /// ```
    pub fn transform_async_function(
        &mut self,
        node_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        // Mark that we need the awaiter helper
        ctx.helpers_needed.awaiter = true;
        ctx.helpers_needed.generator = true;

        // Visit children to transform await expressions
        self.visit_children(node_idx, ctx);

        // In a full implementation:
        // 1. Wrap function body in __awaiter(this, void 0, void 0, function* () { ... })
        // 2. Transform all await expressions to yield expressions
        // 3. Handle async arrow functions specially

        None
    }

    /// Transform generator function to __generator state machine
    ///
    /// ```ts
    /// function* gen() {
    ///     yield 1;
    ///     yield 2;
    /// }
    /// ```
    ///
    /// Becomes:
    ///
    /// ```js
    /// function gen() {
    ///     return __generator(this, function (_a) {
    ///         switch (_a.label) {
    ///             case 0: return [4 /*yield*/, 1];
    ///             case 1:
    ///                 _a.sent();
    ///                 return [4 /*yield*/, 2];
    ///             case 2:
    ///                 _a.sent();
    ///                 return [2 /*return*/];
    ///         }
    ///     });
    /// }
    /// ```
    pub fn transform_generator_function(
        &mut self,
        node_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        // Mark that we need the generator helper
        ctx.helpers_needed.generator = true;

        // Visit children to collect yield points
        self.visit_children(node_idx, ctx);

        // In a full implementation:
        // 1. Remove asterisk from function
        // 2. Wrap body in __generator(this, function(_a) { switch (_a.label) { ... } })
        // 3. Transform yield to return [4 /*yield*/, value]
        // 4. Add state labels for each yield point

        None
    }

    /// Transform await expression
    ///
    /// In async context: `await x` → `yield x`
    pub fn transform_await_expression(
        &mut self,
        node_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        // Get the await expression
        let await_expr = match ctx.arena.get(node_idx) {
            Some(Node::AwaitExpression(expr)) => expr.clone(),
            _ => return None,
        };

        // Transform await to yield
        // await x → yield x
        let yield_expr = YieldExpression {
            base: NodeBase::new_ext(syntax_kind_ext::YIELD_EXPRESSION, 0, 0),
            asterisk_token: false,
            expression: await_expr.expression,
        };
        Some(ctx.arena.add(Node::YieldExpression(yield_expr)))
    }

    /// Create an identifier node
    fn create_identifier(name: &str, ctx: &mut TransformContext) -> NodeIndex {
        let id = Identifier {
            base: NodeBase::new(SyntaxKind::Identifier, 0, 0),
            escaped_text: name.to_string(),
            original_text: None,
            type_arguments: None,
        };
        ctx.arena.add(Node::Identifier(id))
    }

    /// Create a void 0 expression (for undefined)
    fn create_void_0(&self, ctx: &mut TransformContext) -> NodeIndex {
        use crate::parser::ast::PrefixUnaryExpression;
        use crate::parser::ast::NumericLiteral;

        // Create the numeric literal 0
        let zero = NumericLiteral {
            base: NodeBase::new(SyntaxKind::NumericLiteral, 0, 0),
            text: "0".to_string(),
            value: 0.0,
        };
        let zero_idx = ctx.arena.add(Node::NumericLiteral(zero));

        // Create void 0 using PrefixUnaryExpression with VoidKeyword
        let void_expr = PrefixUnaryExpression {
            base: NodeBase::new_ext(syntax_kind_ext::PREFIX_UNARY_EXPRESSION, 0, 0),
            operator: SyntaxKind::VoidKeyword,
            operand: zero_idx,
        };
        ctx.arena.add(Node::PrefixUnaryExpression(void_expr))
    }

    /// Create a generator function expression for __awaiter
    fn create_generator_wrapper(
        &self,
        body_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> NodeIndex {
        // Create function* () { body }
        let params = NodeList::new();

        let func_expr = FunctionExpression {
            base: NodeBase::new_ext(syntax_kind_ext::FUNCTION_EXPRESSION, 0, 0),
            modifiers: None,
            asterisk_token: true,  // This is a generator
            name: NodeIndex::NONE,
            type_parameters: None,
            parameters: params,
            type_annotation: NodeIndex::NONE,
            body: body_idx,
        };
        ctx.arena.add(Node::FunctionExpression(func_expr))
    }

    /// Create __awaiter(this, void 0, void 0, function* () { body }) call
    fn create_awaiter_call(
        &self,
        body_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> NodeIndex {
        // Create __awaiter identifier
        let awaiter_id = Self::create_identifier("__awaiter", ctx);

        // Create 'this' reference
        let this_id = ctx.arena.add(Node::Token(NodeBase::new(SyntaxKind::ThisKeyword, 0, 0)));

        // Create void 0 (undefined)
        let void_0_1 = self.create_void_0(ctx);
        let void_0_2 = self.create_void_0(ctx);

        // Create generator wrapper
        let generator_func = self.create_generator_wrapper(body_idx, ctx);

        // Create call: __awaiter(this, void 0, void 0, function* () { body })
        let mut args = NodeList::new();
        args.push(this_id);
        args.push(void_0_1);
        args.push(void_0_2);
        args.push(generator_func);

        let call_expr = CallExpression {
            base: NodeBase::new_ext(syntax_kind_ext::CALL_EXPRESSION, 0, 0),
            expression: awaiter_id,
            type_arguments: None,
            arguments: args,
        };
        ctx.arena.add(Node::CallExpression(call_expr))
    }

    /// Create a return statement with the __awaiter call
    fn create_awaiter_return(
        &self,
        body_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> NodeIndex {
        let awaiter_call = self.create_awaiter_call(body_idx, ctx);

        let return_stmt = ReturnStatement {
            base: NodeBase::new_ext(syntax_kind_ext::RETURN_STATEMENT, 0, 0),
            expression: awaiter_call,
        };
        ctx.arena.add(Node::ReturnStatement(return_stmt))
    }

    /// Transform yield expression
    ///
    /// `yield x` → `return [4 /*yield*/, x]`
    pub fn transform_yield_expression(
        &mut self,
        _node_idx: NodeIndex,
        _ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        // Transform yield to state machine return
        None
    }

    /// Transform for-await-of loop
    ///
    /// ```ts
    /// for await (const x of asyncIter) { ... }
    /// ```
    ///
    /// Becomes:
    ///
    /// ```js
    /// var _a, _b;
    /// try {
    ///     for (var asyncIter_1 = __asyncValues(asyncIter); _a = yield asyncIter_1.next(), !_a.done;) {
    ///         const x = _a.value;
    ///         ...
    ///     }
    /// } catch (e_1_1) { e_1 = { error: e_1_1 }; }
    /// finally {
    ///     try { if (_a && !_a.done && (_b = asyncIter_1.return)) yield _b.call(asyncIter_1); }
    ///     finally { if (e_1) throw e_1.error; }
    /// }
    /// ```
    pub fn transform_for_await_of(
        &mut self,
        _node_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        // Mark that we need async values helper
        ctx.helpers_needed.async_values = true;
        None
    }
}

/// Node kind for async-related transforms
#[derive(Clone, Copy, PartialEq, Eq)]
enum AsyncNodeKind {
    FunctionDeclaration,
    FunctionExpression,
    ArrowFunction,
    AwaitExpression,
    YieldExpression,
    ForOfStatement,
    SourceFile,
    Block,
    Other,
}

fn get_async_node_kind(node_idx: NodeIndex, ctx: &TransformContext) -> AsyncNodeKind {
    match ctx.arena.get(node_idx) {
        Some(Node::FunctionDeclaration(_)) => AsyncNodeKind::FunctionDeclaration,
        Some(Node::FunctionExpression(_)) => AsyncNodeKind::FunctionExpression,
        Some(Node::ArrowFunction(_)) => AsyncNodeKind::ArrowFunction,
        Some(Node::AwaitExpression(_)) => AsyncNodeKind::AwaitExpression,
        Some(Node::YieldExpression(_)) => AsyncNodeKind::YieldExpression,
        Some(Node::ForOfStatement(_)) => AsyncNodeKind::ForOfStatement,
        Some(Node::SourceFile(_)) => AsyncNodeKind::SourceFile,
        Some(Node::Block(_)) => AsyncNodeKind::Block,
        _ => AsyncNodeKind::Other,
    }
}

impl Transformer for AsyncTransformer {
    fn visit_node(&mut self, node_idx: NodeIndex, ctx: &mut TransformContext) -> Option<NodeIndex> {
        let kind = get_async_node_kind(node_idx, ctx);

        match kind {
            AsyncNodeKind::FunctionDeclaration
            | AsyncNodeKind::FunctionExpression
            | AsyncNodeKind::ArrowFunction => {
                // Check if this is an async function
                if self.is_async_function(node_idx, ctx) {
                    return self.transform_async_function(node_idx, ctx);
                }
                // Check if this is a generator function
                if self.is_generator_function(node_idx, ctx) {
                    return self.transform_generator_function(node_idx, ctx);
                }
            }
            AsyncNodeKind::AwaitExpression => {
                return self.transform_await_expression(node_idx, ctx);
            }
            AsyncNodeKind::YieldExpression => {
                return self.transform_yield_expression(node_idx, ctx);
            }
            AsyncNodeKind::ForOfStatement => {
                // Check if it's for-await-of
                if let Some(Node::ForOfStatement(for_of)) = ctx.arena.get(node_idx) {
                    if for_of.await_modifier {
                        return self.transform_for_await_of(node_idx, ctx);
                    }
                }
            }
            _ => {}
        }

        self.visit_children(node_idx, ctx);
        None
    }

    fn visit_children(&mut self, node_idx: NodeIndex, ctx: &mut TransformContext) {
        let children = collect_async_children(node_idx, ctx);

        for child_idx in children {
            self.visit_node(child_idx, ctx);
        }
    }
}

fn collect_async_children(node_idx: NodeIndex, ctx: &TransformContext) -> Vec<NodeIndex> {
    let mut children = Vec::new();

    match ctx.arena.get(node_idx) {
        Some(Node::SourceFile(sf)) => {
            for stmt in &sf.statements.nodes {
                children.push(*stmt);
            }
        }
        Some(Node::Block(block)) => {
            for stmt in &block.statements.nodes {
                children.push(*stmt);
            }
        }
        Some(Node::FunctionDeclaration(f)) => {
            if !f.body.is_none() {
                children.push(f.body);
            }
        }
        Some(Node::FunctionExpression(f)) => {
            if !f.body.is_none() {
                children.push(f.body);
            }
        }
        Some(Node::ArrowFunction(f)) => {
            children.push(f.body);
        }
        Some(Node::AwaitExpression(await_expr)) => {
            children.push(await_expr.expression);
        }
        Some(Node::YieldExpression(yield_expr)) => {
            if !yield_expr.expression.is_none() {
                children.push(yield_expr.expression);
            }
        }
        Some(Node::ForOfStatement(for_of)) => {
            children.push(for_of.initializer);
            children.push(for_of.expression);
            children.push(for_of.statement);
        }
        Some(Node::ExpressionStatement(expr_stmt)) => {
            children.push(expr_stmt.expression);
        }
        Some(Node::ReturnStatement(ret_stmt)) => {
            if !ret_stmt.expression.is_none() {
                children.push(ret_stmt.expression);
            }
        }
        _ => {}
    }

    children
}

#[cfg(test)]
#[path = "async_gen_tests.rs"]
mod async_gen_tests;
