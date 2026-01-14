# Worker 8 Task List

**Maintained by:** EM-2
**Branch:** worker-8 → rust
**Worktree:** /tmp/orchestrator-workspace/worktrees/worker-8

---

## Completed Tasks

### ✅ Task 1: TS2454 - Variable Use Before Assignment Detection

**Status:** COMPLETED (2026-01-14)
**Commit:** c0c454e33
**Error Code:** TS2454 ("Variable '{0}' is used before being assigned")

#### What Was Fixed
Fixed critical bug where TS2454 errors were not reported for variables used as arguments in call expressions when the callee was not found (returning ANY/ERROR type).

#### Root Cause
The `get_type_of_call_expression` function returned early when `callee_type` was ANY or ERROR, **before checking the arguments**. This meant variables used as arguments were never checked for definite assignment violations.

#### Solution Implemented
- Moved `args` extraction to occur before the ANY/ERROR check
- Added argument type checking even in the ANY/ERROR case using a dummy context helper
- Added 4 comprehensive TS2454 test cases

#### Files Modified
- `wasm/src/thin_checker.rs` - Fixed early return logic
- `wasm/src/thin_checker_tests.rs` - Added 4 test cases

#### Test Results
- All 4 TS2454 tests pass
- Fix ensures TS2454 errors are properly reported for:
  ```typescript
  function test() {
      let x: string;
      console.log(x);  // Now correctly reports TS2454
  }
  ```

---

## Current Task (IN PROGRESS)

### Task 2: Implement TS2564 - Property Initialization Detection

**Priority:** HIGH (Priority #1 from README)
**Error Code:** TS2564 ("Property '{0}' has no initializer and is not assigned in constructor")
**Impact:** 443 missing errors in conformance tests
**Location:** `wasm/src/checker/`, `wasm/src/thin_checker.rs`
**Dependencies:** Task 1 complete (reuses CFA infrastructure)

#### Background
Class properties must be either:
1. Initialized in the declaration: `prop: string = "default"`
2. Assigned in the constructor: `this.prop = "value"`
3. Marked with definite assignment assertion: `prop!: string`

The compiler currently fails to report TS2564 when properties are declared without initialization and not assigned in the constructor.

#### Requirements
1. Check class properties during type checking
2. Track property assignments in constructor body
3. Use Control Flow Analysis to verify definite assignment on all paths
4. Allow `!` definite assignment assertion to suppress the error

#### Acceptance Criteria
- [ ] Properties without initializers report TS2564 if not assigned in constructor
- [ ] Properties with `!` assertion do NOT report TS2564
- [ ] Properties with initializers do NOT report TS2564
- [ ] Abstract classes do NOT report TS2564
- [ ] `cargo test --lib` passes in wasm/ directory

#### Testing Strategy
```typescript
// Should report TS2564
class Foo {
    name: string;  // Error: Property 'name' has no initializer
}

// Should NOT report TS2564 (has initializer)
class Bar {
    name: string = "default";
}

// Should NOT report TS2564 (assigned in constructor)
class Baz {
    name: string;
    constructor() {
        this.name = "value";
    }
}

// Should NOT report TS2564 (definite assignment assertion)
class Qux {
    name!: string;
}
```

#### Implementation Notes
- Property checking occurs in `check_class_declaration`
- The `should_check_property_initialization` function already exists
- The `emit_property_initialization_error` function already exists
- May need to connect these to the flow analysis from Task 1

---

## Queue (Future Tasks)

### Task 3: Fix TS2322 - Solver Strictness Improvements
**Priority:** MEDIUM (Priority #2 from README)
**Error Code:** TS2322 ("Type '{0}' is not assignable to type '{1}'")
**Impact:** 310 missing errors
**Location:** `wasm/src/solver/`
**Approach:** Change solver fallback from `Any` to `Unknown/Error`

### Task 4: Reduce TS2339 False Positives
**Priority:** MEDIUM (Priority #3 from README)
**Error Code:** TS2339 ("Property '{0}' does not exist on type '{1}'")
**Impact:** 292 extra errors (false positives)
**Approach:** Improve type narrowing, implement apparent members for primitives

---

## Anti-Priorities (DO NOT WORK ON)
- New emitter transforms (ES3, obscure module formats)
- LSP features (semantic tokens, code actions)
- CLI argument parsing
- Performance micro-optimizations

---

## Status Updates
- **Created:** 2026-01-14
- **Last Updated:** 2026-01-14
- **Current Focus:** TS2564 Property Initialization Detection
- **Progress:** 1/4 tasks complete (25%)
