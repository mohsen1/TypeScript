//! Tests for declaration_emitter.rs

use crate::declaration_emitter::*;
use crate::thin_parser::ThinParserState;

fn emit_declaration(source: &str) -> String {
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    
    let mut emitter = DeclarationEmitter::new(&parser.arena);
    emitter.emit(root)
}

#[test]
fn test_interface_declaration() {
    let output = emit_declaration("export interface Foo { x: number; y: string; }");
    eprintln!("Output: [{}]", output);
    assert!(output.contains("interface"), "Should contain interface: {}", output);
}

#[test]
fn test_type_alias_declaration() {
    let source = "export type Foo = string | number;";
    let output = emit_declaration(source);
    // Verify export type alias is emitted correctly
    assert!(output.contains("export type Foo"),
        "Should contain 'export type Foo': {}", output);
    assert!(output.contains("string"), "Should contain string: {}", output);
    assert!(output.contains("number"), "Should contain number: {}", output);
}

#[test]
fn test_enum_declaration() {
    let output = emit_declaration("export enum Color { Red, Green, Blue }");
    assert!(output.contains("enum Color"), "Should contain enum Color: {}", output);
    assert!(output.contains("Red"), "Should contain Red: {}", output);
}

#[test]
fn test_type_only_export() {
    // This should be emitted as: export type { Foo };
    let output = emit_declaration("export type { Foo } from './foo';");
    assert!(output.contains("export"), "Should contain export: {}", output);
}

#[test]
fn test_function_declaration() {
    let output = emit_declaration("export function add(a: number, b: number): number { return a + b; }");
    assert!(output.contains("function add"), "Should contain function add: {}", output);
    assert!(output.contains("number"), "Should contain number: {}", output);
}

#[test]
fn test_class_declaration() {
    let output = emit_declaration("export class MyClass { constructor(public name: string) {} }");
    assert!(output.contains("class MyClass"), "Should contain class MyClass: {}", output);
}

#[test]
fn test_class_heritage_clauses() {
    let source = "export class Child extends Base<T> implements IFace, Other {}";
    let output = emit_declaration(source);
    assert!(output.contains("class Child"), "Should contain class Child: {}", output);
    assert!(output.contains("extends Base<T>"), "Should contain extends clause: {}", output);
    assert!(output.contains("implements IFace, Other"), "Should contain implements clause: {}", output);
}

#[test]
fn test_interface_with_methods() {
    let output = emit_declaration("export interface Service { start(): void; stop(): Promise<void>; }");
    assert!(output.contains("interface Service"), "Should contain interface Service: {}", output);
    assert!(output.contains("void"), "Should contain void: {}", output);
}

#[test]
fn test_generic_interface() {
    let output = emit_declaration("export interface Container<T> { value: T; }");
    assert!(output.contains("interface Container"), "Should contain interface Container: {}", output);
    assert!(output.contains("<T>") || output.contains("< T >"), "Should contain generic param: {}", output);
}

#[test]
fn test_interface_heritage_clauses() {
    let source = "export interface Child extends Base<T>, ns.Other {}";
    let output = emit_declaration(source);
    assert!(output.contains("interface Child"), "Should contain interface Child: {}", output);
    assert!(
        output.contains("extends Base<T>, ns.Other"),
        "Should contain extends clause list: {}",
        output
    );
}

#[test]
fn test_namespace_export() {
    let output = emit_declaration("export * from './module';");
    assert!(output.contains("export"), "Should contain export: {}", output);
}

#[test]
fn test_named_exports() {
    let output = emit_declaration("export { foo, bar } from './module';");
    assert!(output.contains("export"), "Should contain export: {}", output);
}
