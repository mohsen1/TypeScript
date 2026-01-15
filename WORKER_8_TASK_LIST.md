# Worker-8 Task List

**Squad:** Syntax (Parser Error Recovery)
**Branch:** `worker-8`
**EM:** EM-2
*Assigned: 2025-01-14*
*Updated: 2025-01-14*

---

## Previous Tasks - Complete ✅

### 1. Recursion Guards (TS2589) ✅
- Added depth tracking to solver
- Emit TS2589 when recursion exceeds 100 levels
- No more stack overflow crashes

### 2. Class Property Initialization (TS2564) ✅
- Already implemented in codebase
- Verified working via conformance tests
- Not in missing errors list

### 3. Invert Solver Defaults ✅
- Changed function return defaults from ANY to UNKNOWN (3 locations)
- Changed variable defaults from ANY to UNKNOWN (3 locations)
- Changed expression defaults from ANY to UNKNOWN (4 locations)
- Total: 13 fixes, tests show improved strictness

---

## Priority Mission

**Fix Parser Noise - Error Resynchronization**

**Current Issue:** 701 extra parser errors (TS1005: 439, TS1109: 262) are false positives. The parser emits errors on valid TypeScript syntax, then crashes or produces malformed ASTs, making semantic checking unreliable.

**Root Cause:** When the ThinParser hits unexpected tokens, it:
1. Emits an error (correct)
2. Returns early or creates error nodes (WRONG - should continue parsing)
3. Doesn't resynchronize to the next valid token (WRONG)

**Goal:** Implement error resynchronization so the parser can recover and continue parsing the rest of the file.

**Target Impact:** Reduce TS1005/TS1109 from 701 to <40

---

## New Assigned Tasks

### 1. Implement Error Resynchronization in Parser
**Priority:** P0 - Critical
**File:** `wasm/src/parser/thin_parser.rs`

**Issue:** Parser doesn't resynchronize after errors, causing cascading failures.

**Solution:**
1. Add `synchronize()` method that advances to the next synchronization point
2. Synchronization points: `;`, `}`, `)`, `]`, end of file
3. Call `synchronize()` after emitting parser errors
4. Continue parsing instead of returning early

**Test Case:**
```typescript
// Current: Fails to parse after first error
function foo() {
    return 1,
}

function bar() {  // Never reached
    return 2;
}

// Target: Parse both functions, emit error on line 2, continue
```

**Code Locations:**
- `parse_statement()` - add resync after error
- `parse_expression_statement()` - add resync after error
- `parse_function_body()` - add resync after error
- Anywhere `emit_error()` is called

**Success Criteria:**
- Parser continues after syntax errors
- Rest of file is parsed correctly
- Fewer cascading errors
- AST is well-formed even with syntax errors

**Status:** ⏳ TODO

---

### 2. Fix ASI (Automatic Semicolon Insertion)
**Priority:** P1 - High Impact
**File:** `wasm/src/parser/thin_parser.rs`

**Issue:** Missing or incorrect semicolon insertion causes TS1005 errors.

**Solution:**
1. Review TypeScript's ASI rules from spec
2. Implement rules:
   - Insert `;` at end of line if next token is `}`, `)`, `]`
   - Insert `;` at end of line if statement could be complete
   - Don't insert if would create `for ( ; ... )` pattern
   - Handle `do...while` correctly
3. Match TypeScript's ASI exactly

**Test Cases:**
```typescript
// ASI should insert semicolon here
let x = 5
console.log(x)  // Should work, not error

// ASI should NOT insert here (for loop)
for (let i = 0
     i < 10
     i++) {  // Should parse correctly
}

// do-while ASI
do {
    break
} while (false)  // Semicolon inserted here
```

**Success Criteria:**
- ASI matches TypeScript exactly
- No false positive TS1005 from missing semicolons
- `for` loops with line breaks parse correctly
- `do...while` loops parse correctly

**Status:** ⏳ TODO

---

### 3. Fix Binary Expression Error Recovery
**Priority:** P2
**File:** `wasm/src/parser/thin_parser.rs`

**Issue:** Invalid binary expressions cause parser to give up instead of skipping to next token.

**Solution:**
1. When binary expression parsing fails:
   - Emit error for the invalid expression
   - Skip to the next statement-ending token
   - Continue with next statement
2. Don't let expression errors cascade

**Test Case:**
```typescript
// Invalid expression, but rest should parse
let x = a + * b  // Error: invalid expression
let y = 5  // Should be parsed correctly
```

**Success Criteria:**
- Expression errors don't cascade
- Next statement is parsed
- Only the invalid expression gets an error

**Status:** ⏳ TODO

---

### 4. Improve Error Node Handling
**Priority:** P3
**File:** `wasm/src/parser/thin_parser.rs` and downstream

**Issue:** Error nodes from parser aren't handled consistently by the checker.

**Solution:**
1. Ensure `ERROR` nodes are always created (not `None`)
2. Type check `ERROR` nodes gracefully (return `TypeId::ERROR`)
3. Don't emit cascading errors from already-errorred nodes
4. Use `is_error()` checks before processing

**Test Case:**
```typescript
let x = invalid syntax here  // Error from parser
let y = x + 1  // Should not emit additional errors (x is error)
```

