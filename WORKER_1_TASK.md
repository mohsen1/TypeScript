# Worker 1 Task

## Current Task

**Director: Plan and distribute work**

You are the Director. Read PROJECT_DIRECTION.md and TEAM_STRUCTURE.md to understand the project state.

PROJECT_DIRECTION.md contents:
# PROJECT Zang

## Mission

Project Zang is a complete rewrite of the TypeScript compiler and type checker in Rust, compiled to WebAssembly. The goal is to achieve performance improvements while maintaining compatibility with the original TypeScript compiler.

## Architecture Overview

**Core Principle:** TypeScript source files (`src/`) remain **read-only** and identical to upstream Microsoft TypeScript. All custom implementation lives in the `wasm/` directory.


### Never Break The Build

- No commit should break the build or cause test failures
- All changes must pass the unit tests 
- No change should reduce conformance test accuracy

### Key Components

| Component | Location | Purpose |
|-----------|----------|---------|
| Parser | `wasm/src/thin_parser.rs` | Rust implementation of TypeScript parser |
| Checker | `wasm/src/thin_checker.rs` | Type checking and semantic analysis |
| Solver | `wasm/src/solver/` | Type resolution and constraint solving |
| Binder | `wasm/src/binder/` | Symbol binding and scope management |
| Diagnostics | `wasm/src/checker/types/diagnostics.rs` | Error codes and messages |

### Spec Documents

- `wasm/specs/WASM_ARCHITECTURE.md` - Architecture deep dive
- `wasm/specs/SOLVER.md` - Type solver design
- `wasm/specs/` - Other component designs

---

## Running Conformance Tests

To get current metrics, run the conformance test suite:

```bash
# Navigate to differential test directory
cd wasm/differential-test

# Quick test (200 files, ~1 min)
bash run-conformance.sh --max=200 --workers=4

# Standard test (500 files, ~3 min)
bash run-conformance.sh --max=500 --workers=8

# Full test (all files, ~15 min)
bash run-conformance.sh --all --workers=14
```

### Understanding Results

| Metric | Meaning |
|--------|---------|
| Exact Match | WASM and TSC emit identical error codes |
| Same Error Count | Same number of errors (may differ in codes) |
| Missing Errors | TSC emits but WASM doesn't (under-reporting) |
| Extra Errors | WASM emits but TSC doesn't (over-reporting) |
| Crashed | WASM panicked during test |

**Target:** 95%+ exact match before production release.

### Building WASM

```bash
cd wasm
wasm-pack build --target web --out-dir pkg
```

---

## Priority Issues

### Tier 0: Quality & Stability Foundations

**Goal:** Fix cross-cutting gaps that block correctness across all tiers. LSP strictness work can be deferred.

| Issue | Description | Owner | 
|-------|-------------|-------|
| Application type expansion | `TypeKey::Application` is not expanded, leading to incorrect diagnostics/assignability | Unassigned |
| Readonly types | `readonly` arrays/tuples are currently treated as mutable | Unassigned |
| AST child enumeration | `get_children` returns empty in parser arenas, breaking traversal-based features | Unassigned |
| Solver test coverage | `infer/subtype/evaluate` tests are commented out due to API drift | Unassigned |
| Panic hardening | Non-test paths still `panic!/unwrap` instead of recovering or re-parsing | Unassigned |
| Definite assignment gaps | TS2565 not implemented; interface type parameters TODO | Unassigned |

**Key Files:** `wasm/src/solver/evaluate.rs`, `wasm/src/solver/intern.rs`, `wasm/src/parser/arena.rs`, `wasm/src/parser/thin_node.rs`, `wasm/src/solver/subtype.rs`, `wasm/src/solver/infer.rs`, `wasm/src/cli/driver.rs`, `wasm/src/interner.rs`, `wasm/src/thin_checker.rs`

