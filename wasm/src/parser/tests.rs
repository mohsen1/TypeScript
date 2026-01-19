//! Tests for parser module.

use super::*;
use crate::scanner::SyntaxKind;

#[test]
fn test_node_flags() {
    assert_eq!(node_flags::NONE, 0);
    assert_eq!(node_flags::LET, 1);
    assert_eq!(node_flags::CONST, 2);
    assert_eq!(node_flags::AWAIT_USING, 6); // Const | Using
}

#[test]
fn test_modifier_flags() {
    assert_eq!(modifier_flags::NONE, 0);
    assert_eq!(modifier_flags::PUBLIC, 1);
    assert_eq!(modifier_flags::EXPORT, 32);
    assert_eq!(modifier_flags::ASYNC, 1024);
}

#[test]
fn test_node_index() {
    let index = NodeIndex(0);
    assert!(index.is_some());
    assert!(!index.is_none());

    let none = NodeIndex::NONE;
    assert!(none.is_none());
    assert!(!none.is_some());
}

#[test]
fn test_node_arena() {
    let mut arena = NodeArena::new();

    let id = Identifier::new("test".to_string(), 0, 4);
    let idx = arena.add(Node::Identifier(id));

    assert_eq!(idx.0, 0);
    assert_eq!(arena.len(), 1);

    let node = arena.get(idx).unwrap();
    assert_eq!(node.kind(), SyntaxKind::Identifier as u16);
    assert_eq!(node.pos(), 0);
    assert_eq!(node.end(), 4);
}

#[test]
fn test_identifier() {
    let id = Identifier::new("myVar".to_string(), 10, 15);
    assert_eq!(id.escaped_text, "myVar");
    assert_eq!(id.base.kind, SyntaxKind::Identifier as u16);
    assert_eq!(id.base.pos, 10);
    assert_eq!(id.base.end, 15);
}

#[test]
fn test_source_file() {
    let sf = SourceFile::new("test.ts".to_string(), "const x = 1;".to_string());
    // SourceFile kind is 308 in TypeScript, stored directly in kind field
    assert_eq!(sf.base.kind as u16, syntax_kind_ext::SOURCE_FILE);
    assert_eq!(sf.file_name, "test.ts");
    assert_eq!(sf.text.len(), 12);
}

// Tests for NodeArena::get_children

use super::arena::NodeArena;
use super::thin_node::NodeAccess;
use super::ast::{
    expressions::*, statements::*, declarations::*, base::NodeList,
};

#[test]
fn test_get_children_empty_for_tokens() {
    let mut arena = NodeArena::new();

    // Identifiers have no children
    let id = Identifier::new("test".to_string(), 0, 4);
    let idx = arena.add(Node::Identifier(id));

    let children = arena.get_children(idx);
    assert!(children.is_empty());
}

#[test]
fn test_get_children_binary_expression() {
    let mut arena = NodeArena::new();

    // Create left and right identifiers
    let left_id = Identifier::new("a".to_string(), 0, 1);
    let left_idx = arena.add(Node::Identifier(left_id));

    let right_id = Identifier::new("b".to_string(), 4, 5);
    let right_idx = arena.add(Node::Identifier(right_id));

    // Create binary expression: a + b
    let binary = BinaryExpression {
        base: NodeBase::new_ext(syntax_kind_ext::BINARY_EXPRESSION, 0, 5),
        left: left_idx,
        operator_token: SyntaxKind::PlusToken,
        right: right_idx,
    };
    let binary_idx = arena.add(Node::BinaryExpression(binary));

    let children = arena.get_children(binary_idx);
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], left_idx);
    assert_eq!(children[1], right_idx);
}

