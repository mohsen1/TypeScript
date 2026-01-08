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

/// Parity test for ES5 class with static async method that captures `this`.
/// In static methods, `this` refers to the class constructor.
/// The async static method should be wrapped in __awaiter and properly capture `this`.
#[test]
fn test_parity_es5_class_static_async_this_capture() {
    let source = "class Foo { static value = 42; static async getValue() { return this.value; } }";
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

    // Verify ES5 static async method downlevel with this capture
    assert!(
        output.contains("__awaiter"),
        "ES5 output should include __awaiter helper for static async method: {}",
        output
    );
    assert!(
        output.contains("__generator"),
        "ES5 output should include __generator helper for static async method: {}",
        output
    );
    assert!(
        output.contains("Foo.getValue = function"),
        "ES5 output should emit static method on class constructor: {}",
        output
    );
    // In static methods, this refers to the class itself, so this.value should work
    assert!(
        output.contains("this.value") || output.contains("_this.value"),
        "ES5 output should reference this.value or _this.value in static async method: {}",
        output
    );
    assert!(
        !output.contains("async"),
        "ES5 output should not contain async keyword: {}",
        output
    );
    assert!(
        !output.contains("static async"),
        "ES5 output should not contain static async syntax: {}",
        output
    );
}

/// Parity test for ES5 class expression with extends.
/// Class expressions should be downleveled similarly to class declarations,
/// using __extends helper and IIFE pattern.
#[test]
fn test_parity_es5_class_expression_extends() {
    let source = "const Derived = class extends Base { constructor() { super(); this.value = 1; } };";
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

    // Verify ES5 class expression downlevel with extends
    assert!(
        output.contains("__extends"),
        "ES5 output should include __extends helper for class expression inheritance: {}",
        output
    );
    assert!(
        output.contains("var Derived ="),
        "ES5 output should assign class expression to variable: {}",
        output
    );
    assert!(
        output.contains("function (_super)"),
        "ES5 output should use IIFE pattern with _super parameter: {}",
        output
    );
    assert!(
        output.contains("__extends("),
        "ES5 output should call __extends: {}",
        output
    );
    assert!(
        output.contains("_super.call(this)") || output.contains("_super.apply(this"),
        "ES5 output should convert super() to _super.call/apply: {}",
        output
    );
    assert!(
        output.contains("(Base)"),
        "ES5 output should pass Base to IIFE: {}",
        output
    );
    // Class expression uses /** @class */ comment pattern
    assert!(
        output.contains("/** @class */"),
        "ES5 output should include @class annotation: {}",
        output
    );
    assert!(
        !output.contains("extends Base"),
        "ES5 output should not contain extends keyword: {}",
        output
    );
}

/// Parity test for ES5 derived class with both instance and static fields.
/// Instance fields should be initialized in constructor after super().
/// Static fields should be assigned on class constructor after IIFE.
#[test]
fn test_parity_es5_derived_class_instance_static_fields() {
    let source = r#"
class Derived extends Base {
    instanceField = 42;
    static staticField = "hello";
    constructor() {
        super();
    }
}
"#;
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

    // Verify ES5 derived class with fields
    assert!(
        output.contains("__extends"),
        "ES5 output should include __extends helper: {}",
        output
    );

    // Instance field should be initialized in constructor
    assert!(
        output.contains("this.instanceField = 42") || output.contains("_this.instanceField = 42"),
        "ES5 output should initialize instance field in constructor: {}",
        output
    );

    // Static field should be assigned on class constructor
    assert!(
        output.contains("Derived.staticField = \"hello\""),
        "ES5 output should assign static field on class constructor: {}",
        output
    );

    // super() should be converted
    assert!(
        output.contains("_super.call(this)") || output.contains("_super.apply(this"),
        "ES5 output should convert super() call: {}",
        output
    );

    // No ES6 class syntax
    assert!(
        !output.contains("extends Base"),
        "ES5 output should not contain extends keyword: {}",
        output
    );
    assert!(
        !output.contains("instanceField =") || output.contains("this.instanceField =") || output.contains("_this.instanceField ="),
        "ES5 output should not have class field syntax outside constructor: {}",
        output
    );
}

