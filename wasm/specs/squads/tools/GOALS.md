# Squad Tools Goals

Updated: 2026-01-08

## Current Milestone
Emitter Fidelity - Make JavaScript output semantically identical to TypeScript.

## Objectives (Ranked)

1. **ES5 Downleveling Correctness**
   - Context: ES5 class transforms and async/await lowering must produce semantically identical JavaScript to `tsc`. Current known issues with computed `super["m"]` in async contexts.
   - Success Criteria: All ES5 transform tests pass; `super()` calls in derived classes with property initializers work correctly; `this` capture in nested arrow/async functions is correct.
   - Key Files: `wasm/src/transforms/class_es5.rs`, `wasm/src/transforms/async_es5.rs`
   - Estimated Complexity: High

2. **Source Map Validation**
   - Context: Source maps must be valid and usable by debuggers. Current coverage is incomplete.
   - Success Criteria: Generated source maps pass validation; debuggers can attach breakpoints correctly.
   - Key Files: `wasm/src/thin_emitter/source_writer.rs`, `wasm/src/thin_emitter/source_map.rs`
   - Estimated Complexity: Medium

3. **Async ES5 Edge Cases**
   - Context: Async ES5 transforms have known issues with computed super and nested arrow captures.
   - Success Criteria: All async ES5 tests pass; `await` in various contexts (for-of, try-catch, template literals) maps correctly.
   - Key Files: `wasm/src/transforms/async_es5.rs`, `wasm/src/transforms/async_es5_tests.rs`
   - Estimated Complexity: High

4. **End-to-End Compilation**
   - Context: Stop adding features; start compiling real code.
   - Success Criteria: Successfully compile a non-trivial generic library (e.g., `redux` types) without panicking.
   - Key Files: `wasm/src/cli/driver.rs`, `wasm/src/thin_emitter/mod.rs`
   - Estimated Complexity: Medium

## Anti-Priorities
- New LSP features (Semantic Tokens, Code Actions)
- CLI argument parsing or fancy terminal output
- Performance micro-optimizations
- New emitter features beyond ES5 parity

## Cross-Squad Dependencies
- Solver squad owns type checking; emitter may expose Solver bugs
- Coordinate if transforms need new type information

## Notes to EM
- Read `wasm/specs/WASM_ARCHITECTURE.md` for architecture
- Use Docker for tests: `./wasm/test.sh`
- ES5 computed `super["m"]` fix recently landed; async version still needs work
- Focus on regression tests to guard against breakage

## Squad Status
- Last EM Report: Not yet started
- Workers Active: 0/3
- Branches Pending Merge: None
- Current Focus: Awaiting EM assignment
- Blockers: None
