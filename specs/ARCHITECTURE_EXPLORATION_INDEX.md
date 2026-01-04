# WASM TypeScript Compiler - Architecture Documentation Index

This index provides quick navigation to all architecture documentation created during the comprehensive exploration.

## Documents Generated

### 1. **WASM_ARCHITECTURE.md** (1,117 lines)
Complete architectural overview of the Rust/WASM TypeScript compiler port.

**Contents**:
- Project goals and key metrics
- Complete directory structure (26,475 LOC across 50+ modules)
- Compilation pipeline (source → scanner → parser → binder → checker → emitter)
- Core modules breakdown (Scanner, Parser, Binder, Checker, Emitter, Services, Transforms)
- Performance-first architecture (Phase 0 - ThinNode, Interner, Parallelism)
- Data structures (ThinNode, Symbol, Type arenas; index-based references)
- Key design patterns (arena allocation, typed pools, scope stacks)
- Parallelism infrastructure (Rayon, work-stealing, symbol merging)
- Memory management strategies
- Testing infrastructure
- WASM integration details
- Compilation phases summary
- Architecture decision records (ADRs)
- Future directions

**Use this for**: Understanding the complete system architecture, design decisions, and implementation details.

**Files referenced**: ~50+ Rust files from wasm/src/

### 2. **WASM_EXPLORATION_SUMMARY.txt** (379 lines)
Quick reference guide and executive summary of the exploration.

**Sections**:
1. Complete directory structure (visual tree)
2. Key metrics and analysis (LOC, node sizes, improvements)
3. Architecture highlights (performance, design patterns)
4. Compilation phases status
5. Key files by purpose
6. Data structures summary
7. Parallelism infrastructure details
8. Testing status
9. WASM integration overview
10. Recent progress and next steps
11. File paths (absolute)

**Use this for**: Quick lookups, status checks, and getting started.

**Best for**: Developers new to the codebase or needing quick reference.

---

## Directory Structure Reference

```
wasm/src/ (26,475 LOC)
├── Phase 0 (Performance-First):
│   ├── lib.rs (814 LOC)
│   ├── thin_parser.rs (5,130 LOC) ⭐ Primary parser
│   ├── thin_binder.rs (894 LOC) ⭐ Primary binder
│   ├── thin_checker.rs (847 LOC) ⭐ Expanding checker
│   ├── thin_emitter.rs (1,923 LOC) ⭐ Primary emitter
│   ├── interner.rs (202 LOC)
│   └── parallel.rs (688 LOC)
│
├── Phase 1-3 (Parsing):
│   ├── scanner_impl.rs (2,289 LOC)
│   ├── parser_impl.rs (6,228 LOC) - Legacy
│   └── parser/ (8 submodules)
│
├── Phase 4 (Binding):
│   └── binder.rs (1,984 LOC) - Legacy
│
├── Phase 5 (Type Checking):
│   └── checker/ (~750+ KB)
│
├── Phase 6 (Emission):
│   ├── emitter.rs (3,194 LOC) - Legacy
│   ├── declaration_emitter.rs (611 LOC)
│   ├── source_map.rs (515 LOC)
│   └── comments.rs (328 LOC)
│
├── Phase 6.5+ (Transforms):
│   └── transforms/ (5 submodules)
│
└── Phase 7 (Services):
    └── services/
```

---

## Key Metrics at a Glance

| Metric | Value | Notes |
|--------|-------|-------|
| **Total Lines** | 26,475 LOC | Across 50+ modules |
| **Parser Size** | 11,417 LOC | Both implementations combined |
| **ThinNode Size** | 16 bytes | vs 208 bytes legacy (13x improvement) |
| **Nodes/Cache-Line** | 4 | vs 0.31 with legacy Node enum |
| **Cache Improvement** | 13x | Most significant optimization |
| **Tests Passing** | 595+ | Comprehensive coverage |
| **Parser Tests** | 63+ | Complete feature coverage |
| **Parallel Speedup** | 3-4x | On quad-core for parsing |

---

## Architecture Highlights

### 1. Performance-First Design (Phase 0)

```
ThinNode:   16 bytes  (4 nodes/cache-line)
Legacy:     208 bytes (0.31 nodes/cache-line)
Gain:       13x cache locality improvement
```

### 2. Complete Compilation Pipeline

```
Source → Scanner → Parser → Binder → Checker → Emitter → Output
                                                ↓
                                        Source Maps
                                        Comments
                                        Declarations
```

### 3. Parallel Processing

```
Files → [Parallel Parse + Bind] → [Sequential Symbol Merge] → Checker
        (Rayon par_iter)          (5-10% overhead)
```

### 4. Modern Rust Patterns

- Arena allocation (Vec<T> with indices)
- Index-based references (u32 NodeIndex, TypeId, SymbolId)
- Zero GC overhead
- Cache-friendly data layout

---

## Key Files by Purpose

### Parsing (11,417 LOC)
- **`thin_parser.rs`** (5,130 LOC) - PRIMARY: Cache-optimized parser
- **`parser_impl.rs`** (6,228 LOC) - Legacy: Full-featured parser
- **`parser/thin_node.rs`** - 16-byte nodes + 60+ typed pools
- **`parser/ast/`** - AST definitions (8 submodules)

### Binding (2,878 LOC)
- **`thin_binder.rs`** (894 LOC) - PRIMARY: ThinNode-based binder
- **`binder.rs`** (1,984 LOC) - Legacy: Full-featured binder

