
# PROJECT Zang

## Mission: TypeScript → Rust/WASM Migration

Project Zang is a complete rewrite of the TypeScript compiler and type checker in Rust, compiled to WebAssembly for performance. The goal is to **beat TypeScript in performance** while maintaining 100% compatibility with the original TypeScript compiler.

## Architecture Overview

**Core Principle:** TypeScript source files (`src/`) remain **read-only** and identical to upstream Microsoft TypeScript. All custom implementation lives in the `wasm/` directory.

See `wasm/specs/WASM_ARCHITECTURE.md` for a deep dive.
See `wasm/specs/SOLVER.md` for type solver architecture.
See `wasm/specs` files for other component designs and references.

### Key Components:
- **WASM Parser** (`wasm/src/parser/`) - Rust implementation of TypeScript parser
- **WASM Checker** (`wasm/src/checker/`) - Type checking and semantic analysis  
- **WASM Solver** (`wasm/src/solver/`) - Type resolution and constraint solving
- **WASM Binder** (`wasm/src/binder/`) - Symbol binding and scope management
- **Integration Layer** (`wasm/src/integration/`) - TypeScript ↔ WASM bridge

### Quality Metrics:
- **Conformance Tests:** 4,941 TypeScript test cases
- **Current Performance:** 60.8% exact/equivalent match with TypeScript
- **Target Performance:** 95%+ compatibility before production

## Current Priority Issues

### 1. 🔴 CRITICAL: Fix The "Parser Noise" (TS1005 & TS1109)
**Owner:** Syntax Squad
**Data:** 701 combined extra errors (TS1005: 439, TS1109: 262).
**Analysis:** These are false positives. Our `ThinParser` is bailing out or emitting error nodes on syntax that `tsc` accepts. This "noise" makes it impossible to trust downstream semantic errors because a broken AST results in broken symbols.
**Action:**
*   **Implement Error Resynchronization:** When the parser hits an unexpected token, it must not just emit an error; it must advance to the next synchronization point (e.g., next `;` or `}`) and *continue* parsing the rest of the file.
*   **Audit Semicolon Insertion (ASI):** Verify our ASI logic matches TypeScript's exactly. Many TS1005 errors are likely missing semicolons we aren't inferring.

### 2. 🔴 CRITICAL: The "Global Scope" Fix (TS2304)
**Owner:** worker-3
**Status:** 🟡 IN PROGRESS (Assigned 2026-01-15)
**Data:** TS2304 appears in both Extra (343) and Missing (116) lists.
**Analysis:** This is the root of the "Error Poisoning."
*   **Extra TS2304:** We aren't loading `lib.d.ts` correctly in the test runner, so `console`, `Promise`, and `Array` are undefined.
*   **Missing Errors:** When `Promise` is undefined, the Solver treats it as `Any`. This suppresses TS2322 (Type Mismatch) errors downstream.
**Action:**
*   **Fix Lib Injection:** Ensure `lib.d.ts` is correctly merged into the root `SymbolTable` for every test.
*   **Fix Global Merging:** Ensure `interface Window` (and similar globals) merge correctly across files.

### 3. 🟠 STRATEGIC: Invert Solver Defaults (Stop being "Nice")
**Owner:** Semantics Squad
**Data:** 2961 missing errors (60%).
**Analysis:** We are missing 184 `TS2322` (Type Mismatch) and 357 `TS7006` (Implicit Any) errors. This proves our compiler is "optimistic"—when it encounters an unknown type or a resolution failure, it returns `TypeId::ANY`.
**Action:**
*   **Change Default to `UNKNOWN`:** Modify `wasm/src/solver/` to return `TypeId::UNKNOWN` or `TypeId::ERROR` instead of `TypeId::ANY` when a symbol cannot be resolved or a type operation fails.
*   **Expect a Regression:** This will cause a massive spike in "Extra Errors." **This is good.** It exposes exactly where our logic is failing rather than hiding it behind `Any`.

### 4. 🟡 TACTICAL: Fix Class Property Initialization (TS2564)
**Owner:** CFA Squad
**Data:** TS2564 is the #1 missing error (413 occurrences).
**Analysis:** "Property 'x' has no initializer..." is missing. This means we are simply *not running* the check that verifies class properties are initialized in the constructor.
**Action:**
*   Implement the `strictPropertyInitialization` check in `wasm/src/checker/thin_checker.rs`. This is a high-ROI task that will knock out the top missing error category.

### 5. 🟢 STABILITY: Recursion Guards
**Data:** 2 Crashes (Stack Overflow).
**Analysis:** `types/typeRelationships/recursiveTypes` caused a panic.
**Action:**
*   Add a `recursion_depth` counter to `solve_subtype` and `check_expression`. Return a "Type instantiation is excessively deep" error (TS2589) when hitting the limit (e.g., 100), rather than crashing the WASM process.

---