**Notes:**
- Application type expansion should reuse existing instantiation logic (`wasm/src/solver/instantiate.rs`) and re-enable solver tests once updated.
- Readonly semantics need distinct type representation plus assignability/write checks.
- AST children should be enumerated per-node-kind for deterministic traversals (do not allocate large temporary trees).

**Deferred (OK to postpone):** LSP strictness from tsconfig (hover/completions/signature/diagnostics) until Tier 0 and Tier 1 stabilize.

### Tier 1: Parser Accuracy

**Goal:** Parser should accept all valid TypeScript syntax without emitting false errors.

| Issue | Description | Owner |
|-------|-------------|-------|
| TS1109 extra | Parser emits "Expression expected" for valid syntax | Unassigned |
| TS1005 extra | Parser emits "X expected" for valid constructs | Unassigned |
| ASI handling | Automatic semicolon insertion edge cases | Unassigned |

**Key Files:** `wasm/src/thin_parser.rs`

**Validation:** Run conformance tests and check "Extra Errors" for TS1xxx codes.

---

### Tier 2: Type Checker Accuracy

**Goal:** Checker should emit the same semantic errors as TSC.

| Issue | Description | Owner |
|-------|-------------|-------|
| TS2571 extra | "Object is of type 'unknown'" over-reported | Unassigned |
| TS2683 missing | "'this' implicitly has type 'any'" not emitted | Unassigned |
| TS2507 incomplete | Non-constructor extends not fully checked | Unassigned |
| TS2348 extra | "Cannot invoke expression" over-reported | Unassigned |
| TS2322 accuracy | Type assignability false positives | Unassigned |

**Key Files:** `wasm/src/thin_checker.rs`, `wasm/src/solver/`

**Root Cause - TS2571/TS2683:** When `this` is used inside a regular function (not a method), it should emit TS2683 but instead types as `unknown` and emits TS2571 on property access.

```typescript
function foo() {
    this.x = 1;  // Should: TS2683, Currently: TS2571
}
```

**Fix Location:** `thin_checker.rs` - `current_this_type()` handling around line 629

---

### Tier 3: Symbol Resolution

**Goal:** All symbols should resolve correctly, including globals and modules.

| Issue | Description | Owner |
|-------|-------------|-------|
| TS2304 gaps | "Cannot find name" for valid symbols | Unassigned |
| TS2524 missing | Module member resolution failures | Unassigned |
| Global merging | Interface/namespace merging across files | Unassigned |

**Key Files:** `wasm/src/binder/`, `wasm/src/thin_checker.rs`

---

### Tier 4: Implicit Any Checks

**Goal:** Emit TS7006/TS7008 only when type cannot be inferred.

| Issue | Description | Owner |
|-------|-------------|-------|
| TS7006 extra | Parameter implicit any over-reported | Unassigned |
| TS7005 extra | Variable implicit any over-reported | Unassigned |

**Key Files:** `wasm/src/thin_checker.rs` - implicit any checking functions

**Rule:** Skip implicit any errors when:
- Parameter has default value (`param.initializer.is_some()`)
- Property has initializer (`prop.initializer.is_some()`)
- Type can be inferred from usage

---

### Tier 5: Async/Await

**Goal:** Correct handling of async functions, generators, and await expressions.

| Issue | Description | Owner |
|-------|-------------|-------|
| TS2705 gaps | Async function return type checking | Unassigned |
| TS1359 missing | 'await' reserved word detection | Unassigned |
| Async generators | `AsyncGenerator` vs `Promise` return types | Unassigned |

**Key Files:** `wasm/src/thin_checker.rs` - async-related functions

---

## Team Assignment Guidelines

### How to Assign Work

1. **Run conformance tests** to get current error counts
2. **Identify top errors** from "Extra Errors" (over-reporting) or "Missing Errors" (under-reporting)
3. **Assign by tier** - Parser issues block checker accuracy
4. **One issue per engineer** - Focused work prevents conflicts

### Priority Order

