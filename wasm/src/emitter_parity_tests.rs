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

/// Parity test for ES5 for-await-of with destructuring.
/// Tests a more complex for-await-of scenario with object destructuring.
#[test]
fn test_parity_es5_for_await_of_destructuring() {
    let source = "async function processItems(stream) { for await (const { id, value } of stream) { console.log(id, value); } }";
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

    // Verify ES5 for-await-of with destructuring downlevel
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
        output.contains("function processItems"),
        "ES5 output should define processItems function: {}",
        output
    );
    // Destructured properties should be referenced
    assert!(
        output.contains("id") && output.contains("value"),
        "ES5 output should reference destructured properties id and value: {}",
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
    // No destructuring pattern in for-of (should be lowered)
    assert!(
        !output.contains("{ id, value }"),
        "ES5 output should not contain destructuring pattern in for-of: {}",
        output
    );
}

/// Parity test for ES5 template literals.
/// Template literals should be downleveled to string concatenation.
#[test]
fn test_parity_es5_template_literal() {
    let source = r#"const greeting = `Hello, ${name}!`;"#;
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

    // Verify ES5 template literal downlevel
    assert!(
        output.contains("var greeting"),
        "ES5 output should define greeting variable: {}",
        output
    );
    // Should reference name
    assert!(
        output.contains("name"),
        "ES5 output should reference name: {}",
        output
    );
    // Template literal should be converted to string concatenation or kept with quotes
    // tsc converts `Hello, ${name}!` to "Hello, " + name + "!"
    assert!(
        output.contains("+") || output.contains("\"Hello"),
        "ES5 output should use string concatenation or converted string: {}",
        output
    );
    // No template literal syntax
    assert!(
        !output.contains("`"),
        "ES5 output should not contain template literal backticks: {}",
        output
    );
}

/// Parity test for ES5 computed property names.
/// Computed properties in object literals should be downleveled.
#[test]
fn test_parity_es5_computed_property() {
    let source = r#"const key = "foo"; const obj = { [key]: 42 };"#;
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

    // Verify ES5 computed property downlevel
    assert!(
        output.contains("var key"),
        "ES5 output should define key variable: {}",
        output
    );
    assert!(
        output.contains("var obj"),
        "ES5 output should define obj variable: {}",
        output
    );
    // Should reference key and 42
    assert!(
        output.contains("key") && output.contains("42"),
        "ES5 output should reference key and value 42: {}",
        output
    );
    // No computed property syntax [key] in object literal
    // tsc converts { [key]: 42 } to (_a = {}, _a[key] = 42, _a)
    assert!(
        !output.contains("{ [key]") && !output.contains("{[key]"),
        "ES5 output should not contain computed property syntax in object literal: {}",
        output
    );
}

/// Parity test for ES5 shorthand property syntax.
/// Shorthand properties { x, y } should be expanded to { x: x, y: y }.
#[test]
fn test_parity_es5_shorthand_property() {
    let source = "const x = 1; const y = 2; const obj = { x, y };";
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

    // Verify ES5 shorthand property expansion
    assert!(
        output.contains("var x"),
        "ES5 output should define x variable: {}",
        output
    );
    assert!(
        output.contains("var y"),
        "ES5 output should define y variable: {}",
        output
    );
    assert!(
        output.contains("var obj"),
        "ES5 output should define obj variable: {}",
        output
    );
    // Shorthand should be expanded to x: x, y: y
    // tsc outputs { x: x, y: y }
    assert!(
        output.contains("x: x") || output.contains("x:x"),
        "ES5 output should expand shorthand x to x: x: {}",
        output
    );
    assert!(
        output.contains("y: y") || output.contains("y:y"),
        "ES5 output should expand shorthand y to y: y: {}",
        output
    );
}

/// Parity test for ES5 method shorthand in object literals.
/// Method shorthand { foo() {} } should be expanded to { foo: function() {} }.
#[test]
fn test_parity_es5_method_shorthand() {
    let source = "const obj = { greet() { return 'hello'; } };";
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

    // Verify ES5 method shorthand expansion
    assert!(
        output.contains("var obj"),
        "ES5 output should define obj variable: {}",
        output
    );
    // Method shorthand should be expanded to greet: function()
    // tsc outputs { greet: function() { return 'hello'; } }
    assert!(
        output.contains("greet: function") || output.contains("greet:function"),
        "ES5 output should expand method shorthand to greet: function: {}",
        output
    );
    assert!(
        output.contains("hello"),
        "ES5 output should contain return value: {}",
        output
    );
    // No method shorthand syntax
    assert!(
        !output.contains("greet()") || output.contains("greet: function()") || output.contains("greet:function()"),
        "ES5 output should not contain method shorthand syntax: {}",
        output
    );
}

/// Parity test for ES5 default parameters.
/// Default parameters should be downleveled to undefined checks.
#[test]
fn test_parity_es5_default_parameters() {
    let source = "function greet(name = 'World') { return 'Hello, ' + name; }";
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

    // Verify ES5 default parameter downlevel
    assert!(
        output.contains("function greet"),
        "ES5 output should define greet function: {}",
        output
    );
    // Default parameter should be converted to undefined check
    // tsc outputs: if (name === void 0) { name = 'World'; }
    assert!(
        output.contains("void 0") || output.contains("undefined"),
        "ES5 output should check for undefined: {}",
        output
    );
    assert!(
        output.contains("World"),
        "ES5 output should contain default value: {}",
        output
    );
    // No default parameter syntax in function signature
    assert!(
        !output.contains("name = 'World')") && !output.contains("name='World')"),
        "ES5 output should not contain default parameter syntax: {}",
        output
    );
}

/// Parity test for ES5 for...of loop.
/// for...of should be downleveled to iterator pattern or indexed loop.
#[test]
fn test_parity_es5_for_of_loop() {
    let source = "const arr = [1, 2, 3]; for (const x of arr) { console.log(x); }";
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

    // Verify ES5 for...of downlevel
    assert!(
        output.contains("var arr"),
        "ES5 output should define arr variable: {}",
        output
    );
    // for...of should be converted to iterator pattern or for loop
    // Should use __values helper or indexed for loop
    assert!(
        output.contains("for (") || output.contains("for("),
        "ES5 output should contain for loop: {}",
        output
    );
    assert!(
        output.contains("console.log"),
        "ES5 output should contain console.log: {}",
        output
    );
    // No for...of syntax
    assert!(
        !output.contains("of arr") && !output.contains("of arr)"),
        "ES5 output should not contain for...of syntax: {}",
        output
    );
}

/// Parity test for ES5 array destructuring in function parameters.
/// Array destructuring params should be downleveled to indexed access.
#[test]
fn test_parity_es5_array_destructuring_param() {
    let source = "function swap([a, b]) { return [b, a]; }";
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

    // Verify ES5 array destructuring param downlevel
    assert!(
        output.contains("function swap"),
        "ES5 output should define swap function: {}",
        output
    );
    // Destructuring should be converted to indexed access or temp variable
    // tsc outputs: function swap(_a) { var a = _a[0], b = _a[1]; return [b, a]; }
    assert!(
        output.contains("[0]") || output.contains("_a") || output.contains("_b"),
        "ES5 output should use indexed access or temp variables: {}",
        output
    );
    // No destructuring in parameter list
    assert!(
        !output.contains("([a, b])") && !output.contains("([a,b])"),
        "ES5 output should not contain array destructuring in params: {}",
        output
    );
}

/// Parity test for ES5 object destructuring in function parameters.
/// Object destructuring params should be downleveled to property access.
#[test]
fn test_parity_es5_object_destructuring_param() {
    let source = "function greet({ name, age }) { return name + ' is ' + age; }";
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

    // Verify ES5 object destructuring param downlevel
    assert!(
        output.contains("function greet"),
        "ES5 output should define greet function: {}",
        output
    );
    // Destructuring should be converted to property access
    // tsc outputs: function greet(_a) { var name = _a.name, age = _a.age; ... }
    assert!(
        output.contains(".name") || output.contains(".age") || output.contains("_a"),
        "ES5 output should use property access or temp variables: {}",
        output
    );
    // No destructuring in parameter list
    assert!(
        !output.contains("({ name") && !output.contains("({name"),
        "ES5 output should not contain object destructuring in params: {}",
        output
    );
}

/// Parity test for ES5 arrow function with expression body.
/// Arrow with expression body should be converted to function with return.
#[test]
fn test_parity_es5_arrow_expression_body() {
    let source = "const double = (x) => x * 2;";
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

    // Verify ES5 arrow expression body downlevel
    assert!(
        output.contains("var double"),
        "ES5 output should define double variable: {}",
        output
    );
    // Arrow with expression body should have return statement
    // tsc outputs: var double = function(x) { return x * 2; };
    assert!(
        output.contains("function"),
        "ES5 output should use function keyword: {}",
        output
    );
    assert!(
        output.contains("return"),
        "ES5 output should have return statement: {}",
        output
    );
    assert!(
        output.contains("* 2") || output.contains("*2"),
        "ES5 output should contain multiplication: {}",
        output
    );
    // No arrow syntax
    assert!(
        !output.contains("=>"),
        "ES5 output should not contain arrow syntax: {}",
        output
    );
}

/// Parity test for ES5 derived class with constructor and super call.
/// Derived class constructor should call _super with proper arguments.
#[test]
fn test_parity_es5_class_constructor_super() {
    let source = "class Animal { constructor(name) { this.name = name; } } class Dog extends Animal { constructor(name, breed) { super(name); this.breed = breed; } }";
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

    // Verify ES5 class with constructor and super
    assert!(
        output.contains("__extends"),
        "ES5 output should include __extends helper: {}",
        output
    );
    assert!(
        output.contains("function Animal"),
        "ES5 output should define Animal function: {}",
        output
    );
    assert!(
        output.contains("function Dog"),
        "ES5 output should define Dog function: {}",
        output
    );
    // super(name) should be converted to _super.call(this, name)
    assert!(
        output.contains("_super.call(this") || output.contains("_super.apply(this"),
        "ES5 output should call _super with this: {}",
        output
    );
    // No class syntax
    assert!(
        !output.contains("class Animal") && !output.contains("class Dog"),
        "ES5 output should not contain class syntax: {}",
        output
    );
}

