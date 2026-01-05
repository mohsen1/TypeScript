//! Generator Function Transforms
//!
//! Transforms generator functions (function*) to ES5-compatible state machines.
//! This implements TypeScript's generator transformation pattern.
//!
//! # Overview
//!
//! Generator functions are transformed into regular functions that return
//! an iterator object. The function body becomes a state machine implemented
//! as a switch statement.
//!
//! # Example
//!
//! Input:
//! ```typescript
//! function* gen() {
//!     yield 1;
//!     yield 2;
//!     return 3;
//! }
//! ```
//!
//! Output (ES5):
//! ```javascript
//! function gen() {
//!     return __generator(this, function (_a) {
//!         switch (_a.label) {
//!             case 0: return [4 /*yield*/, 1];
//!             case 1:
//!                 _a.sent();
//!                 return [4 /*yield*/, 2];
//!             case 2:
//!                 _a.sent();
//!                 return [2 /*return*/, 3];
//!         }
//!     });
//! }
//! ```

use crate::parser::{Node, NodeIndex};
use super::{TransformContext, Transformer};

// =============================================================================
// Instruction Codes (match TypeScript's __generator helper)
// =============================================================================

/// Instruction codes used by the __generator helper
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    /// next()
    Next = 0,
    /// throw()
    Throw = 1,
    /// return()
    Return = 2,
    /// Break/jump to label
    Break = 3,
    /// yield value
    Yield = 4,
    /// yield* iterator
    YieldStar = 5,
    /// catch exception
    Catch = 6,
    /// end finally block
    Endfinally = 7,
}

// =============================================================================
// Code Blocks
// =============================================================================

/// Types of code blocks that affect control flow
#[derive(Clone, Debug)]
pub enum CodeBlock {
    /// Try/catch/finally block
    Exception {
        try_label: u32,
        catch_label: Option<u32>,
        finally_label: Option<u32>,
        end_label: u32,
        catch_variable: Option<String>,
    },
    /// Loop block (for break/continue)
    Loop {
        continue_label: u32,
        break_label: u32,
    },
    /// Labeled statement
    Labeled {
        name: String,
        break_label: u32,
    },
    /// Switch statement
    Switch {
        break_label: u32,
    },
}

// =============================================================================
// Operation
// =============================================================================

/// An operation in the generator state machine
#[derive(Clone, Debug)]
pub enum Operation {
    /// No operation (forces new case)
    Nop,
    /// Regular statement
    Statement(NodeIndex),
    /// Assignment
    Assign {
        target: NodeIndex,
        value: NodeIndex,
    },
    /// Unconditional break
    Break(u32),
    /// Break if true
    BreakWhenTrue {
        label: u32,
        condition: NodeIndex,
    },
    /// Break if false
    BreakWhenFalse {
        label: u32,
        condition: NodeIndex,
    },
    /// Yield value
    Yield {
        expression: Option<NodeIndex>,
        is_delegating: bool,
    },
    /// Return from generator
    Return(Option<NodeIndex>),
    /// Throw exception
    Throw(NodeIndex),
    /// End finally block
    EndFinally,
}

// =============================================================================
// Generator Transformer
// =============================================================================

/// Transforms generator functions to state machines
pub struct GeneratorTransformer {
    /// Next label id
    next_label: u32,
    /// Current operations
    operations: Vec<Operation>,
    /// Label offsets (label -> operation index)
    label_offsets: Vec<Option<u32>>,
    /// Active code blocks
    block_stack: Vec<CodeBlock>,
    /// Whether we're in a generator body
    in_generator_body: bool,
    /// Whether we're in a statement containing yield
    in_statement_with_yield: bool,
}

impl GeneratorTransformer {
    pub fn new() -> Self {
        GeneratorTransformer {
            next_label: 1,
            operations: Vec::new(),
            label_offsets: Vec::new(),
            block_stack: Vec::new(),
            in_generator_body: false,
            in_statement_with_yield: false,
        }
    }

