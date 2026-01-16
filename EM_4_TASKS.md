# EM-4 Task List

**Team:** worker-12, worker-13, worker-14
**Branch:** em-team-4
**Worktree:** /var/folders/57/xp3brw212ygckhkk_ml783fr0000gn/T/cco-workspace-TypeScript-1768516891306/worktrees/em-4

---

## Latest Status (2026-01-15 18:45)

### ✅ Push Complete - Blockage Resolved
- **Action:** Force push to origin per Director instruction
- **Result:** Remote updated from 5ba878e63 → bfc642c2e
- **Local State:** Clean, synced with rust, validated (44.4% exact match)
- **Latest Commit:** bfc642c2e "docs: Update EM_4_TASKS.md - push blockage persists"
- **Status:** All local commits now on origin, ready for Director review

### Worker Reassignment Complete ✅
All workers reassigned to **Tier 0: Quality & Stability Foundations**

| Worker | New Assignment | Priority | Status |
|--------|---------------|----------|--------|
| worker-12 | Application Type Expansion | 🔴 CRITICAL | 🔵 Active |
| worker-13 | Solver Test Coverage | 🔴 CRITICAL | 🔵 Active |
| worker-14 | AST Child Enumeration | 🔴 CRITICAL | 🔵 Active |

### Previous Completed Tasks
- ✅ worker-13: Async/await checks (TS2705/TS1359)
- ✅ worker-12: Parser fixes (TS1109/TS1005)
- ✅ worker-14: TS2304 symbol resolution investigation

## Baseline Metrics (200 files)

| Metric | Value |
|--------|-------|
| Exact Match | 55/190 (28.9%) |
| Same Error Count | 0 |
| Missing Errors | 93 tests (48.9%) |
| Extra Errors | 42 tests (22.1%) |
| Crashed | 0 |

## Top Error Codes (from baseline)

### Most Missing Errors (TSC emits, WASM doesn't):
1. **TS2705** - 34 occurrences - Async function return type checking
2. **TS2524** - 12 occurrences - Module member resolution failures
3. **TS1109** - 7 occurrences - "Expression expected" (parser)
4. **TS1359** - 7 occurrences - 'await' reserved word detection
5. **TS2304** - 7 occurrences - "Cannot find name" (symbol resolution)

### Most Extra Errors (WASM emits, TSC doesn't):
1. **TS1109** - 7 occurrences - Parser false positives
2. **TS2322** - 6 occurrences - Type assignability false positives
3. **TS1005** - 5 occurrences - Parser "X expected" false positives
4. **TS2304** - 5 occurrences - Symbol resolution false positives
5. **TS2339** - 5 occurrences - Property access errors

## Task Assignments

### worker-12: Type Checking (Tier 2) - *Previous Assignment*
**Note:** Worker-12 was assigned TS2571/TS2683 (this type checking) under EM-3 before EM-4 activation.
**Status:** Merged task list only - needs reassignment to current EM-4 priorities.

**Previous Focus (from worker-12 branch):**
- TS2571 - "Object is of type 'unknown'" over-reporting
- TS2683 - "'this' implicitly has type 'any'" missing errors
- Error reclassification for non-method function `this` handling

**Current EM-4 Assignment (Parser Tier 1):**
- TS1109 - 7 missing, 7 extra (parser accuracy)
- TS1005 - 5 extra (parser "X expected")

---

### worker-13: Async/Await (Tier 5)
**Focus:** Fix TS2705 and TS1359 - async function return types and await detection

**Issues:**
- TS2705 - 34 missing (async function return type checking)
- TS1359 - 7 missing ('await' reserved word detection)

**Key Files:**
- `wasm/src/thin_checker.rs` - async-related functions around line 6000-7000
- `wasm/src/checker/types/diagnostics.rs` - add TS2705/TS1359 if missing

**Approach:**
1. Examine async test failures (category: "async")
2. TS2705: Verify async function return type is wrapped in Promise
3. TS1359: Detect 'await' usage in non-async contexts
4. Add missing diagnostics if codes don't exist

**Success Criteria:** TS2705 missing errors reduced by 70%

---

### worker-14: Type Checking (Tier 2) - *Previous Assignment*
**Note:** Worker-14 was assigned union type assignability (TS2322) before EM-4 activation.
**Status:** Merged task list only - no code changes yet.
**Action:** EM-4 to review and reassign to current priorities (Symbol Resolution Tier 3).

**Previous Focus (from worker-14 branch):**
- TS2322 - ~548 extra errors (union type assignability)
- Union-to-union and base-to-union type compatibility

**Current EM-4 Assignment (Symbol Resolution Tier 3):**
- TS2304 - 7 missing, 5 extra (symbol resolution accuracy)
- TS2524 - 12 missing (module member resolution)

**Key Files:**
- `wasm/src/binder/` - symbol table and scope management
- `wasm/src/thin_checker.rs` - symbol resolution logic
- `wasm/src/parallel.rs` - module handling

**Approach:**
1. Examine symbol test failures (category: "Symbols", "ambient")
2. TS2304: Fix global/local symbol lookup gaps
3. TS2524: Implement module member resolution checking
4. Add missing diagnostics if codes don't exist

**Success Criteria:** TS2304/TS2524 missing errors reduced by 60%

---

## Team Workflow

### For EM-4:
1. Merge worker branches locally after validation
2. Run full conformance (500 files) before escalating to Director
3. Update this file with progress
4. Reassign workers as tasks complete

### For Workers:
1. Read your assigned `WORKER_<id>_TASK_LIST.md`
2. Create feature branch from em-team-4
3. Build WASM: `cd wasm && wasm-pack build --target web --out-dir pkg`
4. Test: `cd wasm/differential-test && bash run-conformance.sh --max=200 --workers=4`
5. Commit with descriptive messages
6. Notify EM-4 when ready for review

## Validation Commands

```bash
# Build WASM
cd wasm && wasm-pack build --target web --out-dir pkg

# Quick test
cd wasm/differential-test && bash run-conformance.sh --max=200 --workers=4

# Standard test
cd wasm/differential-test && bash run-conformance.sh --max=500 --workers=8

# Target: 95%+ exact match for production
```

## Progress Tracking

| Worker | Task | Status | Notes |
|--------|------|--------|-------|
| worker-12 | Parser (TS1109/TS1005) | Merged | Previous assignment (TS2571/TS2683) merged - needs reassignment |
| worker-13 | Async (TS2705/TS1359) | Pending | - |
| worker-14 | Symbols (TS2304/TS2524) | Merged | Previous assignment (TS2322) merged - needs reassignment |

---

**Last Updated:** 2026-01-15
**Next Review:** After worker code submissions

## Recent Activity

### 2026-01-15 - EM-4 Activated
- Synced with rust branch
- Created baseline: 28.9% exact match (55/190 tests)
- Assigned tasks to workers 12, 13, 14
- Merged worker-14 branch (previous assignment: TS2322 union types)
- Merged worker-12 branch (previous assignment: TS2571/TS2683 under EM-3)
- Pushed em-team-4 to origin for Director review

**Status:** All workers merged with previous assignments - awaiting reassignment to EM-4 priorities.
