# Squad Anvil Goals

Updated: 2026-01-09

Priority: 2

---
## 📢 EM-ANVIL: DIRECTIVE UPDATE

**Operation Crucible - NEW APPROACH:**

1. **Workers 1-2**: Bug fixes ONLY (`super["m"]` in async, nested arrow `this` capture)
2. **Workers 3-5**: Build CONFORMANCE HARNESS (not manual parity tests)
3. **⛔ STOP writing manual parity tests** - low leverage approach

### Why Conformance Harness > Manual Tests

The `tests/cases/compiler/` directory has **6,500+ test files** with expected baselines.
We have **124+ hand-written parity tests** that duplicate this effort.

**Better approach:**
```rust
#[test]
fn test_conformance_2dArrays() {
    run_conformance_test("tests/cases/compiler/2dArrays.ts");
}
```

Where `run_conformance_test` reads the file, compiles it, and diffs against `tests/baselines/reference/`.

---

## ⚠️ OPERATION CRUCIBLE - CONFORMANCE HARNESS

**Build automated conformance testing, not manual parity tests.**

- **Squad reduced to 2 workers** (workers 1-2) on bug fixes
- **Workers 3-5**: Build conformance harness infrastructure
- **⛔ NO MORE MANUAL PARITY TESTS** - invest in automation

## Current Milestone
Phase 8 - Conformance, Convergence, and Hardening: Output fidelity across the integrated pipeline, driven by conformance tests.

## Project Direction Alignment
- Emitter is 80% complete - stop adding new features
- Bug fixes only: `super["m"]` in async, nested arrow `this` capture
- ⚠️ Anti-pattern: Do NOT use regex substitutions for code transforms. Always operate on AST.

## Focus Areas (MAINTENANCE ONLY)
- `wasm/src/thin_emitter/` - Critical bug fixes only
- `wasm/src/transforms/` - Blocking ES5 regressions only

## Objectives (Ranked)

1. **Build Conformance Harness** (Workers 3-5)
   - Read test files from `tests/cases/compiler/*.ts`
   - Compile through Rust pipeline
   - Compare output against `tests/baselines/reference/*.js`
   - Track pass rate (currently ~40/76 for JS)
   - Key Files: New `wasm/src/conformance_harness.rs`

2. **Critical Bug Fixes Only** (Workers 1-2)
   - Fix blocking ES5 regressions (`super["m"]` in async, nested arrow `this` capture)
   - Key Files: `transforms/class_es5.rs`, `transforms/async_es5.rs`
   - ⛔ NO NEW FEATURES

3. **Source Map Bug Fixes**
   - Only fix bugs that block debugger attachment
   - Key Files: `thin_emitter/source_writer.rs`, `thin_emitter/source_map.rs`

## Anti-Priorities (ENFORCED)
- ⛔ **Manual parity tests** - use conformance harness instead
- ⛔ **New Emitter transforms** - we have enough
- ⛔ **Hand-written expected output** - compare against existing baselines
- New LSP features
- CLI argument parsing

## Cross-Squad Dependencies
- Forge squad owns type checking; coordinate on shared conformance regressions

## Notes to EM
- **Only 2 workers active** (workers 1-2)
- Workers 3-5 are now on Crucible Tasks (test porting) - see their plans
- Bug fixes only - reject any PR that adds new transforms
- Read `wasm/specs/WASM_ARCHITECTURE.md` for architecture
- Use Docker for tests: `./wasm/test.sh`

## Management Strategy
Per Project Direction: **Autocratic Scheduling + Bisect-on-Merge**
- PRs that regress ANY existing baseline are auto-rejected
- Zero-Idle: If a high-priority task is blocked, swarm it

## Squad Status
- Last EM Report: 2026-01-09 - Conformance harness approach adopted
- Workers Active: 5/5
- Current Focus: Conformance harness + bug fixes
- Direction: Automation over manual tests
- Worker Assignments:
  - W1: Bug fixes - `super["m"]` in async (`transforms/async_es5.rs`)
  - W2: Bug fixes - nested arrow `this` capture (`transforms/class_es5.rs`)
  - W3: **HARNESS** - Build `run_conformance_test()` function
  - W4: **HARNESS** - Read `tests/cases/compiler/*.ts`, parse test directives
  - W5: **HARNESS** - Diff against `tests/baselines/reference/*.js`, report pass rate
