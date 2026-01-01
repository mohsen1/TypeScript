# TypeScript to Rust/WASM Migration Project

## AUTONOMOUS MODE ENABLED

This project is configured for autonomous overnight operation.
When running `/auto-migrate`, Claude should:
- **NEVER ask questions** - make best judgment calls
- **NEVER stop working** - always find the next task
- **Commit frequently** - small, working increments
- **Self-recover from errors** - fix and continue

## Quick Start for Autonomous Sessions

```bash
# Claude should run this at session start:
/auto-migrate
```

This reads MIGRATION_PLAN.md and works on the next pending task automatically.

## Project Overview

Incrementally migrating TypeScript compiler to Rust/WASM using "Strangler Fig" pattern.
Rust components progressively replace TypeScript while compiler stays functional.

## Critical Rules (Autonomous Mode)

### Decision Making Without Human Input
1. **When uncertain between options**: Choose the simpler/safer one
2. **When implementation is unclear**: Match TypeScript behavior exactly
3. **When stuck >10 minutes**: Skip with TODO, move to next task
4. **When tests fail**: Fix up to 3 times, then revert and continue

### Build Verification (Run After Every Change)
```bash
source ~/.cargo/env && cd /Users/mohsenazimi/code/TypeScript/wasm && cargo build && cargo test
```

### Commit Pattern (After Each Passing Test)
```bash
git add -A && git commit -m "feat(wasm): <description>"
```

## Key Files

| File | Purpose |
|------|---------|
| `MIGRATION_PLAN.md` | **THE SOURCE OF TRUTH** - read this first, find `[ ]` items |
| `wasm/src/lib.rs` | Main Rust entry, wasm-bindgen factory functions |
| `wasm/src/scanner_impl.rs` | Scanner implementation (~800 lines) |
| `wasm/src/parser.rs` | AST node definitions (~2000 lines, ~120 types) |
| `wasm/src/parser_impl.rs` | Parser implementation (~2600 lines) |
| `src/compiler/wasm.ts` | TypeScript bridge to WASM |

## Current State (Auto-Updated)

**Phase**: 3 - Parser
**Tests Passing**: 62 Rust tests
**Last Completed**: Type parsing (union, intersection, array, tuple, literal types)

### Next Pending Tasks (from MIGRATION_PLAN.md)
1. `[ ]` Implement AST-to-TypeScript-AST conversion
2. `[ ]` Add `--useRustParser` CLI flag
3. `[ ]` Add function type parsing `(x: number) => string`
4. `[ ]` Add conditional type parsing `T extends U ? X : Y`

## Error Recovery Patterns

### Borrow Checker Error
```rust
// Problem: cannot borrow *self as mutable more than once
self.method(self.other())  // ❌

// Solution: extract to local
let val = self.other();    // ✅
self.method(val)
```

### Missing AST Type
1. Add struct to `wasm/src/parser.rs`
2. Add variant to `Node` enum
3. Add to `base()` and `base_mut()` match arms
4. Add to parser_impl.rs imports

### wasm-bindgen Complex Type
```rust
#[wasm_bindgen(skip)]  // Add this for types that can't cross JS boundary
pub arena: NodeArena,
```

## Testing Commands

```bash
# Quick Rust test (run frequently)
cd /Users/mohsenazimi/code/TypeScript/wasm && cargo test

# Full build (run before commits)
cd /Users/mohsenazimi/code/TypeScript && npx hereby local

# Scanner verification
node scripts/verifyScanner.mjs src/compiler/checker.ts
```

## Architecture Notes

- **Arena allocation**: `NodeArena` + `NodeIndex` for AST nodes
- **Flag modules**: `pub mod node_flags { pub const X: u32 = 1; }` (not enums, for wasm-bindgen)
- **Serialization**: JSON for Rust↔JS during migration phase
- **Feature flags**: `--useRustScanner`, `--useRustParser` (add to sys.ts + tsc.ts)
