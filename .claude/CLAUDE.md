# TypeScript → Rust/WASM Migration

## Mission
Incrementally migrate TypeScript compiler to Rust/WASM using "Strangler Fig" pattern.
Run autonomously overnight to complete Phase 5 (Type Checker) and beyond.

## Current State
- **Phase 5: Type Checker** - 60% complete, 222 Rust tests passing
- **Blocked:** `test_property_access_on_union` hangs (infinite loop in interface resolution)
- **Task List:** See `@fix_plan.md` for prioritized work

## Autonomous Workflow

```
LOOP:
  1. Read @fix_plan.md → pick next unchecked task
  2. Implement in wasm/src/*.rs
  3. Run: ./wasm/test.sh <test_name>
  4. If pass → mark complete in @fix_plan.md, commit, continue
  5. If fail 3x → add to Blocked section, skip, continue
  6. After 5 tasks → run full test suite: ./wasm/test.sh
```

## Commands

### ⚠️ ALWAYS USE DOCKER FOR RUST (prevents RAM explosion)

```bash
# PREFERRED: Use the test script (handles everything)
./wasm/test.sh              # Run all tests
./wasm/test.sh test_name    # Run specific test
./wasm/test.sh --rebuild    # Force rebuild image
./wasm/test.sh --clean      # Clean cached volumes

# Manual Docker commands (if needed):
# Build with BuildKit caching (first time slower, subsequent builds fast)
DOCKER_BUILDKIT=1 docker build -t rust-wasm-tests ./wasm

# Run tests with cached volumes (40-60% faster with nextest)
docker run --rm --memory="1g" --cpus="2.0" \
  -v cargo-registry:/usr/local/cargo/registry \
  -v cargo-git:/usr/local/cargo/git \
  rust-wasm-tests

# Specific test
docker run --rm --memory="1g" --cpus="2.0" \
  -v cargo-registry:/usr/local/cargo/registry \
  -v cargo-git:/usr/local/cargo/git \
  rust-wasm-tests cargo nextest run test_name

# Full TypeScript test suite (before committing)
docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby runtests-parallel

# Build compiler
docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby local

# Lint & format (required before commit)
docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby lint
docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby format

# Verify components
node scripts/verifyScanner.mjs
node scripts/verifyParser.mjs
```

## Key Files

| File | Purpose |
|------|---------|
| `wasm/src/checker.rs` | Type checking (~8000 lines, main focus) |
| `wasm/src/parser.rs` | AST node definitions |
| `wasm/src/parser_impl.rs` | Parsing logic |
| `wasm/src/binder.rs` | Symbol binding |
| `wasm/src/scanner_impl.rs` | Token scanning |
| `src/compiler/wasm.ts` | WASM bridge to TypeScript |
| `@fix_plan.md` | Task list (single source of truth) |

## Architecture

```
Source → Scanner → Parser → Binder → Checker → Emitter
         (Rust)    (Rust)   (Rust)   (Rust)    (TODO)
```

### Arena Pattern
All major structures use arena allocation with IDs:
- `NodeArena` + `NodeIndex` - AST nodes
- `SymbolArena` + `SymbolId` - Symbols  
- `TypeArena` + `TypeId` - Types
- `FlowNodeArena` + `FlowNodeId` - Control flow

## Adding Features

### New Type Feature
1. Add type variant to `Type` enum in `checker.rs`
2. Update `TypeArena::create_*` method
3. Handle in `is_type_assignable_to()` relation
4. Add test: `#[test] fn test_feature_name()`

### New Syntax
1. Add AST node to `parser.rs`
2. Add parsing in `parser_impl.rs`
3. Handle in `binder.rs` if declares symbols
4. Handle in `checker.rs` `get_type_of_node_worker()`

## Commit Format
```
[wasm] <component>: <description>

Examples:
[wasm] checker: add discriminated union narrowing
[wasm] parser: handle optional chaining in call expressions
```

## Known Blockers

| Issue | Location | Workaround |
|-------|----------|------------|
| Property access on unions hangs | interface resolution | Skip test, debug later |

## Reference Docs
- `docs/TYPE_CHECKER_MINDMAP.md` - Type system architecture
- `docs/TYPE_CHECKER_IMPLEMENTATION.md` - Implementation patterns
- `~/code/typescript-go` - Microsoft's Go port (reference for patterns)
- `specs/migration_plan.md` - Full migration plan (read-only reference)
