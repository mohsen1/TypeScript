//! Tests for emitter.rs

use crate::emitter::*;
use crate::parser_impl::ParserState;

fn parse_and_emit(source: &str) -> String {
    let mut parser = ParserState::new("test.ts".to_string(), source.to_string());
    let root_idx = parser.parse_source_file();

    let mut printer = Printer::new();
    if let Some(root) = parser.arena.get(root_idx) {
        printer.emit_node(root, &parser.arena);
    }
    printer.take_output()
}

#[test]
fn test_emit_variable_declaration() {
    let output = parse_and_emit("let x = 42;");
    assert!(output.contains("let"), "Should contain 'let': {}", output);
    assert!(output.contains("x"), "Should contain 'x': {}", output);
    assert!(output.contains("42"), "Should contain '42': {}", output);
}

#[test]
fn test_emit_function_declaration() {
    let output = parse_and_emit("function add(a, b) { return a + b; }");
    assert!(output.contains("function"), "Should contain 'function': {}", output);
    assert!(output.contains("add"), "Should contain 'add': {}", output);
    assert!(output.contains("return"), "Should contain 'return': {}", output);
}

#[test]
fn test_emit_if_statement() {
    let output = parse_and_emit("if (x > 0) { y = 1; }");
    assert!(output.contains("if"), "Should contain 'if': {}", output);
    assert!(output.contains("x"), "Should contain 'x': {}", output);
}

#[test]
fn test_emit_binary_expression() {
    let output = parse_and_emit("let result = 1 + 2 * 3;");
    assert!(output.contains("+"), "Should contain '+': {}", output);
    assert!(output.contains("*"), "Should contain '*': {}", output);
}

#[test]
fn test_emit_arrow_function() {
    let output = parse_and_emit("const fn = (x) => x * 2;");
    assert!(output.contains("=>"), "Should contain '=>': {}", output);
    assert!(output.contains("const"), "Should contain 'const': {}", output);
}

#[test]
fn test_emit_string_literal_escaping() {
    // Test that string literals with special characters are properly escaped

    // Test basic escaping (newline)
    let mut printer = Printer::new();
    printer.emit_escaped_string("hello\nworld", '"');
    assert_eq!(printer.take_output(), "hello\\nworld");

    // Test quote escaping
    let mut printer = Printer::new();
    printer.emit_escaped_string("say \"hello\"", '"');
    assert_eq!(printer.take_output(), "say \\\"hello\\\"");

    // Test single quote in double-quoted string (no escape needed for other quote)
    let mut printer = Printer::new();
    printer.emit_escaped_string("it's fine", '"');
    assert_eq!(printer.take_output(), "it's fine");

    // Test backslash escaping
    let mut printer = Printer::new();
    printer.emit_escaped_string("path\\to\\file", '"');
    assert_eq!(printer.take_output(), "path\\\\to\\\\file");

    // Test tab and carriage return
    let mut printer = Printer::new();
    printer.emit_escaped_string("a\tb\rc", '"');
    assert_eq!(printer.take_output(), "a\\tb\\rc");
}

// =========================================================================
// Roundtrip tests: parse → emit → parse
// =========================================================================

fn roundtrip_test(source: &str) -> bool {
    // First pass: parse and emit
    let emitted = parse_and_emit(source);

    // Second pass: parse the emitted code
    let mut parser2 = ParserState::new("test2.ts".to_string(), emitted.clone());
    let root2 = parser2.parse_source_file();

    // Verify we got a valid parse
    if root2.is_none() {
        eprintln!("=== ROUNDTRIP FAILED (null parse) ===");
        eprintln!("Source: {}", source);
        eprintln!("Emitted: {}", emitted);
        return false;
    }

    // Third pass: emit again
    let mut printer2 = Printer::new();
    if let Some(root) = parser2.arena.get(root2) {
        printer2.emit_node(root, &parser2.arena);
    }
    let emitted2 = printer2.take_output();

    // The second emission should be identical to the first
    // (we've reached a fixed point)
    if emitted != emitted2 {
        eprintln!("=== ROUNDTRIP FAILED (mismatch) ===");
        eprintln!("Source: {}", source);
        eprintln!("First emit: [{}]", emitted);
        eprintln!("Second emit: [{}]", emitted2);
        return false;
    }
    true
}

#[test]
fn test_roundtrip_variable() {
    assert!(roundtrip_test("let x = 42;"), "Variable declaration should roundtrip");
}

#[test]
fn test_roundtrip_function() {
    assert!(roundtrip_test("function foo(a, b) { return a + b; }"), "Function should roundtrip");
}

