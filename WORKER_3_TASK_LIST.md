# Worker-3 Task List

## ✅ COMPLETED: Parser Noise Fix (TS1005 & TS1109)
**Priority:** 🔴 CRITICAL (Highest Priority)
**Owner:** worker-3
**Branch:** worker-3
**Status:** ✅ COMPLETE
**Assigned:** 2026-01-14
**Completed:** 2026-01-15

---

## Task Description

**Problem:** Parser "Noise" - 701 combined extra errors (TS1005: 439, TS1109: 262)

**Root Cause:** The `ThinParser` was emitting error nodes on valid TypeScript syntax that `tsc` accepts.

**Target:** Reduce TS1005/TS1109 from ~700 to <40

---

## Results

### Final Metrics
- **TS1005**: 24 extra errors (down from 439) - **95% reduction** ✅
- **TS1109**: 0 extra errors (down from 262) - **100% reduction** ✅
- **Combined**: 24 extra errors (down from 701) - **97% reduction** ✅
- **Target**: <40 combined errors - **MET** ✅

### Changes Made
1. **Fixed await expression parsing** - Now respects async context (only parses as await expression when in async function, otherwise parses as identifier)
2. **Added AwaitKeyword/YieldKeyword to parse_primary_expression** - Allows these keywords to be parsed as identifiers in expression contexts
3. **Enhanced is_array_element_start** - Added spread operator, this/super support, and proper fallback
4. **Removed duplicate function** - Cleaned up duplicate `is_array_element_start` definition

### Remaining Edge Cases
The 24 remaining TS1005 errors are all the same edge case: `function f(await = await) {}` (non-async function with `await` as parameter name with default value). This is valid TypeScript but requires additional context-aware handling in parameter declarations.

---

## Previous Task: TS2564 ✅ COMPLETE

**Status:** ✅ Merged
**Implementation:** strictPropertyInitialization check in `wasm/src/checker/declarations.rs`
**Tests:** 4 comprehensive unit tests - all passing
**Merge Commit:** `df49a59c190` - Merge EM-1 team (Workers 1-4) into rust

---

## Status

- **Current Task:** None - Parser Noise Fix complete ✅
- **Last Updated:** 2026-01-15
