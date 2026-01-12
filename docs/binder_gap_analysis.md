# Binder Gap Analysis: Global Scope and lib.d.ts Integration

**Author:** Worker 4 (Binder Squad)
**Date:** 2026-01-12
**Task:** BIND-1 - Audit current global scope implementation

## Executive Summary

The TypeScript compiler currently has **no mechanism to load lib.d.ts symbols into the global scope**. This is the root cause of the high TS2304 ("Cannot find name") error count (702 extra errors).

The compiler uses a workaround in the checker (hardcoded lists of known globals) that returns `ANY` or `UNKNOWN` types instead of properly typed symbols, which leads to "error poisoning" - downstream type checking is suppressed.

## Current State

### 1. SymbolTable Implementation (`wasm/src/binder.rs`)

The `SymbolTable` is correctly implemented as a `FxHashMap<String, SymbolId>`:

```rust
// Lines 170-224
pub struct SymbolTable {
    symbols: FxHashMap<String, SymbolId>,
}
```

### 2. Source File Binding (`wasm/src/binder.rs:671-699`)

The `bind_source_file` function initializes the scope chain with a **SourceFile scope** but does **NOT** populate it with lib.d.ts symbols:

```rust
pub fn bind_source_file(&mut self, arena: &NodeArena, root: NodeIndex) {
    // Initialize scope chain with source file scope
    self.scope_chain.clear();
    self.scope_chain.push(ScopeContext::new(ContainerKind::SourceFile, root, None));
    self.current_scope_idx = 0;
    self.current_scope = SymbolTable::new();

    // ❌ NO LIB SYMBOLS LOADED HERE

    // Bind user code...
}
```

**Problem:** The global scope starts empty. Any reference to `console`, `Array`, `Promise`, etc. will fail to resolve.

### 3. Global Value Resolution (`wasm/src/thin_checker.rs:11773-11775`)

The checker attempts to resolve global values by looking in `file_locals`:

```rust
fn resolve_global_value_symbol(&self, name: &str) -> Option<SymbolId> {
    self.ctx.binder.file_locals.get(name)
}
```

**Problem:** `file_locals` only contains symbols from the user's source file, not from lib.d.ts.

### 4. The Workaround: Known Globals Lists

Instead of properly loading lib symbols, the compiler uses hardcoded lists:

#### Known Global Values (`thin_checker.rs:11777-11799`)

```rust
fn is_known_global_value_name(&self, name: &str) -> bool {
    matches!(
        name,
        "console" | "Math" | "JSON" | "Object" | "Array" | "String"
        | "Number" | "Boolean" | "Function" | "Date" | "RegExp" | "Error" | "Promise"
        | "Map" | "Set" | "WeakMap" | "WeakSet" | "WeakRef" | "Proxy"
        // ... ~40 more names
    )
}
```

#### Known Global Types (`thin_checker.rs:11801-11815`)

```rust
fn is_known_global_type_name(&self, name: &str) -> bool {
    matches!(
        name,
        "Object" | "String" | "Number" | "Boolean" | "Symbol" | "Function"
        | "Promise" | "PromiseLike" | "Array" | "ReadonlyArray" | "ArrayLike"
        // ... ~30 more names
    )
}
```

When these names are encountered, the checker returns permissive types (`ANY` or `UNKNOWN`) instead of properly typed symbols.

### 5. lib.d.ts File Resolution

The config system **can resolve** lib.d.ts files (`wasm/src/cli/config.rs:544-574`):

```rust
fn resolve_lib_files(lib_list: &[String]) -> Result<Vec<PathBuf>> {
    let lib_dir = default_lib_dir()?;
    let lib_map = build_lib_map(&lib_dir)?;
    // ...
}
```

A default lib.d.ts exists at `tests/lib/lib.d.ts` with proper declarations like:

```typescript
declare var NaN: number;
declare var Infinity: number;
declare function parseInt(s: string, radix?: number): number;
interface Object { ... }
interface Array<T> { ... }
// ...
```

**Problem:** The resolved lib file paths are **never parsed** and their symbols are **never injected** into the global scope.

## The Gap

### What Should Happen

