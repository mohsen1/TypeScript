# Worker-11 Task List

**Squad:** Infrastructure
**Branch:** `worker-11`
**EM:** EM-3
*Assigned: 2026-01-14*

---

## Priority Mission

Improve test runner and debug capabilities. **Target: Better visibility into symbol resolution failures.**

---

## Assigned Tasks

### 1. Improve Test Runner Lib Injection
**Priority:** P0 - High
**Files:** Test runner scripts, `wasm/src/binder/`

**Tasks:**
1. Verify `lib.d.ts` loads correctly for all test configurations
2. Add validation that global symbols are present after binding
3. Log missing lib symbols at test start
4. Ensure lib symbols merge across multiple test files

### 2. Add Debug Logging for Symbol Resolution
**Priority:** P1
**Files:** `wasm/src/binder/`, `wasm/src/solver/`

**Tasks:**
1. Add optional debug flag for symbol lookup traces
2. Log when symbol falls through to `file_locals`
3. Log when symbol falls through to `lib_binders`
4. Track and report symbol resolution attempts vs successes

### 3. Categorize Conformance Tests
**Priority:** P2
**Files:** Test infrastructure

**Tasks:**
1. Parse test file paths to categorize by component
2. Create categories: parser, binder, solver, checker, integration
3. Generate report showing error distribution by category
4. Help identify which component needs most attention

---

## Success Criteria
- [ ] Lib symbols load reliably in all test scenarios
- [ ] Debug logging available for symbol resolution
- [ ] Conformance tests categorized by component
- [ ] Error distribution report generated

---

## Status
**Status:** 🔄 IN PROGRESS
**Assigned:** 2026-01-14
