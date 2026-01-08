//! Tests for ThinChecker - Type checker using ThinNodeArena and Solver

use crate::thin_checker::ThinCheckerState;
use crate::parser::thin_node::ThinNodeArena;
use crate::thin_binder::ThinBinderState;
use crate::solver::{TypeId, TypeInterner};

#[test]
fn test_thin_checker_creation() {
    let arena = ThinNodeArena::new();
    let binder = ThinBinderState::new();
    let types = TypeInterner::new();
    let checker = ThinCheckerState::new(&arena, &binder, &types, "test.ts".to_string());

    // Basic sanity check
    assert!(checker.ctx.diagnostics.is_empty());
}

#[test]
fn test_thin_checker_basic_types() {
    let arena = ThinNodeArena::new();
    let binder = ThinBinderState::new();
    let types = TypeInterner::new();
    let _checker = ThinCheckerState::new(&arena, &binder, &types, "test.ts".to_string());

    // Verify intrinsic TypeIds are constants (compile-time values)
    assert_eq!(TypeId::NUMBER.0, 9);
    assert_eq!(TypeId::STRING.0, 10);
    assert_eq!(TypeId::BOOLEAN.0, 8);
    assert_eq!(TypeId::ANY.0, 4);
    assert_eq!(TypeId::NEVER.0, 2);
}

#[test]
fn test_thin_checker_type_interner() {
    let arena = ThinNodeArena::new();
    let binder = ThinBinderState::new();
    let types = TypeInterner::new();
    let checker = ThinCheckerState::new(&arena, &binder, &types, "test.ts".to_string());

    // Test that TypeInterner is properly initialized
    // Intrinsics should be pre-registered
    assert!(checker.ctx.types.lookup(TypeId::STRING).is_some());
    assert!(checker.ctx.types.lookup(TypeId::NUMBER).is_some());
    assert!(checker.ctx.types.lookup(TypeId::ANY).is_some());
}

#[test]
fn test_thin_checker_structural_equality() {
    let arena = ThinNodeArena::new();
    let binder = ThinBinderState::new();
    let types = TypeInterner::new();
    let checker = ThinCheckerState::new(&arena, &binder, &types, "test.ts".to_string());

    // Test structural equality via TypeInterner
    // Same string literal should get same TypeId
    let str1 = checker.ctx.types.literal_string("hello");
    let str2 = checker.ctx.types.literal_string("hello");
    let str3 = checker.ctx.types.literal_string("world");

    assert_eq!(str1, str2); // Same structure = same TypeId
    assert_ne!(str1, str3); // Different structure = different TypeId
}

#[test]
fn test_thin_checker_union_normalization() {
    let arena = ThinNodeArena::new();
    let binder = ThinBinderState::new();
    let types = TypeInterner::new();
    let checker = ThinCheckerState::new(&arena, &binder, &types, "test.ts".to_string());

    // Test union normalization
    // Union with `any` should be `any`
    let with_any = checker.ctx.types.union(vec![TypeId::STRING, TypeId::ANY]);
    assert_eq!(with_any, TypeId::ANY);

    // Union with `never` should exclude `never`
    let with_never = checker.ctx.types.union(vec![TypeId::STRING, TypeId::NEVER]);
    assert_eq!(with_never, TypeId::STRING);

    // Single-element union should return the element
    let single = checker.ctx.types.union(vec![TypeId::STRING]);
    assert_eq!(single, TypeId::STRING);
}

#[test]
fn test_excess_property_in_variable_declaration() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
type Foo = { x: number };
const ok: Foo = { x: 1 };
const bad: Foo = { x: 1, y: 2 };
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    assert!(parser.get_diagnostics().is_empty(), "Parse errors: {:?}", parser.get_diagnostics());

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let excess_count = codes.iter().filter(|&&code| code == 2353).count();
    assert_eq!(excess_count, 1,
        "Expected exactly one error 2353 (Excess property), got codes: {:?}", codes);
}

#[test]
fn test_excess_property_in_call_argument() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
type Foo = { x: number };
function takesFoo(arg: Foo) {}
takesFoo({ x: 1, y: 2 });
const obj = { x: 1, y: 2 };
takesFoo(obj);
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let excess_count = codes.iter().filter(|&&code| code == 2353).count();
    assert_eq!(excess_count, 1,
        "Expected exactly one error 2353 (Excess property), got codes: {:?}", codes);
}

#[test]
fn test_array_literal_best_common_type() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
const numbers = [1, 2];
const mixed = [1, "a"];
numbers;
mixed;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();
    assert!(parser.get_diagnostics().is_empty(), "Parse errors: {:?}", parser.get_diagnostics());

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let expr_stmts: Vec<_> = source_file
        .statements
        .nodes
        .iter()
        .copied()
        .filter(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .collect();
    assert_eq!(expr_stmts.len(), 2, "Expected two expression statements");

    let numbers_expr = arena
        .get_expression_statement(arena.get(expr_stmts[0]).expect("numbers expr node"))
        .expect("numbers expr");
    let mixed_expr = arena
        .get_expression_statement(arena.get(expr_stmts[1]).expect("mixed expr node"))
        .expect("mixed expr");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let numbers_type = checker.get_type_of_node(numbers_expr.expression);
    let mixed_type = checker.get_type_of_node(mixed_expr.expression);

    let number_array = checker.ctx.types.array(TypeId::NUMBER);
    let number_or_string = checker.ctx.types.union(vec![TypeId::NUMBER, TypeId::STRING]);
    let mixed_array = checker.ctx.types.array(number_or_string);

    assert_eq!(numbers_type, number_array);
    assert_eq!(mixed_type, mixed_array);
}

#[test]
fn test_thin_checker_resolves_function_parameter_from_bound_state() {
    use crate::binder::SymbolTable;
    use crate::checker::types::diagnostics::diagnostic_codes;
    use crate::parallel;

    let source = r#"
export function f(node: { body: number }) {
    if (node.body) {
        return node.body;
    }
    return node.body;
}
"#;

    let program = parallel::compile_files(vec![("test.ts".to_string(), source.to_string())]);
    let file = &program.files[0];

    let mut file_locals = SymbolTable::new();
    for (name, &sym_id) in program.file_locals[0].iter() {
        file_locals.set(name.clone(), sym_id);
    }
    for (name, &sym_id) in program.globals.iter() {
        if !file_locals.has(name) {
            file_locals.set(name.clone(), sym_id);
        }
    }

    let binder = ThinBinderState::from_bound_state_with_scopes(
        program.symbols.clone(),
        file_locals,
        file.node_symbols.clone(),
        file.scopes.clone(),
        file.node_scope_ids.clone(),
    );

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(&file.arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(file.source_file);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        !codes.contains(&diagnostic_codes::CANNOT_FIND_NAME),
        "Unexpected 'Cannot find name' diagnostics: {:?}",
        checker.ctx.diagnostics
    );
}

#[test]
fn test_excess_property_in_return_statement() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
type Foo = { x: number };
function makeFoo(): Foo {
    return { x: 1, y: 2 };
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let excess_count = codes.iter().filter(|&&code| code == 2353).count();
    assert_eq!(excess_count, 1,
        "Expected exactly one error 2353 (Excess property), got codes: {:?}", codes);
}

#[test]
fn test_thin_checker_subtype_intrinsics() {
    let arena = ThinNodeArena::new();
    let binder = ThinBinderState::new();
    let types = TypeInterner::new();
    let checker = ThinCheckerState::new(&arena, &binder, &types, "test.ts".to_string());

    // Test intrinsic subtype relations
    // Any is assignable to everything
    assert!(checker.is_assignable_to(TypeId::ANY, TypeId::STRING));
    assert!(checker.is_assignable_to(TypeId::ANY, TypeId::NUMBER));

    // Everything is assignable to any
    assert!(checker.is_assignable_to(TypeId::STRING, TypeId::ANY));
    assert!(checker.is_assignable_to(TypeId::NUMBER, TypeId::ANY));

    // Everything is assignable to unknown
    assert!(checker.is_assignable_to(TypeId::STRING, TypeId::UNKNOWN));
    assert!(checker.is_assignable_to(TypeId::NUMBER, TypeId::UNKNOWN));

    // Never is assignable to everything
    assert!(checker.is_assignable_to(TypeId::NEVER, TypeId::STRING));
    assert!(checker.is_assignable_to(TypeId::NEVER, TypeId::NUMBER));

    // Nothing is assignable to never (except never)
    assert!(!checker.is_assignable_to(TypeId::STRING, TypeId::NEVER));
    assert!(!checker.is_assignable_to(TypeId::NUMBER, TypeId::NEVER));
    assert!(checker.is_assignable_to(TypeId::NEVER, TypeId::NEVER));
}

#[test]
fn test_thin_checker_subtype_literals() {
    let arena = ThinNodeArena::new();
    let binder = ThinBinderState::new();
    let types = TypeInterner::new();
    let checker = ThinCheckerState::new(&arena, &binder, &types, "test.ts".to_string());

    // String literal is subtype of string
    let hello = checker.ctx.types.literal_string("hello");
    assert!(checker.is_assignable_to(hello, TypeId::STRING));

    // Number literal is subtype of number
    let forty_two = checker.ctx.types.literal_number(42.0);
    assert!(checker.is_assignable_to(forty_two, TypeId::NUMBER));

    // Boolean literal is subtype of boolean
    let t = checker.ctx.types.literal_boolean(true);
    assert!(checker.is_assignable_to(t, TypeId::BOOLEAN));

    // String literal is NOT assignable to number
    assert!(!checker.is_assignable_to(hello, TypeId::NUMBER));
}

#[test]
fn test_thin_checker_subtype_unions() {
    let arena = ThinNodeArena::new();
    let binder = ThinBinderState::new();
    let types = TypeInterner::new();
    let checker = ThinCheckerState::new(&arena, &binder, &types, "test.ts".to_string());

    // Create string | number union
    let string_or_number = checker.get_union_type(vec![TypeId::STRING, TypeId::NUMBER]);

    // String is assignable to string | number
    assert!(checker.is_assignable_to(TypeId::STRING, string_or_number));
    assert!(checker.is_assignable_to(TypeId::NUMBER, string_or_number));

    // Boolean is NOT assignable to string | number
    assert!(!checker.is_assignable_to(TypeId::BOOLEAN, string_or_number));

    // string | number is assignable to string | number | boolean
    let three_types = checker.get_union_type(vec![TypeId::STRING, TypeId::NUMBER, TypeId::BOOLEAN]);
    assert!(checker.is_assignable_to(string_or_number, three_types));
}

#[test]
fn test_thin_checker_type_identity() {
    let arena = ThinNodeArena::new();
    let binder = ThinBinderState::new();
    let types = TypeInterner::new();
    let checker = ThinCheckerState::new(&arena, &binder, &types, "test.ts".to_string());

    // Same type is identical to itself
    assert!(checker.are_types_identical(TypeId::STRING, TypeId::STRING));
    assert!(checker.are_types_identical(TypeId::NUMBER, TypeId::NUMBER));

    // Different types are not identical
    assert!(!checker.are_types_identical(TypeId::STRING, TypeId::NUMBER));

    // Same literal values produce identical types (via interning)
    let lit1 = checker.ctx.types.literal_string("test");
    let lit2 = checker.ctx.types.literal_string("test");
    assert!(checker.are_types_identical(lit1, lit2));
}

// ============== Function overload validation ==============

#[test]
fn test_function_overload_missing_implementation_2391() {
    use crate::thin_parser::ThinParserState;
    let source = r#"function foo();"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2391),
        "Expected error 2391 (Function implementation is missing), got: {:?}", codes);
}

#[test]
fn test_function_overload_with_implementation() {
    use crate::thin_parser::ThinParserState;
    let source = r#"
function foo(): void;
function foo() {}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(!codes.contains(&2391),
        "Should not have error 2391 when implementation exists, got: {:?}", codes);
}

#[test]
fn test_function_overload_wrong_name_2389() {
    use crate::thin_parser::ThinParserState;
    let source = r#"
function foo(): void;
function bar() {}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2389) || codes.contains(&2391),
        "Expected error 2389 or 2391 for wrong implementation name, got: {:?}", codes);
}

#[test]
fn test_overload_call_reports_no_overload_matches() {
    use crate::checker::types::diagnostics::diagnostic_codes;
    use crate::thin_parser::ThinParserState;

    let source = r#"
function f(x: string): void;
function f(x: number, y: number): void;
function f(x: any, y?: any) {}
f(true);
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&diagnostic_codes::NO_OVERLOAD_MATCHES_CALL),
        "Expected error 2769 for overload call mismatch, got: {:?}",
        codes
    );
}

#[test]
fn test_class_method_overload_reports_no_overload_matches() {
    use crate::checker::types::diagnostics::diagnostic_codes;
    use crate::thin_parser::ThinParserState;

    let source = r#"
class C {
    foo(x: string): void;
    foo(x: number): void;
    foo(x: any) {}
}
const c = new C();
c.foo(true);
c.foo("ok");
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let count_2769 = codes
        .iter()
        .filter(|&&c| c == diagnostic_codes::NO_OVERLOAD_MATCHES_CALL)
        .count();
    assert_eq!(
        count_2769,
        1,
        "Expected exactly one overload mismatch (2769), got: {:?}",
        codes
    );
}

#[test]
fn test_new_expression_infers_class_instance_type() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
class Foo {
    name: string;
    count = 1;
    readonly tag: string = "x";
    greet(msg: string): number { return 1; }
}
const f = new Foo();
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    eprintln!("=== debug Box ===");
    if let Some(box_sym) = binder.file_locals.get("Box") {
        let box_type = checker.get_type_of_symbol(box_sym);
        eprintln!("Box type id: {:?}", box_type);
        eprintln!("Box type key: {:?}", types.lookup(box_type));
    } else {
        eprintln!("Box symbol missing");
    }

    let f_sym = binder.file_locals.get("f").expect("f should exist");
    let f_type = checker.get_type_of_symbol(f_sym);
    let f_key = types.lookup(f_type).expect("f type should exist");
    match f_key {
        TypeKey::Object(shape_id) => {
            let shape = types.object_shape(shape_id);
            let props = shape.properties.as_slice();
            let name_atom = types.intern_string("name");
            let count_atom = types.intern_string("count");
            let tag_atom = types.intern_string("tag");
            let greet_atom = types.intern_string("greet");

            assert!(
                props.iter().any(|p| p.name == name_atom && p.type_id == TypeId::STRING),
                "Expected name: string in class instance properties, got: {:?}",
                props
            );
            assert!(
                props.iter().any(|p| p.name == count_atom && p.type_id == TypeId::NUMBER),
                "Expected count: number in class instance properties, got: {:?}",
                props
            );
            let tag_prop = props.iter().find(|p| p.name == tag_atom)
                .expect("tag property should exist");
            assert!(tag_prop.readonly, "Expected tag to be readonly");
            assert!(
                props.iter().any(|p| p.name == greet_atom && p.is_method),
                "Expected greet method in class instance properties, got: {:?}",
                props
            );
        }
        _ => panic!("Expected f to be Object type, got {:?}", f_key),
    }
}

