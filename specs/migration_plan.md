# Migration Plan: TypeScript Compiler → Rust via WebAssembly

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

---

# ✅ COMPLETED WORK

## Phase 0: Performance Architecture ✅
- **ThinNode**: 16 bytes/node (13x cache improvement from 208B)
- **Zero-Alloc Scanner**: Atom interning, zero-copy accessors
- **Parallelism**: Rayon-based parallel parsing/binding/checking
- **Lazy Diagnostics**: Deferred string formatting, zero waste
- **Solver Operations**: Pure type logic, structured results

## Phases 1-5: Core Compiler ✅
| Component | Lines | Status |
|-----------|-------|--------|
| Utilities | ~300 | ✅ |
| Scanner | ~2,500 | ✅ |
| Parser (ThinParser) | ~11,300 | ✅ 100% pass rate (4483/4483) |
| Binder (ThinBinder) | ~2,900 | ✅ |
| Type Checker + Solver | ~29,300 | ✅ 99% |

**Total**: ~74,350 Rust LOC | 1015 tests passing

## Solver (specs/SOLVER.md) ✅
Complete implementation in `wasm/src/solver/`:
- Type interning (O(1) equality via TypeId)
- Semantic subtyping with coinductive recursion
- Full type lowering (typeof, keyof, this, conditional, mapped, infer)
- Generic instantiation and constraint-based inference
- Contextual typing, discriminated union narrowing
- Lazy diagnostics with structured args

## Phase 7: Language Service (60%)
- ✅ Go-to-definition, find references, completions, signature help, cross-file navigation
- ⬜ Formatting engine, code fixes/refactorings

---

# 🏛️ ARCHITECTURAL DECISIONS

## Solver-Checker Separation: "Check Fast, Explain Slow"

**Critical principle to avoid the "Fat Controller" anti-pattern.**

### The Problem: Boolean Blindness
If `thin_checker.rs` has to reverse-engineer *why* a type check failed, it becomes unmaintainable spaghetti:
```rust
// ❌ BAD: Checker guessing what went wrong
if !self.is_assignable_to(init_type, declared_type) {
    // Checker has to inspect types manually to find the missing property!
    self.error(node, "Type X is not assignable to Y", ...);
}
```

### The Solution: Re-entrant Error Elaboration
Adopt TypeScript's own pattern: **check silently, explain on demand.**

1. **Fast Path (The Judge)**: `is_subtype_of(A, B) -> bool`
   - Fast, cached, silent
   - Used 99% of the time

2. **Slow Path (The Detective)**: `explain_subtype_failure(A, B) -> Diagnostic`
   - Slow, uncached, verbose
   - Called *only* when Fast Path returns `false` and we need to report

### Implementation Pattern
```rust
// ✅ GOOD: thin_checker.rs stays dumb
if !self.solver.is_assignable_to(source, target) {
    // Ask the solver for the "Why"
    let diagnostic = self.solver.explain_assignability_error(source, target);
    self.report_diagnostic(node, diagnostic);
}
```

```rust
// solver/subtype.rs - Explain API
pub fn explain_failure(&self, sub: TypeId, sup: TypeId) -> PendingDiagnostic {
    match (self.peek(sub), self.peek(sup)) {
        (TypeKey::Object(s_props), TypeKey::Object(t_props)) => {
            // Re-run object logic to find EXACT missing property
            for t_prop in t_props {
                if !s_props.contains(t_prop.name) {
                    return PendingDiagnostic::new(
                        code::PROPERTY_MISSING,
                        vec![arg(t_prop.name), arg(sub), arg(sup)]
                    );
                }
            }
            // ... recurse into property types ...
        }
        // ... handle unions, functions, etc.
    }
    // Fallback generic error
    PendingDiagnostic::new(code::TYPE_NOT_ASSIGNABLE, vec![arg(sub), arg(sup)])
}
```

### Rules
1. **`thin_checker.rs` only traverses AST and calls solver** - no type inspection logic
2. **`solver/` owns all type reasoning** - including explaining failures
3. **Use `PendingDiagnostic`** - solver creates structured data, checker renders strings

### Key Files
| Purpose | Location |
|---------|----------|
| Checker (AST traversal only) | `wasm/src/thin_checker.rs` |
| Solver (type logic + explain) | `wasm/src/solver/` |
| Explain API | `wasm/src/solver/subtype.rs` (`explain_failure()`, `SubtypeFailureReason`) |
| Diagnostic structures | `wasm/src/solver/diagnostics.rs` |

---

# 🎯 CURRENT FOCUS: Parallel Tracks

## Track A: Phase 8 - Baseline Compatibility

**Goal**: Match TypeScript's test baselines for `tests/cases/compiler`.

### Test Pass Rates
| Category | Pass Rate |
|----------|-----------|
| compiler | 99.9% (6389/6393) |
| conformance | 99.98% (5654/5655) |
| fourslash | 0% (not started) |

### Baseline Comparison (First 100 tests)
| Baseline | Pass Rate | Blockers |
|----------|-----------|----------|
| .errors.txt | **58.4%** (45/77) | Type parameter scoping, parser error recovery |
| .js emit | 0% | Emitter format mismatch |

