# Worker 5 Plan - Forge Squad

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: **IDLE - NEEDS NEW ASSIGNMENT**
Priority: **P0 🚨 CRASH INVESTIGATION**

---

## 🚨 CRITICAL ASSIGNMENT: Crash Investigation (PRIORITY 0)

**Status:** ~478 parser panics (up from ~143) - 10% test suite broken!
**Assigned:** 2026-01-11 (Director direct assignment - EM unavailable)

### IMMEDIATE ACTION:

1. **Sync to latest rust:**
   ```bash
   cd /Users/claude/code/TypeScript-forge-5-track
   git fetch origin
   git merge origin/rust
   ```

2. **Identify crash sites:**
   ```bash
   # Find all unwrap/expect/panic in checker/solver/binder
   grep -rn "unwrap()\|expect(\|panic!" wasm/src/checker/ wasm/src/solver/ wasm/src/binder/
   ```

3. **Fix crashes systematically:**
   - Add proper `Option`/`Result` handling
   - Replace `unwrap()` with proper error propagation
   - Test against failing test cases

4. **Verify fixes:**
   ```bash
   cd wasm/differential-test
   node conformance-runner.mjs --max=1000 2>&1 | grep -i panic
   ```

### Key Files to Investigate:
- `wasm/src/thin_checker.rs` - 1,453+ potential panic sites
- `wasm/src/solver/*.rs` - type resolution failures
- `wasm/src/checker/*.rs` - checker logic panics
- `wasm/src/binder/*.rs` - binding failures

### Success Criteria:
- **Reduce panic count from ~478 to <100**
- No new crashes introduced
- All crash fixes have proper error handling

---

## Previous Assignment (COMPLETED):
**TS2304 - Cannot find name errors** - COMPLETED ✅

### Completed Work:
- [x] Added tests for try/finally fallthrough and switch fallthrough
- [x] Updated control flow analysis for try/finally + switch
- [x] Fixed nested break detection bug
- [x] All tests passing

### Results:
- Return-path analysis helpers added
- TS7010 tests passing
- Control flow fixes merged to rust

---

## Notes:
- **This is the highest priority task across both squads**
- 10% of test suite is invalid due to crashes
- Work independently - EM-Forge is currently idle
- Report progress by committing and pushing to origin/worker/forge-5
