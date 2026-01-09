# Anvil Worker 2 - TS2304 Namespace Scoping (Symbol Resolution)

## Operation Conformance Assignment

**Mission**: Fix false positive TS2304 errors by improving symbol resolution in the checker.

**Target Error**: TS2304 "Cannot find name 'X'" - 75 false positives (shared with Worker 1)
**Root Cause**: Symbol lookup doesn't properly traverse namespace parent chain

## Problem Analysis

Even with correct scope chains (Worker 1's fix), the checker must correctly traverse them:

```typescript
namespace Outer {
  export interface Config { value: number; }
  namespace Inner {
    function process(c: Config) { } // TS2304 if symbol lookup fails
  }
}
```

The checker's `resolve_name` or equivalent must walk up the scope chain to find `Config`.

## Implementation Tasks

### Task 1: Audit Symbol Resolution in Checker
**File**: `wasm/src/checker/mod.rs`

1. Find the `resolve_name` / `resolve_symbol` function
2. Trace how it walks scope chains
3. Identify where namespace parent traversal fails

### Task 2: Fix Symbol Lookup Traversal
**File**: `wasm/src/checker/mod.rs`

```rust
// CURRENT (broken): Only checks current scope
fn resolve_name(&self, name: &str, scope: ScopeId) -> Option<Symbol> {
    self.scopes[scope].symbols.get(name)
}

// FIXED: Walk up scope chain
fn resolve_name(&self, name: &str, scope: ScopeId) -> Option<Symbol> {
    let mut current = Some(scope);
    while let Some(scope_id) = current {
        let scope = &self.scopes[scope_id];
        if let Some(symbol) = scope.symbols.get(name) {
            return Some(symbol.clone());
        }
        current = scope.parent;
    }
    None
}
```

### Task 3: Handle Export Visibility
**File**: `wasm/src/checker/mod.rs`

When traversing from child to parent namespace, respect export visibility:
```rust
fn resolve_name(&self, name: &str, scope: ScopeId) -> Option<Symbol> {
    let mut current = Some(scope);
    let mut is_first_scope = true;
    while let Some(scope_id) = current {
        let scope = &self.scopes[scope_id];
        if let Some(symbol) = scope.symbols.get(name) {
            // In parent scopes, check if exported
            if is_first_scope || symbol.is_exported() {
                return Some(symbol.clone());
            }
        }
        current = scope.parent;
        is_first_scope = false;
    }
    None
}
```

### Task 4: Write Regression Tests
**File**: `wasm/src/checker/tests.rs`

```typescript
// Test 1: Exported type visible in nested namespace
namespace A {
  export type ID = string;
  namespace B {
    let x: ID; // Should resolve
  }
}

// Test 2: Non-exported NOT visible
namespace A {
  type Internal = number;
  namespace B {
    let x: Internal; // Should error TS2304
  }
}

// Test 3: Class resolution
namespace Models {
  export class User {}
  namespace Helpers {
    function getUser(): User { return new User(); }
  }
}
```

## Success Criteria

- [x] Symbol lookup correctly traverses namespace parent chain
- [x] Export visibility is respected
- [ ] TS2304 false positives reduced significantly
- [ ] No new false negatives (missing real errors)

## Coordination

- **Worker 1** fixes binder scope chains
- This worker ensures checker uses those chains correctly
- Test both fixes together for full coverage

## Files to Modify

1. `wasm/src/checker/mod.rs` - Symbol resolution fixes
2. `wasm/src/checker/symbols.rs` - If symbol structures need updates
3. Test files as needed

## Verification

Run after changes:
```bash
node wasm/differential-test/conformance-runner.mjs --max=200 -v 2>&1 | grep TS2304
```

Target: Combined with Worker 1, reduce TS2304 from 75 to <20.

## Status
Active - implementation done, awaiting push

Ready for Merge: No (push blocked: git auth)

## Notes
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-2`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
