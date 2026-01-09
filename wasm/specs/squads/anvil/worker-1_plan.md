# Anvil Worker 1 - TS2304 Namespace Scoping (Binder Scope Chain)

## Operation Conformance Assignment

**Mission**: Fix false positive TS2304 errors caused by incorrect scope chain traversal in the binder.

**Target Error**: TS2304 "Cannot find name 'X'" - 75 false positives
**Root Cause**: Namespace declarations don't properly establish parent scope chains

## Problem Analysis

The WASM checker incorrectly reports TS2304 for valid code like:
```typescript
namespace Outer {
  export const x = 1;
  namespace Inner {
    const y = x; // FALSE POSITIVE: TS2304 "Cannot find name 'x'"
  }
}
```

TSC correctly resolves `x` by walking up the scope chain from Inner -> Outer.

## Implementation Tasks

### Task 1: Audit Binder Scope Chain Creation
**File**: `wasm/src/binder/mod.rs`

1. Find where `SyntaxKind::ModuleDeclaration` is bound
2. Trace how `parent_scope` is set when entering a namespace
3. Verify child namespaces link to parent namespace scope (not just module scope)

### Task 2: Fix Namespace Parent Linking
**File**: `wasm/src/binder/mod.rs`

The issue is likely in the `bind_module_declaration` or equivalent function:
```rust
// CURRENT (broken): Resets to module scope
fn bind_module_declaration(&mut self, node: &Node) {
    let scope = self.create_scope(ScopeKind::Namespace);
    scope.parent = self.module_scope; // BUG: Should be current_scope
    // ...
}

// FIXED: Chain to current scope
fn bind_module_declaration(&mut self, node: &Node) {
    let scope = self.create_scope(ScopeKind::Namespace);
    scope.parent = self.current_scope; // Proper chain
    // ...
}
```

### Task 3: Add Scope Chain Debug Output
**File**: `wasm/src/binder/mod.rs`

Add debug logging to verify scope chains:
```rust
#[cfg(debug_assertions)]
fn debug_scope_chain(&self, scope_id: ScopeId) {
    let mut current = Some(scope_id);
    while let Some(id) = current {
        let scope = &self.scopes[id];
        eprintln!("  Scope {:?}: {:?}", id, scope.kind);
        current = scope.parent;
    }
}
```

### Task 4: Write Regression Tests
**File**: `wasm/src/binder/tests.rs` or similar

```typescript
// Test 1: Basic nested namespace resolution
namespace A {
  export const x = 1;
  namespace B {
    const y = x; // Should NOT error
  }
}

// Test 2: Deep nesting
namespace A {
  export const a = 1;
  namespace B {
    export const b = 2;
    namespace C {
      const c = a + b; // Should resolve both
    }
  }
}

// Test 3: Shadowing should work
namespace A {
  const x = 1;
  namespace B {
    const x = 2; // Shadows outer x
    const y = x; // Should be 2, not error
  }
}
```

## Success Criteria

- [ ] Nested namespace identifiers resolve correctly
- [ ] TS2304 false positives drop by 50+ occurrences
- [ ] All regression tests pass
- [ ] No new errors introduced

## Coordination

- **Worker 2** handles symbol resolution side of TS2304
- This worker focuses on binder/scope chain infrastructure
- Sync on shared scope structures before modifying

## Files to Modify

1. `wasm/src/binder/mod.rs` - Main scope chain fixes
2. `wasm/src/binder/scope.rs` - If scope structures need updates
3. Test files as needed

## Verification

Run after changes:
```bash
node wasm/differential-test/conformance-runner.mjs --max=200 -v 2>&1 | grep TS2304
```

Target: Reduce TS2304 false positives from 75 to <30.

## Status
Active

## Notes
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
