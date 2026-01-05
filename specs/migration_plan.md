# Migration Plan: TypeScript Compiler → Rust via WebAssembly

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

## Guiding Principles

1. **Never Break the Build** – Every commit must pass `hereby runtests-parallel`.
2. **Iterate in Small Slices** – One function, one module at a time.
3. **Test Before & After** – Existing test suite is the source of truth.
4. **Performance First** – We are building for speed.
5. **Use Gemini** – For code reviews at milestones.

---

# ✅ COMPLETED PHASES (Summary)

## Phase 0: Performance-First Architecture ✅

| Component | Status | Key Achievement |
|-----------|--------|-----------------|
| **ThinNode (0.1)** | ✅ Done | 16 bytes/node (13x improvement from 208B), 60+ typed data pools |
| **Zero-Alloc Scanner (0.2)** | ✅ Done | Atom interning (u32), zero-copy accessors |
| **Arena Type Analysis (0.3)** | ✅ Analyzed | Type enum = 48 bytes (already optimized) |
| **Parallelism (0.4)** | ✅ Done | Rayon-based parallel parsing/binding/checking |

## Phase 1-5: Core Compiler ✅

| Phase | Component | Lines | Tests | Status |
|-------|-----------|-------|-------|--------|
| 1 | Utilities | ~300 | 21 | ✅ Done |
| 2 | Scanner | ~2,500 | 22 | ✅ Done |
| 3 | Parser (ThinParser) | ~11,300 | 160+ | ✅ Done |
| 4 | Binder (ThinBinder) | ~2,900 | 26+ | ✅ Done |
| 5 | Type Checker | ~23,500 | 485 | ✅ 99% |

**Total Rust Code**: ~59,500 lines | **Total Tests**: 993 passing

### ThinParser Capabilities (Complete)
- All expressions, statements, declarations
- Full type syntax: unions, intersections, generics, conditional, mapped, indexed access
- JSX support, async/await, generators
- Import/export (ES6, CommonJS, type-only)
- 96.9% pass rate on batch tests (single-file, 800/825)

### Parser Features Added (Sessions 22-30)
- Generic function types, type assertions, template literals
- Tuple rest/optional/named elements, constructor types
- All compound assignment operators, nullish coalescing
- Binding patterns, index signatures, labeled statements
- Recursion depth limits, tagged template literals
- 'as const' assertions, multi-declaration for loops
- Comma expressions, generic function calls with type arguments
- 'this' type, 'declare abstract class', regex literals
- Dynamic import expressions (import(...) and import.meta)

---

# 🎯 CURRENT FOCUS: Phase 7.5 - Semantic Structural Solver

**Goal**: Build a mathematically correct, high-performance type system based on **Semantic Subtyping** (see `specs/SOLVER.md` for theoretical foundations).

## Architecture Overview

```
ThinParser → ThinNodeArena → ThinBinder → ThinChecker → TypeInterner → TypeId
                                              ↓
                                     solver::TypeInterner
                                     solver::SubtypeChecker
                                     solver::TypeLowering
                                     solver::InferenceContext
```

## Current State

### ✅ Implemented (`wasm/src/solver/`)

| Module | Lines | Tests | Status |
|--------|-------|-------|--------|
| `types.rs` | ~250 | 5+ | ✅ Complete |
| `intern.rs` | ~260 | 5+ | ✅ Complete |
| `lower.rs` | ~510 | 5+ | ✅ Core done |
| `subtype.rs` | ~420 | 5+ | ✅ Core done |
| `infer.rs` | ~185 | 5+ | ✅ Basic done |

### ✅ What's Working

1. **TypeId Interning** (O(1) equality)
   - Compile-time constant TypeIds for intrinsics (TypeId::NUMBER = 9)
   - Structural deduplication (same structure = same TypeId)
   - Automatic union/intersection normalization

2. **Type Lowering** (AST → TypeId)
   - Keyword types, literal types, identifiers
   - Union/intersection types, array/tuple types
   - Function types, type literals (objects)
   - Conditional types, mapped types, indexed access

3. **Subtype Checking**
   - Intrinsic subtyping
   - Literal to intrinsic promotion
   - Union/intersection logic
   - Object structural subtyping
   - Function subtyping (covariant return, bivariant params)
   - **Coinductive cycle detection** (recursive types)

