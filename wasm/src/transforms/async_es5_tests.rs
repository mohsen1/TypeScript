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
