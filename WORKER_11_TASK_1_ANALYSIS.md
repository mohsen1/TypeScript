# Worker 11 Task 1 Analysis Report

## Rust/WASM Setup Analysis

Generated: 2026-01-14  
Worker: Worker 11  
Target Branch: rust

---

## Executive Summary

Project Zang is a performance-first TypeScript compiler written in Rust, designed as a drop-in replacement for `tsc` with both native and WASM targets. The codebase contains **~458,000 lines of Rust code** organized in a sophisticated architecture with comprehensive testing infrastructure.

---

## 1. Directory Structure

```
wasm/
├── src/              # Main source code
│   ├── bin/          # CLI binary (tsz)
│   ├── checker/      # Type checker with control flow analysis
│   ├── cli/          # Command-line interface
│   ├── lsp/          # Language Server Protocol features
│   ├── parser/       # Parser with AST definitions
│   ├── solver/       # Type solver and compatibility layer
│   ├── thin_emitter/ # Code generation
│   └── transforms/   # ES5/CommonJS transforms
├── benches/          # Performance benchmarks (6 harnesses)
├── tests/            # Test libraries
├── differential-test/# Conformance testing
├── scripts/          # Build and test utilities
└── specs/            # Architecture specifications
```

---

## 2. Rust Dependencies (Cargo.toml)

### Core Dependencies
- **anyhow** (1.0) - Error handling
- **clap** (4.5) - CLI argument parsing
- **colored** (2.1) - Terminal output
- **globset** (0.4) - Pattern matching
- **notify** (6.1) - File watching
- **wasm-bindgen** (0.2) - WASM bindings
- **serde** (1.0) + **serde_json** - Serialization
- **rustc-hash** (2.0) - Fast hashing (FxHashMap equivalent)
- **rayon** (1.10) - Parallel processing
- **memchr** (2.7) - SIMD byte searching
- **walkdir** (2.5) - Directory traversal
- **smallvec** (1.13) - Stack-allocated vectors
- **ena** (0.14) - Union-Find for type unification
- **indexmap** (2.6) - Deterministic iteration
- **bitflags** (2.6) - Efficient TypeFlags

### Dev Dependencies
- **criterion** (0.5) - Benchmarking framework
- **cargo-nextest** - Faster test execution (40-60% speedup)

### Build Configuration
- **Edition**: Rust 2024
- **Crate Types**: `cdylib` (WASM), `rlib` (Rust library)
- **Binary**: `tsz` CLI at `src/bin/tsz.rs`

---

## 3. Build Process

### 3.1 Native Build
```bash
# Standard cargo build
cargo build --release
cargo test
cargo nextest run  # Faster tests
```

### 3.2 WASM Build

**Via Docker (build-wasm.sh):**
```bash
./wasm/build-wasm.sh
```
- Uses `rust:latest` Docker image
- Installs `wasm-pack`
- Builds for `nodejs` target
- Outputs to `wasm/pkg/`

**Via Hereby (build-wasm task):**
```javascript
// Herebyfile.mjs build-wasm task
wasm-pack build wasm --target nodejs --out-dir ../built/local/wasm
```
Output: `built/local/wasm/wasm_bg.wasm`

### 3.3 Dockerfile (wasm/Dockerfile)
- Multi-stage build for dependency caching
- Installs `cargo-nextest` for fast tests
- Runs tests in containerized environment

---

## 4. Key Architecture Components

### Parser (`src/parser/`)
- **ThinNode architecture**: Struct-of-Arrays design for performance
- 500 MB/s parsing speed
- Recursive descent parser with error recovery
- 85 remaining parser errors (down from 1,122 - 92% improvement)

### Type Checker (`src/checker/`)
- Control Flow Analysis (CFA) infrastructure
- Flow graph arena for variable tracking
- **Current Gap**: TS2454/TS2564 not fully implemented (1,016 missing errors)

### Type Solver (`src/solver/`)
- **"Judge"** (Sound Set Theory) + **"Lawyer"** (Compat Layer)
- TypeKey system for efficient type representation
- **Current Gap**: Fallback to `Any` instead of `Unknown` causes missing errors

### Emitter (`src/thin_emitter/`)
- ES5, CommonJS, async/await transforms
- JSX support
- Source map generation

### LSP Features (`src/lsp/`)
- Diagnostics, completions, hover, go-to-definition
- Semantic tokens, code actions, inlay hints
- Formatting, signature help, references

---

## 5. Current Conformance Status

### Metrics (as of 2026-01-11)
| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| Exact Match | **30.8%** | 50%+ | +7.5pp improvement |
| Missing Errors | **57.8%** | <30% | -10.4pp improvement |
| Extra Errors | **28.9%** | <20% | -6.9pp improvement |
| Parser Errors | **~85** | <100 | ✅ TARGET MET |

### Priority Issues
1. **TS2454/TS2564** (1,016 missing) - Control Flow Analysis
2. **TS2322** (310 missing) - Solver strictness
3. **TS2339** (292 extra) - Property access narrowing

---

## 6. Testing Infrastructure

### Unit Tests
```bash
./wasm/test.sh                    # Docker-based tests
./wasm/test.sh --bench            # Run benchmarks
```

### Conformance Tests
```bash
./wasm/differential-test/run-conformance.sh --max=10000
./wasm/differential-test/run-conformance.sh --all
```

### Individual Test Scripts
```bash
node wasm/scripts/run-single-test.mjs <file> --verbose
node wasm/scripts/compare-baselines.mjs 100 compiler
```

### Benchmark Harnesses (6 total)
- scanner_bench, parser_bench, emitter_bench
- real_world_bench, solver_bench, cfa_bench

---

## 7. Distribution Plans

1. **CLI**: `tsz` native binary
2. **Rust crate**: `tsz` library + CLI
3. **WASM bindings**: `@tsz/tsz` npm package
4. **Compat package**: `@tsz/tsc` for drop-in replacement
5. **Playground**: Online TypeScript playground

---

## 8. Key Insights for Next Phase

### Architectural Strengths
- ✅ Excellent parsing performance (Data-Oriented Design)
- ✅ Robust error recovery (92% improvement)
- ✅ Fast test execution (cargo-nextest)
- ✅ Parallel processing (rayon)

### Critical Gaps to Address
1. **Control Flow Graph side-table** needed for TS2454/TS2564
2. **Solver fallback** should use `Unknown` instead of `Any`
3. **Binding improvements** for TS2304 resolution
4. **Parser error recovery** for malformed ASTs

### Build/CI Recommendations
- Ensure WASM builds are green before merging
- Run conformance tests with `--max=1000` for PR validation
- Use `--all` for release validation

---

## 9. File Count Summary

- **Rust source files**: ~180+ `.rs` files
- **Lines of Rust code**: ~458,000
- **Benchmark harnesses**: 6
- **Test files**: Extensive (unit + conformance)
- **LSP features**: 20+ modules

---

## Conclusion

Project Zang has a sophisticated Rust/WASM architecture with strong foundation. The current phase (Phase 8) focuses on **conformance improvement** through Control Flow Analysis and solver strictness. The build process is well-established with Docker, wasm-pack, and Hereby integration.

**Next actionable steps** for Worker 11:
1. Await EM-3 task assignments
2. Focus on conformance priorities: TS2454/TS2564, TS2322, TS2339
3. Maintain green builds on `rust` branch
4. Run conformance tests before pushing