4. **Inference** (Basic)
   - Inference variables via `ena` Union-Find
   - Type parameter binding
   - Basic unification

---

## 🔴 TODO: Complete the Solver

Reference: `specs/SOLVER.md` for theoretical foundations.

### Priority 1: Full Type Lowering

| Task | Section in SOLVER.md | Status |
|------|---------------------|--------|
| Type parameters in generics | §5.1 | ⬜ |
| `this` type resolution | §4.4 | ✅ Done (Session 28) |
| `typeof` in type position | §4.4 | ✅ Done (Session 28) |
| `keyof` type operator | §4.4 | ✅ Done (Session 28) |
| `readonly` type modifier | §4.4 | ✅ Done (Session 28) |
| `unique symbol` type | §4.4 | ✅ Done (Session 28) |
| Template literal types | §4.5 | ✅ Done (Session 28) |
| Named tuple elements | §2.2 | ✅ Done (Session 28) |
| Rest/optional elements in tuples | §2.2 | ✅ Done (Session 28) |
| Constructor types | §3.4 | ✅ Done (Session 28) |
| `infer` types | §4.1 | ✅ Done (Session 28) |

### Priority 2: Advanced Subtyping ✅ COMPLETE

| Task | Section in SOLVER.md | Status |
|------|---------------------|--------|
| **Ref resolution** - resolve TypeKey::Ref to structural type | §2.4 | ✅ Done (Session 29) |
| Generic instantiation | §5.5 | ✅ Done (Session 29) |
| Discriminated union narrowing | §3.3 | ✅ Done (Session 29) |
| Index signature matching | §3.2 | ✅ Done (Session 29) |
| Call signature overloads | §3.4 | ✅ Done (Session 29) |

### Priority 3: Meta-Type Resolution

These are TypeScript's "type-level functions" (see §4 of SOLVER.md).

| Task | Section in SOLVER.md | Status |
|------|---------------------|--------|
| **Conditional type evaluation** | §4.1 | ✅ Done (Session 29) |
| **Distributive conditional types** | §4.2 | ✅ Done (Session 29) |
| **Mapped type instantiation** | §4.3 | ✅ Done (Session 29) |
| **Index access type resolution** | §4.4 | ✅ Done (Session 29) |
| **keyof type evaluation** | §4.4 | ✅ Done (Session 29) |
| Lazy vs eager evaluation strategy | §4.5 | ✅ Done (via deferred types) |

### Priority 4: Full Inference ✅ COMPLETE

| Task | Section in SOLVER.md | Status |
|------|---------------------|--------|
| **Generic instantiation/substitution** | §5.5 | ✅ Done (Session 29) |
| **Constraint collection** | §5.2 | ✅ Done (Session 29) |
| **Bounds checking** (L <: α <: U) | §5.3 | ✅ Done (Session 29) |
| **Contextual typing** (reverse inference) | §5.4 | ✅ Done (Session 29) |
| Best common type calculation | §5.2 | ✅ Done (Session 29) |

### Priority 5: Error Handling & Polish ✅ COMPLETE

| Task | Section in SOLVER.md | Status |
|------|---------------------|--------|
| Error type propagation ("poison pill") | §6.4 | ✅ Basic |
| Diagnostic generation | - | ✅ Done (Session 29) |
| Source location tracking | - | ✅ Done (Session 29) |

---

## Key Design Decisions (from SOLVER.md)

### Types as Sets (§1)
- $T = \{ v \in \mathcal{U} \mid \text{predicate}_T(v) \text{ is true} \}$
- `unknown` = $\top$ (top type), `never` = $\bot$ (bottom type)
- Subtyping = Set Inclusion: $S <: T \iff S \subseteq T$

### Coinduction for Recursive Types (§1.4)
- Greatest Fixed Point semantics
- "If we traverse a cycle without finding a contradiction, return TRUE"
- Implemented via `in_progress` set in SubtypeChecker ✅

### Union/Intersection Normalization (§1.3)
- Unions: flatten, sort, dedupe, remove `never`, absorb `any`/`unknown`
- Intersections: flatten, sort, dedupe, absorb `never`
- Implemented in TypeInterner::union/intersection ✅