#[test]
fn test_get_children_if_statement() {
    let mut arena = NodeArena::new();

    // Create condition
    let cond = Identifier::new("cond".to_string(), 4, 8);
    let cond_idx = arena.add(Node::Identifier(cond));

    // Create then statement (empty block)
    let then_block = Block {
        base: NodeBase::new_ext(syntax_kind_ext::BLOCK, 10, 12),
        statements: NodeList::new(),
        multi_line: false,
    };
    let then_idx = arena.add(Node::Block(then_block));

    // Create else statement (empty block)
    let else_block = Block {
        base: NodeBase::new_ext(syntax_kind_ext::BLOCK, 18, 20),
        statements: NodeList::new(),
        multi_line: false,
    };
    let else_idx = arena.add(Node::Block(else_block));

    // Create if statement with else
    let if_stmt = IfStatement {
        base: NodeBase::new_ext(syntax_kind_ext::IF_STATEMENT, 0, 20),
        expression: cond_idx,
        then_statement: then_idx,
        else_statement: else_idx,
    };
    let if_idx = arena.add(Node::IfStatement(if_stmt));

    let children = arena.get_children(if_idx);
    assert_eq!(children.len(), 3);
    assert_eq!(children[0], cond_idx);
    assert_eq!(children[1], then_idx);
    assert_eq!(children[2], else_idx);
}

#[test]
fn test_get_children_if_statement_no_else() {
    let mut arena = NodeArena::new();

    // Create condition
    let cond = Identifier::new("cond".to_string(), 4, 8);
    let cond_idx = arena.add(Node::Identifier(cond));

    // Create then statement
    let then_block = Block {
        base: NodeBase::new_ext(syntax_kind_ext::BLOCK, 10, 12),
        statements: NodeList::new(),
        multi_line: false,
    };
    let then_idx = arena.add(Node::Block(then_block));

    // Create if statement without else
    let if_stmt = IfStatement {
        base: NodeBase::new_ext(syntax_kind_ext::IF_STATEMENT, 0, 12),
        expression: cond_idx,
        then_statement: then_idx,
        else_statement: NodeIndex::NONE,
    };
    let if_idx = arena.add(Node::IfStatement(if_stmt));

    let children = arena.get_children(if_idx);
    assert_eq!(children.len(), 2); // Only condition and then, no else
    assert_eq!(children[0], cond_idx);
    assert_eq!(children[1], then_idx);
}

#[test]
fn test_get_children_block_with_statements() {
    let mut arena = NodeArena::new();

    // Create expression statements
    let expr1 = Identifier::new("a".to_string(), 2, 3);
    let expr1_idx = arena.add(Node::Identifier(expr1));
    let stmt1 = ExpressionStatement {
        base: NodeBase::new_ext(syntax_kind_ext::EXPRESSION_STATEMENT, 2, 4),
        expression: expr1_idx,
    };
    let stmt1_idx = arena.add(Node::ExpressionStatement(stmt1));

    let expr2 = Identifier::new("b".to_string(), 6, 7);
    let expr2_idx = arena.add(Node::Identifier(expr2));
    let stmt2 = ExpressionStatement {
        base: NodeBase::new_ext(syntax_kind_ext::EXPRESSION_STATEMENT, 6, 8),
        expression: expr2_idx,
    };
    let stmt2_idx = arena.add(Node::ExpressionStatement(stmt2));

    // Create block with statements
    let mut stmts = NodeList::new();
    stmts.push(stmt1_idx);
    stmts.push(stmt2_idx);

    let block = Block {
        base: NodeBase::new_ext(syntax_kind_ext::BLOCK, 0, 10),
        statements: stmts,
        multi_line: true,
    };
    let block_idx = arena.add(Node::Block(block));

    let children = arena.get_children(block_idx);
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], stmt1_idx);
    assert_eq!(children[1], stmt2_idx);
}

