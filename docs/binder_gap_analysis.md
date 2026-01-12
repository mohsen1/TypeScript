# Binder Gap Analysis: Global Symbol Loading

**Author:** Worker 4 (Binder Squad)
**Date:** 2025-01-12
**Task:** BIND-1 - Audit current global scope implementation

---

## Executive Summary

The WASM Rust binder (`wasm/src/binder.rs` and `wasm/src/thin_binder.rs`) does **not** automatically load global symbols from `lib.d.ts` files. This is the root cause of the TS2304 "Cannot find name" errors for built-in JavaScript globals like `console`, `Array`, `Promise`, `Object`, etc.

---

## Current Implementation

### 1. SymbolTable Structure

**Location:** `wasm/src/binder.rs:173-183`

```rust
#[derive(Clone, Debug, Default, Serialize)]
pub struct SymbolTable {
    symbols: FxHashMap<String, SymbolId>,
}
```

The `SymbolTable` is a simple hash map from symbol names to `SymbolId`. It correctly handles:
- Scope management via `ScopeContext` stack
- Function vs block scoping
- Variable hoisting for `var` declarations
- Declaration merging for interfaces/namespaces/classes

### 2. Source File Binding

**Location:** `wasm/src/binder.rs:671-699`

```rust
pub fn bind_source_file(&mut self, arena: &NodeArena, root: NodeIndex) {
    // Initialize scope chain with source file scope
    self.scope_chain.clear();
    self.scope_chain.push(ScopeContext::new(ContainerKind::SourceFile, root, None));
    self.current_scope_idx = 0;
    self.current_scope = SymbolTable::new();  // <-- EMPTY scope!

    // Create START flow node for the file
    let start_flow = self.flow_nodes.alloc(flow_flags::START);
    self.current_flow = start_flow;
    // ... rest of binding
}
```

**Key Issue:** The `SymbolTable::new()` creates an **empty** scope. No global symbols are injected.

### 3. LibContext Infrastructure Exists but Unused

**Location:** `wasm/src/checker/context.rs:248-257`

```rust
/// Lib file contexts for global type resolution (lib.es5.d.ts, lib.dom.d.ts, etc.).
/// Each entry is a (arena, binder) pair from a pre-parsed lib file.
pub lib_contexts: Vec<LibContext>,
/// Initialized as Vec::new() - EMPTY!

/// Context for a lib file (arena + binder) for global type resolution.
pub struct LibContext {
    pub arena: Arc<ThinNodeArena>,
    pub binder: Arc<ThinBinderState>,
}
```

The infrastructure exists (`LibContext`, `lib_contexts` vector) but:
1. It is **never populated** with actual lib.d.ts files
2. The binder doesn't inject lib symbols during `bind_source_file()`

---

## Global Symbols That Are Missing

### ECMAScript Built-ins (from `src/lib/es5.d.ts`, `src/lib/es2015.d.ts`, etc.)

| Symbol | Type | Location |
|--------|------|----------|
| `console` | `Console` interface | `dom.generated.d.ts:38688`, `webworker.generated.d.ts:13042` |
| `Array` | interface | `lib.es5.d.ts` |
| `Object` | interface | `lib.es5.d.ts` |
| `String` | interface | `lib.es5.d.ts` |
| `Number` | interface | `lib.es5.d.ts` |
| `Boolean` | interface | `lib.es5.d.ts` |
| `Promise` | interface | `lib.es2015.promise.d.ts` |
| `Symbol` | interface | `lib.es2015.symbol.d.ts` |
| `Map` | interface | `lib.es2015.collection.d.ts` |
| `Set` | interface | `lib.es2015.collection.d.ts` |
| `eval`, `parseInt`, `parseFloat` | functions | `lib.es5.d.ts` |
| `NaN`, `Infinity` | variables | `lib.es5.d.ts` |

### Example: console Declaration

**Location:** `src/lib/dom.generated.d.ts:38570-38688`

```typescript
interface Console {
    assert(condition?: boolean, ...data: any[]): void;
    clear(): void;
    count(label?: string): void;
    debug(...data: any[]): void;
    // ... many more methods
}

declare var console: Console;
```

