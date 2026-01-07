# Checker Track Plan (ThinChecker + Control Flow)

## Mission
Reach compiler-case parity for TypeScript semantics with a performance-first, solver-driven checker.

## Scope
Files: `wasm/src/thin_checker.rs`, `wasm/src/checker/*`, `wasm/src/solver/*` (integration), `wasm/src/thin_binder.rs`.

## Current Status
Status: Complete
- Solver TypeDatabase + lowering/inference/compat layers are integrated.
- Control flow narrowing covers typeof/truthiness, discriminant/literal equality, logical `&&`/`||`, and loose nullish checks.
- Namespace member resolution covers nested namespaces and import-equals aliases.
- Namespace value member access resolves exported members in property access chains.
- Checker uses binder persistent scopes with SymbolId type caching; local scope stack removed.
- Flow positions are recorded for identifier nodes to enable branch narrowing.
- Solver inference skips constraining defaulted placeholders in union targets to preserve defaults.
- Type literal lowering uses checker paths for type params while preserving ref semantics for named members.
- Solver diagnostics rendering preserves related messages without spans via fallback span.

## Highest-Impact Next Tasks (pick one at a time)
- [x] Replace local scope stack with binder persistent scopes
  - Use `ThinBinderState::node_scope_ids` + `resolve_identifier` for symbol lookup.
  - Store types per `SymbolId` instead of per-scope maps.
  - This unlocks stateless queries and improves LSP random-access behavior.
- [x] Handle type parameters in call/construct signatures
  - Populate `SolverCallSignature.type_params` from interface signature nodes.
  - Thread through call resolution / inference; add tests for generic call signatures.
- [x] Honor readonly modifiers on property/method signatures
  - Read `readonly` in `thin_checker.rs` when lowering interface members.
  - Enforce readonly assignment rules in `checker/expr.rs` + subtype checks.
- [x] Complete control-flow narrowing for false branches
  - `typeof` false branch exclusion; truthiness false branch (null/undefined/false/0/"").
  - Add tests in `checker/control_flow.rs`.
- [x] Namespace member resolution parity
  - Verify 2694/2700 errors for missing members.
  - Add tests for nested namespaces and `import Alias = ns.Member`.
- [x] Resolve namespace value member access
  - Allow `Namespace.value` and nested namespace value chains.
  - Add tests for property access via namespaces and import-equals aliases.
- [x] Add discriminant + literal equality narrowing in control flow
  - Handle property/element access discriminants and direct literal checks.
  - Support loose nullish equality (`==` / `!=`) narrowing.
- [x] Add control-flow tests for discriminant, literal equality, and loose nullish checks
- [x] Record flow nodes for identifiers in all contexts
  - Use current flow when binding identifier nodes to enable branch narrowing.
- [x] Add checker test to verify flow narrowing inside if branches
- [x] Narrow logical `&&`/`||` conditions in control flow
  - Apply sequential narrowing for `&&` and union-of-paths narrowing for `||`.
- [x] Add control-flow tests for logical `&&` and `||` narrowing
- [x] Add switch/case narrowing using `SWITCH_CLAUSE` flow nodes
  - Narrow discriminants per case and handle fallthrough/default.
  - Add tests covering switch unions and default behavior.
- [x] Implement `instanceof` and `in` operator narrowing
  - Respect structural/object checks and report safe narrowings only.
  - Add tests for primitive/object and union cases.
- [x] Use user-defined type predicate signatures in flow narrowing
  - Narrow based on call expressions returning `x is T` or `asserts x is T`.
  - Add tests for predicate functions and alias references.
  - Risk: predicate narrowing depends on cached call-expression types and is conservative for overloads/complex `this` predicates.
- [x] Track assignment/mutation flow to widen/clear stale narrowings
  - Emit assignment/array-mutation flow nodes and update flow analyzer.
  - Add tests for reassignment inside branches.
  - Risk: assignment clears to declared type (RHS type not tracked) and array mutation detection is method-name based; destructuring/aliasing writes still conservative.

## Baseline / Validation
- `./wasm/test.sh`
- `node scripts/baseline-test-rust.mjs`
- Track pass rate for `tests/cases/compiler`.

## Success Criteria
- Passes `tests/cases/compiler` with >95% error parity.
- Checker queries are stateless and SymbolId-based (no manual scope stack).
- No new allocations in hot paths beyond solver interner.