#[test]
fn test_roundtrip_if_else() {
    assert!(roundtrip_test("if (x > 0) { y = 1; } else { y = 2; }"), "If-else should roundtrip");
}

#[test]
fn test_roundtrip_for_loop() {
    assert!(roundtrip_test("for (let i = 0; i < 10; i++) { sum = sum + i; }"), "For loop should roundtrip");
}

#[test]
fn test_roundtrip_while_loop() {
    assert!(roundtrip_test("while (x > 0) { x = x - 1; }"), "While loop should roundtrip");
}

#[test]
fn test_roundtrip_arrow_function() {
    assert!(roundtrip_test("const fn = (x) => x * 2;"), "Arrow function should roundtrip");
}

#[test]
fn test_roundtrip_class() {
    assert!(roundtrip_test("class Foo { }"), "Empty class should roundtrip");
}

#[test]
fn test_roundtrip_object_literal() {
    assert!(roundtrip_test("const obj = { a: 1, b: 2 };"), "Object literal should roundtrip");
}

#[test]
fn test_roundtrip_array_literal() {
    assert!(roundtrip_test("const arr = [1, 2, 3];"), "Array literal should roundtrip");
}

#[test]
fn test_roundtrip_call_expression() {
    assert!(roundtrip_test("foo(a, b, c);"), "Call expression should roundtrip");
}

#[test]
fn test_emit_try_catch() {
    let output = parse_and_emit("try { x(); } catch (e) { console.log(e); }");
    assert!(output.contains("try"), "Should contain 'try': {}", output);
    assert!(output.contains("catch"), "Should contain 'catch': {}", output);
}

#[test]
fn test_emit_try_finally() {
    let output = parse_and_emit("try { x(); } finally { cleanup(); }");
    assert!(output.contains("try"), "Should contain 'try': {}", output);
    assert!(output.contains("finally"), "Should contain 'finally': {}", output);
}

#[test]
fn test_emit_switch() {
    let output = parse_and_emit("switch (x) { case 1: break; default: y(); }");
    assert!(output.contains("switch"), "Should contain 'switch': {}", output);
    assert!(output.contains("case"), "Should contain 'case': {}", output);
    assert!(output.contains("default"), "Should contain 'default': {}", output);
}

// Note: Labeled statement test skipped - parser doesn't generate LabeledStatement nodes yet

#[test]
fn test_roundtrip_try_catch() {
    assert!(roundtrip_test("try { x(); } catch (e) { log(e); }"), "Try-catch should roundtrip");
}

#[test]
fn test_roundtrip_switch() {
    assert!(roundtrip_test("switch (x) { case 1: break; }"), "Switch should roundtrip");
}

#[test]
fn test_emit_for_in() {
    let output = parse_and_emit("for (let key in obj) { console.log(key); }");
    assert!(output.contains("for"), "Should contain 'for': {}", output);
    assert!(output.contains("in"), "Should contain 'in': {}", output);
    assert!(output.contains("key"), "Should contain 'key': {}", output);
}

#[test]
fn test_emit_for_of() {
    let output = parse_and_emit("for (let item of arr) { console.log(item); }");
    assert!(output.contains("for"), "Should contain 'for': {}", output);
    assert!(output.contains("of"), "Should contain 'of': {}", output);
    assert!(output.contains("item"), "Should contain 'item': {}", output);
}

#[test]
fn test_emit_spread() {
    let output = parse_and_emit("const arr = [1, ...other, 3];");
    assert!(output.contains("..."), "Should contain '...': {}", output);
    assert!(output.contains("other"), "Should contain 'other': {}", output);
}

#[test]
fn test_emit_spread_object() {
    let output = parse_and_emit("const obj = { a: 1, ...other };");
    assert!(output.contains("..."), "Should contain '...': {}", output);
    assert!(output.contains("other"), "Should contain 'other': {}", output);
}

// Note: yield/await tests disabled pending parser fixes for async/generator functions
// TODO: Fix infinite loop when parsing yield/await expressions
// #[test]
// fn test_emit_await() {
//     let output = parse_and_emit("async function f() { await fetch(url); }");
//     assert!(output.contains("await"), "Should contain 'await': {}", output);
//     assert!(output.contains("async"), "Should contain 'async': {}", output);
// }

// #[test]
// fn test_emit_yield() {
//     let output = parse_and_emit("function* gen() { yield 1; }");
//     assert!(output.contains("yield"), "Should contain 'yield': {}", output);
// }

