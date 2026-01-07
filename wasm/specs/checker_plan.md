# Checker Track Plan (ThinChecker + Control Flow)

## Mission
Reach compiler-case parity for TypeScript semantics with a performance-first, solver-driven checker.

## Scope
Files: `wasm/src/thin_checker.rs`, `wasm/src/checker/*`, `wasm/src/solver/*` (integration), `wasm/src/thin_binder.rs`.

## Current Status
Status: Active
- Solver TypeDatabase + lowering/inference/compat layers are integrated.
- Control flow narrowing covers typeof/truthiness, discriminant/literal equality, logical `&&`/`||`, and loose nullish checks.
- Namespace member resolution covers nested namespaces and import-equals aliases.
- Namespace value member access resolves exported members in property access chains.
- Checker uses binder persistent scopes with SymbolId type caching; local scope stack removed.
- Flow positions are recorded for identifier nodes to enable branch narrowing.
- Solver inference skips constraining defaulted placeholders in union targets to preserve defaults.
- Type literal lowering uses checker paths for type params while preserving ref semantics for named members.
- Solver diagnostics rendering preserves related messages without spans via fallback span.
- TS2693 namespace type-only access coverage re-validated.
- Alias type-only namespace value error test re-validated.

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
- [x] Clear narrowing for destructuring and compound assignments
  - Inspect assignment patterns (array/object) to detect bound references.
  - Add focused tests for destructuring and compound assignment narrowing clears.
  - Risk: still conservative for nested aliasing and property writes beyond the base identifier.
- [x] Clear narrowing for destructuring default initializers and aliases
  - Handle assignment patterns like `[x = 1] = ...` and `{ y: x = 1 } = ...`.
  - Add focused tests for default initializer and alias patterns (including alias-only assignment).
  - Risk: still conservative for nested initializer side effects.
- [x] Re-enable namespace member tests
  - Remove stale TODO suppression.
  - Add focused tests for missing namespace value members.
- [x] Expand namespace member tests for alias resolution
  - Add coverage for missing members through nested namespace aliases.
- [x] Expand namespace member error coverage for non-exported values
  - Ensure namespace value access reports TS2339 for non-exported members.
- [x] Expand namespace member error coverage for nested values
  - Add coverage for missing nested namespace value members.
- [x] Enforce type-only namespace members in value position
  - Report TS2693 when accessing interface/type alias exports as values.
- [x] Enforce type-only namespace aliases in value position
  - Report TS2693 when `import Alias = NS.Type` is used as a value.
- [x] Enforce type-only namespace members through alias chains
  - Add coverage for `import Alias = NS; Alias.Foo` and nested alias member access.
- [x] Enforce type-only namespace members in nested access
  - Add coverage for `Outer.Inner.Type` used as a value.
- [x] Report TS2693 for local type-only symbols in value position
  - Error on interface/type alias usage in expression contexts.
  - Add focused tests for interface and type alias values.
- [x] Report TS2693 for type-only symbols in `typeof` type queries
  - Error when `typeof` references interface/type alias exports.
  - Add focused tests for type-query usage.
- [x] Report TS2304 for unknown names in `typeof` type queries
  - Error on missing identifier targets in `typeof`.
  - Add focused tests for unknown `typeof` names.
- [x] Report TS2304 for unknown qualified names in `typeof` type queries
  - Error when `typeof Missing.Member` references an unknown base identifier.
  - Add focused tests for unknown qualified `typeof` names.
- [x] Report TS2694 for missing namespace members in `typeof` type queries
  - Error when `typeof Ns.Missing` references a namespace member that is not exported.
  - Add focused tests for missing namespace members in `typeof`.
- [x] Report TS2749 for value-only symbols in type positions
  - Error on local value names, functions, and namespace symbols used as types.
  - Add coverage for namespace alias chains and namespace-as-type usage.
- [x] Add loop flow labels for while/do statements
  - Enable narrowing within `while` bodies and keep `do` bodies conservative.
  - Add focused flow tests for loop narrowing.
- [x] Add loop flow labels for for/for-in/for-of statements
  - Narrow within `for` condition bodies and keep iterator loops conservative.
  - Add focused flow tests for `for` condition narrowing.
  - Risk: loop exit narrowing remains conservative (no fixed point for assignments).
- [x] Add flow tests for `for-in`/`for-of` loop bodies
  - Ensure iterator loop headers do not narrow unrelated variables.
- [x] Keep loop-exit narrowing conservative for while/for/do statements
  - Avoid applying false-condition narrowing after loop exits that can break early.
  - Add focused tests for while/for loop exit behavior.
  - Add focused test for do-while loop exit behavior.
- [x] Resolve namespace alias members in flow narrowing/clearing
  - Allow `typeof Alias.value` to narrow namespace members in true branches.
  - Clear narrowings when namespace members are reassigned via aliases.
- [x] Support namespace member element access in value + flow contexts
  - Resolve `Ns["value"]` to exported value members (including alias access).
  - Apply flow narrowing for string-literal element access and report TS2693 for type-only members.
  - Risk: computed element access (non-literal) still falls back to index signature rules.
- [x] Clear flow narrowing when property/element access bases are reassigned
  - Treat property/element access chains with stable names as narrowable references.
  - Clear narrowings when the base object is reassigned.
  - Risk: complex computed access remains conservative.
- [x] Clear flow narrowing when property/element access targets are reassigned
  - Clear narrowings after `obj.prop = ...` and `obj["prop"] = ...`.
  - Risk: computed access with non-literal keys remains conservative.
- [x] Keep property and element access references aligned in flow narrowing
  - Narrow across `obj.prop` and `obj["prop"]` forms.
  - Clear narrowings when assignments use the opposite access form.
- [x] Keep computed element access conservative in flow narrowing
  - Do not narrow `obj[key]` when the key is not a literal string.
- [x] Narrow element access with literal string keys
  - Apply flow narrowing for `obj["prop"]` in type guard branches.

## Baseline / Validation
- `./wasm/test.sh`
- `node scripts/baseline-test-rust.mjs`
- Track pass rate for `tests/cases/compiler`.

## Success Criteria
- Passes `tests/cases/compiler` with >95% error parity.
- Checker queries are stateless and SymbolId-based (no manual scope stack).
- No new allocations in hot paths beyond solver interner.
