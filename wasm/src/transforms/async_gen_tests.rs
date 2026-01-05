use crate::emitter::ScriptTarget;
use crate::parser_impl::ParserState;
use crate::transforms::{transform_source_file, HelpersNeeded};

fn parse_and_transform(source: &str, target: ScriptTarget) -> HelpersNeeded {
    let mut parser = ParserState::new("test.ts".to_string(), source.to_string());
    let source_file = parser.parse_source_file();
    let mut arena = parser.arena;
    transform_source_file(source_file, target, &mut arena)
}

#[test]
fn test_async_function_needs_awaiter() {
    let helpers = parse_and_transform(
        "async function foo() { await bar(); }",
        ScriptTarget::ES5,
    );
    assert!(helpers.awaiter);
    assert!(helpers.generator);
}

#[test]
fn test_generator_function_needs_generator() {
    let helpers = parse_and_transform(
        "function* gen() { yield 1; }",
        ScriptTarget::ES5,
    );
    assert!(helpers.generator);
}

#[test]
fn test_for_await_of_needs_async_values() {
    let helpers = parse_and_transform(
        "async function foo() { for await (const x of iter) {} }",
        ScriptTarget::ES5,
    );
    assert!(helpers.async_values);
}

#[test]
fn test_await_to_yield_transform() {
    use crate::parser::{Node, NodeBase, NodeList, syntax_kind_ext};
    use crate::parser::ast::{AwaitExpression, Identifier};
    use crate::scanner::SyntaxKind;
    use super::AsyncTransformer;
    use super::super::TransformContext;

    let mut parser = ParserState::new("test.ts".to_string(), "".to_string());
    let _source_file = parser.parse_source_file();
    let mut arena = parser.arena;

    // Manually create an await expression for testing
    let bar_id = Identifier {
        base: NodeBase::new(SyntaxKind::Identifier, 0, 3),
        escaped_text: "bar".to_string(),
        original_text: None,
        type_arguments: None,
    };
    let bar_idx = arena.add(Node::Identifier(bar_id));

    let await_expr = AwaitExpression {
        base: NodeBase::new_ext(syntax_kind_ext::AWAIT_EXPRESSION, 0, 10),
        expression: bar_idx,
    };
    let await_idx = arena.add(Node::AwaitExpression(await_expr));

    // Now transform
    let mut ctx = TransformContext::new(ScriptTarget::ES5, &mut arena);
    let mut transformer = AsyncTransformer::new();
    let result = transformer.transform_await_expression(await_idx, &mut ctx);

    // Verify transformation produced a yield expression
    assert!(result.is_some(), "Await transform should produce a result");
    let result_idx = result.unwrap();
    assert!(matches!(ctx.arena.get(result_idx), Some(Node::YieldExpression(_))),
        "Await should be transformed to yield");
}

#[test]
fn test_awaiter_helper_creation() {
    use super::AsyncTransformer;
    use super::super::TransformContext;
    use crate::parser::Node;

    let mut parser = ParserState::new("test.ts".to_string(), "{}".to_string());
    let source_file = parser.parse_source_file();
    let mut arena = parser.arena;

    let mut ctx = TransformContext::new(ScriptTarget::ES5, &mut arena);
    let transformer = AsyncTransformer::new();

    // Get the block from source file
    if let Some(Node::SourceFile(sf)) = ctx.arena.get(source_file) {
        if let Some(&block_idx) = sf.statements.nodes.first() {
            // Create awaiter call
            let awaiter_return = transformer.create_awaiter_return(block_idx, &mut ctx);

            // Verify it's a return statement
            assert!(matches!(ctx.arena.get(awaiter_return), Some(Node::ReturnStatement(_))),
                "Should create a return statement");
        }
    }
}

// Note: async arrow test would require more complex modifier parsing
// which is handled but the test depends on the specific AST structure
