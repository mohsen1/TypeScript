use super::*;
use std::mem::size_of;

#[test]
fn test_thin_node_size() {
    // This is the critical test - ThinNode MUST be 16 bytes
    assert_eq!(
        size_of::<ThinNode>(),
        16,
        "ThinNode must be exactly 16 bytes"
    );

    // 4 nodes per cache line
    let nodes_per_cache_line = 64 / size_of::<ThinNode>();
    assert_eq!(
        nodes_per_cache_line, 4,
        "Should fit 4 ThinNodes per 64-byte cache line"
    );
}

#[test]
fn test_thin_node_arena_basic() {
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Add a token (no data)
    let token = arena.add_token(SyntaxKind::AsteriskToken as u16, 0, 5);
    assert_eq!(token.0, 0);

    // Add an identifier
    let ident = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        10,
        15,
        IdentifierData {
            escaped_text: "hello".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );
    assert_eq!(ident.0, 1);

    // Verify we can retrieve them
    let node = arena.get(token).unwrap();
    assert_eq!(node.kind, SyntaxKind::AsteriskToken as u16);
    assert_eq!(node.pos, 0);
    assert_eq!(node.end, 5);
    assert!(!node.has_data());

    let node = arena.get(ident).unwrap();
    assert_eq!(node.kind, SyntaxKind::Identifier as u16);
    assert!(node.has_data());

    let data = arena.get_identifier(node).unwrap();
    assert_eq!(data.escaped_text, "hello");
}

#[test]
fn test_data_pool_sizes() {
    // Verify data pool element sizes are reasonable
    assert!(
        size_of::<IdentifierData>() <= 120,
        "IdentifierData too large"
    );
    assert!(size_of::<FunctionData>() <= 168, "FunctionData too large");
    assert!(size_of::<ClassData>() <= 200, "ClassData too large");
    assert!(
        size_of::<SourceFileData>() <= 200,
        "SourceFileData too large"
    );
}