#[test]
fn test_roundtrip_for_in() {
    assert!(roundtrip_test("for (let key in obj) { log(key); }"), "For-in should roundtrip");
}

#[test]
fn test_roundtrip_for_of() {
    assert!(roundtrip_test("for (let item of arr) { log(item); }"), "For-of should roundtrip");
}

#[test]
fn test_roundtrip_spread_array() {
    assert!(roundtrip_test("const arr = [1, ...other, 3];"), "Spread array should roundtrip");
}

#[test]
fn test_roundtrip_spread_object() {
    assert!(roundtrip_test("const obj = { a: 1, ...other };"), "Spread object should roundtrip");
}

// =========================================================================
// TypeScript-specific tests
// =========================================================================

// TODO: Parser doesn't create AsExpression nodes yet
// #[test]
// fn test_emit_as_expression() {
//     let output = parse_and_emit("const x = value as string;");
//     assert!(output.contains("as"), "Should contain 'as': {}", output);
//     assert!(output.contains("string"), "Should contain 'string': {}", output);
// }

// TODO: Parser doesn't create TypeAssertion nodes yet
// #[test]
// fn test_emit_type_assertion() {
//     let output = parse_and_emit("const x = <string>value;");
//     assert!(output.contains("<string>"), "Should contain '<string>': {}", output);
// }

#[test]
fn test_emit_non_null_assertion() {
    let output = parse_and_emit("const x = value!;");
    assert!(output.contains("!"), "Should contain '!': {}", output);
}

// TODO: Parser doesn't create InterfaceDeclaration nodes from statements
// #[test]
// fn test_emit_interface() {
//     let output = parse_and_emit("interface Foo { x: number; }");
//     assert!(output.contains("interface"), "Should contain 'interface': {}", output);
//     assert!(output.contains("Foo"), "Should contain 'Foo': {}", output);
//     assert!(output.contains("x"), "Should contain 'x': {}", output);
// }

#[test]
fn test_emit_type_alias() {
    let output = parse_and_emit("type Foo = string | number;");
    assert!(output.contains("type"), "Should contain 'type': {}", output);
    assert!(output.contains("Foo"), "Should contain 'Foo': {}", output);
}

// TODO: Parser doesn't create EnumDeclaration nodes from statements
// #[test]
// fn test_emit_enum() {
//     let output = parse_and_emit("enum Color { Red, Green, Blue }");
//     assert!(output.contains("enum"), "Should contain 'enum': {}", output);
//     assert!(output.contains("Color"), "Should contain 'Color': {}", output);
//     assert!(output.contains("Red"), "Should contain 'Red': {}", output);
// }

#[test]
fn test_emit_import() {
    let output = parse_and_emit("import { foo } from 'bar';");
    assert!(output.contains("import"), "Should contain 'import': {}", output);
    assert!(output.contains("from"), "Should contain 'from': {}", output);
}

#[test]
fn test_emit_export() {
    let output = parse_and_emit("export { foo, bar };");
    assert!(output.contains("export"), "Should contain 'export': {}", output);
}

// TODO: Parser doesn't handle export default correctly yet
// #[test]
// fn test_emit_export_default() {
//     let output = parse_and_emit("export default function() {}");
//     assert!(output.contains("export"), "Should contain 'export': {}", output);
//     assert!(output.contains("default"), "Should contain 'default': {}", output);
// }

// TODO: Parser doesn't create TemplateExpression nodes yet
// #[test]
// fn test_emit_template_literal() {
//     let output = parse_and_emit("const s = `hello ${name}!`;");
//     assert!(output.contains("`"), "Should contain backtick: {}", output);
//     assert!(output.contains("${"), "Should contain '${': {}", output);
// }

#[test]
fn test_emit_typeof() {
    let output = parse_and_emit("const t = typeof x;");
    assert!(output.contains("typeof"), "Should contain 'typeof': {}", output);
}

#[test]
fn test_emit_void() {
    let output = parse_and_emit("void 0;");
    assert!(output.contains("void"), "Should contain 'void': {}", output);
}

#[test]
fn test_emit_delete() {
    let output = parse_and_emit("delete obj.prop;");
    assert!(output.contains("delete"), "Should contain 'delete': {}", output);
}

// =========================================================================
// Type node emission tests
// =========================================================================

#[test]
fn test_emit_type_reference() {
    let output = parse_and_emit("type Foo = Array<string>;");
    assert!(output.contains("Array"), "Should contain 'Array': {}", output);
    assert!(output.contains("<"), "Should contain '<': {}", output);
    assert!(output.contains("string"), "Should contain 'string': {}", output);
}

