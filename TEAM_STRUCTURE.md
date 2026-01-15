# Team Structure - Project Zang Orchestrator

**Last Updated:** 2026-01-15 16:15
**Director:** Claude Code Orchestrator
**Target Branch:** rust

---

## Team Structure: FOUR EMs ESTABLISHED

Project Zang has **4 Engineering Managers (EMs)** and **14 Workers**.

### Team Size Policy
- **Maximum team size: 4 workers per EM**
- Teams are resized dynamically after every merge
- If team size exceeds 4, spin up new EM and reassign workers

---

## Team Assignments (2026-01-15 16:15)

### EM-1: "Syntax Squad" 🟢 ACTIVE
**Focus:** Tier 1 (Parser Accuracy)

| Worker | Branch | Squad | Assigned Focus | Status | Throughput |
|--------|--------|-------|----------------|--------|------------|
| Worker 3 | worker-3 | Syntax | TS1109 suppression | 🔵 Week 1 mentorship | TBD |
| Worker 4 | worker-4 | Syntax | TS1005 suppression | 🔵 Week 1 mentorship | TBD |
| Worker 5 | worker-5 | Syntax | Parser recovery ✅ Complete | 🟢 Mentor | High |

**EM Branch:** em-team-1

**Leadership:** Worker 5 (exceptional throughput, mentor for Workers 3-4)

**Status:**
- ✅ Workers 3-4 restarted under Worker 5's mentorship
- ✅ Worker 5 complete - leading Syntax Squad
- 📅 Week 1 Checkpoint: 2026-01-22
- 📅 Decision point: 2026-02-05

**Worker 3 Assessment:**
- Assigned: TS1109 suppression (262 extra errors)
- Actual: Documentation only - needs restart

**Worker 4 Assessment:**
- Assigned: TS1005 suppression (345 extra errors)
- Actual: Did wrong task - needs redirection

---

### EM-2: "Support Squad" 🟡 REASSIGNED
**Focus:** Tier 0 (Quality & Stability Support) + LSP Integration

| Worker | Branch | Squad | Current Focus | Status | Throughput |
|--------|--------|-------|---------------|--------|------------|
| Worker 6 | worker-6 | Support | Definite assignment (TS2565) | 🔄 Reassigned | Medium |
| Worker 7 | worker-7 | Support | LSP config integration | 🔄 Reassigned | Medium |
| Worker 8 | worker-8 | LSP | TypeScript config integration | ✅ Complete | High |

**EM Branch:** em-team-2

**Status:**
- ⚠️ **Module resolution reassigned** (2026-01-15 15:30)
- 📋 Module resolution moved to EM-1 (Worker 10) and EM-3 (Worker 1)
- 🔄 Workers 6-7 reassigned to support tasks (lower complexity)
- ✅ Worker 8 complete - LSP TypeScript config integration merged

**Reassignment Rationale:**
- Workers 6-7 had 45+ minutes with 0 commits on module resolution
- Workers 1, 10 demonstrated high throughput on similar tasks (TS2683, TS2571)
- Module resolution too critical for project to delay further (800+ errors)

---

### EM-3: "Type Checking Squad" 🟢 ACTIVE
**Focus:** Tier 2 (Type Checker Accuracy) + Tier 3 (Module Resolution)

| Worker | Branch | Squad | Focus Area | Status | Throughput |
|--------|--------|-------|------------|--------|------------|
| Worker 1 | worker-1 | Type | Module resolution (TS7005/TS7008) 🆕 | 🟢 Ready | High |
| Worker 2 | worker-2 | Type | TS2322 accuracy | 🟢 Ready | High |
| Worker 11 | worker-11 | Type | TS2571 over-reporting | 🔄 Active | TBD |

**EM Branch:** em-team-3

**Leadership:** Workers 1-2 (both high-throughput, type checking expertise)

**Completed Work:**
- ✅ Worker 1: TS2683 (implicit this in functions)
- ✅ Worker 2: super() call handling

**Status:**
- 🆕 Worker 1 assigned module resolution (TS7005/TS7008) - reassigned from EM-2
- Worker 2 assigned TS2322 categorization and fixes
- Worker 11 working on TS2571 over-reporting

---

### EM-4: "Quality & Stability Squad" 🟢 ACTIVE
**Focus:** Tier 0 (Cross-cutting) + Tier 1 (Parser) + Tier 3 (Symbol Resolution) + Tier 5 (Async)

| Worker | Branch | Squad | Focus Area | Status | Throughput |
|--------|--------|-------|------------|--------|------------|
| Worker 12 | worker-12 | Quality | Parser (TS1109/TS1005) | 🔄 Reassigned | TBD |
| Worker 13 | worker-13 | Quality | Async/Await (TS2705/TS1359) | 🟡 Assigned | TBD |
| Worker 14 | worker-14 | Quality | Symbol Resolution (TS2304/TS2524) | 🔄 Reassigned | TBD |

**EM Branch:** em-team-4

