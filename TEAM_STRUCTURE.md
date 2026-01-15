# Team Structure - Project Zang Orchestrator

**Last Updated:** 2026-01-15
**Director:** Managing 3 Engineering Managers, 12 Workers (restructuring to 8-9)
**Branch:** rust

---

## Executive Summary

Project Zang is migrating the TypeScript compiler to Rust/WASM. Current performance is **44.2%** conformance with TypeScript. Our mission is to reach **95%+ compatibility** by fixing critical error discrepancies.

**Recent Merges:**
- EM-3: ✅ Merged to rust (2026-01-15) - TS2564, TS2322 improvements
- EM-1: 🔄 Escalated for review
- EM-2: 🔄 Escalated for review

This document defines the team structure and focus areas for each Engineering Manager (EM) and their assigned workers.

---

## Team Allocation

### EM-1 (Team: Syntax & Foundation)
- **Branch:** em-team-1
- **Workers:** worker-1, worker-2, worker-3, worker-4
- **Worktree:** /tmp/orchestrator-workspace/worktrees/em-1
- **Squad Focus:** Parser noise and foundational stability

### EM-2 (Team: Semantics & Core Type System)
- **Branch:** em-team-2
- **Workers:** worker-5, worker-6, worker-7, worker-8
- **Worktree:** /tmp/orchestrator-workspace/worktrees/em-2
- **Squad Focus:** Solver strictness, binder global scope, class property checks

### EM-3 (Team: Advanced Features & Integration)
- **Branch:** em-team-3
- **Workers:** worker-9, worker-10, worker-11, worker-12
- **Worktree:** /tmp/orchestrator-workspace/worktrees/em-3
- **Squad Focus:** Feature parity, integration layer, advanced type constructs

---

## Priority Allocation by Team

### Priority 1: Parser Noise (TS1005 & TS1109) → **EM-1**
**701 combined extra errors** blocking all downstream semantic analysis.

- **Root Cause:** `ThinParser` bails out on syntax errors instead of recovering
- **Impact:** Broken AST → broken symbols → poisoned error reports
- **Action Items:**
  - Implement error resynchronization (continue parsing after error)
  - Audit semicolon insertion (ASI) logic
- **Target:** Reduce from ~700 to <40 extra errors
- **Assigned to:** worker-1

### Priority 2: Global Scope Fix (TS2304) → **EM-2**
**343 extra errors** from missing `lib.d.ts` and global symbol poisoning.

- **Root Cause:** `console`, `Promise`, `Array` undefined in test runner
- **Impact:** Undefined symbols treated as `Any` → suppresses downstream errors
- **Action Items:**
  - Fix `lib.d.ts` injection into root `SymbolTable`
  - Fix global merging across files (`interface Window` etc.)
- **Target:** Reduce Extra TS2304 from 343 to <10
- **Assigned to:** worker-6

### Priority 3: Solver Defaults (Stop being "Nice") → **EM-2**
**2961 missing errors (60%)** from optimistic `Any` fallback.

- **Root Cause:** Unresolvable symbols return `TypeId::ANY` instead of error
- **Impact:** TypeScript errors → "it's fine, it's any"
- **Action Items:**
  - Change defaults to `TypeId::UNKNOWN` or `TypeId::ERROR`
  - Expect regression spike (this exposes real bugs)
- **Target:** Stop hiding errors; short-term more errors = long-term accuracy
- **Assigned to:** worker-7

### Priority 4: Class Property Initialization (TS2564) → **EM-2**
**413 missing errors** - the #1 missing error category.

- **Root Cause:** `strictPropertyInitialization` check not implemented
- **Action Items:**
  - Implement CFA to verify class properties initialized in constructor
  - Respect definite assignment assertions (`!`)
- **Target:** Reduce from 413 to <20 missing errors
- **Assigned to:** worker-8

### Priority 5: Recursion Guards (Stability) → **EM-1**
**2 crashes** from stack overflow on recursive types.

- **Root Cause:** No depth limit in `solve_subtype` / `check_expression`
- **Action Items:**
  - Add `recursion_depth` counter
  - Return TS2589 error at limit (e.g., 100) instead of crashing
- **Target:** Zero crashes
- **Assigned to:** worker-4

---

## Work Distribution

### EM-1 Team (Syntax & Foundation)
| Worker | Squad | Priority | Task | Target |
|--------|-------|----------|------|--------|
| worker-1 | Syntax | 🔴 P1 | Parser Noise (TS1005/TS1109) | <40 errors |
| worker-2 | Binder | 🔴 P2 | Global Scope Fix (TS2304) | <10 errors |
| worker-3 | CFA | 🟡 P4 | Class Property Init (TS2564) | <20 errors |
| worker-4 | Stability | 🟢 P5 | Recursion Guards | Zero crashes |

### EM-2 Team (Semantics & Core)
| Worker | Squad | Priority | Task | Target |
|--------|-------|----------|------|--------|
| worker-5 | Syntax | 🔴 P1 | Parser Noise (backup/validation) | <40 errors |
| worker-6 | Binder | 🔴 P2 | Global Scope Fix (TS2304) | <10 errors |
| worker-7 | Semantics | 🟠 P3 | Invert Solver Defaults | Accuracy > comfort |
| worker-8 | CFA | 🟡 P4 | Class Property Init (TS2564) | <20 errors |

