# Squad Forge Goals

Updated: 2026-01-09

Priority: 1

---
## ⚠️ TMUX REMINDER - CHECK FOR HANGING PROMPTS

**NEVER forget to pause 1 second before pressing Enter in tmux!**

1. After sending any message, wait 1 second, THEN send Enter (C-m)
2. Check all worker panes for prompts that may be hanging (message sent but no activity)
3. If a prompt is hanging, send Enter again: `sleep 1 && tmux send-keys -t <pane> C-m`

**Do this check NOW and periodically throughout your session.**

---
## 📢 OPERATION CONFORMANCE - NEW DIRECTIVE

**Conformance test results show 18.1% exact match rate. This is unacceptable.**

The WASM checker has infrastructure but is MISSING critical checking passes. All workers are now reassigned to implement missing checks.

### Current State (from conformance runner)
- Total Tests: 698
- Exact Match: 126 (18.1%)
- Missing Errors: 444 tests (63.6%) - WASM doesn't catch errors TSC catches
- Extra Errors: 271 tests (38.8%) - WASM reports false positives

### Root Causes Identified
1. **Property Initialization** (TS2564) - 135 tests affected - NOT IMPLEMENTED
2. **Definite Assignment** (TS2454) - 104 tests affected - NOT IMPLEMENTED
3. **Access Modifiers** (TS2341/TS2445) - 63 tests affected - NOT ENFORCED
4. **Function Return Checking** (TS7010/TS7006) - 80 tests affected - NOT IMPLEMENTED
5. **Namespace Scoping Bug** (TS2304) - 75 false positives - BROKEN

---

## Current Milestone
**Phase 9 - Conformance Parity**: Implement missing checker passes to reach 50%+ conformance.

## Focus Areas
- `wasm/src/thin_checker.rs` - Main checker (add missing passes)
- `wasm/src/checker/` - Modular checker components
- `wasm/src/thin_binder.rs` - Fix namespace scoping bug

## Objectives (Ranked by Test Impact)

### 1. **Property Initialization Checking** (TS2564) - 135 tests
   - Error: "Property has no initializer and is not definitely assigned in constructor"
   - Implementation: Track property assignments in constructor, emit error for uninitialized
   - Key Files: `thin_checker.rs`, `checker/control_flow.rs`
   - Assigned: **Workers 1-2**

### 2. **Definite Assignment Analysis** (TS2454) - 104 tests
   - Error: "Variable is used before being assigned"
   - Implementation: Track variable initialization through control flow
   - Key Files: `thin_checker.rs`, `checker/control_flow.rs`
   - Assigned: **Worker 3**

### 3. **Access Modifier Enforcement** (TS2341/TS2445) - 63 tests
   - Errors: "Property is private/protected and only accessible within class"
   - Implementation: Check visibility on every property/method access
   - Key Files: `thin_checker.rs` (functions exist but don't emit errors)
   - Assigned: **Worker 4**

### 4. **Function Return Type Checking** (TS7010/TS7006) - 80 tests
   - Errors: "Function must return a value", "Parameter implicitly has 'any' type"
   - Implementation: Validate all code paths return, check implicit any
   - Key Files: `thin_checker.rs`, `checker/statements.rs`
   - Assigned: **Worker 5**

## Anti-Priorities
- ⛔ Redux/Lodash blocker (handled separately)
- ⛔ New solver features
- ⛔ Template literal types
- ⛔ Generic inference edge cases

## Cross-Squad Dependencies
- Anvil workers fixing namespace scoping bug (affects 75 false positives)
- Share conformance runner results: `wasm/differential-test/conformance-runner.mjs`

## Notes to EM
- Run conformance tests: `cd wasm/differential-test && node conformance-runner.mjs --max=500`
- Each worker should add tests for their error codes FIRST, then implement
- Track progress by re-running conformance tests after each PR
- Use Docker for tests: `./wasm/test.sh`

## Management Strategy
**Conformance-Driven Development**:
- PRs must include before/after conformance numbers
- Target: 50% exact match by end of sprint

## Squad Status
- Last EM Report: 2026-01-09 - Operation Conformance initiated
- Conformance Baseline: **18.1% exact match**
- Workers Active: 5/5
- Current Focus: Missing checker passes
- Worker Assignments:
  - W1: Property initialization (TS2564) - constructor tracking
  - W2: Property initialization (TS2564) - class field analysis
  - W3: Definite assignment (TS2454) - variable tracking
  - W4: Access modifiers (TS2341/TS2445) - visibility enforcement
  - W5: Function returns (TS7010/TS7006) - return validation
