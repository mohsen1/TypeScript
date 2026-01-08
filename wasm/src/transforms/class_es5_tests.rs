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
fn test_class_es5_default_constructor_arrow_this_capture() {
    let source = "class Foo { bar = () => this.x; }";
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
        output.contains("var _this = this;"),
        "Expected this capture in default constructor: {}",
        output
    );
    assert!(
        output.contains("this.bar = function"),
        "Expected arrow to be emitted as function: {}",
        output
    );
    assert!(
        output.contains("_this.x"),
        "Expected arrow body to use _this: {}",
        output
    );
}

#[test]
fn test_class_es5_derived_constructor_preserves_pre_super_statements() {
    let source = r#"class Base {}
class Derived extends Base {
    foo = 1;
    constructor() {
        console.log("before");
        super();
    }
}"#;
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let root_node = parser.arena.get(root).expect("expected source file node");
    let source_file = parser
        .arena
        .get_source_file(root_node)
        .expect("expected source file data");

    let mut derived_idx = None;
    for &stmt_idx in &source_file.statements.nodes {
        let Some(node) = parser.arena.get(stmt_idx) else { continue };
        let Some(class) = parser.arena.get_class(node) else { continue };
        if class.name.is_none() {
            continue;
        }
        let Some(name_node) = parser.arena.get(class.name) else { continue };
        let Some(ident) = parser.arena.get_identifier(name_node) else { continue };
        if ident.escaped_text == "Derived" {
            derived_idx = Some(stmt_idx);
            break;
        }
    }

    let class_idx = derived_idx.expect("expected Derived class declaration");
    let mut emitter = ClassES5Emitter::new(&parser.arena);
    let output = emitter.emit_class(class_idx);

    let log_pos = output
        .find("console.log(\"before\")")
        .expect("expected pre-super statement");
    let super_pos = output
        .find("_super.call(this")
        .expect("expected transformed super call");
    let init_pos = output
        .find("_this.foo = 1")
        .expect("expected property initializer");

    assert!(
        log_pos < super_pos,
        "Expected pre-super statement before super call: {}",
        output
    );
    assert!(
        super_pos < init_pos,
        "Expected property initializer after super call: {}",
        output
    );
}
