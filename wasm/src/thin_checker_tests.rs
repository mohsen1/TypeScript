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
    assert!(checker.diagnostics.is_empty());
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
    assert!(checker.types.lookup(TypeId::STRING).is_some());
    assert!(checker.types.lookup(TypeId::NUMBER).is_some());
    assert!(checker.types.lookup(TypeId::ANY).is_some());
}

#[test]
fn test_thin_checker_structural_equality() {
    let arena = ThinNodeArena::new();
    let binder = ThinBinderState::new();
    let types = TypeInterner::new();
    let checker = ThinCheckerState::new(&arena, &binder, &types, "test.ts".to_string());

    // Test structural equality via TypeInterner
    // Same string literal should get same TypeId
    let str1 = checker.types.literal_string("hello");
    let str2 = checker.types.literal_string("hello");
    let str3 = checker.types.literal_string("world");

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
    let with_any = checker.types.union(vec![TypeId::STRING, TypeId::ANY]);
    assert_eq!(with_any, TypeId::ANY);

    // Union with `never` should exclude `never`
    let with_never = checker.types.union(vec![TypeId::STRING, TypeId::NEVER]);
    assert_eq!(with_never, TypeId::STRING);

    // Single-element union should return the element
    let single = checker.types.union(vec![TypeId::STRING]);
    assert_eq!(single, TypeId::STRING);
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
    let hello = checker.types.literal_string("hello");
    assert!(checker.is_assignable_to(hello, TypeId::STRING));

    // Number literal is subtype of number
    let forty_two = checker.types.literal_number(42.0);
    assert!(checker.is_assignable_to(forty_two, TypeId::NUMBER));

    // Boolean literal is subtype of boolean
    let t = checker.types.literal_boolean(true);
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
    let lit1 = checker.types.literal_string("test");
    let lit2 = checker.types.literal_string("test");
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

    let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
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

    let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
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

    let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2389) || codes.contains(&2391),
        "Expected error 2389 or 2391 for wrong implementation name, got: {:?}", codes);
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

    let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
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

    let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
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

    let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
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

    let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
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

    let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
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
    let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
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
    let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
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
    for d in &checker.diagnostics {
        eprintln!("  code={}, msg={}", d.code, d.message_text);
    }

    // Should have error 2511 for `new A()` but not for `new B()`
    let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
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
    for d in &checker.diagnostics {
        eprintln!("  code={}, msg={}", d.code, d.message_text);
    }

    let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2662),
        "Expected error 2662 (Cannot find name 'foo'. Did you mean the static member 'C.foo'?), got: {:?}", codes);

    // Should NOT have generic "cannot find name" error 2304
    assert!(!codes.contains(&2304),
        "Should not have generic error 2304, should have specific 2662 instead. Got: {:?}", codes);
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
    for d in &checker.diagnostics {
        eprintln!("  code={}, msg={}", d.code, d.message_text);
    }

    let codes: Vec<u32> = checker.diagnostics.iter().map(|d| d.code).collect();
    assert!(codes.contains(&2427),
        "Expected error 2427 (Interface name cannot be 'string'), got: {:?}", codes);
}
