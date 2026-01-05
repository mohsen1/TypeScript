# TypeScript-WASM Architecture

> **Rust/WASM Port of the TypeScript Compiler**

Native Rust port targeting WebAssembly, optimized for **cache efficiency** and **parallelism**.

---

## Key Metrics

| Component | Lines | Tests | Status |
|-----------|-------|-------|--------|
| Scanner | ~2,500 | 22 | ✅ Complete |
| Parser (ThinParser) | ~5,100 | 160+ | ✅ Complete |
| Binder (ThinBinder) | ~900 | 26+ | ✅ Complete |
| Type Checker + Solver | ~29,300 | 670+ | ✅ 99% |
| Emitter | ~5,100 | 92+ | ✅ 75% |
| Language Service | ~2,000 | 5 | 🟡 60% |
| **Total** | **~74,350** | **1006** | |

---

## Architecture

```
Source Text
    ↓
Scanner (scanner_impl.rs) → Tokens
    ↓
ThinParser (thin_parser.rs) → ThinNodeArena (16 bytes/node)
    ↓
ThinBinder (thin_binder.rs) → SymbolArena + SymbolTable
    ↓
ThinChecker + Solver → TypeId (O(1) equality)
    ↓
ThinEmitter → JavaScript + Source Maps + .d.ts
```

### Parallelism (Rayon)

```
Files → par_iter() → Parse each file (independent arenas)
                   → Bind each file (local symbols)
                   → Sequential symbol merge → MergedProgram
                   → Type check
```

---

## Performance Innovations

### ThinNode (13x Cache Improvement)

```rust
// Old: 208 bytes per node (0.31 nodes/cache-line)
pub enum Node { ... }

// New: 16 bytes per node (4 nodes/cache-line)
#[repr(C)]
pub struct ThinNode {
    pub kind: u16,        // SyntaxKind
    pub flags: u16,       // NodeFlags
    pub pos: u32,         // Start position
    pub end: u32,         // End position
    pub data_index: u32,  // Index into typed pool
}
```

### Typed Data Pools

```rust
pub struct ThinNodeArena {
    pub nodes: Vec<ThinNode>,           // Headers only
    pub identifiers: Vec<IdentifierData>,
    pub binary_exprs: Vec<BinaryExprData>,
    pub functions: Vec<FunctionData>,
    // ... 60+ typed pools
}
```

### String Interning

```rust
pub struct Interner {
    map: FxHashMap<String, Atom>,  // O(1) lookup
    strings: Vec<String>,           // O(1) retrieval
}
// Comparison is integer comparison: Atom(u32)
```

### Type Interning (Solver)

```rust
pub struct TypeId(pub u32);  // 4 bytes, O(1) equality
// Same structure = same TypeId (structural deduplication)
```

---

## Directory Structure

```
wasm/src/
├── lib.rs              # WASM entry point
├── scanner_impl.rs     # Tokenization
├── thin_parser.rs      # AST generation (5,100 LOC)
├── thin_binder.rs      # Symbol binding (900 LOC)
├── thin_checker.rs     # Type checking orchestration
├── thin_emitter.rs     # Code generation
├── parallel.rs         # Multi-file processing
├── interner.rs         # String deduplication
├── parser/
│   ├── thin_node.rs    # ThinNode definition
│   └── ast/            # Node data structures
├── checker/
│   ├── types/          # Type definitions
│   └── relations.rs    # Type compatibility
├── solver/             # Pure type logic (5,800 LOC)
│   ├── intern.rs       # TypeInterner
│   ├── subtype.rs      # SubtypeChecker
│   ├── infer.rs        # InferenceContext
│   ├── lower.rs        # AST → TypeId
│   ├── evaluate.rs     # Conditional/mapped types
│   └── diagnostics.rs  # Lazy error formatting
├── services/           # IDE features
└── transforms/         # ES2015+ downleveling
```

---

## Testing

```bash
# Run all tests (Docker required)
./wasm/test.sh

# TypeScript integration
npx hereby runtests-parallel

# Baseline comparison
node scripts/baseline-test-rust.mjs
```

---

## WASM Exports

```rust
#[wasm_bindgen(js_name = createScanner)]
pub fn create_scanner(text: String, skip_trivia: bool) -> ScannerState;

#[wasm_bindgen(js_name = createParser)]
pub fn create_parser(file_name: String, source_text: String) -> ParserState;
```

```typescript
// Usage from JavaScript
const parser = wasm.createParser("test.ts", "const x = 42;");
const sourceFile = parser.parse_source_file();
```

---

## Dependencies

```toml
wasm-bindgen = "0.2"    # WASM-JS interop
serde = "1.0"           # Serialization
rustc-hash = "2.0"      # FxHashMap (fast hashing)
rayon = "1.10"          # Parallel iteration
ena = "0.14"            # Union-Find (type inference)
```

---

## vs TypeScript-Go

| Aspect | TypeScript-Go | TypeScript-WASM |
|--------|---------------|-----------------|
| Node Size | ~120 bytes | 16 bytes |
| Cache Locality | ~0.5 nodes/line | 4 nodes/line |
| Strings | GC strings | Interned Atoms (u32) |
| Memory | GC with pauses | Arenas (O(1) cleanup) |
| Parallelism | Goroutines + mutex | Rayon (compile-time safety) |
| Type Equality | Pointer comparison | TypeId (u32) interning |

---

See also:
- [migration_plan.md](migration_plan.md) - Current status and next steps
- [SOLVER.md](SOLVER.md) - Type system mathematical foundations
- [NEW_ENGINE.md](NEW_ENGINE.md) - Query-based architecture
- [TS_UNSOUNDNESS_CATALOG.md](TS_UNSOUNDNESS_CATALOG.md) - Intentional unsoundness rules
