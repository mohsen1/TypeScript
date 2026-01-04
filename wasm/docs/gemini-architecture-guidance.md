# Architecture Analysis & Roadmap: TypeScript-to-Rust Migration

**Generated:** 2026-01-04
**Reviewer:** Gemini 3 Pro (via ask-gemini.mjs)
**Scope:** Overall architecture of `wasm/src/` and strategic guidance

---

## Executive Summary

The TypeScript-to-Rust migration has a **solid architectural foundation** with modern Rust compiler patterns (arena allocation, string interning, module mirroring). However, critical issues in UTF-8 handling, borrow checker conflicts, and the WASM interface bottleneck must be addressed before production readiness.

---

## 1. Architecture Assessment

### What's Working Well

| Decision | Assessment | Evidence |
|----------|------------|----------|
| **Arena Allocation** | Excellent | `NodeArena`, `TypeArena`, `SymbolArena` using `Vec<T>` with `u32` indices. Solves self-referential struct problems, improves cache locality. |
| **Module Structure** | Excellent | Mirrors TypeScript (`scanner`, `parser`, `binder`, `checker`, `emitter`). Enables 1:1 algorithm verification. |
| **String Interning** | Excellent | `Interner` returning `Atom` (u32). O(1) symbol comparison during binding/checking. |
| **Control Flow Graph** | Good | Correctly models CFG structure in `FlowNodeArena`. |

### What Needs Improvement

| Decision | Assessment | Problem |
|----------|------------|---------|
| **WASM Bindings** | Critical Flaw | JSON serialization of entire AST negates Rust performance benefits. |
| **Scanner UTF-8** | Blocker | `self.pos += 1` assumes 1 byte = 1 character. Panics on non-ASCII. |
| **Binder Scope Erasure** | Blocker | Local scopes discarded after binding. Language Service can't resolve locals. |
| **Checker Borrows** | Blocker | Recursive mutable borrows cause borrow checker violations. |

---

## 2. Architectural Debt & Design Flaws

### The "JSON Bridge" Bottleneck (Critical)

**Problem:** Methods like `get_source_file_json` and `get_diagnostics_json` serialize massive structures.

**Impact:** O(N) serialization cost on every interaction. For a 10k LOC file, this freezes the UI.

**Fix:** Move to **Lazy Accessors** or **Shared Memory**:
```rust
// Instead of: getAST() -> JSON
// Implement:  getNodeType(nodeIndex) -> u32
//             getChildCount(nodeIndex) -> u32
//             getChildAt(nodeIndex, index) -> nodeIndex
```

### Binder Scope Erasure (Blocker for Language Service)

**Problem:** `BinderState` flattens everything into `file_locals` or discards local scopes after processing.

**Impact:** Cannot resolve symbols for local variables, parameters, or class members.

**Fix:** Persist `NodeId -> SymbolId` mapping for every identifier node, not just declarations.

### Scanner UTF-8 Safety (Correctness)

**Problem:** `scanner_impl.rs` uses `self.pos += 1` assuming 1 byte = 1 character.

**Impact:** Panics or corrupts state on emoji or non-Latin identifiers.

**Fix:** Use `char_indices()` iterator or `char_len_at()` for correct UTF-8 handling.

### Recursive Mutable Borrows (Rust Specific)

**Problem:** Type Checker follows TypeScript's recursive logic, but Rust forbids mutable borrow while holding immutable reference.

**Impact:** Excessive cloning or runtime panics if `RefCell` overused.

**Fix:** Split `CheckerState` into:
- **Immutable Context:** Arenas, configuration
- **Mutable Caches:** `relation_cache`, `node_types` in `RefCell`

---

## 3. Comparison with TypeScript Architecture

| Feature | TypeScript | Rust Migration | Analysis |
|---------|------------|----------------|----------|
| **Memory** | V8 GC | Arena (Index-based) | **Rust is better.** Deterministic, no GC pauses. |
| **AST** | Object References | Integer Indices | **Rust is better** for cache locality. |
| **Strings** | V8 Interning | Custom Interner | **Parity.** Both O(1) comparison. |
| **Laziness** | Highly Lazy (Pull) | Eager Parsing | **TS is better.** Rust parses whole file eagerly. |
| **Control Flow** | Graph on Heap | `FlowNodeArena` | **Parity.** Correct graph structure. |
| **Error Recovery** | Robust | Basic | **TS is better.** Rust needs better sync points. |

---

## 4. Top 10 Actionable Tasks (Prioritized)

### 1. Fix Scanner UTF-8 Panic
**Change:** Use `char_len_at(self.pos)` instead of `self.pos += 1`
- **Complexity:** M
- **Impact:** BLOCKER - Panics on non-ASCII
- **Files:** `scanner_impl.rs`

### 2. Resolve Checker Borrow Violations
**Change:** Decouple `TypeArena` during operations, or read-phase/write-phase pattern
- **Complexity:** XL
- **Impact:** BLOCKER - Cannot compile narrowing code
- **Files:** `checker/narrowing.rs`, `checker/state.rs`