**Status:**
- ✅ EM_4_TASKS.md created with baseline metrics (2025-01-15)
- 📋 Baseline: 28.9% exact match (55/190 files)
- 🔄 Workers 12, 14 need reassignment (previous tasks were EM-3 work)

**Priority Issues:**
- Worker 12: TS1109 (7 missing, 7 extra), TS1005 (5 extra)
- Worker 13: TS2705 (34 missing), TS1359 (7 missing)
- Worker 14: TS2304 (7 missing), TS2524 (12 missing)

---

## Priority Order by Team

### EM-1 (Syntax Squad) Priority Queue
1. **Restart Workers 3-4** - Under Worker 5's mentorship
2. **TS1109 suppression** - Reduce from 262 to <40 errors (Worker 3 or 5)
3. **TS1005 suppression** - Reduce from 345 to <50 errors (Worker 4 or 5)
4. **ASI handling** - Edge cases in automatic semicolon insertion
5. **TS2792 import() resolution** - 161 missing errors (Worker 10) 🆕

### EM-2 (Support Squad) Priority Queue
1. **Definite Assignment (TS2565)** - Worker 6 (Tier 0 quality task)
2. **LSP Config Integration** - Worker 7 (support Worker 8 if needed)
3. **Type Checker Accuracy** - After support tasks complete

### EM-3 (Type Checking Squad) Priority Queue
1. **Module Resolution (TS7005/TS7008)** - 825 errors (Worker 1) 🆕
   - TS7005: "Symbol cannot be referenced from module" (489 extra)
   - TS7008: "Module has no exported member" (336 extra)
   - Reassigned from EM-2 (Workers 6-7 had zero progress)

2. **TS2322 Type Assignability** - 548 extra errors (Worker 2)
   - Categorize: legitimate vs false positive
   - Fix assignability logic for false positives
   - Handle union/intersection type edge cases

3. **TS2571 Over-reporting** (Worker 11)
   - Should be TS2683 in many cases
   - Fix `this` type inference in non-class methods

### EM-4 (Quality & Stability Squad) Priority Queue
1. **Async/Await (TS2705/TS1359)** - Worker 13
2. **Symbol Resolution (TS2304/TS2524)** - Worker 14
3. **Parser (TS1109/TS1005)** - Worker 12

---

## Merge Workflow

1. **Workers** commit to their worker branches (worker-1 through worker-14)
2. **EMs** merge from worker branches to EM branches (em-team-1 through em-team-4)
3. **Director** merges from EM branches to `rust` branch
4. **After merge:** Director evaluates team sizes and reassigns if needed

---

## Current Blocking Issues

| Issue | Workers Blocked | Resolution Path |
|-------|-----------------|-----------------|
| Workers 3-4 restart | Workers 3-4 | EM-1 must assign tasks under Worker 5's mentorship |
| Module resolution | All EMs (825 errors) | Worker 1 (EM-3) + Worker 10 (EM-1) taking over |
| EM-4 activation | Workers 12-14 | EM-4 must assign Tier 0 tasks |

---

## Documentation

- **Director Planning:** Root directory
- **EM-1 Tasks:** `EM_1_TASKS.md`
- **EM-2 Tasks:** `EM_2_TASKS.md`
- **EM-3 Tasks:** `EM_3_TASKS.md`
- **EM-4 Tasks:** `EM_4_TASKS.md`
- **Project Direction:** `PROJECT_DIRECTION.md`
- **Workflow:** `AGENTS.md`

---

## Success Metrics

| Team | Exact Match Target | Current | Priority |
|------|-------------------|---------|----------|
| **EM-1** | Reduce parser noise by 80% | TS1109: ~262, TS1005: ~345 | 🔴 High |
| **EM-2** | Support tasks | Reassigned | 🟡 Medium |
| **EM-3** | Fix 800+ module errors | TS7005: 489, TS7008: 336 | 🔴 Critical |
| **EM-4** | Fix Tier 0 cross-cutting gaps | 28.9% exact match | 🟡 High |

**Overall Project Goal:** 95%+ exact match rate before production

---

## Capacity Planning

**Current State (Post-Reassignment):** 🟡 IN TRANSITION
- EM-1: 3 workers (at capacity, Workers 3-5) + Worker 10 (TS2792)
- EM-2: 3 workers (reassigned to support tasks, Workers 6-8)
- EM-3: 3 workers (at capacity, Workers 1-2, 11) + Worker 1 (module resolution)
- EM-4: 3 workers (at capacity, Workers 12-14)

**Key Reassignments (2025-01-15):**
- ✅ Module resolution: EM-2 → EM-1 (Worker 10) + EM-3 (Worker 1)
- ✅ Workers 6-7 → Support tasks (Tier 0 quality, LSP)
- ✅ EM-2 focus shifted to support role

**Future Considerations:**
- EM-1 needs mentorship progress (checkpoint: 2025-01-22)
- EM-3 module resolution is critical path (825 errors)
- EM-4 needs to establish baseline metrics
