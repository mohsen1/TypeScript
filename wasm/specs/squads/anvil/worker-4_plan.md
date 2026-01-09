# Anvil Worker 4 - Parser Edge Cases (TS1005/TS1068)

## Operation Conformance Assignment

**Mission**: Fix false positive parser errors TS1005 and TS1068 for valid TypeScript syntax.

**Target Errors**:
- TS1005 "';' expected" - ~35 false positives
- TS1068 "Unexpected token" - ~25 false positives
**Root Cause**: Parser doesn't handle all valid TypeScript syntax edge cases

## Problem Analysis

The WASM parser incorrectly reports errors for valid code:

```typescript
// TS1005 false positive - arrow function in object
const obj = {
  handler: () => { }  // Parser expects ';' incorrectly
};

// TS1068 false positive - type assertions
const x = <string>value;  // Old-style type assertion
```

## Implementation Tasks

### Task 1: Audit Parser Error Sites
**File**: `wasm/src/parser/mod.rs`

1. Search for where TS1005 is emitted
2. Search for where TS1068 is emitted
3. Identify patterns that incorrectly trigger these

### Task 2: Fix Arrow Function Parsing in Objects
**File**: `wasm/src/parser/mod.rs`

Arrow functions in object literals need special handling:
```rust
fn parse_object_literal_element(&mut self) -> Result<Node, Error> {
    // Handle shorthand, method, getter/setter, spread
    // CRITICAL: Arrow function property values
    if self.is_arrow_function_expression() {
        return self.parse_property_assignment_with_arrow();
    }
    // ...
}
```

### Task 3: Fix Type Assertion Parsing
**File**: `wasm/src/parser/mod.rs`

Handle angle-bracket type assertions:
```rust
fn parse_unary_expression(&mut self) -> Result<Node, Error> {
    // Check for <Type> assertion (not JSX in .ts files)
    if self.token() == Token::LessThan && !self.is_jsx_context() {
        return self.parse_type_assertion();
    }
    // ...
}
```

### Task 4: Fix Other Common Edge Cases

1. **Generic arrow functions in JSX context**:
   ```typescript
   const f = <T,>(x: T) => x; // Trailing comma disambiguates from JSX
   ```

2. **Computed property names with expressions**:
   ```typescript
   const obj = {
     [Symbol.iterator]() { }
   };
   ```

3. **Optional chaining with method calls**:
   ```typescript
   obj?.method();
   ```

### Task 5: Write Regression Tests
**File**: `wasm/src/parser/tests.rs`

```typescript
// Test 1: Arrow in object
const x = { f: () => 1 };

// Test 2: Type assertion
const y = <number>someValue;

// Test 3: Generic arrow
const id = <T,>(x: T): T => x;

// Test 4: Computed property
const obj = { [key]: value };

// Test 5: Optional chain
result?.method?.();

// Test 6: Nullish coalescing
const val = a ?? b;
```

## Success Criteria

- [ ] Arrow functions in objects parse correctly
- [ ] Type assertions parse correctly
- [ ] Generic arrow functions parse correctly
- [ ] TS1005/TS1068 false positives drop by 40+ occurrences

## Files to Modify

1. `wasm/src/parser/mod.rs` - Main parser fixes
2. `wasm/src/parser/scanner.rs` - If token handling needs updates
3. `wasm/src/parser/expressions.rs` - Expression parsing
4. Test files as needed

## Verification

Run after changes:
```bash
node wasm/differential-test/conformance-runner.mjs --max=200 -v 2>&1 | grep -E "TS1005|TS1068"
```

Target: Reduce TS1005+TS1068 false positives from 60 to <20.

## Progress
- Updated `wasm/src/thin_parser.rs` to allow `var` as a class member name and to broaden angle-bracket type assertion detection.
- Added parser regression tests in `wasm/src/thin_parser_tests.rs` for arrow functions in object literals, angle-bracket type assertions (including literal types), TSX generic arrows with trailing commas, and class members named `var`.
- Pending: run conformance runner; push to origin blocked by SSH permission (git@github.com: Permission denied).

## Status
Active (changes committed; push blocked)

## Notes
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
