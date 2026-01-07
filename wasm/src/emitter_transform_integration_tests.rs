//! Integration tests for Transform/Print separation architecture
//!
//! These tests demonstrate the complete two-phase emission pipeline:
//! 1. LoweringPass analyzes AST and produces TransformDirective entries
//! 2. ThinPrinter consults TransformContext and applies transforms during emission

use crate::emit_context::EmitContext;
use crate::lowering_pass::LoweringPass;
use crate::thin_emitter::ThinPrinter;
use crate::thin_parser::ThinParserState;
use crate::transform_context::{TransformContext, TransformDirective};

#[test]
fn test_two_phase_emission_es5_class() {
    // Parse source
    let source = "class Point { constructor(x, y) { this.x = x; this.y = y; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    // Phase 1: Lowering Pass (Transform)
    let mut ctx = EmitContext::default();
    ctx.target_es5 = true;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    // Verify transforms were generated
    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate ES5Class transform"
    );

    // Phase 2: Print Pass (with Transforms)
    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();

    // Verify ES5 IIFE pattern was emitted
    assert!(
        output.contains("var Point"),
        "ES5 output should contain 'var Point'"
    );
    assert!(
        output.contains("function ()"),
        "ES5 output should contain IIFE pattern"
    );
    assert!(
        output.contains("return Point"),
        "ES5 output should return constructor"
    );
}

#[test]
fn test_two_phase_emission_es6_class_no_transform() {
    // Parse source
    let source = "class Point { constructor(x, y) { this.x = x; this.y = y; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    // Phase 1: Lowering Pass (Transform) - ES6 target
    let ctx = EmitContext::default(); // ES6 by default

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    // Verify no transforms were generated for ES6
    assert!(
        transforms.is_empty(),
        "LoweringPass should NOT generate transforms for ES6 target"
    );

    // Phase 2: Print Pass (with empty Transforms)
    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(false); // ES6 target
    printer.emit(root);

    let output = printer.get_output();

    // Verify native class syntax was emitted
    assert!(
        output.contains("class Point"),
        "ES6 output should contain 'class Point'"
    );
    assert!(
        !output.contains("var Point"),
        "ES6 output should NOT contain 'var Point' (no IIFE)"
    );
}

#[test]
fn test_two_phase_emission_es5_class_try_throw_parenthesized() {
    let source = "class Foo { method() { try { throw new Error(\"x\"); } catch (e) { return (e); } } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.target_es5 = true;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("try {"),
        "ES5 output should contain try block: {}",
        output
    );
    assert!(
        output.contains("throw new Error(\"x\")"),
        "ES5 output should contain throw statement: {}",
        output
    );
    assert!(
        output.contains("catch (e)"),
        "ES5 output should contain catch clause: {}",
        output
    );
    assert!(
        output.contains("return (e);"),
        "ES5 output should preserve parenthesized return: {}",
        output
    );
}

#[test]
fn test_two_phase_emission_es5_class_for_destructuring() {
    let source = "class Foo { method(obj) { for (var { x, y } = obj; ; ) { return x + y; } } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.target_es5 = true;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("for (var _a = obj, x = _a.x, y = _a.y;"),
        "ES5 output should destructure for-loop initializer: {}",
        output
    );
}

#[test]
fn test_two_phase_emission_es5_class_object_rest_param() {
    let source = "class Foo { method({ x, ...rest }) { return rest; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.target_es5 = true;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("var __rest"),
        "ES5 output should include __rest helper: {}",
        output
    );
    assert!(
        output.contains("rest = __rest(_a, [\"x\"])"),
        "ES5 output should downlevel object rest parameter: {}",
        output
    );
}

#[test]
fn test_two_phase_emission_es5_class_for_in_of() {
    let source = "class Foo { method(obj, arr) { for (var k in obj) { k; } for (var v of arr) { v; } } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.target_es5 = true;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("for (var k in obj)"),
        "ES5 output should contain for-in loop: {}",
        output
    );
    assert!(
        output.contains("__values(arr)"),
        "ES5 output should downlevel for-of with __values helper: {}",
        output
    );
    assert!(
        output.contains("var v ="),
        "ES5 output should bind iterator values: {}",
        output
    );
    assert!(
        output.contains(".return"),
        "ES5 output should close iterators: {}",
        output
    );
}

