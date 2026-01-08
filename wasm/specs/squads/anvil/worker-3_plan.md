# Worker 3 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 3

## Current Assignment
- Validate source map `sourcesContent`/`file` fields are set for JS + d.ts outputs when source text is available; adjust `wasm/src/source_writer.rs`, `wasm/src/thin_emitter/mod.rs`, or `wasm/src/declaration_emitter.rs` if needed; add regression in `wasm/src/cli/driver_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [ ] Add a `wasm/src/cli/driver_tests.rs` assertion that `sourcesContent` is present and matches the input when `sourceMap`/`declarationMap` are enabled.
- [ ] Add a `wasm/src/source_map_tests.rs` check that transformed output still records `names` entries for identifiers.
- [ ] Verify `sourceRoot` and `file` fields remain stable (non-empty `file`, empty `sourceRoot`) and lock with a test.

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Project Direction: integration and conformance-first; prioritize emitter fidelity (ES5 downleveling/source maps) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance focus: tie regressions to official TypeScript conformance cases when possible.
- Commit format: `[wasm] emitter: <description>` or `[wasm] cli: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
