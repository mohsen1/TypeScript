# Worker 2 Task List

**Maintained by**: EM-1
**Worker**: Worker 2
**Worktree**: /tmp/orchestrator-workspace/worktrees/worker-2
**Target Branch**: rust

---

## Tasks

### [ ] Task 2: Implement Private Name Field Branding (Category 2)
Implement TS2322 errors for private name field branding checks in the Rust thin checker.

**Test Files**:
1. `classes/members/privateNames/privateNameReadonly.ts`
   - Line 6: Private method reassignment
2. `classes/members/privateNames/privateNamesUnique-1.ts`
   - Line 13: Different private field brands
3. `classes/members/privateNames/privateNamesUnique-5.ts`
   - Line 16: Different private field brands

**Implementation Location**: `wasm/src/thin_checker.rs`

**Action**:
1. Study private name branding in TypeScript spec
2. Implement private field brand checking
3. Ensure TS2322 is emitted when private fields from different classes are mixed
4. Handle private method readonly violations
5. Run tests to verify implementation
6. Commit with message: "feat: implement private name field branding checks"
7. Push to origin worker-2

**Status**: Pending

---

## Completed Tasks

### ✅ Task 1: Cherry-pick super() call handling from rust branch
Successfully implemented special handling for `super()` calls in the Rust thin checker. Changes merged into rust branch and pushed to origin.

---

## Notes
- Private names in TypeScript use brand checking - private fields are only accessible within the class that declared them
- Refer to TypeScript's private name implementation for brand comparison logic
- Focus on detecting cross-class private field access