#[test]
fn test_two_phase_emission_es5_for_of_statement() {
    let source = "for (var v of arr) { v; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.target_es5 = true;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("__values(arr)"),
        "ES5 output should downlevel for-of with __values helper: {}",
        output
    );
    assert!(
        output.contains(".return"),
        "ES5 output should close iterators: {}",
        output
    );
    assert!(
        !output.contains("for (var v of arr)"),
        "ES5 output should not contain raw for-of: {}",
        output
    );
}

#[test]
fn test_two_phase_emission_es5_class_switch_break_continue_do() {
    let source = "class Foo { method(x) { while (x) { continue; } switch (x) { case 1: break; default: return; } do { x--; } while (x); } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.target_es5 = true;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("while (x)"),
        "ES5 output should contain while loop: {}",
        output
    );
    assert!(
        output.contains("continue;"),
        "ES5 output should contain continue statement: {}",
        output
    );
    assert!(
        output.contains("switch (x)"),
        "ES5 output should contain switch statement: {}",
        output
    );
    assert!(
        output.contains("case 1:"),
        "ES5 output should contain case clause: {}",
        output
    );
    assert!(
        output.contains("break;"),
        "ES5 output should contain break statement: {}",
        output
    );
    assert!(
        output.contains("default:"),
        "ES5 output should contain default clause: {}",
        output
    );
    assert!(
        output.contains("return;"),
        "ES5 output should contain return statement: {}",
        output
    );
    assert!(
        output.contains("do {"),
        "ES5 output should contain do statement: {}",
        output
    );
    assert!(
        output.contains("} while (x);"),
        "ES5 output should contain do-while condition: {}",
        output
    );
}

#[test]
fn test_two_phase_backward_compatibility() {
    // Parse source
    let source = "class Foo {}";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    // Old way: printer without transforms (backward compatibility)
    let mut printer_old = ThinPrinter::new(&arena);
    printer_old.set_target_es5(true);
    printer_old.emit(root);
    let output_old = printer_old.get_output();

    // New way: printer with empty transforms
    let transforms = crate::transform_context::TransformContext::new();
    let mut printer_new = ThinPrinter::with_transforms(&arena, transforms);
    printer_new.set_target_es5(true);
    printer_new.emit(root);
    let output_new = printer_new.get_output();

    // Both should produce the same output (backward compatibility)
    assert_eq!(
        output_old, output_new,
        "Old and new emission paths should produce identical output"
    );
}

#[test]
fn test_two_phase_emission_es5_arrow_function() {
    let source = "const add = (a, b) => a + b;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.target_es5 = true;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate ES5ArrowFunction transform"
    );

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("function"),
        "ES5 arrow output should contain 'function'"
    );
    assert!(
        !output.contains("=>"),
        "ES5 arrow output should not contain '=>'"
    );
}

#[test]
fn test_two_phase_emission_es5_async_function() {
    let source = "async function foo() { return 1; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.target_es5 = true;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate ES5AsyncFunction transform"
    );

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("__awaiter"),
        "ES5 async output should contain '__awaiter'"
    );
    assert!(
        output.contains("__generator"),
        "ES5 async output should contain '__generator'"
    );
}

#[test]
fn test_two_phase_emission_amd_module_wrapper() {
    let source = "import { foo } from \"./bar\"; export const x = foo;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.options.module = crate::thin_emitter::ModuleKind::AMD;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate ModuleWrapper transform"
    );

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("define([\"require\", \"exports\", \"./bar\"]"),
        "AMD output should include define dependency list"
    );
    assert!(
        output.contains("function (require, exports"),
        "AMD output should include factory signature"
    );
}

#[test]
fn test_two_phase_emission_umd_module_wrapper() {
    let source = "export const x = 1;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.options.module = crate::thin_emitter::ModuleKind::UMD;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate ModuleWrapper transform"
    );

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("(function (factory) {"),
        "UMD output should include wrapper header"
    );
    assert!(
        output.contains("factory(require, exports)"),
        "UMD output should include CommonJS factory path"
    );
}

#[test]
fn test_two_phase_emission_system_module_wrapper() {
    let source = "import { foo } from \"./bar\"; export const x = foo;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.options.module = crate::thin_emitter::ModuleKind::System;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate ModuleWrapper transform"
    );

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("System.register([\"./bar\"]"),
        "System output should include System.register dependency list"
    );
    assert!(
        output.contains("execute: function ()"),
        "System output should include execute block"
    );
}

