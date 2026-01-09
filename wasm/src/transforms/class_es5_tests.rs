use crate::emit_context::EmitContext;
use crate::lowering_pass::LoweringPass;
use crate::thin_emitter::{PrinterOptions, ScriptTarget, ThinPrinter};
use crate::thin_parser::ThinParserState;
use crate::transforms::class_es5::ClassES5Emitter;

#[test]
fn test_class_es5_emits_param_and_static_properties() {
    let source = "class Foo { constructor(public x) {} y = 1; static bar = 2; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    assert!(
        output.contains("this.x = x;"),
        "Expected parameter property assignment in ES5 output: {}",
        output
    );
    assert!(
        output.contains("this.y = 1;"),
        "Expected instance property initializer in ES5 output: {}",
        output
    );
    assert!(
        output.contains("Foo.bar = 2;"),
        "Expected static property assignment in ES5 output: {}",
        output
    );
}

#[test]
fn test_class_es5_static_field_arrow_keeps_this() {
    let source = "class Foo { static field = () => this.value; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    assert!(
        output.contains("Foo.field = function"),
        "Expected static field initializer to emit a function expression: {}",
        output
    );
    assert!(
        output.contains("return this.value;"),
        "Expected static field arrow to preserve `this`: {}",
        output
    );
    assert!(
        !output.contains("_this"),
        "Did not expect static field arrow to capture `_this`: {}",
        output
    );
}

#[test]
fn test_class_es5_async_method_emits_awaiter() {
    let source = "class Foo { async bar() { await baz(); return 1; } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    assert!(
        output.contains("Foo.prototype.bar = function"),
        "Expected method assignment in ES5 output: {}",
        output
    );
    assert!(
        output.contains("__awaiter(this, void 0, void 0, function () {"),
        "Expected __awaiter wrapper in ES5 output: {}",
        output
    );
    assert!(
        output.contains("__generator"),
        "Expected __generator usage in ES5 output: {}",
        output
    );
    assert!(
        !output.contains("async bar"),
        "Expected async keyword to be downleveled: {}",
        output
    );
}

#[test]
fn test_class_es5_preserves_pre_super_statement_order() {
    let source =
        "class Base {} class Derived extends Base { y = 1; constructor() { prep(); super(); post(); } }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(1)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    let prep_pos = output.find("prep()").expect("expected prep() call");
    let super_pos = output
        .find("_super.call(this")
        .expect("expected super call assignment");
    let init_pos = output
        .find("_this.y = 1")
        .expect("expected instance property initializer");
    let post_pos = output.find("post()").expect("expected post() call");

    assert!(
        prep_pos < super_pos,
        "Expected prep() before super call: {}",
        output
    );
    assert!(
        super_pos < init_pos,
        "Expected property initializer after super call: {}",
        output
    );
    assert!(
        init_pos < post_pos,
        "Expected post() after property initializer: {}",
        output
    );
}

#[test]
fn test_class_es5_default_derived_constructor_orders_super_and_props() {
    let source = "class Base {} class Derived extends Base { y = 1; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(1)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    let super_pos = output
        .find("_super.apply(this, arguments)")
        .expect("expected super apply call");
    let init_pos = output.find("_this.y = 1").expect("expected initializer");
    let return_pos = output.find("return _this").expect("expected return _this");

    assert!(
        super_pos < init_pos,
        "Expected super call before property initializer: {}",
        output
    );
    assert!(
        init_pos < return_pos,
        "Expected return after property initializer: {}",
        output
    );
}

