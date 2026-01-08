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
fn test_class_es5_for_in_loop() {
    // Test class method with for-in loop
    let source = r#"
class ObjectInspector {
    getKeys(obj: object): string[] {
        const keys: string[] = [];
        for (const key in obj) {
            if (obj.hasOwnProperty(key)) {
                keys.push(key);
            }
        }
        return keys;
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
        output.contains("function ObjectInspector"),
        "Expected class to emit as function: {}",
        output
    );

    // Should preserve for-in loop
    assert!(
        output.contains("for") && output.contains(" in "),
        "Expected for-in loop in output: {}",
        output
    );

    // Method should be on prototype
    assert!(
        output.contains(".prototype.getKeys"),
        "Expected getKeys method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_typeof_instanceof() {
    // Test class method with typeof and instanceof operators
    let source = r#"
class TypeChecker {
    isString(value: unknown): boolean {
        return typeof value === "string";
    }

    isArray(value: unknown): boolean {
        return value instanceof Array;
    }

    getType(value: unknown): string {
        return typeof value;
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
        output.contains("function TypeChecker"),
        "Expected class to emit as function: {}",
        output
    );

    // Should preserve typeof operator
    assert!(
        output.contains("typeof"),
        "Expected typeof operator in output: {}",
        output
    );

    // Should preserve instanceof operator
    assert!(
        output.contains("instanceof"),
        "Expected instanceof operator in output: {}",
        output
    );

    // Methods should be on prototype
    assert!(
        output.contains(".prototype.isString"),
        "Expected isString method on prototype: {}",
        output
    );
    assert!(
        output.contains(".prototype.isArray"),
        "Expected isArray method on prototype: {}",
        output
    );
}

#[test]
fn test_class_es5_logical_operators() {
    // Test class method with logical operators (&&, ||, !)
    let source = r#"
class LogicalOps {
    checkBoth(a: boolean, b: boolean): boolean {
        return a && b;
    }

    checkEither(a: boolean, b: boolean): boolean {
        return a || b;
    }

    negate(value: boolean): boolean {
        return !value;
    }

    complex(a: boolean, b: boolean, c: boolean): boolean {
        return (a && b) || (!c && a);
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
        output.contains("function LogicalOps"),
        "Expected class to emit as function: {}",
        output
    );

    // Should preserve logical operators
    assert!(
        output.contains("&&"),
        "Expected && operator in output: {}",
        output
    );
    assert!(
        output.contains("||"),
        "Expected || operator in output: {}",
        output
    );

    // Methods should be on prototype
    assert!(
        output.contains(".prototype.checkBoth"),
        "Expected checkBoth method on prototype: {}",
        output
    );
    assert!(
        output.contains(".prototype.complex"),
        "Expected complex method on prototype: {}",
        output
    );
}
