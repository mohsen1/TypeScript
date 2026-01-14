# Worker 9 Task List - Rust Migration (Zang)

**Target Branch**: `rust`
**Worktree**: `/private/tmp/orchestrator-workspace/worktrees/worker-9`
**Manager**: EM-3

## Project Context

The TypeScript compiler is being rewritten in Rust (codename: "Zang"). Current status:
- **Conformance**: 30.8% exact match
- **Performance**: 41.7 tests/sec
- **Rust Code**: 182 files in `wasm/` directory

---

## Phase 8 Priorities (January 2026)

### CRITICAL PATH: Binding & Scope Resolution (TS2304)

**Impact**: Global scope issues cause "Any poisoning" that masks other bugs.

#### ~~Task 1: Fix Global Scope and Lib Injection~~ ✅ COMPLETE
- **Problem**: `Cannot find name 'console'` errors in standard lib tests
- **Location**: `wasm/src/binder/thin_binder.rs`
- **Files**: Check `global_this` handling, `lib.d.ts` injection logic
- **Acceptance**:
  - All `lib.d.ts` globals resolve without TS2304
  - `console.log`, `Math`, `Object`, etc. work in tests
  - No regressions in existing passing tests
- **Status**: ✅ Fixed in commit 6401587ca
  - Modified `wasm/src/cli/driver.rs` to load default lib.d.ts files during binding
  - Ensures global symbols are available in globals table for type checking

#### ~~Task 2: Fix Scope Chain Resolution~~ ✅ COMPLETE (No Bug Found)
- **Problem**: Variables in outer scopes not found in nested closures
- **Location**: `wasm/src/binder/scope.rs`, `thin_binder.rs`
- **Acceptance**:
  - Closure variable capture works correctly
  - Block scoping (`let`/`const`) is isolated
  - Module scope boundaries respected
- **Status**: ✅ Investigated in commit 68a4acb66
  - Added test `test_closure_variable_capture` (PASS)
  - Finding: Scope chain resolution is working correctly
  - Variables in outer scopes are properly resolvable inside closures

---

### HIGH PRIORITY: Control Flow Analysis (TS2454/TS2564)

#### ~~Task 3: Variable Initialization Checking~~ ✅ COMPLETE (Already Implemented)
- **Error**: TS2454 - "Variable is used before being assigned"
- **Status**: ✅ Already implemented (verified in commit b253d673a)
  - TS2454 checks working in `get_type_of_identifier` (thin_checker.rs:5289-5292)
  - Tests added: `test_ts2454_variable_used_before_assigned` (PASS)
  - Flow graph and definite assignment analysis already exist
  - Note: `check_flow_usage` function is dead code; actual check is inline

#### Task 4: Property Initialization (Class Fields)
- **Error**: TS2564 - "Property not initialized in constructor"
- **Location**: `wasm/src/checker/thin_checker.rs`
- **Acceptance**:
  - Class fields without init flagged
  - Definite assignment analysis (`!`) works
  - Optional properties excluded

---

### HIGH PRIORITY: Solver Strictness (TS2322)

#### Task 5: Switch Fallback from `Any` to `Unknown`
- **Problem**: Failed inferences fall back to `Any`, hiding bugs
- **Location**: `wasm/src/solver/`
- **Files**: Check inference failure handling, constraint solving
- **Acceptance**:
  - Unknown type used for failed inferences
  - Better error messages on type mismatches
  - No regression in valid inferences

---

### MEDIUM PRIORITY: Parser Error Recovery

#### Task 6: Fix False Positive Syntax Errors
- **Errors**: TS1005/TS1109 - "Expected '}'" on valid code
- **Location**: `wasm/src/parser/scanner.rs`, `thin_parser.rs`
- **Acceptance**:
  - Recover from missing semicolons
  - Handle trailing commas gracefully
  - ASI (Automatic Semicolon Insertion) robust

---

## Testing Requirements

ALL tasks must pass:

```bash
# Rust unit tests
./wasm/test.sh

# Quick conformance check
./wasm/differential-test/run-conformance.sh --max=1000

# Specific test category
./wasm/scripts/run-single-test.mjs tests/cases/compiler/<test_name>.ts
```

---

## Workflow Rules

1. **One task per commit**
2. **Commit message format**: `[Task N] <description>`
3. **After each task**: Run `./wasm/test.sh` and conformance
4. **Document regressions**: Add known issues to `wasm/README.md`
5. **Ask for help**: If blocked > 30 minutes, escalate to EM-3

---

## Current Status

**Status**: ACTIVE - Working on Task 4

**Last Completed**:
- Task 3: Variable Initialization Checking (verified already implemented)
- Task 2: Fix Scope Chain Resolution (investigated, no bug found)
- Task 1: Fix Global Scope and Lib Injection (fixed)

**Next Task**: Task 4 (Property Initialization - TS2564)

---

*This file is maintained by EM-3. Worker 9 executes tasks sequentially.*
