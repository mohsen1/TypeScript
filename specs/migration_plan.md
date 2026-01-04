# Migration Plan: TypeScript Compiler → Rust via WebAssembly

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. The migration follows the "Strangler Fig"
pattern: Rust components progressively replace TypeScript modules while the
compiler remains fully functional at every step.

## Guiding Principles

1. **Never Break the Build** – Every commit must pass `hereby runtests-parallel`.
2. **Iterate in Small Slices** – One function, one module at a time.
3. **Test Before & After** – Existing test suite is the source of truth.
4. **Performance First** – we are building for performance

---

# 🚀 PRIORITY ZERO: DESIGN FOR SPEED

**Goal: Beat TypeScript-Go in performance.**

Stop "porting" TypeScript line-by-line. Start **architecting for the hardware**.

## Phase 0.1: Thin Nodes (2-3x Parser Speedup) - 🟡 In Progress

Current `Node` enum is sized to largest variant (~208 bytes). This destroys cache locality.

### Completed Infrastructure (~1,500 lines)
- [x] Size analysis complete: Node=208B, ClassDeclaration=200B, FunctionDeclaration=168B
- [x] ThinNode struct implemented: exactly 16 bytes (4 nodes per cache line)
- [x] 60+ typed data pool structs for all node categories:
  - Names: IdentifierData, QualifiedNameData, ComputedPropertyData
  - Literals: LiteralData (string, numeric, regex)
  - Expressions: BinaryExprData, UnaryExprData, CallExprData, AccessExprData, ConditionalExprData
  - Functions: FunctionData, ClassData, InterfaceData, TypeAliasData, EnumData
  - Statements: IfStatementData, LoopData, BlockData, SwitchData, TryData
  - Types: TypeRefData, CompositeTypeData, FunctionTypeData, MappedTypeData, ConditionalTypeData
  - Imports: ImportDeclData, ExportDeclData, SpecifierData
  - JSX: JsxElementData, JsxOpeningData, JsxAttributeData
  - Source: SourceFileData with full metadata
- [x] ThinNodeArena with all typed storage pools (40+ pools)
- [x] Arena methods for adding all node types (add_token, add_identifier, add_literal, etc.)
- [x] NodeView wrapper for ergonomic node access
- [x] Kind utilities (is_identifier, is_function_like, is_statement, is_type_node, etc.)
- [x] Kind validation in accessor methods (type safety)
- [x] Tests passing (5 thin_node tests, 595 total)

### NodeAccess Trait (Unified Interface)
- [x] NodeAccess trait defined with common node access methods
- [x] NodeInfo struct for common node information
- [x] ThinNodeArena implements NodeAccess
- [x] NodeArena implements NodeAccess
- 596 tests passing

### Next Steps
- [ ] Migrate parser to output ThinNodeArena
- [ ] Update binder to use NodeAccess trait
- [ ] Update checker to use NodeAccess trait
- [ ] Update emitter to use NodeAccess trait
- [ ] Benchmark: compare ThinNodeArena vs NodeArena parsing performance

### Architecture (wasm/src/parser/thin_node.rs)
```rust
#[repr(C)]
pub struct ThinNode {
    kind: u16,        // SyntaxKind
    flags: u16,       // Packed NodeFlags
    pos: u32,         // Start position
    end: u32,         // End position
    data_index: u32,  // Index into type-specific pool
}  // = 16 bytes total (13x improvement from 208 bytes)
```

### Performance Impact
- Before: 208 bytes/node = 0.31 nodes per cache line
- After: 16 bytes/node = 4 nodes per cache line
- Improvement: **13x better cache locality**

## Phase 0.2: Zero-Allocation Scanner (Massive Memory Reduction) - 🟡 In Progress

Current scanner does `self.source[...].to_string()` = malloc per token.

### Completed (2026-01-04)
- [x] Integrate `Interner` directly into `Scanner`
  - ScannerState now has `interner: Interner` field
  - Pre-interns common keywords via `intern_common()`
- [x] All identifiers become `Atom` (u32) - O(1) string comparison
  - `scan_identifier()` interns all identifiers during scanning
  - `get_token_atom()` returns the interned Atom for identifier tokens
  - `resolve_atom()` resolves Atom back to string
  - Non-identifier tokens have `Atom::NONE`
- [x] 3 new tests: identifier interning, non-identifier atoms, keyword interning
- [x] 605 tests passing

### TODO
- [ ] Scanner returns `&str` slices of source, never `String`
- [ ] Zero heap allocations during parsing (token_value still allocates)

## Phase 0.3: Arena-Based Type Checker (O(1) Cleanup)

### TODO
- [ ] Apply "Thin" pattern to `Type` enum (currently huge)
- [ ] Use `bumpalo` or `typed-arena` for Type objects
- [ ] All allocations for `check` go into single arena
- [ ] Deallocation = reset pointer (no `Drop` overhead)

## Phase 0.4: Parallelism (Fearless Concurrency)

