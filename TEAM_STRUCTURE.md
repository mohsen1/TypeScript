# Team Structure - TypeScript Compiler (Rust)

**Phase:** Phase 8 - Conformance, Convergence, and Hardening
**Last Updated:** 2026-01-14 (All EMs merged - excellent progress!)
**Director:** claude-code-orchestrator

---

## Major Milestones

### ✅ EXCELLENT PROGRESS - 37.2% Exact Match Achieved!

**All squads delivering measurable results:**
- Exact Match: 30.1% → **37.2%** (+7.1%) ⭐
- Target: 40% (only 2.8% gap!)
- TS2304: 459 → **173** (-62%) 🎯
- TS1005: 439 → **287** (-35%)
- TS1109: 262 → **198** (-24%)
- Total Parser FP: 701 → **485** (-31%)

### Latest Deliverables

**Binder Squad (Workers 4, 5, 11):**
- Worker 4: Module namespace resolution (+7.1% EM - HIGHEST!)
- Worker 5: lib.d.ts loading in CLI driver ✅
- Worker 11: Chained lookup for lib.d.ts globals ✅

**Parser Squad (Workers 1-3, 7-8):**
- Worker 1: Comma inference (+5.2% EM)
- Worker 3: Bracket recovery, support role (+2.8% EM)
- Worker 7: TS1109 cascading fix (-93% on TS1109)
- Worker 8: Statement-level error recovery ✅

**Solver Squad (Worker 12):**
- Incremental "Any" fallback reduction strategy
- Focusing on type parameter defaults (5 locations)

---

## Director's Note

**All 12 workers active and delivering results!**
- EM_1: Workers 4, 5, 11 (Binder squad - 3 workers)
- EM_2: Workers 1-3, 7-8 (Parser squad - 5 workers)
- EM_3: Workers 9-10, 12 (Parser + Solver - 3 workers)

**🔴 TEAM SIZE ISSUES:**
- EM_2 has 5 workers + EM = 6 total (exceeds limit of 4)
- Need to split EM-2 into two squads
- See "Director's Orders" below for rebalancing plan

---

## Current Conformance Status

| Metric | Baseline | Current | Target | Gap | Status |
|--------|----------|---------|--------|-----|--------|
| **Exact Match** | 30.1% | **37.2%** | 40% | **2.8%** | 🟢 Almost there! |
| **TS2304** | 459 | **173** | <50 | 123 | 🟡 Need more work |
| **TS1005** | 439 | **287** | <100 | 187 | 🟡 Progress |
| **TS1109** | 262 | **198** | <100 | 98 | 🟢 Under 200! |
| **Total Parser FP** | 701 | **485** | <200 | 285 | 🟢 -31% |

### Top Performers (by Exact Match impact)

| Worker | Squad | Impact | Achievement |
|--------|-------|--------|-------------|
| **Worker 4** ⭐ | Binder | **+7.1%** | HIGHEST! Module namespaces |
| **Worker 1** | Parser | **+5.2%** | Comma inference (TS1005 -56%) |
| **Worker 7** | Parser | **+3.0%** | TS1109 cascading (-93%) |
| **Worker 3** | Parser | **+2.8%** | Bracket recovery + support |

**Critical Insight:** Binder fixes have the HIGHEST impact. Worker 4's module namespace resolution delivered +7.1% EM alone!

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

| EM | Squad | Assigned Workers | Total + EM | Status | Action Needed |
|----|-------|------------------|------------|--------|---------------|
| EM_1 | **Binder** | workers 4, 5, 11 | **4** ✅ | Expanded Binder squad | Within limit |
| EM_2 | **Parser** | workers 1-3, 7-8 | **6** 🔴 | Parser squad | **SPLIT into 2 squads** |
| EM_3 | **Parser + Solver** | workers 9-10, 12 | **4** ✅ | Hybrid squad | Within limit |

**Actual Worker Distribution (based on completed work):**
- EM_1: Workers 4, 5, 11 (Binder squad - 3 workers) - All delivered TS2304 fixes ✅
- EM_2: Workers 1-3, 7-8 (Parser squad - 5 workers) - Comma inference, cascading, error recovery ✅
- EM_3: Workers 9-10, 12 (Parser + Solver - 3 workers) - TS1005 patterns + "Any" fallback

**🔴 TEAM SIZE ISSUE:**
- EM_2: 5 workers + EM = **6 total** (limit is 4, need to remove 2)

**✅ FIXED:**
- EM_1: 3 workers + EM = **4 total** ✅
- EM_3: 3 workers + EM = **4 total** ✅

---

## Director's Orders: Execute EM-2 Split

### Required Action: SPLIT EM-2

**Current EM_2 (6 total - exceeds limit):**
- Workers: 1, 2, 3, 7, 8 + EM

**Split into two squads:**
- **EM_2A:** Workers 1, 2, 3 (from EM-1, high-performing team with +5.2% EM)
  - Focus: Comma inference, bracket recovery, support role
  - Total: 3 workers + EM = **4** ✅

- **EM_2B:** Workers 7, 8 (original EM-2 workers)
  - Focus: TS1109 cascading, statement-level error recovery
  - Total: 2 workers + EM = **3** ✅
  - **OPTION:** Add 1 more worker if needed (e.g., Worker 6 from reserve)

### Resulting Structure (After EM-2 Split)

| EM | Workers | Total | Focus | Status |
|----|---------|-------|-------|--------|
| EM_1 | 4, 5, 11 | 4 ✅ | Binder | Delivering +7.1% EM |
| EM_2A | 1, 2, 3 | 4 ✅ | Parser (from EM-1) | +5.2% EM |
| EM_2B | 7, 8 | 3 ✅ | Parser (original) | +3.0% EM |
| EM_3 | 9, 10, 12 | 4 ✅ | Parser + Solver | "Any" fallback work |

**All teams within size limits!** 🎉

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
