# Worker 3 Task List

Maintained by EM-1

## Current Tasks

### [ACTIVE] Task 1: Fix TS7006/TS7005 Extra Errors (Implicit Any Over-reporting)

**Priority:** High (Tier 4 - Implicit Any Checks)

**Status:** In Progress - Not Started

**Description:**
The type checker is emitting TS7006 (parameter implicit any) and TS7005 (variable implicit any) errors in cases where the type can be inferred. This creates noise and false positives.

**Error Examples:**
```typescript
// Should NOT error - type inferred from default value
function foo(param = 5) {  // Currently emits TS7006, should not
    return param;
}

// Should NOT error - type inferred from initializer
const x = 5;  // Currently may emit TS7005, should not

// SHOULD error - no type inference possible
function bar(param) {  // Should emit TS7006
    return param;
}
```

**Action Items:**
1. **Locate implicit any checking code** in `wasm/src/thin_checker.rs`
   - Search for `TS7006` and `TS7005` error codes
   - Find functions that check parameter and variable types

2. **Add inference checks** before emitting errors:
   - Skip TS7006 when `param.initializer.is_some()`
   - Skip TS7005 when `prop.initializer.is_some()`
   - Add logic to detect when type can be inferred from usage

3. **Test cases to verify:**
   ```typescript
   // Should NOT error
   function test1(x = 5) { return x; }
   function test2({ a = 1 } = {}) { return a; }
   const y = 10;

   // SHOULD error
   function test3(z) { return z; }
   ```

4. **Run conformance tests:**
   ```bash
   cd wasm/differential-test
   bash run-conformance.sh --max=200 --workers=4
   ```
   - Track TS7006 count (target: reduce by ~150+)
   - Track TS7005 count (target: reduce by ~100+)

5. **Create minimal repro tests** for validation

**Target Metrics:**
| Error Code | Current | Target |
|------------|---------|--------|
| TS7006 extra | ~200 | <50 |
| TS7005 extra | ~150 | <30 |

**Key Files:**
- `wasm/src/thin_checker.rs` - implicit any checking functions
- `wasm/src/checker/types/diagnostics.rs` - error code definitions

**Reference:** See `PROJECT_DIRECTION.md` Tier 4 section for rules on when to skip implicit any errors.

---

## Completed Tasks

*None yet*

---

## Notes

- **Worktree:** `/tmp/orchestrator-workspace/worktrees/worker-3`
- **Branch:** `worker-3`
- **Target branch:** `em-team-1`
- **Squad:** AnyCheck Squad
- **Focus:** Implicit any detection (TS7006/TS7005)

## Workflow

1. Sync with em-team-1: `git pull origin em-team-1`
2. Create feature branch: `git checkout -b worker-3`
3. Make changes in `wasm/` directory only
4. Commit with format: `[wasm] checker: fix TS7006 over-reporting`
5. Push to worker-3 branch
6. Run tests locally
7. Update this task list with status
8. Notify EM-1 when ready for merge

## Validation Checklist Before Merge

- [ ] TS7006 errors reduced by target amount
- [ ] TS7005 errors reduced by target amount
- [ ] No regression in other error codes
- [ ] Conformance tests pass
- [ ] Minimal repro tests validate fix
- [ ] Code follows Rust best practices
