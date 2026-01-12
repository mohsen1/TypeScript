# Worker 4 Plan - Forge Squad

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: **IDLE - NEW ASSIGNMENT**
Priority: **PRIORITY 1**
Assigned: 2026-01-11 (Director direct assignment - EM unavailable)

---

## 🔥 ASSIGNMENT: TS2792 - Module Resolution (204 tests)

**Error:** "Cannot find module 'x' or its corresponding type declarations"

### IMMEDIATE ACTION:

1. **Sync to latest rust:**
   ```bash
   git fetch origin
   git merge origin/rust
   ```

2. **Understand the requirement:**
   - Track which imports couldn't be resolved
   - Emit proper error code (2792 vs 2307)
   - 2792 for package imports that lack type declarations
   - 2307 for modules that can't be found at all
   - Handle relative vs package imports

3. **Implementation approach:**
   - Track unresolved imports in `thin_binder.rs`
   - Distinguish between "module not found" (2307) and "module found but no types" (2792)
   - Emit proper diagnostic codes

4. **Test conformance:**
   ```bash
   cd wasm/differential-test
   node conformance-runner.mjs --max=500 | grep -E "TS2792|TS2307"
   ```

### Key Files:
- `wasm/src/thin_binder.rs` - import resolution
- `wasm/src/thin_checker.rs` - module diagnostics
- `wasm/src/thin_checker_tests.rs` - add regression tests

### Success Criteria:
- Emit TS2792 for package modules without type declarations (currently missing)
- Emit TS2307 for modules that can't be found
- Correctly distinguish relative vs package imports
- Add regression tests for module resolution edge cases

---

## Previous Assignment (COMPLETED):
**TS2339 - Property does not exist** - COMPLETED ✅

### Completed Work:
- [x] Fixed optional chaining - `?.` correctly suppresses TS2339
- [x] Added 7 comprehensive TS2339 tests
- [x] All 20 TS2339 tests passing
- [x] is_private_field() helper added

---

## Notes:
- This is a PRIORITY 1 task affecting conformance
- Work independently - EM-Forge is currently idle
- Commit progress frequently and push to origin/worker/forge-4
- Reference: `wasm/specs/squads/forge/GOALS.md`
