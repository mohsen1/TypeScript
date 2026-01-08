# Squad Anvil Goals

Updated: 2026-01-08

Priority: 2

## Current Milestone
Output Fidelity - Make JavaScript output semantically identical to TypeScript.

## Focus Areas (Director can reassign)
- `wasm/src/thin_emitter/` - JavaScript emission
- `wasm/src/transforms/` - ES5 downleveling, source maps
- `wasm/src/cli/` - Command-line interface
- `wasm/src/lsp/` - Language Server Protocol

## Objectives (Ranked)

1. **ES5 Downleveling Correctness**
   - Context: ES5 transforms must produce semantically identical JavaScript to `tsc`
   - Success Criteria: All ES5 transform tests pass; `super()` and `this` capture work
   - Key Files: `transforms/class_es5.rs`, `transforms/async_es5.rs`
   - Estimated Complexity: High

2. **Source Map Validation**
   - Context: Source maps must be valid and usable by debuggers
   - Success Criteria: Generated source maps pass validation
   - Key Files: `thin_emitter/source_writer.rs`, `thin_emitter/source_map.rs`
   - Estimated Complexity: Medium

3. **Async ES5 Edge Cases**
   - Context: Async ES5 transforms have known issues with computed super
   - Success Criteria: All async ES5 tests pass
   - Key Files: `transforms/async_es5.rs`, `transforms/async_es5_tests.rs`
   - Estimated Complexity: High

4. **End-to-End Compilation**
   - Context: Compile real code without panicking
   - Success Criteria: Successfully compile redux types
   - Key Files: `cli/driver.rs`, `thin_emitter/mod.rs`
   - Estimated Complexity: Medium

## Anti-Priorities
- New LSP features (Semantic Tokens, Code Actions)
- CLI argument parsing or fancy terminal output
- Performance micro-optimizations

## Cross-Squad Dependencies
- Forge squad owns type checking; emitter may expose Forge bugs

## Notes to EM
- Read `wasm/specs/WASM_ARCHITECTURE.md` for architecture
- Use Docker for tests: `./wasm/test.sh`
- Focus on regression tests to guard against breakage

## Squad Status
- Last EM Report: Not yet started
- Workers Active: 0/5
- Branches Pending Merge: None
- Current Focus: Awaiting EM assignment
- Blockers: None
