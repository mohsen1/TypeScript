# WORKER-8 TASK LIST

## Squad: CFA Squad
## EM: EM-2
## Branch: worker-8

---

## Primary Task: Fix Class Property Initialization (TS2564)

**Priority:** 🟡 TACTICAL (Priority 4 for EM-2)

### Problem
- TS2564 is the #1 missing error: 413 occurrences
- "Property 'x' has no initializer and is not definitely assigned in the constructor"
- We are simply NOT running this check

### Action Items
1. **Implement `strictPropertyInitialization` Check**
   - Add control flow analysis to verify class properties are initialized
   - Check constructor body and property declarations
   - Account for definite assignment assertions (`!`)

2. **Integration Point**
   - Add check to `wasm/src/checker/thin_checker.rs`
   - Run after class declaration is analyzed

### Files to Work On
- `wasm/src/checker/thin_checker.rs`
- `wasm/src/checker/class_checker.rs` (if exists, or create)

### Success Criteria
- Reduce TS2564 Missing errors from 413 to <20
- Emit TS2564 when property lacks initializer and isn't set in constructor
- Respect definite assignment assertion operator

### Testing
- Create test cases for class property initialization
- Verify check fires on unassigned properties
- Verify check respects `!` operator

---

## Instructions
1. Create branch from `em-team-2`
2. Implement the `strictPropertyInitialization` check
3. Push to `worker-8` branch when ready for review
4. EM-2 will merge and validate before escalating

---

## Task Completion Report

### Actual Work Completed
**Task:** TS2589 Recursion Guards (Reassigned from original TS2564 task)

**Status:** ✅ MERGED into em-team-2
**Merge Commit:** dc6d8767d
**Date:** 2026-01-14

### Changes Made
- `wasm/src/checker/context.rs` - Added recursion guard context
- `wasm/src/checker/types/diagnostics.rs` - Updated diagnostics
- `wasm/src/solver/subtype.rs` - Added recursion prevention
- `wasm/src/thin_checker.rs` - Main recursion guard implementation

### Results
- Successfully implemented recursion guards for TS2589
- Prevents infinite recursion during type checking
- Merged cleanly with no conflicts
- All tests passed