### EM-3 Team (Advanced Features & Integration)
| Worker | Squad | Priority | Task | Target |
|--------|-------|----------|------|--------|
| worker-9 | Integration | 🔵 P6 | WASM Bridge Layer | Seamless TS↔WASM |
| worker-10 | Type System | 🔵 P6 | Advanced Type Constructs | Generic parity |
| worker-11 | Performance | 🔵 P6 | Optimization & Profiling | Beat TS-Go |
| worker-12 | Testing | 🔵 P6 | Test Infrastructure | CI/CD stability |

---

## Success Metrics (All Teams)

### Baseline
- **Exact Match:** 30.1%
- **Parser Errors (TS1005/TS1109):** ~700 extra
- **Binder Errors (TS2304):** 343 extra, 116 missing
- **CFA Errors (TS2564):** 413 missing
- **Crashes:** 2

### Targets (Next Report)
- **Exact Match:** 80%+
- **TS1005/TS1109:** <40
- **TS2304 (Extra):** <10
- **TS2564 (Missing):** <20
- **Crashes:** 0

---

## Team Resizing Rules

1. **Max team size:** 4 workers per EM (current allocation at limit)
2. **Re-size after every merge** to rust
3. **Kill/recreate failing teams** if:
   - No progress after 3 merge cycles
   - EM unresponsive for >24 hours
   - Consistent test failures with no resolution plan
4. **Reassign workers** dynamically based on priority shifts

---

## 🔄 TEAM RESTRUCTURING (2026-01-15)

### Decision Summary

**Status:** EXECUTING - EM-3 merged, EM-1 and EM-2 pending merge

**Rationale:** Phase 1 (parser, binder, CFA) complete. Remaining work is type accuracy (TS2322, TS7006) which benefits from smaller, focused teams.

### New Team Structure

| Team | Previous Size | New Size | Change | Status |
|------|--------------|----------|--------|--------|
| EM-1 (Syntax & Foundation) | 4 workers | 4 workers | No change | ✅ Stable |
| EM-2 (Semantics & Core) | 4 workers | 2-3 workers | -1 to -2 | ⚠️ Resize pending |
| EM-3 (Advanced Features) | 4 workers | 2 workers | -2 | ✅ Approved |
| **Total** | **12 workers** | **8-9 workers** | **-3 to -4** | **In progress** |

### Worker Redistribution

#### Retaining Core Contributors (Keep)
| Worker | Team | Reason | New Focus |
|--------|------|--------|-----------|
| worker-9 | EM-3 | High performer, TS2322 expertise | Type accuracy |
| worker-11 | EM-3 | Has critical TS2322 analysis | Type accuracy |
| worker-1 | EM-1 | Parser fixes complete | Validation/support |
| worker-2 | EM-1 | Conformance validation (critical) | Testing |
| worker-3 | EM-1 | TS2564 Phase 2 complete | Available |
| worker-4 | EM-1 | TS2322 literal narrowing | Type accuracy |

#### Reassigning Available Workers
| Worker | From | Reason | New Assignment |
|--------|------|--------|----------------|
| worker-5 | EM-2 | High performer, available | TS7006 or validation |
| worker-6 | EM-2 | Off-track (wrong task) | Redirect or reassign |
| worker-7 | EM-2 | Solver defaults complete | Validation or testing |
| worker-8 | EM-2 | Recursion guards done | Overflow work |
| worker-10 | EM-3 | Module resolution complete | Validation or module testing |
| worker-12 | EM-3 | Task not started | Available for reassignment |

### Float Pool (Overflow Workers)

Workers not actively assigned to core teams form a "Float Pool" for:
- Validation and testing overflow
- Quick bug fixes
- Documentation and tooling
- Backup for critical workers

**Current Float Pool:** worker-5, worker-7, worker-8, worker-10, worker-12

### Updated Focus Areas

#### EM-1 (Syntax & Foundation) - UNCHANGED
- Focus: Parser validation, conformance testing, type accuracy
- Workers: worker-1, worker-2, worker-3, worker-4
- Status: All assigned to critical path

#### EM-2 (Semantics & Core) - RESIZING
- Focus: Type accuracy, validation, testing
- Workers: 2-3 workers (from 4)
- Status: Awaiting merge, then redistribute

#### EM-3 (Advanced Features) - RESIZED
- Focus: Type accuracy (TS2322, TS7006)
- Workers: worker-9, worker-11 (from 4)
- Status: Merged to rust, active on type accuracy

---

## Priority Order

**CRITICAL PATH:** Parser (Noise) → Binder (Poisoning) → Solver (Strictness) → CFA (Features)

**Do NOT work on:**
- New features until Parser noise is cleared
- Performance optimization until error parity is stable
- Advanced type constructs until core checker works

---

## EM Escalation Path

1. **Worker** → Pushes to worker branch, marks "Ready for Merge: Yes"
2. **EM** → Reviews, merges locally, runs conformance tests
3. **EM** → Escalates to Director only when stable
4. **Director** → Reviews EM branch, merges to rust, re-sizes teams

**Director merges ONLY EM branches—never worker branches directly.**
