# Worker-3 Task List

## 🔴 CURRENT TASK: Parser Noise Fix (TS1005 & TS1109)
**Priority:** 🔴 CRITICAL (Highest Priority)
**Owner:** worker-3
**Branch:** worker-3
**Status:** 🟡 IN PROGRESS
**Assigned:** 2026-01-14

---

## Task Description

**Problem:** Parser "Noise" - 701 combined extra errors (TS1005: 439, TS1109: 262)

**Root Cause:** The `ThinParser` is bailing out or emitting error nodes on valid TypeScript syntax that `tsc` accepts. This "noise" makes it impossible to trust downstream semantic errors because a broken AST results in broken symbols.

**Target:** Reduce TS1005/TS1109 from ~700 to <40

---

## Analysis Required

### Phase 1: Investigation (DO THIS FIRST)

**Before making any changes:**

1. **Understand the errors:**
   - TS1005: "',' expected" - Missing comma, semicolon, or other syntax element
   - TS1109: "Expression expected" - Parser expecting expression but found something else

2. **Find test cases:**
   ```bash
   # Find tests with TS1005 errors
   grep -r "TS1005" /tmp/orchestrator-workspace/worktrees/worker-3/src/tests/
   # Find tests with TS1109 errors
   grep -r "TS1109" /tmp/orchestrator-workspace/worktrees/worker-3/src/tests/
   ```

3. **Run differential tests to see the noise:**
   ```bash
   cd /tmp/orchestrator-workspace/worktrees/worker-3/wasm
   ./differential-test/run-conformance.sh --max=100
   ```

4. **Study TypeScript's parser implementation:**
   - Read `src/compiler/parser.ts` - How does tsc handle error recovery?
   - Look for "error recovery" or "resync" in the parser
   - Study the `parseErrorAtNextSemicolon` function

5. **Study ThinParser error handling:**
   - Read `wasm/src/parser/thin_parser.rs`
   - Find the `resync_after_error` method (should already exist)
   - Understand when it's called vs. when it should be called

---

## Implementation Plan

### Phase 2: Error Resynchronization

**Goal:** When the parser hits an unexpected token, it must advance to the next synchronization point and continue parsing.

**Key Methods to Implement/Improve:**

1. **`resync_after_error` (may already exist):**
   - Already implemented in `thin_parser.rs` around line 653
   - Verify it's being called correctly
   - Check if synchronization points are correct

2. **Identify Synchronization Points:**
   - Semicolons (`;`) - Statement boundaries
   - Closing braces (`}`) - Block boundaries
   - Opening braces (`{`) - Object literals
   - Keywords - `function`, `class`, `interface`, `if`, `for`, etc.

3. **Improve Error Recovery in Key Areas:**
   - **Object literal parsing** - Very common source of TS1005
   - **Array literal parsing** - Missing commas, trailing commas
   - **Function declarations** - Parameter lists, return types
   - **Class declarations** - Property declarations, methods
   - **Statement parsing** - Expression statements, control flow

### Phase 3: ASI (Automatic Semicolon Insertion) Audit

**Goal:** Verify our ASI logic matches TypeScript's exactly.

**Key Areas:**

1. **Review ASI rules in TypeScript spec:**
   - Section 11.9.1 - Rules of Automatic Semicolon Insertion
   - Line terminator vs. semicolon
   - Restricted productions

2. **Audit ThinParser ASI implementation:**
   - Search for "asi" or "semicolon insertion" in `thin_parser.rs`
   - Compare with TypeScript's `parser.ts` implementation
   - Check edge cases:
     - `return\nx` - should insert semicolon after return
     - `throw\nx` - should insert semicolon after throw
     - `continue\nlabel` - should NOT insert semicolon
     - `break\nlabel` - should NOT insert semicolon

3. **Test ASI edge cases:**
   ```typescript
   return // ASI should insert semicolon
   {
     x: 1
   }

   a = b + c // ASI should insert semicolon
   (d + e).print()
   ```

---

## Success Criteria

- [ ] TS1005 errors reduced from 439 to <20
- [ ] TS1109 errors reduced from 262 to <20
- [ ] Combined total <40 (down from 701)
- [ ] No regression in valid code parsing
- [ ] Conformance tests show improvement in exact match
- [ ] Unit tests added for error recovery scenarios

---

## Workflow

1. **Sync with latest rust:**
   ```bash
   git fetch origin
   git rebase origin/rust
   ```

2. **Investigation Phase:**
   - Run baseline conformance tests
   - Study TypeScript's parser implementation
   - Identify specific failing test cases
   - Document findings in task list

3. **Implementation Phase:**
   - Fix error resynchronization
   - Fix ASI logic
   - Add tests for edge cases
   - Run conformance tests after each major change

4. **Validation:**
   - Run full conformance test suite
   - Verify TS1005/TS1109 counts reduced
   - Check for regressions

5. **Commit and Push:**
   ```bash
   git add -A
   git commit -m "feat(parser): fix error recovery for TS1005/TS1109"
   git push origin worker-3 --force
   ```

6. **STOP** - Wait for EM-1 review

---

## Deliverables

1. Error recovery improvements in `wasm/src/parser/thin_parser.rs`
2. ASI fixes in `wasm/src/parser/` (if needed)
3. Unit tests for error recovery scenarios
4. Conformance test report showing improvement
5. Updated task list with "Complete" status

---

## Known Issues to Investigate

1. **Object literal parsing:**
   ```typescript
   const obj = {
     a: 1
     b: 2  // Missing comma - does TS1005 get reported?
   }
   ```

2. **Array literal parsing:**
   ```typescript
   const arr = [
     1,
     2,
     3  // Trailing comma - is this handled?
   ]
   ```

3. **Function parameters:**
   ```typescript
   function foo(
     x: number
     y: string  // Missing comma - does this break parsing?
   ) {}
   ```

4. **Statement boundaries:**
   ```typescript
   const x = 1
   const y = 2  // Missing semicolon - can we recover?
   ```

---

## Previous Task: TS2564 ✅ COMPLETE

**Status:** ✅ Merged
**Implementation:** strictPropertyInitialization check in `wasm/src/checker/declarations.rs`
**Tests:** 4 comprehensive unit tests - all passing
**Merge Commit:** `df49a59c190` - Merge EM-1 team (Workers 1-4) into rust

---

## Status

- **Current Task:** Parser Noise Fix (TS1005 & TS1109)
- **Phase:** Investigation (Phase 1)
- **Last Updated:** 2026-01-14
- **Ready to Start:** ✅ YES