#[test]
fn test_new_expression_infers_parameter_properties() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
class Foo {
    constructor(public id: number, readonly tag: string, count: number) {}
}
const f = new Foo(1, "x", 2);
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let f_sym = binder.file_locals.get("f").expect("f should exist");
    let f_type = checker.get_type_of_symbol(f_sym);
    let f_key = types.lookup(f_type).expect("f type should exist");
    match f_key {
        TypeKey::Object(shape_id) => {
            let shape = types.object_shape(shape_id);
            let props = shape.properties.as_slice();
            let id_atom = types.intern_string("id");
            let tag_atom = types.intern_string("tag");
            let count_atom = types.intern_string("count");

            assert!(
                props.iter().any(|p| p.name == id_atom && p.type_id == TypeId::NUMBER),
                "Expected id: number in class instance properties, got: {:?}",
                props
            );
            let tag_prop = props.iter().find(|p| p.name == tag_atom)
                .expect("tag property should exist");
            assert_eq!(tag_prop.type_id, TypeId::STRING);
            assert!(tag_prop.readonly, "Expected tag to be readonly");
            assert!(
                !props.iter().any(|p| p.name == count_atom),
                "Expected count to be absent from class instance properties, got: {:?}",
                props
            );
        }
        _ => panic!("Expected f to be Object type, got {:?}", f_key),
    }
}

#[test]
fn test_new_expression_infers_base_class_properties() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
class Base<T> {
    value: T;
}
class Derived extends Base<string> {
    count = 1;
}
const d = new Derived();
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let d_sym = binder.file_locals.get("d").expect("d should exist");
    let d_type = checker.get_type_of_symbol(d_sym);
    let d_key = types.lookup(d_type).expect("d type should exist");
    match d_key {
        TypeKey::Object(shape_id) => {
            let shape = types.object_shape(shape_id);
            let props = shape.properties.as_slice();
            let value_atom = types.intern_string("value");
            let count_atom = types.intern_string("count");
            let value_prop = props
                .iter()
                .find(|p| p.name == value_atom)
                .expect("value property should exist");
            assert_eq!(value_prop.type_id, TypeId::STRING);
            assert!(
                props.iter().any(|p| p.name == count_atom && p.type_id == TypeId::NUMBER),
                "Expected count: number in class instance properties, got: {:?}",
                props
            );
        }
        _ => panic!("Expected d to be Object type, got {:?}", d_key),
    }
}

#[test]
fn test_new_expression_infers_generic_class_type_params() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
class Box<T> {
    value: T;
    constructor(value: T) {
        this.value = value;
    }
}
const b = new Box("hi");
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let b_sym = binder.file_locals.get("b").expect("b should exist");
    let b_type = checker.get_type_of_symbol(b_sym);
    let b_key = types.lookup(b_type).expect("b type should exist");
    match b_key {
        TypeKey::Object(shape_id) => {
            let shape = types.object_shape(shape_id);
            let props = shape.properties.as_slice();
            let value_atom = types.intern_string("value");
            let value_prop = props
                .iter()
                .find(|p| p.name == value_atom)
                .expect("value property should exist");
            assert_eq!(value_prop.type_id, TypeId::STRING);
        }
        _ => panic!("Expected b to be Object type, got {:?}", b_key),
    }
}

#[test]
fn test_new_expression_reports_overload_mismatch() {
    use crate::checker::types::diagnostics::diagnostic_codes;
    use crate::thin_parser::ThinParserState;

    let source = r#"
class Foo {
    constructor(x: string);
    constructor(x: number, y: number);
    constructor(x: any, y?: any) {}
}
new Foo(true);
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&diagnostic_codes::NO_OVERLOAD_MATCHES_CALL),
        "Expected error 2769 for constructor overload mismatch, got: {:?}",
        codes
    );
}

#[test]
fn test_parameter_property_in_function_2369() {
    use crate::thin_parser::ThinParserState;
    // Parameter properties (public/private/protected/readonly on params)
    // are only allowed in constructor implementations
    let source = r#"function F(public x: string) { }"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2369),
        "Expected error 2369 for parameter property in function, got: {:?}", codes);
}

#[test]
fn test_parameter_property_in_arrow_2369() {
    use crate::thin_parser::ThinParserState;
    let source = r#"var v = (public x: string) => { };"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2369),
        "Expected error 2369 for parameter property in arrow function, got: {:?}", codes);
}

#[test]
fn test_parameter_property_in_constructor_overload_2369() {
    use crate::thin_parser::ThinParserState;
    // Constructor overload signatures should error on parameter properties
    let source = r#"
class C {
    constructor(public p1: string);
    constructor(public p2: number) {}
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    // Should have exactly one 2369 error for the overload, not for the implementation
    let count_2369 = codes.iter().filter(|&&c| c == 2369).count();
    assert_eq!(count_2369, 1,
        "Expected exactly 1 error 2369 for constructor overload, got {} from: {:?}", count_2369, codes);
}

#[test]
fn test_parameter_property_in_constructor_implementation_ok() {
    use crate::thin_parser::ThinParserState;
    // Constructor implementations are allowed to have parameter properties
    let source = r#"
class C {
    constructor(public x: string) {}
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(!codes.contains(&2369),
        "Should not have error 2369 in constructor implementation, got: {:?}", codes);
}

#[test]
fn test_class_name_any_error_2414() {
    use crate::thin_parser::ThinParserState;

    // Test that class name 'any' produces error 2414
    let code = "class any {}";
    let mut parser = ThinParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2414),
        "Expected error 2414 (Class name cannot be 'any'), got: {:?}", codes);
}

#[test]
fn test_local_variable_scope_resolution() {
    use crate::thin_parser::ThinParserState;

    // Test that local variables inside functions are properly resolved
    // This should NOT produce "Cannot find name 'x'" error
    let code = r#"
        function test() {
            let x: number = 1;
            let y = x + 1;
        }
    "#;
    let mut parser = ThinParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    // Should have no "Cannot find name" errors (2304)
    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(!codes.contains(&2304),
        "Should not have 'Cannot find name' error for local variable, got: {:?}", codes);
}

#[test]
fn test_for_loop_variable_scope() {
    use crate::thin_parser::ThinParserState;

    // Test that for loop variables are properly scoped
    let code = r#"
        function test() {
            for (let i = 0; i < 10; i++) {
                let x = i * 2;
            }
        }
    "#;
    let mut parser = ThinParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    // Should have no "Cannot find name" errors (2304) for loop variable 'i'
    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(!codes.contains(&2304),
        "Should not have 'Cannot find name' error for loop variable, got: {:?}", codes);
}

#[test]
fn test_abstract_class_in_local_scope_2511() {
    use crate::thin_parser::ThinParserState;
    use crate::binder::symbol_flags;

    // Test case from tests/cases/compiler/abstractClassInLocalScopeIsAbstract.ts
    // Abstract class declared inside an IIFE should still error on instantiation
    let code = r#"
        (() => {
            abstract class A {}
            class B extends A {}
            new A();
            new B();
        })()
    "#;

    let mut parser = ThinParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    // Debug: Check symbols
    let symbols = binder.get_symbols();
    eprintln!("=== Symbols ===");
    for i in 0..symbols.len() {
        if let Some(sym) = symbols.get(crate::binder::SymbolId(i as u32)) {
            eprintln!("  {:?}: {} flags={:#x} abstract={}", sym.id, sym.escaped_name, sym.flags, sym.flags & symbol_flags::ABSTRACT != 0);
        }
    }

    // Also try manually checking new expression
    eprintln!("=== Class name lookup test ===");
    if let Some(sym_id) = binder.get_symbols().find_by_name("A") {
        eprintln!("Found symbol A: {:?}", sym_id);
        if let Some(symbol) = binder.get_symbol(sym_id) {
            eprintln!("  flags={:#x} abstract={}", symbol.flags, symbol.flags & symbol_flags::ABSTRACT != 0);
        }
    } else {
        eprintln!("Symbol A not found!");
    }

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    // Debug: Check diagnostics
    eprintln!("=== Diagnostics ===");
    for d in &checker.ctx.diagnostics {
        eprintln!("  code={}, msg={}", d.code, d.message_text);
    }

    // Should have error 2511 for `new A()` but not for `new B()`
    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2511),
        "Expected error 2511 for abstract class instantiation in local scope, got: {:?}", codes);

    // Should only have one 2511 error (for A, not B)
    let count_2511 = codes.iter().filter(|&&c| c == 2511).count();
    assert_eq!(count_2511, 1,
        "Expected exactly 1 error 2511 (for abstract class A only), got {} from: {:?}", count_2511, codes);

    // Should NOT have error 2304 (Cannot find name) - both A and B should be found
    let count_2304 = codes.iter().filter(|&&c| c == 2304).count();
    assert_eq!(count_2304, 0,
        "Should NOT have 'Cannot find name' error (2304) for classes in local scope, got {} from: {:?}", count_2304, codes);
}

#[test]
fn test_static_member_suggestion_2662() {
    // Error 2662: Cannot find name 'foo'. Did you mean the static member 'C.foo'?
    use crate::thin_parser::ThinParserState;
    let source = r#"
class C {
    static foo: string;

    bar() {
        let k = foo;
    }
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    // Debug: show all diagnostics
    eprintln!("=== Diagnostics for static member suggestion ===");
    for d in &checker.ctx.diagnostics {
        eprintln!("  code={}, msg={}", d.code, d.message_text);
    }

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2662),
        "Expected error 2662 (Cannot find name 'foo'. Did you mean the static member 'C.foo'?), got: {:?}", codes);

    // Should NOT have generic "cannot find name" error 2304
    assert!(!codes.contains(&2304),
        "Should not have generic error 2304, should have specific 2662 instead. Got: {:?}", codes);
}

#[test]
fn test_abstract_property_in_constructor_2715() {
    // Error 2715: Abstract property 'prop' in class 'AbstractClass' cannot be accessed in the constructor.
    use crate::thin_parser::ThinParserState;

    let source = r#"
abstract class AbstractClass {
    constructor(str: string) {
        let val = this.prop.toLowerCase();
    }

    abstract prop: string;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2715),
        "Expected error 2715 (Abstract property cannot be accessed in constructor), got: {:?}", codes);
}

#[test]
fn test_interface_name_cannot_be_reserved_2427() {
    // Error 2427: Interface name cannot be 'string' (or other primitive types)
    use crate::thin_parser::ThinParserState;
    let source = r#"interface string {}"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    // Debug: show all diagnostics
    eprintln!("=== Diagnostics for 'interface string {{}}' ===");
    for d in &checker.ctx.diagnostics {
        eprintln!("  code={}, msg={}", d.code, d.message_text);
    }

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2427),
        "Expected error 2427 (Interface name cannot be 'string'), got: {:?}", codes);
}

#[test]
fn test_const_modifier_on_class_property_1248() {
    // Error 1248: A class member cannot have the 'const' keyword
    use crate::thin_parser::ThinParserState;
    let source = r#"class AtomicNumbers { static const H = 1; }"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    // Debug: show all diagnostics
    eprintln!("=== Diagnostics for 'static const H = 1' ===");
    for d in &checker.ctx.diagnostics {
        eprintln!("  code={}, msg={}", d.code, d.message_text);
    }

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&1248),
        "Expected error 1248 (A class member cannot have the 'const' keyword), got: {:?}", codes);
}

#[test]
fn test_accessor_type_compatibility_2322() {
    // Error 2322: Type 'string' is not assignable to type 'number'
    // When getter returns string but setter expects number
    use crate::thin_parser::ThinParserState;
    let source = r#"class C {
    public set AnnotatedSetter(a: number) { }
    public get AnnotatedSetter() { return ""; }
}"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    // Debug: show all diagnostics
    eprintln!("=== Diagnostics for accessor type mismatch ===");
    for d in &checker.ctx.diagnostics {
        eprintln!("  code={}, msg={}", d.code, d.message_text);
    }

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2322),
        "Expected error 2322 (Type not assignable), got codes: {:?} diagnostics: {:?}",
        codes,
        checker.ctx.diagnostics.iter().map(|d| (d.code, d.message_text.clone())).collect::<Vec<_>>());
}

#[test]
fn test_abstract_class_through_type_alias_2511() {
    // Error 2511: Cannot create an instance of an abstract class - through type alias
    use crate::thin_parser::ThinParserState;

    let source = r#"
abstract class AbstractA { a: string; }
type Abstracts = typeof AbstractA;
declare const cls2: Abstracts;
new cls2();
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());

    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2511),
        "Expected error 2511 for abstract class instantiation through type alias, got: {:?}", codes);
}

#[test]
fn test_abstract_class_union_type_2511() {
    // Error 2511: Cannot create an instance of an abstract class - through union type
    use crate::thin_parser::ThinParserState;

    let source = r#"
class ConcreteA {}
abstract class AbstractA { a: string; }

type ConcretesOrAbstracts = typeof ConcreteA | typeof AbstractA;

declare const cls1: ConcretesOrAbstracts;

new cls1();
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());

    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2511),
        "Expected error 2511 for abstract class in union type instantiation, got: {:?}", codes);
}

#[test]
fn test_property_used_before_initialization_2729() {
    // Error 2729: Property is used before its initialization
    use crate::thin_parser::ThinParserState;

    let source = r#"
class Foo {
    x = this.a;  // Error: Property 'a' is used before its initialization
    a = 1;
}

class NoError {
    a = 1;
    x = this.a;  // OK: 'a' is declared before 'x'
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());

    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();

    // Should have exactly one 2729 error (in class Foo)
    let count_2729 = codes.iter().filter(|&&c| c == 2729).count();
    assert_eq!(count_2729, 1,
        "Expected exactly 1 error 2729 for property used before initialization, got {} in: {:?}", count_2729, codes);
}

#[test]
fn test_property_not_assignable_to_same_in_base_2416() {
    // Error 2416: Property 'num' in type 'WrongTypePropertyImpl' is not assignable
    // to the same property in base type 'WrongTypeProperty'.
    use crate::thin_parser::ThinParserState;

    let source = r#"
abstract class WrongTypeProperty {
    abstract num: number;
}
class WrongTypePropertyImpl extends WrongTypeProperty {
    num = "nope, wrong";
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    // Debug: Print parsed classes
    let arena = parser.get_arena();
    println!("Number of classes in arena: {}", arena.classes.len());
    for (i, class) in arena.classes.iter().enumerate() {
        println!("Class {}: has heritage = {}", i, class.heritage_clauses.is_some());
        if let Some(ref hc) = class.heritage_clauses {
            println!("  Heritage clause nodes: {}", hc.nodes.len());
        }
    }

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    // Debug: print file locals
    println!("File locals count: {}", binder.file_locals.len());

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());

    checker.check_source_file(root);

    println!("Diagnostics:");
    for diag in &checker.ctx.diagnostics {
        println!("  TS{}: {}", diag.code, diag.message_text);
    }

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();

    // Should have at least one 2416 error for the incompatible property type
    let count_2416 = codes.iter().filter(|&&c| c == 2416).count();
    assert!(count_2416 >= 1,
        "Expected at least 1 error 2416 for property not assignable to base, got {} in: {:?}", count_2416, codes);
}