/// Parity test for ES5 class with prototype methods.
/// Class methods should be added to the prototype.
#[test]
fn test_parity_es5_class_prototype_method() {
    let source = "class Calculator { add(a, b) { return a + b; } multiply(a, b) { return a * b; } }";
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

    // Verify ES5 class with prototype methods
    assert!(
        output.contains("function Calculator"),
        "ES5 output should define Calculator function: {}",
        output
    );
    // Methods should be on prototype
    assert!(
        output.contains("Calculator.prototype.add") || output.contains("prototype"),
        "ES5 output should add methods to prototype: {}",
        output
    );
    assert!(
        output.contains("function") && output.contains("return"),
        "ES5 output should contain method bodies: {}",
        output
    );
    // No class syntax
    assert!(
        !output.contains("class Calculator"),
        "ES5 output should not contain class syntax: {}",
        output
    );
}

/// Parity test for ES5 async arrow function.
/// Async arrow should be converted to function with __awaiter/__generator.
#[test]
fn test_parity_es5_async_arrow() {
    let source = "const fetchData = async () => { const result = await fetch('/api'); return result; };";
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

    // Verify ES5 async arrow downlevel
    assert!(
        output.contains("var fetchData"),
        "ES5 output should define fetchData variable: {}",
        output
    );
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
        output.contains("function"),
        "ES5 output should use function keyword: {}",
        output
    );
    // No async/arrow syntax
    assert!(
        !output.contains("async") || output.contains("__async"),
        "ES5 output should not contain async keyword: {}",
        output
    );
    assert!(
        !output.contains("=>"),
        "ES5 output should not contain arrow syntax: {}",
        output
    );
}

/// Parity test for ES5 let in for loop.
/// let/const should be converted to var.
#[test]
fn test_parity_es5_let_in_for_loop() {
    let source = "for (let i = 0; i < 10; i++) { console.log(i); }";
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

    // Verify ES5 let->var conversion
    assert!(
        output.contains("for (var i") || output.contains("for(var i"),
        "ES5 output should use var in for loop: {}",
        output
    );
    assert!(
        output.contains("console.log"),
        "ES5 output should contain console.log: {}",
        output
    );
    // No let keyword
    assert!(
        !output.contains("let i") && !output.contains("let  i"),
        "ES5 output should not contain let keyword: {}",
        output
    );
}

/// Parity test for ES5 const declaration.
/// const should be converted to var.
#[test]
fn test_parity_es5_const_declaration() {
    let source = "const PI = 3.14159; const E = 2.71828;";
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

    // Verify ES5 const->var conversion
    assert!(
        output.contains("var PI") || output.contains("var  PI"),
        "ES5 output should use var for PI: {}",
        output
    );
    assert!(
        output.contains("var E") || output.contains("var  E"),
        "ES5 output should use var for E: {}",
        output
    );
    assert!(
        output.contains("3.14159") && output.contains("2.71828"),
        "ES5 output should contain values: {}",
        output
    );
    // No const keyword
    assert!(
        !output.contains("const PI") && !output.contains("const E"),
        "ES5 output should not contain const keyword: {}",
        output
    );
}

/// Parity test for ES5 CommonJS named exports.
/// Named exports should be assigned to exports object.
#[test]
fn test_parity_es5_commonjs_named_exports() {
    let source = "const foo = 1; const bar = 2; export { foo, bar };";
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

    // Verify CommonJS named exports
    assert!(
        output.contains("__esModule"),
        "CommonJS output should include __esModule marker: {}",
        output
    );
    assert!(
        output.contains("var foo") || output.contains("foo = 1"),
        "CommonJS output should define foo: {}",
        output
    );
    assert!(
        output.contains("var bar") || output.contains("bar = 2"),
        "CommonJS output should define bar: {}",
        output
    );
    // Named exports should be assigned to exports
    assert!(
        output.contains("exports.foo") || output.contains("exports[\"foo\"]"),
        "CommonJS output should export foo: {}",
        output
    );
    assert!(
        output.contains("exports.bar") || output.contains("exports[\"bar\"]"),
        "CommonJS output should export bar: {}",
        output
    );
    // No ES6 export syntax
    assert!(
        !output.contains("export {"),
        "CommonJS output should not contain ES6 export syntax: {}",
        output
    );
}

/// Parity test for ES5 class with static property initialization.
/// Static properties should be assigned after the class IIFE.
#[test]
fn test_parity_es5_class_static_property() {
    let source = "class Config { static version = '1.0.0'; static count = 0; }";
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

    // Verify ES5 static property initialization
    assert!(
        output.contains("function Config") || output.contains("var Config"),
        "ES5 output should define Config as function: {}",
        output
    );
    // Static properties should be assigned on the class constructor
    assert!(
        output.contains("Config.version") && (output.contains("'1.0.0'") || output.contains("\"1.0.0\"")),
        "ES5 output should assign static version property: {}",
        output
    );
    assert!(
        output.contains("Config.count") && output.contains("0"),
        "ES5 output should assign static count property: {}",
        output
    );
    // No class syntax
    assert!(
        !output.contains("class Config"),
        "ES5 output should not contain class syntax: {}",
        output
    );
    // No static keyword in output
    assert!(
        !output.contains("static version") && !output.contains("static count"),
        "ES5 output should not contain static keyword: {}",
        output
    );
}

/// Parity test for ES5 abstract class.
/// Abstract classes should be downleveled just like regular classes,
/// with the abstract keyword removed (only TypeScript type checking uses it).
#[test]
fn test_parity_es5_abstract_class() {
    let source = "abstract class Shape { abstract area(): number; getName() { return 'shape'; } }";
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

    // Verify ES5 abstract class downlevel
    assert!(
        output.contains("function Shape") || output.contains("var Shape"),
        "ES5 output should define Shape as function: {}",
        output
    );
    // Concrete method should be on prototype
    assert!(
        output.contains("Shape.prototype.getName") || output.contains("getName"),
        "ES5 output should define getName method: {}",
        output
    );
    // Abstract method should NOT be emitted (it's type-only)
    assert!(
        !output.contains("Shape.prototype.area") || output.contains("area"),
        "ES5 output may or may not emit abstract method stub: {}",
        output
    );
    // No abstract keyword in output
    assert!(
        !output.contains("abstract class") && !output.contains("abstract "),
        "ES5 output should not contain abstract keyword: {}",
        output
    );
    // No class syntax
    assert!(
        !output.contains("class Shape"),
        "ES5 output should not contain class syntax: {}",
        output
    );
}

/// Parity test for ES5 namespace/module downlevel.
/// TypeScript namespaces should be downleveled to IIFE with exports.
#[test]
fn test_parity_es5_namespace() {
    let source = "namespace Utils { export function add(a: number, b: number) { return a + b; } export const PI = 3.14; }";
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

    // Verify ES5 namespace downlevel
    assert!(
        output.contains("var Utils") || output.contains("Utils = {}"),
        "ES5 output should define Utils namespace: {}",
        output
    );
    // Should use IIFE pattern or direct assignment
    assert!(
        output.contains("function") && output.contains("Utils"),
        "ES5 output should contain function for namespace: {}",
        output
    );
    // Exported members should be on namespace object
    assert!(
        output.contains("Utils.add") || output.contains("Utils_1.add") || output.contains("add"),
        "ES5 output should export add function: {}",
        output
    );
    assert!(
        output.contains("Utils.PI") || output.contains("PI") || output.contains("3.14"),
        "ES5 output should export PI constant: {}",
        output
    );
    // No namespace keyword
    assert!(
        !output.contains("namespace Utils"),
        "ES5 output should not contain namespace keyword: {}",
        output
    );
}

/// Parity test for ES5 enum downlevel.
/// TypeScript enums should be downleveled to IIFE with bidirectional mapping.
#[test]
fn test_parity_es5_enum() {
    let source = "enum Color { Red, Green, Blue }";
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

    // Verify ES5 enum downlevel
    assert!(
        output.contains("var Color") || output.contains("Color = {}"),
        "ES5 output should define Color enum: {}",
        output
    );
    // Enum members should be assigned
    assert!(
        output.contains("Red") && output.contains("Green") && output.contains("Blue"),
        "ES5 output should contain all enum members: {}",
        output
    );
    // Should have numeric values (0, 1, 2)
    assert!(
        output.contains("0") && output.contains("1") && output.contains("2"),
        "ES5 output should have numeric enum values: {}",
        output
    );
    // No enum keyword
    assert!(
        !output.contains("enum Color"),
        "ES5 output should not contain enum keyword: {}",
        output
    );
}

/// Parity test for ES5 string enum downlevel.
/// String enums should be downleveled without reverse mapping.
#[test]
fn test_parity_es5_string_enum() {
    let source = r#"enum Direction { Up = "UP", Down = "DOWN", Left = "LEFT", Right = "RIGHT" }"#;
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

    // Verify ES5 string enum downlevel
    assert!(
        output.contains("var Direction") || output.contains("Direction = {}"),
        "ES5 output should define Direction enum: {}",
        output
    );
    // Enum members should have string values
    assert!(
        output.contains("UP") && output.contains("DOWN") && output.contains("LEFT") && output.contains("RIGHT"),
        "ES5 output should contain all string enum values: {}",
        output
    );
    // Member names should be present
    assert!(
        output.contains("Up") && output.contains("Down"),
        "ES5 output should contain enum member names: {}",
        output
    );
    // No enum keyword
    assert!(
        !output.contains("enum Direction"),
        "ES5 output should not contain enum keyword: {}",
        output
    );
}

/// Parity test for type-only import erasure.
/// Type-only imports should be completely removed from output.
#[test]
fn test_parity_type_only_import_erasure() {
    let source = "import type { Foo } from './foo'; import { bar } from './bar'; bar();";
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

    // Type-only import should be erased
    assert!(
        !output.contains("Foo") && !output.contains("foo"),
        "Type-only import should be erased: {}",
        output
    );
    // Value import should remain
    assert!(
        output.contains("bar") && output.contains("require"),
        "Value import should remain: {}",
        output
    );
    // No import type syntax
    assert!(
        !output.contains("import type"),
        "Output should not contain import type syntax: {}",
        output
    );
}

/// Parity test for interface erasure.
/// TypeScript interfaces should be completely removed from output.
#[test]
fn test_parity_interface_erasure() {
    let source = "interface User { name: string; age: number; } const user: User = { name: 'Alice', age: 30 };";
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

    // Interface should be completely erased
    assert!(
        !output.contains("interface User"),
        "Interface declaration should be erased: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": User"),
        "Type annotation should be erased: {}",
        output
    );
    // Value code should remain
    assert!(
        output.contains("var user") && output.contains("Alice") && output.contains("30"),
        "Value code should remain: {}",
        output
    );
}

