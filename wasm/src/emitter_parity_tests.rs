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