#[test]
fn test_property_not_assignable_to_generic_base_2416() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
abstract class Base<T> {
    abstract value: T;
}
class Derived extends Base<string> {
    value = 123;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2416),
        "Expected error 2416 for generic base property mismatch, got: {:?}",
        codes
    );
}

#[test]
fn test_non_abstract_class_missing_implementations_2654() {
    // Error 2654: Non-abstract class 'C' is missing implementations for
    // the following members of 'B': 'prop', 'm'.
    use crate::thin_parser::ThinParserState;

    let source = r#"
abstract class B {
    abstract prop: number;
    abstract m(): void;
}
class C extends B {
    // Missing implementations for 'prop' and 'm'
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());

    checker.check_source_file(root);

    println!("Diagnostics:");
    for diag in &checker.ctx.diagnostics {
        println!("  TS{}: {}", diag.code, diag.message_text);
    }

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();

    // Should have error 2654 for missing abstract implementations
    let count_2654 = codes.iter().filter(|&&c| c == 2654).count();
    assert!(count_2654 >= 1,
        "Expected at least 1 error 2654 for missing abstract implementations, got {} in: {:?}", count_2654, codes);

    // Check the message mentions the missing members
    let has_prop = checker.ctx.diagnostics.iter()
        .any(|d| d.code == 2654 && d.message_text.contains("'prop'"));
    let has_m = checker.ctx.diagnostics.iter()
        .any(|d| d.code == 2654 && d.message_text.contains("'m'"));
    assert!(has_prop, "Error 2654 should mention missing 'prop'");
    assert!(has_m, "Error 2654 should mention missing 'm'");
}

#[test]
fn test_readonly_property_assignment_2540() {
    // Error 2540: Cannot assign to 'ro' because it is a read-only property.
    use crate::thin_parser::ThinParserState;

    let source = r#"
class C {
    readonly ro: string = "readonly please";
}
let c = new C();
c.ro = "error: lhs of assignment can't be readonly";
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());

    checker.check_source_file(root);

    println!("Diagnostics:");
    for diag in &checker.ctx.diagnostics {
        println!("  TS{}: {}", diag.code, diag.message_text);
    }

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();

    // Should have error 2540 for readonly property assignment
    let count_2540 = codes.iter().filter(|&&c| c == 2540).count();
    assert!(count_2540 >= 1,
        "Expected at least 1 error 2540 for readonly property assignment, got {} in: {:?}", count_2540, codes);
}

#[test]
fn test_readonly_element_access_assignment_2540() {
    // Error 2540: Cannot assign to 'name' because it is a read-only property.
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Config {
    readonly name: string;
}
let config: Config = { name: "ok" };
config["name"] = "error";
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());

    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();

    let count_2540 = codes.iter().filter(|&&c| c == 2540).count();
    assert!(count_2540 >= 1,
        "Expected at least 1 error 2540 for readonly element access assignment, got {} in: {:?}", count_2540, codes);
}

#[test]
fn test_readonly_array_element_assignment_2540() {
    // Error 2540: Cannot assign to '0' because it is a read-only property.
    use crate::thin_parser::ThinParserState;

    let source = r#"
const xs: readonly number[] = [1, 2];
xs[0] = 3;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());

    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();

    let count_2540 = codes.iter().filter(|&&c| c == 2540).count();
    assert!(count_2540 >= 1,
        "Expected at least 1 error 2540 for readonly array element assignment, got {} in: {:?}", count_2540, codes);
}

#[test]
fn test_readonly_method_signature_assignment_2540() {
    // Error 2540: Cannot assign to 'run' because it is a read-only property.
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Service {
    readonly run(): void;
}
let svc: Service = { run() {} };
svc.run = () => {};
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());

    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();

    let count_2540 = codes.iter().filter(|&&c| c == 2540).count();
    assert!(count_2540 >= 1,
        "Expected at least 1 error 2540 for readonly method signature assignment, got {} in: {:?}", count_2540, codes);
}

#[test]
fn test_readonly_index_signature_element_access_assignment_2540() {
    // Error 2540: Cannot assign to 'a' because it is a read-only property.
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface ReadonlyMap {
    readonly [key: string]: number;
}
let map: ReadonlyMap = { a: 1 };
map["a"] = 2;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());

    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();

    let count_2540 = codes.iter().filter(|&&c| c == 2540).count();
    assert!(count_2540 >= 1,
        "Expected at least 1 error 2540 for readonly index signature assignment, got {} in: {:?}", count_2540, codes);
}

#[test]
fn test_readonly_index_signature_variable_access_assignment_2540() {
    // Error 2540: Cannot assign via readonly index signature.
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface ReadonlyMap {
    readonly [key: string]: number;
}
let map: ReadonlyMap = { a: 1 };
let key: string = "a";
map[key] = 2;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());

    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();

    let count_2540 = codes.iter().filter(|&&c| c == 2540).count();
    assert!(count_2540 >= 1,
        "Expected at least 1 error 2540 for readonly index signature assignment, got {} in: {:?}", count_2540, codes);
}

#[test]
fn test_abstractPropertyNegative_errors() {
    // Test the full abstractPropertyNegative test case to verify expected errors
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface A {
    prop: string;
    m(): string;
}
abstract class B implements A {
    abstract prop: string;
    public abstract readonly ro: string;
    abstract get readonlyProp(): string;
    abstract m(): string;
    abstract get mismatch(): string;
    abstract set mismatch(val: number);
}
class C extends B {
    readonly ro = "readonly please";
    abstract notAllowed: string;
    get concreteWithNoBody(): string;
}
let c = new C();
c.ro = "error: lhs of assignment can't be readonly";
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());

    checker.check_source_file(root);

    println!("Diagnostics:");
    for diag in &checker.ctx.diagnostics {
        println!("  TS{}: {}", diag.code, diag.message_text);
    }

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();

    // Expected errors:
    // - 2654: Non-abstract class 'C' is missing implementations
    // - 1253: Abstract properties can only appear within an abstract class
    // - 2540: Cannot assign to 'ro' because it is a read-only property
    // - 2676: Accessors must both be abstract or non-abstract (on mismatch getter/setter)

    // We should NOT have 2322 (accessor type compatibility) for abstract accessors
    let count_2322 = codes.iter().filter(|&&c| c == 2322).count();
    assert_eq!(count_2322, 0, "Should not produce 2322 errors for abstract accessor pairs");

    // We should have the expected errors
    assert!(codes.contains(&2654), "Should have error 2654 for missing implementations");
    assert!(codes.contains(&1253), "Should have error 1253 for abstract in non-abstract class");
    assert!(codes.contains(&2540), "Should have error 2540 for readonly assignment");
}

#[test]
fn test_contextual_typing_for_function_parameters() {
    use crate::solver::ContextualTypeContext;

    // Test that ContextualTypeContext can extract parameter types from function types
    let types = TypeInterner::new();

    // Create a function type: (x: string, y: number) => boolean
    use crate::solver::{FunctionShape, ParamInfo};
    use std::sync::Arc;

    let func_shape = FunctionShape {
        type_params: vec![],
        params: vec![
            ParamInfo { name: Some(types.intern_string("x")), type_id: TypeId::STRING, optional: false, rest: false },
            ParamInfo { name: Some(types.intern_string("y")), type_id: TypeId::NUMBER, optional: false, rest: false },
        ],
        this_type: None,
        return_type: TypeId::BOOLEAN,
        type_predicate: None,
        is_constructor: false,
    };

    let func_type = types.function(func_shape);

    // Create contextual context
    let ctx = ContextualTypeContext::with_expected(&types, func_type);

    // Test parameter type extraction
    assert_eq!(ctx.get_parameter_type(0), Some(TypeId::STRING));
    assert_eq!(ctx.get_parameter_type(1), Some(TypeId::NUMBER));
    assert_eq!(ctx.get_parameter_type(2), None); // Out of bounds

    // Test return type extraction
    assert_eq!(ctx.get_return_type(), Some(TypeId::BOOLEAN));
}

#[test]
fn test_contextual_typing_skips_this_parameter() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;
    use crate::parser::syntax_kind_ext;

    let source = r#"
function takesHandler(fn: (this: { value: number }, x: string) => void) {}
takesHandler(function(this: { value: number }, x) {
    let y: number = x;
});
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");
    let expr_stmt_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("expr stmt node"))
        .expect("expr stmt data");
    let call_idx = expr_stmt.expression;
    let call_expr = arena.get_call_expr(arena.get(call_idx).expect("call node")).expect("call expr");
    let args = call_expr.arguments.as_ref().expect("call arguments");
    let func_idx = *args.nodes.first().expect("function argument");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.get_type_of_node(call_idx);

    let func_type = checker.get_type_of_node(func_idx);
    let Some(TypeKey::Function(shape_id)) = checker.ctx.types.lookup(func_type) else {
        panic!("expected function type for argument");
    };
    let shape = checker.ctx.types.function_shape(shape_id);
    assert!(shape.this_type.is_some(), "expected this type on contextual function");
    assert_eq!(shape.params.len(), 1, "expected single parameter besides this");
    assert_eq!(shape.params[0].type_id, TypeId::STRING, "expected contextual string parameter");
}

#[test]
fn test_contextual_typing_for_variable_initializer() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
const handler: (x: string) => void = (x) => {
    let y: number = x;
};
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2322),
        "Expected error 2322 (Type not assignable) from contextual typing, got: {:?}",
        codes);
}

#[test]
fn test_contextual_typing_overload_by_arity() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
function register(cb: (x: string) => void): void;
function register(cb: (x: number, y: boolean) => void, flag: boolean): void;
function register(cb: unknown, flag?: boolean) {}

register((x) => {
    let y: string = x;
});
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(!codes.contains(&2322),
        "Did not expect error 2322 for overload-by-arity contextual typing, got: {:?}",
        codes);
}

#[test]
fn test_contextual_typing_for_object_properties() {
    use crate::solver::ContextualTypeContext;

    // Test that ContextualTypeContext can extract property types from object types
    let types = TypeInterner::new();

    // Create an object type: { name: string, age: number }
    use crate::solver::PropertyInfo;
    use std::sync::Arc;

    let obj_type = types.object(vec![
        PropertyInfo { name: types.intern_string("name"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
        PropertyInfo { name: types.intern_string("age"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ]);

    // Create contextual context
    let ctx = ContextualTypeContext::with_expected(&types, obj_type);

    // Test property type extraction
    assert_eq!(ctx.get_property_type("name"), Some(TypeId::STRING));
    assert_eq!(ctx.get_property_type("age"), Some(TypeId::NUMBER));
    assert_eq!(ctx.get_property_type("unknown"), None);
}

#[test]
fn test_contextual_property_type_infers_callback_param() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
type Handler = { cb: (x: number) => void };
const h: Handler = { cb: x => x.toUpperCase() };
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2339),
        "Expected error 2339 for contextual property param mismatch, got: {:?}", codes);
}

#[test]
fn test_strict_null_checks_property_access() {
    use crate::solver::{PropertyAccessEvaluator, PropertyAccessResult, PropertyInfo};
    use std::sync::Arc;

    // Test property access on nullable types
    let types = TypeInterner::new();

    // Create object type: { x: number }
    let obj_type = types.object(vec![
        PropertyInfo { name: types.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ]);

    // Create union type: { x: number } | null
    let nullable_obj = types.union(vec![obj_type, TypeId::NULL]);

    let evaluator = PropertyAccessEvaluator::new(&types);

    // Access property on nullable type should return PossiblyNullOrUndefined
    let result = evaluator.resolve_property_access(nullable_obj, "x");
    match result {
        PropertyAccessResult::PossiblyNullOrUndefined { property_type, cause } => {
            // Should have property_type = number
            assert_eq!(property_type, Some(TypeId::NUMBER));
            // Cause should be null
            assert_eq!(cause, TypeId::NULL);
        }
        _ => panic!("Expected PossiblyNullOrUndefined, got {:?}", result),
    }
}

#[test]
fn test_strict_null_checks_undefined_type() {
    use crate::solver::{PropertyAccessEvaluator, PropertyAccessResult, PropertyInfo};
    use std::sync::Arc;

    // Test property access on possibly undefined types
    let types = TypeInterner::new();

    // Create object type: { y: string }
    let obj_type = types.object(vec![
        PropertyInfo { name: types.intern_string("y"), type_id: TypeId::STRING,
 write_type: TypeId::STRING, optional: false, readonly: false, is_method: false },
    ]);

    // Create union type: { y: string } | undefined
    let possibly_undefined = types.union(vec![obj_type, TypeId::UNDEFINED]);

    let evaluator = PropertyAccessEvaluator::new(&types);

    // Access property on possibly undefined type
    let result = evaluator.resolve_property_access(possibly_undefined, "y");
    match result {
        PropertyAccessResult::PossiblyNullOrUndefined { property_type, cause } => {
            assert_eq!(property_type, Some(TypeId::STRING));
            assert_eq!(cause, TypeId::UNDEFINED);
        }
        _ => panic!("Expected PossiblyNullOrUndefined, got {:?}", result),
    }
}

#[test]
fn test_strict_null_checks_both_null_and_undefined() {
    use crate::solver::{PropertyAccessEvaluator, PropertyAccessResult, PropertyInfo, TypeKey};
    use std::sync::Arc;

    // Test property access on type that is both null and undefined
    let types = TypeInterner::new();

    // Create object type: { z: boolean }
    let obj_type = types.object(vec![
        PropertyInfo { name: types.intern_string("z"), type_id: TypeId::BOOLEAN,
 write_type: TypeId::BOOLEAN, optional: false, readonly: false, is_method: false },
    ]);

    // Create union type: { z: boolean } | null | undefined
    let nullable_undefined = types.union(vec![obj_type, TypeId::NULL, TypeId::UNDEFINED]);

    let evaluator = PropertyAccessEvaluator::new(&types);

    // Access property on possibly null or undefined type
    let result = evaluator.resolve_property_access(nullable_undefined, "z");
    match result {
        PropertyAccessResult::PossiblyNullOrUndefined { property_type, cause } => {
            assert_eq!(property_type, Some(TypeId::BOOLEAN));
            // Cause should be a union of null | undefined
            let cause_key = types.lookup(cause);
            match cause_key {
                Some(TypeKey::Union(members)) => {
                    let members = types.type_list(members);
                    assert!(members.contains(&TypeId::NULL), "Cause should contain null");
                    assert!(members.contains(&TypeId::UNDEFINED), "Cause should contain undefined");
                }
                _ => panic!("Expected cause to be union of null | undefined"),
            }
        }
        _ => panic!("Expected PossiblyNullOrUndefined, got {:?}", result),
    }
}

#[test]
fn test_strict_null_checks_non_nullable_success() {
    use crate::solver::{PropertyAccessEvaluator, PropertyAccessResult, PropertyInfo};
    use std::sync::Arc;

    // Test that non-nullable types succeed normally
    let types = TypeInterner::new();

    // Create object type: { x: number }
    let obj_type = types.object(vec![
        PropertyInfo { name: types.intern_string("x"), type_id: TypeId::NUMBER,
 write_type: TypeId::NUMBER, optional: false, readonly: false, is_method: false },
    ]);

    let evaluator = PropertyAccessEvaluator::new(&types);

    // Access property on non-nullable type should succeed
    let result = evaluator.resolve_property_access(obj_type, "x");
    match result {
        PropertyAccessResult::Success { type_id: prop_type, .. } => {
            assert_eq!(prop_type, TypeId::NUMBER);
        }
        _ => panic!("Expected Success, got {:?}", result),
    }
}

#[test]
fn test_strict_null_checks_null_only() {
    use crate::solver::{PropertyAccessEvaluator, PropertyAccessResult};

    // Test accessing property directly on null type
    let types = TypeInterner::new();

    let evaluator = PropertyAccessEvaluator::new(&types);

    let result = evaluator.resolve_property_access(TypeId::NULL, "anything");
    match result {
        PropertyAccessResult::PossiblyNullOrUndefined { property_type, cause } => {
            assert_eq!(property_type, None);
            assert_eq!(cause, TypeId::NULL);
        }
        _ => panic!("Expected PossiblyNullOrUndefined, got {:?}", result),
    }
}

// ============== Symbol type checking tests ==============

#[test]
fn test_symbol_constructor_call_signature() {
    use crate::thin_parser::ThinParserState;

    // Test Symbol() with valid arguments
    let source = r#"const s1 = Symbol();
const s2 = Symbol("name");
const s3 = Symbol(42);"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    // Should have no errors - all calls are valid
    assert_eq!(checker.ctx.diagnostics.len(), 0);
}

#[test]
fn test_symbol_constructor_too_many_args() {
    use crate::thin_parser::ThinParserState;

    // Test Symbol() with too many arguments - should error TS2554
    let source = r#"const s = Symbol("name", "extra");"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    // Should have an error for too many arguments
    // Could be 2554 (expected arguments) or 2349 (cannot invoke) depending on validation path
    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2554) || codes.contains(&2349),
        "Expected error 2554 or 2349 for too many arguments, got: {:?}", codes);
}