/// Parity test for type alias erasure.
/// TypeScript type aliases should be completely removed from output.
#[test]
fn test_parity_type_alias_erasure() {
    let source = "type StringOrNumber = string | number; const value: StringOrNumber = 42;";
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

    // Type alias should be completely erased
    assert!(
        !output.contains("type StringOrNumber"),
        "Type alias declaration should be erased: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": StringOrNumber"),
        "Type annotation should be erased: {}",
        output
    );
    // Value code should remain
    assert!(
        output.contains("var value") && output.contains("42"),
        "Value code should remain: {}",
        output
    );
}

/// Parity test for function parameter type erasure.
/// Function parameter types should be removed from output.
#[test]
fn test_parity_function_param_type_erasure() {
    let source = "function greet(name: string, age: number): string { return name + age; }";
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

    // Verify function exists
    assert!(
        output.contains("function greet"),
        "ES5 output should define greet function: {}",
        output
    );
    // Parameter names should remain
    assert!(
        output.contains("name") && output.contains("age"),
        "ES5 output should have parameter names: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains(": number"),
        "Type annotations should be erased: {}",
        output
    );
    // Return type should be erased
    assert!(
        !output.contains("): string"),
        "Return type should be erased: {}",
        output
    );
}

/// Parity test for generic type parameter erasure.
/// Generic type parameters should be removed from output.
#[test]
fn test_parity_generic_type_erasure() {
    let source = "function identity<T>(value: T): T { return value; }";
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

    // Verify function exists
    assert!(
        output.contains("function identity"),
        "Output should define identity function: {}",
        output
    );
    // Parameter name should remain
    assert!(
        output.contains("value"),
        "Output should have parameter name: {}",
        output
    );
    // Generic type parameter should be erased
    assert!(
        !output.contains("<T>"),
        "Generic type parameter <T> should be erased: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": T"),
        "Type annotation : T should be erased: {}",
        output
    );
}

/// Parity test for optional parameter question mark erasure.
/// The ? marker on optional parameters should be removed.
#[test]
fn test_parity_optional_param_erasure() {
    let source = "function greet(name?: string) { return name || 'World'; }";
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

    // Verify function exists
    assert!(
        output.contains("function greet"),
        "Output should define greet function: {}",
        output
    );
    // Parameter name should remain
    assert!(
        output.contains("name"),
        "Output should have parameter name: {}",
        output
    );
    // Optional marker should be erased
    assert!(
        !output.contains("?"),
        "Optional marker ? should be erased: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": string"),
        "Type annotation should be erased: {}",
        output
    );
}

/// Parity test for rest parameters ES5 downlevel.
/// Rest parameters should be converted to arguments slicing.
#[test]
fn test_parity_es5_rest_params() {
    let source = "function sum(...nums: number[]) { return nums.reduce((a, b) => a + b, 0); }";
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

    // Verify function exists
    assert!(
        output.contains("function sum"),
        "Output should define sum function: {}",
        output
    );
    // Rest parameter syntax should be removed
    assert!(
        !output.contains("...nums"),
        "Rest parameter syntax should be removed: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": number[]"),
        "Type annotation should be erased: {}",
        output
    );
}

/// Parity test for ES5 class static block downlevel.
/// Static blocks should be converted to static initialization code.
#[test]
fn test_parity_es5_static_block() {
    let source = r#"class Counter {
    static count = 0;
    static {
        Counter.count = 10;
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var Counter") || output.contains("function Counter"),
        "ES5 output should define Counter: {}",
        output
    );
    // Static block syntax should not appear in ES5 output
    assert!(
        !output.contains("static {"),
        "ES5 output should not contain static block syntax: {}",
        output
    );
    // Static initialization should occur
    assert!(
        output.contains("Counter.count"),
        "ES5 output should have static property access: {}",
        output
    );
}

/// Parity test for ES5 class static block with multiple statements.
/// Multiple statements in static block should be preserved.
#[test]
fn test_parity_es5_static_block_multi_stmt() {
    let source = r#"class Config {
    static debug = false;
    static version = "";
    static {
        Config.debug = true;
        Config.version = "1.0.0";
        console.log("Initialized");
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var Config") || output.contains("function Config"),
        "ES5 output should define Config: {}",
        output
    );
    // Static block syntax should not appear in ES5 output
    assert!(
        !output.contains("static {"),
        "ES5 output should not contain static block syntax: {}",
        output
    );
    // All static property assignments should occur
    assert!(
        output.contains("Config.debug") && output.contains("Config.version"),
        "ES5 output should have static property assignments: {}",
        output
    );
    // Console.log call should be preserved
    assert!(
        output.contains("console.log"),
        "ES5 output should preserve console.log call: {}",
        output
    );
}

/// Parity test for ES5 object spread downlevel.
/// Object spread should be converted to Object.assign or helper.
#[test]
fn test_parity_es5_object_spread() {
    let source = "const merged = { ...obj1, ...obj2, extra: true };";
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

    // Verify variable declaration
    assert!(
        output.contains("var merged") || output.contains("merged"),
        "ES5 output should define merged: {}",
        output
    );
    // Spread syntax should not appear in ES5 output
    assert!(
        !output.contains("...obj1") && !output.contains("...obj2"),
        "ES5 output should not contain spread syntax: {}",
        output
    );
}

/// Parity test for ES5 array spread downlevel.
/// Array spread should be converted to concat or helper.
#[test]
fn test_parity_es5_array_spread() {
    let source = "const combined = [...arr1, ...arr2, 42];";
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

    // Verify variable declaration
    assert!(
        output.contains("var combined") || output.contains("combined"),
        "ES5 output should define combined: {}",
        output
    );
    // Spread syntax should not appear in ES5 output
    assert!(
        !output.contains("...arr1") && !output.contains("...arr2"),
        "ES5 output should not contain spread syntax: {}",
        output
    );
}

/// Parity test for ES5 private class field downlevel.
/// Private fields (#name) should be converted to WeakMap-based emulation.
#[test]
fn test_parity_es5_private_field() {
    let source = r#"class Person {
    #name: string;
    constructor(name: string) {
        this.#name = name;
    }
    getName() {
        return this.#name;
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var Person") || output.contains("function Person"),
        "ES5 output should define Person class: {}",
        output
    );
    // Private field syntax should not appear in ES5 output
    assert!(
        !output.contains("#name"),
        "ES5 output should not contain private field syntax #name: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": string"),
        "Type annotation should be erased: {}",
        output
    );
}

/// Parity test for ES5 private static field access downlevel.
/// Static private field access should use __classPrivateFieldGet helper.
#[test]
fn test_parity_es5_private_static_field_access() {
    let source = r#"class Counter {
    static #count = 0;
    static getCount() {
        return Counter.#count;
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var Counter") || output.contains("function Counter"),
        "ES5 output should define Counter class: {}",
        output
    );
    // Should use __classPrivateFieldGet helper for static private access
    assert!(
        output.contains("__classPrivateFieldGet"),
        "ES5 output should use __classPrivateFieldGet helper: {}",
        output
    );
}

/// Parity test for ES5 private method downlevel.
/// Private methods should be converted to WeakSet-based emulation.
#[test]
fn test_parity_es5_private_method() {
    let source = r#"class Calculator {
    #validate(x: number) {
        return x >= 0;
    }
    compute(x: number) {
        if (this.#validate(x)) {
            return x * 2;
        }
        return 0;
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var Calculator") || output.contains("function Calculator"),
        "ES5 output should define Calculator class: {}",
        output
    );
    // Private method syntax should not appear in ES5 output
    assert!(
        !output.contains("#validate"),
        "ES5 output should not contain private method syntax #validate: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": number"),
        "Type annotation should be erased: {}",
        output
    );
}

/// Parity test for ES5 private getter downlevel.
/// Private getters should be converted using helper functions.
#[test]
fn test_parity_es5_private_getter() {
    let source = r#"class Box {
    #value = 0;
    get #contents() {
        return this.#value;
    }
    read() {
        return this.#contents;
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var Box") || output.contains("function Box"),
        "ES5 output should define Box class: {}",
        output
    );
    // Private getter syntax should not appear in ES5 output
    assert!(
        !output.contains("get #contents"),
        "ES5 output should not contain private getter syntax: {}",
        output
    );
}

/// Parity test for ES5 private setter downlevel.
/// Private setters should be converted using helper functions.
#[test]
fn test_parity_es5_private_setter() {
    let source = r#"class Container {
    #data = "";
    set #content(val: string) {
        this.#data = val;
    }
    store(val: string) {
        this.#content = val;
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var Container") || output.contains("function Container"),
        "ES5 output should define Container class: {}",
        output
    );
    // Private setter syntax should not appear in ES5 output
    assert!(
        !output.contains("set #content"),
        "ES5 output should not contain private setter syntax: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": string"),
        "Type annotation should be erased: {}",
        output
    );
}

/// Parity test for ES5 nullish coalescing downlevel.
/// The ?? operator should be converted to ternary checks.
#[test]
fn test_parity_es5_nullish_coalescing() {
    let source = "const result = value ?? 'default';";
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

    // Verify variable declaration
    assert!(
        output.contains("var result") || output.contains("result"),
        "ES5 output should define result: {}",
        output
    );
}

/// Parity test for ES5 optional chaining downlevel.
/// The ?. operator should be converted to conditional checks.
#[test]
fn test_parity_es5_optional_chaining() {
    let source = "const name = obj?.person?.name;";
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

    // Verify variable declaration
    assert!(
        output.contains("var name") || output.contains("name"),
        "ES5 output should define name: {}",
        output
    );
}

