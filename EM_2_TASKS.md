# EM-2 Tasks - Parser Squad

**Branch:** `em-team-2`
**Priority:** 🟠 HIGH
**Assigned Workers:** workers 1, 2, 3 (transferred from EM-1 after validation)
**Last Updated:** 2026-01-14 (Post EM-1 validation)

---

## Mission: Complete Parser False Positive Elimination

### EM-1 Validation Results (Baseline for EM-2)

| Metric | Original | After EM-1 | EM-2 Target |
|--------|----------|------------|-------------|
| **TS1005** | 439 | **267** | **<100** |
| **TS1109** | 262 | **122** | **<100** |
| **Total Parser FP** | 701 | **389** | **<200** |
| **Exact Match** | 30.1% | **34.5%** | **40%** |

**EM-1 Achievements:**
- Worker 1: TS1005 patterns 1-5 → -127 errors
- Worker 2: TS1109 definite assignment → -64 errors
- Worker 3: Cascading error suppression → -150 errors
- Combined: 312 errors eliminated (-44%)

### EM-2 Responsibility

**Reduce remaining 389 parser false positives to <200.**

Focus areas:
1. **Comma inference** in object/array literals (~85 cases)
2. **new.target context** validation (~52 cases)
3. **Statement termination** edge cases (~38 cases)
4. **Template literal** expressions (~27 cases)
5. Remaining edge cases (~187 cases)

---

## Worker Assignments

### Worker 1 (TS1005 - Comma & Statement Focus)
**Expertise:** TS1005 patterns, semicolon/ASI handling
**Status:** Transferred from EM-1 ✅
**Priority Target:** ~123 cases to eliminate

**Tasks:**
1. Fix comma inference in object literals (~50 cases)
   - `{a: 1 b: 2}` should recover on missing comma
   - Improve ASI detection in object literal parsing
2. Fix comma inference in array literals (~35 cases)
3. Fix statement termination edge cases (~38 cases)
   - Class methods, getters/setters
   - Semicolon vs ASI confusion

### Worker 2 (TS1109 - new.target & Templates Focus)
**Expertise:** TS1109 patterns, expression parsing
**Status:** Transferred from EM-1 ✅
**Priority Target:** ~79 cases to eliminate

**Tasks:**
1. Fix new.target context validation (~52 cases) - HIGH IMPACT
   - Parser emits TS1109 when `new.target` used outside constructor
   - Need better context tracking
2. Fix template strings in type positions (~14 cases)
3. Fix private names in `in` expressions (~9 cases)
4. Fix remaining destructuring edge cases (~4 cases)

### Worker 3 (Error Recovery Infrastructure)
**Expertise:** Cascading error suppression, parser recovery
**Status:** Transferred from EM-1 ✅
**Priority Target:** Support role + ~67 cases

**Tasks:**
1. **SUPPORT:** Review and amplify Workers 1 & 2 fixes
   - Test comma inference fixes with cascading error suppression
   - Ensure new.target fixes don't create cascading errors
2. Fix type parameter bracket recovery (~25 cases)
   - Missing > in generics cascades to multiple TS1005
   - Coordinate with Worker 1's comma inference work
3. Fix remaining edge cases (~42 cases)
   - Import/export declaration errors
   - Heritage clause commas

---

## EM-2 Responsibilities

### 1. Branch Hygiene
- [ ] Sync em-team-2 with rust
- [ ] Merge worker branches locally only after validation
- [ ] Run full conformance suite before any merge to rust
- [ ] Escalate to Director only when metrics show stable improvement

### 2. Task Assignment Strategy
- **Parallel work:** Workers 1-2 can work independently on different patterns
- **Support role:** Worker 3 amplifies and validates other fixes
- **Weekly baselines:** All workers run conformance every Friday

### 3. Validation Protocol
Before merging any worker branch:
1. Worker must run conformance tests and report metrics
2. Verify no regressions in other error codes
3. Ensure build passes (`cargo build --release`)
4. Check that the specific metric improved (e.g., TS1005 count decreased)

---

## Success Criteria

- TS1005 reduced from 267 to <100
- TS1109 reduced from 122 to <100
- Total parser FP reduced from 389 to <200
- Exact Match increases from 34.5% to 40%
- All workers transferred and productive
- Ready to merge em-team-2 to rust

---

## Key Files

| Focus Area | File |
|------------|------|
| **Parser** | `src/compiler/parser.ts` |
| **Error Recovery** | `wasm/src/thin_parser.rs` |
| **Conformance** | `tests/conformance/` |

## Merge Protocol
1. Workers push to their feature branches
2. EM-2 merges worker branches locally to em-team-2
3. Run validation: `cargo test && npm run conformance`
4. Only escalate to Director when metrics improve and tests pass

## Blocking Issues
None - all workers can start in parallel

## Next Actions
1. Assign tasks to worker-5 through worker-8
2. Monitor progress via WORKER_*_TASK_LIST.md updates
3. Merge completed work to em-team-2
4. Run conformance to measure impact
