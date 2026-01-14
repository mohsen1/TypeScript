# Team Structure - TypeScript Compiler (Rust)

**Phase:** Phase 8 - Conformance, Convergence, and Hardening
**Last Updated:** 2026-01-14 (Post EM-1 worker transfer to EM-2)
**Director:** claude-code-orchestrator

---

## Major Milestones

### ✅ EM-1 Validation Complete - Workers Transferred
**EM-1 completed validation cycle and transferred workers 1-3 to EM-2:**
- Exact Match: 30.1% → 34.5% (+4.4% from EM-1)
- Worker 1 (EM-2): TS1005 56% reduction (439→194), +5.2% EM
- Parser false positives: 701 → 194 (-72% combined!)

**EM-1 is now pure Binder squad (Worker 4 only):**
- Focus: Continue TS2304 reduction (459→276, need <50)
- Worker 4's Binder fix had HIGHEST impact (+3.8% EM)

### ✅ EM-3 Solver Worker Active
**Worker 12 assigned to Solver work:**
- Task: Switch solver fallback from `Any` to `Unknown`

---

## Director's Note

**Major reorganization complete:**
- EM_1: Pure Binder squad (Worker 4 only)
- EM_2: Parser squad (Workers 1-3 transferred from EM-1 + Workers 5-8)
- EM_3: Parser + Solver (Workers 9-12)

**🔴 TEAM SIZE ALERT:** EM-2 (8 workers) and EM-3 (5 workers) exceed limit of 4. Need to split/resize.

---

## Current Conformance Status

| Metric | Baseline | EM-1 | EM-2 (Worker 1) | Target | Progress |
|--------|----------|------|-----------------|--------|----------|
| Exact Match | 30.1% | 34.5% (+4.4%) | **35.3%** (+5.2%) | 40% | ✅ +5.2% |
| TS1005 | 439 | 267 (-39%) | **194** (-56%) | <100 | ✅ -245 |
| TS1109 | 262 | 122 (-53%) | **122** | <100 | ✅ -140 |
| TS2304 | 459 | 276 (-37%) | 276 | <50 | 🟡 -183 |
| Total Parser FP | 701 | 389 (-44%) | **316** (-55%) | <200 | ✅ -385 |

### Critical Issues - SIGNIFICANT PROGRESS
- ✅ **Parser False Positives:** Down to 316 from 701 (-55%)!
- ✅ **Worker 1 (EM-2):** Delivered +5.2% EM with comma inference fixes
- ✅ **Worker 4 (EM-1):** TS2304 -37% (276 remaining, need <50)
- ✅ **Solver Work:** Worker 12 on `Unknown` fallback

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
| EM_1 | **Binder** | worker-4 | **2** ✅ | Pure Binder focus | None - within limit |
| EM_2 | **Parser** | workers 1-3, 5-8 | **9** 🔴 | Expanded squad | **SPLIT into 2 squads** |
| EM_3 | **Parser + Solver** | workers 9-12 | **5** 🔴 | Hybrid squad | **TRANSFER 1 worker** |

**Actual Worker Distribution (after EM-1→EM-2 transfer):**
- EM_1: Worker 4 only (1 worker) - Pure Binder
- EM_2: Workers 1-3 (from EM-1) + Workers 5-8 = 8 workers - Parser squad
- EM_3: Workers 9-12 = 4 workers - Parser + Solver

**🔴 TEAM SIZE ISSUES:**
- EM_2: 8 workers + EM = **9 total** (limit is 4, need to remove 5)
- EM_3: 4 workers + EM = **5 total** (limit is 4, need to remove 1)

---

## Director's Orders: Team Rebalancing

### Required Actions

**1. SPLIT EM-2 into two squads:**
- EM-2A: Workers 1-3 (transferred from EM-1, high-performing team)
- EM-2B: Workers 5-8 (original EM-2 workers)
- Both squads focus on Parser (TS1005/TS1109)

**2. RESIZE EM-3:**
- Transfer **Worker 10** to EM-1 (Binder squad)
- EM-3 will have Workers 9, 11, 12 (3 workers + EM = 4 total)

**3. REASSIGN Worker 10 to Binder:**
- Worker 10 joins EM-1 to support Worker 4
- Focus: TS2304 reduction (276 → <50)

### Resulting Structure (After Rebalancing)

| EM | Workers | Total | Focus |
|----|---------|-------|-------|
| EM_1 | 4, 10 | 3 + EM = **4** ✅ | Binder |
| EM_2A | 1, 2, 3 | 3 + EM = **4** ✅ | Parser (from EM-1) |
| EM_2B | 5, 6, 7, 8 | 4 + EM = **5** 🔴 | Parser - still need to split |
| EM_3 | 9, 11, 12 | 3 + EM = **4** ✅ | Parser + Solver |

**Note:** EM_2B still at 5 total. Need to create EM-4 or further rebalance.

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