### Lazy Evaluation (§4.5)
- Store the "recipe" (TypeKey::Conditional, TypeKey::Mapped)
- Only evaluate when subtype check needs the result
- Prevents infinite loops in recursive type definitions

---

## Implementation Order

```
1. Type Lowering Gaps
   └─ typeof, keyof, this, template literals

2. Ref Resolution
   └─ TypeKey::Ref → structural expansion (lazy)

3. Conditional Type Evaluation
   └─ check_type <: extends_type ? true_branch : false_branch
   └─ Distributivity: (A | B) extends U ? ... distributes

4. Mapped Type Instantiation
   └─ { [K in keyof T]: Transform<T[K]> }

5. Full Inference Pipeline
   └─ Constraint collection → Bounds checking → Resolution
   └─ Contextual typing for arrow functions

6. Integration with ThinChecker
   └─ Wire solver into expression checking
   └─ Connect to diagnostics
```

---

# Phase 6: Emitter (75% Complete)

| Feature | Status |
|---------|--------|
| Declaration file emission | ✅ |
| ES2015+ transforms (arrow → function) | ✅ |
| CommonJS import/export rewriting | ✅ |
| Async/await transforms | ✅ |
| Generator transforms | 🟡 Partial |

# Phase 7: Language Service (60% Complete)

| Feature | Status |
|---------|--------|
| Go-to-definition | ✅ |
| Find references | ✅ |
| Completions (global, member, type) | ✅ |
| Signature help | ✅ |
| Cross-file navigation | ✅ |
| Formatting engine | ⬜ |
| Code fixes/refactorings | ⬜ |

# Phase 7.6: Cleanup

| Task | Status |
|------|--------|
| Separate test files | ✅ All use `#[path = "..._tests.rs"]` |
| Fix Rust warnings | ✅ 67 → 15 (remaining are dead_code) |
| UB transmute investigation | ✅ Safe (validates range) |
| Remove legacy Node/NodeArena | 🚫 Blocked by transforms |
| Code quality tooling | ⬜ |
| Update architecture docs | ⬜ |

---

# Phase 8: Running `tests/cases` 🎯 NEXT GOAL

**Goal: Every test case in `tests/cases` compiles faster than TypeScript-Go.**

## Current Progress

| Category | Total | Passing | % |
|----------|-------|---------|---|
| compiler | 6,397 | ~679 | ~10% |
| conformance | 5,691 | TBD | 0% |
| fourslash | 6,563 | TBD | 0% |

**Batch Test Results** (single-file compiler tests):
- Pass Rate: **82.3%** (679/825)
- 0 crashes
- ~107 remaining failures (multi-file tests, intentional errors)

## Blockers for Higher Pass Rate

1. **Multi-file tests** with `@filename:` directives (~50 files)
2. **Intentional syntax errors** (malformed test cases)
3. **Legacy syntax** (anonymous modules, multiple extends)

## Path to 100%

1. Complete solver (Priority 1-4 above)
2. Connect solver to ThinChecker for type inference
3. Generate diagnostics matching TypeScript baselines
4. Compare output: `.errors.txt`, `.types`, `.js` files

---

# Phase 9: Full Rust Mode ⬜

- [ ] Remove TypeScript fallbacks
- [ ] Performance optimization pass
- [ ] Memory usage optimization
- [ ] WASM interface optimization (binary protocol)

---

# Quick Reference

## Commands

```bash
# Tests (MUST use Docker)
./wasm/test.sh                    # All tests
./wasm/test.sh <test_name>        # Specific test

# Never run cargo directly (60GB+ RAM explosion)

# TypeScript integration
npx hereby runtests-parallel      # Full test suite

# Gemini reviews (at milestones)
node scripts/ask-gemini.mjs --review
```

## Key Files

| Purpose | Location |
|---------|----------|
| Migration plan | `specs/migration_plan.md` |
| Solver theory | `specs/SOLVER.md` |
| Solver implementation | `wasm/src/solver/` |
| ThinParser | `wasm/src/parser/thin_parser.rs` |
| ThinChecker | `wasm/src/checker/thin_checker.rs` |
| Test runner | `scripts/batch-test-rust.mjs` |