#[test]
fn test_variable_redeclaration_same_type() {
    use crate::thin_parser::ThinParserState;

    // Test that redeclaring a variable with the same type is allowed
    let source = r#"function test() {
    var x: string;
    var x: string;
}"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    // Should have no errors - same type is allowed
    assert_eq!(checker.ctx.diagnostics.len(), 0);
}

#[test]
fn test_variable_redeclaration_different_type_2403() {
    use crate::thin_parser::ThinParserState;

    // Test that redeclaring a variable with different type causes error TS2403
    // Must be inside a function where local scopes are active
    let source = r#"function test() {
    var x: string;
    var x: number;
}"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    // Should have error 2403: Subsequent variable declarations must have the same type
    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2403), "Expected error 2403 for variable redeclaration, got: {:?}", codes);
}

#[test]
fn test_symbol_property_access_description() {
    use crate::solver::{PropertyAccessEvaluator, PropertyAccessResult, TypeKey};
    use std::sync::Arc;

    // Test accessing .description on symbol type
    let types = TypeInterner::new();
    let evaluator = PropertyAccessEvaluator::new(&types);

    let result = evaluator.resolve_property_access(TypeId::SYMBOL, "description");
    match result {
        PropertyAccessResult::Success { type_id: prop_type, .. } => {
            // description should be string | undefined
            let key = types.lookup(prop_type).expect("Property type should exist");
            match key {
                TypeKey::Union(members) => {
                    let members = types.type_list(members);
                    assert_eq!(members.len(), 2);
                    assert!(members.contains(&TypeId::STRING));
                    assert!(members.contains(&TypeId::UNDEFINED));
                }
                _ => panic!("Expected union type for description, got: {:?}", key),
            }
        }
        _ => panic!("Expected Success for symbol.description, got: {:?}", result),
    }
}

#[test]
fn test_symbol_property_access_methods() {
    use crate::solver::{PropertyAccessEvaluator, PropertyAccessResult};

    // Test accessing methods on symbol type
    let types = TypeInterner::new();
    let evaluator = PropertyAccessEvaluator::new(&types);

    // toString and valueOf should return ANY for now (function types are complex)
    let result_to_string = evaluator.resolve_property_access(TypeId::SYMBOL, "toString");
    match result_to_string {
        PropertyAccessResult::Success { type_id: prop_type, .. } => {
            assert_eq!(prop_type, TypeId::ANY);
        }
        _ => panic!("Expected Success for symbol.toString, got: {:?}", result_to_string),
    }

    let result_value_of = evaluator.resolve_property_access(TypeId::SYMBOL, "valueOf");
    match result_value_of {
        PropertyAccessResult::Success { type_id: prop_type, .. } => {
            assert_eq!(prop_type, TypeId::ANY);
        }
        _ => panic!("Expected Success for symbol.valueOf, got: {:?}", result_value_of),
    }
}

#[test]
fn test_symbol_property_not_found() {
    use crate::solver::{PropertyAccessEvaluator, PropertyAccessResult};

    // Test accessing non-existent property on symbol type
    let types = TypeInterner::new();
    let evaluator = PropertyAccessEvaluator::new(&types);
    let name_atom = types.intern_string("nonexistent");

    let result = evaluator.resolve_property_access(TypeId::SYMBOL, "nonexistent");
    match result {
        PropertyAccessResult::PropertyNotFound { type_id, property_name } => {
            assert_eq!(type_id, TypeId::SYMBOL);
            assert_eq!(property_name, name_atom);
        }
        _ => panic!("Expected PropertyNotFound for unknown property, got: {:?}", result),
    }
}

// ============== Property access from index signature tests (error 4111) ==============

#[test]
fn test_property_access_from_index_signature_4111() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface StringMap {
    [key: string]: number;
}
const obj: StringMap = {} as any;
const val = obj.someProperty;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&4111), "Expected error 4111 for property access from index signature, got: {:?}", codes);
}

#[test]
fn test_explicit_property_no_error_4111() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface MixedType {
    explicitProp: string;
    [key: string]: string | number;
}
const obj: MixedType = {} as any;
const val = obj.explicitProp;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(!codes.contains(&4111), "Should not have error 4111 for explicit property");
}

#[test]
fn test_union_with_index_signature_4111() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
type Mixed = { x: number } | { [key: string]: number };
const obj: Mixed = {} as any;
const val = obj.x;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&4111), "Expected error 4111 for union with index signature member");
}

#[test]
fn test_checker_lowers_full_source_file() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::{TypeKey, SymbolRef};

    let source = r#"
interface Foo { x: number; }
type Bar = Foo | string;
type Baz = [string, number];
type Qux = { [key: string]: Foo };
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let foo_sym = binder.file_locals.get("Foo").expect("Foo should exist");
    let bar_sym = binder.file_locals.get("Bar").expect("Bar should exist");
    let baz_sym = binder.file_locals.get("Baz").expect("Baz should exist");
    let qux_sym = binder.file_locals.get("Qux").expect("Qux should exist");

    let foo_type = checker.get_type_of_symbol(foo_sym);
    let foo_key = types.lookup(foo_type).expect("Foo type should exist");
    match foo_key {
        TypeKey::Object(shape_id) => {
            let shape = types.object_shape(shape_id);
            let prop = shape
                .properties
                .iter()
                .find(|prop| types.resolve_atom(prop.name) == "x")
                .expect("Expected property x");
            assert_eq!(prop.type_id, TypeId::NUMBER);
        }
        _ => panic!("Expected Foo to be Object type, got {:?}", foo_key),
    }

    let bar_type = checker.get_type_of_symbol(bar_sym);
    let bar_key = types.lookup(bar_type).expect("Bar type should exist");
    match bar_key {
        TypeKey::Union(members) => {
            let members = types.type_list(members);
            assert_eq!(members.len(), 2);
            assert!(members.contains(&TypeId::STRING));
            assert!(members.contains(&foo_type));
        }
        _ => panic!("Expected Bar to be Union type, got {:?}", bar_key),
    }

    let baz_type = checker.get_type_of_symbol(baz_sym);
    let baz_key = types.lookup(baz_type).expect("Baz type should exist");
    match baz_key {
        TypeKey::Tuple(elements) => {
            let elements = types.tuple_list(elements);
            assert_eq!(elements.len(), 2);
            assert_eq!(elements[0].type_id, TypeId::STRING);
            assert_eq!(elements[1].type_id, TypeId::NUMBER);
        }
        _ => panic!("Expected Baz to be Tuple type, got {:?}", baz_key),
    }

    let qux_type = checker.get_type_of_symbol(qux_sym);
    let qux_key = types.lookup(qux_type).expect("Qux type should exist");
    match qux_key {
        TypeKey::ObjectWithIndex(shape_id) => {
            let shape = types.object_shape(shape_id);
            let string_index = shape.string_index.as_ref().expect("Expected string index signature");
            assert_eq!(string_index.key_type, TypeId::STRING);
            let value_key = types.lookup(string_index.value_type).expect("Index value type should exist");
            match value_key {
                TypeKey::Ref(SymbolRef(sym_id)) => assert_eq!(sym_id, foo_sym.0),
                _ => panic!("Expected Foo reference type, got {:?}", value_key),
            }
        }
        _ => panic!("Expected Qux to be ObjectWithIndex type, got {:?}", qux_key),
    }
}

#[test]
fn test_interface_extends_inherits_properties() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Base {
    base: string;
}
interface Derived extends Base {
    derived: number;
}
const obj: Derived = { base: "x", derived: 1 };
const base_value = obj.base;
const derived_value = obj.derived;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let base_sym = binder.file_locals.get("base_value").expect("base_value should exist");
    let base_type = checker.get_type_of_symbol(base_sym);
    assert_eq!(base_type, TypeId::STRING);

    let derived_sym = binder.file_locals.get("derived_value").expect("derived_value should exist");
    let derived_type = checker.get_type_of_symbol(derived_sym);
    assert_eq!(derived_type, TypeId::NUMBER);
}

#[test]
fn test_interface_extends_applies_type_arguments() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Box<T> {
    value: T;
}
interface Derived extends Box<string> {
    count: number;
}
const obj: Derived = { value: "x", count: 1 };
const value = obj.value;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    assert_eq!(value_type, TypeId::STRING);
}

#[test]
fn test_interface_extends_type_alias_applies_type_arguments() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
type Box<T> = { value: T };
interface Derived extends Box<string> {
    count: number;
}
const obj: Derived = { value: "x", count: 1 };
const value = obj.value;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    assert_eq!(value_type, TypeId::STRING);
}

#[test]
fn test_interface_extends_class_applies_type_arguments() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
class Box<T> {
    value: T;
}
interface Derived extends Box<string> {
    count: number;
}
const obj: Derived = { value: "x", count: 1 };
const value = obj.value;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    assert_eq!(value_type, TypeId::STRING);
}

#[test]
fn test_interface_extends_readonly_property_mismatch_2430() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Base {
    x: number;
}
interface Derived extends Base {
    readonly x: number;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2430), "Expected error 2430 for readonly property mismatch, got: {:?}", codes);
}

#[test]
fn test_interface_extends_optional_property_mismatch_2430() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Base {
    x: number;
}
interface Derived extends Base {
    x?: number;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2430), "Expected error 2430 for optional property mismatch, got: {:?}", codes);
}

#[test]
fn test_interface_extends_string_literal_property_mismatch_2430() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Base {
    "x": number;
}
interface Derived extends Base {
    "x"?: number;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2430), "Expected error 2430 for string literal property mismatch, got: {:?}", codes);
}

#[test]
fn test_interface_extends_generic_argument_mismatch_2430() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Base<T> {
    x: T;
}
interface Derived extends Base<string> {
    x: number;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2430), "Expected error 2430 for generic argument mismatch, got: {:?}", codes);
}

#[test]
fn test_interface_extends_generic_argument_match() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Base<T> {
    x: T;
}
interface Derived extends Base<string> {
    x: string;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);
}

#[test]
fn test_interface_extends_namespace_qualified_base_2430() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export interface Base {
        x: string;
    }
}
interface Derived extends NS.Base {
    x: number;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2430), "Expected error 2430 for namespace-qualified base mismatch, got: {:?}", codes);
}

#[test]
fn test_interface_extends_generic_method_compatible() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Base {
    m<T>(value: T): T;
}
interface Derived extends Base {
    m<T>(value: T): T;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);
}

#[test]
fn test_checker_cross_namespace_type_reference() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
namespace Outer {
    export interface Inner { y: string; }
}
type Alias = Outer.Inner;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let alias_sym = binder.file_locals.get("Alias").expect("Alias should exist");
    let alias_type = checker.get_type_of_symbol(alias_sym);
    let alias_key = types.lookup(alias_type).expect("Alias type should exist");
    match alias_key {
        TypeKey::Object(shape_id) => {
            let shape = types.object_shape(shape_id);
            let prop = shape
                .properties
                .iter()
                .find(|prop| types.resolve_atom(prop.name) == "y")
                .expect("Expected property y");
            assert_eq!(prop.type_id, TypeId::STRING);
        }
        _ => panic!("Expected Alias to resolve to Object type, got {:?}", alias_key),
    }
}

#[test]
fn test_checker_module_augmentation_merges_exports() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
namespace Outer {
    export interface A { x: number; }
}
namespace Outer {
    export interface B { y: string; }
}
type AliasA = Outer.A;
type AliasB = Outer.B;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let alias_a_sym = binder.file_locals.get("AliasA").expect("AliasA should exist");
    let alias_b_sym = binder.file_locals.get("AliasB").expect("AliasB should exist");

    let alias_a_type = checker.get_type_of_symbol(alias_a_sym);
    let alias_b_type = checker.get_type_of_symbol(alias_b_sym);

    let alias_a_key = types.lookup(alias_a_type).expect("AliasA type should exist");
    match alias_a_key {
        TypeKey::Object(shape_id) => {
            let shape = types.object_shape(shape_id);
            let prop = shape
                .properties
                .iter()
                .find(|prop| types.resolve_atom(prop.name) == "x")
                .expect("Expected property x");
            assert_eq!(prop.type_id, TypeId::NUMBER);
        }
        _ => panic!("Expected AliasA to resolve to Object type, got {:?}", alias_a_key),
    }

    let alias_b_key = types.lookup(alias_b_type).expect("AliasB type should exist");
    match alias_b_key {
        TypeKey::Object(shape_id) => {
            let shape = types.object_shape(shape_id);
            let prop = shape
                .properties
                .iter()
                .find(|prop| types.resolve_atom(prop.name) == "y")
                .expect("Expected property y");
            assert_eq!(prop.type_id, TypeId::STRING);
        }
        _ => panic!("Expected AliasB to resolve to Object type, got {:?}", alias_b_key),
    }
}

#[test]
fn test_checker_lower_generic_type_reference_applies_args() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::{TypeKey, SymbolRef};

    let source = r#"
