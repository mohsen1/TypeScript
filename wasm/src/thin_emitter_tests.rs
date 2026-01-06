//! Tests for ThinEmitter

use crate::thin_parser::ThinParserState;
use crate::thin_emitter::ThinPrinter;
use crate::thin_binder::ThinBinderState;
use crate::thin_checker::ThinCheckerState;
use crate::solver::TypeInterner;

#[test]
fn test_thin_printer_creation() {
    use crate::parser::thin_node::ThinNodeArena;
    let arena = ThinNodeArena::new();
    let printer = ThinPrinter::new(&arena);
    assert!(printer.get_output().is_empty());
}

// Note: write() is private, so we can't test it directly.
// The write functionality is tested indirectly through emit tests.

#[test]
fn test_thin_emit_variable_declaration() {
    let source = "let x = 42";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    // Initialize scanner by parsing source file (which calls next_token)
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    // ThinPrinter defaults to ES5 target, which emits 'var' instead of 'let'
    assert!(output.contains("var"), "Expected 'var' in output: {}", output);
    assert!(output.contains("x"), "Expected 'x' in output: {}", output);
    assert!(output.contains("42"), "Expected '42' in output: {}", output);
}

#[test]
fn test_thin_emit_function_declaration() {
    let source = "function add(a, b) { return a + b; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    assert!(output.contains("function"), "Expected 'function' in output: {}", output);
    assert!(output.contains("add"), "Expected 'add' in output: {}", output);
    assert!(output.contains("return"), "Expected 'return' in output: {}", output);
}

#[test]
fn test_thin_emit_if_statement() {
    let source = "if (x > 0) { y = 1; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    assert!(output.contains("if"), "Expected 'if' in output: {}", output);
    assert!(output.contains(">"), "Expected '>' in output: {}", output);
}

#[test]
fn test_thin_emit_class_declaration() {
    let source = "class Foo { }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    assert!(output.contains("class"), "Expected 'class' in output: {}", output);
    assert!(output.contains("Foo"), "Expected 'Foo' in output: {}", output);
}

#[test]
fn test_thin_emit_arrow_function() {
    let source = "let f = (x) => x * 2";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    // ES5 emit: arrow functions become regular function expressions
    assert!(output.contains("function"), "Expected 'function' in ES5 output: {}", output);
    assert!(output.contains("return x * 2"), "Expected 'return x * 2' in ES5 output: {}", output);
}

#[test]
fn test_thin_emit_interface_declaration() {
    // Interface declarations are TypeScript-only, so JavaScript emit should be empty
    let source = "interface Point { x: number; y: number; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    // For JavaScript emit, interface should NOT be in output
    assert!(!output.contains("interface"), "JavaScript output should NOT contain 'interface': {}", output);
}

#[test]
fn test_thin_emit_enum_declaration() {
    let source = "enum Color { Red, Green, Blue }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    assert!(output.contains("enum"), "Expected 'enum' in output: {}", output);
    assert!(output.contains("Color"), "Expected 'Color' in output: {}", output);
    assert!(output.contains("Red"), "Expected 'Red' in output: {}", output);
}

/// Full ThinNode pipeline integration test:
/// ThinParser → ThinBinder → ThinChecker → ThinEmitter
#[test]
fn test_thin_pipeline_integration() {
    let source = r#"
        function add(a: number, b: number): number {
            return a + b;
        }
        let result = add(1, 2);
    "#;

    // Step 1: Parse
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let source_file = parser.parse_source_file();
    assert!(!source_file.is_none(), "Source file should be parsed");

    // Step 2: Bind
    let mut binder = ThinBinderState::new();
    binder.bind_source_file(&parser.arena, source_file);
    // Verify symbols were created
    let symbol_count = binder.symbols.len();
    assert!(symbol_count >= 2, "Expected at least 2 symbols (add, result), got {}", symbol_count);

    // Step 3: Check (type inference)
    let types = TypeInterner::new();
    let checker = ThinCheckerState::new(&parser.arena, &binder, &types, "test.ts".to_string());
    // Basic check - the checker exists and can be created
    let _ = &checker.ctx.types; // Access types arena to verify it exists

    // Step 4: Emit
    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(source_file);

    let output = printer.get_output();
    assert!(output.contains("function"), "Output should contain 'function': {}", output);
    assert!(output.contains("add"), "Output should contain 'add': {}", output);
    // JavaScript emit strips types, so "number" should NOT be in output
    assert!(!output.contains("number"), "JavaScript output should NOT contain 'number' (types are stripped): {}", output);
    assert!(output.contains("return"), "Output should contain 'return': {}", output);
    // ThinPrinter defaults to ES5 target, which emits 'var' instead of 'let'
    assert!(output.contains("var"), "Output should contain 'var': {}", output);
    assert!(output.contains("result"), "Output should contain 'result': {}", output);
}

