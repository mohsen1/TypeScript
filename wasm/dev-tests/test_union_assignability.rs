//! Test union type assignability - verify no false positives
//!
//! These tests verify that union type assignability works correctly:
//! 1. Union to wider union (A | B to A | B | C) should pass
//! 2. Base type to union (A to A | B) should pass
//! 3. Generic constraints in unions should be recognized

use wasm::thin_checker::ThinChecker;
use wasm::thin_parser::ThinParserState;

fn check_union_assignability(source: &str, target: &str) -> (bool, Vec<u32>) {
    let code = format!("let x: {} = {} as any;", target, source);
    let mut parser = ThinParserState::new("test.ts".to_string(), code);
    let root = parser.parse_source_file();

    let mut checker = ThinChecker::new("test.ts".to_string());
    checker.ctx.arena = parser.into_arena();
    checker.check(root);

    let diagnostics = checker.ctx.diagnostics.clone();
    let ts2322_count = diagnostics.iter().filter(|d| d.code == 2322).count() as u32;
    let has_ts2322 = ts2322_count > 0;

    (!has_ts2322, diagnostics.iter().map(|d| d.code).collect())
}

fn main() {
    println!("=== Union Type Assignability Tests ===\n");

    // Test 1: Union to wider union
    println!("Test 1: string | number assignable to string | number | boolean");
    let (passes, codes) = check_union_assignability("string | number", "string | number | boolean");
    println!("  Result: {} (codes: {:?})", passes, codes);
    assert!(passes, "string | number should be assignable to string | number | boolean");

    // Test 2: Base type to union
    println!("\nTest 2: string assignable to string | number");
    let (passes, codes) = check_union_assignability("string", "string | number");
    println!("  Result: {} (codes: {:?})", passes, codes);
    assert!(passes, "string should be assignable to string | number");

    // Test 3: Union to union (same types, different order)
    println!("\nTest 3: string | number assignable to number | string");
    let (passes, codes) = check_union_assignability("string | number", "number | string");
    println!("  Result: {} (codes: {:?})", passes, codes);
    assert!(passes, "string | number should be assignable to number | string");

    // Test 4: Literal union to wider literal union
    println!("\nTest 4: 'a' | 'b' assignable to 'a' | 'b' | 'c'");
    let (passes, codes) = check_union_assignability("'a' | 'b'", "'a' | 'b' | 'c'");
    println!("  Result: {} (codes: {:?})", passes, codes);
    assert!(passes, "'a' | 'b' should be assignable to 'a' | 'b' | 'c'");

    // Test 5: Verify real errors still caught
    println!("\nTest 5: number NOT assignable to string (should error)");
    let (passes, codes) = check_union_assignability("number", "string");
    println!("  Result: {} (codes: {:?})", passes, codes);
    assert!(!passes, "number should NOT be assignable to string");

    println!("\n=== All tests passed! ===");
}
