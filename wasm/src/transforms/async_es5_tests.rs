use super::*;
use crate::thin_parser::ThinParserState;

fn parse_and_emit_async(source: &str) -> String {
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
fn test_body_contains_await_in_for_loop_condition() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { for (let i = 0; await cond(i); i++) { work(); } }".to_string(),
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
                            "Should detect await in for-loop condition"
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
        "async function foo() { for (let i = await init(); i < 1; i++) { work(i); } }"
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
                            "Should detect await in for-loop initializer"
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
        "async function foo() { for (let i = 0; i < 1; i = await next(i)) { work(i); } }"
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
                            "Should detect await in for-loop incrementor"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_variable_initializer() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { let value = await getValue(), other = 1; }".to_string(),
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
                            "Should detect await in variable initializer"
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
        "async function foo() { const list = [1, await getValue(), 3]; }"
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
                            "Should detect await in array literal"
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
        "async function foo() { const list = [...await getValues()]; }"
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
                            "Should detect await in array literal spread"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_object_literal_property() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { const obj = { value: await getValue() }; }"
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
                            "Should detect await in object literal property"
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
        "async function foo() { const obj = { ...await getValue() }; }"
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
                            "Should detect await in object literal spread"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_object_binding_computed_property() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { const { [await key()]: value } = source; }"
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
                            "Should detect await in object binding computed name"
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
        "async function foo() { const obj = { [await key()]: 1 }; }"
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
                            "Should detect await in computed property name"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_object_binding_initializer() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { const { value = await getValue() } = data; }"
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
                            "Should detect await in object binding initializer"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_array_binding_initializer() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { const [value = await getValue()] = data; }"
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
                            "Should detect await in array binding initializer"
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
        "async function foo() { for (const item of await getItems()) { work(item); } }"
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
        "async function foo() { for (const key in await getMap()) { use(key); } }".to_string(),
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
fn test_body_contains_await_in_while_condition() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { while (await shouldContinue()) { work(); } }"
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
        "async function foo() { do { work(); } while (await shouldContinue()); }"
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
                            "Should detect await in do-while condition"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_try_finally() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { try { work(); } finally { await cleanup(); } }".to_string(),
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
                            "Should detect await in try/finally"
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
        "async function foo() { try { work(); } catch (e) { await handle(e); } }".to_string(),
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
fn test_body_contains_await_in_switch_discriminant() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { switch (await getKind()) { case 1: break; } }".to_string(),
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
                            "Should detect await in switch discriminant"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_switch_case_expression() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { switch (kind) { case await getCase(): break; } }".to_string(),
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
                            "Should detect await in switch case expression"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_switch_case() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { switch (kind) { case 1: await bar(); break; default: break; } }"
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
                            "Should detect await in switch case"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_switch_default() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { switch (kind) { case 1: break; default: await bar(); } }".to_string(),
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
                            "Should detect await in switch default"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_with_statement() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { with (obj) { await bar(); } }".to_string(),
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
                            "Should detect await in with statement"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_body_contains_await_in_labeled_statement() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { label: await bar(); }".to_string(),
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
                            "Should detect await in labeled statement"
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

#[test]
fn test_emit_lexical_this_capture_in_return() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { return this.value; }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let mut emitter = AsyncES5Emitter::new(&parser.arena);
                        emitter.set_use_this_capture(true);
                        let output = emitter.emit_simple_generator_body(func.body);
                        assert!(
                            output.contains("_this.value"),
                            "expected lexical this capture in output: {output}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_emit_lexical_this_capture_with_await() {
    let mut parser = ThinParserState::new(
        "test.ts".to_string(),
        "async function foo() { await this.bar(); }".to_string(),
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        if let Some(source_file) = parser.arena.get_source_file(root_node) {
            if let Some(&func_idx) = source_file.statements.nodes.first() {
                if let Some(func_node) = parser.arena.get(func_idx) {
                    if let Some(func) = parser.arena.get_function(func_node) {
                        let mut emitter = AsyncES5Emitter::new(&parser.arena);
                        emitter.set_use_this_capture(true);
                        let output = emitter.emit_generator_body_with_await(func.body);
                        assert!(
                            output.contains("_this.bar"),
                            "expected lexical this capture in output: {output}"
                        );
                    }
                }
            }
        }
    }
}