    /// Create a new label
    fn new_label(&mut self) -> u32 {
        let label = self.next_label;
        self.next_label += 1;
        // Ensure label_offsets has space
        while self.label_offsets.len() <= label as usize {
            self.label_offsets.push(None);
        }
        label
    }

    /// Mark a label at the current operation
    fn mark_label(&mut self, label: u32) {
        let offset = self.operations.len() as u32;
        if label as usize >= self.label_offsets.len() {
            self.label_offsets.resize(label as usize + 1, None);
        }
        self.label_offsets[label as usize] = Some(offset);
    }

    /// Add an operation
    fn emit_op(&mut self, op: Operation) {
        self.operations.push(op);
    }

    /// Reset state for a new generator
    fn reset(&mut self) {
        self.next_label = 1;
        self.operations.clear();
        self.label_offsets.clear();
        self.block_stack.clear();
    }

    /// Check if a function is a generator
    fn is_generator_function(&self, node_idx: NodeIndex, ctx: &TransformContext) -> bool {
        match ctx.arena.get(node_idx) {
            Some(Node::FunctionDeclaration(f)) => f.asterisk_token,
            Some(Node::FunctionExpression(f)) => f.asterisk_token,
            _ => false,
        }
    }

    /// Transform a generator function body to a state machine
    pub fn transform_generator_function(
        &mut self,
        node_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        // Mark that we need the generator helper
        ctx.helpers_needed.generator = true;

        self.reset();
        self.in_generator_body = true;

        // Initial label (case 0)
        let initial_label = self.new_label();
        self.mark_label(initial_label);

        // Visit the function body to collect operations
        let body_idx = match ctx.arena.get(node_idx) {
            Some(Node::FunctionDeclaration(f)) => f.body,
            Some(Node::FunctionExpression(f)) => f.body,
            _ => return None,
        };

        // Process the body
        self.visit_block(body_idx, ctx);

        // Add implicit return at end
        if !matches!(self.operations.last(), Some(Operation::Return(_))) {
            self.emit_op(Operation::Return(None));
        }

        self.in_generator_body = false;

        // Build the state machine
        self.build_state_machine(node_idx, ctx)
    }

    /// Visit a block and collect operations
    fn visit_block(&mut self, block_idx: NodeIndex, ctx: &mut TransformContext) {
        if block_idx.is_none() {
            return;
        }

        if let Some(Node::Block(block)) = ctx.arena.get(block_idx) {
            let statements = block.statements.nodes.clone();
            for stmt_idx in statements {
                self.visit_statement(stmt_idx, ctx);
            }
        }
    }

    /// Visit a statement and generate operations
    fn visit_statement(&mut self, stmt_idx: NodeIndex, ctx: &mut TransformContext) {
        if stmt_idx.is_none() {
            return;
        }

        match ctx.arena.get(stmt_idx) {
            Some(Node::ExpressionStatement(expr_stmt)) => {
                // Check if contains yield
                if self.contains_yield(expr_stmt.expression, ctx) {
                    self.visit_expression_for_yield(expr_stmt.expression, ctx);
                } else {
                    self.emit_op(Operation::Statement(stmt_idx));
                }
            }
            Some(Node::ReturnStatement(ret)) => {
                if ret.expression.is_none() {
                    self.emit_op(Operation::Return(None));
                } else if self.contains_yield(ret.expression, ctx) {
                    // Transform yield in return expression
                    let result = self.visit_expression_for_yield(ret.expression, ctx);
                    self.emit_op(Operation::Return(result));
                } else {
                    self.emit_op(Operation::Return(Some(ret.expression)));
                }
            }
            Some(Node::IfStatement(_)) => {
                self.visit_if_statement(stmt_idx, ctx);
            }
            Some(Node::Block(_)) => {
                self.visit_block(stmt_idx, ctx);
            }
            Some(Node::ForStatement(_)) | Some(Node::WhileStatement(_)) | Some(Node::DoStatement(_)) => {
                self.visit_loop_statement(stmt_idx, ctx);
            }
            Some(Node::TryStatement(_)) => {
                self.visit_try_statement(stmt_idx, ctx);
            }
            Some(Node::ThrowStatement(throw_stmt)) => {
                self.emit_op(Operation::Throw(throw_stmt.expression));
            }
            Some(Node::BreakStatement(break_stmt)) => {
                self.visit_break_statement(break_stmt, ctx);
            }
            Some(Node::ContinueStatement(cont_stmt)) => {
                self.visit_continue_statement(cont_stmt, ctx);
            }
            _ => {
                // Default: emit as-is
                self.emit_op(Operation::Statement(stmt_idx));
            }
        }
    }

