# Worker-1 Task List

## ✅ COMPLETED: Parser Noise (TS1005 & TS1109) - Phase 1
**Status:** @ COMPLETED (2026-01-15)
**Priority:** 🔴 CRITICAL
**Owner:** worker-1
**Branch:** worker-1

### Summary
Successfully implemented parser error recovery improvements to reduce TS1005 and TS1109 extra errors.

### Completed Work
**Three commits merged to em-team-1:**
1. `9596bd4f1bb` - [wasm] parser: allow reserved keywords in dotted module names
2. `72ec386a349` - [wasm] parser: fix await identifier allowed in static blocks
3. `2c8b88308b0` - [wasm] parser: comprehensive error suppression for TS1005/TS1109

### Files Modified
- `wasm/src/thin_parser.rs` - Error recovery improvements (78 insertions, 19 deletions)

### Improvements Delivered
1. **Module names with reserved keywords**: `declare namespace test.class {}` now valid
2. **Await in static blocks**: `static { let await = 1; }` now correctly parsed
3. **Error recovery suppression**: More lenient parsing at recovery boundaries to reduce false positives

### Target vs Results
- **Target:** Reduce TS1005/TS1109 from ~700 to <40
- **Implementation:** ✅ Complete
- **Status:** ⏳ Validation pending (awaiting worker-2's conformance test results)

---

## Current Status: ⏸️ AWAITING VALIDATION RESULTS

**Status:** @ PENDING (2026-01-15)
**Reason:** Worker-2 is running comprehensive conformance tests to validate all completed work

### Next Steps (After Validation Results)

**Scenario A: Target Met (<40 extra errors)**
- Mark Phase 1 as fully complete
- Assign new task (see recommendations below)

**Scenario B: Target Not Met (Still >40 extra errors)**
- Begin Phase 2: Additional parser refinements
- Focus on remaining edge cases
- Address specific error patterns identified in validation

---

## Potential Next Tasks (Awaiting Assignment)

### Option 1: Phase 2 Parser Refinement (If validation shows need)
- Target: Remaining TS1005/TS1109 edge cases
- Focus: ASI improvements, additional recovery patterns
- Estimated effort: 2-3 days

### Option 2: TS2322/TS7006 Error Accuracy (New tactical work)
- Target: Reduce type mismatch and implicit any missing errors
- Focus: Type checking improvements in `wasm/src/checker/`
- Estimated effort: 3-5 days

### Option 3: Conformance Test Infrastructure
- Target: Improve test automation and reporting
- Focus: Better diagnostics, automated regression detection
- Estimated effort: 2-3 days

---

## Notes
- Work in: /tmp/orchestrator-workspace/worktrees/worker-1
- Push to worker-1 branch when complete
- Do not touch other teams' directories
- Awaiting EM-1 guidance on next task assignment
- Last Updated: 2026-01-15

---

## Worker-1 Merge Summary (2026-01-15)

**Merge Commit:** `7c7df3b2acc` (pushed to origin/em-team-1)

### Latest Changes Merged:
**[wasm] parser: allow keywords as labels in labeled statements**
- Fixed parsing of labeled statements where label is a reserved keyword
- Example: `await: if (true) { ... }`
- Addresses TS1109 "Expression expected" errors in valid code
- Particularly affects static blocks using keyword labels

### Code Changes:
- `wasm/src/thin_parser.rs` - Extended parse_statement() to handle keywords as labels
- Check if token is identifier/keyword AND followed by colon (labeled statement)

### Overall Progress:
- **Phase 1:** ✅ Complete (4 parser improvements total)
- **Validation:** ⏳ Pending conformance test results
- **Status:** Awaiting validation to determine if Phase 2 needed