```
Quality & Stability (Tier 0) → Parser (Tier 1) → Symbol Resolution (Tier 3) → Type Checker (Tier 2) → Implicit Any (Tier 4) → Async (Tier 5)
```

**Rationale:** Parser errors create broken ASTs that poison downstream analysis. Fix syntax handling before semantic checking.

### Debugging Workflow

1. **Find failing test:**
   ```bash
   # Check conformance output for example files
   # e.g., "classes/classDeclarations/classExtendingNonConstructor.ts"
   ```

2. **Create minimal repro:**
   ```bash
   # Create test file in /tmp/test.mjs
   node /tmp/test.mjs
   ```

3. **Compare outputs:**
   - TSC diagnostics (expected)
   - WASM diagnostics (actual)
   - Identify missing/extra error codes

4. **Trace through code:**
   - Parser: `thin_parser.rs`
   - Checker: `thin_checker.rs`
   - Add logging if needed

---

## File Reference

| File | Purpose |
|------|---------|
| `wasm/src/thin_parser.rs` | Main parser implementation |
| `wasm/src/thin_checker.rs` | Main type checker (~22k lines) |
| `wasm/src/binder/` | Symbol table and scope management |
| `wasm/src/solver/` | Type resolution engine |
| `wasm/src/checker/types/diagnostics.rs` | Error codes and message templates |
| `wasm/differential-test/` | Conformance test infrastructure |
| `wasm/differential-test/run-conformance.sh` | Docker-based test runner |
| `wasm/differential-test/conformance-child.mjs` | Per-test execution logic |

---

## Adding New Diagnostics

1. Add code to `wasm/src/checker/types/diagnostics.rs`:
   ```rust
   pub const NEW_ERROR_CODE: u32 = XXXX;
   ```

2. Add message template:
   ```rust
   pub const NEW_ERROR_MESSAGE: &str = "Error message with {0} placeholder.";
   ```

3. Emit in checker:
   ```rust
   use crate::checker::types::diagnostics::{diagnostic_codes, diagnostic_messages, format_message};

   let message = format_message(diagnostic_messages::NEW_ERROR_MESSAGE, &[arg]);
   self.error_at_node(node_idx, &message, diagnostic_codes::NEW_ERROR_CODE);
   ```

4. Rebuild and test:
   ```bash
   wasm-pack build --target web --out-dir pkg
   node /tmp/test.mjs
   ```


CRITICAL INSTRUCTIONS:
1. Check if TEAM_STRUCTURE.md exists - if not, create it with task assignments
2. If TEAM_STRUCTURE.md already exists, REVIEW it and pick ONE unassigned Tier 0 or Tier 1 issue to FIX
3. PRIORITIZE actual CODE CHANGES over documentation:
   - Tier 0 fixes in wasm/src/solver/*.rs, wasm/src/thin_checker.rs
   - Tier 1 fixes in wasm/src/thin_parser.rs
4. When making code changes:
   - Read the relevant .rs files first
   - Make minimal, focused changes
   - Test with: cd wasm && cargo build
5. Commit your ACTUAL CODE CHANGES (not just documentation)
6. Documentation updates are secondary - code fixes are primary

## Requirements

- Complete the task as described

## Files to Modify

- Determine which files need modification based on the task

## Acceptance Criteria

- [ ] Task completed successfully
- [ ] Code compiles/builds without errors
- [ ] Tests pass (if applicable)

## Context

- **Branch:** worker-1
- **Base Branch:** rust
- **Mode:** hierarchy
- **Team:** em-team-1
- **Task ID:** 944bc713-5601-44bf-813b-84dab6c35d21
- **Priority:** high

## Instructions

1. Read and understand the task requirements above
2. Make changes incrementally with clear, descriptive commit messages
3. Test your changes before marking the task complete
4. Do not modify files outside your task scope unless necessary
5. When done, commit all changes and push to your branch

Your changes will be automatically merged after review.

---
*Generated by CCO at 2026-01-16T21:45:59.429Z*