### TODO
- [ ] Parse files in parallel with `Rayon`
- [ ] Pipeline: Parse → Bind (parallel) → Merge symbols (sequential) → Check bodies (parallel)
- [ ] Check function bodies in parallel (local inference doesn't affect global scope)

## Phase 0.5: SIMD Scanning (Advanced)

### TODO
- [ ] Use portable SIMD for whitespace/identifier scanning
- [ ] Reference: swc, oxc scanner implementations

## Why Rust Beats Go

| Feature | TypeScript-Go | Rust (This Project) |
|---------|---------------|---------------------|
| Memory | Heap + GC | Arena (O(1) free) |
| Data Layout | Pointers (scattered) | Contiguous arrays (cache-friendly) |
| Strings | GC Strings | Interned Atoms (u32) |
| Concurrency | Goroutines | Rayon (no races on AST) |

---

# REMAINING WORK

## Phase 6: Emitter (75% Complete)

| Metric | Value |
|--------|-------|
| Lines of Code | ~5,000 |
| Tests | 85+ |

### TODO
- [x] Declaration file emission (node filtering, export visibility, type-only imports)
- [x] ES2015+ transforms: arrow function → function expression
- [x] Module transforms: CommonJS import rewriting (require, __importDefault, __importStar)
- [x] Module transforms: CommonJS export rewriting (exports.x, __exportStar)
- [x] Async/await transforms: await→yield + __awaiter helper creation
- [~] Generator transforms (helper detection done, state machine pending)

## Phase 7: Language Service (55% Complete)

| Metric | Value |
|--------|-------|
| Lines of Code | ~1,500 |
| Tests | 5 |

### Completed (2026-01-04)
- [x] Signature help (basic implementation)
- [x] Context-aware completions:
  - `get_properties_of_type()` for type property enumeration
  - Member completions when typing after `.`
  - `find_property_access_at_position()` for context detection
  - Global completions refactored to separate method
- [x] 2 new tests: member completions, get_properties_of_type

### TODO
- [ ] Type completions (after `:` or in type position)
- [ ] Cross-file navigation support
- [ ] Formatting engine
- [ ] Code fixes and refactorings

## Phase 8: Full Rust Mode

- [ ] Remove TypeScript fallbacks
- [ ] Performance optimization pass
- [ ] Memory usage optimization
- [ ] WASM interface optimization (replace JSON with binary protocol)

---

# PROGRESS SUMMARY

| Phase | Component | Lines | Tests | Status |
|-------|-----------|-------|-------|--------|
| 0 | Infrastructure | - | - | ✅ Done |
| 1 | Utilities | ~300 | 21 | ✅ Done |
| 2 | Scanner | ~2,500 | 22 | ✅ Done |
| 3 | Parser | ~5,000 | 100+ | ✅ Done |
| 4 | Binder | ~1,900 | 20+ | ✅ Done |
| 5 | Type Checker | ~23,500 | 485 | ✅ 99% |
| 6 | Emitter | ~5,000 | 85+ | 🟡 75% |
| 7 | Language Service | ~1,500 | 5 | 🟡 55% |
| 8 | Full Rust Mode | - | - | ⬜ Pending |

**Total Rust Code**: ~42,000 lines
**Total Tests**: 607 passing
**Overall Progress**: ~88% of full compiler functionality

---

# MILESTONES

## 2026-01-04: Language Service Member Completions
- Added `get_properties_of_type()` to checker for enumerating type properties
- Context-aware completions: detects PropertyAccessExpression context
- Member completions when typing after `.` (obj.property)
- Refactored global completions into separate method
- 2 new tests for member completions and property enumeration
- 607 tests passing

## 2026-01-04: Scanner Interner Integration (Phase 0.2)
- Integrated Interner directly into ScannerState
- All identifiers now interned as Atom (u32) for O(1) string comparison
- `get_token_atom()` and `resolve_atom()` methods
- Pre-interning of common keywords via `intern_common()`
- 3 new tests for interner functionality
- 605 tests passing

## 2026-01-04: Thin Nodes Architecture + NodeAccess Trait
- Implemented ThinNode struct (16 bytes vs 208 bytes = 13x improvement)
- Created 60+ typed data pool structures for all node categories
- ThinNodeArena with all typed storage pools (~1,700 lines)
- NodeView wrapper for ergonomic node access
- Kind utilities (is_identifier, is_function_like, is_statement, etc.)
- NodeAccess trait for unified arena interface (both arenas implement it)
- NodeInfo struct for common node information
- 596 tests passing

## 2026-01-04: Architecture Fixes
- Fixed 4 BLOCKER/CRITICAL issues from Gemini architecture review
- String literal escaping, optional properties, enum nominal typing, function arity
- 577 tests passing

## 2026-01-04: Language Service Core
- Go-to-definition, find references, rename, quick info all working
- NodeSymbolMap for local symbol resolution
- Diagnostics API implemented

## 2026-01-03: Emitter Foundation
- Source map support (VLQ encoding, inline/external)
- Comment preservation
- JS transform scaffolding (ES2015, async, modules)

## 2026-01-02: Type Checker Complete
- 23,500 lines, 485 tests
- Full structural typing, generics, control flow analysis
- JSX support, discriminated unions, exhaustiveness checking

## 2026-01-01: Parser & Binder Complete
- Full AST with 130+ node types
- Symbol table with scope chain and control flow graph
- All TypeScript syntax supported
