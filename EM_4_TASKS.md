# EM-4 Task List

**Team:** worker-12, worker-13, worker-14
**Branch:** em-team-4
**Worktree:** /var/folders/57/xp3brw212ygckhkk_ml783fr0000gn/T/cco-workspace-TypeScript-1768508073410/worktrees/em-4

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

### worker-12: Parser Accuracy (Tier 1)
**Focus:** Fix TS1109 and TS1005 extra errors - parser false positives

**Issues:**
- TS1109 "Expression expected" - appears as both missing (7) and extra (7)
- TS1005 "X expected" - 5 extra occurrences

**Key Files:**
- `wasm/src/thin_parser.rs`
- `wasm/src/scanner.rs`

**Approach:**
1. Find failing test examples from conformance output
2. Create minimal repro files
3. Compare TSC vs WASM parse trees
4. Fix parser edge cases (ASI, expression statement detection, etc.)

**Success Criteria:** TS1109/TS1005 extra errors reduced by 70%

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
| worker-12 | Parser (TS1109/TS1005) | Pending | - |
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
- Pushed em-team-4 to origin for Director review
