# Team Structure - TypeScript Compiler (Rust)

**Phase:** Phase 8 - Conformance, Convergence, and Hardening
**Last Updated:** 2026-01-14 (Post EM-1 validation & EM-3 Solver work)
**Director:** claude-code-orchestrator

---

## Major Milestones

### ✅ EM-1 Validation Complete
**Delivered excellent conformance improvements:**
- Exact Match: 30.1% → 34.5% (+4.4%)
- Parser false positives: 701 → 389 (-44%)
- TS2304 errors: 459 → 276 (-37%)

**Worker 4's Binder fix had HIGHEST impact (+3.8% EM)** - validates that fixing "Any" poisoning is critical.

### ✅ EM-3 Adds Solver Worker
**Worker 12 assigned to Solver work:**
- Task: Switch solver fallback from `Any` to `Unknown`
- This exposes hidden bugs by stopping silent error masking
- Addresses critical gap in Solver capacity

---

## Director's Note

**EM squads are rebalancing based on validated results:**
- EM_1: Hybrid (Parser + Binder) - VALIDATED, keep together
- EM_2: Parser (reassigned from Task Master)
- EM_3: Parser + Solver (added worker-12 for Solver work)

**Key Insight:** Binder fixes (like Worker 4's) have highest impact. Need more Binder capacity.

---

## Current Conformance Status

| Metric | Before EM-1 | After EM-1 | Target | Progress |
|--------|-------------|------------|--------|----------|
| Exact Match | 30.1% (1488/4939) | **34.5%** | **40%** | ✅ +4.4% |
| Parser False Positives | 701 (TS1005: 439, TS1109: 262) | **389** (-44%) | **<100** | ✅ -312 errors |
| TS2304 Errors | 459 | **276** (-37%) | **<50** | ✅ -183 errors |
| Missing Errors | 60.0% (2961) | TBD | **<50%** | 🟡 TBD |

### Critical Issues - IMPROVING
- ✅ **Error Poisoning:** Worker 4's Binder fix reduced TS2304 by 37%
- ✅ **Parser False Positives:** Combined fixes reduced by 44% (701→389)
- ✅ **Solver Work:** Worker 12 assigned to switch `Any` → `Unknown` fallback
- 🟡 **Remaining:** TS2304 still at 276 (target <50), Parser FP at 389 (target <100)

---

## Squad Assignments

### EM_1: HYBRID Squad (Parser + Binder) - VALIDATED ✅
**Branch:** `em-team-1`
**Priority:** 🔴 HIGHEST
**Target Errors:** TS2304 (Binder), TS1005/TS1109 (Parser)
**Status:** 🟢 VALIDATED - Delivered +4.4% Exact Match improvement

**Director's Decision:** Keep EM-1 together as a high-performing team. Do NOT split up workers.

**Results Delivered:**
- Worker 1 (TS1005): 439→312 (-29%), Exact Match +1.1%
- Worker 2 (TS1109): 262→198 (-24%), Exact Match +0.4%
- Worker 3 (Cascading): 701→551 (-21%), Exact Match +1.8%
- Worker 4 (Binder): 459→276 (-37%), Exact Match +3.8% ⭐ HIGHEST
- **Combined:** 30.1% → 34.5% Exact Match (+4.4%)

**Next Steps:**
- Workers 1-3: Continue Parser work (remaining TS1005/TS1109 patterns)
- Worker 4: Continue Binder work (module namespace resolution, ~48 cases)
- All workers: High throughput, keep together as a team

**Key Files:**
- `src/lib_loader.rs`, `src/thin_binder.rs` (Binder)
- `src/compiler/parser.ts` (Parser)

---

### EM_2: Parser Squad (Parser-only focus)
**Branch:** `em-team-2`
**Priority:** 🟠 HIGH
**Target Errors:** TS1005, TS1109
**Status:** 🟢 ACTIVE - Managing 4 workers on Parser work

**Director's Note:** EM-2 was initially a "Task Master" managing all three focus areas, but has now reassigned all workers to Parser work.

**Workers:**
- worker-5: TS1005 audit and fixes
- worker-6: TS1005 audit and fixes
- worker-7: TS1109 focus - **DELIVERED: TS1109 cascading error fix (commit `afba575b5`)**
- worker-8: Error recovery and resynchronization

**Progress:**
- Worker 7 implemented TS1109 cascading error fix
  - Prevents TS1109 errors when TS1005 already reported at same position
  - Reduces false positive pollution in conformance measurements

**Success Metrics:**
- Parser false positives < 100 (all workers active)

---

### EM_3: Parser + Solver Squad
**Branch:** `em-team-3`
**Priority:** 🟠 HIGH
**Target Errors:** TS1005, TS1109, TS2322, TS7006
**Status:** 🟢 ACTIVE - 3 workers on Parser, 1 worker on Solver

**Director's Note:** EM-3 has added worker-12 for Solver work! This addresses the critical gap.

**Workers:**
- worker-9: TS1005 patterns 6-10 (object/array literals)
- worker-10: TS1109 expression expected errors
- worker-11: TS1005 patterns 11-15 (edge cases)
- **worker-12: Solver** - Switch `Any` → `Unknown` fallback ⭐ NEW

**Key Files:**
- `src/compiler/parser.ts`, `src/compiler/scanner.ts` (Parser)
- `wasm/src/solver/` (Solver)

**Success Metrics:**
- Parser: TS1005 < 100, TS1109 < 50
- Solver: Switch to `Unknown` fallback complete

---

## Anti-Priorities (DO NOT WORK ON)
- Performance optimization (speed is sufficient at 41.7 tests/sec)
- New Emitter features (downleveling is stable)
- LSP polish (wait for semantic model accuracy)

---

## Resource Allocation

| EM | Squad | Assigned Workers | Total | Status | Notes |
|----|-------|------------------|-------|--------|-------|
| EM_1 | **HYBRID** | workers 1-4 | 5 | ✅ VALIDATED | +4.4% Exact Match, keep together |
| EM_2 | **Parser** | workers 5-8 | 5 | 🟡 ACTIVE | All workers on Parser |
| EM_3 | **Parser + Solver** | workers 9-12 | 5 | 🟢 ACTIVE | 3 Parser, 1 Solver (worker-12) |

**Current Status (Post-Rebalancing):**
- All 12 workers are now active
- **Binder:** Worker 4 only (1 worker) - still under-resourced
- **Solver:** Worker 12 only (1 worker) - critical work started ✅
- **Parser:** Workers 1-3, 5-11 (10 workers) - well-resourced

**Director Assessment:**
- ✅ EM-1 validated, high-performing team
- ✅ EM-3 added Solver worker (addresses critical gap)
- 🟡 EM-2 still 100% on Parser (consider reassigning 1 to Binder)
- 🔴 Binder work still under-resourced (only Worker 4)

**Note:** CFA (Control Flow Analysis) work remains on hold until TS2304 is under control.

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
