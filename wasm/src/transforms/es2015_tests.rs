use crate::emitter::ScriptTarget;
use crate::parser_impl::ParserState;
use crate::parser::{Node, NodeArena};

fn parse_and_transform(source: &str, target: ScriptTarget) -> (super::super::HelpersNeeded, NodeArena) {
    let mut parser = ParserState::new("test.ts".to_string(), source.to_string());
    let source_file = parser.parse_source_file();
    let mut arena = parser.arena;

    let helpers = super::super::transform_source_file(source_file, target, &mut arena);
    (helpers, arena)
}

#[test]
fn test_transformer_creates_context() {
    let (helpers, _) = parse_and_transform("let x = 1;", ScriptTarget::ES5);
    // Just verify it doesn't crash
    assert!(!helpers.extends);
}

#[test]
fn test_arrow_function_transform() {
    let (_, arena) = parse_and_transform("const fn = (x) => x * 2;", ScriptTarget::ES5);

    // Find if a FunctionExpression was created (arrow → function transform)
    let has_func_expr = arena.nodes.iter().any(|node| {
        matches!(node, Node::FunctionExpression(_))
    });

    // The transform should have replaced the ArrowFunction with FunctionExpression
    assert!(has_func_expr, "Arrow function should be transformed to FunctionExpression");
}

#[test]
fn test_arrow_function_with_block_body() {
    let (_, arena) = parse_and_transform("const fn = (x) => { return x * 2; };", ScriptTarget::ES5);

    // Find if a FunctionExpression was created
    let has_func_expr = arena.nodes.iter().any(|node| {
        matches!(node, Node::FunctionExpression(_))
    });

    assert!(has_func_expr, "Arrow function with block body should be transformed");
}

#[test]
fn test_arrow_function_expression_body_gets_return() {
    let (_, arena) = parse_and_transform("const add = (a, b) => a + b;", ScriptTarget::ES5);

    // The expression body should be wrapped in a return statement
    let has_return_stmt = arena.nodes.iter().any(|node| {
        matches!(node, Node::ReturnStatement(_))
    });

    assert!(has_return_stmt, "Expression body should be wrapped in return statement");
}

#[test]
fn test_for_of_needs_values_helper() {
    let (helpers, _) = parse_and_transform("for (const x of arr) { console.log(x); }", ScriptTarget::ES5);
    // For-of should require __values helper
    assert!(helpers.values);
}

#[test]
fn test_no_transform_for_es2015_target() {
    let (_, arena) = parse_and_transform("const fn = (x) => x * 2;", ScriptTarget::ES2015);

    // Arrow functions should NOT be transformed when targeting ES2015
    let has_func_expr = arena.nodes.iter().any(|node| {
        matches!(node, Node::FunctionExpression(_))
    });

    assert!(!has_func_expr, "Arrow function should NOT be transformed for ES2015 target");
}