### 3. Persist Local Scopes in Binder
**Change:** Add `node_scope_map: HashMap<NodeIndex, SymbolTable>` to `BinderState`
- **Complexity:** L
- **Impact:** BLOCKER - Language Service only works for globals
- **Files:** `binder.rs`, `services/mod.rs`

### 4. Implement String Literal Escaping in Emitter
**Change:** Escape quotes, newlines, backslashes in `emit_string_literal`
- **Complexity:** S
- **Impact:** BLOCKER - Generates invalid code
- **Files:** `emitter.rs`

### 5. Fix Optional Property Type Checking
**Change:** Check `OPTIONAL` flag before failing on missing property
- **Complexity:** M
- **Impact:** CRITICAL - Basic interface assignability broken
- **Files:** `checker/relations.rs`

### 6. Implement Generics and Modifiers Emission
**Change:** Add `emit_type_parameters`, `emit_modifiers` helpers
- **Complexity:** M
- **Impact:** CRITICAL - Output strips all generics/modifiers
- **Files:** `emitter.rs`

### 7. Fix Enum Compatibility Logic
**Change:** Compare `TypeId`/`SymbolId` identity, not string names
- **Complexity:** S
- **Impact:** CRITICAL - Violates nominal typing
- **Files:** `checker/relations.rs`

### 8. Optimize String Interner Memory
**Change:** Use `Rc<str>` to share allocation between map and vector
- **Complexity:** M
- **Impact:** HIGH - 50% memory reduction for identifiers
- **Files:** `interner.rs`

### 9. Fix Scanner State Bugs
**Change:** Clear `token_value` at start of `scan()`, fix template escape bounds
- **Complexity:** S
- **Impact:** CRITICAL - Incorrect parsing, crashes on bad input
- **Files:** `scanner_impl.rs`

### 10. Implement Function Arity Check
**Change:** Implement `min_argument_count` check in `is_signature_related`
- **Complexity:** S
- **Impact:** CRITICAL - Allows unsound function assignments
- **Files:** `checker/relations.rs`

---

## 5. Phased Roadmap

### Phase 1: Foundation Integrity (Weeks 1-3)

| Task | Complexity | Files |
|------|------------|-------|
| Fix Scanner UTF-8 handling | M | `scanner_impl.rs` |
| Refactor Checker borrow architecture | XL | `checker/narrowing.rs`, `checker/state.rs` |
| Ensure AST parent pointers populated | S | `parser_impl.rs` |
| Fix string literal escaping | S | `emitter.rs` |

### Phase 2: Semantic Correctness (Weeks 4-8)

| Task | Complexity | Files |
|------|------------|-------|
| Fix Binder scoping (persist locals) | L | `binder.rs` |
| Fix optional property checking | M | `checker/relations.rs` |
| Fix enum compatibility | S | `checker/relations.rs` |
| Fix function arity check | S | `checker/relations.rs` |
| Baseline tests: >80% pass rate | L | `checker/baseline_tests.rs` |
| Complete narrowing (`typeof`, `instanceof`) | L | `checker/narrowing.rs` |

### Phase 3: WASM Performance (Weeks 9-12)

| Task | Complexity | Files |
|------|------------|-------|
| Replace JSON with binary protocol | L | `lib.rs`, new WASM bindings |
| Implement lazy accessor API | M | `lib.rs` |
| Add lazy/skip parsing modes | L | `parser_impl.rs` |

### Phase 4: Language Service Features (Weeks 13+)

| Task | Complexity | Files |
|------|------------|-------|
| Context-aware completions | L | `services/mod.rs` |
| Cross-file symbol resolution | XL | `services/mod.rs`, `checker/state.rs` |
| Implement `Program` concept | L | New module |
| Go-to-type-definition | M | `services/mod.rs` |
| Diagnostics API exposure | M | `services/mod.rs` |

---

## 6. Key Architectural Recommendations

### Immediate Actions

1. **Stop using JSON for WASM boundary** - Implement lazy accessors
2. **Fix UTF-8 handling** - Use `char_len_at()` everywhere
3. **Split CheckerState** - Separate immutable arenas from mutable caches
4. **Persist local scopes** - Add `NodeIndex -> SymbolId` mapping

### Medium-Term Goals

1. **Implement incremental parsing** - Use `syntax_cursor` for AST reuse
2. **Add error recovery** - Better sync points in parser
3. **Optimize interner** - Use `Rc<str>` for memory efficiency

### Long-Term Vision

1. **Full TypeScript conformance** - Pass official test suite
2. **Sub-millisecond response** - Zero-cost WASM interface
3. **Multi-file support** - `Program` managing multiple `SourceFile`s
4. **Parallel type checking** - Per-file checking with shared global state

---

## 7. Summary

The project has a **solid Rust-idiomatic core** (arenas, interning, module structure) but is currently hobbled by:

1. **Naive WASM interface** (JSON serialization)
2. **Incomplete semantic binding** (scope erasure)
3. **UTF-8 safety issues** (scanner panics)
4. **Borrow checker violations** (narrowing code)

Addressing the **Binder's scope tracking** and **WASM serialization bottleneck** are the two highest-leverage activities for production readiness.

**Estimated timeline to production:** 16-20 weeks with focused effort on the phased roadmap above.
