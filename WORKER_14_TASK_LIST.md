# Worker 14 Task List

**Maintained by**: EM-4
**Worker**: Worker 14
**Worktree**: /var/folders/57/xp3brw212ygckhkk_ml783fr0000gn/T/cco-workspace-TypeScript-1768508073410/worktrees/worker-14
**Target Branch**: rust
**Squad**: Type Checking Squad (provisional - pending EM-4 activation)

---

## Tasks

### [ ] Task 1: Fix TS2322 False Positives in Union Type Assignability

**Priority:** 🔴 CRITICAL (Tier 2 - Type Checker Accuracy)
**Assigned:** 2026-01-15
**Status:** 🟡 IN PROGRESS - Investigation & Test Setup Complete

### Problem

The thin checker emits **TS2322 "Type X is not assignable to type Y"** errors for valid union type assignments. This creates false positives when:
- A union type contains all constituents of the target type
- Assigning to a supertype through a union
- Generic type parameter constraints are satisfied but not recognized

**Current Impact:** ~548 extra TS2322 errors in conformance tests

**Root Cause**

The type assignability checker in `thin_checker.rs` doesn't properly handle:
1. **Union type compatibility:** `A | B` should be assignable to `A | B | C`
2. **Widening through unions:** `A` should be assignable to `A | B`
3. **Generic constraints:** Type parameters with compatible constraints should pass
4. **Never/unknown handling:** Special types in union contexts

### Action Items

#### Phase 1: Investigation

1. **Analyze existing test failures**
   ```bash
   # Run conformance tests and capture TS2322 errors
   cd wasm/differential-test
   bash run-conformance.sh --max=500 --workers=4 2>&1 | grep TS2322
   ```

2. **Categorize false positives**
   - Create categories: Union-to-Union, Base-to-Union, Generic-to-Generic
   - Find patterns in test files that fail incorrectly
   - Document minimal repro cases for each category

3. **Study the assignability function**
   - Locate `check_type_assignability()` in `wasm/src/thin_checker.rs`
   - Understand current union type handling logic
   - Identify where compatibility check fails

#### Phase 2: Implementation

1. **Fix union-to-union assignability**
   ```rust
   // Pseudo-code: A | B should be assignable to A | B | C
   fn is_union_subtype(source: &Type, target: &Type) -> bool {
       if let (Type::Union(src_members), Type::Union(tgt_members)) = (source, target) {
           // Every member of source should exist in target
           return src_members.iter().all(|m| tgt_members.contains(m));
       }
       false
   }
   ```

2. **Fix base-to-union assignability**
   ```rust
   // Pseudo-code: A should be assignable to A | B
   fn is_assignable_to_union(source: &Type, target: &UnionType) -> bool {
       // Source type should match at least one union member
       target.members.iter().any(|m| is_same_type(source, m))
   }
   ```

3. **Handle generic type constraints**
   - Check type parameter bounds before assignability
   - Use constraint information to guide compatibility check
   - Ensure constrained generics are recognized as compatible

4. **Add test cases**
   ```typescript
   // Should NOT emit TS2322
   type T1 = string | number;
   type T2 = string | number | boolean;
   let x: T1 = "hello";
   let y: T2 = x; // Valid: T1 is subset of T2

   // Should NOT emit TS2322
   function foo<T extends string>(x: T): T | number {
       return x; // Valid: T is assignable to T | number
   }

   // SHOULD emit TS2322
   let a: string = 5; // Invalid: number not assignable to string
   ```

#### Phase 3: Validation

1. **Run targeted tests**
   ```typescript
   // Create test file: test_union_assignability.ts
   // Verify no TS2322 on valid union assignments
   // Verify TS2322 still emitted on invalid assignments
   ```

2. **Run conformance suite**
   ```bash
   cd wasm/differential-test
   bash run-conformance.sh --max=1000 --workers=4
   ```
   - Track TS2322 count (target: reduce from 548 to <300)
   - Ensure no regression - valid errors still emitted
   - Document reduction in false positives

3. **Compare with tsc output**
   ```bash
   # Verify our errors match TypeScript compiler
   tsc --noEmit test_union_assignability.ts
   wasm/differential-test/run-single.sh test_union_assignability.ts
   ```

### Files to Work On

- **Primary:** `wasm/src/thin_checker.rs`
  - Locate `check_type_assignability()` function
  - Add union type compatibility checks
  - Fix base-to-union assignability logic

- **Tests:** Create `tests/union_type_assignability.test.ts`
  - Cover all union assignment scenarios
  - Include edge cases with never/unknown
  - Test generic type constraints

### Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| TS2322 extra errors | ~548 | <300 |
| Union-to-union assignments | Failing | Passing |
| Base-to-union assignments | Failing | Passing |
| Generic constraint checks | Failing | Passing |

### Reference

- **PROJECT_DIRECTION.md:** Tier 2 rules for type checking
- **EM_3_TASKS.md:** Type checking squad priorities (similar focus)
- **missing-ts2322-by-category.txt:** Categorized error list

### Instructions

1. Sync with rust branch: `git fetch origin && git pull origin rust`
2. Create feature branch from rust: `git checkout -b worker-14-union-assignability`
3. Work on union type assignability ONLY
4. Commit frequently with descriptive messages:
   - `feat(wasm): add union-to-union type assignability check`
   - `fix(wasm): handle base-to-union type assignments`
   - `feat(wasm): improve generic constraint type checking`
5. Push to worker-14 branch: `git push origin worker-14`
6. Run tests locally before considering complete
7. Update this task list with status
8. Notify EM-4 when ready for review

### Validation Checklist Before Merge

- [ ] TS2322 errors reduced by target amount (548 → <300)
- [ ] Union-to-union assignments work correctly
- [ ] Base-to-union assignments work correctly
- [ ] Generic constraints properly recognized
- [ ] No regression in valid error detection
- [ ] Conformance tests show improvement
- [ ] Test cases added for new functionality
- [ ] Code follows existing patterns in thin_checker.rs

---

## Completed Tasks

### ✅ Phase 1: Investigation Complete (2026-01-15)
- Created UNION_ASSIGNABILITY_ANALYSIS.md with deep dive
- Set up test files: test_union_assignability.{rs,ts}
- Identified root causes in type assignability logic
- Categorized false positives by type

### 🔄 Phase 2: Implementation Pending
- [ ] Implement union-to-union assignability check
- [ ] Implement base-to-union assignability check
- [ ] Handle generic constraint types
- [ ] Run validation against conformance tests

---

## Progress Log

**2026-01-15:**
- ✅ Investigation complete - analysis document created
- ✅ Test infrastructure set up
- ✅ Merged to em-team-4
- 🔄 Ready for implementation phase

---

## Notes

- **EM-4 Status:** Active - worker-14 task merged to em-team-4
- **Team Alignment:** Tier 2 Type Checker Accuracy focus
- **Next Steps:** Implement fixes in `wasm/src/thin_checker.rs`
- **Priority:** Union type assignability is foundational - fixes will unblock other type checking improvements