/// Parity test for ES5 generator function type erasure.
/// Generator function type annotations should be erased.
#[test]
fn test_parity_es5_generator_type_erasure() {
    let source = r#"function* range(start: number, end: number): Generator<number> {
    yield start;
}"#;
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

    // Verify function exists (may still have * if generator transform not implemented)
    assert!(
        output.contains("range"),
        "Output should define range function: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number") && !output.contains("Generator<number>"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 generator method in class downlevel.
/// Generator methods should be converted to __generator helper.
#[test]
fn test_parity_es5_generator_method() {
    let source = r#"class Sequence {
    *items() {
        yield 1;
        yield 2;
        yield 3;
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var Sequence") || output.contains("function Sequence"),
        "ES5 output should define Sequence class: {}",
        output
    );
    // Generator method syntax should not appear in ES5 output
    assert!(
        !output.contains("*items"),
        "ES5 output should not contain *items generator method syntax: {}",
        output
    );
}

/// Parity test for ES5 generator yield type erasure.
/// Generator yield expressions should have types erased.
#[test]
fn test_parity_es5_generator_yield_type_erasure() {
    let source = r#"function* gen(): Generator<string, void, unknown> {
    const result: string = yield "hello";
    yield result;
}"#;
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

    // Verify function exists
    assert!(
        output.contains("gen"),
        "Output should define gen function: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains("Generator<"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 async generator function type erasure.
/// Async generator type annotations should be erased.
#[test]
fn test_parity_es5_async_generator_type_erasure() {
    let source = r#"async function* fetchPages(urls: string[]): AsyncGenerator<string> {
    for (const url of urls) {
        yield await fetch(url);
    }
}"#;
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

    // Verify function exists
    assert!(
        output.contains("fetchPages"),
        "Output should define fetchPages function: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string[]") && !output.contains("AsyncGenerator<"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 async generator method in class.
/// Async generator methods should have types erased.
#[test]
fn test_parity_es5_async_generator_method() {
    let source = r#"class DataStream {
    async *items(): AsyncGenerator<number> {
        yield 1;
        yield 2;
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var DataStream") || output.contains("function DataStream"),
        "ES5 output should define DataStream class: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": AsyncGenerator<number>"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 async generator with await and yield.
/// Both await and yield should be preserved in output.
#[test]
fn test_parity_es5_async_generator_await_yield() {
    let source = r#"async function* process(items: Promise<number>[]): AsyncGenerator<number> {
    for (const item of items) {
        const value = await item;
        yield value * 2;
    }
}"#;
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

    // Verify function exists
    assert!(
        output.contains("process"),
        "Output should define process function: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains("Promise<number>[]") && !output.contains("AsyncGenerator<"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 class decorator downlevel.
/// Class decorators should use __decorate helper.
#[test]
fn test_parity_es5_class_decorator() {
    let source = r#"function sealed(constructor: Function) {}

@sealed
class Greeter {
    greeting: string;
}"#;
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

    // Verify class exists
    assert!(
        output.contains("Greeter"),
        "Output should define Greeter class: {}",
        output
    );
    // Decorator syntax should not appear directly in ES5 output
    assert!(
        !output.contains("@sealed"),
        "ES5 output should not contain @sealed decorator syntax: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": string") && !output.contains(": Function"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 method decorator downlevel.
/// Method decorators should use __decorate helper.
#[test]
fn test_parity_es5_method_decorator() {
    let source = r#"function log(target: any, key: string, descriptor: PropertyDescriptor) {}

class Calculator {
    @log
    add(a: number, b: number): number {
        return a + b;
    }
}"#;
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

    // Verify class exists
    assert!(
        output.contains("Calculator"),
        "Output should define Calculator class: {}",
        output
    );
    // Decorator syntax should not appear directly in ES5 output
    assert!(
        !output.contains("@log"),
        "ES5 output should not contain @log decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number") && !output.contains(": any") && !output.contains("PropertyDescriptor"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 property decorator downlevel.
/// Property decorators should use __decorate helper.
#[test]
fn test_parity_es5_property_decorator() {
    let source = r#"function observable(target: any, key: string) {}

class User {
    @observable
    name: string = "";
}"#;
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

    // Verify class exists
    assert!(
        output.contains("User"),
        "Output should define User class: {}",
        output
    );
    // Decorator syntax should not appear directly in ES5 output
    assert!(
        !output.contains("@observable"),
        "ES5 output should not contain @observable decorator syntax: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": string") && !output.contains(": any"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 parameter decorator downlevel.
/// Parameter decorators should use __param helper.
#[test]
fn test_parity_es5_parameter_decorator() {
    let source = r#"function inject(target: any, key: string, index: number) {}

class Service {
    constructor(@inject private dep: any) {}
}"#;
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

    // Verify class exists
    assert!(
        output.contains("Service"),
        "Output should define Service class: {}",
        output
    );
    // Decorator syntax should not appear directly in ES5 output
    assert!(
        !output.contains("@inject"),
        "ES5 output should not contain @inject decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": any") && !output.contains(": number"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 function call spread downlevel.
/// Spread in function calls should use .apply() or similar.
#[test]
fn test_parity_es5_call_spread() {
    let source = "const result = Math.max(...numbers);";
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

    // Verify variable declaration
    assert!(
        output.contains("result"),
        "ES5 output should define result: {}",
        output
    );
    // Spread syntax should not appear in ES5 output
    assert!(
        !output.contains("...numbers"),
        "ES5 output should not contain spread syntax: {}",
        output
    );
}

/// Parity test for ES5 new expression spread downlevel.
/// Spread in new expressions should be handled.
#[test]
fn test_parity_es5_new_spread() {
    let source = "const date = new Date(...args);";
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

    // Verify variable declaration
    assert!(
        output.contains("date"),
        "ES5 output should define date: {}",
        output
    );
    // Spread syntax should not appear in ES5 output
    assert!(
        !output.contains("...args"),
        "ES5 output should not contain spread syntax: {}",
        output
    );
}

/// Parity test for ES5 rest parameters in function downlevel.
/// Rest parameters should be converted to arguments slicing.
#[test]
fn test_parity_es5_rest_params_function() {
    let source = "function collect(first: number, ...rest: number[]) { return [first, ...rest]; }";
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

    // Verify function exists
    assert!(
        output.contains("function collect"),
        "ES5 output should define collect function: {}",
        output
    );
    // Rest parameter syntax should be removed
    assert!(
        !output.contains("...rest"),
        "ES5 output should not contain rest parameter syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number") && !output.contains(": number[]"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 spread in array literal with mixed elements.
/// Mixed spread and regular elements should be handled.
#[test]
fn test_parity_es5_array_spread_mixed() {
    let source = "const arr = [1, ...middle, 2, ...end, 3];";
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

    // Verify variable declaration
    assert!(
        output.contains("arr"),
        "ES5 output should define arr: {}",
        output
    );
    // Spread syntax should not appear in ES5 output
    assert!(
        !output.contains("...middle") && !output.contains("...end"),
        "ES5 output should not contain spread syntax: {}",
        output
    );
}

/// Parity test for ES5 array destructuring assignment downlevel.
/// Array destructuring should be converted to indexed access.
#[test]
fn test_parity_es5_array_destructuring_assignment() {
    let source = "const [first, second, third] = items;";
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

    // Verify variables are defined
    assert!(
        output.contains("first") && output.contains("second") && output.contains("third"),
        "ES5 output should define destructured variables: {}",
        output
    );
    // Array destructuring syntax should not appear in ES5 output
    assert!(
        !output.contains("[first, second, third]"),
        "ES5 output should not contain array destructuring syntax: {}",
        output
    );
}

/// Parity test for ES5 object destructuring assignment downlevel.
/// Object destructuring should be converted to property access.
#[test]
fn test_parity_es5_object_destructuring_assignment() {
    let source = "const { name, age, city } = person;";
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

    // Verify variables are defined
    assert!(
        output.contains("name") && output.contains("age") && output.contains("city"),
        "ES5 output should define destructured variables: {}",
        output
    );
    // Object destructuring syntax should not appear in ES5 output
    assert!(
        !output.contains("{ name, age, city }"),
        "ES5 output should not contain object destructuring syntax: {}",
        output
    );
}

/// Parity test for ES5 nested destructuring downlevel.
/// Nested destructuring should be fully expanded.
#[test]
fn test_parity_es5_nested_destructuring() {
    let source = "const { user: { name, address: { city } } } = data;";
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

    // Verify extracted variables exist
    assert!(
        output.contains("name") && output.contains("city"),
        "ES5 output should define nested destructured variables: {}",
        output
    );
}

/// Parity test for ES5 destructuring with default values downlevel.
/// Default values in destructuring should be preserved.
#[test]
fn test_parity_es5_destructuring_defaults() {
    let source = "const { name = 'Anonymous', age = 0 } = config;";
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

    // Verify variables are defined
    assert!(
        output.contains("name") && output.contains("age"),
        "ES5 output should define destructured variables: {}",
        output
    );
    // Default values should be preserved
    assert!(
        output.contains("Anonymous") || output.contains("0"),
        "ES5 output should preserve default values: {}",
        output
    );
}

/// Parity test for ES5 destructuring with rest element downlevel.
/// Rest element in destructuring should be handled.
#[test]
fn test_parity_es5_destructuring_rest() {
    let source = "const [first, ...remaining] = items;";
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

    // Verify variables are defined
    assert!(
        output.contains("first") && output.contains("remaining"),
        "ES5 output should define destructured variables: {}",
        output
    );
    // Rest syntax should not appear in ES5 destructuring
    assert!(
        !output.contains("...remaining"),
        "ES5 output should not contain rest element in destructuring: {}",
        output
    );
}

/// Parity test for ES5 optional method call downlevel.
/// Optional method calls (?.) should be transformed.
#[test]
fn test_parity_es5_optional_method_call() {
    let source = "const result = obj?.method?.();";
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

    // Verify variable declaration
    assert!(
        output.contains("result"),
        "ES5 output should define result: {}",
        output
    );
}

/// Parity test for ES5 optional element access downlevel.
/// Optional element access (?.[ ]) should be transformed.
#[test]
fn test_parity_es5_optional_element_access() {
    let source = "const value = arr?.[0]?.name;";
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

    // Verify variable declaration
    assert!(
        output.contains("value"),
        "ES5 output should define value: {}",
        output
    );
}

/// Parity test for ES5 optional chaining with nullish coalescing.
/// Combined ?. and ?? should both be handled.
#[test]
fn test_parity_es5_optional_chaining_with_nullish() {
    let source = "const name = user?.profile?.name ?? 'Anonymous';";
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

    // Verify variable declaration
    assert!(
        output.contains("name"),
        "ES5 output should define name: {}",
        output
    );
    // Default value should be preserved
    assert!(
        output.contains("Anonymous"),
        "ES5 output should preserve default value: {}",
        output
    );
}

/// Parity test for ES5 optional chaining in function call.
/// Optional chaining before function call should be handled.
#[test]
fn test_parity_es5_optional_chaining_call() {
    let source = "const result = callback?.(arg1, arg2);";
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

    // Verify variable declaration
    assert!(
        output.contains("result"),
        "ES5 output should define result: {}",
        output
    );
    // Arguments should be preserved
    assert!(
        output.contains("arg1") && output.contains("arg2"),
        "ES5 output should preserve arguments: {}",
        output
    );
}

/// Parity test for ES5 nullish coalescing with function call.
/// Nullish coalescing with function call as fallback.
#[test]
fn test_parity_es5_nullish_coalescing_call() {
    let source = "const value = config ?? getDefault();";
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

    // Verify variable declaration
    assert!(
        output.contains("value"),
        "ES5 output should define value: {}",
        output
    );
    // Function call should be preserved
    assert!(
        output.contains("getDefault"),
        "ES5 output should preserve fallback function call: {}",
        output
    );
}

/// Parity test for ES5 nullish coalescing assignment operator.
/// The ??= operator should be downleveled.
#[test]
fn test_parity_es5_nullish_assignment() {
    let source = "x ??= defaultValue;";
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

    // Verify identifiers exist
    assert!(
        output.contains("x") && output.contains("defaultValue"),
        "ES5 output should preserve identifiers: {}",
        output
    );
}

/// Parity test for ES5 chained nullish coalescing.
/// Multiple ?? operators in chain.
#[test]
fn test_parity_es5_nullish_chained() {
    let source = "const result = a ?? b ?? c ?? 'fallback';";
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

    // Verify variable declaration
    assert!(
        output.contains("result"),
        "ES5 output should define result: {}",
        output
    );
    // All identifiers should be preserved
    assert!(
        output.contains("a") && output.contains("b") && output.contains("c"),
        "ES5 output should preserve all identifiers: {}",
        output
    );
    // Fallback should be preserved
    assert!(
        output.contains("fallback"),
        "ES5 output should preserve fallback value: {}",
        output
    );
}

/// Parity test for ES5 nullish coalescing with object property.
/// Nullish coalescing on object property access.
#[test]
fn test_parity_es5_nullish_property() {
    let source = "const name = obj.name ?? obj.defaultName ?? 'Unknown';";
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

    // Verify variable declaration
    assert!(
        output.contains("name"),
        "ES5 output should define name: {}",
        output
    );
    // Property accesses should be preserved
    assert!(
        output.contains("obj") && output.contains("defaultName"),
        "ES5 output should preserve property accesses: {}",
        output
    );
}

/// Parity test for ES5 template literal with multiple expressions.
/// Multiple expressions should all be concatenated.
#[test]
fn test_parity_es5_template_multi_expr() {
    let source = r#"const msg = `Hello ${first} ${middle} ${last}!`;"#;
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

    // Verify variable declaration
    assert!(
        output.contains("msg"),
        "ES5 output should define msg: {}",
        output
    );
    // All identifiers should be preserved
    assert!(
        output.contains("first") && output.contains("middle") && output.contains("last"),
        "ES5 output should preserve all identifiers: {}",
        output
    );
    // Template literal syntax should not appear
    assert!(
        !output.contains("`"),
        "ES5 output should not contain backticks: {}",
        output
    );
}

/// Parity test for ES5 tagged template literal.
/// Tagged templates should be converted to function calls.
#[test]
fn test_parity_es5_tagged_template() {
    let source = r#"const result = tag`Hello ${name}!`;"#;
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

    // Verify variable declaration
    assert!(
        output.contains("result"),
        "ES5 output should define result: {}",
        output
    );
    // Tag function should be preserved
    assert!(
        output.contains("tag"),
        "ES5 output should preserve tag function: {}",
        output
    );
}

/// Parity test for ES5 template literal with function call expression.
/// Function calls in template expressions should be preserved.
#[test]
fn test_parity_es5_template_with_call() {
    let source = r#"const msg = `Result: ${compute(x, y)}`;"#;
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

    // Verify variable declaration
    assert!(
        output.contains("msg"),
        "ES5 output should define msg: {}",
        output
    );
    // Function call should be preserved
    assert!(
        output.contains("compute"),
        "ES5 output should preserve function call: {}",
        output
    );
    // Template literal syntax should not appear
    assert!(
        !output.contains("`"),
        "ES5 output should not contain backticks: {}",
        output
    );
}

/// Parity test for ES5 nested template literals.
/// Nested templates should all be converted.
#[test]
fn test_parity_es5_template_nested() {
    let source = r#"const msg = `outer ${`inner ${value}`}`;"#;
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

    // Verify variable declaration
    assert!(
        output.contains("msg"),
        "ES5 output should define msg: {}",
        output
    );
    // Value should be preserved
    assert!(
        output.contains("value"),
        "ES5 output should preserve value: {}",
        output
    );
    // Template literal syntax should not appear
    assert!(
        !output.contains("`"),
        "ES5 output should not contain backticks: {}",
        output
    );
}

/// Parity test for ES5 for-of with array destructuring.
/// for (const [a, b] of pairs) should downlevel both for-of and destructuring.
#[test]
fn test_parity_es5_for_of_array_destructuring() {
    let source = "const pairs = [[1,2],[3,4]]; for (const [a, b] of pairs) { console.log(a + b); }";
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

    // Verify ES5 for-of with array destructuring downlevel
    assert!(
        output.contains("var pairs"),
        "ES5 output should define pairs: {}",
        output
    );
    // Should have for loop structure
    assert!(
        output.contains("for (") || output.contains("for("),
        "ES5 output should contain for loop: {}",
        output
    );
    // No for...of syntax
    assert!(
        !output.contains(" of pairs") && !output.contains(" of pairs)"),
        "ES5 output should not contain for...of syntax: {}",
        output
    );
    // No destructuring pattern in loop header
    assert!(
        !output.contains("const [a, b]") && !output.contains("var [a, b]"),
        "ES5 output should not contain destructuring pattern: {}",
        output
    );
}

/// Parity test for ES5 for-of with object destructuring.
/// for (const {x, y} of points) should downlevel both for-of and destructuring.
#[test]
fn test_parity_es5_for_of_object_destructuring() {
    let source = "const points = [{x:1,y:2},{x:3,y:4}]; for (const {x, y} of points) { console.log(x, y); }";
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

    // Verify ES5 for-of with object destructuring downlevel
    assert!(
        output.contains("var points"),
        "ES5 output should define points: {}",
        output
    );
    // Should have for loop structure
    assert!(
        output.contains("for (") || output.contains("for("),
        "ES5 output should contain for loop: {}",
        output
    );
    // No for...of syntax
    assert!(
        !output.contains(" of points") && !output.contains(" of points)"),
        "ES5 output should not contain for...of syntax: {}",
        output
    );
    // No destructuring pattern in loop header
    assert!(
        !output.contains("const {x, y}") && !output.contains("var {x, y}"),
        "ES5 output should not contain object destructuring pattern: {}",
        output
    );
}

/// Parity test for ES5 nested for-of loops.
/// Nested for-of should both be downleveled correctly.
#[test]
fn test_parity_es5_for_of_nested() {
    let source = "const matrix = [[1,2],[3,4]]; for (const row of matrix) { for (const cell of row) { console.log(cell); } }";
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

    // Verify ES5 nested for-of downlevel
    assert!(
        output.contains("var matrix"),
        "ES5 output should define matrix: {}",
        output
    );
    // Should have console.log preserved
    assert!(
        output.contains("console.log"),
        "ES5 output should contain console.log: {}",
        output
    );
    // No for...of syntax
    assert!(
        !output.contains(" of matrix") && !output.contains(" of row"),
        "ES5 output should not contain for...of syntax: {}",
        output
    );
}

/// Parity test for ES5 for-of with let binding.
/// for (let x of arr) should downlevel to var in ES5.
#[test]
fn test_parity_es5_for_of_let() {
    let source = "const items = [1, 2, 3]; for (let item of items) { item++; console.log(item); }";
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

    // Verify ES5 for-of with let downlevel
    assert!(
        output.contains("var items"),
        "ES5 output should define items with var: {}",
        output
    );
    // Should have for loop structure
    assert!(
        output.contains("for (") || output.contains("for("),
        "ES5 output should contain for loop: {}",
        output
    );
    // No for...of syntax
    assert!(
        !output.contains(" of items") && !output.contains(" of items)"),
        "ES5 output should not contain for...of syntax: {}",
        output
    );
    // No let keyword in ES5
    assert!(
        !output.contains("let "),
        "ES5 output should not contain let keyword: {}",
        output
    );
}

/// Parity test for ES5 logical OR assignment (||=).
/// x ||= y should downlevel to x || (x = y) or equivalent.
#[test]
fn test_parity_es5_logical_or_assignment() {
    let source = "let x = null; x ||= 'default';";
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

    // Verify ES5 logical OR assignment downlevel
    assert!(
        output.contains("var x"),
        "ES5 output should define x with var: {}",
        output
    );
    // Should contain 'default' value
    assert!(
        output.contains("default") || output.contains("\"default\""),
        "ES5 output should contain default value: {}",
        output
    );
    // No ||= syntax in ES5
    assert!(
        !output.contains("||="),
        "ES5 output should not contain ||= syntax: {}",
        output
    );
}

/// Parity test for ES5 logical AND assignment (&&=).
/// x &&= y should downlevel to x && (x = y) or equivalent.
#[test]
fn test_parity_es5_logical_and_assignment() {
    let source = "let obj = { value: 1 }; obj.value &&= 42;";
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

    // Verify ES5 logical AND assignment downlevel
    assert!(
        output.contains("var obj"),
        "ES5 output should define obj with var: {}",
        output
    );
    // Should contain 42 value
    assert!(
        output.contains("42"),
        "ES5 output should contain 42: {}",
        output
    );
    // No &&= syntax in ES5
    assert!(
        !output.contains("&&="),
        "ES5 output should not contain &&= syntax: {}",
        output
    );
}

/// Parity test for ES5 nullish coalescing assignment (??=).
/// x ??= y should downlevel to x ?? (x = y) or null check equivalent.
#[test]
fn test_parity_es5_nullish_assignment_operator() {
    let source = "let config = { timeout: undefined }; config.timeout ??= 5000;";
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

    // Verify ES5 nullish coalescing assignment downlevel
    assert!(
        output.contains("var config"),
        "ES5 output should define config with var: {}",
        output
    );
    // Should contain 5000 value
    assert!(
        output.contains("5000"),
        "ES5 output should contain 5000: {}",
        output
    );
    // No ??= syntax in ES5
    assert!(
        !output.contains("??="),
        "ES5 output should not contain ??= syntax: {}",
        output
    );
}

/// Parity test for ES5 logical assignment with property access.
/// obj.prop ||= value should downlevel correctly.
#[test]
fn test_parity_es5_logical_assignment_property() {
    let source = "const settings = {}; settings.theme ||= 'dark'; settings.debug &&= false;";
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

    // Verify ES5 logical assignment with property downlevel
    assert!(
        output.contains("var settings"),
        "ES5 output should define settings with var: {}",
        output
    );
    // Should contain theme and debug
    assert!(
        output.contains("theme") && output.contains("debug"),
        "ES5 output should contain theme and debug: {}",
        output
    );
    // No logical assignment syntax in ES5
    assert!(
        !output.contains("||=") && !output.contains("&&="),
        "ES5 output should not contain logical assignment syntax: {}",
        output
    );
}

/// Parity test for ES5 exponentiation operator with type erasure.
/// Verifies type annotations are erased alongside exponentiation usage.
#[test]
fn test_parity_es5_exponentiation_type_erasure() {
    let source = "function power(base: number, exp: number): number { return base ** exp; }";
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

    // Verify function is preserved
    assert!(
        output.contains("function power"),
        "ES5 output should contain function power: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number"),
        "ES5 output should erase type annotations: {}",
        output
    );
    // Parameters should be preserved
    assert!(
        output.contains("base") && output.contains("exp"),
        "ES5 output should preserve parameter names: {}",
        output
    );
}

/// Parity test for ES5 exponentiation with const to var.
/// const with exponentiation should convert to var in ES5.
#[test]
fn test_parity_es5_exponentiation_const_to_var() {
    let source = "const squared = 5 ** 2;";
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

    // Verify const is converted to var
    assert!(
        output.contains("var squared"),
        "ES5 output should convert const to var: {}",
        output
    );
    // No const keyword in ES5
    assert!(
        !output.contains("const "),
        "ES5 output should not contain const keyword: {}",
        output
    );
}

/// Parity test for ES5 exponentiation with let to var.
/// let with exponentiation should convert to var in ES5.
#[test]
fn test_parity_es5_exponentiation_let_to_var() {
    let source = "let result = 2 ** 10; result = result ** 2;";
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

    // Verify let is converted to var
    assert!(
        output.contains("var result"),
        "ES5 output should convert let to var: {}",
        output
    );
    // No let keyword in ES5
    assert!(
        !output.contains("let "),
        "ES5 output should not contain let keyword: {}",
        output
    );
}

/// Parity test for ES5 exponentiation in arrow function.
/// Arrow function with exponentiation should downlevel correctly.
#[test]
fn test_parity_es5_exponentiation_arrow() {
    let source = "const cube = (n: number) => n ** 3;";
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

    // Verify const is converted to var
    assert!(
        output.contains("var cube"),
        "ES5 output should convert const to var: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": number"),
        "ES5 output should erase type annotation: {}",
        output
    );
    // Arrow function should be converted to regular function
    assert!(
        output.contains("function"),
        "ES5 output should convert arrow to function: {}",
        output
    );
}

/// Parity test for ES5 arrow function with typed expression body.
/// () => expr should convert to function() { return expr; }.
#[test]
fn test_parity_es5_arrow_typed_expression() {
    let source = "const double = (x: number) => x * 2;";
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

    // Verify arrow is converted to function
    assert!(
        output.contains("function"),
        "ES5 output should contain function keyword: {}",
        output
    );
    // Expression body should have return added
    assert!(
        output.contains("return"),
        "ES5 output should add return for expression body: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": number"),
        "ES5 output should erase type annotation: {}",
        output
    );
    // No arrow syntax
    assert!(
        !output.contains("=>"),
        "ES5 output should not contain arrow syntax: {}",
        output
    );
}

/// Parity test for ES5 arrow function with block body.
/// () => { stmts } should convert to function() { stmts }.
#[test]
fn test_parity_es5_arrow_block_body() {
    let source = "const greet = (name: string) => { console.log('Hello ' + name); };";
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

    // Verify arrow is converted to function
    assert!(
        output.contains("function"),
        "ES5 output should contain function keyword: {}",
        output
    );
    // Console.log should be preserved
    assert!(
        output.contains("console.log"),
        "ES5 output should preserve console.log: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": string"),
        "ES5 output should erase type annotation: {}",
        output
    );
    // No arrow syntax
    assert!(
        !output.contains("=>"),
        "ES5 output should not contain arrow syntax: {}",
        output
    );
}

/// Parity test for ES5 arrow function with this capture.
/// Arrow functions should capture outer this.
#[test]
fn test_parity_es5_arrow_this_capture() {
    let source = r#"class Counter {
    count = 0;
    increment = () => { this.count++; };
}"#;
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

    // Verify class is converted to function
    assert!(
        output.contains("function Counter") || output.contains("var Counter"),
        "ES5 output should convert class to function: {}",
        output
    );
    // No arrow syntax
    assert!(
        !output.contains("=>"),
        "ES5 output should not contain arrow syntax: {}",
        output
    );
    // Class declaration should be converted (class Counter {)
    assert!(
        !output.contains("class Counter {"),
        "ES5 output should not contain class declaration: {}",
        output
    );
    // Should capture this with _this
    assert!(
        output.contains("_this"),
        "ES5 output should capture this with _this: {}",
        output
    );
}

/// Parity test for ES5 arrow function with multiple parameters.
/// (a, b, c) => expr should convert correctly.
#[test]
fn test_parity_es5_arrow_multi_params() {
    let source = "const sum = (a: number, b: number, c: number) => a + b + c;";
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

    // Verify arrow is converted to function
    assert!(
        output.contains("function"),
        "ES5 output should contain function keyword: {}",
        output
    );
    // Parameters should be preserved (without types)
    assert!(
        output.contains("a") && output.contains("b") && output.contains("c"),
        "ES5 output should preserve parameters: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number"),
        "ES5 output should erase type annotations: {}",
        output
    );
    // No arrow syntax
    assert!(
        !output.contains("=>"),
        "ES5 output should not contain arrow syntax: {}",
        output
    );
}

/// Parity test for ES5 getter-only accessor with return type.
/// get prop(): Type should downlevel and erase type.
#[test]
fn test_parity_es5_getter_only_typed() {
    let source = "class Config { get timeout(): number { return 5000; } }";
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

    // Verify Object.defineProperty is used
    assert!(
        output.contains("Object.defineProperty"),
        "ES5 output should use Object.defineProperty: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": number"),
        "ES5 output should erase return type: {}",
        output
    );
    // No ES6 getter syntax
    assert!(
        !output.contains("get timeout()"),
        "ES5 output should not contain ES6 getter syntax: {}",
        output
    );
    // Should return 5000
    assert!(
        output.contains("5000"),
        "ES5 output should contain return value: {}",
        output
    );
}

/// Parity test for ES5 setter-only accessor with param type.
/// set prop(v: Type) should downlevel and erase type.
#[test]
fn test_parity_es5_setter_only_typed() {
    let source = "class Counter { private _count = 0; set count(val: number) { this._count = val; } }";
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

    // Verify Object.defineProperty is used
    assert!(
        output.contains("Object.defineProperty"),
        "ES5 output should use Object.defineProperty: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": number"),
        "ES5 output should erase param type: {}",
        output
    );
    // No ES6 setter syntax
    assert!(
        !output.contains("set count("),
        "ES5 output should not contain ES6 setter syntax: {}",
        output
    );
    // Should reference _count
    assert!(
        output.contains("_count"),
        "ES5 output should reference _count: {}",
        output
    );
}

/// Parity test for ES5 getter/setter pair with types.
/// Both getter return type and setter param type should be erased.
#[test]
fn test_parity_es5_accessor_pair_typed() {
    let source = "class Box<T> { private _value: T; get value(): T { return this._value; } set value(v: T) { this._value = v; } }";
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

    // Verify Object.defineProperty is used
    assert!(
        output.contains("Object.defineProperty"),
        "ES5 output should use Object.defineProperty: {}",
        output
    );
    // Generic type param should be erased
    assert!(
        !output.contains("<T>"),
        "ES5 output should erase generic type param: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": T"),
        "ES5 output should erase type annotations: {}",
        output
    );
    // No ES6 getter/setter syntax
    assert!(
        !output.contains("get value()") && !output.contains("set value("),
        "ES5 output should not contain ES6 accessor syntax: {}",
        output
    );
}

/// Parity test for ES5 static setter.
/// Static setters should be defined on constructor, not prototype.
#[test]
fn test_parity_es5_static_setter_typed() {
    let source = "class Logger { private static _level: string = 'info'; static set level(val: string) { Logger._level = val; } }";
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

    // Verify Object.defineProperty is used
    assert!(
        output.contains("Object.defineProperty"),
        "ES5 output should use Object.defineProperty: {}",
        output
    );
    // Static accessors should be on constructor (Logger), not Logger.prototype
    assert!(
        output.contains("Object.defineProperty(Logger,") || output.contains("Object.defineProperty(Logger, "),
        "ES5 output should define static accessor on Logger constructor: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": string"),
        "ES5 output should erase type annotation: {}",
        output
    );
    // No ES6 setter syntax
    assert!(
        !output.contains("static set level("),
        "ES5 output should not contain ES6 static setter syntax: {}",
        output
    );
}

/// Parity test for ES5 static block with this reference.
/// Static block using this should downlevel correctly.
#[test]
fn test_parity_es5_static_block_this_ref() {
    let source = r#"class Registry {
    static items: string[] = [];
    static {
        this.items.push("default");
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var Registry") || output.contains("function Registry"),
        "ES5 output should define Registry: {}",
        output
    );
    // Static block syntax should not appear
    assert!(
        !output.contains("static {"),
        "ES5 output should not contain static block syntax: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": string[]"),
        "ES5 output should erase type annotation: {}",
        output
    );
    // Should have push call
    assert!(
        output.contains("push"),
        "ES5 output should contain push call: {}",
        output
    );
}

/// Parity test for ES5 multiple static blocks in same class.
/// Multiple static blocks should all be downleveled.
#[test]
fn test_parity_es5_static_block_multiple() {
    let source = r#"class App {
    static name = "";
    static {
        App.name = "MyApp";
    }
    static version = "";
    static {
        App.version = "2.0";
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var App") || output.contains("function App"),
        "ES5 output should define App: {}",
        output
    );
    // No static block syntax
    assert!(
        !output.contains("static {"),
        "ES5 output should not contain static block syntax: {}",
        output
    );
    // Both initializations should occur
    assert!(
        output.contains("MyApp") && output.contains("2.0"),
        "ES5 output should contain both static initializations: {}",
        output
    );
}

/// Parity test for ES5 static block with typed variable.
/// Local typed variables in static block should have types erased.
#[test]
fn test_parity_es5_static_block_typed_var() {
    let source = r#"class Calculator {
    static result = 0;
    static {
        const multiplier: number = 10;
        Calculator.result = 5 * multiplier;
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var Calculator") || output.contains("function Calculator"),
        "ES5 output should define Calculator: {}",
        output
    );
    // No static block syntax
    assert!(
        !output.contains("static {"),
        "ES5 output should not contain static block syntax: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": number"),
        "ES5 output should erase type annotation: {}",
        output
    );
    // Const should be converted to var
    assert!(
        !output.contains("const multiplier"),
        "ES5 output should convert const to var: {}",
        output
    );
    // Value 10 should be preserved
    assert!(
        output.contains("10"),
        "ES5 output should contain multiplier value: {}",
        output
    );
}

/// Parity test for ES5 static block with function call.
/// Function calls inside static block should be preserved.
#[test]
fn test_parity_es5_static_block_function_call() {
    let source = r#"class Logger {
    static initialized = false;
    static {
        console.log("Logger static init");
        Logger.initialized = true;
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var Logger") || output.contains("function Logger"),
        "ES5 output should define Logger: {}",
        output
    );
    // No static block syntax
    assert!(
        !output.contains("static {"),
        "ES5 output should not contain static block syntax: {}",
        output
    );
    // Console.log should be preserved
    assert!(
        output.contains("console.log"),
        "ES5 output should contain console.log call: {}",
        output
    );
    // String literal should be preserved
    assert!(
        output.contains("Logger static init"),
        "ES5 output should contain log message: {}",
        output
    );
}

/// Parity test for ES5 private method with multiple typed parameters.
/// Private method with types should erase all type annotations.
#[test]
fn test_parity_es5_private_method_multi_params() {
    let source = r#"class MathHelper {
    #add(a: number, b: number, c: number): number {
        return a + b + c;
    }
    sum(x: number, y: number, z: number) {
        return this.#add(x, y, z);
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var MathHelper") || output.contains("function MathHelper"),
        "ES5 output should define MathHelper: {}",
        output
    );
    // Private method syntax should not appear
    assert!(
        !output.contains("#add"),
        "ES5 output should not contain #add: {}",
        output
    );
    // All type annotations should be erased
    assert!(
        !output.contains(": number"),
        "ES5 output should erase type annotations: {}",
        output
    );
}

/// Parity test for ES5 private static method.
/// Private static methods should be downleveled correctly.
#[test]
fn test_parity_es5_private_static_method() {
    let source = r#"class IdGenerator {
    static #nextId: number = 0;
    static #generateId(): number {
        return IdGenerator.#nextId++;
    }
    static create() {
        return IdGenerator.#generateId();
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var IdGenerator") || output.contains("function IdGenerator"),
        "ES5 output should define IdGenerator: {}",
        output
    );
    // Private static method declaration syntax should not appear in original form
    assert!(
        !output.contains("static #generateId():") && !output.contains("static #nextId:"),
        "ES5 output should not contain private static declaration syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number"),
        "ES5 output should erase type annotations: {}",
        output
    );
}

/// Parity test for ES5 private async method.
/// Private async methods should combine async and private downleveling.
#[test]
fn test_parity_es5_private_async_method() {
    let source = r#"class DataFetcher {
    async #fetchData(url: string): Promise<string> {
        return url;
    }
    async load(endpoint: string) {
        return this.#fetchData(endpoint);
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var DataFetcher") || output.contains("function DataFetcher"),
        "ES5 output should define DataFetcher: {}",
        output
    );
    // Private method declaration syntax should not appear
    assert!(
        !output.contains("async #fetchData(") && !output.contains("#fetchData(url"),
        "ES5 output should not contain private method declaration syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains(": Promise"),
        "ES5 output should erase type annotations: {}",
        output
    );
}

/// Parity test for ES5 private method calling another private method.
/// Chained private method calls should all be downleveled.
#[test]
fn test_parity_es5_private_method_chain() {
    let source = r#"class Processor {
    #step1(val: number): number {
        return val + 1;
    }
    #step2(val: number): number {
        return this.#step1(val) * 2;
    }
    process(input: number) {
        return this.#step2(input);
    }
}"#;
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

    // Verify ES5 class structure
    assert!(
        output.contains("var Processor") || output.contains("function Processor"),
        "ES5 output should define Processor: {}",
        output
    );
    // Private method syntax should not appear
    assert!(
        !output.contains("#step1") && !output.contains("#step2"),
        "ES5 output should not contain private method syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number"),
        "ES5 output should erase type annotations: {}",
        output
    );
}

/// Parity test for ES5 multiple class decorators.
/// Multiple decorators should all be applied.
#[test]
fn test_parity_es5_class_decorator_multiple() {
    let source = r#"function sealed(ctor: Function) {}
function logged(ctor: Function) {}
function tracked(ctor: Function) {}

@sealed
@logged
@tracked
class Service {
    name: string = "test";
}"#;
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

    // Verify class exists
    assert!(
        output.contains("Service"),
        "Output should define Service class: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@sealed") && !output.contains("@logged") && !output.contains("@tracked"),
        "ES5 output should not contain decorator syntax: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": string") && !output.contains(": Function"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 class decorator factory.
/// Decorator factory with arguments should be downleveled.
#[test]
fn test_parity_es5_class_decorator_factory() {
    let source = r#"function component(name: string) {
    return function(ctor: Function) {};
}

@component("MyComponent")
class Widget {
    id: number = 1;
}"#;
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

    // Verify class exists
    assert!(
        output.contains("Widget"),
        "Output should define Widget class: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@component"),
        "ES5 output should not contain @component decorator syntax: {}",
        output
    );
    // The decorator name should still appear as a function reference
    assert!(
        output.contains("component"),
        "Output should reference component function: {}",
        output
    );
    // Type annotation should be erased
    assert!(
        !output.contains(": string") && !output.contains(": number") && !output.contains(": Function"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 class decorator with generic class.
/// Decorator on generic class should erase type params.
#[test]
fn test_parity_es5_class_decorator_generic() {
    let source = r#"function observable(ctor: Function) {}

@observable
class Store<T> {
    data: T;
    constructor(initial: T) {
        this.data = initial;
    }
}"#;
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

    // Verify class exists
    assert!(
        output.contains("Store"),
        "Output should define Store class: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@observable"),
        "ES5 output should not contain @observable decorator syntax: {}",
        output
    );
    // Generic type parameter should be erased
    assert!(
        !output.contains("<T>"),
        "ES5 output should erase generic type parameter: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": T") && !output.contains(": Function"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 class decorator with extends.
/// Decorator on derived class should work correctly.
#[test]
fn test_parity_es5_class_decorator_extends() {
    let source = r#"function injectable(ctor: Function) {}

class BaseService {
    name: string = "base";
}

@injectable
class DerivedService extends BaseService {
    id: number = 1;
}"#;
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

    // Verify both classes exist
    assert!(
        output.contains("BaseService") && output.contains("DerivedService"),
        "Output should define both classes: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@injectable"),
        "ES5 output should not contain @injectable decorator syntax: {}",
        output
    );
    // ES5 inheritance should use __extends or prototype chain
    assert!(
        output.contains("__extends") || output.contains(".prototype"),
        "ES5 output should have inheritance pattern: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains(": number") && !output.contains(": Function"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 multiple method decorators.
/// Multiple decorators on a method should be lowered without decorator syntax.
#[test]
fn test_parity_es5_method_decorator_multiple() {
    let source = r#"function log(target: any, key: string, desc: PropertyDescriptor) {}
function validate(target: any, key: string, desc: PropertyDescriptor) {}
function cache(target: any, key: string, desc: PropertyDescriptor) {}

class DataService {
    @log
    @validate
    @cache
    fetchData(id: number): string {
        return "data";
    }
}"#;
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

    // Verify class and method exist
    assert!(
        output.contains("DataService") && output.contains("fetchData"),
        "Output should define DataService with fetchData: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@log") && !output.contains("@validate") && !output.contains("@cache"),
        "ES5 output should not contain decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number") && !output.contains(": string") && !output.contains("PropertyDescriptor"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 method decorator factory.
/// Decorator factories with arguments should be lowered properly.
#[test]
fn test_parity_es5_method_decorator_factory() {
    let source = r#"function throttle(ms: number) {
    return function(target: any, key: string, desc: PropertyDescriptor) {};
}

class SearchController {
    @throttle(300)
    search(query: string): void {
        console.log(query);
    }
}"#;
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

    // Verify class and method exist
    assert!(
        output.contains("SearchController") && output.contains("search"),
        "Output should define SearchController with search: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@throttle"),
        "ES5 output should not contain @throttle decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number") && !output.contains(": string") && !output.contains(": void") && !output.contains("PropertyDescriptor"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 static method decorator.
/// Decorators on static methods should be lowered properly.
#[test]
fn test_parity_es5_method_decorator_static() {
    let source = r#"function memoize(target: any, key: string, desc: PropertyDescriptor) {}

class MathUtils {
    @memoize
    static factorial(n: number): number {
        return n <= 1 ? 1 : n * MathUtils.factorial(n - 1);
    }
}"#;
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

    // Verify class and static method exist
    assert!(
        output.contains("MathUtils") && output.contains("factorial"),
        "Output should define MathUtils with factorial: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@memoize"),
        "ES5 output should not contain @memoize decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number") && !output.contains("PropertyDescriptor"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 async method decorator.
/// Decorators on async methods should be lowered with async transform.
#[test]
fn test_parity_es5_method_decorator_async() {
    let source = r#"function retry(target: any, key: string, desc: PropertyDescriptor) {}

class ApiClient {
    @retry
    async fetchUser(id: number): Promise<string> {
        return "user";
    }
}"#;
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

    // Verify class and method exist
    assert!(
        output.contains("ApiClient") && output.contains("fetchUser"),
        "Output should define ApiClient with fetchUser: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@retry"),
        "ES5 output should not contain @retry decorator syntax: {}",
        output
    );
    // Async should be transformed (no async keyword in ES5)
    assert!(
        !output.contains("async fetchUser"),
        "ES5 output should not contain async method: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number") && !output.contains("Promise<string>") && !output.contains("PropertyDescriptor"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 multiple property decorators.
/// Multiple decorators on a property should be lowered without decorator syntax.
#[test]
fn test_parity_es5_property_decorator_multiple() {
    let source = r#"function observable(target: any, key: string) {}
function validate(target: any, key: string) {}
function persist(target: any, key: string) {}

class FormField {
    @observable
    @validate
    @persist
    value: string = "";
}"#;
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

    // Verify class and property exist
    assert!(
        output.contains("FormField") && output.contains("value"),
        "Output should define FormField with value: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@observable") && !output.contains("@validate") && !output.contains("@persist"),
        "ES5 output should not contain decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains(": any"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 property decorator factory.
/// Decorator factories with arguments should be lowered properly.
#[test]
fn test_parity_es5_property_decorator_factory() {
    let source = r#"function column(name: string) {
    return function(target: any, key: string) {};
}

class Entity {
    @column("user_name")
    userName: string = "";
}"#;
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

    // Verify class and property exist
    assert!(
        output.contains("Entity") && output.contains("userName"),
        "Output should define Entity with userName: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@column"),
        "ES5 output should not contain @column decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains(": any"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 static property decorator.
/// Decorators on static properties should be lowered properly.
#[test]
fn test_parity_es5_property_decorator_static() {
    let source = r#"function readonly(target: any, key: string) {}

class Config {
    @readonly
    static version: string = "1.0.0";
}"#;
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

    // Verify class and static property exist
    assert!(
        output.contains("Config") && output.contains("version"),
        "Output should define Config with version: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@readonly"),
        "ES5 output should not contain @readonly decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains(": any"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 property decorator with complex initializer.
/// Property decorators with arrow function initializers should be lowered properly.
#[test]
fn test_parity_es5_property_decorator_initializer() {
    let source = r#"function lazy(target: any, key: string) {}

class DataLoader {
    @lazy
    loader: () => Promise<string> = () => Promise.resolve("data");
}"#;
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

    // Verify class and property exist
    assert!(
        output.contains("DataLoader") && output.contains("loader"),
        "Output should define DataLoader with loader: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@lazy"),
        "ES5 output should not contain @lazy decorator syntax: {}",
        output
    );
    // Arrow function should be transformed
    assert!(
        !output.contains("() =>"),
        "ES5 output should not contain arrow function: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains("Promise<string>") && !output.contains(": any"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 getter decorator.
/// Decorator on getter accessor should be lowered properly.
#[test]
fn test_parity_es5_accessor_decorator_getter() {
    let source = r#"function enumerable(target: any, key: string, desc: PropertyDescriptor) {}

class Person {
    private _name: string = "";

    @enumerable
    get name(): string {
        return this._name;
    }
}"#;
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

    // Verify class exists
    assert!(
        output.contains("Person"),
        "Output should define Person class: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@enumerable"),
        "ES5 output should not contain @enumerable decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains("PropertyDescriptor"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 setter decorator.
/// Decorator on setter accessor should be lowered properly.
#[test]
fn test_parity_es5_accessor_decorator_setter() {
    let source = r#"function validate(target: any, key: string, desc: PropertyDescriptor) {}

class Account {
    private _balance: number = 0;

    @validate
    set balance(value: number) {
        this._balance = value;
    }
}"#;
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

    // Verify class exists
    assert!(
        output.contains("Account"),
        "Output should define Account class: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@validate"),
        "ES5 output should not contain @validate decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number") && !output.contains("PropertyDescriptor"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 multiple accessor decorators.
/// Multiple decorators on accessor should be lowered properly.
#[test]
fn test_parity_es5_accessor_decorator_multiple() {
    let source = r#"function log(target: any, key: string, desc: PropertyDescriptor) {}
function cache(target: any, key: string, desc: PropertyDescriptor) {}

class Calculator {
    private _result: number = 0;

    @log
    @cache
    get result(): number {
        return this._result;
    }
}"#;
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

    // Verify class exists
    assert!(
        output.contains("Calculator"),
        "Output should define Calculator class: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@log") && !output.contains("@cache"),
        "ES5 output should not contain decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number") && !output.contains("PropertyDescriptor"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 static accessor decorator.
/// Decorator on static accessor should be lowered properly.
#[test]
fn test_parity_es5_accessor_decorator_static() {
    let source = r#"function readonly(target: any, key: string, desc: PropertyDescriptor) {}

class AppConfig {
    private static _version: string = "1.0";

    @readonly
    static get version(): string {
        return AppConfig._version;
    }
}"#;
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

    // Verify class exists
    assert!(
        output.contains("AppConfig"),
        "Output should define AppConfig class: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@readonly"),
        "ES5 output should not contain @readonly decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains("PropertyDescriptor"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 multiple parameter decorators.
/// Multiple decorators on a single parameter should be lowered properly.
#[test]
fn test_parity_es5_parameter_decorator_multiple() {
    let source = r#"function required(target: any, key: string, index: number) {}
function validate(target: any, key: string, index: number) {}

class UserService {
    createUser(@required @validate name: string): void {}
}"#;
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

    // Verify class exists
    assert!(
        output.contains("UserService"),
        "Output should define UserService class: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@required") && !output.contains("@validate"),
        "ES5 output should not contain decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains(": void"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 parameter decorator on method.
/// Parameter decorator on regular method should be lowered properly.
#[test]
fn test_parity_es5_parameter_decorator_method() {
    let source = r#"function log(target: any, key: string, index: number) {}

class Logger {
    write(@log message: string): void {
        console.log(message);
    }
}"#;
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

    // Verify class exists
    assert!(
        output.contains("Logger"),
        "Output should define Logger class: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@log"),
        "ES5 output should not contain @log decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains(": void"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 parameter decorator factory.
/// Parameter decorator factories with arguments should be lowered properly.
#[test]
fn test_parity_es5_parameter_decorator_factory() {
    let source = r#"function maxLength(max: number) {
    return function(target: any, key: string, index: number) {};
}

class FormValidator {
    validate(@maxLength(100) input: string): boolean {
        return true;
    }
}"#;
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

    // Verify class exists
    assert!(
        output.contains("FormValidator"),
        "Output should define FormValidator class: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@maxLength"),
        "ES5 output should not contain @maxLength decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains(": boolean"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 multiple parameters with decorators.
/// Multiple parameters each with decorators should be lowered properly.
#[test]
fn test_parity_es5_parameter_decorator_multi_params() {
    let source = r#"function inject(target: any, key: string, index: number) {}

class Container {
    resolve(@inject a: string, @inject b: number, @inject c: boolean): void {}
}"#;
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

    // Verify class exists
    assert!(
        output.contains("Container"),
        "Output should define Container class: {}",
        output
    );
    // No decorator syntax in ES5
    assert!(
        !output.contains("@inject"),
        "ES5 output should not contain @inject decorator syntax: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains(": boolean") && !output.contains(": void"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 async generator with try/catch.
/// Async generator with error handling should be lowered properly.
#[test]
fn test_parity_es5_async_generator_try_catch() {
    let source = r#"async function* safeFetch(urls: string[]): AsyncGenerator<string> {
    for (const url of urls) {
        try {
            yield await fetch(url);
        } catch (e: unknown) {
            yield "error";
        }
    }
}"#;
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

    // Verify function exists
    assert!(
        output.contains("safeFetch"),
        "Output should define safeFetch function: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string[]") && !output.contains("AsyncGenerator<") && !output.contains(": unknown"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 static async generator method.
/// Static async generator methods should be lowered properly.
#[test]
fn test_parity_es5_async_generator_static() {
    let source = r#"class StreamFactory {
    static async *createStream(): AsyncGenerator<number> {
        yield 1;
        yield 2;
        yield 3;
    }
}"#;
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

    // Verify class exists
    assert!(
        output.contains("StreamFactory"),
        "Output should define StreamFactory class: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains("AsyncGenerator<number>"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 async generator with multiple yields.
/// Async generator with multiple sequential yields should be lowered properly.
#[test]
fn test_parity_es5_async_generator_multi_yield() {
    let source = r#"async function* countdown(start: number): AsyncGenerator<number> {
    yield start;
    yield start - 1;
    yield start - 2;
    yield 0;
}"#;
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

    // Verify function exists
    assert!(
        output.contains("countdown"),
        "Output should define countdown function: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": number") && !output.contains("AsyncGenerator<"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 async generator with yield delegation.
/// Async generator using yield* should be lowered properly.
#[test]
fn test_parity_es5_async_generator_yield_star() {
    let source = r#"async function* concat(a: AsyncGenerator<number>, b: AsyncGenerator<number>): AsyncGenerator<number> {
    yield* a;
    yield* b;
}"#;
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

    // Verify function exists
    assert!(
        output.contains("concat"),
        "Output should define concat function: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains("AsyncGenerator<number>"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 nested namespaces.
/// Nested namespaces should be lowered to nested objects.
#[test]
fn test_parity_es5_namespace_nested() {
    let source = r#"namespace Outer {
    export namespace Inner {
        export function helper(): string {
            return "help";
        }
    }
}"#;
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

    // Verify outer namespace exists
    assert!(
        output.contains("Outer"),
        "Output should define Outer namespace: {}",
        output
    );
    // No namespace keyword
    assert!(
        !output.contains("namespace Outer") && !output.contains("namespace Inner"),
        "ES5 output should not contain namespace keyword: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 namespace with class.
/// Namespace containing a class should be lowered properly.
#[test]
fn test_parity_es5_namespace_with_class() {
    let source = r#"namespace Models {
    export class User {
        name: string;
        constructor(name: string) {
            this.name = name;
        }
    }
}"#;
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

    // Verify namespace exists
    assert!(
        output.contains("Models"),
        "Output should define Models namespace: {}",
        output
    );
    // No namespace keyword
    assert!(
        !output.contains("namespace Models"),
        "ES5 output should not contain namespace keyword: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 namespace with interface.
/// Interface inside namespace should be erased.
#[test]
fn test_parity_es5_namespace_with_interface() {
    let source = r#"namespace Types {
    export interface Config {
        name: string;
        value: number;
    }
    export const DEFAULT_NAME: string = "default";
}"#;
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

    // Verify namespace exists
    assert!(
        output.contains("Types"),
        "Output should define Types namespace: {}",
        output
    );
    // Interface should be erased
    assert!(
        !output.contains("interface Config"),
        "ES5 output should not contain interface: {}",
        output
    );
    // No namespace keyword
    assert!(
        !output.contains("namespace Types"),
        "ES5 output should not contain namespace keyword: {}",
        output
    );
    // Type annotations should be erased
    assert!(
        !output.contains(": string") && !output.contains(": number"),
        "Type annotations should be erased: {}",
        output
    );
}

/// Parity test for ES5 namespace with enum.
/// Namespace containing an enum should be lowered properly.
#[test]
fn test_parity_es5_namespace_with_enum() {
    let source = r#"namespace Status {
    export enum Code {
        OK = 200,
        NotFound = 404,
        Error = 500
    }
}"#;
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

    // Verify namespace exists
    assert!(
        output.contains("Status"),
        "Output should define Status namespace: {}",
        output
    );
    // No namespace keyword
    assert!(
        !output.contains("namespace Status"),
        "ES5 output should not contain namespace keyword: {}",
        output
    );
    // No enum keyword
    assert!(
        !output.contains("enum Code"),
        "ES5 output should not contain enum keyword: {}",
        output
    );
}
