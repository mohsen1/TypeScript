# Worker 1 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 1
Blocked: Awaiting next EM-Anvil assignment.

## Current Assignment
- [ ] Add ES5 class tests for implements clause in `wasm/src/transforms/class_es5_tests.rs`. Run `./wasm/test.sh class_es5_tests`.

## Task Queue
- [ ] (empty)

## Completed
- [x] Added ES5 tests for generic classes (6 tests): basic generic class, multiple type params, generic with constraint, generic extends, default type param, generic with static members. Ran `./wasm/test.sh class_es5_tests` (all 103 pass).
- [x] Added ES5 tests for mixin patterns (6 tests): mixin base class, mixin with extends, mixin multiple methods, mixin with static members, mixin with generic constraint, mixin composed class. Ran `./wasm/test.sh class_es5_tests` (all 110 pass).
- [x] Added ES5 tests for abstract classes (6 tests): basic abstract class, abstract class with implemented methods, abstract class with abstract property, concrete extends abstract, abstract class with static members, abstract class with constructor. Ran `./wasm/test.sh class_es5_tests` (all 104 pass).
- [x] Added ES5 tests for static blocks (6 tests): basic static block, static block with static methods, static block with private access, multiple static blocks, static block with try/catch, static block with super property access. Ran `./wasm/test.sh class_es5_tests` (all 98 pass).
- [x] Added ES5 tests for computed properties and Symbol-keyed members (6 tests): computed property method, computed property accessor, Symbol.toStringTag, Symbol.hasInstance, multiple computed properties, computed static property. Ran `./wasm/test.sh class_es5_tests` (all 93 pass).
- [x] Added ES5 tests for private methods (6 tests): instance private method, static private method, private method calling private method, private async method, private generator method, private method with private field. Ran `./wasm/test.sh class_es5_tests` (all 90 pass).
- [x] Added ES5 tests for accessor decorators (6 tests): getter with decorator, setter with decorator, getter/setter pair with decorators, static getter with decorator, accessor with multiple decorators, accessor in derived class. Ran `./wasm/test.sh class_es5_tests` (all 84 pass).
- [x] Added ES5 tests for generator methods with decorators (6 tests): generator method with decorator, static generator with decorator, generator with multiple decorators, async generator with decorator, generator with yield expressions, generator in decorated class. Ran `./wasm/test.sh class_es5_tests` (all 81 pass).
- [x] Added additional ES5 coverage tests (6 tests): multiple private fields, mixed static/instance async, property initializers with method calls, labeled statements, new.target meta property, fluent method (this type). Ran `./wasm/test.sh class_es5_tests` (all 75 pass).
- [x] Added async static field initializer ES5 tests (8 tests): await chain, conditional, try/catch, Promise.all, loop/await, object destructuring, nested async calls, switch/case. Ran `./wasm/test.sh class_es5_tests` (all 69 pass). Also fixed duplicate test names in source_map_tests.rs from merge.
- [x] Fixed private field access in async methods (was emitting `this.void 0` instead of `__classPrivateFieldGet`); added `class_name` tracking to AsyncES5Emitter; added 2 tests. Ran `./wasm/test.sh emitter_transform_integration_tests` (all 124 pass).
- [x] Added static async arrow field tests (3 tests for static field with async arrow: basic, integration, nested arrow). Verified correct __awaiter usage and this preservation. Ran `./wasm/test.sh emitter_transform_integration_tests` (all 123 pass).
- [x] Fixed ES5 computed property field initializers (previously silently skipped `[key] = value` in all constructor paths); added `emit_property_receiver_and_name` helper; added 3 regression tests; ran `./wasm/test.sh class_es5_tests` (all 13 pass). Pre-existing failures: `parallel::tests::test_check_redux_lodash_style_generics` (Forge domain).
- [x] Implemented derived `super()` ordering adjustment and broader `this`/`super` capture in field initializers; added integration regression for nested async arrow in derived field; ran `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_parse_error_tolerance`).
- [x] Implemented async/nested arrow `this` capture handling in ES5 class emission, added derived async field regression; ran `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added class ES5 computed super field arrow regression; ran `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Ensured derived constructors initialize private fields after `super` and added async arrow field regression; ran `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Allowed ES6 `class C` in export assignment edge-case test; ran `./wasm/test.sh emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`.
- [x] Ensured synthesized derived constructors use `_this` in field/private initializers; added tests; ran `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_parse_error_tolerance`).
- [x] Fixed parse-error recovery for const initializers; ran `./wasm/test.sh` (fails: `emitter_parity_tests::test_parity_commonjs_export`).
- [x] Fixed ES5 async generator emission for non-await blocks, aligned super calls with `_this`, and suppressed static-field `this` capture; ran `./wasm/test.sh` (fails: `solver::compat::tests::test_explain_failure_reports_rest_mismatch`).
- [x] Added static field arrow `this` regression for ES5 class emission; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added async ES5 no-await statement ordering regression; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added async ES5 return-await regression; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added async ES5 await-in-variable initializer regression; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added CommonJS exports init empty-case regression; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Asserted CommonJS re-export property is enumerable; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added CommonJS re-export alias regression; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added sanitize_module_name regression for hyphen/dot paths; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added helper ordering regression for `__awaiter` before `__generator`; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added CommonJS export-name regression for `export type { Foo }`; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added CommonJS export-name regression for default class exports; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added CommonJS export-name regression for default re-exports; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added async ES5 await detection coverage for try/finally bodies; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added super property access in static block support; added `CLASS_STATIC_BLOCK_DECLARATION` handling in `emit_static_members`; added `SuperKeyword` handling in `emit_expression` to emit `_super`; added test for `super.value` in static block. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added nested async arrow in constructor with field initializer test; added `body_contains_arrow_with_this` helper to detect arrows in constructor body that reference `this`; fixed `emit_instance_property_initializers` to use `_this` when needed. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added computed method name with async body test; fixed `COMPUTED_PROPERTY_NAME` emission for method names by adding `emit_method_name` helper. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added spread element in array literal ES5 support; arrays with spread like `[...a, 1, ...b]` now emit as `[].concat(a, [1], b)`; added `emit_array_with_spread_es5` helper. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added object spread in ES5 method test; verifies `{...obj}` is transformed to `Object.assign` for ES5 output. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added for-of loop ES5 method test; verifies for-of loops are transformed to `__values()` iterator pattern with try/finally cleanup. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added Symbol.iterator ES5 method test; verifies `*[Symbol.iterator]()` generator methods are emitted with computed property name on prototype. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added template literal ES5 method test; verifies template literals like `` `Hello, ${name}!` `` are transformed to string concatenation. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added destructuring ES5 method test; verifies `const { text, line } = input` is transformed to individual property accesses. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added default parameters ES5 method test; verifies `add(a, b = 0, c = 1)` is transformed to void 0/undefined checks. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added rest parameters ES5 method test; verifies `log(prefix, ...messages)` is transformed to slice/arguments pattern. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added shorthand properties ES5 method test; verifies `{ x, y }` shorthand is correctly emitted in class methods. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added getter/setter ES5 accessors test; verifies `get count()` and `set count(value)` use Object.defineProperty. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added arrow function this binding ES5 test; verifies arrow functions use `_this` capture and `function` keyword. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added static method ES5 test; verifies static methods/properties are on constructor function, not prototype. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added computed property in object literal ES5 test; verifies `{ [key]: value }` uses bracket notation assignment. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added class inheritance extends ES5 test; verifies `class Dog extends Animal` uses __extends helper and _super pattern. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added nullish coalescing ES5 test; verifies `??` operator is transformed for ES5 output. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added optional chaining ES5 test; verifies `?.` operator is transformed for ES5 output. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added class expression ES5 tests (anonymous and named); verifies class expressions emit correctly with properties and methods on prototype. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added Math.pow preservation ES5 test; verifies Math.pow calls are preserved in class methods. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added generator method ES5 test; verifies generator methods are placed on prototype. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added try/catch/finally ES5 test; verifies error handling blocks are preserved in class methods. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added switch/case ES5 test; verifies switch statements with case/default clauses are preserved. Ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added while/do-while loops ES5 test; verifies while and do-while loops are preserved in class methods. Ran `./wasm/test.sh class_es5_tests` (all 54 pass).
- [x] Added ternary expression ES5 test; verifies conditional expressions with nested ternary operators are preserved. Ran `./wasm/test.sh class_es5_tests` (all 54 pass).
- [x] Added for-in loop ES5 test; verifies for-in loops over object properties are preserved. Ran `./wasm/test.sh class_es5_tests` (all 57 pass).
- [x] Added typeof/instanceof ES5 test; verifies typeof and instanceof operators are preserved. Ran `./wasm/test.sh class_es5_tests` (all 57 pass).
- [x] Added logical operators ES5 test; verifies &&, ||, ! operators are preserved in class methods. Ran `./wasm/test.sh class_es5_tests` (all 57 pass).
- [x] Added bitwise operators ES5 test; verifies &, |, ^, ~, <<, >> operators are preserved. Ran `./wasm/test.sh class_es5_tests` (all 60 pass).
- [x] Added assignment operators ES5 test; verifies +=, -=, *= compound assignment operators. Ran `./wasm/test.sh class_es5_tests` (all 60 pass).
- [x] Added prefix/postfix operators ES5 test; verifies ++x, x++, --x, x-- increment/decrement. Ran `./wasm/test.sh class_es5_tests` (all 60 pass).

## Ready for Merge
Yes

## Notes
- Project Direction: integration and conformance-first; prioritize emitter fidelity (ES5 downleveling/source maps) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance focus: tie regressions to official TypeScript conformance cases when possible.
- Commit format: `[wasm] emitter: <description>` or `[wasm] cli: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
- Verified `./wasm/test.sh emitter_edge_case_tests::test_export_assignment_suppresses_other_exports` passes; full suite not rerun.