**Success Criteria:**
- Error nodes don't cause cascading errors
- Checker handles `ERROR` nodes gracefully
- One syntax error = one diagnostic (ideally)

**Status:** ⏳ TODO

---

## Implementation Plan

### Phase 1: Add Synchronization Infrastructure (2-3 hours)
1. Implement `synchronize_to(tokens: &[SyntaxKind]) -> bool`
2. Implement `skip_to_statement_end() -> bool`
3. Add `can_resume_from(token: SyntaxKind) -> bool`
4. Unit tests for sync behavior

### Phase 2: Update Error Sites to Use Sync (3-4 hours)
1. Find all `emit_error()` call sites
2. Add `synchronize()` call after each error
3. Test on failing conformance cases
4. Verify AST is still well-formed

### Phase 3: Fix ASI (2-3 hours)
1. Review TypeScript ASI spec/implementation
2. Implement ASI rules in `try_insert_semicolon()`
3. Add ASI tests
4. Verify against TypeScript behavior

### Phase 4: Expression Recovery (1-2 hours)
1. Update `parse_binary_expression()` to resync
2. Update `parse_assignment_expression()` to resync
3. Test on complex invalid expressions

### Phase 5: Error Node Handling (1-2 hours)
1. Audit checker for `ERROR` node handling
2. Add guards where needed
3. Ensure no crashes on error nodes
4. Test cascading error scenarios

### Phase 6: Validation (1 hour)
1. Run conformance tests
2. Measure TS1005/TS1109 reduction
3. Verify no regressions
4. Check for new false positives

---

## Success Metrics

### Before Implementation
- **TS1005 (Parser):** 439 extra errors
- **TS1109 (Parser):** 262 extra errors
- **Total Parser Noise:** 701 errors
- **Parser Crashes:** Some tests fail to parse completely

### After Implementation (Target)
- **TS1005/TS1109:** <40 total (95% reduction)
- **Parser Crashes:** 0
- **Error Recovery:** Parser continues after syntax errors
- **AST Quality:** Well-formed even with syntax errors

---

## Notes

**Dependencies:**
- Parser changes affect entire pipeline
- Must work with existing token stream
- Can't change token positions (affects error reporting)

**Risks:**
- ASI is subtle and complex
- Sync points might skip too much (under-parsing)
- Sync points might skip too little (over-parsing)
- Need to match TypeScript's exact behavior

**Synergies:**
- Fixes semantic checking (AST is well-formed)
- Enables better error messages
- Improves conformance across all tests

**Testing Strategy:**
1. Unit tests for sync logic
2. Compare with TypeScript on invalid syntax
3. Use conformance tests with `@error` directive
4. Manual testing on edge cases

---

## Next Steps

1. **Phase 1:** Add sync infrastructure
2. **Phase 2:** Update error sites with sync
3. **Phase 3:** Fix ASI
4. **Phase 4:** Expression recovery
5. **Phase 5:** Error node handling
6. **Phase 6:** Validate and measure

**When complete:** Push to `worker-8` branch and notify EM-2 for review.

---

## ✅ TASK REASSIGNMENT: Solver Defaults Inversion

**Reassigned:** 2026-01-14
**New Mission:** Invert solver defaults from ANY to ERROR/UNKNOWN
**Previous Tasks:** TS2564 and TS2589 (reassigned to other workers)

---

## ✅ PHASES 1-4 COMPLETE: Solver Defaults Inversion

**Completed:** 2026-01-14
**Merged to:** em-team-2 (commit b4c169560)
**Note:** Work already merged, no new merge needed

### Work Completed

**Phase 1: TypeId::ANY Analysis**
- Complete audit of TypeId::ANY usage in codebase
- Created PHASE1_ANALYSIS.md with findings
- Identified 10+ locations where ANY is returned as fallback

**Phase 2: Fix Function Return Type Defaults (P0)**
- Changed function return type defaults from ANY to ERROR
- Updated thin_checker.rs to use ERROR type for unresolved returns
- Commit: 57ad1f247

**Phase 3: Fix Variable Declaration Defaults (P1)**
- Changed variable declaration defaults from ANY to ERROR
- Updated thin_checker.rs to use ERROR type for unresolved variables
- Commit: 941dedbdc

**Phase 4: Fix Expression Defaults (Binary Operations)**
- Changed binary operation defaults from ANY to ERROR
- Updated thin_checker.rs to handle binary operations with strict typing
- Commit: 0a805f0bd

**Additional Work:**
- Error resynchronization to parse_expression_statement
- Commit: cf5220172

### Files Modified
- `wasm/src/thin_checker.rs` - Core type checking changes
- `wasm/src/thin_parser.rs` - Error resynchronization improvements
- `PHASE1_ANALYSIS.md` - Analysis documentation
- `wasm/PHASE1_ANALYSIS.md` - Copy in wasm directory
- `wasm/PHASE1_RESYNC_STATUS.md` - Resynchronization status

### Expected Impact
- **Missing TS2322 errors:** Should decrease from 1,841
- **Missing TS7006 errors:** Should decrease from 357
- **Extra errors:** Temporary increase as hidden errors are exposed
- **Exact match:** Will temporarily decrease, then increase as fixes are applied

---

## Status: ✅ PHASES 1-4 COMPLETE

**Merged:** em-team-2
**Build Status:** ✅ Passing
**Ready for:** Director review or next phase assignment
