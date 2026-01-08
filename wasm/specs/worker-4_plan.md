# Worker 4 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 4

## Current Assignment
- Stand by for next emitter fidelity task.

## Task Queue
- [ ] Stand by for next emitter fidelity task.
- [x] TODO (was blocked by `cli/driver.rs` E0515 in Docker): re-apply stash "wip async nested super" and run `./wasm/test.sh class_async_nested_arrow_super_computed_key_this_arguments_capture`.
- [ ] TODO: add async arrow with computed super key + this + arguments in nested return.
- [x] TODO: add async method arrow with computed super key + this + arguments (non-nested). Expected: no `=>`, `_this` capture, `arguments[0]` preserved, computed super likely left as `super[key]` (current behavior).
- [ ] TODO: add async nested arrow with computed super key + arguments only. Expected: no `=>`, `arguments[0]` preserved, computed super likely left as `super[key]` (current behavior).
- [ ] TODO: add async method returning arrow with `super.m(arguments[0])` (non-computed). Expected: no `=>`, `_super.prototype.m.call(_this, arguments[0])` (if return-arrow lowering matches other arrow cases).

## Completed
- [x] ES5 async class computed super with this + arguments (current behavior); test `./wasm/test.sh class_async_super_computed_method_this_arguments`.
- [x] ES5 async class nested arrow computed super + this capture (current behavior); test `./wasm/test.sh class_async_nested_arrow_super_computed_this_capture`.
- [x] ES5 async class nested arrow computed super + this + arguments capture (current behavior); test `./wasm/test.sh class_async_nested_arrow_super_computed_this_arguments_capture`.
- [x] ES5 async class nested arrow computed super + arguments capture (current behavior); test `./wasm/test.sh class_async_nested_arrow_super_computed_arguments`.
- [x] ES5 async class nested arrow computed super (current behavior); test `./wasm/test.sh class_async_nested_arrow_super_computed`.
- [x] ES5 async class method returns arrow with computed super + arguments (current behavior); test `./wasm/test.sh class_async_return_arrow_super_computed_arguments`.
- [x] ES5 async class method returns arrow with computed super + this + arguments (current behavior); test `./wasm/test.sh class_async_return_arrow_super_computed_this_arguments_capture`.
- [x] ES5 async class method returns arrow with computed super key + this + arguments (current behavior, dropped body); test `./wasm/test.sh class_async_return_arrow_super_computed_key_this_arguments_capture`.
- [x] ES5 async class method returns arrow with computed super key + arguments (current behavior, dropped body); test `./wasm/test.sh class_async_return_arrow_super_computed_key_arguments_capture`.
- [x] ES5 async class nested arrow super + this + arguments capture; test `./wasm/test.sh class_async_nested_arrow_super_this_arguments_capture`.
- [x] ES5 async class method arrow uses computed super key + arguments (current behavior); test `./wasm/test.sh class_async_arrow_super_computed_key_arguments_capture`.
- [x] ES5 async class method arrow uses computed super key + this capture (current behavior); test `./wasm/test.sh class_async_arrow_super_computed_key_this_capture`.
- [x] ES5 async class method arrow uses computed super key + this + arguments (current behavior); test `./wasm/test.sh class_async_arrow_super_computed_key_this_arguments_capture`.
- [x] ES5 async class nested arrow uses computed super key + this + arguments (current behavior); test `./wasm/test.sh class_async_nested_arrow_super_computed_key_this_arguments_capture`.
- [x] ES5 async class nested arrow super + this capture; test `./wasm/test.sh class_async_nested_arrow_super_this_capture`.
- [x] ES5 async class nested arrow super + arguments capture; test `./wasm/test.sh class_async_nested_arrow_super_arguments`.
- [x] ES5 async class nested arrow super call without args; test `./wasm/test.sh class_async_nested_arrow_super_call_no_args`.
- [x] ES5 async class method arrow super + this + arguments capture; test `./wasm/test.sh class_async_arrow_super_this_arguments_capture`.
- [x] ES5 async class method arrow super + arguments capture; test `./wasm/test.sh class_async_arrow_super_arguments_capture`.
- [x] ES5 async class method arrow super call without args; test `./wasm/test.sh class_async_arrow_super_call_no_args`.
- [x] ES5 async class method arrow computed super call without args (current behavior); test `./wasm/test.sh class_async_arrow_super_computed_call_no_args`.
- [x] ES5 async class method arrow super + this capture; test `./wasm/test.sh class_async_arrow_super_this_capture`.
- [x] ES5 class method arrow super + arguments capture; test `./wasm/test.sh class_method_arrow_super_arguments_capture`.
- [x] ES5 class method nested arrow super + this capture; test `./wasm/test.sh class_method_nested_arrow_super_this_capture`.
- [x] ES5 async class method arrow arguments capture; test `./wasm/test.sh class_async_arrow_arguments_capture`.
- [x] ES5 class method arrow super + this capture; test `./wasm/test.sh class_method_arrow_super_this_capture`.
- [x] ES5 async computed super method call (current async emitter behavior); test `./wasm/test.sh async_super_computed_method_arguments`.
- [x] ES5 computed super method call in class method; test `./wasm/test.sh super_computed_method_arguments`.
- [x] ES5 derived field arrow computed super call lowered; test `./wasm/test.sh derived_field_arrow_super_computed`.
- [x] ES5 derived field arrow handles super + this; test `./wasm/test.sh derived_field_arrow_super_and_this`.
- [x] ES5 ctor arrow lowers super call with lexical this; test `./wasm/test.sh ctor_arrow_super_call`.
- [x] ES5 derived field arrow lowers super call with lexical this; test `./wasm/test.sh derived_field_arrow_super_call`.
- [x] ES5 static field arrow does not capture instance this; test `./wasm/test.sh class_static_field_no_this_capture`.
- [x] ES5 class field multiple arrow this capture; test `./wasm/test.sh class_field_multi_arrow_this_capture`.
- [x] ES5 derived default ctor arrow field capture; test `./wasm/test.sh derived_default_arrow_field_capture`.
- [x] ES5 class field nested arrow this capture emits constructor _this; test `./wasm/test.sh class_field_nested_arrow_this_capture`.
- [x] ES5 class method nested arrow captures `this` + arguments; test `./wasm/test.sh class_method_nested_arrow_arguments`.
- [x] ES5 async nested arrow super call lowered; test `./wasm/test.sh async_super_nested_arrow`.
- [x] ES5 derived async class with arrow field initializer capture; test `./wasm/test.sh async_derived_prop_arrow`.
- [x] ES5 derived class property initializer emitted after super; test `./wasm/test.sh derived_prop_init_after_super`.
- [x] ES5 async derived method super call + arrow this capture; test `./wasm/test.sh async_super_this_capture`.
- [x] ES5 class derived ctor nested arrow this capture after super; test `./wasm/test.sh super_nested_arrow_this_capture`.
- [x] ES5 class derived ctor arrow this capture after super; test `./wasm/test.sh super_arrow_this_capture`.
- [x] ES5 class super method regression with arguments; test `./wasm/test.sh super_method_arguments`.
- [x] Async ES5 nested arrow regression with this + arguments usage; test `./wasm/test.sh nested_arrow_arguments_capture`.
- [x] Async ES5 deep nested arrow regression (async + nested arrow this capture); test `./wasm/test.sh deep_nested_arrow_this_capture`.
- [x] Async ES5 let declaration regression test for arrow this capture; test `./wasm/test.sh let_arrow_this_capture`.
- [x] Async ES5 multi-declarator var statement regression test (await + arrow this capture); test `./wasm/test.sh multi_decl_this_capture`.
- [x] Async ES5 variable statement emission: handle declaration lists so const/let initializers emit in async bodies; test `./wasm/test.sh nested_arrow_this_capture`.
- [x] Async ES5 parity investigation: parity test passed; fixed nested arrow `this` capture in async ES5 emission; tests `./wasm/test.sh test_parity_async_es5`, `./wasm/test.sh nested_arrow`.
- [x] Async ES5 await detection traversal: include property/element/conditional expressions; tests `./wasm/test.sh async_es5`.
- [x] ES5 downleveling edge cases: fixed default-constructor arrow `this` capture and preserved pre-super statements with property initializer ordering; tests `./wasm/test.sh class_es5`.
- [x] ES5 class parity gap: legacy emitter now downlevels classes when targeting ES5; test `./wasm/test.sh test_parity_es5_class`.
- [x] Re-verified async ES5 parity after var-statement fix; test `./wasm/test.sh test_parity_async_es5`.
- [x] ES5 class async method parity: emit __awaiter wrapper in class methods; test `./wasm/test.sh class_es5`.
- [x] ES5 class edge case: preserve pre-super statements before initializer emission in derived constructors; test `./wasm/test.sh class_es5`.
- [x] Emitter extends helper edge case: route test through LoweringPass transforms; test `./wasm/test.sh test_class_extends_helper`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
- `cli/driver.rs` E0515 blocker resolved; async nested super tests re-run.
- Ready for merge.
