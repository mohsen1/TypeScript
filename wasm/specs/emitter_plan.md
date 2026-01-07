# Emitter Track Plan (Transforms + Printer)

## Mission
Correct JS output and source maps with a transform-first pipeline that keeps the printer dumb and fast.

## Scope
Files: `wasm/src/thin_emitter/*`, `wasm/src/transforms/*`, `wasm/src/lowering_pass.rs`, `wasm/src/transform_context.rs`, `wasm/src/source_map.rs`, `wasm/src/declaration_emitter.rs`.

## Current Status
- LoweringPass -> TransformDirective -> ThinPrinter pipeline exists.
- ES5/ESNext transforms largely implemented.
- Remaining TODOs: call/construct signature type parameters, class heritage plumbing, lingering inline ES5 paths.

## Highest-Impact Next Tasks
- [ ] Finish transform-only pipeline
  - Remove or gate `ctx.target_es5` inline paths in `thin_emitter/mod.rs`.
  - Ensure LoweringPass emits directives for all ES5 transforms (class, arrow, async, template, params, object literal).
- [ ] Plumb class heritage in LoweringPass
  - Fill `TransformDirective::ES5Class.heritage` or drop unused field.
  - Add regression tests for `extends` + private fields + helper injection.
- [ ] Emit type parameters for call/construct signatures
  - Implement in `emit_call_signature` and `emit_construct_signature`.
  - Add `.d.ts` tests for generic interface signatures.
- [ ] Validate module wrapper + export transforms
  - Add parity tests for CommonJS/AMD/UMD wrappers and re-exports.
- [ ] Performance check
  - Run `./wasm/bench.sh real_world_bench` and track throughput deltas.

## Success Criteria
- All transforms triggered via TransformContext (no inline ES5 fallbacks).
- Source maps stable for transformed outputs.
- Emitter throughput >= 500 MB/s on real-world bench.