#[test]
fn test_two_phase_emission_commonjs_multi_export_vars() {
    let source = "export const a = 1, b = 2;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.options.module = crate::thin_emitter::ModuleKind::CommonJS;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate CommonJS export transforms"
    );

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("exports.a = a;"),
        "CommonJS output should export a"
    );
    assert!(
        output.contains("exports.b = b;"),
        "CommonJS output should export b"
    );
}

#[test]
fn test_two_phase_emission_commonjs_async_function_export() {
    let source = "export async function foo() { await bar(); }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::es5();
    ctx.options.module = crate::thin_emitter::ModuleKind::CommonJS;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate async/CommonJS transforms"
    );

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("__awaiter"),
        "ES5 output should contain __awaiter: {}",
        output
    );
    assert!(
        output.contains("exports.foo = foo;"),
        "CommonJS output should export foo: {}",
        output
    );
    assert_eq!(
        output.matches("exports.foo = foo;").count(),
        1,
        "Expected a single CommonJS export assignment: {}",
        output
    );
}

#[test]
fn test_lowering_pass_commonjs_default_anonymous_function_directive() {
    let source = "export default function () { return 1; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.options.module = crate::thin_emitter::ModuleKind::CommonJS;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    let root_node = arena.get(root).expect("expected source file node");
    let source_file = arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let stmt_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected export declaration");
    let stmt_node = arena.get(stmt_idx).expect("expected export node");
    let export_decl = arena
        .get_export_decl(stmt_node)
        .expect("expected export declaration data");

    let directive = transforms.get(export_decl.export_clause);
    assert!(
        matches!(directive, Some(TransformDirective::CommonJSExportDefaultExpr)),
        "LoweringPass should emit default export directive for anonymous function, got: {:?}",
        directive
    );
}

#[test]
fn test_lowering_pass_commonjs_default_anonymous_class_directive() {
    let source = "export default class { method() { return 1; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::default();
    ctx.options.module = crate::thin_emitter::ModuleKind::CommonJS;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    let root_node = arena.get(root).expect("expected source file node");
    let source_file = arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let stmt_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected export declaration");
    let stmt_node = arena.get(stmt_idx).expect("expected export node");
    let export_decl = arena
        .get_export_decl(stmt_node)
        .expect("expected export declaration data");

    let directive = transforms.get(export_decl.export_clause);
    assert!(
        matches!(directive, Some(TransformDirective::CommonJSExportDefaultExpr)),
        "LoweringPass should emit default export directive for anonymous class"
    );
}

#[test]
fn test_two_phase_emission_commonjs_default_anonymous_async_function_export() {
    let source = "export default async function () { await bar(); }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::es5();
    ctx.options.module = crate::thin_emitter::ModuleKind::CommonJS;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate ES5 async transforms"
    );

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(ctx.target_es5);
    printer.set_module_kind(crate::thin_emitter::ModuleKind::CommonJS);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("__awaiter"),
        "ES5 output should contain __awaiter: {}",
        output
    );
    assert!(
        output.contains("exports.default = function"),
        "CommonJS output should export default function: {}",
        output
    );
    assert!(
        !output.contains("export default"),
        "CommonJS output should not contain ES module syntax: {}",
        output
    );
}

#[test]
fn test_two_phase_emission_commonjs_default_anonymous_class_export() {
    let source = "export default class { method() { return 1; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::es5();
    ctx.options.module = crate::thin_emitter::ModuleKind::CommonJS;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate ES5 class transforms"
    );

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(ctx.target_es5);
    printer.set_module_kind(crate::thin_emitter::ModuleKind::CommonJS);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("var _a_default = /** @class */"),
        "CommonJS output should downlevel default class: {}",
        output
    );
    assert!(
        output.contains("exports.default = _a_default;"),
        "CommonJS output should export default class temp: {}",
        output
    );
    assert!(
        !output.contains("export default"),
        "CommonJS output should not contain ES module syntax: {}",
        output
    );
}

#[test]
fn test_two_phase_emission_commonjs_export_enum() {
    let source = "export enum E { A, B = 2 }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::es5();
    ctx.options.module = crate::thin_emitter::ModuleKind::CommonJS;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate enum/CommonJS transforms"
    );

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(ctx.target_es5);
    printer.set_module_kind(crate::thin_emitter::ModuleKind::CommonJS);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("var E"),
        "CommonJS output should declare enum variable: {}",
        output
    );
    assert!(
        output.contains("(function (E)"),
        "CommonJS output should emit enum IIFE: {}",
        output
    );
    assert!(
        output.contains("exports.E = E;"),
        "CommonJS output should export enum: {}",
        output
    );
}

