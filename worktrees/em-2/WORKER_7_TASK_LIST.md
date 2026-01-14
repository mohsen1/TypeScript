# WORKER-7 TASK LIST

## Squad: Semantics Squad
## EM: EM-2
## Branch: worker-7

---

## Primary Task: Invert Solver Defaults (Stop being "Nice")

**Priority:** 🟠 STRATEGIC (Priority 3 for EM-2)

### Problem
- 2961 missing errors (60% of our total gap)
- When Solver can't resolve a symbol, it returns `TypeId::ANY`
- This hides errors—TypeScript would error, we say "it's any, so it's fine"
- We're missing 184 TS2322 (Type Mismatch) and 357 TS7006 (Implicit Any) errors

### Action Items
1. **Change Default Return Type**
   - Modify `wasm/src/solver/` to return `TypeId::UNKNOWN` or `TypeId::ERROR` instead of `TypeId::ANY`
   - Apply this when symbol resolution fails or type operations fail

2. **Expect Regression**
   - This WILL cause a spike in "Extra Errors"—**this is good**
   - It exposes where our logic is failing instead of hiding it

### Files to Work On
- `wasm/src/solver/mod.rs`
- `wasm/src/solver/type_resolution.rs`
- Any function returning `TypeId::ANY` as a default/fallback

### Success Criteria
- Stop hiding errors behind optimistic `Any` defaults
- Short-term: More errors (expected)
- Long-term: Accurate error reporting leads to proper fixes

### Testing
- Run conformance suite
- Expect increased error count—verify errors are legitimate, not noise

---

## Instructions
1. Create branch from `em-team-2`
2. Change defaults to ERROR/UNKNOWN
3. Document the regression spike (it's intentional)
4. Push to `worker-7` branch when ready for review
5. EM-2 will merge and validate before escalating

---

## Task Completion Report

### Actual Work Completed
**Task:** Invert Solver Defaults - Return ERROR for unresolved references

**Status:** ✅ MERGED into em-team-2
**Merge Commit:** aeda8a6d6 (first merge), HEAD (documentation)
**Date:** 2026-01-14

### Changes Made
- **wasm/src/solver/evaluate.rs**: Modified to return ERROR type_id for unresolved references
- **DIRECTOR_SUMMARY.md**: Comprehensive summary of all Worker 7 completed work
- **CONFORMANCE_TEST_STATUS.md**: Current conformance test status report
- **MERGE_READINESS_REPORT.md**: Analysis of all worker branch readiness

### Results
- Successfully inverted solver defaults to return ERROR instead of type_id
- This exposes hidden errors instead of masking them with ANY
- Expected short-term: Increased error count (regression is intentional)
- Long-term: Accurate error reporting leads to proper fixes
- See DIRECTOR_SUMMARY.md for comprehensive results

### Documentation
- All work documented and ready for director review
