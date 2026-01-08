# Worker 3 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 3

## Current Assignment
- [ ] Pick the next unsoundness case from `wasm/specs/TS_UNSOUNDNESS_CATALOG.md` and add coverage if missing. Focus on cases that affect solver correctness. Run `./wasm/test.sh` after adding tests.

## Task Queue
- [ ] Once constraint property lookup is implemented, update `test_cross_scope_generic_constraints` to expect 0 errors.
- [ ] Once setter type checking is implemented, update `test_split_accessors_write_error` to expect 1 error.
- [ ] Verify redux minimal repros pass once core issues are fixed

## Completed
- [x] Fixed string enum opaque assignability (TS unsoundness #34)
- [x] Fixed weak type check for empty objects
- [x] Fixed private member nominality error codes
- [x] Fixed static side assignability error codes
- [x] Object vs object vs {} trifecta (TS unsoundness #20)
- [x] Nominal classes (TS unsoundness #5)
- [x] Instantiation depth limit (TS unsoundness #17)
- [x] Class static side rules (TS unsoundness #18)
- [x] Numeric/string enum nominalness (TS unsoundness #7/#24/#34)
- [x] Template string expansion limits (TS unsoundness #22)
- [x] unique symbol nominal primitives (TS unsoundness #37)
- [x] Application type expansion in evaluate()

## Ready for Merge
No

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: <description>`
- Push to: `origin/worker/forge-3`