#[test]
fn test_thin_emit_import() {
    let source = r#"import { foo, bar } from "module";"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    assert!(output.contains("import"), "Output should contain 'import': {}", output);
    assert!(output.contains("foo"), "Output should contain 'foo': {}", output);
    assert!(output.contains("from"), "Output should contain 'from': {}", output);
}

#[test]
fn test_thin_emit_export() {
    let source = "export function greet() { return 'hello'; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    assert!(output.contains("export"), "Output should contain 'export': {}", output);
    assert!(output.contains("function"), "Output should contain 'function': {}", output);
    assert!(output.contains("greet"), "Output should contain 'greet': {}", output);
}

#[test]
fn test_thin_emit_get_accessor() {
    let source = "class Foo { get value() { return this._value; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    assert!(output.contains("get"), "Output should contain 'get': {}", output);
    assert!(output.contains("value"), "Output should contain 'value': {}", output);
}

#[test]
fn test_thin_emit_set_accessor() {
    let source = "class Foo { set value(v) { this._value = v; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    assert!(output.contains("set"), "Output should contain 'set': {}", output);
    assert!(output.contains("value"), "Output should contain 'value': {}", output);
}

#[test]
fn test_thin_emit_decorator() {
    let source = "@Component class MyComponent {}";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    assert!(output.contains("@"), "Output should contain '@': {}", output);
    assert!(output.contains("Component"), "Output should contain 'Component': {}", output);
    assert!(output.contains("class"), "Output should contain 'class': {}", output);
}

#[test]
fn test_thin_emit_static_property() {
    let source = "class Foo { static count = 0; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    // ES5 emit: static properties become ClassName.propName = value;
    assert!(output.contains("Foo.count = 0"), "ES5 output should contain 'Foo.count = 0': {}", output);
}

#[test]
fn test_thin_emit_private_method() {
    // For JavaScript emit, 'private' modifier is stripped
    let source = "class Foo { private doSomething(): void {} }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    assert!(!output.contains("private"), "JavaScript output should NOT contain 'private': {}", output);
    assert!(output.contains("doSomething"), "Output should contain 'doSomething': {}", output);
}

#[test]
fn test_thin_emit_static_readonly() {
    // For JavaScript emit, 'readonly' modifier is stripped and class becomes IIFE
    let source = "class Foo { static readonly MAX = 100; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    // ES5 emit: static properties become ClassName.propName = value;
    assert!(output.contains("Foo.MAX = 100"), "ES5 output should contain 'Foo.MAX = 100': {}", output);
    assert!(!output.contains("readonly"), "JavaScript output should NOT contain 'readonly': {}", output);
}

#[test]
fn test_thin_emit_protected_constructor() {
    let source = "class Singleton { protected constructor() {} }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    // ES5 emit: classes become IIFEs, protected is stripped
    assert!(!output.contains("protected"), "ES5 output should NOT contain 'protected': {}", output);
    assert!(output.contains("function Singleton"), "ES5 output should contain constructor function: {}", output);
}

#[test]
fn test_thin_emit_static_get_accessor() {
    let source = "class Foo { static get instance(): Foo { return null; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    // ES5 emit: class becomes IIFE (static accessors may be handled differently)
    // For now, just verify the class wrapper is emitted
    assert!(output.contains("var Foo"), "ES5 output should contain 'var Foo': {}", output);
    assert!(output.contains("function Foo"), "ES5 output should contain 'function Foo': {}", output);
}

#[test]
fn test_thin_emit_call_signature() {
    // Interfaces are TypeScript-only, so JavaScript emit should be empty
    let source = "interface Callable { (): string; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    // For JavaScript emit, interface should NOT be in output
    assert!(!output.contains("interface"), "JavaScript output should NOT contain 'interface': {}", output);
}

#[test]
fn test_thin_emit_construct_signature() {
    // Interfaces are TypeScript-only, so JavaScript emit should be empty
    let source = "interface Factory { new (): MyClass; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    // For JavaScript emit, interface should NOT be in output
    assert!(!output.contains("interface"), "JavaScript output should NOT contain 'interface': {}", output);
}

#[test]
fn test_thin_emit_readonly_property_signature() {
    // Interfaces are TypeScript-only, so JavaScript emit should be empty
    let source = "interface Config { readonly name: string; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    // For JavaScript emit, interface should NOT be in output
    assert!(!output.contains("interface"), "JavaScript output should NOT contain 'interface': {}", output);
    assert!(!output.contains("readonly"), "JavaScript output should NOT contain 'readonly': {}", output);
}

#[test]
fn test_thin_emit_readonly_index_signature() {
    // Interfaces are TypeScript-only, so JavaScript emit should be empty
    let source = "interface ReadonlyMap { readonly [key: string]: number; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut printer = ThinPrinter::new(&parser.arena);
    printer.emit(root);

    let output = printer.get_output();
    // For JavaScript emit, interface should NOT be in output
    assert!(!output.contains("interface"), "JavaScript output should NOT contain 'interface': {}", output);
    assert!(!output.contains("readonly"), "JavaScript output should NOT contain 'readonly': {}", output);
}