#[test]
fn test_emit_union_type() {
    let output = parse_and_emit("type Foo = string | number;");
    assert!(output.contains("|"), "Should contain '|': {}", output);
    assert!(output.contains("string"), "Should contain 'string': {}", output);
    assert!(output.contains("number"), "Should contain 'number': {}", output);
}

#[test]
fn test_emit_intersection_type() {
    let output = parse_and_emit("type Foo = A & B;");
    assert!(output.contains("&"), "Should contain '&': {}", output);
}

#[test]
fn test_emit_array_type() {
    let output = parse_and_emit("type Foo = string[];");
    assert!(output.contains("[]"), "Should contain '[]': {}", output);
}

#[test]
fn test_emit_tuple_type() {
    let output = parse_and_emit("type Foo = [string, number];");
    assert!(output.contains("["), "Should contain '[': {}", output);
    assert!(output.contains("]"), "Should contain ']': {}", output);
    assert!(output.contains("string"), "Should contain 'string': {}", output);
    assert!(output.contains("number"), "Should contain 'number': {}", output);
}

#[test]
fn test_emit_function_type() {
    let output = parse_and_emit("type Foo = (x: number) => string;");
    assert!(output.contains("=>"), "Should contain '=>': {}", output);
    assert!(output.contains("number"), "Should contain 'number': {}", output);
    assert!(output.contains("string"), "Should contain 'string': {}", output);
}

#[test]
fn test_emit_type_literal() {
    let output = parse_and_emit("type Foo = { x: number; y: string };");
    assert!(output.contains("{"), "Should contain braces: {}", output);
    assert!(output.contains("x"), "Should contain 'x': {}", output);
}

#[test]
fn test_emit_indexed_access_type() {
    let output = parse_and_emit("type Foo = T[K];");
    assert!(output.contains("["), "Should contain '[': {}", output);
    assert!(output.contains("K"), "Should contain 'K': {}", output);
}

#[test]
fn test_emit_mapped_type() {
    let output = parse_and_emit("type Foo = { [K in T]: U };");
    assert!(output.contains("["), "Should contain '[': {}", output);
    assert!(output.contains("in"), "Should contain 'in': {}", output);
}

#[test]
fn test_emit_conditional_type() {
    let output = parse_and_emit("type Foo = T extends U ? X : Y;");
    assert!(output.contains("extends"), "Should contain 'extends': {}", output);
    assert!(output.contains("?"), "Should contain '?': {}", output);
    assert!(output.contains(":"), "Should contain ':': {}", output);
}

#[test]
fn test_emit_infer_type() {
    let output = parse_and_emit("type Foo = T extends (infer U) ? U : never;");
    assert!(output.contains("infer"), "Should contain 'infer': {}", output);
}

#[test]
fn test_emit_type_query() {
    let output = parse_and_emit("type Foo = typeof x;");
    assert!(output.contains("typeof"), "Should contain 'typeof': {}", output);
    assert!(output.contains("x"), "Should contain 'x': {}", output);
}

#[test]
fn test_emit_keyof_type() {
    let output = parse_and_emit("type Foo = keyof T;");
    assert!(output.contains("keyof"), "Should contain 'keyof': {}", output);
}

#[test]
fn test_emit_type_parameter_constraint() {
    let output = parse_and_emit("type Foo<T extends string> = T;");
    assert!(output.contains("extends"), "Should contain 'extends': {}", output);
    assert!(output.contains("string"), "Should contain 'string': {}", output);
}

#[test]
fn test_emit_decorator() {
    let output = parse_and_emit("@decorator class Foo {}");
    assert!(output.contains("@"), "Should contain '@': {}", output);
    assert!(output.contains("decorator"), "Should contain 'decorator': {}", output);
}

#[test]
fn test_emit_interface() {
    let output = parse_and_emit("interface Foo { x: number; }");
    assert!(output.contains("interface"), "Should contain 'interface': {}", output);
    assert!(output.contains("Foo"), "Should contain 'Foo': {}", output);
}

#[test]
fn test_emit_enum_with_values() {
    let output = parse_and_emit("enum Color { Red = 1, Green = 2, Blue = 3 }");
    assert!(output.contains("enum"), "Should contain 'enum': {}", output);
    assert!(output.contains("Red"), "Should contain 'Red': {}", output);
    assert!(output.contains("="), "Should contain '=': {}", output);
}

// NOTE: Destructuring tests cause infinite loop in parsing/emitting - needs investigation
// #[test]
// fn test_emit_destructuring() {
//     let output = parse_and_emit("const { a, b } = obj;");
//     assert!(output.contains("a"), "Should contain 'a': {}", output);
//     assert!(output.contains("b"), "Should contain 'b': {}", output);
// }

