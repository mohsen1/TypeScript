# Emitter Track Plan (Transforms + Printer)

## Mission
Correct JS output and source maps with a transform-first pipeline that keeps the printer dumb and fast.

## Scope
Files: `wasm/src/thin_emitter/*`, `wasm/src/transforms/*`, `wasm/src/lowering_pass.rs`, `wasm/src/transform_context.rs`, `wasm/src/source_map.rs`, `wasm/src/declaration_emitter.rs`.

## Current Status
- LoweringPass -> TransformDirective -> ThinPrinter pipeline exists.
- ES5/ESNext transforms largely implemented.
- Remaining TODOs: push real_world_bench throughput toward 500+ MiB/s (emit-only ~318 MiB/s).

## Highest-Impact Next Tasks
- [ ] Performance tuning: reach 500+ MiB/s emitter throughput
  - [x] Pre-allocate output buffer from source size; write numeric indices without allocations.
  - [ ] Profile emit-only pipeline (LoweringPass + helper detection).
  - [ ] Reduce allocations and repeated scans in emit hot paths.
- [x] Finish transform-only pipeline
  - [x] Remove or gate `ctx.target_es5` inline paths in `thin_emitter/mod.rs`.
  - [x] Ensure LoweringPass emits directives for all ES5 transforms (class, arrow, async, template, params, object literal).
- [x] Plumb class heritage in LoweringPass
  - Fill `TransformDirective::ES5Class.heritage` or drop unused field.
  - Add regression tests for `extends` + private fields + helper injection.
- [x] Emit type parameters for call/construct signatures
  - Implement in `emit_call_signature` and `emit_construct_signature`.
  - Add `.d.ts` tests for generic interface signatures.
- [x] Validate module wrapper + export transforms
  - Add parity tests for AMD/UMD/System wrappers and re-exports.
- [x] Performance check
  - Run `./wasm/bench.sh real_world_bench` and track throughput deltas.
  - Results (real_world_bench):
    - checker_ts_full_pipeline thrpt: 59.830–61.057 MiB/s (change within noise).
    - checker_ts_emit_only thrpt: 318.17–319.40 MiB/s (change within noise).

## Success Criteria
- All transforms triggered via TransformContext (no inline ES5 fallbacks).
- Source maps stable for transformed outputs.
- Emitter throughput >= 500 MB/s on real-world bench.

## Task Ledger (legacy checklist)

Pick the most doable and impactful

Next Steps: Cleanup. Remove the legacy inline transformation logic from ThinPrinter to rely strictly on TransformDirective.

1. Integrate LoweringPass into public API (lib.rs exports) [done]
2. Implement remaining directive handlers (arrow, async, modules) [partial: arrow this-capture (incl. nested) + async arrow downlevel handled; AMD/UMD/System wrappers auto-lower in legacy ThinPrinter]
3. Deprecate old API, make transforms required [partial: emit/emitModern now use two-phase pipeline; ThinPrinter auto-runs LoweringPass for source files]

4. Expand transform system to more node types [partial: ES5 object literal computed/spread directive]
5. Implement System/AMD/UMD module formats (if needed) [done]
6. Public API integration (export LoweringPass) [done - covered by #1]
7. Deprecate inline transform logic (breaking change) [done: removed inline ES5 class/enum/namespace/arrow/for-of/async/object literal computed/spread paths; default anonymous export now driven by directives (ES5 default class directive + chain); ES5 var keyword via ES5VariableDeclarationList]
8. More unit test if there are gaps [added parity tests; auto-detect/export-assignment coverage; export default arrow; arrow this in object literals/await/type assertions/satisfies/tagged templates; ES5 template literal/tagged template downlevel; ES5 class template/tagged template downlevel; ES5 object literal shorthand/method downlevel; ES5 template literal directive coverage; ES5 variable destructuring directive coverage; ES5 function parameter directive coverage; ES5 CommonJS default class directive; ES5 variable list uses var]
9. Emit ES6 class heritage clauses (extends) [done]
10. Class ES5 transform: emit try/throw statements and parenthesized expressions [done]
11. CommonJS export star (export * from) support [done]
12. CommonJS export names: handle binding patterns in destructuring exports [done]
13. Class ES5 transform: destructured parameters emit assignments [done]
14. Declaration emitter: emit heritage clauses (extends/implements) [done]
15. Class ES5 transform: destructure for-loop initializers [done]
16. Class ES5 transform: emit switch/break/continue/do statements [done]
17. ThinEmitter: emit case blocks for switch statements [done]
18. Class ES5 transform: emit for-in/for-of statements [done]
19. Class ES5 transform: emit binding patterns in declarations [done]
20. ES5 emit: downlevel for-of loops with __values helper [done]
21. ES5 emit: close for-of iterators with try/finally [done]
22. Expand transform system: ES5 for-of directive [done]
23. ES5 emit: object/array rest destructuring with __rest helper [done]
24. ES5 emit: downlevel rest parameters in functions/methods [done]
25. ES5 emit: downlevel default parameters in functions/methods [done]
26. ES5 emit: handle destructuring defaults/nested patterns in ES5 bindings [done]
27. ES5 emit: downlevel object spread in object literals [done]
28. Class ES5 emit: downlevel object literal computed/spread properties [done]
29. ThinEmitter: apply TransformDirective::Chain composition [done]
30. LoweringPass: compose ES5/CommonJS transforms with Chain [done]
31. LoweringPass: handle export-declared functions and async detection [done]
32. Export declarations: CommonJS transforms for vars and transform-aware emission [done]
33. Export declarations: handle default anonymous function/class + ES6 default emit [done]
34. CommonJS preamble: include default/named export declarations in exports init [done]
35. CommonJS export declarations: enum transform + exports assignment [done]
36. CommonJS export declarations: namespace transform + exports assignment [done]
37. Enum modifiers: attach const/declare in parser; erase const/declare enum emit + CommonJS exports [done]
38. Imports/exports: handle namespace imports and erase type-only import/export in JS output [done]
39. CommonJS: emit side-effect imports as require() [done]
40. Module wrappers: skip type-only import/export dependencies [done]
41. Module auto-detect: ignore type-only/ambient exports for wrapper/preamble [done]
42. Import equals: emit JS and ignore internal aliases in module auto-detect [done]
43. CommonJS preamble: include export import equals in exports init [done]
44. ES5 emit: downlevel template literals + tagged templates with __makeTemplateObject helper [done]
45. ES5 emit: include __awaiter/__generator helpers when async downleveling [done]
46. ES5 class emitter: downlevel template literals + tagged templates [done]
47. ES5 object literal emit: downlevel shorthand properties + methods [done]
48. ES5 emit: move template literal downleveling behind TransformDirective [done]
49. ES5 emit: move variable destructuring downleveling behind TransformDirective [done]
50. ES5 emit: move function parameter downleveling behind TransformDirective [done]
51. LoweringPass: gate export name extraction for non-exported declarations [done]
52. CommonJS exports: store identifier ids in directives to avoid name cloning [done]
53. Class ES5 emitter: write identifier/literal names without cloning strings [done]
54. ThinEmitter: avoid identifier string clones in ES5 binding/object literal emit [done]

## Quick Reference

```bash
# Tests (Docker)
./wasm/test.sh

# Baseline comparison
node scripts/baseline-test-rust.mjs

# Build WASM
./wasm/build-wasm.sh

# Real-world benchmarks
./wasm/bench.sh real_world_bench
```
