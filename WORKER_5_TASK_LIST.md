# Worker 5 Task List

## Squad: Parser/Scanner - TS1005 Focus

## Current Task - Patterns 1-2 (Property Semicolons & Return Types)
**Reference:** `TS1005_REDUCTION_RESULTS.md` - Patterns 1-3 from Worker 1 (TypeScript)

### Pattern 1 & 2: Property Semicolon Handling
**TypeScript Reference:** `src/compiler/parser.ts:3228-3263` - `parseSemicolonAfterPropertyName()`

**Problem:** Duplicate TS1005 emissions when semicolons are missing in object/class properties with type annotations and initializers.

**Solution:**
- Consolidate error emission to avoid duplicate TS1005 reports
- Rely on semicolon parsing which handles ASI correctly
- Remove early error emission before ASI checks

**Rust Implementation Locations:**
- `wasm/src/thin_parser.rs` - Find property parsing functions
- Look for: `parse_property_declaration`, `parse_class_element`, or similar
- Search for multiple `error_token_expected` calls in same function

### Pattern 3: Return Type Arrow Function Confusion
**TypeScript Reference:** `src/compiler/parser.ts:4878-4894` - `shouldParseReturnType()`

**Problem:** When users write `function f() => T` instead of `function f(): T`, parser emits TS1005 but recovers successfully.

**Solution:**
- Remove TS1005 emission when `=>` is used instead of `:` for return types
- Parser still recovers successfully without the error

**Rust Implementation Locations:**
- `wasm/src/thin_parser.rs` - Find return type parsing
- Look for: `parse_return_type`, `parse_function_type`, or similar
- Find where `=>` token check happens after expecting `:`

## Queue
- [ ] Test parser changes on conformance suite to measure reduction
- [ ] Coordinate with Worker 6 to avoid duplicate work
- [ ] Document any Rust-specific patterns discovered

## Completed
- [x] Initial merge attempt - No commits yet

## Context
TS1005 is the #1 source of parser false positives (conformance shows 42 extra errors in sample). Worker 1 fixed 5 patterns in TypeScript; adapt these to Rust.

### Key Files
- `wasm/src/thin_parser.rs` - main parser implementation (lines ~363-7219 have error emission)
- `wasm/src/scanner.rs` - lexical scanner
- `src/compiler/parser.ts` - TypeScript reference implementation
- `TS1005_REDUCTION_RESULTS.md` - Detailed pattern analysis

### Search Targets in Rust
```rust
// Find TS1005 emissions:
grep -n "error_token_expected\|parse_error_at_current_token.*TOKEN_EXPECTED" wasm/src/thin_parser.rs

// Current emission locations:
- Line 404: error_token_expected() helper
- Line 5597: "'=>' expected"
- Many other parse_error_at_current_token calls
```

### Goal
Adapt TypeScript patterns 1-3 to Rust, reduce TS1005 from current levels toward <50.