    /// Check if an expression contains yield
    fn contains_yield(&self, expr_idx: NodeIndex, ctx: &TransformContext) -> bool {
        if expr_idx.is_none() {
            return false;
        }

        match ctx.arena.get(expr_idx) {
            Some(Node::YieldExpression(_)) => true,
            Some(Node::BinaryExpression(bin)) => {
                self.contains_yield(bin.left, ctx) || self.contains_yield(bin.right, ctx)
            }
            Some(Node::CallExpression(call)) => {
                self.contains_yield(call.expression, ctx)
                    || call.arguments.nodes.iter().any(|&arg| self.contains_yield(arg, ctx))
            }
            Some(Node::ConditionalExpression(cond)) => {
                self.contains_yield(cond.condition, ctx)
                    || self.contains_yield(cond.when_true, ctx)
                    || self.contains_yield(cond.when_false, ctx)
            }
            Some(Node::ParenthesizedExpression(paren)) => {
                self.contains_yield(paren.expression, ctx)
            }
            _ => false,
        }
    }

    /// Visit an expression that may contain yield, returning the transformed result
    fn visit_expression_for_yield(
        &mut self,
        expr_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        if expr_idx.is_none() {
            return None;
        }

        match ctx.arena.get(expr_idx) {
            Some(Node::YieldExpression(yield_expr)) => {
                let is_delegating = yield_expr.asterisk_token;
                let expression = if yield_expr.expression.is_none() {
                    None
                } else {
                    Some(yield_expr.expression)
                };

                // Emit the yield operation
                self.emit_op(Operation::Yield {
                    expression,
                    is_delegating,
                });

                // After yield, we need a new label for the resume point
                let resume_label = self.new_label();
                self.mark_label(resume_label);

                // The result of yield is accessed via _a.sent()
                None // Placeholder - caller should use sent() call
            }
            _ => {
                // Not a yield, emit as statement
                Some(expr_idx)
            }
        }
    }

    /// Visit an if statement
    fn visit_if_statement(
        &mut self,
        stmt_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) {
        // Clone needed data to avoid borrow issues
        let (expression, then_statement, else_statement) = {
            if let Some(Node::IfStatement(if_stmt)) = ctx.arena.get(stmt_idx) {
                (if_stmt.expression, if_stmt.then_statement, if_stmt.else_statement)
            } else {
                return;
            }
        };

        let else_label = self.new_label();
        let end_label = self.new_label();

        // Branch to else if condition is false
        self.emit_op(Operation::BreakWhenFalse {
            label: else_label,
            condition: expression,
        });

        // Then branch
        self.visit_statement(then_statement, ctx);

        if !else_statement.is_none() {
            // Jump over else
            self.emit_op(Operation::Break(end_label));
        }

        // Else branch
        self.mark_label(else_label);
        if !else_statement.is_none() {
            self.visit_statement(else_statement, ctx);
        }

        self.mark_label(end_label);
    }

