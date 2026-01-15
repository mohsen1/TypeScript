# Worker-3 Task List

## 🟢 CURRENT TASK: Recursion Guards (Stack Overflow Prevention)
**Priority:** 🟢 STABILITY (Critical)
**Owner:** worker-3
**Branch:** worker-3
**Status:** 🟡 IN PROGRESS
**Assigned:** 2026-01-15

---

## Task Description

**Problem:** 2 Crashes (Stack Overflow) on `types/typeRelationships/recursiveTypes` test.

**Root Cause:** The `solve_subtype` and `check_expression` functions recurse infinitely on recursive types, causing the WASM process to panic/stack overflow.

**Target:** Add recursion depth counters and return TS2589 error instead of crashing

---

## Analysis Required

### Phase 1: Investigation (DO THIS FIRST)

**Before making changes:**

1. **Find the crash location:**
   ```bash
   cd /tmp/orchestrator-workspace/worktrees/worker-3/wasm
   # Run the failing test
   cargo test recursiveTypes
   # Or run the specific conformance test
   ```

2. **Understand the recursion:**
   - Read `wasm/src/solver/subtype.rs` - find `solve_subtype` function
   - Read `wasm/src/checker/` - find `check_expression` function
   - Look for cycle detection mechanisms (Salsa queries should handle this)
   - Identify where infinite recursion occurs

3. **Study existing cycle handling:**
   - Check if Salsa's cycle recovery is working
   - Look for `CycleStack` or similar tracking
   - Read `wasm/specs/SOLVER.md` section 1.4 on "Coinduction"

4. **Find the test case:**
   ```bash
   find /tmp/orchestrator-workspace/worktrees/worker-3/tests -name "*recursiveTypes*" -o -name "*recursive*"
   ```

---

## Implementation Plan

### Phase 2: Add Recursion Guards

**Goal:** Prevent stack overflow by limiting recursion depth and returning TS2589 error.

**Key Functions to Guard:**

1. **wasm/src/solver/subtype.rs**
   - Add `recursion_depth: u32` parameter to `solve_subtype`
   - Check depth before recursing
   - Return `TypeId::ERROR` or emit TS2589 when limit exceeded

2. **wasm/src/checker/` (expression checking)
   - Add recursion counter to expression type checking
   - Guard recursive property access chains (e.g., `obj.a.b.c.d...`)

**Pattern:**

```rust
// BEFORE (infinite recursion):
fn solve_subtype(&self, sub: TypeId, sup: TypeId) -> bool {
    // ... check cache ...
    
    // Recurse deeply
    self.solve_subtype(sub_prop, sup_prop)
}

// AFTER (guarded):
fn solve_subtype_impl(&self, sub: TypeId, sup: TypeId, depth: u32) -> bool {
    const MAX_DEPTH: u32 = 100;
    
    if depth >= MAX_DEPTH {
        // Return error instead of crashing
        return self.error_excessively_deep();
    }
    
    // ... check cache ...
    
    // Recurse with depth counter
    self.solve_subtype_impl(sub_prop, sup_prop, depth + 1)
}

// Public wrapper (no depth parameter)
fn solve_subtype(&self, sub: TypeId, sup: TypeId) -> bool {
    self.solve_subtype_impl(sub, sup, 0)
}
```

### Phase 3: Error Emission

**When limit exceeded:**
- Emit TS2589: "Type instantiation is excessively deep and possibly infinite."
- Return `TypeId::ERROR` to stop further recursion
- Log the recursion depth for debugging

---

## Success Criteria

- [ ] No stack overflow crashes on recursiveTypes test
- [ ] TS2589 error emitted when recursion depth > 100
- [ ] Conformance tests run without crashes
- [ ] No regression in non-recursive type checking
- [ ] Recursion depth limit is configurable (const)

---

## Workflow

1. **Sync with latest rust:**
   ```bash
   git fetch origin
   git rebase origin/rust
   ```

2. **Investigation Phase:**
   - Run the crashing test to confirm the issue
   - Identify exact recursion point
   - Study existing cycle handling

3. **Implementation Phase:**
   - Add recursion depth parameter to solve_subtype
   - Add depth checking with MAX_DEPTH constant
   - Implement error emission for excessive depth
   - Test with recursive types

4. **Validation:**
   - Run recursiveTypes test - should pass now
   - Run full conformance test suite
   - Verify no regressions
   - Check for TS2589 errors in appropriate places

5. **Commit and Push:**
   ```bash
   git add -A
   git commit -m "feat(solver): add recursion guards to prevent stack overflow"
   git push origin worker-3 --force
   ```

6. **STOP** - Wait for EM-1 review

---

## Deliverables

1. Recursion depth counters in solve_subtype and check_expression
2. TS2589 error emission when depth limit exceeded
3. No more stack overflow crashes
4. Conformance tests showing stability
5. Updated task list with "Complete" status

---

## Known Risks

1. **Breaking valid deep types:**
   - **Mitigation:** Set MAX_DEPTH high enough (100-200) to handle reasonable cases
   
2. **Performance impact:**
   - **Mitigation:** Depth counter is just a u32 increment - minimal overhead
   
3. **False positives:**
   - **Mitigation:** Only trigger on genuinely excessive recursion (>100 levels)

---

## Previous Tasks: ✅ COMPLETE

### Invert Solver Defaults (Stop being "Nice") ✅
**Status:** ✅ Complete
**Results:**
- Changed TypeId::ANY defaults to TypeId::UNKNOWN
- TS7006 (Implicit Any): 11 extra errors - catching previously hidden
- TS2322 (Type Mismatch): 4 extra errors - catching previously hidden
- Exact Match: 44.2% (up from ~30% baseline)

### Parser Noise Fix (TS1005 & TS1109) ✅
**Status:** ✅ Complete
**Results:** 
- TS1005: 24 extra errors (down from 439) - 95% reduction
- TS1109: 0 extra errors (down from 262) - 100% reduction
- Combined: 24 extra errors (down from 701) - 97% reduction

### Class Property Initialization (TS2564) ✅
**Status:** ✅ Complete
**Implementation:** strictPropertyInitialization check in `wasm/src/checker/declarations.rs`
**Tests:** 4 comprehensive unit tests - all passing

---

## Status

- **Current Task:** Recursion Guards (Stack Overflow Prevention)
- **Phase:** Investigation (Phase 1)
- **Last Updated:** 2026-01-15
- **Ready to Start:** ✅ YES