type Box<T> = { value: T };
type Alias = Box<string>;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let box_sym = binder.file_locals.get("Box").expect("Box should exist");
    let alias_sym = binder.file_locals.get("Alias").expect("Alias should exist");

    let alias_type = checker.get_type_of_symbol(alias_sym);
    let alias_key = types.lookup(alias_type).expect("Alias type should exist");
    match alias_key {
        TypeKey::Application(app_id) => {
            let app = types.type_application(app_id);
            assert_eq!(app.args, vec![TypeId::STRING]);
            match types.lookup(app.base) {
                Some(TypeKey::Ref(SymbolRef(sym_id))) => assert_eq!(sym_id, box_sym.0),
                other => panic!("Expected Ref base type, got {:?}", other),
            }
        }
        _ => panic!("Expected Alias to be Application type, got {:?}", alias_key),
    }
}

#[test]
fn test_checker_lowers_generic_function_type_annotation_uses_type_params() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
const f: <T>(value: T) => T = (value) => value;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let f_sym = binder.file_locals.get("f").expect("f should exist");
    let f_type = checker.get_type_of_symbol(f_sym);
    let f_key = types.lookup(f_type).expect("f type should exist");
    match f_key {
        TypeKey::Function(shape_id) => {
            let shape = types.function_shape(shape_id);
            assert_eq!(shape.type_params.len(), 1);
            assert_eq!(types.resolve_atom(shape.type_params[0].name), "T");
            assert_eq!(shape.params.len(), 1);

            let param_key = types.lookup(shape.params[0].type_id).expect("Param type should exist");
            match param_key {
                TypeKey::TypeParameter(info) => {
                    assert_eq!(types.resolve_atom(info.name), "T");
                }
                _ => panic!("Expected param type to be type parameter, got {:?}", param_key),
            }

            let return_key = types.lookup(shape.return_type).expect("Return type should exist");
            match return_key {
                TypeKey::TypeParameter(info) => {
                    assert_eq!(types.resolve_atom(info.name), "T");
                }
                _ => panic!("Expected return type to be type parameter, got {:?}", return_key),
            }
        }
        _ => panic!("Expected f to be Function type, got {:?}", f_key),
    }
}

#[test]
fn test_interface_generic_call_signature_uses_type_params() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
interface Callable {
    <T>(value: T): T;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let callable_sym = binder.file_locals.get("Callable").expect("Callable should exist");
    let callable_type = checker.get_type_of_symbol(callable_sym);
    let callable_key = types.lookup(callable_type).expect("Callable type should exist");
    match callable_key {
        TypeKey::Callable(shape_id) => {
            let shape = types.callable_shape(shape_id);
            assert_eq!(shape.call_signatures.len(), 1);
            let sig = &shape.call_signatures[0];
            assert_eq!(sig.type_params.len(), 1);
            assert_eq!(types.resolve_atom(sig.type_params[0].name), "T");
            assert_eq!(sig.params.len(), 1);

            let param_key = types.lookup(sig.params[0].type_id).expect("Param type should exist");
            match param_key {
                TypeKey::TypeParameter(info) => {
                    assert_eq!(types.resolve_atom(info.name), "T");
                }
                _ => panic!("Expected param type to be type parameter, got {:?}", param_key),
            }

            let return_key = types.lookup(sig.return_type).expect("Return type should exist");
            match return_key {
                TypeKey::TypeParameter(info) => {
                    assert_eq!(types.resolve_atom(info.name), "T");
                }
                _ => panic!("Expected return type to be type parameter, got {:?}", return_key),
            }
        }
        _ => panic!("Expected Callable to be Callable type, got {:?}", callable_key),
    }
}

#[test]
fn test_interface_generic_construct_signature_uses_type_params() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
interface Factory {
    new <T>(value: T): T;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let factory_sym = binder.file_locals.get("Factory").expect("Factory should exist");
    let factory_type = checker.get_type_of_symbol(factory_sym);
    let factory_key = types.lookup(factory_type).expect("Factory type should exist");
    match factory_key {
        TypeKey::Callable(shape_id) => {
            let shape = types.callable_shape(shape_id);
            assert_eq!(shape.construct_signatures.len(), 1);
            let sig = &shape.construct_signatures[0];
            assert_eq!(sig.type_params.len(), 1);
            assert_eq!(types.resolve_atom(sig.type_params[0].name), "T");
            assert_eq!(sig.params.len(), 1);

            let param_key = types.lookup(sig.params[0].type_id).expect("Param type should exist");
            match param_key {
                TypeKey::TypeParameter(info) => {
                    assert_eq!(types.resolve_atom(info.name), "T");
                }
                _ => panic!("Expected param type to be type parameter, got {:?}", param_key),
            }

            let return_key = types.lookup(sig.return_type).expect("Return type should exist");
            match return_key {
                TypeKey::TypeParameter(info) => {
                    assert_eq!(types.resolve_atom(info.name), "T");
                }
                _ => panic!("Expected return type to be type parameter, got {:?}", return_key),
            }
        }
        _ => panic!("Expected Factory to be Callable type, got {:?}", factory_key),
    }
}

#[test]
fn test_checker_lowers_generic_function_declaration_uses_type_params() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
function id<T>(value: T): T {
    return value;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let id_sym = binder.file_locals.get("id").expect("id should exist");
    let id_type = checker.get_type_of_symbol(id_sym);
    let id_key = types.lookup(id_type).expect("id type should exist");
    match id_key {
        TypeKey::Function(shape_id) => {
            let shape = types.function_shape(shape_id);
            assert_eq!(shape.type_params.len(), 1);
            assert_eq!(types.resolve_atom(shape.type_params[0].name), "T");
            assert_eq!(shape.params.len(), 1);

            let param_key = types.lookup(shape.params[0].type_id).expect("Param type should exist");
            match param_key {
                TypeKey::TypeParameter(info) => {
                    assert_eq!(types.resolve_atom(info.name), "T");
                }
                _ => panic!("Expected param type to be type parameter, got {:?}", param_key),
            }

            let return_key = types.lookup(shape.return_type).expect("Return type should exist");
            match return_key {
                TypeKey::TypeParameter(info) => {
                    assert_eq!(types.resolve_atom(info.name), "T");
                }
                _ => panic!("Expected return type to be type parameter, got {:?}", return_key),
            }
        }
        _ => panic!("Expected id to be Function type, got {:?}", id_key),
    }
}

#[test]
fn test_function_return_type_inferred_from_body() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::{TypeId, TypeKey};

    let source = r#"
function id(x: string) {
    return x;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let id_sym = binder.file_locals.get("id").expect("id should exist");
    let id_type = checker.get_type_of_symbol(id_sym);
    let id_key = types.lookup(id_type).expect("id type should exist");
    match id_key {
        TypeKey::Function(shape_id) => {
            let shape = types.function_shape(shape_id);
            assert_eq!(shape.return_type, TypeId::STRING);
        }
        _ => panic!("Expected id to be Function type, got {:?}", id_key),
    }
}

#[test]
fn test_arrow_function_return_type_inferred_union() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::{TypeId, TypeKey};

    let source = r#"
const f = (flag: boolean) => {
    if (flag) {
        return 1;
    }
    return "a";
};
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let f_sym = binder.file_locals.get("f").expect("f should exist");
    let f_type = checker.get_type_of_symbol(f_sym);
    let f_key = types.lookup(f_type).expect("f type should exist");
    match f_key {
        TypeKey::Function(shape_id) => {
            let shape = types.function_shape(shape_id);
            let return_key = types.lookup(shape.return_type).expect("return type should exist");
            match return_key {
                TypeKey::Union(members) => {
                    let members = types.type_list(members);
                    assert!(members.contains(&TypeId::NUMBER));
                    assert!(members.contains(&TypeId::STRING));
                }
                _ => panic!("Expected union return type, got {:?}", return_key),
            }
        }
        _ => panic!("Expected f to be Function type, got {:?}", f_key),
    }
}

#[test]
fn test_checker_lowers_element_access_array() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
const arr: number[] = [1, 2];
const value = arr[0];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    assert_eq!(value_type, TypeId::NUMBER);
}

#[test]
fn test_array_literal_best_common_type_prefers_supertype_element() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::{PropertyInfo, TypeId, TypeKey};

    let source = r#"
const arr = [{ a: "x" }, { a: "y", b: 1 }];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let arr_sym = binder.file_locals.get("arr").expect("arr should exist");
    let arr_type = checker.get_type_of_symbol(arr_sym);
    let arr_key = types.lookup(arr_type).expect("arr type should exist");
    match arr_key {
        TypeKey::Array(elem) => {
            let expected = types.object(vec![PropertyInfo {
                name: types.intern_string("a"),
                type_id: TypeId::STRING,
                write_type: TypeId::STRING,
                optional: false,
                readonly: false,
                is_method: false,
            }]);
            assert_eq!(elem, expected);
        }
        _ => panic!("Expected array type, got {:?}", arr_key),
    }
}

#[test]
fn test_checker_lowers_element_access_tuple_literals() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
const tup: [string, number] = ["a", 1];
const first = tup[0];
const second = tup[1];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let first_sym = binder.file_locals.get("first").expect("first should exist");
    let second_sym = binder.file_locals.get("second").expect("second should exist");

    let first_type = checker.get_type_of_symbol(first_sym);
    let second_type = checker.get_type_of_symbol(second_sym);

    assert_eq!(first_type, TypeId::STRING);
    assert_eq!(second_type, TypeId::NUMBER);
}

#[test]
fn test_checker_tuple_optional_element_access_includes_undefined() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::{TypeId, TypeKey};

    let source = r#"
const tup: [string?] = ["a"];
const first = tup[0];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let first_sym = binder.file_locals.get("first").expect("first should exist");
    let first_type = checker.get_type_of_symbol(first_sym);
    let first_key = types.lookup(first_type).expect("first type should exist");
    match first_key {
        TypeKey::Union(members) => {
            let members = types.type_list(members);
            assert!(members.contains(&TypeId::STRING));
            assert!(members.contains(&TypeId::UNDEFINED));
        }
        _ => panic!("Expected union type for first, got {:?}", first_key),
    }
}

#[test]
fn test_checker_lowers_element_access_string_literal_property() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
const obj = { x: 1, y: "hi" };
const value = obj["x"];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    assert_eq!(value_type, TypeId::NUMBER);
}

#[test]
fn test_checker_lowers_element_access_array_length() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
const arr = [1, 2];
const length = arr["length"];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let length_sym = binder.file_locals.get("length").expect("length should exist");
    let length_type = checker.get_type_of_symbol(length_sym);
    assert_eq!(length_type, TypeId::NUMBER);
}

#[test]
fn test_checker_lowers_element_access_numeric_string_index() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
const arr: number[] = [1, 2];
const value = arr["0"];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    assert_eq!(value_type, TypeId::NUMBER);
}

#[test]
fn test_checker_lowers_element_access_string_index_signature() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface StringMap {
    [key: string]: boolean;
}
const map: StringMap = {} as any;
const value = map["foo"];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    assert_eq!(value_type, TypeId::BOOLEAN);
}

#[test]
fn test_checker_lowers_element_access_number_index_signature() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface NumberMap {
    [key: number]: string;
}
const map: NumberMap = {} as any;
const value = map[1];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    assert_eq!(value_type, TypeId::STRING);
}

#[test]
fn test_checker_element_access_requires_index_signature() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Foo { x: number; }
const obj: Foo = { x: 1 };
let key: string = "x";
const value = obj[key];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&7053), "Expected error 7053 for missing index signature, got: {:?}", codes);
}

#[test]
fn test_checker_element_access_union_string_index_requires_signature() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Foo { x: number; }
const obj: Foo = { x: 1 };
let key: "x" | string;
const value = obj[key];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&7053), "Expected error 7053 for union string index, got: {:?}", codes);
}

#[test]
fn test_checker_element_access_union_string_number_index_requires_signature() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Foo { x: number; }
const obj: Foo = { x: 1 };
let key: string | number;
const value = obj[key];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&7053), "Expected error 7053 for union string/number index, got: {:?}", codes);
}

#[test]
fn test_checker_lowers_element_access_literal_key_union() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
interface Foo { a: number; b: string; }
const obj: Foo = { a: 1, b: "hi" };
let key: "a" | "b";
const value = obj[key];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    let value_key = types.lookup(value_type).expect("value type should exist");
    match value_key {
        TypeKey::Union(members) => {
            let members = types.type_list(members);
            assert!(members.contains(&TypeId::NUMBER));
            assert!(members.contains(&TypeId::STRING));
        }
        _ => panic!("Expected union type for value, got {:?}", value_key),
    }
}

#[test]
fn test_checker_lowers_element_access_literal_key_type() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Foo { a: number; b: string; }
const obj: Foo = { a: 1, b: "hi" };
let key: "a";
const value = obj[key];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    assert_eq!(value_type, TypeId::NUMBER);
}

#[test]
fn test_checker_lowers_element_access_numeric_literal_union() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
const tup: [string, number, boolean] = ["a", 1, true];
let idx: 0 | 2;
const value = tup[idx];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    let value_key = types.lookup(value_type).expect("value type should exist");
    match value_key {
        TypeKey::Union(members) => {
            let members = types.type_list(members);
            assert!(members.contains(&TypeId::STRING));
            assert!(members.contains(&TypeId::BOOLEAN));
            assert_eq!(members.len(), 2);
        }
        _ => panic!("Expected union type for value, got {:?}", value_key),
    }
}

#[test]
fn test_checker_lowers_element_access_mixed_literal_key_union() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
const arr: string[] = ["a"];
let key: "length" | 0;
const value = arr[key];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    let value_key = types.lookup(value_type).expect("value type should exist");
    match value_key {
        TypeKey::Union(members) => {
            let members = types.type_list(members);
            assert!(members.contains(&TypeId::STRING));
            assert!(members.contains(&TypeId::NUMBER));
            assert_eq!(members.len(), 2);
        }
        _ => panic!("Expected union type for value, got {:?}", value_key),
    }
}

#[test]
fn test_checker_element_access_reports_nullable_object() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
type Foo = { a: number };
let obj: Foo | undefined;
const value = obj["a"];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2532), "Expected error 2532 for possibly undefined object, got: {:?}", codes);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    assert_eq!(value_type, TypeId::NUMBER);
}

#[test]
fn test_checker_element_access_optional_chain_nullable_object() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
type Foo = { a: number };
let obj: Foo | undefined;
const value = obj?.["a"];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    let value_key = types.lookup(value_type).expect("value type should exist");
    match value_key {
        TypeKey::Union(members) => {
            let members = types.type_list(members);
            assert!(members.contains(&TypeId::NUMBER));
            assert!(members.contains(&TypeId::UNDEFINED));
        }
        _ => panic!("Expected union type for value, got {:?}", value_key),
    }
}

#[test]
fn test_checker_property_access_optional_chain_nullable_object() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
type Foo = { a: number };
let obj: Foo | undefined;
const value = obj?.a;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    let value_key = types.lookup(value_type).expect("value type should exist");
    match value_key {
        TypeKey::Union(members) => {
            let members = types.type_list(members);
            assert!(members.contains(&TypeId::NUMBER));
            assert!(members.contains(&TypeId::UNDEFINED));
        }
        _ => panic!("Expected union type for value, got {:?}", value_key),
    }
}