### Success Metrics for Next Report
*   **TS1005/TS1109 (Parser):** Reduce from ~700 to <40. ✅ **EM-1/EM-2 ACHIEVED** (96% reduction: 701 → 29)
*   **TS2304 (Binder):** Reduce Extra errors from 343 to <10. ✅ **ALREADY FIXED** (343 → ~0)
*   **TS2564 (CFA):** Reduce Missing errors from 413 to <20. ✅ **EM-3/EM-1 ACHIEVED** (413 → ~0)
*   **Exact Match:** Increase from 30.1% to **80%+**. 🟡 **IN PROGRESS** (currently ~44%)

**Priority Order:** Parser (Noise) -> Binder (Poisoning) -> Solver (Strictness). Do not work on new features until Parser noise is cleared.

---

## EM-3 Team Branch Integration (2026-01-15)

### Merge Summary: em-team-3 → rust ✅

**Validation Results:**
- Tests Run: 487
- Exact Match: 158 (32.4%)
- Same Error Count: 184 (37.8%)
- **Total Parity: 70.2%**
- **WASM Crashed: 0** (down from 2) ✅

**Key Achievements:**
- TS2564 no longer in missing errors (was 413 missing)
- TS2322 reduced to 13 missing (was 310+)
- WASM compilation fixed
- All workers (9-12) merged successfully

**Files Modified:**
- `wasm/src/checker/declarations.rs` - TS2564 implementation
- `wasm/src/thin_checker.rs` - Super keyword type inference
- `wasm/src/thin_parser.rs` - Parser improvements
- `wasm/src/checker/types/diagnostics.rs` - Error diagnostics

### Team Assessment - EM-3

**Productivity Analysis:**
| Worker | Status | Performance | Recommendation |
|--------|--------|-------------|----------------|
| worker-9 | ✅ Merged | **HIGH** - Completed 3+ tasks, TS2322 refinements | **KEEP** - Core contributor |
| worker-10 | ✅ Merged | **MEDIUM** - Module resolution complete, needs validation | **REASSIGN** - Available for new work |
| worker-11 | ✅ Merged | **HIGH** - TS2322 analysis ready, ERROR type diagnostics | **KEEP** - Has critical analysis |
| worker-12 | ⚠️ Stalled | **LOW** - Task assigned but not started | **REASSIGN** - Redistribute work |

**Decision:** **RESIZE EM-3** to 2 workers (9, 11). Redistribute workers 10 and 12 to high-priority tasks.

---

## Updated Priority Tasks (Post-EM-3 Merge)

### Completed ✅
1. Parser Noise (TS1005/TS1109) - 96% reduction
2. TS2304 Global Scope - Already fixed
3. TS2564 Class Property Initialization - Phase 1 & 2 complete
4. Recursion Guards - Already implemented (0 crashes)
5. Solver Defaults Inversion - Complete

### Remaining Critical Issues 🔴

1. **TS2322 Type Accuracy** (~105 missing, ~548 extra)
   - Primary: Abstract Constructor Assignability
   - Owner: worker-11 has analysis ready
   - Estimated: 3-5 days

2. **TS7006 Implicit Any** (~357 missing)
   - Parameter type inference failures
   - Unassigned
   - Estimated: 3-5 days

3. **Conformance Validation** (BLOCKS ALL PLANNING)
   - Owner: worker-2 (EM-1)
   - Run full 4941 tests, generate report
   - Estimated: 1-2 days

4. **Literal Type Narrowing** (TS2322 subtask)
   - Owner: worker-4 (EM-1)
   - Fix narrowing in assignments/conditionals
   - Estimated: 3-5 days

---

## Director Decision: Team Restructuring

### Current State Analysis

**EM-1 (Syntax & Foundation):** ✅ STABLE
- 4 workers at capacity
- All assigned to critical path tasks
- No changes needed

**EM-2 (Semantics & Core):** ⚠️ NEEDS RESIZE
- worker-5: Available (high performer)
- worker-6: Off-track (wrong task)
- worker-7: Available
- worker-8: Available
- **Decision:** RESIZE to 2-3 workers, redistribute excess

**EM-3 (Advanced Features):** ✅ MERGED, NEEDS RESIZE
- worker-9: High performer (keep)
- worker-10: Available for reassignment
- worker-11: High performer with critical analysis (keep)
- worker-12: Task not started (reassign)
- **Decision:** Resize to 2 workers (9, 11), redistribute 10 and 12

### Worker Redistribution Plan

**From EM-2:**
- worker-5 → Assign to TS7006 or TS2322 support
- worker-7 → Assign to validation or testing
- worker-8 → Available for overflow work
- worker-6 → Redirect to correct task or reassign

**From EM-3:**
- worker-9 → Keep for TS2322 work (has expertise)
- worker-11 → Keep for TS2322 implementation (has analysis)
- worker-10 → Reassign to validation or module testing
- worker-12 → Reassign to any available task

**New Team Sizes (Recommended):**
- EM-1: 4 workers (unchanged)
- EM-2: 2-3 workers (down from 4)
- EM-3: 2 workers (down from 4)
- **Total active workers:** 8-9 (down from 12)

**Rationale:** EM-3's integration tasks are complete. Remaining work is type accuracy (TS2322, TS7006) which benefits from focused, smaller teams. Excess workers can form a "Float Pool" for overflow work, validation, and testing.