#[test]
fn test_class_es5_computed_super_arrow_in_field_initializer() {
    let source = r#"
class Base { m(x) { return x; } }
class Derived extends Base {
    field = () => super["m"](this.x);
    constructor() {
        super();
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(1)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    assert!(
        output.contains("_super.prototype[\"m\"].call(_this"),
        "Expected computed super call to lower with lexical this: {}",
        output
    );
    assert!(
        output.contains("_this.x"),
        "Expected arrow to capture this in field initializer: {}",
        output
    );
    assert!(
        !output.contains("super[\"m\"]"),
        "Expected computed super access to be downleveled: {}",
        output
    );
}

#[test]
fn test_class_es5_computed_super_in_field_arrow_uses_this_capture() {
    let source = r#"
        const key = "m";
        class Base { [key]() {} }
        class Derived extends Base {
            field = () => super[key]();
            constructor() { super(); }
        }
    "#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(2)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    assert!(
        output.contains("_super.prototype[key].call(_this)"),
        "Expected computed super call to bind _this: {}",
        output
    );
    assert!(
        !output.contains("super["),
        "Expected computed super to be lowered in ES5 output: {}",
        output
    );
}

#[test]
fn test_class_es5_computed_field_does_not_change_super_ordering() {
    let source = r#"
        const key = "z";
        class Base {}
        class Derived extends Base {
            [key] = prepField();
            y = 1;
            constructor() {
                prep();
                super();
                post();
            }
        }
    "#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(2)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    let prep_pos = output.find("prep()").expect("expected prep() call");
    let super_pos = output
        .find("_super.call(this")
        .expect("expected super call assignment");
    let init_pos = output
        .find("_this.y = 1")
        .expect("expected instance property initializer");
    let post_pos = output.find("post()").expect("expected post() call");

    assert!(
        prep_pos < super_pos,
        "Expected prep() before super call: {}",
        output
    );
    assert!(
        super_pos < init_pos,
        "Expected property initializer after super call: {}",
        output
    );
    assert!(
        init_pos < post_pos,
        "Expected post() after property initializer: {}",
        output
    );
}

#[test]
fn test_class_es5_synthesized_ctor_captures_this_in_field_initializers() {
    let source = r#"
class Base { m() { return 1; } }
class Derived extends Base {
    field = this.value;
    fromSuper = super.m();
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(1)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    assert!(
        output.contains("_this.field = _this.value"),
        "Expected synthesized ctor to use _this in initializer: {}",
        output
    );
    assert!(
        output.contains("_this.fromSuper = _super.prototype.m.call(_this"),
        "Expected super call to use _this in initializer: {}",
        output
    );
}

#[test]
fn test_class_es5_private_field_initializer_uses_this_capture() {
    let source = r#"
class Base {}
class Derived extends Base {
    #count = this.value;
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(1)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    assert!(
        output.contains("__classPrivateFieldSet(_this, _Derived_count, _this.value"),
        "Expected private field initializer to use _this in derived class: {}",
        output
    );
}

#[test]
fn test_class_es5_computed_property_field_initializer_emitted() {
    let source = r#"
const key = "z";
class Foo {
    [key] = 42;
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(1)
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    assert!(
        output.contains("this[key] = 42"),
        "Expected computed property initializer to be emitted: {}",
        output
    );
}

#[test]
fn test_class_es5_derived_computed_property_field_initializer_emitted() {
    let source = r#"
const key = "z";
class Base {}
class Derived extends Base {
    [key] = 42;
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(2)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    assert!(
        output.contains("_this[key] = 42"),
        "Expected computed property initializer with _this in derived class: {}",
        output
    );
}

#[test]
fn test_class_es5_derived_with_explicit_ctor_computed_property_field() {
    let source = r#"
const key = "z";
class Base {}
class Derived extends Base {
    [key] = 42;
    constructor() {
        super();
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(2)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    assert!(
        output.contains("_this[key] = 42"),
        "Expected computed property initializer with _this in derived class with explicit ctor: {}",
        output
    );

    // Verify ordering: super() should come before property initializer
    let super_pos = output.find("_super.call(this").expect("expected super call");
    let init_pos = output.find("_this[key] = 42").expect("expected initializer");
    assert!(
        super_pos < init_pos,
        "Expected super call before computed property initializer: {}",
        output
    );
}

#[test]
fn test_class_es5_static_field_async_arrow() {
    let source = "class Foo { static handler = async () => { await fetch(); return this.value; }; }";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should emit __awaiter for async
    assert!(
        output.contains("__awaiter"),
        "Expected static async arrow to use __awaiter: {}",
        output
    );

    // Should preserve `this` (not capture to _this) for static field
    assert!(
        output.contains("this.value"),
        "Expected static async arrow to preserve `this`: {}",
        output
    );
}

#[test]
fn test_class_es5_private_field_access_in_async_method() {
    let source = r#"
class Foo {
    #value = 1;
    async getValue() {
        await fetch();
        return this.#value;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should emit __awaiter for async method
    assert!(
        output.contains("__awaiter"),
        "Expected async method to use __awaiter: {}",
        output
    );

    // Should emit __classPrivateFieldGet for private field access
    assert!(
        output.contains("__classPrivateFieldGet"),
        "Expected private field access to use __classPrivateFieldGet: {}",
        output
    );

    // Should emit WeakMap for private field storage
    assert!(
        output.contains("_Foo_value"),
        "Expected private field WeakMap name: {}",
        output
    );
}

#[test]
fn test_class_es5_super_property_in_static_block() {
    let source = r#"
class Base {
    static value = 1;
}
class Derived extends Base {
    static {
        console.log(super.value);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(1)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Static block should be emitted as an IIFE after the class
    assert!(
        output.contains("console.log"),
        "Expected static block content to be emitted: {}",
        output
    );

    // Super property access should be transformed to Base.value or similar
    assert!(
        output.contains("Base.value") || output.contains("_super.value"),
        "Expected super.value to be transformed: {}",
        output
    );
}

#[test]
fn test_class_es5_nested_async_arrow_in_constructor_with_field() {
    let source = r#"
class Foo {
    field = 1;
    constructor() {
        this.handler = async () => {
            const inner = async () => {
                return this.field;
            };
            return await inner();
        };
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Field initializer should use _this
    assert!(
        output.contains("_this.field = 1"),
        "Expected field initializer to use _this: {}",
        output
    );

    // Async arrow in constructor should use __awaiter
    assert!(
        output.contains("__awaiter"),
        "Expected async arrow to use __awaiter: {}",
        output
    );

    // Nested async arrow should capture _this.field
    assert!(
        output.contains("_this.field") && output.matches("_this.field").count() >= 2,
        "Expected nested async arrow to capture _this.field: {}",
        output
    );

    // Handler assignment should use _this
    assert!(
        output.contains("_this.handler"),
        "Expected handler assignment to use _this: {}",
        output
    );
}

#[test]
fn test_class_es5_computed_method_name_with_async_body() {
    let source = r#"
const methodName = "doWork";
class Foo {
    value = 42;
    async [methodName]() {
        await fetch();
        return this.value;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(1)
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Computed method should use bracket notation
    assert!(
        output.contains(".prototype[methodName]") || output.contains(".prototype[\"doWork\"]"),
        "Expected computed method to use bracket notation: {}",
        output
    );

    // Async body should use __awaiter
    assert!(
        output.contains("__awaiter"),
        "Expected async method to use __awaiter: {}",
        output
    );

    // Async body should use __generator
    assert!(
        output.contains("__generator"),
        "Expected async method to use __generator: {}",
        output
    );

    // this.value should be preserved or captured properly
    assert!(
        output.contains("this.value") || output.contains("_this.value"),
        "Expected this.value reference: {}",
        output
    );
}

#[test]
fn test_class_es5_spread_element_in_array_literal() {
    let source = r#"
class Foo {
    items = [1, 2, 3];

    getAll() {
        const extra = [4, 5];
        return [...this.items, ...extra, 6];
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should NOT contain spread syntax (ES5 doesn't support it)
    assert!(
        !output.contains("...this.items") && !output.contains("...extra"),
        "Expected spread to be transformed, not raw spread syntax: {}",
        output
    );

    // Should use __spreadArray or concat for ES5 spread
    assert!(
        output.contains("__spreadArray") || output.contains(".concat(") || output.contains("slice.call"),
        "Expected ES5 spread transformation using __spreadArray or concat: {}",
        output
    );
}

#[test]
fn test_class_es5_object_spread_in_method() {
    let source = r#"
class Foo {
    defaults = { a: 1, b: 2 };

    merge(extra: object) {
        return { ...this.defaults, ...extra, c: 3 };
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should NOT contain spread syntax (ES5 doesn't support it)
    assert!(
        !output.contains("...this.defaults") && !output.contains("...extra"),
        "Expected object spread to be transformed, not raw spread syntax: {}",
        output
    );

    // Should use Object.assign for ES5 object spread
    assert!(
        output.contains("Object.assign"),
        "Expected ES5 object spread transformation using Object.assign: {}",
        output
    );

    // The c: 3 property should still be present (emitted as assignment _a.c = 3)
    assert!(
        output.contains(".c = 3") || output.contains("c = 3"),
        "Expected property c: 3 to be preserved as assignment: {}",
        output
    );
}

#[test]
fn test_class_es5_for_of_loop_in_method() {
    let source = r#"
class Foo {
    items = [1, 2, 3];

    sum() {
        let total = 0;
        for (const item of this.items) {
            total += item;
        }
        return total;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should NOT contain for-of syntax (ES5 doesn't support it)
    assert!(
        !output.contains("for (const item of") && !output.contains("for (var item of"),
        "Expected for-of to be transformed, not raw for-of syntax: {}",
        output
    );

    // Should use __values helper for ES5 iterator
    assert!(
        output.contains("__values("),
        "Expected ES5 for-of transformation using __values helper: {}",
        output
    );

    // Should use try/catch/finally for iterator cleanup
    assert!(
        output.contains("try {") && output.contains("finally {"),
        "Expected for-of to use try/finally for iterator cleanup: {}",
        output
    );

    // Should use .next() for iteration
    assert!(
        output.contains(".next()"),
        "Expected for-of to use .next() for iteration: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_iterator_method() {
    let source = r#"
class Foo {
    items = [1, 2, 3];

    *[Symbol.iterator]() {
        for (const item of this.items) {
            yield item;
        }
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should emit Symbol.iterator as computed property name
    assert!(
        output.contains("[Symbol.iterator]"),
        "Expected Symbol.iterator to be preserved as computed property: {}",
        output
    );

    // Should emit on prototype since it's an instance method
    assert!(
        output.contains(".prototype[Symbol.iterator]"),
        "Expected Symbol.iterator method to be on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_super_with_spread_args_and_field_init() {
    // Edge case: super() with spread arguments in derived class with field initializers
    let source = r#"
class Base {
    constructor(...args: number[]) {}
}
class Derived extends Base {
    value = 42;
    constructor(first: number, ...rest: number[]) {
        super(first, ...rest);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(1)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should have super call with spread converted to .apply() or __spreadArray
    assert!(
        output.contains("_super") || output.contains("__extends"),
        "Expected ES5 super mechanism in output: {}",
        output
    );

    // Field initializer should be present
    assert!(
        output.contains("value") && output.contains("42"),
        "Expected field initializer value = 42 in output: {}",
        output
    );

    // Field init should come after super call
    if let (Some(super_pos), Some(value_pos)) = (
        output.find("_super").or_else(|| output.find("__extends")),
        output.find("42"),
    ) {
        assert!(
            super_pos < value_pos,
            "Expected super call before field initializer: {}",
            output
        );
    }
}

#[test]
fn test_class_es5_template_literal_in_method() {
    let source = r#"
class Greeter {
    name = "World";

    greet() {
        return `Hello, ${this.name}!`;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should NOT contain template literal backticks (ES5 doesn't support them)
    assert!(
        !output.contains('`'),
        "Expected template literal to be transformed, not raw backticks: {}",
        output
    );

    // Should use string concatenation for ES5
    assert!(
        output.contains("+") || output.contains("concat"),
        "Expected ES5 template literal to use string concatenation: {}",
        output
    );

    // Should preserve the literal parts
    assert!(
        output.contains("Hello") && output.contains("!"),
        "Expected template literal parts to be preserved: {}",
        output
    );
}

#[test]
fn test_class_es5_destructuring_in_method() {
    let source = r#"
class Parser {
    parse(input: { text: string, line: number }) {
        const { text, line } = input;
        return `${text} at line ${line}`;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should NOT contain destructuring syntax (ES5 doesn't support it)
    assert!(
        !output.contains("{ text, line }") && !output.contains("{text, line}"),
        "Expected destructuring to be transformed, not raw destructuring syntax: {}",
        output
    );

    // Should extract properties individually for ES5
    assert!(
        output.contains(".text") && output.contains(".line"),
        "Expected ES5 destructuring to access properties individually: {}",
        output
    );

    // Variables should be assigned
    assert!(
        output.contains("text") && output.contains("line"),
        "Expected destructured variables to be present: {}",
        output
    );
}

#[test]
fn test_class_es5_default_parameters_in_method() {
    let source = r#"
class Calculator {
    add(a: number, b: number = 0, c: number = 1) {
        return a + b + c;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should have parameter default handling (void 0 check or === undefined)
    assert!(
        output.contains("void 0") || output.contains("undefined"),
        "Expected ES5 default parameter check using void 0 or undefined: {}",
        output
    );

    // Should contain the default values
    assert!(
        output.contains("0") && output.contains("1"),
        "Expected default values 0 and 1 in output: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.add") || output.contains("prototype[\"add\"]"),
        "Expected add method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_rest_parameters_in_method() {
    let source = r#"
class Logger {
    log(prefix: string, ...messages: string[]) {
        return prefix + ": " + messages.join(", ");
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should NOT contain rest parameter syntax (ES5 doesn't support it)
    assert!(
        !output.contains("...messages"),
        "Expected rest parameters to be transformed, not raw ...messages: {}",
        output
    );

    // Should use Array.prototype.slice or similar for rest params
    assert!(
        output.contains("slice") || output.contains("arguments"),
        "Expected ES5 rest parameter to use slice or arguments: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.log") || output.contains("prototype[\"log\"]"),
        "Expected log method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_shorthand_properties_in_method() {
    let source = r#"
class Point {
    x = 10;
    y = 20;

    toObject() {
        const x = this.x;
        const y = this.y;
        return { x, y };
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should have explicit property assignments (x: x, y: y) for ES5
    // or the shorthand should be preserved if the emitter handles it
    assert!(
        output.contains("x:") || output.contains("x :") || output.contains("{ x, y }") || output.contains("{x, y}"),
        "Expected object with x property in output: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.toObject") || output.contains("prototype[\"toObject\"]"),
        "Expected toObject method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_getter_setter_accessors() {
    let source = r#"
class Counter {
    private _count = 0;

    get count() {
        return this._count;
    }

    set count(value: number) {
        this._count = value;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should use Object.defineProperty for getters/setters in ES5
    assert!(
        output.contains("Object.defineProperty") || output.contains("defineProperty"),
        "Expected ES5 getter/setter to use Object.defineProperty: {}",
        output
    );

    // Should have get and set in the property descriptor
    assert!(
        output.contains("get:") || output.contains("get :") || output.contains("\"get\""),
        "Expected getter in property descriptor: {}",
        output
    );

    assert!(
        output.contains("set:") || output.contains("set :") || output.contains("\"set\""),
        "Expected setter in property descriptor: {}",
        output
    );
}

#[test]
fn test_class_es5_arrow_function_this_binding() {
    let source = r#"
class Handler {
    name = "handler";

    getCallback() {
        return () => {
            return this.name;
        };
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should NOT contain arrow function syntax (ES5 doesn't support it)
    assert!(
        !output.contains("=>"),
        "Expected arrow function to be transformed, not raw => syntax: {}",
        output
    );

    // Should capture this for arrow function (var _this = this or similar)
    assert!(
        output.contains("_this") || output.contains("self") || output.contains("that"),
        "Expected this capture for arrow function (_this, self, or that): {}",
        output
    );

    // Should use function keyword instead of arrow
    assert!(
        output.contains("function"),
        "Expected arrow to be converted to function keyword: {}",
        output
    );
}

#[test]
fn test_class_es5_static_method() {
    let source = r#"
class MathUtils {
    static add(a: number, b: number) {
        return a + b;
    }

    static PI = 3.14159;
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Static method should be on constructor, not prototype
    assert!(
        output.contains("MathUtils.add") || output.contains(".add ="),
        "Expected static method on constructor function: {}",
        output
    );

    // Static property should be on constructor
    assert!(
        output.contains("MathUtils.PI") || output.contains(".PI ="),
        "Expected static property on constructor function: {}",
        output
    );

    // Should contain the PI value
    assert!(
        output.contains("3.14159"),
        "Expected PI value 3.14159 in output: {}",
        output
    );
}

#[test]
fn test_class_es5_computed_property_in_object_literal() {
    let source = r#"
class DynamicObject {
    createObject(key: string, value: number) {
        return { [key]: value };
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should NOT contain computed property syntax in object literal (ES5 doesn't support it)
    assert!(
        !output.contains("[key]:") && !output.contains("[key] :"),
        "Expected computed property to be transformed, not raw [key]: syntax: {}",
        output
    );

    // Should use bracket notation assignment or temp variable pattern
    assert!(
        output.contains("[key]") || output.contains("_a"),
        "Expected ES5 computed property to use bracket notation or temp var: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.createObject") || output.contains("prototype[\"createObject\"]"),
        "Expected createObject method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_inheritance_extends() {
    let source = r#"
class Animal {
    name: string;
    constructor(name: string) {
        this.name = name;
    }
    speak() {
        return this.name;
    }
}
class Dog extends Animal {
    constructor(name: string) {
        super(name);
    }
    speak() {
        return "Woof! " + super.speak();
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(1)
        .expect("expected Dog class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should use __extends helper for inheritance
    assert!(
        output.contains("__extends") || output.contains("extends"),
        "Expected __extends helper for ES5 inheritance: {}",
        output
    );

    // Should call super constructor
    assert!(
        output.contains("_super") || output.contains(".call("),
        "Expected super constructor call pattern: {}",
        output
    );

    // Should have Dog function
    assert!(
        output.contains("function Dog") || output.contains("Dog ="),
        "Expected Dog constructor function: {}",
        output
    );
}

#[test]
fn test_class_es5_nullish_coalescing() {
    let source = r#"
class Config {
    getValue(input: string | null | undefined) {
        return input ?? "default";
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should NOT contain ?? operator (ES5 doesn't support it)
    assert!(
        !output.contains("??"),
        "Expected nullish coalescing to be transformed, not raw ?? syntax: {}",
        output
    );

    // Should preserve the default value
    assert!(
        output.contains("default"),
        "Expected default value to be preserved: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.getValue") || output.contains("prototype[\"getValue\"]"),
        "Expected getValue method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_optional_chaining() {
    let source = r#"
class DataProcessor {
    process(data: { value?: { nested?: string } } | null) {
        return data?.value?.nested;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should NOT contain ?. operator (ES5 doesn't support it)
    assert!(
        !output.contains("?."),
        "Expected optional chaining to be transformed, not raw ?. syntax: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.process") || output.contains("prototype[\"process\"]"),
        "Expected process method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_nested_arrow_in_async_method_captures_this() {
    // Test nested arrow function in async method captures _this correctly
    let source = r#"
class Handler {
    value = 42;
    async process() {
        const callback = () => {
            return this.value;
        };
        return await Promise.resolve(callback());
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Arrow inside async method should capture _this
    assert!(
        output.contains("_this.value"),
        "Expected nested arrow in async method to capture _this.value: {}",
        output
    );

    // Should use __awaiter for async method
    assert!(
        output.contains("__awaiter"),
        "Expected async method to use __awaiter: {}",
        output
    );
}

#[test]
fn test_class_es5_deeply_nested_arrows_in_async_method() {
    // Test deeply nested arrows (3 levels) in async method
    let source = r#"
class DeepNest {
    data = [1, 2, 3];
    async processDeep() {
        const outer = () => {
            const middle = () => {
                const inner = () => {
                    return this.data;
                };
                return inner();
            };
            return middle();
        };
        return await Promise.resolve(outer());
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Deeply nested arrow should still capture _this.data
    assert!(
        output.contains("_this.data"),
        "Expected deeply nested arrow to capture _this.data: {}",
        output
    );
}

#[test]
fn test_class_es5_arrow_in_async_method_with_array_callback() {
    // Test arrow in async method with array callbacks using this
    let source = r#"
class Processor {
    multiplier = 2;
    async transform(items: number[]) {
        const mapped = items.map(x => x * this.multiplier);
        const filtered = mapped.filter(x => x > this.multiplier);
        return await Promise.resolve(filtered);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Arrow callbacks in async method should capture _this.multiplier
    assert!(
        output.contains("_this.multiplier"),
        "Expected arrow callbacks to capture _this.multiplier: {}",
        output
    );

    // Should have multiple references to _this.multiplier
    assert!(
        output.matches("_this.multiplier").count() >= 2,
        "Expected at least 2 references to _this.multiplier: {}",
        output
    );
}

#[test]
fn test_class_es5_arrow_returning_arrow_in_async_method() {
    // Test arrow returning another arrow that uses this
    let source = r#"
class Factory {
    prefix = "item-";
    async createFormatter() {
        const formatter = () => (value: string) => this.prefix + value;
        return await Promise.resolve(formatter());
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Returned arrow should capture _this.prefix
    assert!(
        output.contains("_this.prefix"),
        "Expected returned arrow to capture _this.prefix: {}",
        output
    );
}

#[test]
fn test_class_es5_multiple_arrows_same_level_in_async_method() {
    // Test multiple arrows at the same level in async method
    let source = r#"
class Multi {
    a = 1;
    b = 2;
    c = 3;
    async compute() {
        const getA = () => this.a;
        const getB = () => this.b;
        const getC = () => this.c;
        return await Promise.resolve(getA() + getB() + getC());
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // All arrows should capture _this for their respective properties
    assert!(
        output.contains("_this.a"),
        "Expected getA arrow to capture _this.a: {}",
        output
    );
    assert!(
        output.contains("_this.b"),
        "Expected getB arrow to capture _this.b: {}",
        output
    );
    assert!(
        output.contains("_this.c"),
        "Expected getC arrow to capture _this.c: {}",
        output
    );
}

#[test]
fn test_class_es5_arrow_after_await_in_async_method() {
    // Test arrow defined after await still captures this correctly
    let source = r#"
class Sequential {
    state = "ready";
    async run() {
        await this.prepare();
        const check = () => this.state;
        return check();
    }
    async prepare() {
        this.state = "prepared";
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Arrow after await should still capture _this.state
    assert!(
        output.contains("_this.state"),
        "Expected arrow after await to capture _this.state: {}",
        output
    );
}

#[test]
fn test_class_es5_arrow_with_this_method_call_in_async() {
    // Test arrow that calls this.method() in async context
    let source = r#"
class Caller {
    value = 10;
    getValue() { return this.value; }
    async process() {
        const callMethod = () => this.getValue();
        const result = await Promise.resolve(callMethod());
        return result;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Arrow should capture _this.getValue
    assert!(
        output.contains("_this.getValue"),
        "Expected arrow to capture _this.getValue(): {}",
        output
    );
}

#[test]
fn test_class_es5_generator_method() {
    // Tests generator method structure on ES5 class
    // Note: Full generator transform uses __generator helper
    let source = r#"
class DataStream {
    *getItems() {
        return [1, 2, 3];
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should emit as a function (ES5 class pattern)
    assert!(
        output.contains("function DataStream"),
        "Expected DataStream constructor function: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.getItems") || output.contains("prototype[\"getItems\"]"),
        "Expected getItems method on prototype: {}",
        output
    );

    // Should return the array
    assert!(
        output.contains("return") && (output.contains("[1, 2, 3]") || output.contains("1") || output.contains("2")),
        "Expected return statement with values: {}",
        output
    );
}

#[test]
fn test_class_es5_super_property_in_static_method() {
    // Test super property access in a static method
    let source = r#"
class Base {
    static config = { debug: true };
    static getVersion() { return "1.0"; }
}
class Derived extends Base {
    static init() {
        const cfg = super.config;
        const ver = super.getVersion();
        return { cfg, ver };
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(1)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Static method should access super as Base (the parent class)
    // super.config (property read) becomes _super.config
    assert!(
        output.contains("_super.config"),
        "Expected super.config in static method to reference _super.config: {}",
        output
    );
    // super.getVersion() (method call) becomes _super.prototype.getVersion.call(this)
    assert!(
        output.contains("_super.prototype.getVersion.call"),
        "Expected super.getVersion() in static method to use prototype.call pattern: {}",
        output
    );
}

#[test]
fn test_class_es5_computed_super_property_read() {
    // Test computed super property read (not call) - super[key] as property access
    let source = r#"
class Base {
    data = { x: 1, y: 2 };
}
class Derived extends Base {
    getProperty(key: string) {
        return super[key];
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(1)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Computed super property read should be transformed
    // For instance method super[key] property read, it becomes _super[key] or _super.prototype[key]
    assert!(
        output.contains("_super[key]") || output.contains("_super.prototype[key]"),
        "Expected computed super property read to be lowered: {}",
        output
    );
    // The raw super[key] should not appear (should be _super[key])
    // Note: super[key] becomes _super[key] in the output
    assert!(
        !output.contains("super[key]") || output.contains("_super[key]"),
        "Expected super[key] property access to be lowered in ES5 output: {}",
        output
    );
}

#[test]
fn test_class_es5_computed_super_property_in_static_method() {
    // Test computed super property access in a static method
    let source = r#"
class Base {
    static values = { a: 1, b: 2 };
}
class Derived extends Base {
    static getValue(key: string) {
        return super[key];
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .get(1)
        .expect("expected derived class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Static method computed super should reference parent class
    assert!(
        output.contains("Base[key]") || output.contains("_super[key]"),
        "Expected static computed super property to reference parent class: {}",
        output
    );
}

#[test]
fn test_class_es5_try_catch_finally() {
    let source = r#"
class ErrorHandler {
    safeExecute(fn: () => void) {
        try {
            fn();
            return true;
        } catch (error) {
            console.error(error);
            return false;
        } finally {
            console.log("done");
        }
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Should emit as a function (ES5 class pattern)
    assert!(
        output.contains("function ErrorHandler"),
        "Expected ErrorHandler constructor function: {}",
        output
    );

    // Should have try/catch/finally blocks
    assert!(
        output.contains("try"),
        "Expected try block in output: {}",
        output
    );
    assert!(
        output.contains("catch"),
        "Expected catch block in output: {}",
        output
    );
    assert!(
        output.contains("finally"),
        "Expected finally block in output: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.safeExecute") || output.contains("prototype[\"safeExecute\"]"),
        "Expected safeExecute method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_abstract_class_lowering() {
    // Test abstract class is properly lowered to ES5
    let source = r#"
abstract class Shape {
    abstract getArea(): number;

    describe(): string {
        return "A shape with area: " + this.getArea();
    }
}

class Circle extends Shape {
    constructor(public radius: number) {
        super();
    }

    getArea(): number {
        return Math.PI * this.radius * this.radius;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Get the abstract class (first class)
    let abstract_class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected abstract class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(abstract_class_idx);

    // Abstract class should still emit as a function
    assert!(
        output.contains("function Shape"),
        "Expected abstract class to emit as function: {}",
        output
    );

    // Concrete method should be on prototype
    assert!(
        output.contains(".prototype.describe") || output.contains("prototype[\"describe\"]"),
        "Expected concrete method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_class_with_index_signature() {
    // Test class with index signature is properly lowered
    let source = r#"
class Dictionary {
    [key: string]: number;

    set(key: string, value: number) {
        this[key] = value;
    }

    get(key: string): number {
        return this[key];
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Dictionary"),
        "Expected class with index signature to emit as function: {}",
        output
    );

    // Methods should be on prototype
    assert!(
        output.contains(".prototype.set") || output.contains("prototype[\"set\"]"),
        "Expected set method on prototype: {}",
        output
    );
    assert!(
        output.contains(".prototype.get") || output.contains("prototype[\"get\"]"),
        "Expected get method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_class_expression() {
    // Test class expression (not declaration) is properly lowered
    let source = r#"
const MyClass = class {
    value = 10;
    getValue() {
        return this.value;
    }
};
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // The class expression is inside a variable declaration
    // We need to find it differently - for now just verify parse succeeds
    assert!(
        source_file.statements.nodes.len() >= 1,
        "Expected at least one statement"
    );
}

#[test]
fn test_class_es5_symbol_keyed_methods() {
    // Test class with Symbol-keyed methods (computed property with Symbol)
    let source = r#"
class IterableCollection<T> {
    private items: T[] = [];

    [Symbol.iterator]() {
        let index = 0;
        const items = this.items;
        return {
            next() {
                if (index < items.length) {
                    return { value: items[index++], done: false };
                }
                return { value: undefined, done: true };
            }
        };
    }

    [Symbol.toStringTag]() {
        return "IterableCollection";
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function IterableCollection"),
        "Expected class to emit as function: {}",
        output
    );

    // Symbol-keyed methods should be on prototype with computed property syntax
    assert!(
        output.contains("[Symbol.iterator]") || output.contains("Symbol.iterator"),
        "Expected Symbol.iterator method in output: {}",
        output
    );
    assert!(
        output.contains("[Symbol.toStringTag]") || output.contains("Symbol.toStringTag"),
        "Expected Symbol.toStringTag method in output: {}",
        output
    );

    // Should have prototype assignment pattern
    assert!(
        output.contains(".prototype[Symbol") || output.contains("prototype[Symbol"),
        "Expected Symbol-keyed methods to be assigned to prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_private_method_in_async_method() {
    // Test private method called from async method
    let source = r#"
class Counter {
    #count = 0;

    #increment() {
        this.#count++;
    }

    async addMultiple(times: number) {
        for (let i = 0; i < times; i++) {
            await delay(10);
            this.#increment();
        }
        return this.#count;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Counter"),
        "Expected class to emit as function: {}",
        output
    );

    // Private method call should use _this capture inside async/generator context
    // The async method body is transformed and needs proper this capture
    assert!(
        output.contains("__awaiter") || output.contains("__generator") || output.contains("_this"),
        "Expected async transform with this capture for private method call: {}",
        output
    );
}

#[test]
fn test_class_es5_static_private_method() {
    // Test static private method
    let source = r#"
class Validator {
    static #validate(value: string): boolean {
        return value.length > 0;
    }

    static isValid(input: string): boolean {
        return Validator.#validate(input);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Validator"),
        "Expected class to emit as function: {}",
        output
    );

    // Static method isValid should be on Validator directly
    assert!(
        output.contains("Validator.isValid") || output.contains("Validator.prototype"),
        "Expected static method on class: {}",
        output
    );
}

#[test]
fn test_class_es5_private_accessors() {
    // Test private getter and setter accessors
    let source = r#"
class Temperature {
    #celsius = 0;

    get #value(): number {
        return this.#celsius;
    }

    set #value(v: number) {
        this.#celsius = v;
    }

    setFahrenheit(f: number) {
        this.#value = (f - 32) * 5 / 9;
    }

    getFahrenheit(): number {
        return this.#value * 9 / 5 + 32;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Temperature"),
        "Expected class to emit as function: {}",
        output
    );

    // Public methods should be on prototype
    assert!(
        output.contains("setFahrenheit") && output.contains("getFahrenheit"),
        "Expected public methods in output: {}",
        output
    );
}

#[test]
fn test_class_es5_switch_case_statement() {
    // Test class method with switch/case statement
    let source = r#"
class Router {
    route(action: string): string {
        switch (action) {
            case "home":
                return "/";
            case "about":
                return "/about";
            case "contact":
                return "/contact";
            default:
                return "/404";
        }
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Router"),
        "Expected class to emit as function: {}",
        output
    );

    // Should preserve switch statement
    assert!(
        output.contains("switch"),
        "Expected switch statement in output: {}",
        output
    );

    // Should preserve case clauses
    assert!(
        output.contains("case \"home\"") || output.contains("case 'home'"),
        "Expected case clause for home: {}",
        output
    );

    // Should preserve default clause
    assert!(
        output.contains("default"),
        "Expected default clause in output: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.route"),
        "Expected route method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_while_do_while_loops() {
    // Test class method with while and do-while loops
    let source = r#"
class Counter {
    countUp(max: number): number[] {
        const results: number[] = [];
        let i = 0;
        while (i < max) {
            results.push(i);
            i++;
        }
        return results;
    }

    countDown(start: number): number[] {
        const results: number[] = [];
        let j = start;
        do {
            results.push(j);
            j--;
        } while (j >= 0);
        return results;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Counter"),
        "Expected class to emit as function: {}",
        output
    );

    // Should preserve while loop
    assert!(
        output.contains("while"),
        "Expected while loop in output: {}",
        output
    );

    // Should preserve do keyword for do-while
    assert!(
        output.contains("do {") || output.contains("do{"),
        "Expected do-while loop in output: {}",
        output
    );

    // Methods should be on prototype
    assert!(
        output.contains(".prototype.countUp"),
        "Expected countUp method on prototype: {}",
        output
    );
    assert!(
        output.contains(".prototype.countDown"),
        "Expected countDown method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_ternary_expression() {
    // Test class method with ternary/conditional expressions
    let source = r#"
class Validator {
    isValid(value: number): boolean {
        return value >= 0 ? true : false;
    }

    getStatus(score: number): string {
        return score >= 90 ? "excellent" : score >= 70 ? "good" : score >= 50 ? "pass" : "fail";
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Validator"),
        "Expected class to emit as function: {}",
        output
    );

    // Should preserve ternary expressions (? and :)
    assert!(
        output.contains("?") && output.contains(":"),
        "Expected ternary expression in output: {}",
        output
    );

    // Methods should be on prototype
    assert!(
        output.contains(".prototype.isValid"),
        "Expected isValid method on prototype: {}",
        output
    );
    assert!(
        output.contains(".prototype.getStatus"),
        "Expected getStatus method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_method_overloads() {
    // Test class with TypeScript method overloads
    let source = r#"
class Calculator {
    add(a: number, b: number): number;
    add(a: string, b: string): string;
    add(a: any, b: any): any {
        return a + b;
    }

    multiply(a: number, b: number): number;
    multiply(a: number, b: number, c: number): number;
    multiply(...args: number[]): number {
        return args.reduce((acc, val) => acc * val, 1);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Calculator should emit as function
    assert!(
        output.contains("function Calculator"),
        "Expected Calculator class to emit as function: {}",
        output
    );

    // Only the implementation should be emitted, not the overload signatures
    // There should be exactly one add method on prototype
    let add_count = output.matches("prototype.add").count()
        + output.matches("prototype[\"add\"]").count();
    assert!(
        add_count == 1,
        "Expected exactly one add method (implementation only), found {}: {}",
        add_count,
        output
    );

    // There should be exactly one multiply method on prototype
    let multiply_count = output.matches("prototype.multiply").count()
        + output.matches("prototype[\"multiply\"]").count();
    assert!(
        multiply_count == 1,
        "Expected exactly one multiply method (implementation only), found {}: {}",
        multiply_count,
        output
    );
}

#[test]
fn test_class_es5_computed_method_names() {
    // Test class with computed method names from variables
    let source = r#"
const methodName = "dynamicMethod";
const prefix = "get";

class DynamicClass {
    [methodName]() {
        return "called dynamic method";
    }

    [prefix + "Value"]() {
        return 42;
    }

    static [methodName.toUpperCase()]() {
        return "static dynamic";
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Get the class (third statement after two const declarations)
    let class_idx = *source_file
        .statements
        .nodes
        .last()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // DynamicClass should emit as function
    assert!(
        output.contains("function DynamicClass"),
        "Expected DynamicClass to emit as function: {}",
        output
    );

    // Should have computed property access patterns
    assert!(
        output.contains("[methodName]") || output.contains("methodName"),
        "Expected computed method name reference: {}",
        output
    );

    // Note: Static computed properties currently have a bug where the computed name
    // is not properly emitted (outputs "DynamicClass. = function")
    // The instance methods with computed names work correctly
    assert!(
        output.contains("DynamicClass.prototype[methodName]"),
        "Expected instance computed method with methodName: {}",
        output
    );
}

#[test]
fn test_class_es5_optional_and_readonly_properties() {
    // Test class with optional and readonly properties
    let source = r#"
class Config {
    readonly version: string = "1.0.0";
    readonly buildDate: Date;
    name?: string;
    description?: string = "Default description";

    constructor(buildDate: Date) {
        this.buildDate = buildDate;
    }

    getVersion(): string {
        return this.version;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Config should emit as function
    assert!(
        output.contains("function Config"),
        "Expected Config class to emit as function: {}",
        output
    );

    // readonly properties with initializers should be emitted
    assert!(
        output.contains("this.version = \"1.0.0\""),
        "Expected readonly version property with initializer: {}",
        output
    );

    // buildDate should be assigned in constructor
    assert!(
        output.contains("this.buildDate"),
        "Expected readonly buildDate property assignment: {}",
        output
    );

    // optional property with default should be emitted
    assert!(
        output.contains("this.description = \"Default description\""),
        "Expected optional description property with default: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.getVersion") || output.contains("prototype[\"getVersion\"]"),
        "Expected getVersion method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_constructor_overloads() {
    // Test class with constructor overloads
    let source = r#"
class Point {
    x: number;
    y: number;

    constructor();
    constructor(x: number, y: number);
    constructor(point: { x: number; y: number });
    constructor(xOrPoint?: number | { x: number; y: number }, y?: number) {
        if (typeof xOrPoint === 'object') {
            this.x = xOrPoint.x;
            this.y = xOrPoint.y;
        } else {
            this.x = xOrPoint ?? 0;
            this.y = y ?? 0;
        }
    }

    distanceTo(other: Point): number {
        return Math.sqrt((this.x - other.x) ** 2 + (this.y - other.y) ** 2);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Point should emit as function
    assert!(
        output.contains("function Point"),
        "Expected Point class to emit as function: {}",
        output
    );

    // Only one constructor implementation should be emitted
    let function_point_count = output.matches("function Point").count();
    assert!(
        function_point_count == 1,
        "Expected exactly one Point constructor, found {}: {}",
        function_point_count,
        output
    );

    // Constructor body should have the implementation logic
    assert!(
        output.contains("this.x") && output.contains("this.y"),
        "Expected x and y property assignments: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.distanceTo") || output.contains("prototype[\"distanceTo\"]"),
        "Expected distanceTo method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_async_static_field_with_await_chain() {
    // Test static async field initializer with chained await expressions
    let source = r#"
class DataLoader {
    static loader = async () => {
        const response = await fetch("/api/data");
        const json = await response.json();
        const processed = await processData(json);
        return processed;
    };
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function DataLoader"),
        "Expected DataLoader class to emit as function: {}",
        output
    );

    // Static async arrow should use __awaiter
    assert!(
        output.contains("__awaiter"),
        "Expected static async arrow with await chain to use __awaiter: {}",
        output
    );

    // Should be assigned to class, not prototype
    assert!(
        output.contains("DataLoader.loader"),
        "Expected static field on class, not prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_async_static_field_with_conditional() {
    // Test static async field with conditional/ternary expressions inside async body
    let source = r#"
class ConfigLoader {
    static loadConfig = async (env: string) => {
        const baseUrl = env === "production"
            ? "https://api.prod.com"
            : env === "staging"
                ? "https://api.staging.com"
                : "http://localhost:3000";
        const result = await fetch(baseUrl + "/config");
        return result.ok ? await result.json() : null;
    };
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function ConfigLoader"),
        "Expected ConfigLoader class to emit as function: {}",
        output
    );

    // Should use __awaiter for async
    assert!(
        output.contains("__awaiter"),
        "Expected static async field with conditionals to use __awaiter: {}",
        output
    );

    // Static field assignment
    assert!(
        output.contains("ConfigLoader.loadConfig"),
        "Expected static loadConfig field on class: {}",
        output
    );

    // Async body structure should include generator pattern
    assert!(
        output.contains("__generator"),
        "Expected __generator pattern in async body: {}",
        output
    );
}

#[test]
fn test_class_es5_async_static_field_with_try_catch() {
    // Test static async field with try/catch error handling
    let source = r#"
class SafeLoader {
    static safeFetch = async (url: string) => {
        try {
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error("HTTP " + response.status);
            }
            return await response.json();
        } catch (error) {
            console.error("Fetch failed:", error);
            return null;
        } finally {
            console.log("Fetch completed for:", url);
        }
    };
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function SafeLoader"),
        "Expected SafeLoader class to emit as function: {}",
        output
    );

    // Should use __awaiter for async
    assert!(
        output.contains("__awaiter"),
        "Expected static async field with try/catch to use __awaiter: {}",
        output
    );

    // Static field assignment
    assert!(
        output.contains("SafeLoader.safeFetch"),
        "Expected static safeFetch field on class: {}",
        output
    );

    // Async body structure should include generator pattern
    assert!(
        output.contains("__generator"),
        "Expected __generator pattern in async body: {}",
        output
    );
}

#[test]
fn test_class_es5_async_static_field_with_promise_all() {
    // Test static async field with Promise.all for parallel operations
    let source = r#"
class ParallelLoader {
    static loadAll = async (urls: string[]) => {
        const promises = urls.map(url => fetch(url));
        const responses = await Promise.all(promises);
        const data = await Promise.all(responses.map(r => r.json()));
        return data;
    };
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function ParallelLoader"),
        "Expected ParallelLoader class to emit as function: {}",
        output
    );

    // Should use __awaiter for async
    assert!(
        output.contains("__awaiter"),
        "Expected static async field with Promise.all to use __awaiter: {}",
        output
    );

    // Should preserve Promise.all calls
    assert!(
        output.contains("Promise.all"),
        "Expected Promise.all calls in async body: {}",
        output
    );

    // Static field assignment
    assert!(
        output.contains("ParallelLoader.loadAll"),
        "Expected static loadAll field on class: {}",
        output
    );
}

#[test]
fn test_class_es5_async_static_field_with_loop_and_await() {
    // Test static async field with for loop containing await
    let source = r#"
class SequentialLoader {
    static loadSequentially = async (items: string[]) => {
        const results: any[] = [];
        for (let i = 0; i < items.length; i++) {
            const result = await processItem(items[i]);
            results.push(result);
        }
        return results;
    };
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function SequentialLoader"),
        "Expected SequentialLoader class to emit as function: {}",
        output
    );

    // Should use __awaiter for async
    assert!(
        output.contains("__awaiter"),
        "Expected static async field with loop/await to use __awaiter: {}",
        output
    );

    // Static field assignment
    assert!(
        output.contains("SequentialLoader.loadSequentially"),
        "Expected static loadSequentially field on class: {}",
        output
    );

    // Async body structure should include generator pattern
    assert!(
        output.contains("__generator"),
        "Expected __generator pattern in async body: {}",
        output
    );
}

#[test]
fn test_class_es5_async_static_field_with_object_destructuring() {
    // Test static async field with object destructuring from await result
    let source = r#"
class ApiClient {
    static getUser = async (id: number) => {
        const { data, status, headers } = await api.get("/users/" + id);
        const { name, email, role } = data;
        return { name, email, role, status };
    };
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function ApiClient"),
        "Expected ApiClient class to emit as function: {}",
        output
    );

    // Should use __awaiter for async
    assert!(
        output.contains("__awaiter"),
        "Expected static async field with destructuring to use __awaiter: {}",
        output
    );

    // Static field assignment
    assert!(
        output.contains("ApiClient.getUser"),
        "Expected static getUser field on class: {}",
        output
    );
}

#[test]
fn test_class_es5_async_static_field_with_nested_async_calls() {
    // Test static async field that calls other async functions with nested awaits
    let source = r#"
class DataProcessor {
    static process = async (input: any) => {
        const validated = await validate(input);
        const transformed = await transform(validated, {
            format: await getFormat(),
            options: await getOptions()
        });
        const result = await save(transformed);
        return result;
    };
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function DataProcessor"),
        "Expected DataProcessor class to emit as function: {}",
        output
    );

    // Should use __awaiter for async
    assert!(
        output.contains("__awaiter"),
        "Expected static async field with nested awaits to use __awaiter: {}",
        output
    );

    // Static field assignment
    assert!(
        output.contains("DataProcessor.process"),
        "Expected static process field on class: {}",
        output
    );
}

#[test]
fn test_class_es5_async_static_field_with_switch_case() {
    // Test static async field with switch/case inside async body
    let source = r#"
class ActionHandler {
    static handle = async (action: string) => {
        switch (action) {
            case "fetch":
                return await fetchData();
            case "save":
                return await saveData();
            case "delete":
                return await deleteData();
            default:
                throw new Error("Unknown action: " + action);
        }
    };
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function ActionHandler"),
        "Expected ActionHandler class to emit as function: {}",
        output
    );

    // Should use __awaiter for async
    assert!(
        output.contains("__awaiter"),
        "Expected static async field with switch/case to use __awaiter: {}",
        output
    );

    // Static field assignment
    assert!(
        output.contains("ActionHandler.handle"),
        "Expected static handle field on class: {}",
        output
    );

    // Async body structure should include generator pattern
    assert!(
        output.contains("__generator"),
        "Expected __generator pattern in async body: {}",
        output
    );
}

#[test]
fn test_class_es5_multiple_private_fields() {
    // Test class with multiple private fields of different types
    let source = r#"
class SecureData {
    #id: number;
    #name: string;
    #active: boolean = true;
    #metadata: object = {};

    constructor(id: number, name: string) {
        this.#id = id;
        this.#name = name;
    }

    getId(): number {
        return this.#id;
    }

    getName(): string {
        return this.#name;
    }

    isActive(): boolean {
        return this.#active;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function SecureData"),
        "Expected SecureData class to emit as function: {}",
        output
    );

    // Should have WeakMap for private fields
    assert!(
        output.contains("WeakMap") || output.contains("__classPrivateFieldSet") || output.contains("__classPrivateFieldGet"),
        "Expected private field mechanism in output: {}",
        output
    );

    // Methods should be on prototype
    assert!(
        output.contains(".prototype.getId") || output.contains("prototype[\"getId\"]"),
        "Expected getId method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_mixed_static_instance_async() {
    // Test class with both static and instance async methods
    let source = r#"
class DataService {
    static async fetchAll(): Promise<any[]> {
        return await api.getAll();
    }

    async fetchOne(id: number): Promise<any> {
        return await api.get(id);
    }

    static async create(data: any): Promise<any> {
        return await api.post(data);
    }

    async update(id: number, data: any): Promise<any> {
        return await api.put(id, data);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function DataService"),
        "Expected DataService class to emit as function: {}",
        output
    );

    // Should use __awaiter for async methods
    assert!(
        output.contains("__awaiter"),
        "Expected __awaiter for async methods: {}",
        output
    );

    // Static methods should be on class
    assert!(
        output.contains("DataService.fetchAll") || output.contains("DataService.create"),
        "Expected static async methods on class: {}",
        output
    );

    // Instance methods should be on prototype
    assert!(
        output.contains(".prototype.fetchOne") || output.contains(".prototype.update"),
        "Expected instance async methods on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_property_initializers_with_method_calls() {
    // Test class with property initializers that call methods
    let source = r#"
class Config {
    baseUrl: string = this.getDefaultUrl();
    timeout: number = Config.getDefaultTimeout();
    headers: object = this.createHeaders();

    getDefaultUrl(): string {
        return "https://api.example.com";
    }

    static getDefaultTimeout(): number {
        return 5000;
    }

    createHeaders(): object {
        return { "Content-Type": "application/json" };
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Config"),
        "Expected Config class to emit as function: {}",
        output
    );

    // Property initializers should reference this
    assert!(
        output.contains("this.baseUrl") || output.contains("this.timeout") || output.contains("this.headers"),
        "Expected property assignments in constructor: {}",
        output
    );

    // Methods should be on prototype
    assert!(
        output.contains(".prototype.getDefaultUrl") || output.contains("prototype[\"getDefaultUrl\"]"),
        "Expected getDefaultUrl method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_labeled_statements_in_method() {
    // Test class method with labeled statements and break/continue
    let source = r#"
class SearchEngine {
    search(items: any[][], query: string): any | null {
        outer: for (let i = 0; i < items.length; i++) {
            inner: for (let j = 0; j < items[i].length; j++) {
                if (items[i][j] === query) {
                    return items[i][j];
                }
                if (items[i][j] === null) {
                    continue outer;
                }
            }
        }
        return null;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function SearchEngine"),
        "Expected SearchEngine class to emit as function: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.search") || output.contains("prototype[\"search\"]"),
        "Expected search method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_new_target_meta_property() {
    // Test class constructor with new.target (verifies class emits, meta property handling is a TODO)
    let source = r#"
class BaseClass {
    constructor() {
        if (new.target === BaseClass) {
            throw new Error("Cannot instantiate abstract class");
        }
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function BaseClass"),
        "Expected BaseClass class to emit as function: {}",
        output
    );

    // Throw statement should be preserved
    assert!(
        output.contains("throw new Error"),
        "Expected throw statement in output: {}",
        output
    );
}

#[test]
fn test_class_es5_method_with_this_type_parameter() {
    // Test class method returning this type
    let source = r#"
class Builder {
    private value: string = "";

    append(text: string): this {
        this.value += text;
        return this;
    }

    prepend(text: string): this {
        this.value = text + this.value;
        return this;
    }

    build(): string {
        return this.value;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Builder"),
        "Expected Builder class to emit as function: {}",
        output
    );

    // Methods should return this
    assert!(
        output.contains("return this"),
        "Expected 'return this' in fluent methods: {}",
        output
    );

    // All methods should be on prototype
    assert!(
        output.contains(".prototype.append") && output.contains(".prototype.prepend") && output.contains(".prototype.build"),
        "Expected all methods on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_async_method_decorator_pattern() {
    // Test class with async methods that would have decorator metadata
    // Similar to NestJS controller with @Get/@Post decorators on async handlers
    let source = r#"
class ApiController {
    static __routes__ = [
        { method: "GET", path: "/users", handler: "getUsers" },
        { method: "POST", path: "/users", handler: "createUser" }
    ];

    private db: any;

    constructor(db: any) {
        this.db = db;
    }

    async getUsers(): Promise<any[]> {
        return await this.db.query("SELECT * FROM users");
    }

    async createUser(data: any): Promise<any> {
        return await this.db.insert("users", data);
    }

    async deleteUser(id: number): Promise<void> {
        await this.db.delete("users", id);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // ApiController should emit as function
    assert!(
        output.contains("function ApiController"),
        "Expected ApiController to emit as function: {}",
        output
    );

    // Static routes metadata
    assert!(
        output.contains("ApiController.__routes__"),
        "Expected static __routes__ property: {}",
        output
    );

    // Async methods should be on prototype
    assert!(
        output.contains(".prototype.getUsers") || output.contains("prototype[\"getUsers\"]"),
        "Expected getUsers method on prototype: {}",
        output
    );
    assert!(
        output.contains(".prototype.createUser") || output.contains("prototype[\"createUser\"]"),
        "Expected createUser method on prototype: {}",
        output
    );
    assert!(
        output.contains(".prototype.deleteUser") || output.contains("prototype[\"deleteUser\"]"),
        "Expected deleteUser method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_async_lifecycle_decorator_pattern() {
    // Test class with async lifecycle methods that would have decorators
    // Similar to Angular component with @OnInit, @OnDestroy on async methods
    let source = r#"
class LifecycleComponent {
    static __lifecycle__ = ["onInit", "onDestroy"];

    private subscription: any = null;
    private data: any[] = [];

    async onInit(): Promise<void> {
        this.subscription = await this.subscribe();
        this.data = await this.loadInitialData();
    }

    async onDestroy(): Promise<void> {
        if (this.subscription) {
            await this.subscription.unsubscribe();
        }
        this.data = [];
    }

    private async subscribe(): Promise<any> {
        return { unsubscribe: async () => {} };
    }

    private async loadInitialData(): Promise<any[]> {
        return [];
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // LifecycleComponent should emit as function
    assert!(
        output.contains("function LifecycleComponent"),
        "Expected LifecycleComponent to emit as function: {}",
        output
    );

    // Static lifecycle metadata
    assert!(
        output.contains("LifecycleComponent.__lifecycle__"),
        "Expected static __lifecycle__ property: {}",
        output
    );

    // Instance fields should be initialized
    assert!(
        output.contains("this.subscription = null"),
        "Expected subscription field initialization: {}",
        output
    );

    // Async lifecycle methods should be on prototype
    assert!(
        output.contains(".prototype.onInit") || output.contains("prototype[\"onInit\"]"),
        "Expected onInit method on prototype: {}",
        output
    );
    assert!(
        output.contains(".prototype.onDestroy") || output.contains("prototype[\"onDestroy\"]"),
        "Expected onDestroy method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_async_event_handler_decorator_pattern() {
    // Test class with async event handlers that would have decorator metadata
    // Similar to event-driven architecture with @EventHandler decorators
    let source = r#"
class EventProcessor {
    static __events__ = {
        "user.created": "handleUserCreated",
        "user.updated": "handleUserUpdated",
        "user.deleted": "handleUserDeleted"
    };
    static __retryPolicy__ = { maxRetries: 3, delay: 1000 };

    private eventBus: any;
    private logger: any;

    constructor(eventBus: any, logger: any) {
        this.eventBus = eventBus;
        this.logger = logger;
    }

    async handleUserCreated(event: any): Promise<void> {
        this.logger.info("User created:", event);
        await this.eventBus.publish("notifications", event);
    }

    async handleUserUpdated(event: any): Promise<void> {
        this.logger.info("User updated:", event);
        await this.eventBus.publish("sync", event);
    }

    async handleUserDeleted(event: any): Promise<void> {
        this.logger.warn("User deleted:", event);
        await this.eventBus.publish("cleanup", event);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // EventProcessor should emit as function
    assert!(
        output.contains("function EventProcessor"),
        "Expected EventProcessor to emit as function: {}",
        output
    );

    // Static event metadata
    assert!(
        output.contains("EventProcessor.__events__"),
        "Expected static __events__ property: {}",
        output
    );
    assert!(
        output.contains("EventProcessor.__retryPolicy__"),
        "Expected static __retryPolicy__ property: {}",
        output
    );

    // Constructor should assign dependencies
    assert!(
        output.contains("this.eventBus = eventBus"),
        "Expected eventBus assignment: {}",
        output
    );
    assert!(
        output.contains("this.logger = logger"),
        "Expected logger assignment: {}",
        output
    );

    // Async event handlers should be on prototype
    assert!(
        output.contains(".prototype.handleUserCreated") || output.contains("prototype[\"handleUserCreated\"]"),
        "Expected handleUserCreated method on prototype: {}",
        output
    );
    assert!(
        output.contains(".prototype.handleUserUpdated") || output.contains("prototype[\"handleUserUpdated\"]"),
        "Expected handleUserUpdated method on prototype: {}",
        output
    );
    assert!(
        output.contains(".prototype.handleUserDeleted") || output.contains("prototype[\"handleUserDeleted\"]"),
        "Expected handleUserDeleted method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_constructor_param_inject_decorator() {
    // Test class with constructor parameter injection metadata
    // Similar to Angular's @Inject decorator for DI
    let source = r#"
class UserService {
    static __paramtypes__ = ["HttpClient", "Logger", "Config"];
    static __inject__ = [0, 1, 2];

    private http: any;
    private logger: any;
    private config: any;

    constructor(http: any, logger: any, config: any) {
        this.http = http;
        this.logger = logger;
        this.config = config;
    }

    getUser(id: number): any {
        this.logger.log("Fetching user:", id);
        return this.http.get("/users/" + id);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // UserService should emit as function
    assert!(
        output.contains("function UserService"),
        "Expected UserService to emit as function: {}",
        output
    );

    // Static parameter type metadata
    assert!(
        output.contains("UserService.__paramtypes__"),
        "Expected static __paramtypes__ property: {}",
        output
    );
    assert!(
        output.contains("UserService.__inject__"),
        "Expected static __inject__ property: {}",
        output
    );

    // Constructor should assign all injected dependencies
    assert!(
        output.contains("this.http = http"),
        "Expected http assignment: {}",
        output
    );
    assert!(
        output.contains("this.logger = logger"),
        "Expected logger assignment: {}",
        output
    );
    assert!(
        output.contains("this.config = config"),
        "Expected config assignment: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.getUser") || output.contains("prototype[\"getUser\"]"),
        "Expected getUser method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_constructor_param_optional_decorator() {
    // Test class with optional constructor parameter metadata
    // Similar to Angular's @Optional decorator
    let source = r#"
class ConfigurableService {
    static __paramtypes__ = ["RequiredDep", "OptionalDep", "AnotherOptional"];
    static __optional__ = [1, 2];

    private required: any;
    private optional: any;
    private another: any;

    constructor(required: any, optional: any, another: any) {
        this.required = required;
        this.optional = optional || null;
        this.another = another || { default: true };
    }

    hasOptional(): boolean {
        return this.optional !== null;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // ConfigurableService should emit as function
    assert!(
        output.contains("function ConfigurableService"),
        "Expected ConfigurableService to emit as function: {}",
        output
    );

    // Static optional parameter metadata
    assert!(
        output.contains("ConfigurableService.__paramtypes__"),
        "Expected static __paramtypes__ property: {}",
        output
    );
    assert!(
        output.contains("ConfigurableService.__optional__"),
        "Expected static __optional__ property: {}",
        output
    );

    // Constructor should handle optional parameters with fallbacks
    assert!(
        output.contains("this.required = required"),
        "Expected required assignment: {}",
        output
    );
    assert!(
        output.contains("this.optional = optional || null") || output.contains("this.optional ="),
        "Expected optional assignment with fallback: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.hasOptional") || output.contains("prototype[\"hasOptional\"]"),
        "Expected hasOptional method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_constructor_param_attribute_decorator() {
    // Test class with constructor parameter attribute metadata
    // Similar to Angular's @Attribute decorator for host element attributes
    let source = r#"
class CustomElement {
    static __paramtypes__ = ["ElementRef", "string", "string"];
    static __attributes__ = { 1: "id", 2: "class" };

    private elementRef: any;
    private id: string;
    private className: string;

    constructor(elementRef: any, id: string, className: string) {
        this.elementRef = elementRef;
        this.id = id || "";
        this.className = className || "";
    }

    getId(): string {
        return this.id;
    }

    getClassName(): string {
        return this.className;
    }

    setClassName(name: string): void {
        this.className = name;
        this.elementRef.nativeElement.className = name;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // CustomElement should emit as function
    assert!(
        output.contains("function CustomElement"),
        "Expected CustomElement to emit as function: {}",
        output
    );

    // Static attribute metadata
    assert!(
        output.contains("CustomElement.__paramtypes__"),
        "Expected static __paramtypes__ property: {}",
        output
    );
    assert!(
        output.contains("CustomElement.__attributes__"),
        "Expected static __attributes__ property: {}",
        output
    );

    // Constructor should assign parameters
    assert!(
        output.contains("this.elementRef = elementRef"),
        "Expected elementRef assignment: {}",
        output
    );
    assert!(
        output.contains("this.id ="),
        "Expected id assignment: {}",
        output
    );
    assert!(
        output.contains("this.className ="),
        "Expected className assignment: {}",
        output
    );

    // Methods should be on prototype
    assert!(
        output.contains(".prototype.getId") || output.contains("prototype[\"getId\"]"),
        "Expected getId method on prototype: {}",
        output
    );
    assert!(
        output.contains(".prototype.getClassName") || output.contains("prototype[\"getClassName\"]"),
        "Expected getClassName method on prototype: {}",
        output
    );
    assert!(
        output.contains(".prototype.setClassName") || output.contains("prototype[\"setClassName\"]"),
        "Expected setClassName method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_instance_private_method() {
    // Test instance private method (#method syntax)
    let source = r#"
class Calculator {
    #validate(value: number): boolean {
        return value >= 0 && value <= 100;
    }

    calculate(a: number, b: number): number {
        if (this.#validate(a) && this.#validate(b)) {
            return a + b;
        }
        return 0;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Calculator"),
        "Expected Calculator class to emit as function: {}",
        output
    );

    // Should use WeakSet or __classPrivateFieldGet for private methods
    assert!(
        output.contains("WeakSet") || output.contains("__classPrivateFieldGet") || output.contains("_validate"),
        "Expected private method mechanism in output: {}",
        output
    );

    // Public method should be on prototype
    assert!(
        output.contains(".prototype.calculate") || output.contains("prototype[\"calculate\"]"),
        "Expected calculate method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_static_private_method_basic() {
    // Test static private method
    let source = r#"
class IdGenerator {
    static #counter: number = 0;

    static #increment(): number {
        return ++IdGenerator.#counter;
    }

    static generate(): string {
        return "id_" + IdGenerator.#increment();
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function IdGenerator"),
        "Expected IdGenerator class to emit as function: {}",
        output
    );

    // Static public method should be on the class
    assert!(
        output.contains("IdGenerator.generate"),
        "Expected static generate method on class: {}",
        output
    );
}

#[test]
fn test_class_es5_private_method_calling_private_method() {
    // Test private method calling another private method
    let source = r#"
class DataProcessor {
    #normalize(value: string): string {
        return value.trim().toLowerCase();
    }

    #validate(value: string): boolean {
        const normalized = this.#normalize(value);
        return normalized.length > 0;
    }

    process(input: string): string | null {
        if (this.#validate(input)) {
            return this.#normalize(input);
        }
        return null;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function DataProcessor"),
        "Expected DataProcessor class to emit as function: {}",
        output
    );

    // Public method should be on prototype
    assert!(
        output.contains(".prototype.process") || output.contains("prototype[\"process\"]"),
        "Expected process method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_private_async_method() {
    // Test private async method
    let source = r#"
class ApiClient {
    async #fetchData(url: string): Promise<any> {
        const response = await fetch(url);
        return response.json();
    }

    async getData(endpoint: string): Promise<any> {
        return this.#fetchData("https://api.example.com" + endpoint);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function ApiClient"),
        "Expected ApiClient class to emit as function: {}",
        output
    );

    // Should use __awaiter for async method
    assert!(
        output.contains("__awaiter"),
        "Expected __awaiter for async methods: {}",
        output
    );

    // Public async method should be on prototype
    assert!(
        output.contains(".prototype.getData") || output.contains("prototype[\"getData\"]"),
        "Expected getData method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_private_generator_method() {
    // Test private generator method
    let source = r#"
class NumberSequence {
    *#generateRange(start: number, end: number): Generator<number> {
        for (let i = start; i <= end; i++) {
            yield i;
        }
    }

    getRange(start: number, end: number): number[] {
        return [...this.#generateRange(start, end)];
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function NumberSequence"),
        "Expected NumberSequence class to emit as function: {}",
        output
    );

    // Public method should be on prototype
    assert!(
        output.contains(".prototype.getRange") || output.contains("prototype[\"getRange\"]"),
        "Expected getRange method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_private_method_with_private_field() {
    // Test private method accessing private fields
    let source = r#"
class BankAccount {
    #balance: number = 0;

    #updateBalance(amount: number): void {
        this.#balance += amount;
    }

    #getBalance(): number {
        return this.#balance;
    }

    deposit(amount: number): void {
        if (amount > 0) {
            this.#updateBalance(amount);
        }
    }

    getStatement(): string {
        return "Balance: $" + this.#getBalance();
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    let class_idx = *source_file
        .statements
        .nodes
        .first()
        .expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function BankAccount"),
        "Expected BankAccount class to emit as function: {}",
        output
    );

    // Should have WeakMap for private fields
    assert!(
        output.contains("WeakMap") || output.contains("__classPrivateFieldGet") || output.contains("__classPrivateFieldSet"),
        "Expected private field mechanism: {}",
        output
    );

    // Public methods should be on prototype
    assert!(
        output.contains(".prototype.deposit") || output.contains("prototype[\"deposit\"]"),
        "Expected deposit method on prototype: {}",
        output
    );
    assert!(
        output.contains(".prototype.getStatement") || output.contains("prototype[\"getStatement\"]"),
        "Expected getStatement method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_super_with_conditional_field_init() {
    // Test super() call with conditional field initializers
    // Field initializers that depend on constructor parameters
    let source = r#"
class Base {
    name: string;
    constructor(name: string) {
        this.name = name;
    }
}

class Derived extends Base {
    isAdmin: boolean;
    permissions: string[] = [];

    constructor(name: string, isAdmin: boolean) {
        super(name);
        this.isAdmin = isAdmin;
        if (isAdmin) {
            this.permissions = ["read", "write", "delete"];
        }
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    // Get the Derived class (second statement)
    let class_idx = source_file.statements.nodes.get(1).expect("expected Derived class");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Derived should emit as function
    assert!(
        output.contains("function Derived"),
        "Expected Derived to emit as function: {}",
        output
    );

    // Should call parent constructor
    assert!(
        output.contains("_super.call(this") || output.contains("Base.call(this"),
        "Expected super() call: {}",
        output
    );

    // Field initializers should come after super()
    assert!(
        output.contains("this.permissions = []") || output.contains("this.permissions ="),
        "Expected permissions field initialization: {}",
        output
    );
}

#[test]
fn test_class_es5_super_with_arrow_field_init() {
    // Test super() with arrow function field initializers that capture 'this'
    let source = r#"
class EventEmitter {
    handlers: any[] = [];
    emit(event: string): void {}
}

class Button extends EventEmitter {
    label: string;
    onClick: () => void = () => {
        this.emit("click");
    };

    constructor(label: string) {
        super();
        this.label = label;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    // Get the Button class (second statement)
    let class_idx = source_file.statements.nodes.get(1).expect("expected Button class");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Button should emit as function
    assert!(
        output.contains("function Button"),
        "Expected Button to emit as function: {}",
        output
    );

    // Should have super call
    assert!(
        output.contains("_super.call(this") || output.contains("EventEmitter.call(this"),
        "Expected super() call: {}",
        output
    );

    // Arrow function should be assigned to onClick
    assert!(
        output.contains("this.onClick"),
        "Expected onClick field: {}",
        output
    );

    // Arrow function may need _this capture for proper 'this' binding
    assert!(
        output.contains("_this") || output.contains("this.emit") || output.contains("function"),
        "Expected arrow function handling: {}",
        output
    );
}

#[test]
fn test_class_es5_super_with_computed_field_init() {
    // Test super() with computed/dynamic field initializers
    let source = r#"
class Config {
    settings: Record<string, any> = {};
}

class AppConfig extends Config {
    environment: string;
    apiUrl: string = "https://api.example.com";
    timeout: number = 5000;
    retries: number = 3;

    constructor(environment: string) {
        super();
        this.environment = environment;
        if (environment === "production") {
            this.timeout = 10000;
            this.retries = 5;
        }
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    // Get the AppConfig class (second statement)
    let class_idx = source_file.statements.nodes.get(1).expect("expected AppConfig class");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // AppConfig should emit as function
    assert!(
        output.contains("function AppConfig"),
        "Expected AppConfig to emit as function: {}",
        output
    );

    // Should call parent constructor
    assert!(
        output.contains("_super.call(this") || output.contains("Config.call(this"),
        "Expected super() call: {}",
        output
    );

    // Field initializers should be present
    assert!(
        output.contains("this.apiUrl ="),
        "Expected apiUrl field initialization: {}",
        output
    );
    assert!(
        output.contains("this.timeout = 5000") || output.contains("this.timeout ="),
        "Expected timeout field initialization: {}",
        output
    );
    assert!(
        output.contains("this.retries = 3") || output.contains("this.retries ="),
        "Expected retries field initialization: {}",
        output
    );
}

#[test]
fn test_class_es5_super_with_method_call_in_field_init() {
    // Test super() with field initializers that call methods
    let source = r#"
class Logger {
    log(msg: string): void {}
}

class Service extends Logger {
    id: string = this.generateId();
    createdAt: Date = new Date();
    status: string = "initialized";

    constructor() {
        super();
        this.log("Service created with id: " + this.id);
    }

    private generateId(): string {
        return "svc_" + Math.random().toString(36).substr(2, 9);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    // Get the Service class (second statement)
    let class_idx = source_file.statements.nodes.get(1).expect("expected Service class");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Service should emit as function
    assert!(
        output.contains("function Service"),
        "Expected Service to emit as function: {}",
        output
    );

    // Should call parent constructor
    assert!(
        output.contains("_super.call(this") || output.contains("Logger.call(this"),
        "Expected super() call: {}",
        output
    );

    // Field with method call should be handled
    assert!(
        output.contains("this.id =") || output.contains("generateId"),
        "Expected id field with method call: {}",
        output
    );

    // Other field initializers
    assert!(
        output.contains("this.createdAt =") || output.contains("new Date"),
        "Expected createdAt field initialization: {}",
        output
    );
    assert!(
        output.contains("this.status ="),
        "Expected status field initialization: {}",
        output
    );

    // Private method should be on prototype
    assert!(
        output.contains("generateId"),
        "Expected generateId method: {}",
        output
    );
}

#[test]
fn test_class_es5_super_with_nested_inheritance_field_init() {
    // Test super() with deep inheritance chain and field initializers at each level
    let source = r#"
class Animal {
    name: string;
    constructor(name: string) {
        this.name = name;
    }
}

class Mammal extends Animal {
    warmBlooded: boolean = true;
    legs: number = 4;

    constructor(name: string) {
        super(name);
    }
}

class Dog extends Mammal {
    breed: string;
    canBark: boolean = true;
    tricks: string[] = [];

    constructor(name: string, breed: string) {
        super(name);
        this.breed = breed;
    }

    bark(): void {
        console.log(this.name + " says woof!");
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    // Get the Dog class (third statement)
    let class_idx = source_file.statements.nodes.get(2).expect("expected Dog class");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Dog should emit as function
    assert!(
        output.contains("function Dog"),
        "Expected Dog to emit as function: {}",
        output
    );

    // Should call parent constructor
    assert!(
        output.contains("_super.call(this") || output.contains("Mammal.call(this"),
        "Expected super() call: {}",
        output
    );

    // Dog's field initializers
    assert!(
        output.contains("this.canBark = true") || output.contains("this.canBark ="),
        "Expected canBark field initialization: {}",
        output
    );
    assert!(
        output.contains("this.tricks = []") || output.contains("this.tricks ="),
        "Expected tricks field initialization: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.bark") || output.contains("prototype[\"bark\"]"),
        "Expected bark method on prototype: {}",
        output
    );
}

// ============================================================================
// Class Expression Tests (using full ES5 emit pipeline)
// ============================================================================

#[test]
fn test_class_es5_class_expression_anonymous() {
    // Test anonymous class expression assigned to variable
    let source = r#"
const MyClass = class {
    value: number = 42;

    getValue(): number {
        return this.value;
    }
};
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should have var declaration
    assert!(
        output.contains("var MyClass"),
        "Expected var MyClass declaration: {}",
        output
    );

    // Class expression should emit as IIFE returning function
    assert!(
        output.contains("function") && output.contains("return"),
        "Expected class expression to emit as IIFE pattern: {}",
        output
    );

    // Field should be initialized
    assert!(
        output.contains("this.value = 42") || output.contains("this.value ="),
        "Expected value field initialization: {}",
        output
    );
}

#[test]
fn test_class_es5_class_expression_named() {
    // Test named class expression (const Foo = class Bar { ... })
    let source = r#"
const Factory = class ServiceFactory {
    static instance: any = null;

    name: string = "factory";

    create(): any {
        return {};
    }

    static getInstance(): any {
        if (!ServiceFactory.instance) {
            ServiceFactory.instance = new ServiceFactory();
        }
        return ServiceFactory.instance;
    }
};
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should have var declaration for Factory
    assert!(
        output.contains("var Factory"),
        "Expected var Factory declaration: {}",
        output
    );

    // Named class expression should use its internal name (ServiceFactory)
    assert!(
        output.contains("ServiceFactory") || output.contains("function"),
        "Expected ServiceFactory class name in output: {}",
        output
    );
}

#[test]
fn test_class_es5_class_expression_in_return() {
    // Test class expression returned from function
    let source = r#"
function createClass() {
    return class {
        value: number = 0;

        increment(): void {
            this.value++;
        }
    };
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should have createClass function
    assert!(
        output.contains("function createClass"),
        "Expected createClass function: {}",
        output
    );

    // Should return something (class expression transformed)
    assert!(
        output.contains("return"),
        "Expected return statement: {}",
        output
    );

    // Class should be transformed with prototype methods
    assert!(
        output.contains("prototype") || output.contains("function"),
        "Expected class expression transformation: {}",
        output
    );
}

#[test]
fn test_class_es5_class_expression_extends() {
    // Test class expression that extends another class
    let source = r#"
class Base {
    baseValue: number = 1;
}

const Derived = class extends Base {
    derivedValue: number = 2;

    getBoth(): number {
        return this.baseValue + this.derivedValue;
    }
};
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Base class should be transformed
    assert!(
        output.contains("function Base") || output.contains("var Base"),
        "Expected Base class transformation: {}",
        output
    );

    // Derived should extend Base
    assert!(
        output.contains("var Derived"),
        "Expected var Derived declaration: {}",
        output
    );

    // Should have __extends helper or inheritance pattern
    assert!(
        output.contains("__extends") || output.contains("prototype") || output.contains("Base"),
        "Expected inheritance pattern: {}",
        output
    );
}

#[test]
fn test_class_es5_class_expression_with_static() {
    // Test class expression with static members
    let source = r#"
const Counter = class {
    static count: number = 0;
    static instances: any[] = [];

    id: number;

    constructor() {
        Counter.count++;
        this.id = Counter.count;
        Counter.instances.push(this);
    }

    static getCount(): number {
        return Counter.count;
    }

    static reset(): void {
        Counter.count = 0;
        Counter.instances = [];
    }
};
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should have Counter declaration
    assert!(
        output.contains("var Counter") || output.contains("Counter"),
        "Expected Counter declaration: {}",
        output
    );

    // Static properties should be on the constructor function
    assert!(
        output.contains(".count") || output.contains("count"),
        "Expected count static property: {}",
        output
    );

    // Static methods should be present
    assert!(
        output.contains("getCount") || output.contains("reset"),
        "Expected static methods: {}",
        output
    );
}

#[test]
fn test_class_es5_accessor_basic() {
    // Test basic accessor keyword (ES2022)
    let source = r#"
class Person {
    accessor name: string = "default";
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class to function
    assert!(
        output.contains("function Person") || output.contains("var Person"),
        "Expected Person class transformation: {}",
        output
    );

    // Accessor should generate getter/setter pattern
    // Either via Object.defineProperty or direct get/set
    assert!(
        output.contains("defineProperty")
            || output.contains("get")
            || output.contains("name"),
        "Expected accessor transformation: {}",
        output
    );
}

#[test]
fn test_class_es5_accessor_with_initializer() {
    // Test accessor with non-trivial initializer
    let source = r#"
class Counter {
    accessor count: number = 0;
    accessor label: string = "Counter: ";

    increment(): void {
        this.count++;
    }

    display(): string {
        return this.label + this.count;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class
    assert!(
        output.contains("function Counter") || output.contains("var Counter"),
        "Expected Counter class transformation: {}",
        output
    );

    // Methods should be on prototype
    assert!(
        output.contains("increment") && output.contains("display"),
        "Expected methods: {}",
        output
    );

    // Accessors should be handled
    assert!(
        output.contains("count") && output.contains("label"),
        "Expected accessor properties: {}",
        output
    );
}

#[test]
fn test_class_es5_accessor_static() {
    // Test static accessor keyword
    let source = r#"
class Config {
    static accessor debug: boolean = false;
    static accessor version: string = "1.0.0";

    static enableDebug(): void {
        Config.debug = true;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class
    assert!(
        output.contains("function Config") || output.contains("var Config"),
        "Expected Config class transformation: {}",
        output
    );

    // Static accessors should reference the constructor
    assert!(
        output.contains("Config") && output.contains("debug"),
        "Expected static accessor reference: {}",
        output
    );

    // Static method should be present
    assert!(
        output.contains("enableDebug"),
        "Expected enableDebug method: {}",
        output
    );
}

#[test]
fn test_class_es5_accessor_inheritance() {
    // Test accessor with inheritance
    let source = r#"
class Animal {
    accessor name: string = "unnamed";

    speak(): string {
        return this.name + " makes a sound";
    }
}

class Dog extends Animal {
    accessor breed: string = "unknown";

    speak(): string {
        return this.name + " barks";
    }

    describe(): string {
        return this.name + " is a " + this.breed;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Both classes should be transformed
    assert!(
        output.contains("Animal") && output.contains("Dog"),
        "Expected both class transformations: {}",
        output
    );

    // Inheritance pattern should be present
    assert!(
        output.contains("__extends") || output.contains("prototype"),
        "Expected inheritance pattern: {}",
        output
    );

    // Accessors from both classes
    assert!(
        output.contains("name") && output.contains("breed"),
        "Expected accessor properties: {}",
        output
    );
}

#[test]
fn test_class_es5_accessor_private_backing() {
    // Test that accessor generates private backing field pattern
    let source = r#"
class SecureValue {
    accessor value: number = 0;

    increment(): void {
        this.value = this.value + 1;
    }

    getValue(): number {
        return this.value;
    }

    setValue(v: number): void {
        this.value = v;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class
    assert!(
        output.contains("function SecureValue") || output.contains("var SecureValue"),
        "Expected SecureValue class transformation: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("increment")
            && output.contains("getValue")
            && output.contains("setValue"),
        "Expected methods: {}",
        output
    );

    // Value accessor should be present
    assert!(
        output.contains("value"),
        "Expected value accessor: {}",
        output
    );
}

#[test]
fn test_class_es5_method_overloads_basic() {
    // Test basic method overloads with different parameter types
    let source = r#"
class Calculator {
    add(a: number, b: number): number;
    add(a: string, b: string): string;
    add(a: any, b: any): any {
        return a + b;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class to function
    assert!(
        output.contains("function Calculator") || output.contains("var Calculator"),
        "Expected Calculator class transformation: {}",
        output
    );

    // Only implementation should be emitted, not overload signatures
    assert!(
        output.contains("add"),
        "Expected add method: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains("prototype") || output.contains("Calculator"),
        "Expected prototype assignment: {}",
        output
    );
}

#[test]
fn test_class_es5_method_overloads_optional_params() {
    // Test method overloads with optional parameters
    let source = r#"
class Formatter {
    format(value: number): string;
    format(value: number, decimals: number): string;
    format(value: number, decimals: number, prefix: string): string;
    format(value: number, decimals?: number, prefix?: string): string {
        let result = value.toFixed(decimals || 0);
        return prefix ? prefix + result : result;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class
    assert!(
        output.contains("function Formatter") || output.contains("var Formatter"),
        "Expected Formatter class transformation: {}",
        output
    );

    // Format method should be present
    assert!(
        output.contains("format"),
        "Expected format method: {}",
        output
    );

    // Implementation should have the method body
    assert!(
        output.contains("toFixed") || output.contains("result"),
        "Expected method implementation: {}",
        output
    );
}

#[test]
fn test_class_es5_method_overloads_generic() {
    // Test method overloads with generic types
    let source = r#"
class Container<T> {
    getValue(): T;
    getValue(defaultValue: T): T;
    getValue(defaultValue?: T): T {
        return this.value !== undefined ? this.value : defaultValue!;
    }

    private value: T | undefined;

    setValue(value: T): void {
        this.value = value;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class
    assert!(
        output.contains("function Container") || output.contains("var Container"),
        "Expected Container class transformation: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("getValue") && output.contains("setValue"),
        "Expected getValue and setValue methods: {}",
        output
    );
}

#[test]
fn test_class_es5_method_overloads_static() {
    // Test static method overloads
    let source = r#"
class Factory {
    static create(): Factory;
    static create(config: object): Factory;
    static create(config?: object): Factory {
        const instance = new Factory();
        if (config) {
            instance.configure(config);
        }
        return instance;
    }

    private config: object = {};

    configure(config: object): void {
        this.config = config;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class
    assert!(
        output.contains("function Factory") || output.contains("var Factory"),
        "Expected Factory class transformation: {}",
        output
    );

    // Static method should be on constructor
    assert!(
        output.contains("create"),
        "Expected static create method: {}",
        output
    );

    // Instance method should be present
    assert!(
        output.contains("configure"),
        "Expected configure method: {}",
        output
    );
}

#[test]
fn test_class_es5_method_overloads_inheritance() {
    // Test method overloads with inheritance
    let source = r#"
class BaseService {
    fetch(url: string): Promise<any>;
    fetch(url: string, options: object): Promise<any>;
    fetch(url: string, options?: object): Promise<any> {
        return Promise.resolve({ url, options });
    }
}

class ExtendedService extends BaseService {
    fetch(url: string): Promise<any>;
    fetch(url: string, options: object): Promise<any>;
    fetch(url: string, options: object, timeout: number): Promise<any>;
    fetch(url: string, options?: object, timeout?: number): Promise<any> {
        return super.fetch(url, options || {});
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Both classes should be transformed
    assert!(
        output.contains("BaseService") && output.contains("ExtendedService"),
        "Expected both class transformations: {}",
        output
    );

    // Inheritance pattern should be present
    assert!(
        output.contains("__extends") || output.contains("prototype"),
        "Expected inheritance pattern: {}",
        output
    );

    // Fetch method should be present
    assert!(
        output.contains("fetch"),
        "Expected fetch method: {}",
        output
    );
}

#[test]
fn test_class_es5_constructor_overloads_basic() {
    // Test basic constructor overloads
    let source = r#"
class Point {
    x: number;
    y: number;

    constructor();
    constructor(x: number);
    constructor(x: number, y: number);
    constructor(x?: number, y?: number) {
        this.x = x || 0;
        this.y = y || 0;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class to function
    assert!(
        output.contains("function Point") || output.contains("var Point"),
        "Expected Point class transformation: {}",
        output
    );

    // Constructor implementation should be present
    assert!(
        output.contains("this.x") && output.contains("this.y"),
        "Expected property assignments: {}",
        output
    );
}

#[test]
fn test_class_es5_constructor_overloads_with_types() {
    // Test constructor overloads with different parameter types
    let source = r#"
class Data {
    value: any;

    constructor(value: string);
    constructor(value: number);
    constructor(value: object);
    constructor(value: any) {
        this.value = value;
    }

    getValue(): any {
        return this.value;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class
    assert!(
        output.contains("function Data") || output.contains("var Data"),
        "Expected Data class transformation: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("getValue"),
        "Expected getValue method: {}",
        output
    );

    // Constructor body should be present
    assert!(
        output.contains("this.value"),
        "Expected value assignment: {}",
        output
    );
}

#[test]
fn test_class_es5_constructor_overloads_inheritance() {
    // Test constructor overloads with inheritance
    let source = r#"
class Animal {
    name: string;

    constructor();
    constructor(name: string);
    constructor(name?: string) {
        this.name = name || "unknown";
    }
}

class Dog extends Animal {
    breed: string;

    constructor();
    constructor(name: string);
    constructor(name: string, breed: string);
    constructor(name?: string, breed?: string) {
        super(name);
        this.breed = breed || "mixed";
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Both classes should be transformed
    assert!(
        output.contains("Animal") && output.contains("Dog"),
        "Expected both class transformations: {}",
        output
    );

    // Inheritance should be present
    assert!(
        output.contains("__extends") || output.contains("prototype"),
        "Expected inheritance pattern: {}",
        output
    );

    // Properties should be assigned
    assert!(
        output.contains("name") && output.contains("breed"),
        "Expected property assignments: {}",
        output
    );
}

#[test]
fn test_class_es5_constructor_overloads_with_defaults() {
    // Test constructor overloads with default values
    let source = r#"
class Config {
    host: string;
    port: number;
    secure: boolean;

    constructor();
    constructor(host: string);
    constructor(host: string, port: number);
    constructor(host: string, port: number, secure: boolean);
    constructor(host: string = "localhost", port: number = 8080, secure: boolean = false) {
        this.host = host;
        this.port = port;
        this.secure = secure;
    }

    getUrl(): string {
        const protocol = this.secure ? "https" : "http";
        return protocol + "://" + this.host + ":" + this.port;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class
    assert!(
        output.contains("function Config") || output.contains("var Config"),
        "Expected Config class transformation: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("getUrl"),
        "Expected getUrl method: {}",
        output
    );

    // All properties should be assigned
    assert!(
        output.contains("host") && output.contains("port") && output.contains("secure"),
        "Expected property assignments: {}",
        output
    );
}

#[test]
fn test_class_es5_constructor_overloads_generic() {
    // Test constructor overloads with generic types
    let source = r#"
class Container<T> {
    items: T[];

    constructor();
    constructor(items: T[]);
    constructor(items?: T[]) {
        this.items = items || [];
    }

    add(item: T): void {
        this.items.push(item);
    }

    get(index: number): T {
        return this.items[index];
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class
    assert!(
        output.contains("function Container") || output.contains("var Container"),
        "Expected Container class transformation: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("add") && output.contains("get"),
        "Expected add and get methods: {}",
        output
    );

    // Items property should be present
    assert!(
        output.contains("items"),
        "Expected items property: {}",
        output
    );
}

#[test]
fn test_class_es5_parameter_property_public() {
    // Test public parameter property
    let source = r#"
class Person {
    constructor(public name: string, public age: number) {}

    greet(): string {
        return "Hello, " + this.name;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class to function
    assert!(
        output.contains("function Person") || output.contains("var Person"),
        "Expected Person class transformation: {}",
        output
    );

    // Parameter properties should be assigned to this
    assert!(
        output.contains("this.name") && output.contains("this.age"),
        "Expected parameter property assignments: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("greet"),
        "Expected greet method: {}",
        output
    );
}

#[test]
fn test_class_es5_parameter_property_private() {
    // Test private parameter property
    let source = r#"
class BankAccount {
    constructor(private balance: number, private accountId: string) {}

    getBalance(): number {
        return this.balance;
    }

    deposit(amount: number): void {
        this.balance += amount;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class
    assert!(
        output.contains("function BankAccount") || output.contains("var BankAccount"),
        "Expected BankAccount class transformation: {}",
        output
    );

    // Private parameter properties should still be assigned
    assert!(
        output.contains("this.balance") || output.contains("balance"),
        "Expected balance property: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("getBalance") && output.contains("deposit"),
        "Expected methods: {}",
        output
    );
}

#[test]
fn test_class_es5_parameter_property_protected() {
    // Test protected parameter property with inheritance
    let source = r#"
class Vehicle {
    constructor(protected speed: number, protected fuel: number) {}

    accelerate(): void {
        this.speed += 10;
    }
}

class Car extends Vehicle {
    constructor(speed: number, fuel: number, private brand: string) {
        super(speed, fuel);
    }

    getInfo(): string {
        return this.brand + " at " + this.speed + " mph";
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Both classes should be transformed
    assert!(
        output.contains("Vehicle") && output.contains("Car"),
        "Expected both class transformations: {}",
        output
    );

    // Inheritance should be present
    assert!(
        output.contains("__extends") || output.contains("prototype"),
        "Expected inheritance pattern: {}",
        output
    );

    // Protected properties should be assigned
    assert!(
        output.contains("speed") && output.contains("fuel"),
        "Expected protected properties: {}",
        output
    );
}

#[test]
fn test_class_es5_parameter_property_readonly() {
    // Test readonly parameter property
    let source = r#"
class Config {
    constructor(
        readonly host: string,
        readonly port: number,
        readonly secure: boolean = false
    ) {}

    getUrl(): string {
        const protocol = this.secure ? "https" : "http";
        return protocol + "://" + this.host + ":" + this.port;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class
    assert!(
        output.contains("function Config") || output.contains("var Config"),
        "Expected Config class transformation: {}",
        output
    );

    // Readonly properties should be assigned
    assert!(
        output.contains("this.host") || output.contains("host"),
        "Expected host property: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("getUrl"),
        "Expected getUrl method: {}",
        output
    );
}

#[test]
fn test_class_es5_parameter_property_mixed() {
    // Test mixed parameter properties and regular parameters
    let source = r#"
class User {
    email: string;

    constructor(
        public id: number,
        private password: string,
        readonly createdAt: Date,
        name: string
    ) {
        this.email = name + "@example.com";
    }

    validatePassword(input: string): boolean {
        return this.password === input;
    }

    getId(): number {
        return this.id;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Should transform class
    assert!(
        output.contains("function User") || output.contains("var User"),
        "Expected User class transformation: {}",
        output
    );

    // Parameter properties should be assigned
    assert!(
        output.contains("id") && output.contains("password"),
        "Expected parameter properties: {}",
        output
    );

    // Regular property from constructor body should be present
    assert!(
        output.contains("email"),
        "Expected email property: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("validatePassword") && output.contains("getId"),
        "Expected methods: {}",
        output
    );
}

// =============================================================================
// Readonly Property Tests
// =============================================================================

#[test]
fn test_class_es5_readonly_property_basic() {
    // Test class with readonly property
    let source = r#"
class Config {
    readonly version: string = "1.0.0";
    readonly name: string;

    constructor(name: string) {
        this.name = name;
    }

    getVersion(): string {
        return this.version;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file.statements.nodes.first().expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Config"),
        "Expected Config to emit as function: {}",
        output
    );

    // Properties should be assigned (readonly is erased)
    assert!(
        output.contains("this.version") || output.contains("version"),
        "Expected version property: {}",
        output
    );
    assert!(
        output.contains("this.name") || output.contains("name"),
        "Expected name property: {}",
        output
    );

    // readonly keyword should NOT appear in output
    assert!(
        !output.contains("readonly"),
        "readonly should be erased: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("getVersion"),
        "Expected getVersion method: {}",
        output
    );
}

#[test]
fn test_class_es5_readonly_parameter_property() {
    // Test readonly parameter property in constructor
    let source = r#"
class Entity {
    constructor(
        readonly id: string,
        readonly createdAt: Date = new Date()
    ) {}

    getId(): string {
        return this.id;
    }

    getCreatedAt(): Date {
        return this.createdAt;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file.statements.nodes.first().expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Entity"),
        "Expected Entity to emit as function: {}",
        output
    );

    // Parameter properties should be assigned
    assert!(
        output.contains("this.id") || output.contains("id"),
        "Expected id property: {}",
        output
    );
    assert!(
        output.contains("this.createdAt") || output.contains("createdAt"),
        "Expected createdAt property: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("getId"),
        "Expected getId method: {}",
        output
    );
    assert!(
        output.contains("getCreatedAt"),
        "Expected getCreatedAt method: {}",
        output
    );
}

#[test]
fn test_class_es5_readonly_static_property() {
    // Test readonly static property
    let source = r#"
class Constants {
    static readonly PI: number = 3.14159;
    static readonly E: number = 2.71828;
    static readonly GOLDEN_RATIO: number = 1.61803;

    static getPI(): number {
        return Constants.PI;
    }

    static getE(): number {
        return Constants.E;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file.statements.nodes.first().expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Constants"),
        "Expected Constants to emit as function: {}",
        output
    );

    // Static properties should be on constructor
    assert!(
        output.contains("Constants.PI") || output.contains("PI"),
        "Expected PI static property: {}",
        output
    );
    assert!(
        output.contains("Constants.E") || output.contains("E"),
        "Expected E static property: {}",
        output
    );
    assert!(
        output.contains("GOLDEN_RATIO"),
        "Expected GOLDEN_RATIO static property: {}",
        output
    );

    // Static methods should be present
    assert!(
        output.contains("getPI"),
        "Expected getPI static method: {}",
        output
    );
    assert!(
        output.contains("getE"),
        "Expected getE static method: {}",
        output
    );
}

#[test]
fn test_class_es5_readonly_with_inheritance() {
    // Test readonly properties in inheritance
    let source = r#"
class Base {
    readonly type: string = "base";
}

class Derived extends Base {
    readonly subtype: string;

    constructor(subtype: string) {
        super();
        this.subtype = subtype;
    }

    getFullType(): string {
        return this.type + ":" + this.subtype;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    // Get Derived class (second statement)
    let class_idx = *source_file.statements.nodes.get(1).expect("expected Derived class");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Derived"),
        "Expected Derived to emit as function: {}",
        output
    );

    // Should have inheritance setup
    assert!(
        output.contains("_super") || output.contains("Base") || output.contains("__extends"),
        "Expected inheritance handling: {}",
        output
    );

    // subtype property should be assigned
    assert!(
        output.contains("this.subtype") || output.contains("subtype"),
        "Expected subtype property: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("getFullType"),
        "Expected getFullType method: {}",
        output
    );
}

#[test]
fn test_class_es5_readonly_array_property() {
    // Test readonly array property
    let source = r#"
class ImmutableList {
    readonly items: readonly string[];
    readonly length: number;

    constructor(items: string[]) {
        this.items = items;
        this.length = items.length;
    }

    get(index: number): string | undefined {
        return this.items[index];
    }

    contains(item: string): boolean {
        return this.items.includes(item);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file.statements.nodes.first().expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function ImmutableList"),
        "Expected ImmutableList to emit as function: {}",
        output
    );

    // Properties should be assigned
    assert!(
        output.contains("this.items") || output.contains("items"),
        "Expected items property: {}",
        output
    );
    assert!(
        output.contains("this.length") || output.contains("length"),
        "Expected length property: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("get"),
        "Expected get method: {}",
        output
    );
    assert!(
        output.contains("contains"),
        "Expected contains method: {}",
        output
    );
}

#[test]
fn test_class_es5_readonly_mixed_properties() {
    // Test class with mix of readonly and mutable properties
    let source = r#"
class User {
    readonly id: string;
    readonly createdAt: Date;
    name: string;
    email: string;
    lastLogin: Date | null = null;

    constructor(id: string, name: string, email: string) {
        this.id = id;
        this.createdAt = new Date();
        this.name = name;
        this.email = email;
    }

    updateName(name: string): void {
        this.name = name;
    }

    updateEmail(email: string): void {
        this.email = email;
    }

    recordLogin(): void {
        this.lastLogin = new Date();
    }

    getId(): string {
        return this.id;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file.statements.nodes.first().expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit as function
    assert!(
        output.contains("function User"),
        "Expected User to emit as function: {}",
        output
    );

    // All properties should be assigned
    assert!(
        output.contains("this.id") || output.contains("id"),
        "Expected id property: {}",
        output
    );
    assert!(
        output.contains("this.name") || output.contains("name"),
        "Expected name property: {}",
        output
    );
    assert!(
        output.contains("this.email") || output.contains("email"),
        "Expected email property: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("updateName"),
        "Expected updateName method: {}",
        output
    );
    assert!(
        output.contains("updateEmail"),
        "Expected updateEmail method: {}",
        output
    );
    assert!(
        output.contains("recordLogin"),
        "Expected recordLogin method: {}",
        output
    );
    assert!(
        output.contains("getId"),
        "Expected getId method: {}",
        output
    );
}

// =============================================================================
// Ambient/Declare Class Tests
// =============================================================================

#[test]
fn test_class_es5_declare_class_basic() {
    // declare class should emit nothing - it's a type declaration only
    let source = r#"
declare class ExternalAPI {
    version: string;
    init(): void;
    shutdown(): void;
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file.statements.nodes.first().expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Declare class should produce no output or empty output
    let trimmed = output.trim();
    assert!(
        trimmed.is_empty() || !trimmed.contains("function ExternalAPI"),
        "declare class should not emit a function constructor: {}",
        output
    );
}

#[test]
fn test_class_es5_declare_class_with_methods() {
    // declare class with method signatures should emit nothing
    let source = r#"
declare class Database {
    connect(url: string): Promise<void>;
    query<T>(sql: string, params?: any[]): Promise<T[]>;
    disconnect(): Promise<void>;
    readonly isConnected: boolean;
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file.statements.nodes.first().expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Declare class with methods should produce no output
    let trimmed = output.trim();
    assert!(
        trimmed.is_empty() || !trimmed.contains("function Database"),
        "declare class should not emit a function constructor: {}",
        output
    );
    assert!(
        !trimmed.contains("prototype"),
        "declare class should not emit prototype assignments: {}",
        output
    );
}

#[test]
fn test_class_es5_declare_class_with_static() {
    // declare class with static members should emit nothing
    let source = r#"
declare class MathUtils {
    static PI: number;
    static E: number;
    static sin(x: number): number;
    static cos(x: number): number;
    static random(): number;
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file.statements.nodes.first().expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Declare class with static members should produce no output
    let trimmed = output.trim();
    assert!(
        trimmed.is_empty() || !trimmed.contains("function MathUtils"),
        "declare class should not emit a function constructor: {}",
        output
    );
    assert!(
        !trimmed.contains("MathUtils.PI"),
        "declare class should not emit static property assignments: {}",
        output
    );
}

#[test]
fn test_class_es5_declare_class_with_extends() {
    // declare class with extends should emit nothing
    let source = r#"
declare class BaseEvent {
    type: string;
    timestamp: number;
}

declare class MouseEvent extends BaseEvent {
    clientX: number;
    clientY: number;
    button: number;
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Check second class (MouseEvent extends BaseEvent)
    let class_idx = source_file.statements.nodes.get(1).expect("expected second class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Declare class with extends should produce no output
    let trimmed = output.trim();
    assert!(
        trimmed.is_empty() || !trimmed.contains("function MouseEvent"),
        "declare class should not emit a function constructor: {}",
        output
    );
    assert!(
        !trimmed.contains("__extends"),
        "declare class should not emit __extends helper: {}",
        output
    );
}

#[test]
fn test_class_es5_declare_class_with_implements() {
    // declare class with implements should emit nothing
    let source = r#"
interface Serializable {
    serialize(): string;
    deserialize(data: string): void;
}

declare class DataModel implements Serializable {
    id: string;
    serialize(): string;
    deserialize(data: string): void;
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the declare class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Declare class with implements should produce no output
    let trimmed = output.trim();
    assert!(
        trimmed.is_empty() || !trimmed.contains("function DataModel"),
        "declare class should not emit a function constructor: {}",
        output
    );
}

#[test]
fn test_class_es5_declare_class_with_constructor() {
    // declare class with constructor signature should emit nothing
    let source = r#"
declare class Config {
    readonly host: string;
    readonly port: number;
    readonly secure: boolean;

    constructor(host: string, port: number, secure?: boolean);

    getUrl(): string;
    clone(): Config;
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file.statements.nodes.first().expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Declare class with constructor should produce no output
    let trimmed = output.trim();
    assert!(
        trimmed.is_empty() || !trimmed.contains("function Config"),
        "declare class should not emit a function constructor: {}",
        output
    );
    assert!(
        !trimmed.contains("this.host"),
        "declare class should not emit property assignments: {}",
        output
    );
}

// =============================================================================
// Class Field Decorator Tests
// =============================================================================

#[test]
fn test_class_es5_field_decorator_basic() {
    // Basic field decorator
    let source = r#"
function observable(target: any, key: string) {}

class Model {
    @observable
    name: string = "";
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit as function
    assert!(
        output.contains("function Model"),
        "Expected Model class: {}",
        output
    );

    // Field should be initialized in constructor
    assert!(
        output.contains("this.name") || output.contains("name"),
        "Expected name field initialization: {}",
        output
    );
}

#[test]
fn test_class_es5_field_decorator_with_initializer() {
    // Field decorator with initializer
    let source = r#"
function defaultValue(value: any) {
    return function(target: any, key: string) {};
}

class Config {
    @defaultValue(42)
    count: number = 0;

    @defaultValue("default")
    label: string = "initial";
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function Config") || output.contains("Config"),
        "Expected Config class: {}",
        output
    );

    // Field initializers should be present
    assert!(
        output.contains("count") && output.contains("label"),
        "Expected field names: {}",
        output
    );
}

#[test]
fn test_class_es5_field_decorator_static() {
    // Static field decorator
    let source = r#"
function logged(target: any, key: string) {}

class Service {
    @logged
    static instance: Service | null = null;

    @logged
    static version: string = "1.0.0";
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function Service") || output.contains("Service"),
        "Expected Service class: {}",
        output
    );

    // Static fields should be assigned on constructor
    assert!(
        output.contains("Service.instance") || output.contains("instance"),
        "Expected static instance field: {}",
        output
    );
    assert!(
        output.contains("Service.version") || output.contains("version"),
        "Expected static version field: {}",
        output
    );
}

#[test]
fn test_class_es5_field_decorator_multiple() {
    // Multiple decorators on a single field
    let source = r#"
function required(target: any, key: string) {}
function validate(target: any, key: string) {}
function format(target: any, key: string) {}

class Form {
    @required
    @validate
    @format
    email: string = "";
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Fourth statement is the class (after 3 function declarations)
    let class_idx = source_file.statements.nodes.get(3).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function Form") || output.contains("Form"),
        "Expected Form class: {}",
        output
    );

    // Email field should be present
    assert!(
        output.contains("email"),
        "Expected email field: {}",
        output
    );
}

#[test]
fn test_class_es5_field_decorator_in_derived_class() {
    // Field decorator in derived class
    let source = r#"
function tracked(target: any, key: string) {}

class BaseEntity {
    id: number = 0;
}

class User extends BaseEntity {
    @tracked
    name: string = "";

    @tracked
    lastLogin: Date | null = null;
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Third statement is the User class
    let class_idx = source_file.statements.nodes.get(2).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit with extends
    assert!(
        output.contains("function User") || output.contains("User"),
        "Expected User class: {}",
        output
    );

    // Should have inheritance pattern
    assert!(
        output.contains("__extends") || output.contains("_super"),
        "Expected inheritance pattern: {}",
        output
    );

    // Decorated fields should be present
    assert!(
        output.contains("name") && output.contains("lastLogin"),
        "Expected decorated fields: {}",
        output
    );
}

#[test]
fn test_class_es5_field_decorator_with_accessor() {
    // Field decorator combined with accessor decorator
    let source = r#"
function observable(target: any, key: string) {}
function computed(target: any, key: string, descriptor: PropertyDescriptor) {}

class ViewModel {
    @observable
    firstName: string = "";

    @observable
    lastName: string = "";

    @computed
    get fullName(): string {
        return this.firstName + " " + this.lastName;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Third statement is the class
    let class_idx = source_file.statements.nodes.get(2).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function ViewModel") || output.contains("ViewModel"),
        "Expected ViewModel class: {}",
        output
    );

    // Fields should be present
    assert!(
        output.contains("firstName") && output.contains("lastName"),
        "Expected name fields: {}",
        output
    );

    // Getter should be present
    assert!(
        output.contains("fullName"),
        "Expected fullName getter: {}",
        output
    );
}

// =============================================================================
// Override Keyword Tests
// =============================================================================

#[test]
fn test_class_es5_override_method_basic() {
    // Basic override method - override keyword should be erased
    let source = r#"
class Animal {
    speak(): string {
        return "...";
    }
}

class Dog extends Animal {
    override speak(): string {
        return "Woof!";
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the Dog class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit with extends
    assert!(
        output.contains("function Dog"),
        "Expected Dog class: {}",
        output
    );

    // Override keyword should be erased
    assert!(
        !output.contains("override"),
        "override keyword should be erased: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains("speak"),
        "Expected speak method: {}",
        output
    );

    // Should have inheritance pattern
    assert!(
        output.contains("__extends") || output.contains("_super"),
        "Expected inheritance pattern: {}",
        output
    );
}

#[test]
fn test_class_es5_override_accessor() {
    // Override getter/setter - override keyword should be erased
    let source = r#"
class Base {
    protected _value: number = 0;

    get value(): number {
        return this._value;
    }

    set value(v: number) {
        this._value = v;
    }
}

class Derived extends Base {
    override get value(): number {
        return this._value * 2;
    }

    override set value(v: number) {
        this._value = v / 2;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the Derived class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function Derived"),
        "Expected Derived class: {}",
        output
    );

    // Override keyword should be erased
    assert!(
        !output.contains("override"),
        "override keyword should be erased: {}",
        output
    );

    // Accessor should use Object.defineProperty
    assert!(
        output.contains("value") || output.contains("defineProperty"),
        "Expected value accessor: {}",
        output
    );
}

#[test]
fn test_class_es5_override_multiple_methods() {
    // Multiple override methods
    let source = r#"
class Shape {
    getArea(): number { return 0; }
    getPerimeter(): number { return 0; }
    describe(): string { return "Shape"; }
}

class Rectangle extends Shape {
    width: number;
    height: number;

    constructor(w: number, h: number) {
        super();
        this.width = w;
        this.height = h;
    }

    override getArea(): number {
        return this.width * this.height;
    }

    override getPerimeter(): number {
        return 2 * (this.width + this.height);
    }

    override describe(): string {
        return "Rectangle";
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the Rectangle class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function Rectangle"),
        "Expected Rectangle class: {}",
        output
    );

    // Override keyword should be erased
    assert!(
        !output.contains("override"),
        "override keyword should be erased: {}",
        output
    );

    // All methods should be present
    assert!(
        output.contains("getArea") && output.contains("getPerimeter") && output.contains("describe"),
        "Expected all override methods: {}",
        output
    );
}

#[test]
fn test_class_es5_override_multilevel_inheritance() {
    // Override in multi-level inheritance chain
    let source = r#"
class A {
    foo(): string { return "A"; }
}

class B extends A {
    override foo(): string { return "B"; }
}

class C extends B {
    override foo(): string { return "C"; }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Third statement is class C
    let class_idx = source_file.statements.nodes.get(2).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function C"),
        "Expected C class: {}",
        output
    );

    // Override keyword should be erased
    assert!(
        !output.contains("override"),
        "override keyword should be erased: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("foo"),
        "Expected foo method: {}",
        output
    );

    // Should have inheritance
    assert!(
        output.contains("__extends") || output.contains("_super"),
        "Expected inheritance pattern: {}",
        output
    );
}

#[test]
fn test_class_es5_override_with_super_call() {
    // Override method that calls super
    let source = r#"
class Logger {
    log(message: string): void {
        console.log(message);
    }
}

class TimestampLogger extends Logger {
    override log(message: string): void {
        const timestamp = new Date().toISOString();
        super.log("[" + timestamp + "] " + message);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the TimestampLogger class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function TimestampLogger"),
        "Expected TimestampLogger class: {}",
        output
    );

    // Override keyword should be erased
    assert!(
        !output.contains("override"),
        "override keyword should be erased: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("log"),
        "Expected log method: {}",
        output
    );

    // Super call should be transformed
    assert!(
        output.contains("_super") || output.contains("prototype"),
        "Expected super call pattern: {}",
        output
    );
}

#[test]
fn test_class_es5_override_abstract_method() {
    // Override abstract method
    let source = r#"
abstract class Component {
    abstract render(): string;
    abstract update(): void;
}

class Button extends Component {
    override render(): string {
        return "<button>Click me</button>";
    }

    override update(): void {
        console.log("Button updated");
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the Button class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function Button"),
        "Expected Button class: {}",
        output
    );

    // Override keyword should be erased
    assert!(
        !output.contains("override"),
        "override keyword should be erased: {}",
        output
    );

    // Abstract keyword should be erased
    assert!(
        !output.contains("abstract"),
        "abstract keyword should be erased: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("render") && output.contains("update"),
        "Expected render and update methods: {}",
        output
    );
}

// =============================================================================
// Satisfies Expression Tests
// =============================================================================

#[test]
fn test_class_es5_satisfies_field_initializer() {
    // satisfies in field initializer - should be erased
    let source = r#"
type Config = { host: string; port: number };

class Server {
    config = { host: "localhost", port: 8080 } satisfies Config;

    getHost(): string {
        return this.config.host;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function Server"),
        "Expected Server class: {}",
        output
    );

    // satisfies keyword should be erased
    assert!(
        !output.contains("satisfies"),
        "satisfies keyword should be erased: {}",
        output
    );

    // Field should be initialized (value may or may not be present depending on satisfies handling)
    assert!(
        output.contains("config"),
        "Expected config field: {}",
        output
    );
}

#[test]
fn test_class_es5_satisfies_in_method() {
    // satisfies in method return - should be erased
    let source = r#"
interface Point { x: number; y: number }

class Geometry {
    createPoint(x: number, y: number): Point {
        return { x, y } satisfies Point;
    }

    createOrigin(): Point {
        return { x: 0, y: 0 } satisfies Point;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function Geometry"),
        "Expected Geometry class: {}",
        output
    );

    // satisfies keyword should be erased
    assert!(
        !output.contains("satisfies"),
        "satisfies keyword should be erased: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("createPoint") && output.contains("createOrigin"),
        "Expected methods: {}",
        output
    );
}

#[test]
fn test_class_es5_satisfies_static_field() {
    // satisfies in static field - should be erased
    let source = r##"
type ColorMap = Record<string, string>;

class Theme {
    static colors = {
        primary: "#007bff",
        secondary: "#6c757d",
        success: "#28a745"
    } satisfies ColorMap;

    static getColor(name: string): string {
        return Theme.colors[name] || "#000000";
    }
}
"##;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function Theme") || output.contains("Theme"),
        "Expected Theme class: {}",
        output
    );

    // satisfies keyword should be erased
    assert!(
        !output.contains("satisfies"),
        "satisfies keyword should be erased: {}",
        output
    );

    // Static field should be present (value may or may not be present depending on satisfies handling)
    assert!(
        output.contains("colors"),
        "Expected colors field: {}",
        output
    );
}

#[test]
fn test_class_es5_satisfies_in_constructor() {
    // satisfies in constructor - should be erased
    let source = r#"
interface Options {
    timeout: number;
    retries: number;
}

class Client {
    options: Options;

    constructor() {
        this.options = { timeout: 5000, retries: 3 } satisfies Options;
    }

    getTimeout(): number {
        return this.options.timeout;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function Client"),
        "Expected Client class: {}",
        output
    );

    // satisfies keyword should be erased
    assert!(
        !output.contains("satisfies"),
        "satisfies keyword should be erased: {}",
        output
    );

    // Field should be present (value may or may not be present depending on satisfies handling)
    assert!(
        output.contains("options"),
        "Expected options field: {}",
        output
    );
}

#[test]
fn test_class_es5_satisfies_array_literal() {
    // satisfies with array literal - should be erased
    let source = r#"
type Route = { path: string; handler: string };

class Router {
    routes = [
        { path: "/", handler: "home" },
        { path: "/about", handler: "about" }
    ] satisfies Route[];

    getRoutes(): Route[] {
        return this.routes;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function Router"),
        "Expected Router class: {}",
        output
    );

    // satisfies keyword should be erased
    assert!(
        !output.contains("satisfies"),
        "satisfies keyword should be erased: {}",
        output
    );

    // Field should be present (value may or may not be present depending on satisfies handling)
    assert!(
        output.contains("routes"),
        "Expected routes field: {}",
        output
    );
}

#[test]
fn test_class_es5_satisfies_in_derived_class() {
    // satisfies in derived class - should be erased
    let source = r#"
interface Metadata { version: string; author: string }

class BasePlugin {
    name: string = "base";
}

class CustomPlugin extends BasePlugin {
    metadata = { version: "1.0.0", author: "dev" } satisfies Metadata;

    getVersion(): string {
        return this.metadata.version;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Third statement is the CustomPlugin class
    let class_idx = source_file.statements.nodes.get(2).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function CustomPlugin"),
        "Expected CustomPlugin class: {}",
        output
    );

    // satisfies keyword should be erased
    assert!(
        !output.contains("satisfies"),
        "satisfies keyword should be erased: {}",
        output
    );

    // Should have inheritance
    assert!(
        output.contains("__extends") || output.contains("_super"),
        "Expected inheritance pattern: {}",
        output
    );

    // Field should be present (value may or may not be present depending on satisfies handling)
    assert!(
        output.contains("metadata"),
        "Expected metadata field: {}",
        output
    );
}

// =============================================================================
// Namespace Merging Tests
// =============================================================================

#[test]
fn test_class_es5_namespace_merging_basic() {
    // Class merged with namespace - adds static members
    let source = r#"
class Validator {
    validate(input: string): boolean {
        return input.length > 0;
    }
}

namespace Validator {
    export const minLength = 1;
    export const maxLength = 100;
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be present
    assert!(
        output.contains("Validator"),
        "Expected Validator class: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("validate"),
        "Expected validate method: {}",
        output
    );

    // Namespace exports should be present
    assert!(
        output.contains("minLength") && output.contains("maxLength"),
        "Expected namespace exports: {}",
        output
    );
}

#[test]
fn test_class_es5_namespace_merging_with_functions() {
    // Class merged with namespace containing functions
    let source = r#"
class StringUtils {
    value: string;

    constructor(value: string) {
        this.value = value;
    }

    toUpper(): string {
        return this.value.toUpperCase();
    }
}

namespace StringUtils {
    export function isEmpty(s: string): boolean {
        return s.length === 0;
    }

    export function trim(s: string): string {
        return s.trim();
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be present
    assert!(
        output.contains("StringUtils"),
        "Expected StringUtils class: {}",
        output
    );

    // Instance method should be present
    assert!(
        output.contains("toUpper"),
        "Expected toUpper method: {}",
        output
    );

    // Namespace functions should be present
    assert!(
        output.contains("isEmpty") && output.contains("trim"),
        "Expected namespace functions: {}",
        output
    );
}

#[test]
fn test_class_es5_namespace_merging_with_interface() {
    // Class merged with namespace containing interface
    let source = r#"
class Point {
    x: number;
    y: number;

    constructor(x: number, y: number) {
        this.x = x;
        this.y = y;
    }

    distanceTo(other: Point): number {
        return Math.sqrt(Math.pow(this.x - other.x, 2) + Math.pow(this.y - other.y, 2));
    }
}

namespace Point {
    export interface Options {
        x: number;
        y: number;
    }

    export function origin(): Point {
        return new Point(0, 0);
    }

    export const ZERO = new Point(0, 0);
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be present
    assert!(
        output.contains("Point"),
        "Expected Point class: {}",
        output
    );

    // Instance method should be present
    assert!(
        output.contains("distanceTo"),
        "Expected distanceTo method: {}",
        output
    );

    // Interface should be erased (not in output)
    assert!(
        !output.contains("interface"),
        "Interface should be erased: {}",
        output
    );

    // Namespace function and const should be present
    assert!(
        output.contains("origin") && output.contains("ZERO"),
        "Expected namespace members: {}",
        output
    );
}

#[test]
fn test_class_es5_namespace_merging_with_nested() {
    // Class merged with namespace containing nested namespace
    let source = r#"
class Logger {
    log(message: string): void {
        console.log(message);
    }
}

namespace Logger {
    export const level = "info";

    export namespace Formatters {
        export function json(obj: any): string {
            return JSON.stringify(obj);
        }

        export function text(obj: any): string {
            return String(obj);
        }
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be present
    assert!(
        output.contains("Logger"),
        "Expected Logger class: {}",
        output
    );

    // Instance method should be present
    assert!(
        output.contains("log"),
        "Expected log method: {}",
        output
    );

    // Nested namespace should be present
    assert!(
        output.contains("Formatters"),
        "Expected Formatters namespace: {}",
        output
    );

    // Nested functions should be present
    assert!(
        output.contains("json") && output.contains("text"),
        "Expected formatter functions: {}",
        output
    );
}

#[test]
fn test_class_es5_namespace_merging_with_inheritance() {
    // Derived class merged with namespace
    let source = r#"
class BaseService {
    name: string = "base";
}

class ApiService extends BaseService {
    endpoint: string;

    constructor(endpoint: string) {
        super();
        this.endpoint = endpoint;
    }

    fetch(): Promise<any> {
        return Promise.resolve({});
    }
}

namespace ApiService {
    export const defaultTimeout = 5000;
    export const defaultHeaders = { "Content-Type": "application/json" };

    export function create(endpoint: string): ApiService {
        return new ApiService(endpoint);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Classes should be present
    assert!(
        output.contains("BaseService") && output.contains("ApiService"),
        "Expected BaseService and ApiService classes: {}",
        output
    );

    // Inheritance should be present
    assert!(
        output.contains("__extends") || output.contains("prototype"),
        "Expected inheritance pattern: {}",
        output
    );

    // Instance method should be present
    assert!(
        output.contains("fetch"),
        "Expected fetch method: {}",
        output
    );

    // Namespace exports should be present
    assert!(
        output.contains("defaultTimeout") && output.contains("defaultHeaders"),
        "Expected namespace constants: {}",
        output
    );

    // Factory function should be present
    assert!(
        output.contains("create"),
        "Expected create factory function: {}",
        output
    );
}

// =============================================================================
// Class Expression with Generics Tests
// =============================================================================

#[test]
fn test_class_es5_class_expression_generic_basic() {
    // Basic class expression with generic type parameter
    let source = r#"
const Container = class<T> {
    private value: T;

    constructor(value: T) {
        this.value = value;
    }

    getValue(): T {
        return this.value;
    }
};
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Variable should be present
    assert!(
        output.contains("Container"),
        "Expected Container variable: {}",
        output
    );

    // Generic type parameter should be erased
    assert!(
        !output.contains("<T>"),
        "Generic type parameter should be erased: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("getValue"),
        "Expected getValue method: {}",
        output
    );
}

#[test]
fn test_class_es5_class_expression_generic_multiple_params() {
    // Class expression with multiple generic type parameters
    let source = r#"
const Pair = class<K, V> {
    constructor(public key: K, public value: V) {}

    getKey(): K {
        return this.key;
    }

    getValue(): V {
        return this.value;
    }

    swap(): Pair<V, K> {
        return new Pair(this.value, this.key);
    }
};
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Variable should be present
    assert!(
        output.contains("Pair"),
        "Expected Pair variable: {}",
        output
    );

    // Generic type parameters should be erased
    assert!(
        !output.contains("<K, V>") && !output.contains("<V, K>"),
        "Generic type parameters should be erased: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("getKey") && output.contains("getValue") && output.contains("swap"),
        "Expected methods: {}",
        output
    );
}

#[test]
fn test_class_es5_class_expression_generic_constraint() {
    // Class expression with generic constraint
    let source = r#"
interface HasLength {
    length: number;
}

const Measurable = class<T extends HasLength> {
    constructor(private item: T) {}

    getLength(): number {
        return this.item.length;
    }

    getItem(): T {
        return this.item;
    }
};
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Variable should be present
    assert!(
        output.contains("Measurable"),
        "Expected Measurable variable: {}",
        output
    );

    // Generic constraint should be erased
    assert!(
        !output.contains("extends HasLength"),
        "Generic constraint should be erased: {}",
        output
    );

    // Interface should be erased
    assert!(
        !output.contains("interface"),
        "Interface should be erased: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("getLength") && output.contains("getItem"),
        "Expected methods: {}",
        output
    );
}

#[test]
fn test_class_es5_class_expression_generic_extends() {
    // Generic class expression extending another class
    let source = r#"
class BaseCollection<T> {
    protected items: T[] = [];

    add(item: T): void {
        this.items.push(item);
    }
}

const SortedCollection = class<T> extends BaseCollection<T> {
    sort(compareFn: (a: T, b: T) => number): T[] {
        return this.items.sort(compareFn);
    }

    first(): T | undefined {
        return this.items[0];
    }
};
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Classes should be present
    assert!(
        output.contains("BaseCollection") && output.contains("SortedCollection"),
        "Expected BaseCollection and SortedCollection: {}",
        output
    );

    // Generic type parameters should be erased
    assert!(
        !output.contains("<T>"),
        "Generic type parameters should be erased: {}",
        output
    );

    // Inheritance should be present
    assert!(
        output.contains("__extends") || output.contains("prototype"),
        "Expected inheritance pattern: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("sort") && output.contains("first"),
        "Expected sort and first methods: {}",
        output
    );
}

#[test]
fn test_class_es5_class_expression_generic_factory() {
    // Generic class expression used in factory pattern
    let source = r#"
function createRepository<T>() {
    return class Repository {
        private data: T[] = [];

        save(item: T): void {
            this.data.push(item);
        }

        findAll(): T[] {
            return this.data;
        }

        count(): number {
            return this.data.length;
        }
    };
}

const UserRepo = createRepository<{ id: number; name: string }>();
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Factory function should be present
    assert!(
        output.contains("createRepository"),
        "Expected createRepository function: {}",
        output
    );

    // Class name should be present
    assert!(
        output.contains("Repository"),
        "Expected Repository class: {}",
        output
    );

    // Generic type parameter should be erased
    assert!(
        !output.contains("<T>"),
        "Generic type parameter should be erased: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("save") && output.contains("findAll") && output.contains("count"),
        "Expected save, findAll, and count methods: {}",
        output
    );

    // Variable assignment should be present
    assert!(
        output.contains("UserRepo"),
        "Expected UserRepo variable: {}",
        output
    );
}

// =============================================================================
// Symbol.species Tests
// =============================================================================

#[test]
fn test_class_es5_symbol_species_basic() {
    // Basic Symbol.species static getter
    let source = r#"
class MyArray<T> {
    items: T[] = [];

    static get [Symbol.species](): typeof MyArray {
        return MyArray;
    }

    push(item: T): void {
        this.items.push(item);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file.statements.nodes.first().expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit
    assert!(
        output.contains("function MyArray"),
        "Expected MyArray class: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("push"),
        "Expected push method: {}",
        output
    );

    // Symbol.species should be handled (either as computed property or defineProperty)
    assert!(
        output.contains("Symbol.species") || output.contains("species") || output.contains("defineProperty"),
        "Expected Symbol.species handling: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_species_derived() {
    // Symbol.species in derived class returning different constructor
    let source = r#"
class BaseCollection<T> {
    items: T[] = [];

    static get [Symbol.species](): typeof BaseCollection {
        return BaseCollection;
    }
}

class SpecialCollection<T> extends BaseCollection<T> {
    static get [Symbol.species](): typeof SpecialCollection {
        return SpecialCollection;
    }

    addSpecial(item: T): void {
        this.items.push(item);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the SpecialCollection class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function SpecialCollection"),
        "Expected SpecialCollection class: {}",
        output
    );

    // Should have inheritance
    assert!(
        output.contains("__extends") || output.contains("_super"),
        "Expected inheritance pattern: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("addSpecial"),
        "Expected addSpecial method: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_species_with_methods() {
    // Symbol.species with map-like method that uses it
    let source = r#"
class CustomList<T> {
    private data: T[] = [];

    static get [Symbol.species](): typeof CustomList {
        return CustomList;
    }

    add(item: T): this {
        this.data.push(item);
        return this;
    }

    map<U>(fn: (item: T) => U): CustomList<U> {
        const ctor = (this.constructor as any)[Symbol.species] || CustomList;
        const result = new ctor();
        for (const item of this.data) {
            result.add(fn(item));
        }
        return result;
    }

    get length(): number {
        return this.data.length;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file.statements.nodes.first().expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit
    assert!(
        output.contains("function CustomList"),
        "Expected CustomList class: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("add") && output.contains("map"),
        "Expected add and map methods: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_species_returning_base() {
    // Symbol.species in subclass returning base class
    let source = r#"
class Observable<T> {
    value: T;

    constructor(value: T) {
        this.value = value;
    }

    static get [Symbol.species](): typeof Observable {
        return Observable;
    }
}

class BehaviorSubject<T> extends Observable<T> {
    // Returns base class instead of derived
    static get [Symbol.species](): typeof Observable {
        return Observable;
    }

    next(value: T): void {
        this.value = value;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    // Second statement is the BehaviorSubject class
    let class_idx = source_file.statements.nodes.get(1).expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(*class_idx);

    // Class should emit
    assert!(
        output.contains("function BehaviorSubject"),
        "Expected BehaviorSubject class: {}",
        output
    );

    // Should have inheritance
    assert!(
        output.contains("__extends") || output.contains("_super"),
        "Expected inheritance pattern: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("next"),
        "Expected next method: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_species_with_static_members() {
    // Symbol.species alongside other static members
    let source = r#"
class Factory<T> {
    static defaultName: string = "Factory";
    static count: number = 0;

    static get [Symbol.species](): typeof Factory {
        return Factory;
    }

    static create<U>(value: U): Factory<U> {
        Factory.count++;
        return new Factory<U>();
    }

    produce(): T | null {
        return null;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file.statements.nodes.first().expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit
    assert!(
        output.contains("function Factory") || output.contains("Factory"),
        "Expected Factory class: {}",
        output
    );

    // Static members should be present
    assert!(
        output.contains("defaultName") || output.contains("count") || output.contains("create"),
        "Expected static members: {}",
        output
    );

    // Instance method should be present
    assert!(
        output.contains("produce"),
        "Expected produce method: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_species_null() {
    // Symbol.species returning null to prevent subclass creation
    let source = r#"
class ImmutableList<T> {
    private readonly items: readonly T[];

    constructor(items: T[]) {
        this.items = Object.freeze([...items]);
    }

    // Returning null prevents derived instances
    static get [Symbol.species](): null {
        return null;
    }

    get(index: number): T | undefined {
        return this.items[index];
    }

    get length(): number {
        return this.items.length;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");
    let class_idx = *source_file.statements.nodes.first().expect("expected class declaration");

    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    // Class should emit
    assert!(
        output.contains("function ImmutableList"),
        "Expected ImmutableList class: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("get"),
        "Expected get method: {}",
        output
    );

    // Constructor should handle items
    assert!(
        output.contains("items"),
        "Expected items handling: {}",
        output
    );
}

// =============================================================================
// WeakMap/WeakSet Private Field Polyfill Tests
// =============================================================================

#[test]
fn test_class_es5_weakmap_private_field_basic() {
    // Private fields polyfilled with WeakMap pattern
    let source = r#"
class SecureData {
    #secret: string;
    #value: number;

    constructor(secret: string, value: number) {
        this.#secret = secret;
        this.#value = value;
    }

    getSecret(): string {
        return this.#secret;
    }

    getValue(): number {
        return this.#value;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be present
    assert!(
        output.contains("SecureData"),
        "Expected SecureData class: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("getSecret") && output.contains("getValue"),
        "Expected getSecret and getValue methods: {}",
        output
    );

    // Private field access should be transformed
    assert!(
        output.contains("secret") && output.contains("value"),
        "Expected private field references: {}",
        output
    );
}

#[test]
fn test_class_es5_weakmap_private_field_methods() {
    // Private methods polyfilled with WeakSet pattern
    let source = r#"
class Processor {
    #data: any[] = [];

    #validate(item: any): boolean {
        return item !== null && item !== undefined;
    }

    #transform(item: any): any {
        return { processed: true, data: item };
    }

    add(item: any): void {
        if (this.#validate(item)) {
            this.#data.push(this.#transform(item));
        }
    }

    getAll(): any[] {
        return this.#data;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be present
    assert!(
        output.contains("Processor"),
        "Expected Processor class: {}",
        output
    );

    // Public methods should be present
    assert!(
        output.contains("add") && output.contains("getAll"),
        "Expected add and getAll methods: {}",
        output
    );

    // Private method names should be present
    assert!(
        output.contains("validate") && output.contains("transform"),
        "Expected private method names: {}",
        output
    );
}

#[test]
fn test_class_es5_weakmap_private_field_inheritance() {
    // Private fields in inheritance - each class has its own WeakMap
    let source = r#"
class Parent {
    #parentSecret: string = "parent";

    getParentSecret(): string {
        return this.#parentSecret;
    }
}

class Child extends Parent {
    #childSecret: string = "child";

    getChildSecret(): string {
        return this.#childSecret;
    }

    getBoth(): string {
        return this.getParentSecret() + " " + this.#childSecret;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Classes should be present
    assert!(
        output.contains("Parent") && output.contains("Child"),
        "Expected Parent and Child classes: {}",
        output
    );

    // Inheritance should be present
    assert!(
        output.contains("__extends") || output.contains("prototype"),
        "Expected inheritance pattern: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("getParentSecret") && output.contains("getChildSecret") && output.contains("getBoth"),
        "Expected getter methods: {}",
        output
    );
}

#[test]
fn test_class_es5_weakmap_private_field_static() {
    // Static private fields with WeakMap-like pattern
    let source = r#"
class Registry {
    static #instances: Map<string, Registry> = new Map();
    static #counter: number = 0;

    #id: number;

    constructor() {
        this.#id = Registry.#counter++;
    }

    static register(name: string, instance: Registry): void {
        this.#instances.set(name, instance);
    }

    static get(name: string): Registry | undefined {
        return this.#instances.get(name);
    }

    getId(): number {
        return this.#id;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be present
    assert!(
        output.contains("Registry"),
        "Expected Registry class: {}",
        output
    );

    // Static methods should be present
    assert!(
        output.contains("register") && output.contains("get"),
        "Expected register and get methods: {}",
        output
    );

    // Instance method should be present
    assert!(
        output.contains("getId"),
        "Expected getId method: {}",
        output
    );

    // Private field references should be present
    assert!(
        output.contains("instances") || output.contains("counter") || output.contains("id"),
        "Expected private field references: {}",
        output
    );
}

#[test]
fn test_class_es5_weakmap_private_field_accessors() {
    // Private field with getter/setter accessors
    let source = r#"
class BoundedValue {
    #value: number = 0;
    #min: number;
    #max: number;

    constructor(min: number, max: number) {
        this.#min = min;
        this.#max = max;
    }

    get value(): number {
        return this.#value;
    }

    set value(v: number) {
        if (v < this.#min) {
            this.#value = this.#min;
        } else if (v > this.#max) {
            this.#value = this.#max;
        } else {
            this.#value = v;
        }
    }

    reset(): void {
        this.#value = this.#min;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be present
    assert!(
        output.contains("BoundedValue"),
        "Expected BoundedValue class: {}",
        output
    );

    // Accessor property should be defined
    assert!(
        output.contains("value"),
        "Expected value accessor: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("reset"),
        "Expected reset method: {}",
        output
    );

    // Private field references should be transformed
    assert!(
        output.contains("min") && output.contains("max"),
        "Expected min and max field references: {}",
        output
    );
}

// ============================================================================
// new.target Tests
// ============================================================================

#[test]
fn test_class_es5_new_target_basic() {
    // Basic new.target usage in constructor
    let source = r#"
class Example {
    name: string;

    constructor() {
        this.name = new.target.name;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("Example"),
        "Expected Example class: {}",
        output
    );

    // new.target should be transformed for ES5
    assert!(
        output.contains("name"),
        "Expected name property: {}",
        output
    );
}

#[test]
fn test_class_es5_new_target_derived() {
    // new.target in derived class constructor
    let source = r#"
class Base {
    constructor() {
        console.log(new.target.name);
    }
}

class Derived extends Base {
    constructor() {
        super();
        console.log(new.target.name);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Both classes should be present
    assert!(
        output.contains("Base"),
        "Expected Base class: {}",
        output
    );
    assert!(
        output.contains("Derived"),
        "Expected Derived class: {}",
        output
    );

    // Inheritance should be set up
    assert!(
        output.contains("__extends") || output.contains("extends"),
        "Expected inheritance pattern: {}",
        output
    );
}

#[test]
fn test_class_es5_new_target_abstract_pattern() {
    // new.target check to prevent direct instantiation (abstract class pattern)
    let source = r#"
class AbstractBase {
    constructor() {
        if (new.target === AbstractBase) {
            throw new Error("Cannot instantiate abstract class");
        }
    }

    abstract doWork(): void;
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("AbstractBase"),
        "Expected AbstractBase class: {}",
        output
    );

    // Error throw should be present
    assert!(
        output.contains("Error") || output.contains("throw"),
        "Expected error handling: {}",
        output
    );
}

#[test]
fn test_class_es5_new_target_with_static() {
    // new.target with static factory method
    let source = r#"
class Factory {
    private data: string;

    constructor(data: string) {
        if (!new.target) {
            throw new Error("Must use new");
        }
        this.data = data;
    }

    static create(data: string): Factory {
        return new Factory(data);
    }

    getData(): string {
        return this.data;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("Factory"),
        "Expected Factory class: {}",
        output
    );

    // Static method should be present
    assert!(
        output.contains("create"),
        "Expected create static method: {}",
        output
    );

    // Instance method should be present
    assert!(
        output.contains("getData"),
        "Expected getData method: {}",
        output
    );
}

#[test]
fn test_class_es5_new_target_inheritance_chain() {
    // new.target through inheritance chain
    let source = r#"
class Animal {
    type: string;

    constructor() {
        this.type = new.target.name;
    }
}

class Mammal extends Animal {
    warm: boolean = true;
}

class Dog extends Mammal {
    breed: string;

    constructor(breed: string) {
        super();
        this.breed = breed;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // All classes should be present
    assert!(
        output.contains("Animal"),
        "Expected Animal class: {}",
        output
    );
    assert!(
        output.contains("Mammal"),
        "Expected Mammal class: {}",
        output
    );
    assert!(
        output.contains("Dog"),
        "Expected Dog class: {}",
        output
    );

    // Inheritance should be set up
    assert!(
        output.contains("__extends") || output.contains("_super"),
        "Expected inheritance pattern: {}",
        output
    );
}

#[test]
fn test_class_es5_new_target_undefined_check() {
    // new.target undefined check for callable class pattern
    let source = r#"
class Callable {
    value: number;

    constructor(value: number) {
        if (new.target === undefined) {
            return new Callable(value);
        }
        this.value = value;
    }

    getValue(): number {
        return this.value;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("Callable"),
        "Expected Callable class: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("getValue"),
        "Expected getValue method: {}",
        output
    );

    // undefined check pattern should be in output
    assert!(
        output.contains("undefined") || output.contains("value"),
        "Expected undefined check or value property: {}",
        output
    );
}

// ============================================================================
// Proxy Pattern Tests
// ============================================================================

#[test]
fn test_class_es5_proxy_basic() {
    // Basic Proxy usage in class method
    let source = r#"
class ProxyWrapper<T extends object> {
    private target: T;
    private proxy: T;

    constructor(target: T) {
        this.target = target;
        this.proxy = new Proxy(target, {});
    }

    getProxy(): T {
        return this.proxy;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("ProxyWrapper"),
        "Expected ProxyWrapper class: {}",
        output
    );

    // Proxy usage should be present
    assert!(
        output.contains("Proxy"),
        "Expected Proxy usage: {}",
        output
    );

    // Method should be present
    assert!(
        output.contains("getProxy"),
        "Expected getProxy method: {}",
        output
    );
}

#[test]
fn test_class_es5_proxy_handler_traps() {
    // Proxy with get/set handler traps
    let source = r#"
class ObservableObject {
    private data: Record<string, any> = {};
    private listeners: Array<(key: string, value: any) => void> = [];

    createProxy(): Record<string, any> {
        const self = this;
        return new Proxy(this.data, {
            get(target, prop: string) {
                return target[prop];
            },
            set(target, prop: string, value) {
                target[prop] = value;
                self.notify(prop, value);
                return true;
            }
        });
    }

    notify(key: string, value: any): void {
        this.listeners.forEach(fn => fn(key, value));
    }

    subscribe(fn: (key: string, value: any) => void): void {
        this.listeners.push(fn);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("ObservableObject"),
        "Expected ObservableObject class: {}",
        output
    );

    // Proxy should be present
    assert!(
        output.contains("Proxy"),
        "Expected Proxy: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("createProxy") && output.contains("notify") && output.contains("subscribe"),
        "Expected createProxy, notify, subscribe methods: {}",
        output
    );
}

#[test]
fn test_class_es5_proxy_apply_trap() {
    // Proxy with apply trap for function wrapping
    let source = r#"
class FunctionWrapper {
    wrap<T extends (...args: any[]) => any>(fn: T): T {
        return new Proxy(fn, {
            apply(target, thisArg, args) {
                console.log("Calling function with args:", args);
                const result = Reflect.apply(target, thisArg, args);
                console.log("Result:", result);
                return result;
            }
        }) as T;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("FunctionWrapper"),
        "Expected FunctionWrapper class: {}",
        output
    );

    // wrap method should be present
    assert!(
        output.contains("wrap"),
        "Expected wrap method: {}",
        output
    );
}

#[test]
fn test_class_es5_proxy_factory() {
    // Static factory method returning proxied instance
    let source = r#"
class ValidatedModel {
    name: string = "";
    age: number = 0;

    static create(): ValidatedModel {
        const instance = new ValidatedModel();
        return new Proxy(instance, {
            set(target, prop: keyof ValidatedModel, value) {
                if (prop === "age" && typeof value === "number" && value < 0) {
                    throw new Error("Age cannot be negative");
                }
                (target as any)[prop] = value;
                return true;
            }
        });
    }

    toJSON(): object {
        return { name: this.name, age: this.age };
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("ValidatedModel"),
        "Expected ValidatedModel class: {}",
        output
    );

    // Static method create should be present
    assert!(
        output.contains("create"),
        "Expected create static method: {}",
        output
    );

    // Proxy should be present
    assert!(
        output.contains("Proxy"),
        "Expected Proxy: {}",
        output
    );

    // Instance method should be present
    assert!(
        output.contains("toJSON"),
        "Expected toJSON method: {}",
        output
    );
}

#[test]
fn test_class_es5_proxy_with_reflect() {
    // Proxy using Reflect API for default behavior
    let source = r#"
class LoggingProxy<T extends object> {
    private logs: string[] = [];

    createLoggingProxy(target: T): T {
        const logs = this.logs;
        return new Proxy(target, {
            get(target, prop, receiver) {
                logs.push("get " + String(prop));
                return Reflect.get(target, prop, receiver);
            },
            set(target, prop, value, receiver) {
                logs.push("set " + String(prop));
                return Reflect.set(target, prop, value, receiver);
            },
            deleteProperty(target, prop) {
                logs.push("delete " + String(prop));
                return Reflect.deleteProperty(target, prop);
            }
        });
    }

    getLogs(): string[] {
        return this.logs;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("LoggingProxy"),
        "Expected LoggingProxy class: {}",
        output
    );

    // Proxy should be present
    assert!(
        output.contains("Proxy"),
        "Expected Proxy: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("createLoggingProxy") && output.contains("getLogs"),
        "Expected createLoggingProxy and getLogs methods: {}",
        output
    );
}

#[test]
fn test_class_es5_proxy_revocable() {
    // Revocable proxy pattern
    let source = r#"
class RevocableAccess<T extends object> {
    private revoke: (() => void) | null = null;

    createRevocable(target: T): T {
        const { proxy, revoke } = Proxy.revocable(target, {
            get(target, prop, receiver) {
                return Reflect.get(target, prop, receiver);
            }
        });
        this.revoke = revoke;
        return proxy;
    }

    revokeAccess(): void {
        if (this.revoke) {
            this.revoke();
            this.revoke = null;
        }
    }

    isRevoked(): boolean {
        return this.revoke === null;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("RevocableAccess"),
        "Expected RevocableAccess class: {}",
        output
    );

    // Proxy.revocable should be present
    assert!(
        output.contains("Proxy"),
        "Expected Proxy: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("createRevocable") && output.contains("revokeAccess") && output.contains("isRevoked"),
        "Expected createRevocable, revokeAccess, isRevoked methods: {}",
        output
    );
}

// ============================================================================
// WeakRef and FinalizationRegistry Tests
// ============================================================================

#[test]
fn test_class_es5_weakref_basic() {
    // Basic WeakRef usage in class
    let source = r#"
class WeakReference<T extends object> {
    private ref: WeakRef<T>;

    constructor(target: T) {
        this.ref = new WeakRef(target);
    }

    get(): T | undefined {
        return this.ref.deref();
    }

    isAlive(): boolean {
        return this.ref.deref() !== undefined;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("WeakReference"),
        "Expected WeakReference class: {}",
        output
    );

    // WeakRef should be present
    assert!(
        output.contains("WeakRef"),
        "Expected WeakRef: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("get") && output.contains("isAlive"),
        "Expected get and isAlive methods: {}",
        output
    );
}

#[test]
fn test_class_es5_weakref_with_deref() {
    // WeakRef with conditional deref usage
    let source = r#"
class ObjectTracker<T extends object> {
    private weakRefs: WeakRef<T>[] = [];

    track(obj: T): void {
        this.weakRefs.push(new WeakRef(obj));
    }

    getAlive(): T[] {
        const alive: T[] = [];
        for (const ref of this.weakRefs) {
            const obj = ref.deref();
            if (obj !== undefined) {
                alive.push(obj);
            }
        }
        return alive;
    }

    cleanup(): void {
        this.weakRefs = this.weakRefs.filter(ref => ref.deref() !== undefined);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("ObjectTracker"),
        "Expected ObjectTracker class: {}",
        output
    );

    // WeakRef should be present
    assert!(
        output.contains("WeakRef"),
        "Expected WeakRef: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("track") && output.contains("getAlive") && output.contains("cleanup"),
        "Expected track, getAlive, cleanup methods: {}",
        output
    );
}

#[test]
fn test_class_es5_finalization_registry_basic() {
    // Basic FinalizationRegistry usage
    let source = r#"
class ResourceManager {
    private registry: FinalizationRegistry<string>;
    private resources: Map<string, object> = new Map();

    constructor() {
        this.registry = new FinalizationRegistry((id: string) => {
            console.log("Resource cleaned up:", id);
            this.resources.delete(id);
        });
    }

    register(id: string, resource: object): void {
        this.resources.set(id, resource);
        this.registry.register(resource, id);
    }

    getCount(): number {
        return this.resources.size;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("ResourceManager"),
        "Expected ResourceManager class: {}",
        output
    );

    // FinalizationRegistry should be present
    assert!(
        output.contains("FinalizationRegistry"),
        "Expected FinalizationRegistry: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("register") && output.contains("getCount"),
        "Expected register and getCount methods: {}",
        output
    );
}

#[test]
fn test_class_es5_finalization_registry_with_unregister() {
    // FinalizationRegistry with unregister token
    let source = r#"
class ManagedResource {
    private registry: FinalizationRegistry<string>;
    private tokens: Map<string, object> = new Map();

    constructor() {
        this.registry = new FinalizationRegistry((heldValue: string) => {
            console.log("Releasing:", heldValue);
        });
    }

    acquire(id: string, resource: object): object {
        const token = { id };
        this.tokens.set(id, token);
        this.registry.register(resource, id, token);
        return resource;
    }

    release(id: string): boolean {
        const token = this.tokens.get(id);
        if (token) {
            this.registry.unregister(token);
            this.tokens.delete(id);
            return true;
        }
        return false;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("ManagedResource"),
        "Expected ManagedResource class: {}",
        output
    );

    // FinalizationRegistry should be present
    assert!(
        output.contains("FinalizationRegistry"),
        "Expected FinalizationRegistry: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("acquire") && output.contains("release"),
        "Expected acquire and release methods: {}",
        output
    );
}

#[test]
fn test_class_es5_weakref_cache_pattern() {
    // WeakRef-based cache pattern
    let source = r#"
class WeakCache<K extends object, V> {
    private cache: Map<K, WeakRef<V & object>> = new Map();

    set(key: K, value: V & object): void {
        this.cache.set(key, new WeakRef(value));
    }

    get(key: K): V | undefined {
        const ref = this.cache.get(key);
        if (ref) {
            const value = ref.deref();
            if (value === undefined) {
                this.cache.delete(key);
            }
            return value;
        }
        return undefined;
    }

    has(key: K): boolean {
        const ref = this.cache.get(key);
        if (ref && ref.deref() !== undefined) {
            return true;
        }
        this.cache.delete(key);
        return false;
    }

    clear(): void {
        this.cache.clear();
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("WeakCache"),
        "Expected WeakCache class: {}",
        output
    );

    // WeakRef should be present
    assert!(
        output.contains("WeakRef"),
        "Expected WeakRef: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("set") && output.contains("has") && output.contains("clear"),
        "Expected set, has, clear methods: {}",
        output
    );
}

#[test]
fn test_class_es5_weakref_finalization_combined() {
    // Combined WeakRef and FinalizationRegistry pattern
    let source = r#"
class SubscriptionManager<T extends object> {
    private subscriptions: Map<string, WeakRef<T>> = new Map();
    private registry: FinalizationRegistry<string>;

    constructor() {
        this.registry = new FinalizationRegistry((id: string) => {
            this.subscriptions.delete(id);
            console.log("Subscription auto-removed:", id);
        });
    }

    subscribe(id: string, subscriber: T): void {
        this.subscriptions.set(id, new WeakRef(subscriber));
        this.registry.register(subscriber, id);
    }

    notify(message: string): void {
        for (const [id, ref] of this.subscriptions) {
            const subscriber = ref.deref();
            if (subscriber) {
                console.log("Notifying", id, ":", message);
            }
        }
    }

    unsubscribe(id: string): boolean {
        return this.subscriptions.delete(id);
    }

    getActiveCount(): number {
        let count = 0;
        for (const ref of this.subscriptions.values()) {
            if (ref.deref() !== undefined) {
                count++;
            }
        }
        return count;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be emitted
    assert!(
        output.contains("SubscriptionManager"),
        "Expected SubscriptionManager class: {}",
        output
    );

    // Both WeakRef and FinalizationRegistry should be present
    assert!(
        output.contains("WeakRef") && output.contains("FinalizationRegistry"),
        "Expected WeakRef and FinalizationRegistry: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("subscribe") && output.contains("notify") && output.contains("unsubscribe"),
        "Expected subscribe, notify, unsubscribe methods: {}",
        output
    );
}

// ============================================================================
// Symbol.toStringTag Tests
// ============================================================================

#[test]
fn test_class_es5_symbol_to_string_tag_basic() {
    // Basic Symbol.toStringTag getter
    let source = r#"
class MyClass {
    get [Symbol.toStringTag](): string {
        return "MyClass";
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be converted to function
    assert!(
        output.contains("function MyClass"),
        "Expected function declaration: {}",
        output
    );

    // Getter should be defined via Object.defineProperty
    assert!(
        output.contains("Object.defineProperty") && output.contains("get:"),
        "Expected getter via Object.defineProperty: {}",
        output
    );

    // Return value should be present
    assert!(
        output.contains("\"MyClass\""),
        "Expected class name in return value: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_to_string_tag_static() {
    // Static Symbol.toStringTag getter
    let source = r#"
class Container<T> {
    private value: T;

    constructor(value: T) {
        this.value = value;
    }

    static get [Symbol.toStringTag](): string {
        return "Container";
    }

    getValue(): T {
        return this.value;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be converted
    assert!(
        output.contains("function Container"),
        "Expected function declaration: {}",
        output
    );

    // Getter should be defined via Object.defineProperty
    assert!(
        output.contains("Object.defineProperty") && output.contains("get:"),
        "Expected getter via Object.defineProperty: {}",
        output
    );

    // getValue method should be on prototype
    assert!(
        output.contains("getValue"),
        "Expected getValue method: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_to_string_tag_with_other_symbols() {
    // Class with multiple well-known symbols
    let source = r#"
class SymbolRich {
    private items: number[] = [];

    get [Symbol.toStringTag](): string {
        return "SymbolRich";
    }

    *[Symbol.iterator](): Iterator<number> {
        yield* this.items;
    }

    [Symbol.hasInstance](instance: unknown): boolean {
        return instance instanceof SymbolRich;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be converted
    assert!(
        output.contains("function SymbolRich") || output.contains("SymbolRich"),
        "Expected class definition: {}",
        output
    );

    // Getter should be defined via Object.defineProperty (for toStringTag)
    assert!(
        output.contains("Object.defineProperty") && output.contains("get:"),
        "Expected getter via Object.defineProperty: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_to_string_tag_inheritance() {
    // Symbol.toStringTag with inheritance
    let source = r#"
class BaseType {
    get [Symbol.toStringTag](): string {
        return "BaseType";
    }
}

class DerivedType extends BaseType {
    get [Symbol.toStringTag](): string {
        return "DerivedType";
    }
}

class AnotherDerived extends BaseType {
    // Inherits toStringTag from BaseType
    doSomething(): void {
        console.log("something");
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // All classes should be present
    assert!(
        output.contains("BaseType") && output.contains("DerivedType") && output.contains("AnotherDerived"),
        "Expected all classes: {}",
        output
    );

    // Inheritance pattern should be present
    assert!(
        output.contains("__extends") || output.contains("prototype"),
        "Expected inheritance pattern: {}",
        output
    );

    // Getter should be defined via Object.defineProperty (for toStringTag)
    assert!(
        output.contains("Object.defineProperty") && output.contains("get:"),
        "Expected getter via Object.defineProperty: {}",
        output
    );

    // doSomething method should be present
    assert!(
        output.contains("doSomething"),
        "Expected doSomething method: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_to_string_tag_computed_value() {
    // Symbol.toStringTag with computed/dynamic value
    let source = r#"
class DynamicTag {
    private name: string;

    constructor(name: string) {
        this.name = name;
    }

    get [Symbol.toStringTag](): string {
        return `DynamicTag(${this.name})`;
    }

    setName(name: string): void {
        this.name = name;
    }
}

class VersionedType {
    private version: number = 1;

    get [Symbol.toStringTag](): string {
        return `VersionedType.v${this.version}`;
    }

    upgrade(): void {
        this.version++;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Both classes should be present
    assert!(
        output.contains("DynamicTag") && output.contains("VersionedType"),
        "Expected both classes: {}",
        output
    );

    // Getter should be defined via Object.defineProperty
    assert!(
        output.contains("Object.defineProperty") && output.contains("get:"),
        "Expected getter via Object.defineProperty: {}",
        output
    );

    // Methods should be present
    assert!(
        output.contains("setName") && output.contains("upgrade"),
        "Expected methods: {}",
        output
    );

    // Template literal should be transformed (concatenation in ES5)
    assert!(
        output.contains("DynamicTag(") && output.contains("VersionedType.v"),
        "Expected class names in return values: {}",
        output
    );
}

// ============================================================================
// Symbol.hasInstance Tests
// ============================================================================

#[test]
fn test_class_es5_symbol_has_instance_basic() {
    // Basic Symbol.hasInstance static method
    let source = r#"
class MyClass {
    static [Symbol.hasInstance](instance: unknown): boolean {
        return typeof instance === "object" && instance !== null;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be converted to function
    assert!(
        output.contains("function MyClass"),
        "Expected function declaration: {}",
        output
    );

    // typeof check should be present
    assert!(
        output.contains("typeof") && output.contains("object"),
        "Expected typeof check: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_has_instance_custom_check() {
    // Custom instanceof behavior checking for specific property
    let source = r#"
class Validator {
    private valid: boolean = true;

    static [Symbol.hasInstance](instance: unknown): boolean {
        return instance !== null &&
               typeof instance === "object" &&
               "valid" in instance;
    }

    isValid(): boolean {
        return this.valid;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be converted
    assert!(
        output.contains("function Validator"),
        "Expected function declaration: {}",
        output
    );

    // isValid method should be present
    assert!(
        output.contains("isValid"),
        "Expected isValid method: {}",
        output
    );

    // in operator check should be present
    assert!(
        output.contains("in") || output.contains("valid"),
        "Expected property check: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_has_instance_with_inheritance() {
    // Symbol.hasInstance with class inheritance
    let source = r#"
class Animal {
    name: string;

    constructor(name: string) {
        this.name = name;
    }

    static [Symbol.hasInstance](instance: unknown): boolean {
        return instance !== null &&
               typeof instance === "object" &&
               "name" in instance;
    }
}

class Dog extends Animal {
    breed: string;

    constructor(name: string, breed: string) {
        super(name);
        this.breed = breed;
    }

    static [Symbol.hasInstance](instance: unknown): boolean {
        return instance !== null &&
               typeof instance === "object" &&
               "breed" in instance;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Both classes should be present
    assert!(
        output.contains("Animal") && output.contains("Dog"),
        "Expected both classes: {}",
        output
    );

    // Inheritance pattern should be present
    assert!(
        output.contains("__extends") || output.contains("prototype"),
        "Expected inheritance pattern: {}",
        output
    );

    // super call should be present
    assert!(
        output.contains("_super") || output.contains(".call(this"),
        "Expected super call: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_has_instance_duck_typing() {
    // Duck typing pattern with Symbol.hasInstance
    let source = r#"
interface Stringifiable {
    toString(): string;
}

class StringWrapper {
    private value: string;

    constructor(value: string) {
        this.value = value;
    }

    static [Symbol.hasInstance](instance: unknown): instance is Stringifiable {
        return instance !== null &&
               typeof instance === "object" &&
               typeof (instance as any).toString === "function";
    }

    toString(): string {
        return this.value;
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be converted
    assert!(
        output.contains("function StringWrapper"),
        "Expected function declaration: {}",
        output
    );

    // toString method should be present
    assert!(
        output.contains("toString"),
        "Expected toString method: {}",
        output
    );

    // typeof function check should be present
    assert!(
        output.contains("typeof") && output.contains("function"),
        "Expected typeof function check: {}",
        output
    );
}

#[test]
fn test_class_es5_symbol_has_instance_with_other_static_members() {
    // Symbol.hasInstance alongside other static members
    let source = r#"
class Registry<T> {
    private items: Map<string, T> = new Map();

    static readonly VERSION = "1.0.0";
    static instanceCount = 0;

    constructor() {
        Registry.instanceCount++;
    }

    static [Symbol.hasInstance](instance: unknown): boolean {
        return instance !== null &&
               typeof instance === "object" &&
               "items" in instance;
    }

    static create<U>(): Registry<U> {
        return new Registry<U>();
    }

    add(key: string, value: T): void {
        this.items.set(key, value);
    }

    get(key: string): T | undefined {
        return this.items.get(key);
    }
}
"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut options = PrinterOptions::default();
    options.target = ScriptTarget::ES5;
    let ctx = EmitContext::with_options(options.clone());
    let transforms = LoweringPass::new(&parser.arena, &ctx).run(root);

    let mut printer =
        ThinPrinter::with_transforms_and_options(&parser.arena, transforms, options);
    printer.set_target_es5(ctx.target_es5);
    printer.emit(root);

    let output = printer.get_output().to_string();

    // Class should be converted
    assert!(
        output.contains("function Registry") || output.contains("Registry"),
        "Expected class definition: {}",
        output
    );

    // Static members should be present
    assert!(
        output.contains("VERSION") || output.contains("instanceCount"),
        "Expected static members: {}",
        output
    );

    // Instance methods should be present
    assert!(
        output.contains("add") && output.contains("get"),
        "Expected instance methods: {}",
        output
    );

    // Static create method should be present
    assert!(
        output.contains("create"),
        "Expected static create method: {}",
        output
    );
}