### Type Checking (750+ KB)
- **`thin_checker.rs`** (847 LOC) - PRIMARY: Expanding ThinNode checker
- **`checker/state.rs`** (66 KB) - Diagnostics & state
- **`checker/type_retrieval.rs`** (182 KB) - Type inference engine
- **`checker/relations.rs`** (25 KB) - Type compatibility
- **`checker/narrowing.rs`** (49 KB) - Type guards

### Code Generation (5,117 LOC)
- **`thin_emitter.rs`** (1,923 LOC) - PRIMARY: ThinNode emitter
- **`emitter.rs`** (3,194 LOC) - Legacy: Full-featured emitter
- **`declaration_emitter.rs`** (611 LOC) - .d.ts generation
- **`source_map.rs`** (515 LOC) - Source map generation

### Infrastructure
- **`lib.rs`** (814 LOC) - WASM exports, string utilities
- **`interner.rs`** (202 LOC) - String interning
- **`parallel.rs`** (688 LOC) - Rayon parallelism
- **`scanner_impl.rs`** (2,289 LOC) - Tokenization

---

## Compilation Phases Status

| Phase | Component | Status | Key File |
|-------|-----------|--------|----------|
| 0.1 | ThinNode Architecture | ✓ Complete | `parser/thin_node.rs` |
| 0.2 | String Interning | ✓ Complete | `interner.rs` |
| 0.3 | Character Codes | ✓ Complete | `char_codes.rs` |
| 0.4 | Parallelism | ✓ Complete | `parallel.rs` |
| 1-3 | Parsing | ✓ Complete | `thin_parser.rs` |
| 4 | Binding | ✓ Complete | `thin_binder.rs` |
| 5 | Type Checking | ◐ Partial | `thin_checker.rs` |
| 6 | Emission | ✓ Complete | `thin_emitter.rs` |
| 6.2 | Source Maps | ✓ Complete | `source_map.rs` |
| 6.3 | Comments | ✓ Complete | `comments.rs` |
| 6.4 | Declarations | ✓ Complete | `declaration_emitter.rs` |
| 6.5+ | Transforms | ◐ Partial | `transforms/` |
| 7 | Services | - Minimal | `services/mod.rs` |

---

## Data Structures Overview

### Node Storage (16 bytes)
```rust
pub struct ThinNode {
    pub kind: u16,           // SyntaxKind
    pub flags: u16,          // NodeFlags
    pub pos: u32,            // Start position
    pub end: u32,            // End position
    pub data_index: u32,     // Index into typed pool
}
```

### Symbol Storage
```rust
pub struct Symbol {
    pub flags: u32,          // SymbolFlags (31 bits)
    pub escaped_name: String,
}
```

### Type Storage
```rust
pub enum Type {
    Intrinsic(IntrinsicType),
    Literal(LiteralType),
    Reference(Box<TypeReference>),
    Union(Box<UnionType>),
    // ... 15+ more variants
}
```

---

## Related Documentation

- **Migration Plan**: `/specs/migration_plan.md` - Project roadmap and priorities
- **TypeScript-Go Architecture**: `/specs/TYPESCRIPT_GO_ARCHITECTURE.md` - Reference implementation
- **CLAUDE.md**: `./.claude/CLAUDE.md` - Project instructions and guidelines

---

## Quick Start References

### For New Developers
1. Read **WASM_EXPLORATION_SUMMARY.txt** (Section 1-3) - 15 min overview
2. Review **Directory Structure** above - understand layout
3. Skim **WASM_ARCHITECTURE.md** sections 2-4 - detailed pipeline

### For Performance Optimization
1. Read **Phase 0 Performance-First Architecture** section
2. Focus on **ThinNode Design** (16 bytes vs 208 bytes)
3. Study **Parallelism Infrastructure** section
4. Review **Memory Management** strategies

### For Adding Features
1. Identify compilation phase (1-7)
2. Find **Core Modules** section for that phase
3. Locate primary file (`thin_*` preferred)
4. Review test patterns in that module

### For Bug Investigation
1. Check **Compilation Pipeline** for affected phase
2. Find phase in **Key Files by Purpose**
3. Trace through relevant file sections in WASM_ARCHITECTURE.md
4. Review tests for that component

---

## Testing Information

**Total Tests**: 595+ passing

**Test Categories**:
- Parser: 63+ tests (expressions, statements, types, JSX, async)
- Binding: 6+ tests (variables, functions, classes, hoisting)
- Parallel: 15+ tests (parsing, binding, merging)
- Type Checker: 424+ KB of tests (baseline + comprehensive)

**Running Tests**:
```bash
./wasm/test.sh              # All tests (Docker)
./wasm/test.sh parser       # Parser tests
./wasm/test.sh checker      # Type checker tests
```

---

## WASM Integration

**Public APIs** (via wasm-bindgen):
- `createScanner(text, skipTrivia)` → ScannerState
- `createParser(fileName, sourceText)` → ParserState
- `createBinder()` → BinderState

**Build**:
```bash
wasm-pack build wasm --target nodejs
```

**Output**: `wasm/pkg/` with .wasm, .js, .d.ts, .node files

---

## Performance Goals

**Current State**: 595+ tests passing, all parsing complete

**Next Priorities**:
1. Expand ThinChecker type inference
2. Parallelize type checking
3. Profile allocation hot spots
4. Benchmark vs TypeScript-Go

**Target**: Beat TypeScript-Go in real-world compilation speed

---

## Document Versions

- **Architecture Overview**: WASM_ARCHITECTURE.md (1,117 lines)
- **Quick Reference**: WASM_EXPLORATION_SUMMARY.txt (379 lines)
- **This Index**: ARCHITECTURE_EXPLORATION_INDEX.md

Generated: January 4, 2026
Codebase: 26,475 LOC across 50+ Rust modules

