# Checker Track Plan (ThinChecker + Control Flow)

## Mission
Reach compiler-case parity for TypeScript semantics with a performance-first, solver-driven checker.

## Scope
Files: `wasm/src/thin_checker.rs`, `wasm/src/checker/*`, `wasm/src/solver/*` (integration), `wasm/src/thin_binder.rs`.

## Current Status
- Solver TypeDatabase + lowering/inference/compat layers are integrated.
- Control flow and narrowing are in place but missing some false-branch logic.
- TODOs remain in call/construct signatures and readonly modifiers.
- Checker still uses a local scope stack even though binder now exposes persistent scopes.

## Highest-Impact Next Tasks (pick one at a time)
- [ ] Replace local scope stack with binder persistent scopes
  - Use `ThinBinderState::node_scope_ids` + `resolve_identifier` for symbol lookup.
  - Store types per `SymbolId` instead of per-scope maps.
  - This unlocks stateless queries and improves LSP random-access behavior.
- [ ] Handle type parameters in call/construct signatures
  - Populate `SolverCallSignature.type_params` from interface signature nodes.
  - Thread through call resolution / inference; add tests for generic call signatures.
- [ ] Honor readonly modifiers on property/method signatures
  - Read `readonly` in `thin_checker.rs` when lowering interface members.
  - Enforce readonly assignment rules in `checker/expr.rs` + subtype checks.
- [ ] Complete control-flow narrowing for false branches
  - `typeof` false branch exclusion; truthiness false branch (null/undefined/false/0/"").
  - Add tests in `checker/control_flow.rs`.
- [ ] Namespace member resolution parity
  - Verify 2694/2700 errors for missing members.
  - Add tests for nested namespaces and `import Alias = ns.Member`.

## Baseline / Validation
- `./wasm/test.sh`
- `node scripts/baseline-test-rust.mjs`
- Track pass rate for `tests/cases/compiler`.

## Success Criteria
- Passes `tests/cases/compiler` with >95% error parity.
- Checker queries are stateless and SymbolId-based (no manual scope stack).
- No new allocations in hot paths beyond solver interner.
