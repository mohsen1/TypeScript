# Worker 3 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 1

## Current Assignment
Reduce TS2339 false positives for inherited/prototype properties in `thin_checker`.

Steps:
1. Capture 5-10 TS2339 false-positive conformance samples (use `wasm/differential-test/run-conformance.sh --max=500 --sequential --verbose` or a small script).
2. Audit property access in `wasm/src/thin_checker.rs` (property lookup for class/interface types, base class/interface traversal, implements).
3. Implement minimal fix and add 1-2 regression tests in `wasm/src/thin_checker_tests.rs`:
   - class extends base property access
   - interface extends property access
   - class implements interface property access (if missing)
4. Run `./wasm/test.sh` for new tests and a conformance slice; record deltas here.

## Task Queue
- Check static member lookup across class inheritance chains.
- Validate union/intersection property lookup doesn't regress (no new extra TS2339).
- Ensure `this`-property access in class bodies is resolved via instance type.

## Completed
(none yet for this assignment)

## Ready for Merge
No

## Notes
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
- Focus on false positives (extra TS2339).