#[test]
fn test_checker_property_access_union_type() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
type U = { a: number } | { a: string };
const obj: U = { a: 1 };
const value = obj.a;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let value_sym = binder.file_locals.get("value").expect("value should exist");
    let value_type = checker.get_type_of_symbol(value_sym);
    let value_key = types.lookup(value_type).expect("value type should exist");
    match value_key {
        TypeKey::Union(members) => {
            let members = types.type_list(members);
            assert!(members.contains(&TypeId::NUMBER));
            assert!(members.contains(&TypeId::STRING));
        }
        _ => panic!("Expected union type for value, got {:?}", value_key),
    }
}

#[test]
fn test_checker_namespace_merges_with_class_exports() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::TypeKey;

    let source = r#"
class Foo {}
namespace Foo {
    export interface Bar { x: number; }
}
type Alias = Foo.Bar;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let alias_sym = binder.file_locals.get("Alias").expect("Alias should exist");
    let alias_type = checker.get_type_of_symbol(alias_sym);
    let alias_key = types.lookup(alias_type).expect("Alias type should exist");
    match alias_key {
        TypeKey::Object(shape_id) => {
            let shape = types.object_shape(shape_id);
            let prop = shape
                .properties
                .iter()
                .find(|prop| types.resolve_atom(prop.name) == "x")
                .expect("Expected property x");
            assert_eq!(prop.type_id, TypeId::NUMBER);
        }
        _ => panic!("Expected Alias to resolve to Object type, got {:?}", alias_key),
    }
}

#[test]
fn test_checker_interface_typeof_value_reference() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::{TypeKey, SymbolRef};

    let source = r#"
const Foo = 1;
namespace Ns {
    export const value = 1;
}
interface Bar {
    x: typeof Foo;
    y: typeof Ns.value;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let foo_sym = binder.file_locals.get("Foo").expect("Foo should exist");
    let ns_sym = binder.file_locals.get("Ns").expect("Ns should exist");
    let value_sym = binder.get_symbol(ns_sym)
        .and_then(|symbol| symbol.exports.as_ref())
        .and_then(|exports| exports.get("value"))
        .expect("Ns.value should exist");

    let bar_sym = binder.file_locals.get("Bar").expect("Bar should exist");
    let bar_type = checker.get_type_of_symbol(bar_sym);
    let bar_key = types.lookup(bar_type).expect("Bar type should exist");
    match bar_key {
        TypeKey::Object(shape_id) => {
            let shape = types.object_shape(shape_id);
            let prop_names: Vec<String> = shape
                .properties
                .iter()
                .map(|prop| types.resolve_atom(prop.name))
                .collect();
            let prop_x = shape
                .properties
                .iter()
                .find(|prop| types.resolve_atom(prop.name) == "x")
                .expect("Expected property x");
            let prop_y = shape
                .properties
                .iter()
                .find(|prop| types.resolve_atom(prop.name) == "y")
                .expect(&format!("Expected property y, got {:?}", prop_names));

            match types.lookup(prop_x.type_id) {
                Some(TypeKey::TypeQuery(SymbolRef(sym_id))) => assert_eq!(sym_id, foo_sym.0),
                other => panic!("Expected x to be typeof Foo, got {:?}", other),
            }

            match types.lookup(prop_y.type_id) {
                Some(TypeKey::TypeQuery(SymbolRef(sym_id))) => assert_eq!(sym_id, value_sym.0),
                other => panic!("Expected y to be typeof Ns.value, got {:?}", other),
            }
        }
        _ => panic!("Expected Bar to resolve to Object type, got {:?}", bar_key),
    }
}

#[test]
fn test_checker_typeof_with_type_arguments() {
    use crate::thin_parser::ThinParserState;
    use crate::solver::{TypeKey, SymbolRef};

    let source = r#"
const Foo = <T>(value: T) => value;
type Alias = typeof Foo<string>;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let foo_sym = binder.file_locals.get("Foo").expect("Foo should exist");
    let alias_sym = binder.file_locals.get("Alias").expect("Alias should exist");

    let alias_type = checker.get_type_of_symbol(alias_sym);
    let alias_key = types.lookup(alias_type).expect("Alias type should exist");
    match alias_key {
        TypeKey::Application(app_id) => {
            let app = types.type_application(app_id);
            assert_eq!(app.args, vec![TypeId::STRING]);
            match types.lookup(app.base) {
                Some(TypeKey::TypeQuery(SymbolRef(sym_id))) => assert_eq!(sym_id, foo_sym.0),
                other => panic!("Expected TypeQuery base type, got {:?}", other),
            }
        }
        _ => panic!("Expected Alias to be Application type, got {:?}", alias_key),
    }
}

#[test]
fn test_checker_circular_type_aliases() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
type A = B;
type B = A;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let a_sym = binder.file_locals.get("A").expect("A should exist");
    let b_sym = binder.file_locals.get("B").expect("B should exist");

    assert_eq!(checker.get_type_of_symbol(a_sym), TypeId::ANY);
    assert_eq!(checker.get_type_of_symbol(b_sym), TypeId::ANY);
}

#[test]
fn test_index_signature_at_solver_level() {
    use crate::solver::{PropertyAccessEvaluator, PropertyAccessResult, ObjectShape, IndexSignature};
    use std::sync::Arc;

    // Test that index signature resolution is tracked at solver level
    let types = TypeInterner::new();

    // Create object type with only index signature
    let shape = ObjectShape {
        properties: vec![],
        string_index: Some(IndexSignature {
            key_type: TypeId::STRING,
            value_type: TypeId::NUMBER,
            readonly: false,
        }),
        number_index: None,
    };

    let obj_type = types.object_with_index(shape);
    let evaluator = PropertyAccessEvaluator::new(&types);

    let result = evaluator.resolve_property_access(obj_type, "anyProperty");
    match result {
        PropertyAccessResult::Success { type_id, from_index_signature } => {
            assert_eq!(type_id, TypeId::NUMBER);
            assert_eq!(from_index_signature, true, "Should be marked as from_index_signature");
        }
        _ => panic!("Expected Success, got: {:?}", result),
    }
}

// ============== Ambient module pattern tests (errors 5061, 2819) ==============

#[test]
fn test_ambient_module_relative_path_5061() {
    use crate::thin_parser::ThinParserState;

    // TS5061: Ambient module declaration cannot specify relative module name
    let source = r#"
declare module "./relative-module" {
    export function foo(): void;
}

declare module "../another-relative" {
    export const bar: number;
}

declare module "." {
    export type Baz = string;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let error_count = codes.iter().filter(|&&c| c == 5061).count();

    assert_eq!(error_count, 3,
        "Expected 3 errors with code 5061 for relative module names, got: {:?}", codes);
}

#[test]
fn test_ambient_module_absolute_path_ok() {
    use crate::thin_parser::ThinParserState;

    // Absolute module names should be allowed in ambient declarations
    let source = r#"
declare module "absolute-module" {
    export function foo(): void;
}

declare module "@scoped/package" {
    export const bar: number;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let error_5061_count = codes.iter().filter(|&&c| c == 5061).count();

    assert_eq!(error_5061_count, 0,
        "Expected no error 5061 for absolute module names, got: {:?}", codes);
}

#[test]
fn test_private_identifier_in_ambient_class_2819() {
    use crate::thin_parser::ThinParserState;

    // TS2819: Private identifiers are not allowed in ambient contexts
    let source = r#"
declare class AmbientClass {
    #privateField: string;
    #anotherPrivate: number;

    #privateMethod(): void;

    get #privateGetter(): boolean;
    set #privateSetter(value: boolean);
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let error_count = codes.iter().filter(|&&c| c == 2819).count();

    // Should report error for all 5 private identifiers
    assert!(error_count >= 4,
        "Expected at least 4 errors with code 2819 for private identifiers in ambient class, got {} errors: {:?}",
        error_count, codes);
}

#[test]
fn test_private_identifier_in_non_ambient_class_ok() {
    use crate::thin_parser::ThinParserState;

    // Private identifiers should be allowed in non-ambient classes
    let source = r#"
class RegularClass {
    #privateField: string;

    constructor() {
        this.#privateField = "test";
    }

    #privateMethod(): void {
        console.log(this.#privateField);
    }
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let error_2819_count = codes.iter().filter(|&&c| c == 2819).count();

    assert_eq!(error_2819_count, 0,
        "Expected no error 2819 for private identifiers in non-ambient class, got: {:?}", codes);
}

#[test]
fn test_namespace_with_relative_path_ok() {
    use crate::thin_parser::ThinParserState;

    // Namespace declarations (without declare) can have any name, including relative-like names
    // This test ensures we only check ambient modules (declare module)
    let source = r#"
namespace MyNamespace {
    export function foo(): void {}
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let error_5061_count = codes.iter().filter(|&&c| c == 5061).count();

    assert_eq!(error_5061_count, 0,
        "Expected no error 5061 for namespace declarations (only ambient modules should error), got: {:?}", codes);
}

// ============== Top-level scope tests (fixes critical bug) ==============

#[test]
fn test_top_level_variable_redeclaration_different_type_2403() {
    use crate::thin_parser::ThinParserState;

    // Top-level variables with different types should trigger error 2403
    let source = r#"
var x: string;
var x: number;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2403),
        "Expected error 2403 for top-level variable redeclaration with different type, got: {:?}", codes);
}

#[test]
fn test_top_level_variable_redeclaration_same_type_ok() {
    use crate::thin_parser::ThinParserState;

    // Top-level variables with same type should be allowed
    let source = r#"
var x: string;
var x: string;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let error_2403_count = codes.iter().filter(|&&c| c == 2403).count();

    assert_eq!(error_2403_count, 0,
        "Expected no error 2403 for top-level variable redeclaration with same type, got: {:?}", codes);
}

#[test]
fn test_namespace_member_not_found() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace foo {
    export class Provide {}
}
var p: foo.NotExist;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let diags = &checker.ctx.diagnostics;
    let codes: Vec<u32> = diags.iter().map(|d| d.code).collect();

    // Should produce error 2694: Namespace 'foo' has no exported member 'NotExist'
    assert!(codes.contains(&2694), "Expected error 2694 for namespace member not found, got: {:?}", codes);
}

#[test]
fn test_namespace_value_member_missing_errors() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export const ok = 1;
}
import Alias = NS;
const bad = NS.missing;
const badAlias = Alias.missing;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let missing_count = codes.iter().filter(|&&code| code == 2339).count();
    assert_eq!(
        missing_count,
        2,
        "Expected two 2339 errors for missing namespace value members, got: {:?}",
        codes
    );
}

#[test]
fn test_import_alias_type_resolution() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export class Exported {}
    class NotExported {}
}
import Alias = NS.Exported;
var x: Alias;
var y: NS.Exported;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let diags = &checker.ctx.diagnostics;
    let codes: Vec<u32> = diags.iter().map(|d| d.code).collect();

    // Should produce no errors - both x: Alias and y: NS.Exported should resolve correctly
    assert!(codes.is_empty(), "Expected no errors for import alias type resolution, got: {:?}", codes);
}

#[test]
fn test_import_alias_non_exported_member() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export class Exported {}
    class NotExported {}
}
import Alias = NS.NotExported;
var x: Alias;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let diags = &checker.ctx.diagnostics;
    let codes: Vec<u32> = diags.iter().map(|d| d.code).collect();

    // Should produce error 2694: Namespace 'NS' has no exported member 'NotExported'
    // This error occurs when the alias is used (var x: Alias), which triggers type resolution
    assert!(codes.contains(&2694), "Expected error 2694 for import alias of non-exported member, got: {:?}", codes);
}

#[test]
fn test_nested_namespace_member_resolution() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace Outer {
    export namespace Inner {
        export interface Box<T> { value: T; }
    }
}
let ok: Outer.Inner.Box<number> = { value: 1 };
let bad: Outer.Inner.Box<number> = { value: "oops" };
let missing: Outer.Inner.Missing;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();

    assert!(codes.contains(&2694), "Expected error 2694 for missing nested namespace member, got: {:?}", codes);
    assert!(codes.contains(&2322), "Expected error 2322 for nested namespace generic mismatch, got: {:?}", codes);
}

#[test]
fn test_import_alias_namespace_member_resolution() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export interface Box<T> { value: T; }
}
import Alias = NS;
let ok: Alias.Box<number> = { value: 1 };
let bad: Alias.Box<number> = { value: "oops" };
let missing: Alias.Missing;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();

    assert!(codes.contains(&2694), "Expected error 2694 for alias missing member, got: {:?}", codes);
    assert!(codes.contains(&2322), "Expected error 2322 for alias generic mismatch, got: {:?}", codes);
}

#[test]
fn test_namespace_type_only_member_value_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export interface Foo { value: number; }
}
let ok: NS.Foo;
const bad = NS.Foo;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2693),
        "Expected error 2693 for type-only namespace member used as value, got: {:?}",
        codes
    );
}

#[test]
fn test_namespace_type_only_member_element_access_value_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export interface Foo { value: number; }
}
const bad = NS["Foo"];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2693),
        "Expected error 2693 for type-only namespace member element access used as value, got: {:?}",
        codes
    );
}

#[test]
fn test_namespace_type_only_nested_member_value_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace Outer {
    export namespace Inner {
        export interface Foo { value: number; }
    }
}
let ok: Outer.Inner.Foo;
const bad = Outer.Inner.Foo;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let count = codes.iter().filter(|&&code| code == 2693).count();
    assert_eq!(
        count,
        1,
        "Expected one 2693 error for nested type-only namespace member used as value, got: {:?}",
        codes
    );
    assert!(
        !codes.contains(&2339),
        "Did not expect 2339 for nested type-only namespace member used as value, got: {:?}",
        codes
    );
}

#[test]
fn test_namespace_type_only_alias_value_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export interface Foo { value: number; }
}
import Alias = NS.Foo;
const bad = Alias;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2693),
        "Expected error 2693 for type-only namespace alias used as value, got: {:?}",
        codes
    );
}

#[test]
fn test_namespace_type_only_member_via_alias_value_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export interface Foo { value: number; }
}
import Alias = NS;
let ok: Alias.Foo;
const bad = Alias.Foo;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let count = codes.iter().filter(|&&code| code == 2693).count();
    assert_eq!(
        count,
        1,
        "Expected one 2693 error for type-only namespace member via alias, got: {:?}",
        codes
    );
    assert!(
        !codes.contains(&2339),
        "Did not expect 2339 for type-only namespace member via alias, got: {:?}",
        codes
    );
}

#[test]
fn test_namespace_type_only_nested_member_via_alias_value_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace Outer {
    export namespace Inner {
        export type Foo = number;
    }
}
import Alias = Outer;
let ok: Alias.Inner.Foo;
const bad = Alias.Inner.Foo;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let count = codes.iter().filter(|&&code| code == 2693).count();
    assert_eq!(
        count,
        1,
        "Expected one 2693 error for nested type-only namespace member via alias, got: {:?}",
        codes
    );
    assert!(
        !codes.contains(&2339),
        "Did not expect 2339 for nested type-only namespace member via alias, got: {:?}",
        codes
    );
}

#[test]
fn test_interface_value_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Foo { value: number; }
let ok: Foo;
const bad = Foo;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2693),
        "Expected error 2693 for interface used as value, got: {:?}",
        codes
    );
}

