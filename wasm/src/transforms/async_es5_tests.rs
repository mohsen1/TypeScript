use super::*;
use crate::thin_parser::ThinParserState;

fn parse_and_emit_async(source: &str) -> String {
    parse_and_emit_async_with_capture(source, false)
}

fn parse_and_emit_async_with_capture(source: &str, capture_this: bool) -> String {
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        let has_await = emitter.body_contains_await(func.body);
                        let mut emitter = AsyncES5Emitter::new(&parser.arena);
                        emitter.set_use_this_capture(capture_this);
                        if has_await {
                            return emitter.emit_generator_body_with_await(func.body);
                        } else {
                            return emitter.emit_simple_generator_body(func.body);
                        }
                    }
                }
            }
        }
    }
    String::new()
}

#[test]
fn test_simple_async_empty() {
    let output = parse_and_emit_async("async function foo() { }");
    assert!(output.contains("return __generator"), "Should have generator wrapper");
    assert!(output.contains("[2 /*return*/]"), "Should have return instruction");
    assert!(!output.contains("switch"), "Empty body should not have switch");
}

#[test]
fn test_simple_async_with_return() {
    let output = parse_and_emit_async("async function foo() { return 42; }");
    assert!(output.contains("[2 /*return*/, 42]"), "Should return 42");
}

#[test]
fn test_async_with_await() {
    let output = parse_and_emit_async("async function foo() { await bar(); }");
    assert!(output.contains("switch (_a.label)"), "Should have switch statement");
    assert!(output.contains("[4 /*yield*/"), "Should have yield instruction");
    assert!(output.contains("_a.sent()"), "Should call _a.sent()");
}

#[test]
fn test_async_simple_return_captures_this() {
    let output = parse_and_emit_async_with_capture("async function foo() { return this; }", true);
    assert!(
        output.contains("return [2 /*return*/, _this]"),
        "Should emit _this when capture is enabled: {}",
        output
    );
}

#[test]
fn test_async_var_await_initializer_emits_assignment() {
    let output = parse_and_emit_async("async function foo() { var x = await bar(); return x; }");
    assert!(
        output.contains("return [4 /*yield*/, bar()]"),
        "Should yield awaited initializer: {}",
        output
    );
    assert!(
        output.contains("x = _a.sent()"),
        "Should assign awaited initializer: {}",
        output
    );
    assert!(
        output.contains("return [2 /*return*/, x]"),
        "Should return variable after await: {}",
        output
    );
}

#[test]
fn test_body_contains_await_detection() {
    let mut parser =
        ThinParserState::new("test.ts".to_string(), "async function foo() { await x; }".to_string());
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(emitter.body_contains_await(func.body), "Should detect await");
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_ignores_nested_async() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { const inner = async () => { await bar(); }; return 1; }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(!emitter.body_contains_await(func.body), "Should ignore nested await");
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_conditional_property_access() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return cond ? (await bar()).baz : (await qux())[idx]; }"
            .to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(
                            emitter.body_contains_await(func.body),
                            "Should detect await in conditional property/element access"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_call_argument() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return compute(await value); }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(
                            emitter.body_contains_await(func.body),
                            "Should detect await in call argument"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_call_expression_callee() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return (await getFactory())(); }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(
                            emitter.body_contains_await(func.body),
                            "Should detect await in call expression callee"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_binary_expression() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return (await left) + right; }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(
                            emitter.body_contains_await(func.body),
                            "Should detect await in binary expression"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_unary_expression() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return -(await value); }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(
                            emitter.body_contains_await(func.body),
                            "Should detect await in unary expression"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_object_literal() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return { value: await bar() }; }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(
                            emitter.body_contains_await(func.body),
                            "Should detect await in object literal"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_array_literal() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return [await bar()]; }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(
                            emitter.body_contains_await(func.body),
                            "Should detect await in array literal"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_object_literal_computed_name() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return { [await bar()]: 1 }; }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(
                            emitter.body_contains_await(func.body),
                            "Should detect await in computed property name"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_object_literal_spread() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return { ...await bar() }; }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(
                            emitter.body_contains_await(func.body),
                            "Should detect await in object literal spread"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_array_literal_spread() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return [...await bar()]; }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(
                            emitter.body_contains_await(func.body),
                            "Should detect await in array literal spread"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_template_expression() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return `value ${await bar()}`; }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(
                            emitter.body_contains_await(func.body),
                            "Should detect await in template expression"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_tagged_template() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return tag`value ${await bar()}`; }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(
                            emitter.body_contains_await(func.body),
                            "Should detect await in tagged template"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_no_await_in_simple_function() {
    let mut parser =
        ThinParserState::new("test.ts".to_string(), "async function foo() { return 1; }".to_string());
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let emitter = AsyncES5Emitter::new(&parser.arena);
                        assert!(!emitter.body_contains_await(func.body), "Should not detect await");
                    }
                }
            }
        }
    }
}
