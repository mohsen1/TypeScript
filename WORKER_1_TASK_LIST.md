# WORKER-1 TASK LIST

## Team Assignment
- **EM:** EM-1
- **Branch:** worker-1
- **Parent:** em-team-1

## Priority: Fix Parser Noise (TS1005 & TS1109)

### Mission
Eliminate false positive parser errors. ThinParser is bailing out or emitting error nodes on valid TypeScript syntax, creating "noise" that prevents trusting downstream semantic errors.

### Current Data
- **TS1005:** 439 extra errors ("';' expected")
- **TS1109:** 262 extra errors ("Expected identifier")
- **Total:** 701 extra errors
- **Target:** <40 extra errors

### Tasks

#### Task 1: Implement Error Resynchronization
**Priority:** CRITICAL
**File:** `wasm/src/parser/thin_parser.rs`

When the parser hits an unexpected token:
1. Emit the error
2. Advance to the next synchronization point (`;`, `}`, newline)
3. Continue parsing the rest of the file
4. Do NOT bail out and return a partial AST

**Acceptance Criteria:**
- Parser continues after syntax errors instead of bailing out
- Conformance tests show reduced TS1005/TS1109 counts
- No increase in crashes or panics

#### Task 2: Audit Semicolon Insertion (ASI)
**Priority:** CRITICAL
**File:** `wasm/src/parser/`

Verify ASI logic matches TypeScript's exactly:
1. Review `tsc` ASI rules in TypeScript source
2. Compare with our implementation
3. Fix discrepancies
4. Add test cases for edge cases

**Common ASI edge cases:**
- Newline after `return`, `throw`, `break`, `continue`
- Newline after `++`, `--`
- Anonymous function expressions
- Do-while loops

**Acceptance Criteria:**
- ASI logic documented with TypeScript references
- Test cases added for all edge cases
- TS1005 errors reduced significantly

### Deliverables
1. Updated `thin_parser.rs` with resynchronization
2. ASI audit document with fixes applied
3. Conformance test results showing error reduction

### Success Metric
Reduce combined TS1005/TS1109 errors from **701 to <40**.

## Merge Status

### 2026-01-14 - Latest Merge Attempt ⚠️
**Status:** ⚠️ BRANCH OUT OF DATE - NO MERGE PERFORMED
**Result:** Worker-1 is behind em-team-1
**Action:** Merge blocked - worker-1 has removed 4,766 lines of work

### Issue Detected
Worker-1 branch has diverged negatively from em-team-1:
- **Removed:** ASI conformance tests (222 lines)
- **Removed:** TS2564 property initialization tests (350 lines)
- **Removed:** All test infrastructure (metrics, conformance, error analysis)
- **Removed:** Documentation (TEAM_STRUCTURE.md, MERGE_READINESS_REPORT.md, various audit reports)
- **Removed:** Test files from worker-2 (lib loading, TS2304, global augmentation)
- **Removed:** Code from thin_binder.rs, thin_checker.rs, solver files

**Total:** 30 files affected, 4,766 lines deleted

### Git Behavior
- Merge command returned "Already up to date" (correctly blocked bad merge)
- em-team-1 is ahead of worker-1
- No merge was performed

### Root Cause
Worker-1 branch appears to be on an old commit that predates:
- Worker-4 TS2564 implementation (commit 96ca9f6a5)
- Worker-2 global scope work (commit 2419999cd)
- Worker-10 ASI work (multiple commits)
- Team infrastructure and documentation

### Tasks Completed
- Parser work assigned to worker-1 was completed by worker-10
- Worker-1 branch needs rebase to rust to get latest changes

### Next Steps for Worker-1
- ⚠️ **URGENT:** Rebase worker-1 to rust to sync with latest work
- OR: Delete worker-1 branch and recreate from em-team-1
- Worker-1 should NOT be merged in current state
- Awaiting Director/EM-1 decision on branch remediation

---

### Notes
- Reference TypeScript parser at `src/compiler/parser.ts`
- Run conformance tests after each change
- Work incrementally; test frequently
- Coordinate with EM-1 before merging