#[test]
fn test_type_alias_value_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
type Foo = { value: number };
let ok: Foo;
const bad = Foo;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2693),
        "Expected error 2693 for type alias used as value, got: {:?}",
        codes
    );
}

#[test]
fn test_type_query_interface_value_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
interface Foo { value: number; }
type T = typeof Foo;
let useIt: T;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2693),
        "Expected error 2693 for interface used in type query, got: {:?}",
        codes
    );
}

#[test]
fn test_type_query_type_alias_value_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
type Foo = { value: number };
type T = typeof Foo;
let useIt: T;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2693),
        "Expected error 2693 for type alias used in type query, got: {:?}",
        codes
    );
}

#[test]
fn test_type_query_unknown_name_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
type T = typeof Missing;
let useIt: T;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2304),
        "Expected error 2304 for unknown typeof name, got: {:?}",
        codes
    );
}

#[test]
fn test_type_query_unknown_qualified_name_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
type T = typeof Missing.Member;
let useIt: T;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2304),
        "Expected error 2304 for unknown typeof qualified name, got: {:?}",
        codes
    );
}

#[test]
fn test_type_query_missing_namespace_member_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace Ns {
    export const value = 1;
}
type T = typeof Ns.Missing;
let useIt: T;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2694),
        "Expected error 2694 for missing namespace member in typeof, got: {:?}",
        codes
    );
}

#[test]
fn test_value_symbol_used_as_type_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
const value = 1;
type T = value;
let useIt: T;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2749),
        "Expected error 2749 for value symbol used as type, got: {:?}",
        codes
    );
}

#[test]
fn test_function_symbol_used_as_type_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
function foo() { return 1; }
type T = foo;
let useIt: T;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2749),
        "Expected error 2749 for function symbol used as type, got: {:?}",
        codes
    );
}

#[test]
fn test_namespace_symbol_used_as_type_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export const value = 1;
}
type T = NS;
let useIt: T;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2749),
        "Expected error 2749 for namespace symbol used as type, got: {:?}",
        codes
    );
}

#[test]
fn test_namespace_alias_used_as_type_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export const value = 1;
}
import Alias = NS;
type T = Alias;
let useIt: T;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2749),
        "Expected error 2749 for namespace alias used as type, got: {:?}",
        codes
    );
}

#[test]
fn test_namespace_value_member_used_as_type_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export const value = 1;
}
type T = NS.value;
let useIt: T;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2749),
        "Expected error 2749 for namespace value member used as type, got: {:?}",
        codes
    );
}

#[test]
fn test_namespace_value_member_via_alias_used_as_type_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export const value = 1;
}
import Alias = NS;
type T = Alias.value;
let useIt: T;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2749),
        "Expected error 2749 for namespace value member via alias used as type, got: {:?}",
        codes
    );
}

#[test]
fn test_namespace_value_member_access() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace Outer {
    export const top = 1;
    export namespace Inner {
        export const value = 2;
    }
}
import Alias = Outer.Inner;
const direct = Outer.Inner.value;
const topValue = Outer.top;
const viaAlias = Alias.value;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let direct_sym = binder.file_locals.get("direct").expect("direct should exist");
    let top_sym = binder.file_locals.get("topValue").expect("topValue should exist");
    let alias_sym = binder.file_locals.get("viaAlias").expect("viaAlias should exist");

    assert_eq!(checker.get_type_of_symbol(direct_sym), TypeId::NUMBER);
    assert_eq!(checker.get_type_of_symbol(top_sym), TypeId::NUMBER);
    assert_eq!(checker.get_type_of_symbol(alias_sym), TypeId::NUMBER);
}

#[test]
fn test_namespace_value_member_element_access() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace Ns {
    export const value = 1;
}
import Alias = Ns;
const direct = Ns["value"];
const viaAlias = Alias["value"];
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);
    assert!(checker.ctx.diagnostics.is_empty(), "Unexpected diagnostics: {:?}", checker.ctx.diagnostics);

    let direct_sym = binder.file_locals.get("direct").expect("direct should exist");
    let alias_sym = binder.file_locals.get("viaAlias").expect("viaAlias should exist");

    assert_eq!(checker.get_type_of_symbol(direct_sym), TypeId::NUMBER);
    assert_eq!(checker.get_type_of_symbol(alias_sym), TypeId::NUMBER);
}

#[test]
fn test_namespace_value_member_alias_missing_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace Outer {
    export namespace Inner {
        export const value = 1;
    }
}
import Alias = Outer.Inner;
const ok = Alias.value;
const bad = Alias.missing;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let missing_count = codes.iter().filter(|&&code| code == 2339).count();
    assert_eq!(
        missing_count,
        1,
        "Expected one 2339 error for missing namespace alias member, got: {:?}",
        codes
    );

    let ok_sym = binder.file_locals.get("ok").expect("ok should exist");
    assert_eq!(checker.get_type_of_symbol(ok_sym), TypeId::NUMBER);
}

#[test]
fn test_nested_namespace_value_member_missing_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace Outer {
    export namespace Inner {
        export const ok = 1;
    }
}
const okValue = Outer.Inner.ok;
const badValue = Outer.Inner.missing;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let missing_count = codes.iter().filter(|&&code| code == 2339).count();
    assert_eq!(
        missing_count,
        1,
        "Expected one 2339 error for missing nested namespace value member, got: {:?}",
        codes
    );

    let ok_sym = binder.file_locals.get("okValue").expect("okValue should exist");
    assert_eq!(checker.get_type_of_symbol(ok_sym), TypeId::NUMBER);
}

#[test]
fn test_namespace_value_member_not_exported_error() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace NS {
    export const ok = 1;
    const hidden = 2;
}
const ok = NS.ok;
const bad = NS.hidden;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let missing_count = codes.iter().filter(|&&code| code == 2339).count();
    assert_eq!(
        missing_count,
        1,
        "Expected one 2339 error for non-exported namespace value member, got: {:?}",
        codes
    );

    let ok_sym = binder.file_locals.get("ok").expect("ok should exist");
    assert_eq!(checker.get_type_of_symbol(ok_sym), TypeId::NUMBER);
}

#[test]
fn test_deep_binary_expression_type_check() {
    use crate::thin_parser::ThinParserState;

    const COUNT: usize = 50000;
    let mut source = String::with_capacity(COUNT * 4);
    for i in 0..COUNT {
        if i > 0 {
            source.push_str(" + ");
        }
        source.push('0');
    }
    source.push(';');

    let mut parser = ThinParserState::new("test.ts".to_string(), source);
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    assert!(checker.ctx.diagnostics.is_empty());
}

#[test]
fn test_scoped_identifier_resolution_uses_binder_scopes() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let x = 1;
{
    let x = "hi";
    x;
}
x;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let block_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::BLOCK))
        .expect("block statement");
    let block = arena.get_block(arena.get(block_idx).expect("block node")).expect("block data");
    let inner_expr_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("inner expression statement");
    let inner_expr = arena.get_expression_statement(arena.get(inner_expr_idx).expect("inner expr node"))
        .expect("inner expression data");

    let outer_expr_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("outer expression statement");
    let outer_expr = arena.get_expression_statement(arena.get(outer_expr_idx).expect("outer expr node"))
        .expect("outer expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let inner_type = checker.get_type_of_node(inner_expr.expression);
    let outer_type = checker.get_type_of_node(outer_expr.expression);

    assert_eq!(inner_type, TypeId::STRING);
    assert_eq!(outer_type, TypeId::NUMBER);
}

#[test]
fn test_flow_narrowing_applies_in_if_branch() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let x: string | number;
if (typeof x === "string") {
    x;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let if_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::IF_STATEMENT))
        .expect("if statement");
    let if_node = arena.get(if_idx).expect("if node");
    let if_data = arena.get_if_statement(if_node).expect("if data");

    let then_node = arena.get(if_data.then_statement).expect("then node");
    let block = arena.get_block(then_node).expect("then block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("expression statement");
    let expr_stmt_node = arena.get(expr_stmt_idx).expect("expression node");
    let expr_stmt = arena.get_expression_statement(expr_stmt_node)
        .expect("expression statement data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let narrowed = checker.get_type_of_node(expr_stmt.expression);
    assert_eq!(narrowed, TypeId::STRING);
}

#[test]
fn test_flow_narrowing_not_applied_in_closure() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
let x: string | number;
if (typeof x === "string") {
    const run = () => {
        x.toUpperCase();
    };
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2339),
        "Expected error 2339 for closure without narrowing, got: {:?}",
        codes
    );
}

#[test]
fn test_flow_narrowing_applies_in_while() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let x: string | number;
while (typeof x === "string") {
    x;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let while_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::WHILE_STATEMENT))
        .expect("while statement");
    let while_node = arena.get(while_idx).expect("while node");
    let loop_data = arena.get_loop(while_node).expect("while data");

    let body_node = arena.get(loop_data.statement).expect("while body");
    let block = arena.get_block(body_node).expect("while block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("inner expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("inner expr node"))
        .expect("inner expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let inner_type = checker.get_type_of_node(expr_stmt.expression);
    assert_eq!(inner_type, TypeId::STRING);
}

#[test]
fn test_flow_narrowing_applies_in_for() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let x: string | number;
for (; typeof x === "string"; ) {
    x;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let for_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::FOR_STATEMENT))
        .expect("for statement");
    let for_node = arena.get(for_idx).expect("for node");
    let loop_data = arena.get_loop(for_node).expect("for data");

    let body_node = arena.get(loop_data.statement).expect("for body");
    let block = arena.get_block(body_node).expect("for block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("inner expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("inner expr node"))
        .expect("inner expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let inner_type = checker.get_type_of_node(expr_stmt.expression);
    assert_eq!(inner_type, TypeId::STRING);
}

#[test]
fn test_flow_narrowing_not_applied_in_for_of_body() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let x: string | number;
for (const value of [x]) {
    x;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let for_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::FOR_OF_STATEMENT))
        .expect("for-of statement");
    let for_node = arena.get(for_idx).expect("for-of node");
    let for_data = arena.get_for_in_of(for_node).expect("for-of data");

    let body_node = arena.get(for_data.statement).expect("for-of body");
    let block = arena.get_block(body_node).expect("for-of block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("inner expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("inner expr node"))
        .expect("inner expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let inner_type = checker.get_type_of_node(expr_stmt.expression);
    let expected = checker.ctx.types.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(inner_type, expected);
}

#[test]
fn test_flow_narrowing_not_applied_in_for_in_body() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let x: string | number;
for (const key in { a: x }) {
    x;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let for_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::FOR_IN_STATEMENT))
        .expect("for-in statement");
    let for_node = arena.get(for_idx).expect("for-in node");
    let for_data = arena.get_for_in_of(for_node).expect("for-in data");

    let body_node = arena.get(for_data.statement).expect("for-in body");
    let block = arena.get_block(body_node).expect("for-in block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("inner expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("inner expr node"))
        .expect("inner expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let inner_type = checker.get_type_of_node(expr_stmt.expression);
    let expected = checker.ctx.types.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(inner_type, expected);
}

#[test]
fn test_flow_narrowing_not_applied_in_do_while_body() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
let x: string | number;
do {
    x.toUpperCase();
} while (typeof x === "string");
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2339),
        "Expected error 2339 for do-while body without narrowing, got: {:?}",
        codes
    );
}