#[test]
fn test_emit_array_destructuring() {
    let output = parse_and_emit("const [a, b] = arr;");
    assert!(output.contains("["), "Should contain '[': {}", output);
    assert!(output.contains("a"), "Should contain 'a': {}", output);
}

#[test]
fn test_emit_namespace() {
    let output = parse_and_emit("namespace Foo { export const x = 1; }");
    assert!(output.contains("namespace"), "Should contain 'namespace': {}", output);
    assert!(output.contains("Foo"), "Should contain 'Foo': {}", output);
}

// =========================================================================
// Roundtrip tests for new type nodes
// =========================================================================

#[test]
fn test_roundtrip_type_alias() {
    assert!(roundtrip_test("type Foo = string;"), "Type alias should roundtrip");
}

#[test]
fn test_roundtrip_union_type() {
    assert!(roundtrip_test("type Foo = string | number;"), "Union type should roundtrip");
}

#[test]
fn test_roundtrip_interface() {
    assert!(roundtrip_test("interface Foo { x: number }"), "Interface should roundtrip");
}

#[test]
fn test_roundtrip_enum() {
    assert!(roundtrip_test("enum Color { Red, Green, Blue }"), "Enum should roundtrip");
}

// NOTE: Destructuring tests cause infinite loop - needs investigation
// #[test]
// fn test_roundtrip_destructuring() {
//     assert!(roundtrip_test("const { a, b } = obj;"), "Destructuring should roundtrip");
// }

// =========================================================================
// Source map tests
// =========================================================================

#[test]
fn test_source_map_basic() {
    let mut printer = Printer::with_source_map(
        PrinterOptions::default(),
        "output.js".to_string()
    );
    printer.add_source_file("input.ts".to_string());

    // Parse and emit some code
    let source = "const x = 1;";
    let mut parser = ParserState::new(
        "input.ts".to_string(),
        source.to_string()
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        printer.emit_node(root_node, &parser.arena);
    }

    // Get the source map
    let source_map = printer.get_source_map();
    assert!(source_map.is_some(), "Should generate source map");
    let json = source_map.unwrap();
    assert!(json.contains("\"version\": 3"), "Should be v3 source map");
    assert!(json.contains("\"file\": \"output.js\""), "Should have output file name");
    assert!(json.contains("\"sources\": [\"input.ts\"]"), "Should have input file name");
}

#[test]
fn test_source_map_with_content() {
    let mut printer = Printer::with_source_map(
        PrinterOptions::default(),
        "output.js".to_string()
    );
    let source = "const x = 1;";
    printer.add_source_file_with_content(
        "input.ts".to_string(),
        source.to_string()
    );

    let mut parser = ParserState::new(
        "input.ts".to_string(),
        source.to_string()
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        printer.emit_node(root_node, &parser.arena);
    }

    let source_map = printer.get_source_map();
    assert!(source_map.is_some());
    let json = source_map.unwrap();
    assert!(json.contains("\"sourcesContent\""), "Should have sources content");
    assert!(json.contains("const x = 1;"), "Should contain source text");
}

#[test]
fn test_inline_source_map() {
    let mut printer = Printer::with_source_map(
        PrinterOptions::default(),
        "output.js".to_string()
    );
    printer.add_source_file("input.ts".to_string());

    let source = "const x = 1;";
    let mut parser = ParserState::new(
        "input.ts".to_string(),
        source.to_string()
    );
    let root = parser.parse_source_file();

    if let Some(root_node) = parser.arena.get(root) {
        printer.emit_node(root_node, &parser.arena);
    }

    let inline_map = printer.get_inline_source_map();
    assert!(inline_map.is_some());
    let comment = inline_map.unwrap();
    assert!(comment.starts_with("//# sourceMappingURL=data:application/json;base64,"));
}

#[test]
fn test_position_tracking() {
    let mut printer = Printer::with_source_map(
        PrinterOptions::default(),
        "output.js".to_string()
    );
    printer.add_source_file("input.ts".to_string());

    // Write some text and check position tracking
    printer.write("hello");
    assert_eq!(printer.output_column, 5);
    assert_eq!(printer.output_line, 0);

    printer.write_line();
    assert_eq!(printer.output_column, 0);
    assert_eq!(printer.output_line, 1);

    printer.write("world");
    assert_eq!(printer.output_column, 5);
    assert_eq!(printer.output_line, 1);
}
