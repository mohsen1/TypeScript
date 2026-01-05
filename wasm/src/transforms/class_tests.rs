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
fn test_class_without_heritage() {
    let helpers = parse_and_transform(
        "class Foo { constructor() {} method() {} }",
        ScriptTarget::ES5,
    );
    // No extends, so no __extends helper needed
    assert!(!helpers.extends);
}

#[test]
fn test_class_with_heritage_needs_extends() {
    let helpers = parse_and_transform(
        "class Foo extends Bar { constructor() { super(); } }",
        ScriptTarget::ES5,
    );
    // Class extends Bar, so __extends helper is needed
    assert!(helpers.extends);
}
