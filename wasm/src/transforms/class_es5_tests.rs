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
