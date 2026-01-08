# Squad Anvil Goals

Updated: 2026-01-08

Priority: 2

## Current Milestone
Phase 8 - Conformance, Convergence, and Hardening: Output fidelity across the integrated pipeline, driven by conformance tests.

## Focus Areas (Director can reassign)
- `wasm/src/thin_emitter/` - JavaScript emission
- `wasm/src/transforms/` - ES5 downleveling, source maps
- `wasm/src/cli/` - Command-line interface
- `wasm/src/lsp/` - Language Server Protocol

## Objectives (Ranked)

1. **ES5 Downleveling Correctness**
   - Context: Emitter fidelity is the utility bottleneck per Project Direction
   - Success Criteria: Match `tsc` for `super()` in derived classes with field initializers, nested arrow/async `this` capture, and computed `super[...]` cases
   - Key Files: `transforms/class_es5.rs`, `transforms/async_es5.rs`
   - Estimated Complexity: High

2. **Source Map Validation**
   - Context: Source maps must be valid and usable by debuggers
   - Success Criteria: Generated source maps validate and attach correctly in debuggers across async/await downleveling
   - Key Files: `thin_emitter/source_writer.rs`, `thin_emitter/source_map.rs`
   - Estimated Complexity: Medium

3. **End-to-End Conformance Validation**
   - Context: Stop adding AST nodes; compile real code end-to-end
   - Success Criteria: Compile a non-trivial generic library (e.g., redux/lodash types) without panics and advance conformance test pass rate
   - Key Files: `cli/driver.rs`, `thin_emitter/mod.rs`
   - Estimated Complexity: Medium

## Anti-Priorities
- New LSP features (Semantic Tokens, Code Actions)
- CLI argument parsing or fancy terminal output
- Performance micro-optimizations (unless regression)
- New AST nodes or isolated features outside emitter correctness

## Cross-Squad Dependencies
- Forge squad owns type checking; emitter may expose Forge bugs

## Notes to EM
- Read `wasm/specs/WASM_ARCHITECTURE.md` for architecture
- Use Docker for tests: `./wasm/test.sh`
- Focus on regression tests to guard against breakage

## Squad Status
- Last EM Report: Conformance-driven focus confirmed; prioritize ES5 emitter regressions and source-map fidelity.
- Workers Active: 0/5
- Branches Pending Merge: None
- Current Focus: Conformance-driven emitter regressions and ES5 fidelity
- Blockers: None
