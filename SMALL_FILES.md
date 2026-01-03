# Rust Module Refactoring Guide

## Rule: Keep Files Under 500 Lines

Large files are hard to maintain. Split them into focused modules.

## Current Problem Files

| File | Lines | Status |
|------|-------|--------|
| `checker.rs` | 8000+ | 🔴 Split immediately |
| `parser.rs` | 3000+ | 🟡 Split when adding features |
| `parser_impl.rs` | 2500+ | 🟡 Split when adding features |

## Target Structure

```
wasm/src/
├── lib.rs                    # Re-exports only (~50 lines)
├── scanner/
│   ├── mod.rs               # pub use statements
│   ├── syntax_kind.rs       # SyntaxKind enum
│   ├── token_flags.rs       # TokenFlags
│   └── state.rs             # ScannerState
├── parser/
│   ├── mod.rs
│   ├── ast/
│   │   ├── mod.rs
│   │   ├── node.rs          # Node enum, NodeBase
│   │   ├── expressions.rs
│   │   ├── statements.rs
│   │   ├── declarations.rs
│   │   └── types.rs
│   ├── flags.rs
│   └── state.rs
├── binder/
│   ├── mod.rs
│   ├── symbol.rs
│   ├── scope.rs
│   └── flow.rs
└── checker/
    ├── mod.rs               # CheckerState, main check()
    ├── types/
    │   ├── mod.rs
    │   ├── type_def.rs      # Type enum, TypeFlags
    │   ├── intrinsics.rs    # Primitive types
    │   ├── object.rs        # ObjectType
    │   ├── union.rs         # UnionType
    │   ├── conditional.rs
    │   ├── mapped.rs
    │   └── template.rs
    ├── arena.rs             # TypeArena
    ├── relations/
    │   ├── mod.rs
    │   ├── assignability.rs
    │   └── subtype.rs
    ├── inference/
    │   ├── mod.rs
    │   ├── context.rs
    │   └── contextual.rs
    ├── narrowing/
    │   ├── mod.rs
    │   ├── typeof.rs
    │   ├── instanceof.rs
    │   └── discriminant.rs
    └── diagnostics.rs
```

## How to Split a File

### Step 1: Create the directory
```bash
mkdir -p wasm/src/checker/types
```

### Step 2: Create mod.rs with re-exports
```rust
// wasm/src/checker/mod.rs
mod arena;
mod diagnostics;
mod types;
mod relations;
mod inference;
mod narrowing;

pub use arena::*;
pub use types::*;
// ... etc
```

### Step 3: Move related code to new file
```rust
// wasm/src/checker/types/union.rs
use super::type_def::{Type, TypeId};
use crate::checker::arena::TypeArena;

pub fn get_union_type(arena: &mut TypeArena, types: Vec<TypeId>) -> TypeId {
    // ... code moved from checker.rs
}
```

### Step 4: Update imports in original file
```rust
// wasm/src/checker/mod.rs
use crate::checker::types::union::get_union_type;
```

## Splitting Rules

1. **One concept per file** - `union.rs` has union logic only
2. **Prefer `pub use` re-exports** - Keep public API stable
3. **Tests stay with code** - `#[cfg(test)]` at bottom of each file
4. **Max 500 lines** - Split further if needed
5. **Clear names** - File name = what's inside

## Priority Order

1. **checker.rs** → Split into `checker/` module tree
2. **parser.rs** → Split AST types into `parser/ast/`
3. **parser_impl.rs** → Split by parse phase
4. **Leave small files alone** - If under 500 lines, it's fine

## Verification

After each split:
```bash
cd wasm && cargo test
```

All 220 tests must pass before committing.