#[test]
fn test_flow_narrowing_not_applied_after_while_exit() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let x: string | number;
while (typeof x === "string") {
    break;
}
x;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let expr_stmt_idx = source_file.statements.nodes.iter().copied()
        .filter(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .last()
        .expect("expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("expr node"))
        .expect("expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let after_type = checker.get_type_of_node(expr_stmt.expression);
    let expected = checker.ctx.types.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(after_type, expected);
}

#[test]
fn test_flow_narrowing_not_applied_after_for_exit() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let x: string | number;
for (; typeof x === "string"; ) {
    break;
}
x;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let expr_stmt_idx = source_file.statements.nodes.iter().copied()
        .filter(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .last()
        .expect("expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("expr node"))
        .expect("expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let after_type = checker.get_type_of_node(expr_stmt.expression);
    let expected = checker.ctx.types.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(after_type, expected);
}

#[test]
fn test_flow_narrowing_not_applied_after_do_while_exit() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let x: string | number;
do {
    break;
} while (typeof x === "string");
x;
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let expr_stmt_idx = source_file.statements.nodes.iter().copied()
        .filter(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .last()
        .expect("expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("expr node"))
        .expect("expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let after_type = checker.get_type_of_node(expr_stmt.expression);
    let expected = checker.ctx.types.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(after_type, expected);
}

#[test]
fn test_flow_narrowing_applies_for_namespace_alias_member() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
namespace Ns {
    export let value: string | number;
}
import Alias = Ns;
if (typeof Alias.value === "string") {
    Alias.value;
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let if_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::IF_STATEMENT))
        .expect("if statement");
    let if_node = arena.get(if_idx).expect("if node");
    let if_data = arena.get_if_statement(if_node).expect("if data");

    let then_node = arena.get(if_data.then_statement).expect("then node");
    let block = arena.get_block(then_node).expect("then block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("expr node"))
        .expect("expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let narrowed = checker.get_type_of_node(expr_stmt.expression);
    assert_eq!(narrowed, TypeId::STRING);
}

#[test]
fn test_flow_narrowing_applies_for_namespace_element_access() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
namespace Ns {
    export let value: string | number;
}
if (typeof Ns["value"] === "string") {
    Ns["value"];
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let if_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::IF_STATEMENT))
        .expect("if statement");
    let if_node = arena.get(if_idx).expect("if node");
    let if_data = arena.get_if_statement(if_node).expect("if data");

    let then_node = arena.get(if_data.then_statement).expect("then node");
    let block = arena.get_block(then_node).expect("then block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("expr node"))
        .expect("expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let narrowed = checker.get_type_of_node(expr_stmt.expression);
    assert_eq!(narrowed, TypeId::STRING);
}

#[test]
fn test_flow_narrowing_cleared_by_namespace_member_assignment() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
namespace Ns {
    export let value: string | number;
}
import Alias = Ns;
if (typeof Alias.value === "string") {
    Ns.value = 1;
    Alias.value.toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&2339),
        "Expected error 2339 after namespace member assignment clears narrowing, got: {:?}",
        codes
    );
}

#[test]
fn test_flow_narrowing_cleared_by_property_assignment() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
let obj: { prop: string | number } = { prop: "ok" };
if (typeof obj.prop === "string") {
    obj.prop.toUpperCase();
    obj.prop = 1;
    obj.prop.toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let count = codes.iter().filter(|&&code| code == 2339).count();
    assert_eq!(
        count,
        1,
        "Expected one 2339 after property assignment clears narrowing, got: {:?}",
        codes
    );
}

#[test]
fn test_flow_narrowing_cleared_by_element_assignment() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
let obj: { prop: string | number } = { prop: "ok" };
if (typeof obj["prop"] === "string") {
    obj["prop"].toUpperCase();
    obj["prop"] = 1;
    obj["prop"].toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let count = codes.iter().filter(|&&code| code == 2339).count();
    assert_eq!(
        count,
        1,
        "Expected one 2339 after element assignment clears narrowing, got: {:?}",
        codes
    );
}

#[test]
fn test_flow_narrowing_applies_across_element_to_property_access() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
let obj: { prop: string | number } = { prop: "ok" };
if (typeof obj["prop"] === "string") {
    obj.prop.toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        !codes.contains(&2339),
        "Expected no 2339 when element access narrows property access, got: {:?}",
        codes
    );
}

#[test]
fn test_flow_narrowing_applies_across_property_to_element_access() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
let obj: { prop: string | number } = { prop: "ok" };
if (typeof obj.prop === "string") {
    obj["prop"].toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        !codes.contains(&2339),
        "Expected no 2339 when property access narrows element access, got: {:?}",
        codes
    );
}

#[test]
fn test_flow_narrowing_cleared_by_cross_property_assignment() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
let obj: { prop: string | number } = { prop: "ok" };
if (typeof obj["prop"] === "string") {
    obj.prop.toUpperCase();
    obj.prop = 1;
    obj["prop"].toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let count = codes.iter().filter(|&&code| code == 2339).count();
    assert_eq!(
        count,
        1,
        "Expected one 2339 after cross property assignment clears narrowing, got: {:?}",
        codes
    );
}

#[test]
fn test_flow_narrowing_cleared_by_cross_element_assignment() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
let obj: { prop: string | number } = { prop: "ok" };
if (typeof obj.prop === "string") {
    obj["prop"].toUpperCase();
    obj["prop"] = 1;
    obj.prop.toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let count = codes.iter().filter(|&&code| code == 2339).count();
    assert_eq!(
        count,
        1,
        "Expected one 2339 after cross element assignment clears narrowing, got: {:?}",
        codes
    );
}

#[test]
fn test_flow_narrowing_not_applied_for_computed_element_access() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let obj: { [key: string]: string | number } = { prop: "ok" };
let key: string = "prop";
if (typeof obj[key] === "string") {
    obj[key];
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let if_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::IF_STATEMENT))
        .expect("if statement");
    let if_node = arena.get(if_idx).expect("if node");
    let if_data = arena.get_if_statement(if_node).expect("if data");

    let then_node = arena.get(if_data.then_statement).expect("then node");
    let block = arena.get_block(then_node).expect("then block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("expr node"))
        .expect("expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let expr_type = checker.get_type_of_node(expr_stmt.expression);
    let expected = checker.ctx.types.union(vec![TypeId::STRING, TypeId::NUMBER]);
    assert_eq!(
        expr_type,
        expected,
        "Expected computed element access to remain un-narrowed, got: {:?}",
        expr_type
    );
}

#[test]
fn test_flow_narrowing_applies_for_computed_element_access_literal_key() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let obj: { prop: string | number } = { prop: "ok" };
let key: "prop" = "prop";
if (typeof obj[key] === "string") {
    obj[key].toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let if_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::IF_STATEMENT))
        .expect("if statement");
    let if_node = arena.get(if_idx).expect("if node");
    let if_data = arena.get_if_statement(if_node).expect("if data");

    let then_node = arena.get(if_data.then_statement).expect("then node");
    let block = arena.get_block(then_node).expect("then block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("expr node"))
        .expect("expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let expr_type = checker.get_type_of_node(expr_stmt.expression);
    assert_eq!(
        expr_type,
        TypeId::STRING,
        "Expected computed element access with literal key to narrow to string, got: {:?}",
        expr_type
    );
}

#[test]
fn test_flow_narrowing_cleared_by_computed_element_assignment() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
let obj: { prop: string | number } = { prop: "ok" };
let key: "prop" = "prop";
if (typeof obj[key] === "string") {
    obj[key].toUpperCase();
    obj[key] = 1;
    obj[key].toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let count = codes.iter().filter(|&&code| code == 2339).count();
    assert_eq!(
        count,
        1,
        "Expected one 2339 after computed element assignment clears narrowing, got: {:?}",
        codes
    );
}

#[test]
fn test_flow_narrowing_applies_for_computed_element_access_numeric_literal_key() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let arr: (string | number)[] = ["ok", 1];
let idx: 0 = 0;
if (typeof arr[idx] === "string") {
    arr[idx].toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let if_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::IF_STATEMENT))
        .expect("if statement");
    let if_node = arena.get(if_idx).expect("if node");
    let if_data = arena.get_if_statement(if_node).expect("if data");

    let then_node = arena.get(if_data.then_statement).expect("then node");
    let block = arena.get_block(then_node).expect("then block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("expr node"))
        .expect("expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let expr_type = checker.get_type_of_node(expr_stmt.expression);
    assert_eq!(
        expr_type,
        TypeId::STRING,
        "Expected computed element access with numeric literal key to narrow to string, got: {:?}",
        expr_type
    );
}

#[test]
fn test_flow_narrowing_cleared_by_computed_numeric_element_assignment() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
let arr: (string | number)[] = ["ok", 1];
let idx: 0 = 0;
if (typeof arr[idx] === "string") {
    arr[idx].toUpperCase();
    arr[idx] = 1;
    arr[idx].toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let count = codes.iter().filter(|&&code| code == 2339).count();
    assert_eq!(
        count,
        1,
        "Expected one 2339 after computed numeric element assignment clears narrowing, got: {:?}",
        codes
    );
}

#[test]
fn test_flow_narrowing_applies_for_computed_element_access_const_literal_key() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let obj: { prop: string | number } = { prop: "ok" };
const key = "prop";
if (typeof obj[key] === "string") {
    obj[key].toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let if_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::IF_STATEMENT))
        .expect("if statement");
    let if_node = arena.get(if_idx).expect("if node");
    let if_data = arena.get_if_statement(if_node).expect("if data");

    let then_node = arena.get(if_data.then_statement).expect("then node");
    let block = arena.get_block(then_node).expect("then block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("expr node"))
        .expect("expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let expr_type = checker.get_type_of_node(expr_stmt.expression);
    assert_eq!(
        expr_type,
        TypeId::STRING,
        "Expected computed element access with const literal key to narrow to string, got: {:?}",
        expr_type
    );
}

#[test]
fn test_flow_narrowing_applies_for_computed_element_access_const_numeric_key() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let arr: (string | number)[] = ["ok", 1];
const idx = 0;
if (typeof arr[idx] === "string") {
    arr[idx].toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let if_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::IF_STATEMENT))
        .expect("if statement");
    let if_node = arena.get(if_idx).expect("if node");
    let if_data = arena.get_if_statement(if_node).expect("if data");

    let then_node = arena.get(if_data.then_statement).expect("then node");
    let block = arena.get_block(then_node).expect("then block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("expr node"))
        .expect("expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let expr_type = checker.get_type_of_node(expr_stmt.expression);
    assert_eq!(
        expr_type,
        TypeId::STRING,
        "Expected computed element access with const numeric key to narrow to string, got: {:?}",
        expr_type
    );
}

#[test]
fn test_flow_narrowing_applies_for_computed_element_access_literal_discriminant() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
type U = { kind: "a"; value: string } | { kind: "b"; value: number };
let obj: U = { kind: "a", value: "ok" };
let key: "kind" = "kind";
if (obj[key] === "a") {
    obj.value.toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let if_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::IF_STATEMENT))
        .expect("if statement");
    let if_node = arena.get(if_idx).expect("if node");
    let if_data = arena.get_if_statement(if_node).expect("if data");

    let then_node = arena.get(if_data.then_statement).expect("then node");
    let block = arena.get_block(then_node).expect("then block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("expr node"))
        .expect("expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let expr_type = checker.get_type_of_node(expr_stmt.expression);
    assert_eq!(
        expr_type,
        TypeId::STRING,
        "Expected computed element discriminant to narrow to string, got: {:?}",
        expr_type
    );
}

#[test]
fn test_flow_narrowing_applies_for_literal_element_access() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
let obj: { prop: string | number } = { prop: "ok" };
if (typeof obj["prop"] === "string") {
    obj["prop"];
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");

    let if_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::IF_STATEMENT))
        .expect("if statement");
    let if_node = arena.get(if_idx).expect("if node");
    let if_data = arena.get_if_statement(if_node).expect("if data");

    let then_node = arena.get(if_data.then_statement).expect("then node");
    let block = arena.get_block(then_node).expect("then block");
    let expr_stmt_idx = block.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::EXPRESSION_STATEMENT))
        .expect("expression statement");
    let expr_stmt = arena.get_expression_statement(arena.get(expr_stmt_idx).expect("expr node"))
        .expect("expression data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let expr_type = checker.get_type_of_node(expr_stmt.expression);
    assert_eq!(
        expr_type,
        TypeId::STRING,
        "Expected literal element access to narrow to string, got: {:?}",
        expr_type
    );
}

#[test]
fn test_flow_narrowing_cleared_by_property_base_assignment() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
let obj: { prop: string | number } = { prop: "ok" };
if (typeof obj.prop === "string") {
    obj.prop.toUpperCase();
    obj = { prop: 1 };
    obj.prop.toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let count = codes.iter().filter(|&&code| code == 2339).count();
    assert_eq!(
        count,
        1,
        "Expected one 2339 after property base assignment clears narrowing, got: {:?}",
        codes
    );
}

#[test]
fn test_flow_narrowing_cleared_by_element_base_assignment() {
    use crate::thin_parser::ThinParserState;

    let source = r#"
let obj: { prop: string | number } = { prop: "ok" };
if (typeof obj["prop"] === "string") {
    obj["prop"].toUpperCase();
    obj = { prop: 1 };
    obj["prop"].toUpperCase();
}
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let codes: Vec<u32> = checker.ctx.diagnostics.iter().map(|d| d.code).collect();
    let count = codes.iter().filter(|&&code| code == 2339).count();
    assert_eq!(
        count,
        1,
        "Expected one 2339 after element base assignment clears narrowing, got: {:?}",
        codes
    );
}

#[test]
fn test_parameter_identifier_type_from_symbol_cache() {
    use crate::thin_parser::ThinParserState;
    use crate::parser::syntax_kind_ext;

    let source = r#"
function f(x: number) { return x; }
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    let arena = parser.get_arena();
    let root_node = arena.get(root).expect("root node");
    let source_file = arena.get_source_file(root_node).expect("source file");
    let func_idx = source_file.statements.nodes.iter().copied()
        .find(|&idx| arena.get(idx).map_or(false, |node| node.kind == syntax_kind_ext::FUNCTION_DECLARATION))
        .expect("function declaration");
    let func_node = arena.get(func_idx).expect("function node");
    let func = arena.get_function(func_node).expect("function data");

    let body_node = arena.get(func.body).expect("function body");
    let block = arena.get_block(body_node).expect("function block");
    let return_idx = *block.statements.nodes.first().expect("return statement");
    let return_node = arena.get(return_idx).expect("return node");
    let return_data = arena.get_return_statement(return_node).expect("return data");

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(arena, root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(arena, &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let param_type = checker.get_type_of_node(return_data.expression);
    assert_eq!(param_type, TypeId::NUMBER);
}

#[test]
fn test_generic_library_snippet_compiles_and_checks() {
    use crate::binder::SymbolTable;
    use crate::parallel;

    let source = r#"
type Dictionary<T> = { [key: string]: T };
type ReadonlyDict<T> = { readonly [K in keyof T]: T[K] };
type OptionalDict<T> = { [K in keyof T]?: T[K] };

type Action<T extends string = string> = { type: T };
type PayloadAction<T extends string, P> = { type: T; payload: P };

type Reducer<S, A extends Action = Action> = (state: S, action: A) => S;
type CaseReducer<S, A extends Action> = (state: S, action: A) => S;

type CaseReducers<S, A extends Action = Action> = {
  [T in A["type"]]?: CaseReducer<S, A>;
};

declare function createReducer<S, A extends Action>(
  initial: S,
  reducers: CaseReducers<S, A>
): Reducer<S, A>;

type CounterAction =
  | PayloadAction<"inc", number>
  | PayloadAction<"set", number>;

const reducer = createReducer(0, {
  inc: (state, action) => state + action.payload,
  set: (state, action) => action.payload,
});
"#;

    let program = parallel::compile_files(vec![("lib.ts".to_string(), source.to_string())]);
    let file = &program.files[0];

    let mut file_locals = SymbolTable::new();
    for (name, &sym_id) in program.file_locals[0].iter() {
        file_locals.set(name.clone(), sym_id);
    }
    for (name, &sym_id) in program.globals.iter() {
        if !file_locals.has(name) {
            file_locals.set(name.clone(), sym_id);
        }
    }

    let binder = ThinBinderState::from_bound_state_with_scopes(
        program.symbols.clone(),
        file_locals,
        file.node_symbols.clone(),
        file.scopes.clone(),
        file.node_scope_ids.clone(),
    );

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(&file.arena, &binder, &types, "lib.ts".to_string());
    checker.check_source_file(file.source_file);

    assert!(
        checker.ctx.diagnostics.is_empty(),
        "Unexpected diagnostics: {:?}",
        checker.ctx.diagnostics
    );
}

#[test]
fn test_multi_file_generic_library_snippet_compiles_and_checks() {
    use crate::binder::SymbolTable;
    use crate::parallel;

    let decls = r#"
type Action<T extends string = string> = { type: T };
type PayloadAction<T extends string, P> = { type: T; payload: P };
type Reducer<S, A extends Action = Action> = (state: S, action: A) => S;
type CaseReducer<S, A extends Action> = (state: S, action: A) => S;

type CaseReducers<S, A extends Action = Action> = {
  [T in A["type"]]?: CaseReducer<S, A>;
};

declare function createReducer<S, A extends Action>(
  initial: S,
  reducers: CaseReducers<S, A>
): Reducer<S, A>;
"#;

    let usage = r#"
type CounterAction =
  | PayloadAction<"inc", number>
  | PayloadAction<"set", number>;

const reducer = createReducer(0, {
  inc: (state, action) => state + action.payload,
  set: (state, action) => action.payload,
});
"#;

    let program = parallel::compile_files(vec![
        ("types.ts".to_string(), decls.to_string()),
        ("usage.ts".to_string(), usage.to_string()),
    ]);

    let types = TypeInterner::new();

    for (file_idx, file) in program.files.iter().enumerate() {
        let mut file_locals = SymbolTable::new();
        for (name, &sym_id) in program.file_locals[file_idx].iter() {
            file_locals.set(name.clone(), sym_id);
        }
        for (name, &sym_id) in program.globals.iter() {
            if !file_locals.has(name) {
                file_locals.set(name.clone(), sym_id);
            }
        }

        let binder = ThinBinderState::from_bound_state_with_scopes(
            program.symbols.clone(),
            file_locals,
            file.node_symbols.clone(),
            file.scopes.clone(),
            file.node_scope_ids.clone(),
        );

        let mut checker = ThinCheckerState::new(&file.arena, &binder, &types, file.file_name.clone());
        checker.check_source_file(file.source_file);
        assert!(
            checker.ctx.diagnostics.is_empty(),
            "Unexpected diagnostics in {}: {:?}",
            file.file_name,
            checker.ctx.diagnostics
        );
    }
}
