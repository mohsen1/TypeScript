# Worker 10 Task List - Solver Squad

## Current Task
- [ ] **SOLV-10: Write comprehensive conformance tests for Solver**
  - Create test file: `tests/conformance/solver_tests.ts`
  - Add 100+ cases covering: assignments, generics, unions, intersections, any/unknown
  - Include all TS2322 and TS7006 variations
  - Verify error output matches tsc exactly

## Queue
- [ ] **SOLV-11: Run full conformance suite and analyze results**
  - Execute `cargo test --test conformance`
  - Compare error counts against baseline
  - Generate report: missing errors reduced from 68.2% to <40%
  - Document remaining gaps in `docs/conformance_gap_report.md`
- [ ] **SOLV-12: Verify squad goals achieved**
  - Confirm Exact Match increased from 23.4% to 35%
  - Confirm Missing Errors decreased below 40%
  - Confirm TS2304 false positives eliminated
  - Tag release if all metrics met

## Completed
(none yet)