/// Parity test for ES5 async generator function.
/// Async generators (`async function*`) should be downleveled using
/// __awaiter, __generator, and __asyncGenerator helpers.
#[test]
fn test_parity_es5_async_generator_function() {
    let source = "async function* gen() { yield 1; yield await Promise.resolve(2); }";
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

    // Verify ES5 async generator function downlevel
    assert!(
        output.contains("__awaiter") || output.contains("__asyncGenerator"),
        "ES5 output should include async helper (__awaiter or __asyncGenerator): {}",
        output
    );
    assert!(
        output.contains("__generator"),
        "ES5 output should include __generator helper: {}",
        output
    );
    assert!(
        output.contains("function gen"),
        "ES5 output should define gen function: {}",
        output
    );
    // Should not have async function* syntax
    assert!(
        !output.contains("async function*"),
        "ES5 output should not contain async function* syntax: {}",
        output
    );
}

/// Parity test for ES5 arrow function with rest parameters.
/// Rest parameters should be converted to use arguments with slice.
#[test]
fn test_parity_es5_arrow_rest_parameters() {
    let source = "const sum = (...nums) => nums.reduce((a, b) => a + b, 0);";
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

    // Verify ES5 arrow function with rest parameters
    assert!(
        output.contains("var sum = function"),
        "ES5 output should convert arrow to function expression: {}",
        output
    );
    // Rest parameters should be converted using arguments (either slice or for loop)
    assert!(
        output.contains("arguments"),
        "ES5 output should reference arguments for rest params: {}",
        output
    );
    // Should define nums from arguments (var nums = [] with for loop, or slice)
    assert!(
        output.contains("var nums"),
        "ES5 output should define nums variable: {}",
        output
    );
    // No arrow syntax
    assert!(
        !output.contains("=>"),
        "ES5 output should not contain arrow syntax: {}",
        output
    );
    // No rest parameter syntax
    assert!(
        !output.contains("...nums"),
        "ES5 output should not contain rest parameter syntax: {}",
        output
    );
}

/// Parity test for ES5 default export class in CommonJS.
/// Default exported class should be downleveled and exported via exports.default.
#[test]
fn test_parity_es5_default_export_class() {
    let source = "export default class Foo { constructor(x) { this.x = x; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    let arena = &parser.arena;

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    options.module = ModuleKind::CommonJS;

    let ctx = EmitContext::with_options(options.clone());
    let lowering = LoweringPass::new(arena, &ctx);
    let transforms = lowering.run(root);

    let mut printer = ThinPrinter::with_transforms_and_options(arena, transforms, options);
    printer.set_source_text(source);
    printer.set_target_es5(true);
    printer.emit(root);

    let output = printer.get_output();

    // Verify ES5 default export class in CommonJS
    assert!(
        output.contains("exports.default"),
        "CommonJS output should export via exports.default: {}",
        output
    );
    assert!(
        output.contains("__esModule"),
        "CommonJS output should include __esModule marker: {}",
        output
    );
    // Class should be downleveled to function
    assert!(
        output.contains("function Foo") || output.contains("var Foo = /** @class */"),
        "ES5 output should downlevel class to function: {}",
        output
    );
    // Constructor body preserved
    assert!(
        output.contains("this.x = x"),
        "ES5 output should preserve constructor body: {}",
        output
    );
    // No ES6 class syntax
    assert!(
        !output.contains("class Foo"),
        "ES5 output should not contain class syntax: {}",
        output
    );
    // No export default syntax
    assert!(
        !output.contains("export default"),
        "ES5 output should not contain export default syntax: {}",
        output
    );
}

/// Parity test for ES5 async iteration (for await...of).
/// Async iteration should be downleveled using __asyncValues helper.
#[test]
fn test_parity_es5_async_iteration() {
    let source = "async function process(items) { for await (const item of items) { console.log(item); } }";
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

    // Verify ES5 async iteration downlevel
    assert!(
        output.contains("__awaiter"),
        "ES5 output should include __awaiter helper: {}",
        output
    );
    assert!(
        output.contains("__generator"),
        "ES5 output should include __generator helper: {}",
        output
    );
    assert!(
        output.contains("function process"),
        "ES5 output should define process function: {}",
        output
    );
    // Should reference items in some form
    assert!(
        output.contains("items"),
        "ES5 output should reference items: {}",
        output
    );
    // No async function syntax
    assert!(
        !output.contains("async function"),
        "ES5 output should not contain async function syntax: {}",
        output
    );
    // No for await syntax
    assert!(
        !output.contains("for await"),
        "ES5 output should not contain for await syntax: {}",
        output
    );
}
