# Team Structure - TypeScript Compiler (Rust)

**Phase:** Phase 8 - Conformance, Convergence, and Hardening
**Last Updated:** 2026-01-14 (Post all EM merges)
**Director:** claude-code-orchestrator

---

## Director's Note

**EM squads have taken autonomous approaches:**
- EM_1: Hybrid (Parser + Binder)
- EM_2: Task Master managing all three focus areas
- EM_3: Parser-only focus (TS1005/TS1109)

Squad-based structure not being followed by EMs. Director adapting to reality.

**Current Parser Work Distribution:**
- Workers 1-3 (EM_1): TS1005 patterns 1-5, TS1109 cascading
- Worker 7 (EM_2): TS1109 cascading fix (DELIVERED)
- Workers 9-11 (EM_3): TS1005 patterns 6-15, TS1109 expression errors

**All 3 EMs are contributing to Parser work** - this is the de facto priority.

---

## Current Conformance Status

| Metric | Value | Target |
|--------|-------|--------|
| Exact Match | 30.1% (1488/4939) | **40%** |
| Missing Errors | 60.0% (2961) | **<50%** |
| Extra Errors | 30.9% (1528) | Reduce |

### Critical Issues
- **Error Poisoning:** Solver defaults to `Any` when Binder fails, silencing downstream errors
- **TS2304:** 116 missing + 343 extra (Cannot find name)
- **Parser False Positives:** 701 errors (TS1005: 439, TS1109: 262)

---

## Squad Assignments

### EM_1: HYBRID Squad (Parser + Binder) - TRANSITIONAL
**Branch:** `em-team-1`
**Priority:** 🔴 HIGHEST
**Target Errors:** TS2304 (Binder), TS1005/TS1109 (Parser)
**Status:** 🟡 HYBRID - Awaiting validation cycle, then restructure

**Director's Note:** This is a transitional hybrid squad. Workers 1-3 made progress on Parser work before formal squad structure. After validation, they will transfer to EM_2.

**Focus (Worker 4 - Binder):**
- Fix Global Scope binding
- Ensure `lib.d.ts` symbols merge into root `SymbolTable`
- Fix module augmentation resolution
- Debug `console`, `Promise`, `Array` resolution failures
- **Goal:** Reduce TS2304 extra errors to <50

**Focus (Workers 1-3 - Parser, TRANSFERRING):**
- Fix TS1005 ("expected X") emission
- Fix TS1109 ("expression expected")
- Validate existing fixes and measure impact
- **Goal:** Reduce parser false positives to <100

**Key Files:**
- `src/lib_loader.rs`, `src/thin_binder.rs` (Binder)
- `src/compiler/parser.ts` (Parser)

**Success Metrics:**
- TS2304 extra errors < 50
- Parser false positives < 100
- **Complete validation cycle → Transfer workers 1-3 to EM_2**

---

### EM_2: Task Master - Multi-Squad (Binder + Parser + Solver)
**Branch:** `em-team-2`
**Priority:** 🟠 HIGH
**Target Errors:** TS2304, TS1005, TS1109, TS2322
**Status:** 🟢 ACTIVE - Managing 4 workers across 3 focus areas

**Director's Note:** EM-2 has taken a "Task Master" approach, managing workers 5-8 across all three focus areas. This deviates from the squad-based structure but is delivering results.

**Workers:**
- worker-5, worker-6: Binder (TS2304)
- worker-7: Parser (TS1005/TS1109) - **DELIVERED: TS1109 cascading error fix**
- worker-8: Solver (TS2322/TS7006)

**Progress:**
- Worker 7 implemented TS1109 cascading error fix (committed `afba575b5`)
  - Prevents TS1109 errors when TS1005 already reported at same position
  - Reduces false positive pollution in conformance measurements

**Success Metrics:**
- Parser false positives < 100 (worker-7 active)
- TS2304 extra errors < 50 (workers 5-6 assigned)
- Solver `Unknown` fallback (worker-8 assigned)

---

### EM_3: Parser Squad (Parser-only focus)
**Branch:** `em-team-3`
**Priority:** 🟠 HIGH
**Target Errors:** TS1005, TS1109
**Status:** 🟢 ACTIVE - Managing 3 workers on Parser work

**Director's Note:** EM-3 has taken a Parser-only focus (not Solver as originally assigned). Workers 9-11 are focused on TS1005/TS1109 patterns.

**Workers:**
- worker-9: TS1005 patterns 6-10 (object/array literals)
- worker-10: TS1109 expression expected errors
- worker-11: TS1005 patterns 11-15 (edge cases)

**Key Files:**
- `src/compiler/parser.ts`
- `src/compiler/scanner.ts`
- `TS1005_REDUCTION_RESULTS.md`, `TS1109_ANALYSIS.md`

**Success Metrics:**
- TS1005: 439 → <100
- TS1109: 262 → <50
- Total parser false positives: 701 → <100

---

## Anti-Priorities (DO NOT WORK ON)
- Performance optimization (speed is sufficient at 41.7 tests/sec)
- New Emitter features (downleveling is stable)
- LSP polish (wait for semantic model accuracy)

---

## Resource Allocation

| EM | Squad | Assigned Workers | Total | Notes |
|----|-------|------------------|-------|-------|
| EM_1 | **HYBRID** | workers 1-4 | 5 | Parser + Binder, awaiting validation |
| EM_2 | **Task Master** | workers 5-8 | 5 | Binder + Parser + Solver, delivering results |
| EM_3 | **Parser** | workers 9-11 | 4 | Parser-only focus (worker 12 reassigned) |

**Current Status:**
- Workers 1-11 are active across all 3 EMs
- All 3 EMs are contributing to Parser work (de facto priority)
- Worker 12 status unclear (removed from EM_3 task lists)
- **Solver work** (EM_3's original assignment) is not being actively pursued

**Note:** CFA (Control Flow Analysis) work is on hold until TS2304 is under control.

---

## Merging Protocol
1. EMs submit PRs to `rust` branch
2. Director reviews and merges EM branches only
3. After each merge: re-balance teams to maintain size ≤ 4 (including EM)

---

## Escalation Criteria
EMs should escalate to Director when:
- Team size exceeds 4 (need split)
- Blocker requires cross-squad coordination
- Target metric is achieved (ready for reassignment)
- Technical approach is unclear and needs Director input
