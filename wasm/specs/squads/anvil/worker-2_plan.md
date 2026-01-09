# Worker 2 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 2

## Current Assignment
Fix TS2322 false positives (Type not assignable - 101 occurrences)

Per GOALS.md Phase 10: False Positive Elimination

**Problem:** "Type 'X' is not assignable to type 'Y'" when it should be

Root Causes:
1. Generic type inference too strict
2. Literal types not widening correctly
3. Union type assignability incomplete
4. Contextual typing not applied

Files: `thin_checker.rs`, `solver/subtype.rs`

Steps:
1. Run conformance baseline: `cd wasm/differential-test && bash run-conformance.sh --all --workers=14`
2. Record baseline TS2322 count and exact match %
3. Investigate TS2322 false positive cases in conformance output
4. Identify patterns and fix in checker/solver
5. Run conformance again to verify reduction
6. Commit with message: `[wasm] checker: fix TS2322 false positives`
7. Push to `origin/worker/anvil-2`
8. Update this plan file and push

## Task Queue
(empty - will receive new tasks from EM after completing current assignment)

## Completed
- [x] ES5 template literal type parity tests (449+ tests added)
- [x] TS2304 method type parameter resolution fix
- [x] Extensive emitter parity test coverage

## Ready for Merge
No

## Notes
- Project Direction: conformance-first; prioritize reducing false positives
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance tests: `cd wasm/differential-test && bash run-conformance.sh`
- Commit format: `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-2`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