---

## How TypeScript Compiler Handles This

In the reference TypeScript compiler (`src/compiler/binder.ts`), global symbols from lib.d.ts are loaded during compilation initialization. The compiler:

1. Reads compiler options to determine which lib files to include (e.g., `lib: ["es2015", "dom"]`)
2. Parses each lib.d.ts file
3. Creates symbols for all top-level `declare` statements
4. Merges these symbols into the global scope of each source file

---

## The Gap

### What Should Happen (but doesn't)

1. **During binder initialization**, load default lib.d.ts files based on target ES version
2. **Parse lib.d.ts files** into `ThinNodeArena`
3. **Run binder** on lib files to create `LibContext` entries
4. **Inject lib symbols** into the root scope during `bind_source_file()`

### Current State

| Step | Status | Notes |
|------|--------|-------|
| Parse lib.d.ts files | ❌ Not implemented | No code to load lib files |
| Create LibContext entries | ❌ Not implemented | `lib_contexts` is always empty |
| Inject symbols into root scope | ❌ Not implemented | `bind_source_file()` creates empty scope |
| Use lib_contexts for resolution | ⚠️ Partially implemented | `resolve_lib_type_by_name()` exists but never finds anything |

---

## Impact

The 702 TS2304 errors in test files are caused by this missing global symbol loading:

```
TS2304: Cannot find name 'console'.
TS2304: Cannot find name 'Array'.
TS2304: Cannot find name 'Promise'.
TS2304: Cannot find name 'Object'.
TS2304: Cannot find name 'String'.
... (700+ more)
```

Each of these would resolve correctly if the appropriate lib.d.ts symbols were loaded.

---

## Proposed Fix Location

The fix should be implemented in `wasm/src/binder.rs` or `wasm/src/thin_binder.rs`:

### Option 1: Inject During `bind_source_file()`

Modify `bind_source_file()` to accept a list of `LibContext` entries and merge their `file_locals` into the root scope:

```rust
pub fn bind_source_file_with_libs(
    &mut self,
    arena: &NodeArena,
    root: NodeIndex,
    lib_contexts: &[Arc<ThinBinderState>],
) {
    // Initialize scope chain
    self.scope_chain.clear();
    self.scope_chain.push(ScopeContext::new(ContainerKind::SourceFile, root, None));
    self.current_scope_idx = 0;
    self.current_scope = SymbolTable::new();

    // TODO: Merge lib symbols into current_scope here!
    for lib_binder in lib_contexts {
        for (name, sym_id) in lib_binder.file_locals.iter() {
            self.current_scope.set(name.clone(), *sym_id);
        }
    }

    // ... rest of binding
}
```

### Option 2: Pre-populate Symbol Arena

Copy lib symbols into the main binder's symbol arena at initialization time, preserving the original IDs via arena merging.

---

## Next Steps (for Worker 4)

Based on this analysis, the next tasks are:

1. **BIND-4:** Fix Global SymbolTable initialization
   - Modify `Binder::new` or `bind_source_file()` to inject lib symbols
   - Load `lib.d.ts` types from `stdlib/lib.d.ts`
   - Ensure symbols are merged correctly at module level

2. **BIND-7:** Test TS2304 fixes
   - Create test file for all previously failing global symbols
   - Verify `console`, `Promise`, `Array`, `Object`, `String`, `Number` resolve correctly
   - Goal: Reduce TS2304 extra errors from 702 to <50

---

## Files Referenced

| File | Purpose |
|------|---------|
| `wasm/src/binder.rs` | Main binder implementation (SymbolTable, BinderState) |
| `wasm/src/thin_binder.rs` | ThinBinder using ThinNodeArena (used in production) |
| `wasm/src/checker/context.rs` | CheckerContext with `lib_contexts` vector |
| `src/lib/es5.d.ts` | ECMAScript 5 built-in types |
| `src/lib/dom.generated.d.ts` | DOM API types (includes `console`) |
| `src/lib/webworker.generated.d.ts` | Web Worker API types |
| `tests/lib/lib.d.ts` | Test lib file |