    /// Visit a loop statement
    fn visit_loop_statement(&mut self, stmt_idx: NodeIndex, ctx: &mut TransformContext) {
        // Extract loop data first to avoid borrow issues
        #[derive(Clone)]
        enum LoopKind {
            While { expression: NodeIndex, statement: NodeIndex },
            For { initializer: NodeIndex, condition: NodeIndex, incrementor: NodeIndex, statement: NodeIndex },
            Do { expression: NodeIndex, statement: NodeIndex },
            Unknown,
        }

        let loop_kind = match ctx.arena.get(stmt_idx) {
            Some(Node::WhileStatement(ws)) => LoopKind::While {
                expression: ws.expression,
                statement: ws.statement,
            },
            Some(Node::ForStatement(fs)) => LoopKind::For {
                initializer: fs.initializer,
                condition: fs.condition,
                incrementor: fs.incrementor,
                statement: fs.statement,
            },
            Some(Node::DoStatement(ds)) => LoopKind::Do {
                expression: ds.expression,
                statement: ds.statement,
            },
            _ => LoopKind::Unknown,
        };

        let continue_label = self.new_label();
        let break_label = self.new_label();

        self.block_stack.push(CodeBlock::Loop {
            continue_label,
            break_label,
        });

        self.mark_label(continue_label);

        match loop_kind {
            LoopKind::While { expression, statement } => {
                self.emit_op(Operation::BreakWhenFalse {
                    label: break_label,
                    condition: expression,
                });
                self.visit_statement(statement, ctx);
                self.emit_op(Operation::Break(continue_label));
            }
            LoopKind::For { initializer, condition, incrementor, statement } => {
                if !initializer.is_none() {
                    self.emit_op(Operation::Statement(initializer));
                }

                let condition_label = self.new_label();
                self.mark_label(condition_label);

                if !condition.is_none() {
                    self.emit_op(Operation::BreakWhenFalse {
                        label: break_label,
                        condition,
                    });
                }

                self.visit_statement(statement, ctx);
                self.mark_label(continue_label);

                if !incrementor.is_none() {
                    self.emit_op(Operation::Statement(incrementor));
                }

                self.emit_op(Operation::Break(condition_label));
            }
            LoopKind::Do { expression, statement } => {
                self.visit_statement(statement, ctx);
                self.mark_label(continue_label);

                self.emit_op(Operation::BreakWhenTrue {
                    label: continue_label,
                    condition: expression,
                });
            }
            LoopKind::Unknown => {}
        }

        self.mark_label(break_label);
        self.block_stack.pop();
    }

    /// Visit a try statement
    fn visit_try_statement(
        &mut self,
        stmt_idx: NodeIndex,
        ctx: &mut TransformContext,
    ) {
        // Clone needed data to avoid borrow issues
        let (try_block, catch_clause, finally_block) = {
            if let Some(Node::TryStatement(try_stmt)) = ctx.arena.get(stmt_idx) {
                (try_stmt.try_block, try_stmt.catch_clause, try_stmt.finally_block)
            } else {
                return;
            }
        };

        let try_label = self.new_label();
        let catch_label = if !catch_clause.is_none() {
            Some(self.new_label())
        } else {
            None
        };
        let finally_label = if !finally_block.is_none() {
            Some(self.new_label())
        } else {
            None
        };
        let end_label = self.new_label();

        self.block_stack.push(CodeBlock::Exception {
            try_label,
            catch_label,
            finally_label,
            end_label,
            catch_variable: None,
        });

        // Try block
        self.mark_label(try_label);
        self.visit_block(try_block, ctx);
        self.emit_op(Operation::Break(end_label));

        // Catch block
        if let Some(catch_lbl) = catch_label {
            self.mark_label(catch_lbl);
            // Get catch clause block
            let catch_block = {
                if let Some(Node::CatchClause(cc)) = ctx.arena.get(catch_clause) {
                    cc.block
                } else {
                    NodeIndex::NONE
                }
            };
            self.visit_block(catch_block, ctx);
            self.emit_op(Operation::Break(end_label));
        }

        // Finally block
        if let Some(finally_lbl) = finally_label {
            self.mark_label(finally_lbl);
            self.visit_block(finally_block, ctx);
            self.emit_op(Operation::EndFinally);
        }

        self.mark_label(end_label);
        self.block_stack.pop();
    }

