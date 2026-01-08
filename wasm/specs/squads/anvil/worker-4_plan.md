# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 4

## Current Assignment
- [ ] (empty)

## Task Queue
- [ ] (empty)

## Completed
- [x] Add ES5 class tests for extends expression. Tests: `./wasm/test.sh class_es5_tests`
- [x] Add ES5 class tests for private field initialization. Tests: `./wasm/test.sh class_es5_tests`
- [x] Add ES5 class tests for static accessors. Tests: `./wasm/test.sh class_es5_tests`
- [x] Add ES5 class tests for protected members. Tests: `./wasm/test.sh class_es5_tests`
- [x] Add ES5 class tests for parameter properties. Tests: `./wasm/test.sh class_es5_tests`
- [x] Add ES5 class tests for constructor overloads. Tests: `./wasm/test.sh class_es5_tests`
- [x] Add ES5 class tests for method overloads. Tests: `./wasm/test.sh class_es5_tests`
- [x] Add ES5 class tests for accessor keyword (ES2022). Tests: `./wasm/test.sh class_es5_tests`
- [x] Add ES5 class expression anonymous test. Tests: `./wasm/test.sh class_expression_anonymous`
- [x] Add ES5 class expression named test. Tests: `./wasm/test.sh class_expression_named`
- [x] Add ES5 class expression in return test. Tests: `./wasm/test.sh class_expression_in_return`
- [x] Add ES5 class expression extends test. Tests: `./wasm/test.sh class_expression_extends`
- [x] Add ES5 class expression with static test. Tests: `./wasm/test.sh class_expression_with_static`
- [x] Add ES5 super with conditional field init test. Tests: `./wasm/test.sh super_with_conditional_field`
- [x] Add ES5 super with arrow field init test. Tests: `./wasm/test.sh super_with_arrow_field`
- [x] Add ES5 super with computed field init test. Tests: `./wasm/test.sh super_with_computed_field`
- [x] Add ES5 super with method call in field init test. Tests: `./wasm/test.sh super_with_method_call`
- [x] Add ES5 super with nested inheritance field init test. Tests: `./wasm/test.sh super_with_nested_inheritance`
- [x] Add ES5 constructor param inject decorator test. Tests: `./wasm/test.sh constructor_param_inject`
- [x] Add ES5 constructor param optional decorator test. Tests: `./wasm/test.sh constructor_param_optional`
- [x] Add ES5 constructor param attribute decorator test. Tests: `./wasm/test.sh constructor_param_attribute`
- [x] Add ES5 async method decorator pattern test. Tests: `./wasm/test.sh async_method_decorator_pattern`
- [x] Add ES5 async lifecycle decorator pattern test. Tests: `./wasm/test.sh async_lifecycle_decorator_pattern`
- [x] Add ES5 async event handler decorator pattern test. Tests: `./wasm/test.sh async_event_handler_decorator_pattern`
- [x] Add ES5 component decorator pattern test. Tests: `./wasm/test.sh component_decorator_pattern`
- [x] Add ES5 injectable service pattern test. Tests: `./wasm/test.sh injectable_service_pattern`
- [x] Add ES5 entity decorator pattern test. Tests: `./wasm/test.sh entity_decorator_pattern`
- [x] Add ES5 async static method test. Tests: `./wasm/test.sh async_static_method`
- [x] Add ES5 decorators syntax test. Tests: `./wasm/test.sh decorators_syntax`
- [x] Add ES5 parameter decorator pattern test. Tests: `./wasm/test.sh parameter_decorator_pattern`
- [x] Add ES5 property decorator pattern test. Tests: `./wasm/test.sh property_decorator_pattern`
- [x] Add async/class ES5 transform source-map mappings and offsets. Tests: `./wasm/test.sh source_map`
- [x] Add async nested function source-map offset coverage. Tests: `./wasm/test.sh source_map`
- [x] Add async await detection test for nested functions. Tests: `./wasm/test.sh body_contains_await`
- [x] Add ES5 derived default constructor ordering test. Tests: `./wasm/test.sh default_derived_constructor`
- [x] Add ES5 derived constructor ordering test (super/field/body). Tests: `./wasm/test.sh` (fails in `parallel::tests::test_check_redux_lodash_style_generics`)
- [x] Add async await initializer assignment coverage. Tests: `./wasm/test.sh async_es5`
- [x] Restore async ES5 emitter this-capture setter for new call sites. Tests: `./wasm/test.sh async_es5`
- [x] Add async ES5 this-capture test. Tests: `./wasm/test.sh async_es5`
- [x] Add derived ctor pre-super ordering test with field init. Tests: `./wasm/test.sh derived_constructor_preserves_pre_super`
- [x] Add async await var initializer source-map coverage. Tests: `./wasm/test.sh async_await_var_initializer_mapping`
- [x] Add async await call property source-map coverage. Tests: `./wasm/test.sh async_await_call_property_mapping`
- [x] Add async await element access source-map coverage. Tests: `./wasm/test.sh async_await_element_access_mapping`
- [x] Add async await call source-map coverage. Tests: `./wasm/test.sh async_await_call_mapping`
- [x] Add async await call argument source-map coverage. Tests: `./wasm/test.sh async_await_call_argument_mapping`
- [x] Add async await detection for call arguments. Tests: `./wasm/test.sh body_contains_await_in_call_argument`
- [x] Add async await detection for call expression callee. Tests: `./wasm/test.sh body_contains_await_in_call_expression_callee`
- [x] Add async await detection for binary expressions. Tests: `./wasm/test.sh body_contains_await_in_binary_expression`
- [x] Add async await detection for conditional expressions. Tests: `./wasm/test.sh body_contains_await_in_conditional_expression`
- [x] Add async await detection for unary expressions. Tests: `./wasm/test.sh body_contains_await_in_unary_expression`
- [x] Add async await detection for parenthesized expressions. Tests: `./wasm/test.sh body_contains_await_in_parenthesized_expression`
- [x] Add async await detection for object literals. Tests: `./wasm/test.sh body_contains_await_in_object_literal`
- [x] Add async await detection for array literals. Tests: `./wasm/test.sh body_contains_await_in_array_literal`
- [x] Add async await detection for computed object literal names. Tests: `./wasm/test.sh body_contains_await_in_object_literal_computed_name`
- [x] Add async await detection for object literal spreads. Tests: `./wasm/test.sh body_contains_await_in_object_literal_spread`
- [x] Add async await detection for array literal spreads. Tests: `./wasm/test.sh body_contains_await_in_array_literal_spread`
- [x] Add async await detection for template expressions. Tests: `./wasm/test.sh body_contains_await_in_template_expression`
- [x] Add async await detection for tagged templates. Tests: `./wasm/test.sh body_contains_await_in_tagged_template`
- [x] Add async await detection for as expressions. Tests: `./wasm/test.sh body_contains_await_in_as_expression`
- [x] Add async await detection for type assertions. Tests: `./wasm/test.sh body_contains_await_in_type_assertion`
- [x] Add async await detection for non-null expressions. Tests: `./wasm/test.sh body_contains_await_in_non_null_expression`
- [x] Add async await detection for new expressions. Tests: `./wasm/test.sh body_contains_await_in_new_expression`
- [x] Add async await detection for try statements. Tests: `./wasm/test.sh body_contains_await_in_try_statement`
- [x] Add async await detection for catch clauses. Tests: `./wasm/test.sh body_contains_await_in_catch_clause`
- [x] Add async await detection for finally blocks. Tests: `./wasm/test.sh body_contains_await_in_finally_block`
- [x] Add async await detection for switch expressions. Tests: `./wasm/test.sh body_contains_await_in_switch_expression`
- [x] Add async await detection for switch case statements. Tests: `./wasm/test.sh body_contains_await_in_switch_case_statement`
- [x] Add async await detection for switch default clauses. Tests: `./wasm/test.sh body_contains_await_in_switch_default_clause`
- [x] Add async await detection for for loop conditions. Tests: `./wasm/test.sh body_contains_await_in_for_loop_condition`
- [x] Add async await detection for for loop incrementor. Tests: `./wasm/test.sh body_contains_await_in_for_loop_incrementor`
- [x] Add async await detection for for loop initializer. Tests: `./wasm/test.sh body_contains_await_in_for_loop_initializer`
- [x] Add async await detection for while conditions. Tests: `./wasm/test.sh body_contains_await_in_while_condition`
- [x] Add async await detection for do-while conditions. Tests: `./wasm/test.sh body_contains_await_in_do_while_condition`
- [x] Add async await detection for for-of expressions. Tests: `./wasm/test.sh body_contains_await_in_for_of_expression`
- [x] Add async await detection for for-in expressions. Tests: `./wasm/test.sh body_contains_await_in_for_in_expression`
- [x] Add derived ctor param property ordering test. Tests: `./wasm/test.sh derived_constructor_orders_param_property`
- [x] Add derived ctor private field ordering test. Tests: `./wasm/test.sh derived_constructor_orders_private`
- [x] Add async await property name source-map coverage. Tests: `./wasm/test.sh async_await_property_name_mapping`
- [x] Add async await detection for if statement conditions. Tests: `./wasm/test.sh body_contains_await_in_if_condition`
- [x] Add async await detection for if else branches. Tests: `./wasm/test.sh body_contains_await_in_if_else_branch`
- [x] Add async await detection for throw statements (with fix). Tests: `./wasm/test.sh body_contains_await_in_throw_statement`
- [x] Add async spread await source-map coverage. Tests: `./wasm/test.sh async_spread_await_mapping`
- [x] Add async chained method await source-map coverage. Tests: `./wasm/test.sh async_chained_method_await_mapping`
- [x] Add class ES5 super() call source-map coverage. Tests: `./wasm/test.sh class_super_call_mapping`
- [x] Add arrow function default param source-map coverage. Tests: `./wasm/test.sh arrow_default_param_mapping`
- [x] Add destructuring assignment source-map coverage. Tests: `./wasm/test.sh destructuring_assignment_mapping`
- [x] Add template literal source-map coverage. Tests: `./wasm/test.sh template_literal_mapping`
- [x] Add source map roundtrip accuracy test. Tests: `./wasm/test.sh roundtrip_accuracy`
- [x] Add source map multiple files test. Tests: `./wasm/test.sh multiple_files`
- [x] Add inline source map generation test. Tests: `./wasm/test.sh inline_generation`
- [x] Add class private fields source-map test. Tests: `./wasm/test.sh private_fields_mapping`
- [x] Add nullish coalescing source-map test. Tests: `./wasm/test.sh nullish_coalescing_mapping`
- [x] Add numeric separators source-map test. Tests: `./wasm/test.sh numeric_separators_mapping`
- [x] Add import.meta source-map test. Tests: `./wasm/test.sh import_meta_mapping`
- [x] Add export * as namespace source-map test. Tests: `./wasm/test.sh export_star_as_namespace_mapping`
- [x] Add ES5 class super() with spread args edge case test. Tests: `./wasm/test.sh super_with_spread_args_and_field_init`
- [x] Add async ES5 downlevel source-map offset accuracy test. Tests: `./wasm/test.sh async_es5_offset_accuracy`
- [x] Add generator function ES5 source-map offset accuracy test. Tests: `./wasm/test.sh generator_es5_offset_accuracy`
- [x] Add optional chaining source-map test. Tests: `./wasm/test.sh optional_chaining_mapping`
- [x] Add logical assignment operators source-map test. Tests: `./wasm/test.sh logical_assignment_operators_mapping`
- [x] Add class static block source-map test. Tests: `./wasm/test.sh class_static_block_mapping`
- [x] Add BigInt literals source-map test. Tests: `./wasm/test.sh bigint_literals_mapping`
- [x] Add exponentiation operator source-map test. Tests: `./wasm/test.sh exponentiation_operator_mapping`
- [x] Add rest/spread source-map test. Tests: `./wasm/test.sh rest_spread_mapping`
- [x] Add default parameters source-map test. Tests: `./wasm/test.sh default_parameters_mapping`
- [x] Add computed property names source-map test. Tests: `./wasm/test.sh computed_property_names_mapping`
- [x] Add shorthand properties source-map test. Tests: `./wasm/test.sh shorthand_properties_mapping`
- [x] Add method definitions source-map test. Tests: `./wasm/test.sh method_definitions_mapping`
- [x] Add for-of/for-in loops source-map test. Tests: `./wasm/test.sh for_of_for_in_loops_mapping`
- [x] Add for-await-of ES5 async iteration source-map test. Tests: `./wasm/test.sh for_await_of_es5_mapping`
- [x] Add class getters/setters source-map test. Tests: `./wasm/test.sh class_getters_setters_mapping`
- [x] Add TypeScript namespace source-map test. Tests: `./wasm/test.sh typescript_namespace_mapping`
- [x] Add ES5 computed super[] edge case tests. Tests: `./wasm/test.sh computed_super`
- [x] Add ES5 super property access tests. Tests: `./wasm/test.sh super_property`
- [x] Add ES5 abstract class lowering test. Tests: `./wasm/test.sh abstract_class_lowering`
- [x] Add ES5 class with index signature test. Tests: `./wasm/test.sh class_with_index_signature`
- [x] Add ES5 class expression test. Tests: `./wasm/test.sh class_expression`
- [x] Add ES5 Symbol-keyed methods test. Tests: `./wasm/test.sh symbol_keyed_methods`
- [x] Add ES5 nested class test. Tests: `./wasm/test.sh nested_class`
- [x] Add ES5 generator methods test. Tests: `./wasm/test.sh generator_methods`
- [x] Add ES5 deep inheritance chain test. Tests: `./wasm/test.sh deep_inheritance_chain`
- [x] Add ES5 mixin pattern test. Tests: `./wasm/test.sh mixin_pattern`
- [x] Add ES5 method overloads test. Tests: `./wasm/test.sh method_overloads`
- [x] Add ES5 computed method names test. Tests: `./wasm/test.sh computed_method_names`
- [x] Add ES5 optional and readonly properties test. Tests: `./wasm/test.sh optional_and_readonly`
- [x] Add ES5 constructor overloads test. Tests: `./wasm/test.sh constructor_overloads`

## Ready for Merge
Yes

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance focus: tie regressions to official TypeScript conformance cases when possible.
- Commit format: `[wasm] emitter: <description>` or `[wasm] cli: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
- `./wasm/test.sh` currently fails on `parallel::tests::test_check_redux_lodash_style_generics` (left 6, right 0).
- Proposed next tasks for EM assignment:
  - Validate ES5 class downleveling edge cases for `super()` + field initializers in `wasm/src/transforms/class_es5.rs`.
  - Add coverage for async downlevel source-map offsets in `wasm/src/transforms/async_es5.rs`.
