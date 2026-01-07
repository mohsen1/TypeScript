
# Migration Plan: TypeScript Compiler → Rust via WebAssembly (Emitter)

**See [EMITTER_ACHIEVEMENTS.md](./EMITTER_ACHIEVEMENTS.md) for comprehensive summary.**

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

### Goal
100% accurate JS output and Source Maps for supported features.

## Current Status

Note:
**Further baseline improvement requires checker/binder track** (type-checking capabilities).


## Tasks (current focus)

Pick the most doable and impactful

1. Integrate LoweringPass into public API (lib.rs exports) [done]
2. Implement remaining directive handlers (arrow, async, modules) [partial: arrow/async done; AMD/UMD/System wrappers added]
3. Deprecate old API, make transforms required [partial: emit/emitModern now use two-phase pipeline; LoweringPass handles CommonJS auto-detect]


4. Expand transform system to more node types
5. Implement System/AMD/UMD module formats (if needed) [done]
6. Public API integration (export LoweringPass) [done - covered by #1]
7. Deprecate inline transform logic (breaking change)
8. More unit test if there are gaps [added parity tests; auto-detect/export-assignment coverage]
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