#[test]
fn test_get_children_function_declaration() {
    let mut arena = NodeArena::new();

    // Create function name
    let name = Identifier::new("foo".to_string(), 9, 12);
    let name_idx = arena.add(Node::Identifier(name));

    // Create parameter
    let param_name = Identifier::new("x".to_string(), 13, 14);
    let param_name_idx = arena.add(Node::Identifier(param_name));
    let param = ParameterDeclaration {
        base: NodeBase::new_ext(syntax_kind_ext::PARAMETER, 13, 14),
        modifiers: None,
        dot_dot_dot_token: false,
        name: param_name_idx,
        question_token: false,
        type_annotation: NodeIndex::NONE,
        initializer: NodeIndex::NONE,
    };
    let param_idx = arena.add(Node::ParameterDeclaration(param));

    // Create body
    let body = Block {
        base: NodeBase::new_ext(syntax_kind_ext::BLOCK, 16, 18),
        statements: NodeList::new(),
        multi_line: false,
    };
    let body_idx = arena.add(Node::Block(body));

    // Create parameter list
    let mut params = NodeList::new();
    params.push(param_idx);

    // Create function declaration
    let func = FunctionDeclaration {
        base: NodeBase::new_ext(syntax_kind_ext::FUNCTION_DECLARATION, 0, 18),
        modifiers: None,
        is_async: false,
        asterisk_token: false,
        name: name_idx,
        type_parameters: None,
        parameters: params,
        type_annotation: NodeIndex::NONE,
        body: body_idx,
    };
    let func_idx = arena.add(Node::FunctionDeclaration(func));

    let children = arena.get_children(func_idx);
    // Should include: name, parameters, body (no modifiers, type_parameters, or type_annotation)
    assert_eq!(children.len(), 3);
    assert!(children.contains(&name_idx));
    assert!(children.contains(&param_idx));
    assert!(children.contains(&body_idx));
}

#[test]
fn test_get_children_variable_declaration() {
    let mut arena = NodeArena::new();

    // Create name
    let name = Identifier::new("x".to_string(), 4, 5);
    let name_idx = arena.add(Node::Identifier(name));

    // Create initializer
    let init = NumericLiteral {
        base: NodeBase::new(SyntaxKind::NumericLiteral, 8, 9),
        text: "1".to_string(),
        value: 1.0,
    };
    let init_idx = arena.add(Node::NumericLiteral(init));

    // Create variable declaration
    let var_decl = VariableDeclaration {
        base: NodeBase::new_ext(syntax_kind_ext::VARIABLE_DECLARATION, 4, 9),
        name: name_idx,
        exclamation_token: false,
        type_annotation: NodeIndex::NONE,
        initializer: init_idx,
    };
    let var_idx = arena.add(Node::VariableDeclaration(var_decl));

    let children = arena.get_children(var_idx);
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], name_idx);
    assert_eq!(children[1], init_idx);
}

#[test]
fn test_get_children_call_expression() {
    let mut arena = NodeArena::new();

    // Create function name
    let func_name = Identifier::new("foo".to_string(), 0, 3);
    let func_idx = arena.add(Node::Identifier(func_name));

    // Create arguments
    let arg1 = Identifier::new("a".to_string(), 4, 5);
    let arg1_idx = arena.add(Node::Identifier(arg1));

    let arg2 = Identifier::new("b".to_string(), 7, 8);
    let arg2_idx = arena.add(Node::Identifier(arg2));

    let mut args = NodeList::new();
    args.push(arg1_idx);
    args.push(arg2_idx);

    // Create call expression
    let call = CallExpression {
        base: NodeBase::new_ext(syntax_kind_ext::CALL_EXPRESSION, 0, 9),
        expression: func_idx,
        type_arguments: None,
        arguments: args,
    };
    let call_idx = arena.add(Node::CallExpression(call));

    let children = arena.get_children(call_idx);
    assert_eq!(children.len(), 3);
    assert_eq!(children[0], func_idx);
    assert_eq!(children[1], arg1_idx);
    assert_eq!(children[2], arg2_idx);
}

#[test]
fn test_get_children_none_index() {
    let arena = NodeArena::new();
    let children = arena.get_children(NodeIndex::NONE);
    assert!(children.is_empty());
}

#[test]
fn test_get_children_invalid_index() {
    let arena = NodeArena::new();
    // Valid index but arena is empty
    let children = arena.get_children(NodeIndex(0));
    assert!(children.is_empty());
}
