# WORKER-13 Task List

**EM:** EM-4
**Focus:** Async/Await (Tier 5) - Fix TS2705 and TS1359 missing errors

## Mission

Fix async function return type checking (TS2705) and await reserved word detection (TS1359).

## Current Baseline

From conformance tests (200 files):
- **TS2705 Missing:** 34 occurrences - Most critical missing error
- **TS1359 Missing:** 7 occurrences - Await reserved word not detected
- **TS2705 Extra:** 4 occurrences - Some false positives exist

### What These Errors Mean

**TS2705:** "An async function or method in ES5/ES3 requires the 'Promise' constructor. Make sure you have a declaration for the 'Promise' constructor or include 'ES2015' in your `--lib` option."

This error should be emitted when:
- An async function is declared
- The target is ES5/ES3
- Promise is not available in lib

**TS1359:** "An 'await' expression is only allowed within an async function."

This error should be emitted when:
- `await` is used in a non-async context

## Key Files

| File | Purpose |
|------|---------|
| `wasm/src/thin_checker.rs` | Main type checker (~22k lines) |
| `wasm/src/checker/types/diagnostics.rs` | Error code definitions |
| `wasm/src/binder/` | Symbol table for async context tracking |

## Steps

### 1. Verify Error Codes Exist

First check if TS2705 and TS1359 are defined:
```bash
grep -n "TS2705\|TS1359" wasm/src/checker/types/diagnostics.rs
```

If not defined, add them:
```rust
pub const TS2705: u32 = 2705;
pub const TS2705_MESSAGE: &str = "An async function or method in ES5/ES3 requires the 'Promise' constructor...";

pub const TS1359: u32 = 1359;
pub const TS1359_MESSAGE: &str = "An 'await' expression is only allowed within an async function.";
```

### 2. Find the Check Logic

Search for async-related checking in `thin_checker.rs`:
```bash
grep -n "async.*function\|await" wasm/src/thin_checker.rs | head -50
```

Key areas:
- Around line 6000-7000 for async function checking
- Around line 15000+ for await expression checking

### 3. Reproduce Issues

Create test files:
```typescript
// /tmp/test2705.ts
// Target: ES5, should emit TS2705
async function foo() {}
```

```typescript
// /tmp/test1359.ts
function notAsync() {
    await Promise.resolve();  // Should emit TS1359
}
```

### 4. Implement Missing Checks

**For TS2705:**
1. Check if function is async
2. Check compiler target (ES5/ES3)
3. Check if Promise is available
4. Emit error if conditions met

**For TS1359:**
1. Track async context when entering/exiting async functions
2. When encountering `await`, verify async context is active
3. Emit error if not in async function

### 5. Validate

```bash
# Build
cd wasm && wasm-pack build --target web --out-dir pkg

# Test async category specifically
cd differential-test
bash run-conformance.sh --max=200 --workers=4 | grep -A5 "By Category"
```

Look for improvement in the `async` row.

## Success Criteria

- **TS2705 Missing:** Reduced from 34 to ≤10
- **TS1359 Missing:** Reduced from 7 to ≤2
- **TS2705 Extra:** Not increased (currently 4)

## Resources

- Type checker: `wasm/src/thin_checker.rs`
- Async handling: Look for `check_async_function`, `check_await_expression`
- Diagnostics: `wasm/src/checker/types/diagnostics.rs`

## Progress Log

### 2026-01-15 - Task Completed ✅
- **Merge Commit:** Worker-13 branch merged into em-team-4
- **Changes:**
  - `wasm/src/checker/context.rs`: Added context support (34 lines)
  - `wasm/src/checker/types/diagnostics.rs`: Added 6 diagnostic codes
  - `wasm/src/thin_checker.rs`: Enhanced type checking (67 lines)
- **Validation Results:** 44.4% exact match (20/45 tests)
- **Status:** Merged to em-team-4, ready for EM-4 escalation
- **Note:** Rebased on rust branch before final merge

### Next Steps
- Awaiting new assignment from EM-4
- Current async/await improvements are stable

## Submit Your Work

1. Create branch: `git checkout -b worker-13-async-fixes`
2. Commit with clear messages
3. Run validation: `bash run-conformance.sh --max=500 --workers=8`
4. Notify EM-4 for review
