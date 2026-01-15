# EM-1 TASK LIST

## Team: em-team-1
## Manager: EM-1
## Base Branch: rust
## Team Branch: em-team-1

---

## Team Composition

| Worker | Squad | Focus Area | Status |
|--------|-------|------------|--------|
| worker-1 | Type Squad | Implicit `this` handling (TS2683) | Active |
| worker-2 | Type Squad | `super()` call handling | Active |
| worker-3 | AnyCheck Squad | Implicit any detection (TS7006/TS7005) | Active |
| worker-4 | Async Squad | Async/await type checking (TS2705) | Active |

---

## Team Priorities (Updated 2026-01-15)

### Priority 1: Type Checker Accuracy - `this` and `super` (Tier 2)
**Owner:** worker-1, worker-2

**Goal:** Fix object-oriented type checking edge cases

| Issue | Description | Owner |
|-------|-------------|-------|
| TS2683 missing | "'this' implicitly has type 'any'" not emitted | worker-1 |
| TS2571 extra | "Object is of type 'unknown'" over-reported | worker-1 |
| super() handling | Special handling for super() calls in constructors | worker-2 (completed) |

**Root Cause:** When `this` is used inside a regular function (not a method), it should emit TS2683 but instead types as `unknown` and emits TS2571 on property access.

**Fix Location:** `thin_checker.rs` - `current_this_type()` handling around line 629

### Priority 2: Implicit Any Checks (Tier 4)
**Owner:** worker-3 (AnyCheck Squad)

**Goal:** Emit TS7006/TS7008 only when type cannot be inferred

| Error Code | Current | Target | Owner |
|------------|---------|--------|-------|
| TS7006 extra | ~200 (estimate) | <50 | worker-3 |
| TS7005 extra | ~150 (estimate) | <30 | worker-3 |

**Key Files:** `wasm/src/thin_checker.rs` - implicit any checking functions

**Rules to Implement:**
- Skip implicit any errors when parameter has default value (`param.initializer.is_some()`)
- Skip implicit any errors when property has initializer (`prop.initializer.is_some()`)
- Skip implicit any errors when type can be inferred from usage

### Priority 3: Async/Await Type Checking (Tier 5)
**Owner:** worker-4 (Async Squad)

**Goal:** Correct handling of async functions, generators, and await expressions

| Error Code | Description | Owner |
|------------|-------------|-------|
| TS2705 gaps | Async function return type checking | worker-4 |
| TS1359 missing | 'await' reserved word detection | worker-4 |
| Async generators | `AsyncGenerator` vs `Promise` return types | worker-4 |

**Key Files:** `wasm/src/thin_checker.rs` - async-related functions

---

## Recent Activity Log

### 2026-01-15
- EM-1 initialized
- Created em-team-1 branch from rust
- Merged worker-1 (template only)
- Merged worker-2 (super() handling fix - dc7519914)
- Created EM_1_TASKS.md

### Completed Work
- worker-2: fix: add special handling for super() calls in ThinCheckerState (dc7519914)
- worker-1: fix: implement TS2683 for implicit this in functions (c958fc9cb)

---

## Conformance Test Baseline (2026-01-15)

Current baseline from rust branch (commit 978ce6786):

| Metric | Result | Goal |
|--------|--------|------|
| Exact Match | TBD | 95%+ |
| Same Error Count | TBD | - |
| Missing Errors | TBD | <5% |
| Extra Errors | TBD | <5% |

**Action Items:**
- Run conformance tests to establish EM-1 baseline
- Track TS2683, TS2571, TS7006, TS7005, TS2705 specific counts

---

## Workflow

### For EM-1:
1. **Daily sync**: `git pull origin rust` → merge to em-team-1
2. **Review worker branches**: Check commits, test results
3. **Merge locally**: `git merge worker-X` into em-team-1
4. **Run validation**: `cd wasm/differential-test && bash run-conformance.sh --max=500 --workers=4`
5. **Push to director**: Only when stable and validated

### For Workers:
1. Create branch from em-team-1
2. Work on assigned task ONLY
3. Commit frequently with `[wasm] <component>: <description>`
4. Push to worker-X branch
5. Update task list with status
6. Notify EM-1 when ready for merge

---

## Merge Readiness Status (2026-01-15)

| Worker | Status | Notes |
|--------|--------|-------|
| worker-1 | 🟡 Active | TS2683 fix complete (c958fc9cb), needs validation |
| worker-2 | 🟢 Merged | super() handling fix complete (dc7519914) |
| worker-3 | 🟡 Pending | TS7006/TS7005 task assignment pending |
| worker-4 | 🟡 Pending | TS2705 async/await task assignment pending |

---

## Escalation Path

1. Worker commits → worker-X branch
2. EM-1 merges to em-team-1 → validates
3. EM-1 escalates to Director when stable
4. Director reviews → merges to rust

**Do NOT push directly to rust.**

---

## Next Actions for EM-1

1. ✅ Sync em-team-1 with rust
2. ✅ Merge worker-1 into em-team-1
3. ✅ Merge worker-2 into em-team-1
4. 🔄 Create WORKER_3_TASK_LIST.md with TS7006/TS7005 assignment
5. 🔄 Create WORKER_4_TASK_LIST.md with TS2705 async/await assignment
6. 📅 Run conformance tests to establish baseline
7. 📋 Push em-team-1 to origin for director review

---

## Notes

- All work must stay in `wasm/` directory
- TypeScript source files (`src/compiler/`) are READ-ONLY
- Run `./wasm/test.sh` for Rust tests (Docker-only)
- Use `./scripts/ask-gemini.mjs` before coding (if applicable)
- Target: 95%+ exact match before production
- Worktree location: `/tmp/orchestrator-workspace/worktrees/em-1`
