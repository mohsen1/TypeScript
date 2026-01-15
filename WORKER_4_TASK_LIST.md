# Worker 4 Task List

Maintained by EM-1

## Current Tasks

### [ACTIVE] Task 1: Fix Async/Await Type Checking (TS2705, TS1359)

**Priority:** High (Tier 5 - Async/Await)

**Status:** In Progress - Not Started

**Description:**
The type checker needs correct handling of async functions, generators, and await expressions. Current gaps include TS2705 (async function return type checking), TS1359 ('await' reserved word detection), and async generator return types.

**Error Examples:**
```typescript
// TS2705 - Async function return type checking
async function foo(): Promise<number> {
    return 42;  // Should OK
    return "string";  // Should error: Type 'string' is not assignable to type 'number'
}

// TS1359 - 'await' reserved word detection
function regular() {
    await Promise.resolve(1);  // Should error: 'await' is only allowed in async functions
}

// Async generators
async function* gen(): AsyncGenerator<number> {
    yield 1;  // Should OK
    yield Promise.resolve(2);  // Should handle correctly
}
```

**Action Items:**

1. **Locate async-related code** in `wasm/src/thin_checker.rs`
   - Search for `async`, `await`, `Promise` handling
   - Find function return type checking for async functions
   - Locate generator type handling

2. **Implement TS2705 checks:**
   - Verify async function return type is `Promise<T>` or compatible
   - Check that returned values match the Promise's type parameter
   - Handle implicit Promise wrapping

3. **Implement TS1359 checks:**
   - Detect `await` usage in non-async functions
   - Emit TS1359 error when found
   - Add context check for async function scope

4. **Fix async generator handling:**
   - Distinguish between `AsyncGenerator` and `Promise` return types
   - Handle `yield` expressions in async generators
   - Ensure proper type inference for async generators

5. **Test cases to verify:**
   ```typescript
   // TS2705 - Return type mismatches
   async function bad1(): Promise<number> {
       return "string";  // Should error
   }

   // TS1359 - await in non-async
   function bad2() {
       await Promise.resolve(1);  // Should error
   }

   // Should work correctly
   async function good1(): Promise<number> {
       return 42;
   }

   async function* good2(): AsyncGenerator<number> {
       yield 1;
   }
   ```

6. **Run conformance tests:**
   ```bash
   cd wasm/differential-test
   bash run-conformance.sh --max=200 --workers=4
   ```
   - Track TS2705 missing errors (target: fill gaps)
   - Track TS1359 missing errors (target: emit correctly)
   - Ensure no regressions in async handling

**Target Metrics:**
| Error Code | Current | Target |
|------------|---------|--------|
| TS2705 gaps | Unknown | Fill all gaps |
| TS1359 missing | Unknown | 100% detection |
| Async generator issues | Unknown | Correct handling |

**Key Files:**
- `wasm/src/thin_checker.rs` - async-related functions
- `wasm/src/checker/types/diagnostics.rs` - error code definitions

**Reference:** See `PROJECT_DIRECTION.md` Tier 5 section for async/await requirements.

---

## Completed Tasks

*None yet*

---

## Notes

- **Worktree:** `/tmp/orchestrator-workspace/worktrees/worker-4`
- **Branch:** `worker-4`
- **Target branch:** `em-team-1`
- **Squad:** Async Squad
- **Focus:** Async/await type checking (TS2705, TS1359)

## Workflow

1. Sync with em-team-1: `git pull origin em-team-1`
2. Create feature branch: `git checkout -b worker-4`
3. Make changes in `wasm/` directory only
4. Commit with format: `[wasm] checker: implement TS2705 async return type checking`
5. Push to worker-4 branch
6. Run tests locally
7. Update this task list with status
8. Notify EM-1 when ready for merge

## Validation Checklist Before Merge

- [ ] TS2705 errors emitted for async return type mismatches
- [ ] TS1359 errors emitted for await in non-async functions
- [ ] Async generators return correct types
- [ ] No regression in other error codes
- [ ] Conformance tests pass
- [ ] Minimal repro tests validate fix
- [ ] Code follows Rust best practices
