use crate::emit_context::EmitContext;
use crate::lowering_pass::LoweringPass;
use crate::thin_emitter::{ModuleKind, PrinterOptions, ScriptTarget, ThinPrinter};
use crate::thin_parser::ThinParserState;

fn assert_parity(source: &str, target: ScriptTarget, module: ModuleKind) {
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = &parser.arena;

    let mut options_legacy = PrinterOptions::default();
    options_legacy.target = target;
    options_legacy.module = module;

    let mut printer_legacy = ThinPrinter::with_options(arena, options_legacy);
    printer_legacy.set_source_text(source);
    if matches!(target, ScriptTarget::ES3 | ScriptTarget::ES5) {
        printer_legacy.set_target_es5(true);
    }
    printer_legacy.emit(root);
    let output_legacy = printer_legacy.take_output();

    let mut options_new = PrinterOptions::default();
    options_new.target = target;
    options_new.module = module;

    let ctx = EmitContext::with_options(options_new.clone());
    let lowering = LoweringPass::new(arena, &ctx);
    let transforms = lowering.run(root);

    if source.contains("class") && matches!(target, ScriptTarget::ES3 | ScriptTarget::ES5) {
        assert!(
            !transforms.is_empty(),
            "LoweringPass failed to generate transforms for ES5 class"
        );
    }

    let mut printer_new = ThinPrinter::with_transforms_and_options(arena, transforms, options_new);
    printer_new.set_source_text(source);
    if matches!(target, ScriptTarget::ES3 | ScriptTarget::ES5) {
        printer_new.set_target_es5(true);
    }
    printer_new.emit(root);
    let output_new = printer_new.take_output();

    let output_legacy_trimmed = output_legacy.trim_end_matches('\n');
    let output_new_trimmed = output_new.trim_end_matches('\n');

    assert_eq!(
        output_legacy_trimmed, output_new_trimmed,
        "\nParity mismatch for source:\n{}\n\nLegacy:\n{}\n\nNew:\n{}",
        source, output_legacy, output_new
    );
}

#[test]
fn test_parity_es5_class() {
    assert_parity(
        "class Point { constructor(x, y) { this.x = x; this.y = y; } }",
        ScriptTarget::ES5,
        ModuleKind::None,
    );
}

#[test]
fn test_parity_commonjs_export() {
    assert_parity(
        "export class Foo {}",
        ScriptTarget::ES5,
        ModuleKind::CommonJS,
    );
}

#[test]
fn test_parity_es5_arrow() {
    assert_parity(
        "const add = (a, b) => a + b;",
        ScriptTarget::ES5,
        ModuleKind::None,
    );
}

#[test]
fn test_parity_async_es5() {
    assert_parity(
        "async function foo() { await bar(); }",
        ScriptTarget::ES5,
        ModuleKind::None,
    );
}

/// Parity test for ES5 class with async method calling super.method().
/// This test compares our output against expected tsc output behavior.
/// The async method should:
/// 1. Be wrapped in __awaiter
/// 2. Use _super.prototype.method.call(this) for super calls
/// 3. Include __extends helper for class inheritance
#[test]
fn test_parity_es5_class_async_super_method() {
    let source = "class Derived extends Base { async foo() { return super.method(); } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = &parser.arena;

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    options.module = ModuleKind::None;

    let ctx = EmitContext::with_options(options.clone());
    let lowering = LoweringPass::new(arena, &ctx);
    let transforms = lowering.run(root);

    let mut printer = ThinPrinter::with_transforms_and_options(arena, transforms, options);
    printer.set_source_text(source);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();

    // Verify ES5 class downlevel produces correct output matching tsc behavior
    assert!(
        output.contains("__extends"),
        "ES5 output should include __extends helper for class inheritance: {}",
        output
    );
    assert!(
        output.contains("__awaiter"),
        "ES5 output should include __awaiter helper for async method: {}",
        output
    );
    assert!(
        output.contains("__generator"),
        "ES5 output should include __generator helper for async method: {}",
        output
    );
    assert!(
        output.contains("_super.prototype.method.call(this)"),
        "ES5 output should lower super.method() to _super.prototype.method.call(this): {}",
        output
    );
    assert!(
        output.contains("Derived.prototype.foo = function"),
        "ES5 output should emit method on prototype: {}",
        output
    );
    assert!(
        !output.contains("async"),
        "ES5 output should not contain async keyword: {}",
        output
    );
}

/// Parity test for ES5 class with getter and setter.
/// In ES5, class accessors should be downleveled to Object.defineProperty calls.
/// tsc emits: Object.defineProperty(Foo.prototype, "value", { get: function() {...}, set: function(v) {...}, ... });
#[test]
fn test_parity_es5_class_getter_setter() {
    let source = "class Foo { private _value: number = 0; get value() { return this._value; } set value(v) { this._value = v; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = &parser.arena;

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    options.module = ModuleKind::None;

    let ctx = EmitContext::with_options(options.clone());
    let lowering = LoweringPass::new(arena, &ctx);
    let transforms = lowering.run(root);

    let mut printer = ThinPrinter::with_transforms_and_options(arena, transforms, options);
    printer.set_source_text(source);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();

    // Verify ES5 class accessor downlevel produces correct output matching tsc behavior
    assert!(
        output.contains("Object.defineProperty"),
        "ES5 output should use Object.defineProperty for accessors: {}",
        output
    );
    assert!(
        output.contains("Foo.prototype"),
        "ES5 output should define accessors on prototype: {}",
        output
    );
    assert!(
        output.contains("\"value\""),
        "ES5 output should define property named 'value': {}",
        output
    );
    assert!(
        output.contains("get:") || output.contains("get :"),
        "ES5 output should have getter in property descriptor: {}",
        output
    );
    assert!(
        output.contains("set:") || output.contains("set :"),
        "ES5 output should have setter in property descriptor: {}",
        output
    );
    assert!(
        output.contains("this._value"),
        "ES5 output should reference this._value in accessor bodies: {}",
        output
    );
    assert!(
        !output.contains("get value()"),
        "ES5 output should not contain ES6 getter syntax: {}",
        output
    );
    assert!(
        !output.contains("set value("),
        "ES5 output should not contain ES6 setter syntax: {}",
        output
    );
}

/// Parity test for ES5 class with static getter.
/// Static accessors should be defined on the class constructor, not the prototype.
#[test]
fn test_parity_es5_class_static_getter() {
    let source = "class Foo { private static _instance: Foo; static get instance() { return Foo._instance; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = &parser.arena;

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    options.module = ModuleKind::None;

    let ctx = EmitContext::with_options(options.clone());
    let lowering = LoweringPass::new(arena, &ctx);
    let transforms = lowering.run(root);

    let mut printer = ThinPrinter::with_transforms_and_options(arena, transforms, options);
    printer.set_source_text(source);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();

    // Verify ES5 static accessor downlevel
    assert!(
        output.contains("Object.defineProperty"),
        "ES5 output should use Object.defineProperty for static accessor: {}",
        output
    );
    // Static accessors are defined on the constructor function itself, not prototype
    assert!(
        output.contains("Foo, \"instance\"") || output.contains("Foo,\"instance\""),
        "ES5 output should define static accessor on Foo (constructor): {}",
        output
    );
    assert!(
        output.contains("get:") || output.contains("get :"),
        "ES5 output should have getter in property descriptor: {}",
        output
    );
    assert!(
        output.contains("Foo._instance"),
        "ES5 output should reference Foo._instance in getter body: {}",
        output
    );
    assert!(
        !output.contains("static get instance"),
        "ES5 output should not contain ES6 static getter syntax: {}",
        output
    );
}