    /// Visit a break statement
    fn visit_break_statement(
        &mut self,
        break_stmt: &crate::parser::ast::BreakStatement,
        ctx: &TransformContext,
    ) {
        // Find the target label
        let target = self.find_break_target(break_stmt.label, ctx);
        self.emit_op(Operation::Break(target));
    }

    /// Visit a continue statement
    fn visit_continue_statement(
        &mut self,
        cont_stmt: &crate::parser::ast::ContinueStatement,
        ctx: &TransformContext,
    ) {
        // Find the continue target
        let target = self.find_continue_target(cont_stmt.label, ctx);
        self.emit_op(Operation::Break(target));
    }

    /// Find the break target for a break statement
    fn find_break_target(&self, label: NodeIndex, ctx: &TransformContext) -> u32 {
        // If labeled, find by name
        if !label.is_none() {
            if let Some(Node::Identifier(ident)) = ctx.arena.get(label) {
                for block in self.block_stack.iter().rev() {
                    match block {
                        CodeBlock::Labeled { name, break_label } if *name == ident.escaped_text => {
                            return *break_label;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Find nearest breakable block
        for block in self.block_stack.iter().rev() {
            match block {
                CodeBlock::Loop { break_label, .. } => return *break_label,
                CodeBlock::Switch { break_label } => return *break_label,
                _ => {}
            }
        }

        0 // Should not happen
    }

    /// Find the continue target
    fn find_continue_target(&self, label: NodeIndex, ctx: &TransformContext) -> u32 {
        // If labeled, find by name
        if !label.is_none() {
            if let Some(Node::Identifier(ident)) = ctx.arena.get(label) {
                for block in self.block_stack.iter().rev() {
                    match block {
                        CodeBlock::Labeled { name, .. } if *name == ident.escaped_text => {
                            // Find the loop inside this label
                            // For now, return 0
                            return 0;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Find nearest loop
        for block in self.block_stack.iter().rev() {
            match block {
                CodeBlock::Loop { continue_label, .. } => return *continue_label,
                _ => {}
            }
        }

        0
    }

    /// Build the state machine from collected operations
    fn build_state_machine(
        &self,
        _original_func: NodeIndex,
        ctx: &mut TransformContext,
    ) -> Option<NodeIndex> {
        // For now, return None to indicate the transform is not complete
        // A full implementation would build the switch statement and __generator call
        //
        // The structure would be:
        // return __generator(this, function(_a) {
        //     switch (_a.label) {
        //         case 0: ...
        //         case 1: ...
        //     }
        // });

        // For now, just mark that we need the helper
        ctx.helpers_needed.generator = true;

        None
    }
}

impl Default for GeneratorTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Transformer for GeneratorTransformer {
    fn visit_node(&mut self, node_idx: NodeIndex, ctx: &mut TransformContext) -> Option<NodeIndex> {
        if self.is_generator_function(node_idx, ctx) {
            return self.transform_generator_function(node_idx, ctx);
        }

        self.visit_children(node_idx, ctx);
        None
    }

    fn visit_children(&mut self, node_idx: NodeIndex, ctx: &mut TransformContext) {
        // Get child nodes based on node type
        let children = get_children(node_idx, ctx);
        for child_idx in children {
            self.visit_node(child_idx, ctx);
        }
    }
}

/// Get child nodes for traversal
fn get_children(node_idx: NodeIndex, ctx: &TransformContext) -> Vec<NodeIndex> {
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
        Some(Node::FunctionExpression(f)) => {
            if !f.body.is_none() {
                children.push(f.body);
            }
        }
        Some(Node::ClassDeclaration(c)) => {
            children.extend(c.members.nodes.iter().copied());
        }
        _ => {}
    }

    children
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_codes() {
        assert_eq!(Instruction::Next as u8, 0);
        assert_eq!(Instruction::Yield as u8, 4);
        assert_eq!(Instruction::Endfinally as u8, 7);
    }
}