---

# Session Log (Recent)

## 2026-01-05: Session 29
- **Ref Resolution Complete**
- Added TypeResolver trait for lazy symbol-to-type resolution
- Added TypeEnvironment for pre-populated type mappings
- Updated SubtypeChecker to be generic over `R: TypeResolver`
- Added `with_resolver()` constructor for SubtypeChecker
- Added env-aware methods to ThinChecker:
  - `is_assignable_to_with_env()`, `is_subtype_of_with_env()`
  - `build_type_environment()` - populates env from binder symbols
- Added 5 new tests for Ref resolution

- **Generic Instantiation Complete**
- Added new `solver/instantiate.rs` module
- `TypeSubstitution` - maps type parameter names to concrete TypeIds
- `TypeInstantiator` - recursive type traversal with cycle detection
- `instantiate_type()` and `instantiate_generic()` convenience functions
- Deep substitution through all TypeKey variants
- Added 12 new tests for instantiation

- **Meta-Type Evaluation Complete**
- Added new `solver/evaluate.rs` module
- `TypeEvaluator` - evaluates conditional types, index access types
- Conditional type evaluation with true/false branch resolution
- Distributive conditional types over unions
- Index access type resolution for objects, arrays, and tuples
- Deferred evaluation for unresolved type parameters
- Added 15 new tests for evaluation

- **Mapped Types & Keyof Complete**
- `evaluate_mapped()` - instantiates mapped types to concrete objects
- `evaluate_keyof()` - extracts keys from objects, tuples, arrays
- Supports modifiers (+readonly, +optional, -readonly, -optional)
- Template substitution with type parameter replacement
- keyof union = intersection of keys, keyof intersection = union of keys
- Added 12 new tests for mapped types and keyof

- **Constraint-Based Inference Complete**
- Extended `InferenceContext` with `ConstraintSet` (lower/upper bounds)
- `add_lower_bound()`, `add_upper_bound()` for constraint collection
- `resolve_with_constraints()` - resolves vars using bounds
- `best_common_type()` - union of lower bounds (widening)
- Bounds validation: ensures L <: result <: U
- `resolve_all_with_constraints()` - batch resolution
- Added 15 new tests for constraint system

- **Contextual Typing Complete**
- Added new `solver/contextual.rs` module
- `ContextualTypeContext` - holds expected type for reverse inference
- Parameter type inference from function context
- Return type inference from function context
- Array/tuple element type inference
- Object property type inference
- Union type handling for contextual types
- Nested context support (`for_property`, `for_parameter`, etc.)
- Added 16 new tests for contextual typing
- 936 tests passing
- **Priority 4 (Full Inference) COMPLETE!**

- **Discriminated Union Narrowing Complete**
- Added new `solver/narrowing.rs` module
- `DiscriminantInfo` - describes discriminant property with variants
- `NarrowingContext` - context for type narrowing operations
- `find_discriminants()` - detects discriminant properties in unions
- `narrow_by_discriminant()` - narrows union by matching discriminant value
- `narrow_by_excluding_discriminant()` - narrows by excluding discriminant
- `narrow_by_typeof()` - narrows by typeof check (string, number, etc.)
- `narrow_to_type()` / `narrow_excluding_type()` - general type narrowing
- Added 15 new tests for narrowing
- 951 tests passing

- **Index Signature Matching Complete**
- Added `IndexSignature` struct for `{ [key: T]: V }` patterns
- Added `ObjectShape` struct for objects with index signatures
- Added `ObjectWithIndex` variant to `TypeKey` enum
- Added `object_with_index()` method to TypeInterner
- Implemented subtype checking for:
  - String index to string index (covariant value types)
  - Number index to number index
  - Simple object to indexed object (property values must match index)
  - Indexed object to simple object (named properties checked)
  - Mixed string/number index signatures
- Updated TypeInstantiator for ObjectWithIndex
- Added 10 new tests for index signature matching
- 961 tests passing

- **Call Signature Overloads Complete**
- Added `CallSignature` struct for individual call/construct signatures
- Added `CallableShape` struct with call_signatures, construct_signatures, properties
- Added `Callable` variant to `TypeKey` enum
- Added `callable()` method to TypeInterner
- Implemented subtype checking for:
  - Callable to callable (for each target sig, find matching source sig)
  - Function to callable (single sig must match all targets)
  - Callable to function (at least one sig must match)
  - Covariant return types, bivariant parameters
  - Properties on callable types
