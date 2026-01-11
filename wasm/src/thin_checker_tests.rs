
#[test]
fn test_ts2339_union_shared_property_no_error() {
    use crate::thin_parser::ThinParserState;

    // Property that exists on ALL union members should NOT produce TS2339
    let source = r#"
interface A { common: string; a: string; }
interface B { common: number; b: number; }

function test(obj: A | B) {
    // This should NOT produce TS2339 because 'common' exists on both A and B
    const result = obj.common;
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

    let ts2339_count = checker.ctx.diagnostics.iter().filter(|d| d.code == 2339).count();
    assert_eq!(ts2339_count, 0,
        "Expected no TS2339 errors for shared union property, got {}: {:?}",
        ts2339_count,
        checker.ctx.diagnostics.iter().filter(|d| d.code == 2339)
            .map(|d| &d.message_text).collect::<Vec<_>>()
    );
}

#[test]
fn test_ts2339_index_signature_allows_any_property() {
    use crate::thin_parser::ThinParserState;

    // String/number index signatures should allow any property access
    let source = r#"
interface StringIndexed {
    [key: string]: number;
    a: number; // explicit property
}

interface NumberIndexed {
    [key: number]: string;
}

function test1(obj: StringIndexed) {
    // These should NOT produce TS2339 - index signature allows any string property
    const x = obj.anyProp;
    const y = obj.anotherProp;
    const z = obj.a; // explicit property
}

function test2(obj: NumberIndexed) {
    // This should NOT produce TS2339 - number index signature allows numeric access
    const x = obj[0];
    const y = obj[42];
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

    let ts2339_count = checker.ctx.diagnostics.iter().filter(|d| d.code == 2339).count();
    assert_eq!(ts2339_count, 0,
        "Expected no TS2339 errors for index signature access, got {}: {:?}",
        ts2339_count,
        checker.ctx.diagnostics.iter().filter(|d| d.code == 2339)
            .map(|d| &d.message_text).collect::<Vec<_>>()
    );
}

#[test]
fn test_ts2339_no_index_signature_error() {
    use crate::thin_parser::ThinParserState;

    // Without index signature, accessing non-existent property should produce TS2339
    let source = r#"
interface NoIndex {
    a: string;
}

function test(obj: NoIndex) {
    // This SHOULD produce TS2339 - property 'b' doesn't exist
    const result = obj.b;
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

    let ts2339_count = checker.ctx.diagnostics.iter().filter(|d| d.code == 2339).count();
    assert_eq!(ts2339_count, 1,
        "Expected 1 TS2339 error for missing property without index signature, got {}: {:?}",
        ts2339_count,
        checker.ctx.diagnostics.iter().filter(|d| d.code == 2339)
            .map(|d| &d.message_text).collect::<Vec<_>>()
    );
}

#[test]
fn test_ts2339_nullable_union_with_optional_chaining() {
    use crate::thin_parser::ThinParserState;

    // Test union with null/undefined using optional chaining
    let source = r#"
interface A { a: string; }

function test(obj: A | null) {
    // With optional chaining, this should NOT produce TS2339
    const result = obj?.a;

    // Without optional chaining, this SHOULD produce TS2339 for non-null property access
    // (though it might produce a different error about possible null)
    const result2 = obj.a;
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

    // obj?.a should NOT produce TS2339
    let ts2339_errors: Vec<_> = checker.ctx.diagnostics.iter()
        .filter(|d| d.code == 2339)
        .collect();

    // The optional chaining case should not have TS2339
    // obj.a might have other diagnostics but not TS2339 for property access
    assert!(ts2339_errors.is_empty(),
        "Expected no TS2339 errors, got {}: {:?}",
        ts2339_errors.len(),
        ts2339_errors.iter().map(|d| &d.message_text).collect::<Vec<_>>()
    );
}

#[test]
fn test_ts2339_intersection_property_access() {
    use crate::thin_parser::ThinParserState;

    // Test property access on intersection types
    let source = r#"
type A = { a: string };
type B = { b: number };
type AB = A & B;

function test(obj: AB) {
    // These should NOT produce TS2339 - intersection has both properties
    const x = obj.a;
    const y = obj.b;
}

function test2(obj: A & { c: boolean }) {
    // These should NOT produce TS2339
    const x = obj.a;
    const y = obj.c;
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

    let ts2339_count = checker.ctx.diagnostics.iter().filter(|d| d.code == 2339).count();
    assert_eq!(ts2339_count, 0,
        "Expected no TS2339 errors for intersection property access, got {}: {:?}",
        ts2339_count,
        checker.ctx.diagnostics.iter().filter(|d| d.code == 2339)
            .map(|d| &d.message_text).collect::<Vec<_>>()
    );
}
=======
>>>>>>> origin/worker/anvil-3
=======
fn test_overload_arg_count_exceeds_all_only_ts2554_not_ts2769() {
    use crate::thin_parser::ThinParserState;

    // Regression test for overload calls where argument count exceeds ALL signatures
    // When all overloads fail due to argument count mismatch, should emit TS2554 only, not TS2769
    let code = r#"
declare function mixed(x: string): void;
declare function mixed(x: number, y: number): void;

// This call has 3 arguments, which exceeds both overloads (1 param and 2 params)
// Should emit TS2554 (argument count mismatch) only, not TS2769
mixed(42, 99, 100);
"#;

    let mut parser = ThinParserState::new("test.ts".to_string(), code.to_string());
    let root = parser.parse_source_file();
    assert!(parser.get_diagnostics().is_empty());

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(parser.get_arena(), root);

    let types = TypeInterner::new();
    let mut checker = ThinCheckerState::new(parser.get_arena(), &binder, &types, "test.ts".to_string());
    checker.check_source_file(root);

    let ts2554_errors: Vec<_> = checker.ctx.diagnostics.iter().filter(|d| d.code == 2554).collect();
    let ts2769_errors: Vec<_> = checker.ctx.diagnostics.iter().filter(|d| d.code == 2769).collect();

    // Should have TS2554 (argument count mismatch)
    assert!(
        !ts2554_errors.is_empty(),
        "Should emit TS2554 for argument count mismatch when all overloads fail due to arg count"
    );

    // Should NOT have TS2769 (No overload matches)
    assert!(
        ts2769_errors.is_empty(),
        "Should not emit TS2769 when all overloads fail due to argument count mismatch, got {} TS2769 errors: {:?}",
        ts2769_errors.len(),
        ts2769_errors.iter().map(|d| &d.message_text).collect::<Vec<_>>()
    );

    // Verify TS2554 message
    let first_error_msg = &ts2554_errors[0].message_text;
    assert!(
        first_error_msg.contains("Expected") && first_error_msg.contains("arguments"),
        "TS2554 message should mention expected arguments, got: {}",
        first_error_msg
    );
}
>>>>>>> origin/worker/anvil-4
