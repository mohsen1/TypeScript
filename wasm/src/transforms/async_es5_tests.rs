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
fn test_body_contains_await_in_conditional_expression() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return cond ? await bar() : baz; }".to_string(),
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
                            "Should detect await in conditional expression"
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
fn test_body_contains_await_in_new_expression() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return new Foo(await bar()); }".to_string(),
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
                            "Should detect await in new expression"
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
fn test_body_contains_await_in_parenthesized_expression() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return (await bar()); }".to_string(),
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
                            "Should detect await in parenthesized expression"
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
fn test_body_contains_await_in_as_expression() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return (await bar()) as number; }".to_string(),
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
                            "Should detect await in as expression"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_type_assertion() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return <any>await bar(); }".to_string(),
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
                            "Should detect await in type assertion"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_non_null_expression() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return (await bar())!; }".to_string(),
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
                            "Should detect await in non-null expression"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_switch_expression() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { switch (await bar()) { default: return 1; } }"
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
                            "Should detect await in switch expression"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_switch_case_statement() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { switch (value) { case 1: await bar(); } }"
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
                            "Should detect await in switch case statement"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_switch_default_clause() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { switch (value) { default: await bar(); } }".to_string(),
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
                            "Should detect await in switch default clause"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_for_loop_condition() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { for (; await bar(); ) { } }".to_string(),
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
                            "Should detect await in for loop condition"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_for_loop_incrementor() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { for (let i = 0; i < 1; i = await next()) { } }".to_string(),
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
                            "Should detect await in for loop incrementor"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_for_loop_initializer() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { for (i = await bar(); i < 1; i++) { } }".to_string(),
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
                            "Should detect await in for loop initializer"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_while_condition() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { while (await bar()) { } }".to_string(),
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
                            "Should detect await in while condition"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_do_while_condition() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { do { } while (await bar()); }".to_string(),
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
                            "Should detect await in do-while condition"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_for_of_expression() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { for (const x of await bar()) { } }".to_string(),
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
                            "Should detect await in for-of expression"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_for_in_expression() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { for (const x in await bar()) { } }".to_string(),
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
                            "Should detect await in for-in expression"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_try_statement() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { try { await bar(); } catch (e) { } }".to_string(),
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
                            "Should detect await in try block"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_catch_clause() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { try { } catch (e) { await bar(); } }".to_string(),
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
                            "Should detect await in catch clause"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_finally_block() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { try { } finally { await bar(); } }".to_string(),
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
                            "Should detect await in finally block"
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