- Updated TypeInstantiator with `instantiate_call_signature()` helper
- Added 9 new tests for callable subtyping
- 970 tests passing
- **Priority 2 (Advanced Subtyping) COMPLETE!**

- **ThinChecker Narrowing Integration**
- Added narrowing methods to ThinChecker using solver's NarrowingContext:
  - `narrow_by_typeof()` - typeof guards
  - `narrow_by_typeof_negation()` - negated typeof guards
  - `narrow_by_discriminant()` - discriminated union narrowing
  - `narrow_by_excluding_discriminant()` - excluding discriminant values
  - `find_discriminants()` - detect discriminant properties in unions
  - `narrow_to_type()` / `narrow_excluding_type()` - general narrowing
- ThinChecker now fully leverages solver for narrowing and subtyping

- **Diagnostic Generation Complete**
- Added new `solver/diagnostics.rs` module
- `TypeDiagnostic` - structured diagnostic with message, code, severity, span
- `SourceSpan` - source location tracking (file, start, length)
- `DiagnosticSeverity` - error, warning, suggestion, message
- `TypeFormatter` - formats TypeId to human-readable strings
- `DiagnosticBuilder` - creates common type error diagnostics:
  - `type_not_assignable()` - code 2322
  - `property_missing()` - code 2741
  - `property_not_exist()` - code 2339
  - `argument_not_assignable()` - code 2345
- Comprehensive formatting for all TypeKey variants
- Added 10 new tests for diagnostics
- 980 tests passing

- **Source Location Tracking Complete**
- Added `SourceLocation` struct for tracking AST node positions
- Added `SpannedDiagnosticBuilder` - creates diagnostics with attached spans
- Added `DiagnosticCollector` - accumulates diagnostics with source tracking
- Added conversion from solver's TypeDiagnostic to checker's Diagnostic
- Added new diagnostic codes: CANNOT_FIND_NAME, NOT_CALLABLE, ARG_COUNT_MISMATCH
- Added new DiagnosticBuilder methods:
  - `cannot_find_name()` - code 2304
  - `not_callable()` - code 2349
  - `argument_count_mismatch()` - code 2554
  - `readonly_property()` - code 2540
- Integrated source location tracking into ThinChecker:
  - `get_source_location()` - get SourceLocation from NodeIndex
  - `error_type_not_assignable_at()` - report with source span
  - `error_property_missing_at()` - report with source span
  - `error_property_not_exist_at()` - report with source span
  - `error_argument_not_assignable_at()` - report with source span
  - `error_cannot_find_name_at()` - report with source span
  - `error_argument_count_mismatch_at()` - report with source span
  - `create_diagnostic_collector()` - for batch error reporting
  - `merge_diagnostics()` - merge collector diagnostics
  - `format_type()` - format TypeId using solver's TypeFormatter
- Added 13 new tests for source location tracking
- 993 tests passing
- **Priority 5 (Error Handling & Polish) COMPLETE!**

## 2026-01-05: Session 28
- **Full Type Lowering Complete**
- Added 7 new accessor methods to ThinNodeArena:
  - get_type_query, get_type_operator, get_infer_type
  - get_template_literal_type, get_named_tuple_member
  - get_type_predicate, get_type_parameter
- Added 6 new TypeKey variants:
  - TypeQuery, KeyOf, ReadonlyType, UniqueSymbol, Infer, ThisType
- Implemented lowering for all new type forms in solver/lower.rs
- Added subtype checking rules for new TypeKey variants
- 861 tests passing

## 2026-01-05: Session 27
- Fixed 52 Rust warnings (67 → 15)
- Recursion depth limits in ThinParser
- Tagged template literal support
- UB investigation: SAFE (range validated before transmute)

## 2026-01-05: Session 21
- **Solver Integration Complete**
- TypeInterner replaces TypeArena in ThinChecker
- SubtypeChecker wired up with coinductive cycle detection
- TypeLowering connected to get_type_from_type_node()
- 857 tests passing