### Completed
- ✅ Class/function overload validation (2389, 2390, 2391)
- ✅ Parser error code infrastructure
- ✅ Parser semantic errors (1068, 1440) for class members
- ✅ Parameter property validation (2369) in all contexts
- ✅ Function type parameter property checks (2369)
- ✅ Abstract method handling (skip 2391 for abstract)
- ✅ Declare class parsing (skip impl checks for ambient)
- ✅ Numeric method name support (0(), 1(), etc.)
- ✅ Abstract class instantiation check (2511) - all contexts including local scopes
- ✅ Expanded known globals (WeakRef, TypedArrays, Web APIs, etc.)
- ✅ Nested scope symbol lookup (classes/functions in IIFEs/arrow functions)
- ✅ Type reference validation (2304 for undefined types)
- ✅ Export declaration traversal (check exported classes/functions)

### Next Steps
1. ⬜ Type parameter scoping (generic type parameters in scope)
2. ⬜ Parser semantic errors (1128, additional coverage)
2. ✅ Error elaboration ("...because property 'x' has type...")
   - ✅ `explain_failure()` API in `solver/subtype.rs`
   - ✅ `SubtypeFailureReason::to_diagnostic()` for structured error conversion
   - ✅ `error_type_not_assignable_with_reason_at()` in thin_checker.rs
   - ✅ Wired up: variable declarations, return statements, property declarations
3. ⬜ RelatedInformation (point to definition sites)
4. ✅ Scoped name resolution
   - ✅ Local variables added to scope during type checking
   - ✅ Parameters added to scope in functions/methods/constructors
   - ✅ Block scope support (push/pop scope for BLOCK nodes)
   - ✅ For-loop variable scope (FOR_STATEMENT, FOR_IN, FOR_OF)
   - ✅ Abstract class checks in local scopes (binder + checker enhanced)
   - ✅ Symbol lookup fallback for all symbols (handles nested classes)
   - ⬜ Class member resolution (this.x vs x vs ClassName.x)

---

## Track B: Phase 6 - Emitter Completion (75% → 100%)

**Goal**: Complete emitter with performance-first approach.

### Phase 6.1: Study & Exploration (Use Gemini)
Before implementation, analyze for performance opportunities:
- ⬜ **Benchmark current emit** - measure throughput (bytes/sec)
- ⬜ **Profile hot paths** - string building, whitespace, source maps
- ⬜ **Study TypeScript emitter** - identify simplification opportunities
- ⬜ **Gemini review** - ask for emit architecture recommendations
- ⬜ **Explore alternatives**: rope data structures, streaming output, SIMD text processing

### Phase 6.2: Generator Transforms
- ⬜ `function*` syntax and `yield` expressions
- ⬜ State machine generation for ES5 target
- ⬜ Iterator protocol compliance

### Phase 6.3: Output Format Matching
- ⬜ Match TypeScript baseline whitespace/semicolons
- ⬜ Source map accuracy
- ⬜ Declaration file formatting

### Key Files
| Purpose | Location |
|---------|----------|
| ThinEmitter | `wasm/src/thin_emitter.rs` |
| Transforms | `wasm/src/transforms/` |
| Generator transforms | `wasm/src/transforms/async_gen.rs` |

---

# Phase 9: Full Rust Mode ⬜

- ⬜ Remove TypeScript fallbacks
- ⬜ Performance benchmarks vs tsc and tsc-go
- ⬜ Memory usage optimization
- ⬜ WASM interface optimization (binary protocol)

---

# WASM Strategy

## WasmGC (Garbage Collection) 🟢 **Ready Now**
Use `externref` via `wasm-bindgen` for the Language Service API:
```rust
// Instead of JSON serialization:
pub fn get_completions() -> js_sys::Array { ... }
```
- Removes serialization bottleneck at API boundary
- Low risk, high payoff

## Wasm64 (Memory64) 🔴 **Not Ready**
- Experimental flag required in most runtimes
- `wasm-bindgen` friction with 64-bit pointers/BigInt
- **Not needed yet**: ThinNode (16 bytes) fits ~250M nodes in 4GB
- **Decision**: Keep u32 indices, revisit in 12-18 months

---

# Quick Reference

## Commands
```bash
# Tests (Docker)
./wasm/test.sh

# TypeScript
npx hereby runtests-parallel

# Baseline comparison
node scripts/baseline-test-rust.mjs
```

## Key Files
| Purpose | Location |
|---------|----------|
| This plan | specs/migration_plan.md |
| Session log | [specs/SESSION_LOG.md](SESSION_LOG.md) |
| Solver spec | specs/SOLVER.md |
| Unsoundness rules | specs/TS_UNSOUNDNESS_CATALOG.md |
| Architecture | specs/WASM_ARCHITECTURE.md |
| Solver code | wasm/src/solver/ |
| ThinParser | wasm/src/parser/thin_parser.rs |
| ThinChecker | wasm/src/checker/thin_checker.rs |
| Baseline comparison | scripts/baseline-test-rust.mjs |