#[test]
fn test_node_view() {
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Add an identifier
    let ident_idx = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        10,
        15,
        IdentifierData {
            escaped_text: "myVar".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    // Create a view and access data through it
    let view = NodeView::new(&arena, ident_idx).unwrap();
    assert_eq!(view.kind(), SyntaxKind::Identifier as u16);
    assert_eq!(view.pos(), 10);
    assert_eq!(view.end(), 15);
    assert!(view.has_data());

    let ident = view.as_identifier().unwrap();
    assert_eq!(ident.escaped_text, "myVar");
}

#[test]
fn test_node_kind_utilities() {
    use super::super::syntax_kind_ext::*;
    use crate::scanner::SyntaxKind;

    let ident = ThinNode::new(SyntaxKind::Identifier as u16, 0, 5);
    assert!(ident.is_identifier());
    assert!(!ident.is_string_literal());

    let func = ThinNode::new(FUNCTION_DECLARATION, 0, 100);
    assert!(func.is_function_declaration());
    assert!(func.is_function_like());
    assert!(func.is_declaration());

    let class = ThinNode::new(CLASS_DECLARATION, 0, 200);
    assert!(class.is_class_declaration());
    assert!(class.is_declaration());

    let block = ThinNode::new(BLOCK, 0, 50);
    assert!(block.is_statement());

    let type_ref = ThinNode::new(TYPE_REFERENCE, 0, 10);
    assert!(type_ref.is_type_node());
}

#[test]
fn test_node_access_trait() {
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Add an identifier
    let ident_idx = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        10,
        20,
        IdentifierData {
            escaped_text: "testVar".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    // Test NodeAccess trait methods
    assert!(arena.exists(ident_idx));
    assert!(!arena.exists(NodeIndex::NONE));

    assert_eq!(arena.kind(ident_idx), Some(SyntaxKind::Identifier as u16));
    assert_eq!(arena.pos_end(ident_idx), Some((10, 20)));
    assert_eq!(arena.get_identifier_text(ident_idx), Some("testVar"));

    // Test NodeInfo
    let info = arena.node_info(ident_idx).unwrap();
    assert_eq!(info.kind, SyntaxKind::Identifier as u16);
    assert_eq!(info.pos, 10);
    assert_eq!(info.end, 20);
}

#[test]
fn test_parent_mapping() {
    use crate::parser::syntax_kind_ext::BINARY_EXPRESSION;
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Create a simple expression tree: (a + b)
    // Binary expression with two identifier children
    let left_ident = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        0,
        1,
        IdentifierData {
            escaped_text: "a".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    let right_ident = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        4,
        5,
        IdentifierData {
            escaped_text: "b".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    let binary_expr = arena.add_binary_expr(
        BINARY_EXPRESSION,
        0,
        5,
        BinaryExprData {
            left: left_ident,
            operator_token: SyntaxKind::PlusToken as u16,
            right: right_ident,
        },
    );

    // Verify parent mapping
    let left_extended = arena.get_extended(left_ident).unwrap();
    assert_eq!(
        left_extended.parent, binary_expr,
        "Left identifier should have binary expression as parent"
    );

    let right_extended = arena.get_extended(right_ident).unwrap();
    assert_eq!(
        right_extended.parent, binary_expr,
        "Right identifier should have binary expression as parent"
    );

    // Verify binary expression has no parent (it's the root)
    let binary_extended = arena.get_extended(binary_expr).unwrap();
    assert!(
        binary_extended.parent.is_none(),
        "Binary expression should have no parent (it's the root)"
    );
}

#[test]
fn test_parent_mapping_nested() {
    use crate::parser::syntax_kind_ext::BINARY_EXPRESSION;
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Create nested expression: (a + b) * c
    // This creates a tree where:
    //   multiply
    //   ├─ add
    //   │  ├─ a
    //   │  └─ b
    //   └─ c

    let a = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        0,
        1,
        IdentifierData {
            escaped_text: "a".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );
    let b = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        4,
        5,
        IdentifierData {
            escaped_text: "b".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );
    let add = arena.add_binary_expr(
        BINARY_EXPRESSION,
        0,
        5,
        BinaryExprData {
            left: a,
            operator_token: SyntaxKind::PlusToken as u16,
            right: b,
        },
    );

    let c = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        9,
        10,
        IdentifierData {
            escaped_text: "c".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );
    let multiply = arena.add_binary_expr(
        BINARY_EXPRESSION,
        0,
        10,
        BinaryExprData {
            left: add,
            operator_token: SyntaxKind::AsteriskToken as u16,
            right: c,
        },
    );

    // Verify parent chain: a -> add -> multiply
    assert_eq!(arena.get_extended(a).unwrap().parent, add);
    assert_eq!(arena.get_extended(b).unwrap().parent, add);
    assert_eq!(arena.get_extended(add).unwrap().parent, multiply);
    assert_eq!(arena.get_extended(c).unwrap().parent, multiply);
    assert!(arena.get_extended(multiply).unwrap().parent.is_none());
}

#[test]
fn test_parent_mapping_function() {
    use crate::parser::NodeList;
    use crate::parser::syntax_kind_ext::{BLOCK, FUNCTION_DECLARATION, RETURN_STATEMENT};
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Create a simple function: function foo() { return 42; }
    let name = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        9,
        12,
        IdentifierData {
            escaped_text: "foo".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    let literal = arena.add_literal(
        SyntaxKind::NumericLiteral as u16,
        23,
        25,
        LiteralData {
            text: "42".to_string(),
            raw_text: None,
            value: Some(42.0),
        },
    );

    let return_stmt = arena.add_return(
        RETURN_STATEMENT,
        16,
        26,
        ReturnData {
            expression: literal,
        },
    );

    let block = arena.add_block(
        BLOCK,
        14,
        28,
        BlockData {
            statements: NodeList {
                nodes: vec![return_stmt],
                pos: 16,
                end: 26,
                has_trailing_comma: false,
            },
            multi_line: true,
        },
    );

    let func = arena.add_function(
        FUNCTION_DECLARATION,
        0,
        28,
        FunctionData {
            modifiers: None,
            is_async: false,
            asterisk_token: false,
            name,
            type_parameters: None,
            parameters: NodeList {
                nodes: vec![],
                pos: 13,
                end: 14,
                has_trailing_comma: false,
            },
            type_annotation: NodeIndex::NONE,
            body: block,
            equals_greater_than_token: false,
        },
    );

    // Verify parent chain
    assert_eq!(
        arena.get_extended(name).unwrap().parent,
        func,
        "Function name should have function as parent"
    );
    assert_eq!(
        arena.get_extended(block).unwrap().parent,
        func,
        "Function body should have function as parent"
    );
    assert_eq!(
        arena.get_extended(return_stmt).unwrap().parent,
        block,
        "Return statement should have block as parent"
    );
    assert_eq!(
        arena.get_extended(literal).unwrap().parent,
        return_stmt,
        "Literal should have return statement as parent"
    );
    assert!(
        arena.get_extended(func).unwrap().parent.is_none(),
        "Function should have no parent"
    );
}

#[test]
fn test_get_children_binary_expression() {
    use crate::parser::syntax_kind_ext::BINARY_EXPRESSION;
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Create a binary expression: a + b
    let left = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        0,
        1,
        IdentifierData {
            escaped_text: "a".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    let right = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        4,
        5,
        IdentifierData {
            escaped_text: "b".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    let binary = arena.add_binary_expr(
        BINARY_EXPRESSION,
        0,
        5,
        BinaryExprData {
            left,
            operator_token: SyntaxKind::PlusToken as u16,
            right,
        },
    );

    // Test get_children
    let children = arena.get_children(binary);
    assert_eq!(children.len(), 2, "Binary expression should have 2 children");
    assert_eq!(children[0], left, "First child should be left operand");
    assert_eq!(children[1], right, "Second child should be right operand");
}

#[test]
fn test_get_children_block() {
    use crate::parser::NodeList;
    use crate::parser::syntax_kind_ext::{BLOCK, EXPRESSION_STATEMENT};
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Create expression statements
    let expr1 = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        0,
        1,
        IdentifierData {
            escaped_text: "x".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );
    let stmt1 = arena.add_expr_statement(
        EXPRESSION_STATEMENT,
        0,
        2,
        ExprStatementData { expression: expr1 },
    );

    let expr2 = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        3,
        4,
        IdentifierData {
            escaped_text: "y".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );
    let stmt2 = arena.add_expr_statement(
        EXPRESSION_STATEMENT,
        3,
        5,
        ExprStatementData { expression: expr2 },
    );

    // Create a block with statements
    let block = arena.add_block(
        BLOCK,
        0,
        6,
        BlockData {
            statements: NodeList {
                nodes: vec![stmt1, stmt2],
                pos: 0,
                end: 6,
                has_trailing_comma: false,
            },
            multi_line: true,
        },
    );

    // Test get_children
    let children = arena.get_children(block);
    assert_eq!(children.len(), 2, "Block should have 2 children (statements)");
    assert_eq!(children[0], stmt1, "First child should be first statement");
    assert_eq!(children[1], stmt2, "Second child should be second statement");
}

#[test]
fn test_get_children_if_statement() {
    use crate::parser::syntax_kind_ext::{BLOCK, IF_STATEMENT};
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Create condition
    let condition = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        4,
        5,
        IdentifierData {
            escaped_text: "x".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    // Create then-block
    let then_block = arena.add_block(
        BLOCK,
        6,
        8,
        BlockData {
            statements: super::super::NodeList {
                nodes: vec![],
                pos: 6,
                end: 8,
                has_trailing_comma: false,
            },
            multi_line: false,
        },
    );

    // Create else-block
    let else_block = arena.add_block(
        BLOCK,
        14,
        16,
        BlockData {
            statements: super::super::NodeList {
                nodes: vec![],
                pos: 14,
                end: 16,
                has_trailing_comma: false,
            },
            multi_line: false,
        },
    );

    // Create if statement with else clause
    let if_stmt = arena.add_if_statement(
        IF_STATEMENT,
        0,
        16,
        IfStatementData {
            expression: condition,
            then_statement: then_block,
            else_statement: else_block,
        },
    );

    // Test get_children with else
    let children = arena.get_children(if_stmt);
    assert_eq!(children.len(), 3, "If statement with else should have 3 children");
    assert_eq!(children[0], condition, "First child should be condition");
    assert_eq!(children[1], then_block, "Second child should be then block");
    assert_eq!(children[2], else_block, "Third child should be else block");
}

#[test]
fn test_get_children_call_expression() {
    use crate::parser::NodeList;
    use crate::parser::syntax_kind_ext::CALL_EXPRESSION;
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Create function identifier
    let func_name = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        0,
        3,
        IdentifierData {
            escaped_text: "foo".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    // Create arguments
    let arg1 = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        4,
        5,
        IdentifierData {
            escaped_text: "a".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );
    let arg2 = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        7,
        8,
        IdentifierData {
            escaped_text: "b".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    // Create call expression: foo(a, b)
    let call = arena.add_call_expr(
        CALL_EXPRESSION,
        0,
        9,
        CallExprData {
            expression: func_name,
            type_arguments: None,
            arguments: Some(NodeList {
                nodes: vec![arg1, arg2],
                pos: 3,
                end: 9,
                has_trailing_comma: false,
            }),
        },
    );

    // Test get_children
    let children = arena.get_children(call);
    assert_eq!(children.len(), 3, "Call expression should have 3 children");
    assert_eq!(children[0], func_name, "First child should be function expression");
    assert_eq!(children[1], arg1, "Second child should be first argument");
    assert_eq!(children[2], arg2, "Third child should be second argument");
}

#[test]
fn test_get_children_empty_for_token() {
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Create a simple token (has no children)
    let token = arena.add_token(SyntaxKind::PlusToken as u16, 0, 1);

    // Test get_children
    let children = arena.get_children(token);
    assert!(children.is_empty(), "Token should have no children");
}

#[test]
fn test_get_children_empty_for_identifier() {
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Create an identifier (has no children)
    let ident = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        0,
        5,
        IdentifierData {
            escaped_text: "hello".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    // Test get_children
    let children = arena.get_children(ident);
    assert!(children.is_empty(), "Identifier should have no children");
}

#[test]
fn test_get_children_unary_expressions() {
    use crate::parser::syntax_kind_ext::{
        DELETE_EXPRESSION, PREFIX_UNARY_EXPRESSION, TYPE_OF_EXPRESSION, VOID_EXPRESSION,
    };
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Create operand
    let operand = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        2,
        3,
        IdentifierData {
            escaped_text: "x".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    // Test PREFIX_UNARY_EXPRESSION (!x)
    let prefix = arena.add_unary_expr(
        PREFIX_UNARY_EXPRESSION,
        0,
        3,
        UnaryExprData {
            operator: SyntaxKind::ExclamationToken as u16,
            operand,
        },
    );
    let children = arena.get_children(prefix);
    assert_eq!(children.len(), 1, "Prefix unary expression should have 1 child");
    assert_eq!(children[0], operand, "Child should be the operand");

    // Test DELETE_EXPRESSION (delete x)
    let operand2 = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        10,
        11,
        IdentifierData {
            escaped_text: "y".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );
    let delete_expr = arena.add_unary_expr(
        DELETE_EXPRESSION,
        7,
        11,
        UnaryExprData {
            operator: SyntaxKind::DeleteKeyword as u16,
            operand: operand2,
        },
    );
    let children = arena.get_children(delete_expr);
    assert_eq!(children.len(), 1, "Delete expression should have 1 child");
    assert_eq!(children[0], operand2, "Child should be the operand");

    // Test TYPEOF_EXPRESSION (typeof x)
    let operand3 = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        20,
        21,
        IdentifierData {
            escaped_text: "z".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );
    let typeof_expr = arena.add_unary_expr(
        TYPE_OF_EXPRESSION,
        14,
        21,
        UnaryExprData {
            operator: SyntaxKind::TypeOfKeyword as u16,
            operand: operand3,
        },
    );
    let children = arena.get_children(typeof_expr);
    assert_eq!(children.len(), 1, "Typeof expression should have 1 child");
    assert_eq!(children[0], operand3, "Child should be the operand");

    // Test VOID_EXPRESSION (void x)
    let operand4 = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        30,
        31,
        IdentifierData {
            escaped_text: "w".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );
    let void_expr = arena.add_unary_expr(
        VOID_EXPRESSION,
        25,
        31,
        UnaryExprData {
            operator: SyntaxKind::VoidKeyword as u16,
            operand: operand4,
        },
    );
    let children = arena.get_children(void_expr);
    assert_eq!(children.len(), 1, "Void expression should have 1 child");
    assert_eq!(children[0], operand4, "Child should be the operand");
}

#[test]
fn test_get_children_optional_fields() {
    use crate::parser::NodeList;
    use crate::parser::syntax_kind_ext::IF_STATEMENT;
    use crate::scanner::SyntaxKind;

    let mut arena = ThinNodeArena::new();

    // Create condition
    let condition = arena.add_identifier(
        SyntaxKind::Identifier as u16,
        4,
        5,
        IdentifierData {
            escaped_text: "x".to_string(),
            original_text: None,
            type_arguments: None,
        },
    );

    // Create then-block
    let then_block = arena.add_block(
        crate::parser::syntax_kind_ext::BLOCK,
        6,
        8,
        BlockData {
            statements: NodeList {
                nodes: vec![],
                pos: 6,
                end: 8,
                has_trailing_comma: false,
            },
            multi_line: false,
        },
    );

    // Create if statement WITHOUT else clause (optional field is NONE)
    let if_stmt = arena.add_if_statement(
        IF_STATEMENT,
        0,
        8,
        IfStatementData {
            expression: condition,
            then_statement: then_block,
            else_statement: NodeIndex::NONE, // No else clause
        },
    );

    // Test get_children - should have 2 children (condition and then-block)
    let children = arena.get_children(if_stmt);
    assert_eq!(
        children.len(),
        2,
        "If statement without else should have 2 children"
    );
    assert_eq!(children[0], condition, "First child should be condition");
    assert_eq!(children[1], then_block, "Second child should be then block");
    // Note: else_statement is NONE, so it shouldn't be included
}

#[test]
fn test_get_children_none_index() {
    let arena = ThinNodeArena::new();

    // Test with NONE index
    let children = arena.get_children(NodeIndex::NONE);
    assert!(children.is_empty(), "NONE index should return empty children");
}