#[test]
fn test_two_phase_emission_commonjs_export_const_enum_is_erased() {
    let source = "export const enum E { A = 0 }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::es5();
    ctx.options.module = crate::thin_emitter::ModuleKind::CommonJS;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(ctx.target_es5);
    printer.set_module_kind(crate::thin_emitter::ModuleKind::CommonJS);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        !output.contains("var E"),
        "CommonJS output should not emit const enum: {}",
        output
    );
    assert!(
        !output.contains("exports.E"),
        "CommonJS output should not export const enum: {}",
        output
    );
}

#[test]
fn test_two_phase_emission_commonjs_export_namespace() {
    let source = "export namespace N { export function foo() { return 1; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::es5();
    ctx.options.module = crate::thin_emitter::ModuleKind::CommonJS;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate namespace/CommonJS transforms"
    );

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(ctx.target_es5);
    printer.set_module_kind(crate::thin_emitter::ModuleKind::CommonJS);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("var N"),
        "CommonJS output should declare namespace variable: {}",
        output
    );
    assert!(
        output.contains("(function (N)"),
        "CommonJS output should emit namespace IIFE: {}",
        output
    );
    assert!(
        output.contains("N.foo = foo;"),
        "CommonJS output should export namespace member: {}",
        output
    );
    assert!(
        output.contains("exports.N = N;"),
        "CommonJS output should export namespace: {}",
        output
    );
}

#[test]
fn test_two_phase_emission_commonjs_auto_detect_exports() {
    let source = "export const x = 1;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::es5();
    ctx.auto_detect_module = true;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    assert!(
        !transforms.is_empty(),
        "LoweringPass should generate CommonJS export transforms via auto-detect"
    );

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(ctx.target_es5);
    printer.set_auto_detect_module(true);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("exports.x = x;"),
        "CommonJS auto-detect should export x"
    );
}

#[test]
fn test_two_phase_emission_commonjs_export_assignment_suppresses_named_exports() {
    let source = "export = foo;\nexport const x = 1;";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let mut ctx = EmitContext::es5();
    ctx.auto_detect_module = true;

    let lowering = LoweringPass::new(&arena, &ctx);
    let transforms = lowering.run(root);

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(ctx.target_es5);
    printer.set_auto_detect_module(true);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("module.exports = foo;"),
        "CommonJS export assignment should emit module.exports"
    );
    assert!(
        !output.contains("exports.x = x;"),
        "Named exports should be suppressed when export assignment is present"
    );
}

#[test]
fn test_transform_directive_chain_es5_class_commonjs_export() {
    let source = "class Foo {}";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = parser.arena;

    let root_node = arena.get(root).expect("expected source file node");
    let source_file = arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut transforms = TransformContext::new();
    transforms.insert(
        class_idx,
        TransformDirective::Chain(vec![
            TransformDirective::ES5Class {
                class_node: class_idx,
                class_name: Some("Foo".to_string()),
                heritage: None,
                members: Vec::new(),
            },
            TransformDirective::CommonJSExport {
                names: vec!["Foo".to_string()],
                is_default: false,
                inner: Box::new(TransformDirective::Identity),
            },
        ]),
    );

    let mut printer = ThinPrinter::with_transforms(&arena, transforms);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();
    assert!(
        output.contains("var Foo = /** @class */"),
        "Chained ES5 class transform should emit IIFE: {}",
        output
    );
    assert!(
        output.contains("exports.Foo = Foo;"),
        "Chained CommonJS export should emit assignment: {}",
        output
    );
    assert!(
        !output.contains("class Foo"),
        "Chained transforms should downlevel class syntax: {}",
        output
    );
}

#[test]
fn test_transform_directive_composability() {
    // This test verifies that the architecture supports composable transforms
    // For now, we just verify that the TransformContext can be created and passed around
    use crate::parser::NodeIndex;

    let mut ctx = TransformContext::new();

    // Add a transform directive
    ctx.insert(
        NodeIndex(1),
        TransformDirective::ES5Class {
            class_node: NodeIndex(1),
            class_name: Some("TestClass".to_string()),
            heritage: None,
            members: vec![],
        },
    );

    // Verify it was stored
    assert!(ctx.has_transform(NodeIndex(1)));
    assert!(!ctx.has_transform(NodeIndex(2)));
}