1. User's `tsconfig.json` specifies `compilerOptions.lib: ["ES2022"]`
2. Config system resolves lib.d.ts file paths (e.g., `lib.es2022.d.ts`)
3. **Each lib.d.ts file is parsed** into AST
4. **Each lib.d.ts file is bound** - creating symbols for `console`, `Array`, `Promise`, etc.
5. **Lib symbols are merged into the global scope** before binding user files
6. User code can reference `console.log()` with proper types

### What Actually Happens

1. User's `tsconfig.json` specifies `compilerOptions.lib: ["ES2022"]`
2. Config system resolves lib.d.ts file paths
3. ❌ **Lib files are never parsed**
4. ❌ **Lib symbols are never created**
5. ❌ **Global scope remains empty**
6. User code `console.log(...)` fails to resolve
7. Checker's workaround returns `ANY` type
8. Type checking is suppressed (error poisoning)

## Impact on Conformance

Per PROJECT_DIRECTION.md:

> **TS2304** (Cannot find name) is the #1 extra error (702 hits).
> When the Binder fails to resolve `console`, `Promise`, or `Array`, the Solver defaults the type to `Any` (Error Poisoning).

This suppresses downstream errors, contributing to the **68.2% missing error rate**.

## Root Cause Analysis

| Component | File | Status | Issue |
|-----------|------|--------|-------|
| Config resolution | `cli/config.rs` | ✅ Works | Resolves lib file paths correctly |
| lib.d.ts storage | `tests/lib/lib.d.ts` | ✅ Exists | Contains proper type declarations |
| Parser | `parser/*.rs` | ✅ Capable | Can parse .d.ts files |
| Binder | `binder.rs` | ❌ Incomplete | Never loads lib symbols into global scope |
| Checker | `thin_checker.rs` | ⚠️ Workaround | Uses hardcoded lists instead of proper symbols |

## Recommended Fix Approach

### Phase 1: Load Lib Files (BIND-4)

**Location:** `wasm/src/binder.rs` - `BinderState::bind_source_file`

**Changes needed:**

1. Add a `bind_lib_file(&mut self, arena: &NodeArena, root: NodeIndex)` method
2. Call `bind_lib_file` **before** binding user code
3. Merge lib symbols into the root scope's `file_locals`

**Pseudo-code:**

```rust
pub fn bind_source_file(&mut self, arena: &NodeArena, root: NodeIndex) {
    // 1. Initialize scope chain
    self.scope_chain.clear();
    self.scope_chain.push(ScopeContext::new(ContainerKind::SourceFile, root, None));
    self.current_scope_idx = 0;

    // 2. ✅ NEW: Load and bind lib.d.ts symbols first
    if let Some(lib_arenas) = &self.lib_arenas {
        for lib_arena in lib_arenas {
            self.bind_lib_file(lib_arena, lib_arena.root);
        }
    }

    // 3. Then bind user code (user symbols can override lib)
    // ...
}
```

### Phase 2: Symbol Merging

**Challenge:** lib.d.ts files use `declare` keywords and ambient contexts.

**Solution:**
- Treat lib.d.ts symbols as having `AMBIENT` flag
- Allow user code to override lib declarations (TypeScript behavior)
- Merge interfaces from lib with user-defined interfaces

### Phase 3: Remove Workarounds

**Location:** `wasm/src/thin_checker.rs`

Once lib symbols are properly loaded:
1. Remove `is_known_global_value_name()` usage
2. Remove `is_known_global_type_name()` usage
3. Delete or deprecate these functions

## Test Plan (BIND-7)

After implementing the fix:

```typescript
// Should resolve without TS2304 errors:
console.log("hello");
const x: Promise<number> = Promise.resolve(42);
const arr: Array<string> = ["a", "b"];
const obj: Object = new Object();
```

**Expected:** TS2304 errors for global symbols drop from 702 to <50.

## Files Requiring Changes

1. **`wasm/src/binder.rs`** - Add lib loading to `bind_source_file`
2. **`wasm/src/cli/driver.rs`** - Parse lib files and create lib arenas
3. **`wasm/src/thin_checker.rs`** - Remove workaround code
4. **`wasm/src/lib.rs`** - Add public API for lib binding

## Notes

- The parser already supports `.d.ts` files (ambient declarations)
- The `SymbolTable` architecture is sound - just needs population
- Module augmentation (merging `interface Window` across files) may need additional work
- The `resolve_lib_files()` function returns correct paths - use it!
