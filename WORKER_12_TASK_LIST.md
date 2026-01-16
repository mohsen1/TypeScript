# WORKER 12 TASK LIST

## Worker: worker-12
## Reports to: EM-4
## EM Branch: em-team-4
## Worker Branch: worker-12
## Base Branch: rust

---

## Assignment: Application Type Expansion

**Priority:** 🔴 CRITICAL (Tier 0 - Quality & Stability)
**Status:** 🔵 Active
**Started:** 2026-01-15

---

## Mission

Fix `TypeKey::Application` type expansion. Application types are not being expanded, leading to incorrect diagnostics and assignability results. This is a foundational issue that affects type checking correctness across multiple tiers.

---

## Problem Analysis

**Current Behavior:**
- `TypeKey::Application` types are stored but not expanded during type checking
- This leads to incorrect type comparisons and diagnostics
- Affects assignability checks, subtype relationships, and error reporting

**Why This Matters:**
- Application types (e.g., `Promise<T>`, `Array<T>`) need expansion to check their actual type arguments
- Without expansion, `Promise<string>` and `Promise<number>` might be considered equal
- Foundational to correct type checking behavior

---

## Task Breakdown

### Phase 1: Investigation
- [ ] Study existing instantiation logic in `wasm/src/solver/instantiate.rs`
- [ ] Understand how `TypeKey::Application` is currently stored
- [ ] Find where type expansion should occur but doesn't
- [ ] Identify test cases showing incorrect behavior

### Phase 2: Code Analysis
- [ ] Review `wasm/src/solver/intern.rs` - type interning logic
- [ ] Review `wasm/src/solver/subtype.rs` - subtype checking
- [ ] Find where Application types should be expanded
- [ ] Map the expansion flow from instantiation to comparison

### Phase 3: Implementation
- [ ] Implement type expansion for Application types:
  - When comparing Application types, expand both to their actual types
  - Use existing instantiation logic from `instantiate.rs`
  - Ensure recursive expansion for nested Applications
- [ ] Add expansion calls at key comparison points
- [ ] Test with generic types: `Promise<T>`, `Array<T>`, `Map<K,V>`

### Phase 4: Validation
- [ ] Run Rust tests: `./wasm/test.sh` (Docker required)
- [ ] Re-enable solver tests once API is updated
- [ ] Run conformance tests to verify no regression
- [ ] Create test cases for Application type comparisons

---

## Key Files

| File | Purpose |
|------|---------|
| `wasm/src/solver/instantiate.rs` | Existing instantiation logic to reuse |
| `wasm/src/solver/intern.rs` | Type interning and TypeKey definitions |
| `wasm/src/solver/subtype.rs` | Subtype checking where expansion is needed |
| `wasm/src/solver/evaluate.rs` | Type evaluation logic |
| `wasm/differential-test/` | Conformance test suite |

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Application types expand correctly | ✅ |
| Generic type comparisons work | ✅ |
| Solver tests re-enabled | ✅ |
| No regression in conformance tests | ✅ |

---

## Workflow

1. **Sync with EM-4:**
   ```bash
   git fetch origin
   git pull origin rust --rebase
   ```

2. **Work on task:**
   - Make changes in `wasm/src/solver/` directory
   - Commit frequently: `git commit -m "[wasm] solver: <description>"`
   - Push to worker-12: `git push origin worker-12`

3. **Validation:**
   - Run Rust tests before pushing
   - Document test results in commit messages

4. **When complete:**
   - Update this task list with completion status
   - Notify EM-4 for merge review

---

## Progress Log

### 2026-01-15 - New Assignment 🔵
- **Previous:** Completed TS1109/TS1005 Parser Fixes
- **New:** Application Type Expansion (Tier 0)
- **Status:** Starting investigation phase

### Previous Completed Tasks
- ✅ TS1109/TS1005 Parser Fixes (2026-01-15)
- ✅ TS2571/TS2683 (transferred from EM-3, incomplete)

---

## Notes

- **READ-ONLY:** Never modify `src/compiler/` (TypeScript source)
- **Docker required:** Rust tests need Docker environment
- **Commit format:** `[wasm] checker: <clear description>`
- **Target:** Consistent error messages with TypeScript
- **Reference:** Worker 1's commit c958fc9cb for TS2683 implementation

---

## Escalation Path

1. Worker 12 commits → worker-12 branch
2. EM-3 reviews → merges to em-team-3
3. EM-3 validates → escalates to Director
4. Director reviews → merges to rust

**STOP after pushing to worker-12 and wait for EM-3 merge approval.**
