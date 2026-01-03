# Lessons from TypeScript-Go for the Rust/WASM Port

## Executive Summary

This document analyzes the TypeScript-Go (Corsa) port to extract performance patterns, architectural decisions, and lessons applicable to the Rust/WASM migration of the TypeScript compiler. The Go port represents a mature, production-ready native implementation with significant performance gains over the original TypeScript implementation (Strada).

**Key Takeaway**: The Go port achieves a 10x+ performance improvement through careful architectural decisions around memory allocation, parallelism, and lazy evaluation—not through language-level micro-optimizations.

---

## Table of Contents

1. [Project Status Comparison](#project-status-comparison)
2. [Architecture Comparison](#architecture-comparison)
3. [Performance Patterns from Go](#performance-patterns-from-go)
4. [Memory Management Strategies](#memory-management-strategies)
5. [Parallelism and Concurrency](#parallelism-and-concurrency)
6. [File Organization](#file-organization)
7. [Type System Implementation](#type-system-implementation)
8. [Recommendations for Rust/WASM](#recommendations-for-rustwasm)
9. [Appendix: Code Size Comparison](#appendix-code-size-comparison)

---

## 1. Project Status Comparison

### TypeScript-Go (Corsa) - Production Ready

| Feature | Status | Notes |
|---------|--------|-------|
| Scanner/Parser | ✅ Complete | Identical output to TS 5.9 |
| Binder | ✅ Complete | Full symbol table support |
| Checker | ✅ Complete | 31,145 lines in checker.go alone |
| Emit | 🟡 In Progress | ESNext well-supported |
| LSP | 🟡 In Progress | Most functionality working |
| Watch Mode | 🟡 Prototype | No incremental rechecking |
| API | 🔴 Not Ready | - |

### Rust/WASM Port - Early Development

| Feature | Status | Notes |
|---------|--------|-------|
| Scanner | ✅ Complete | With benchmarks |
| Parser | ✅ Complete | Arena-based nodes |
| Binder | 🟡 Basic | Symbol arena implemented |
| Checker | 🔴 Early | Type arena, basic flags only |
| Emit | 🔴 Not Started | - |

---

## 2. Architecture Comparison

### Go Architecture

```
typescript-go/internal/
├── ast/           # AST nodes (11,262 lines in ast.go)
├── binder/        # Symbol binding (2,703 lines)
├── checker/       # Type checking (31,145 lines, split into multiple files)
│   ├── checker.go      # Main checker logic
│   ├── relater.go      # Type relationship (4,926 lines)
│   ├── flow.go         # Control flow analysis (2,722 lines)
│   ├── inference.go    # Type inference
│   ├── types.go        # Type definitions
│   └── ... (14 more files)
├── compiler/      # Program coordination
├── core/          # Utilities, pools, workgroups
├── parser/        # Parsing (6,597 lines)
├── scanner/       # Lexing (2,633 lines)
└── ... (30+ more packages)
```

### Rust Architecture (Current)

```
TypeScript/wasm/src/
├── lib.rs              # Entry point
├── scanner.rs          # Token types
├── scanner_impl.rs     # Scanner implementation
├── parser/             # AST types (split into modules)
│   ├── mod.rs          # Re-exports, syntax_kind_ext
│   ├── flags.rs        # node_flags, modifier_flags
│   ├── arena.rs        # NodeArena
│   └── ast/
│       ├── mod.rs
│       ├── base.rs         # NodeBase, NodeIndex
│       ├── node.rs         # Node enum
│       ├── literals.rs     # Identifier, StringLiteral, etc.
│       ├── expressions.rs  # Expression structs
│       ├── statements.rs   # Statement structs
│       ├── declarations.rs # Declaration structs
│       ├── types.rs        # Type node structs
│       └── jsx.rs          # JSX node structs
├── parser_impl.rs      # Parser implementation
├── binder.rs           # Symbol binding
└── checker/            # Type checking (split into modules)
    ├── mod.rs          # Re-exports
    ├── state.rs        # CheckerState struct
    ├── arena.rs        # TypeArena
    ├── type_retrieval.rs   # get_type_of_node
    ├── relations.rs    # Type relationship checking
    ├── narrowing.rs    # Type narrowing
    ├── tests.rs        # Tests
    └── types/
        ├── mod.rs
        ├── type_def.rs     # Type enum
        ├── flags.rs        # type_flags, object_flags
        └── diagnostics.rs  # diagnostic_codes
```

### Key Difference: File Splitting Strategy

The Go codebase splits the checker into **21 separate files** totaling ~45,000+ lines. The Rust checker has been split into 11 modules following this pattern. Both parser and checker now use modular structures.

**Completed splits**:
- ✅ `relations.rs` - Type relationship checking
- ✅ `narrowing.rs` - Type narrowing
- ✅ `type_retrieval.rs` - Core type inference
- ✅ `state.rs` - CheckerState struct

**Future splits** (as needed):
- `flow.rs` - Control flow analysis
- `inference.rs` - Generic type inference
- `grammar.rs` - Grammar validation
- `jsx.rs` - JSX-specific logic

---

## 3. Performance Patterns from Go

### 3.1 Object Pooling (Critical for Performance)

The Go codebase extensively uses object pools to reduce GC pressure. This is the **#1 performance technique**.

**Go Pattern (from ast.go lines 54-99):**
```go
type NodeFactory struct {
    arrayTypeNodePool                 core.Pool[ArrayTypeNode]
    binaryExpressionPool              core.Pool[BinaryExpression]
    blockPool                         core.Pool[Block]
    callExpressionPool                core.Pool[CallExpression]
    identifierPool                    core.Pool[Identifier]
    // ... 35+ more typed pools
}
```

**Go Pool Implementation (from core/pool.go):**
```go
type Pool[T any] struct {
    data []T
}

func (p *Pool[T]) New() *T {
    if len(p.data) == cap(p.data) {
        nextSize := nextPoolSize(len(p.data))
        p.data = slices.Grow[[]T](nil, nextSize)
    }
    index := len(p.data)
    p.data = p.data[:index+1]
    return &p.data[index]
}
```

**Rust Equivalent (Recommended):**
```rust
// Use typed_arena crate or similar
use typed_arena::Arena;

pub struct NodeFactory<'a> {
    identifier_arena: Arena<Identifier>,
    call_expr_arena: Arena<CallExpression>,
    binary_expr_arena: Arena<BinaryExpression>,
    // ... more arenas
}
```

**Current Rust Approach:**
The Rust WASM port uses `Vec`-based arenas for types and nodes:
```rust
pub struct TypeArena {
    types: Vec<Type>,
    // singleton intrinsics cached
}

pub struct NodeArena {
    nodes: Vec<Node>,
}
```

This is a good start, but the Go approach of **type-specific pools** may provide better cache locality.

### 3.2 Lazy Evaluation with Caching

Go uses extensive caching with lazy initialization:

**Go Pattern (from checker.go):**
```go
type CachedTypeKey struct {
    kind   CachedTypeKind
    typeId TypeId
}

const (
    CachedTypeKindLiteralUnionBaseType CachedTypeKind = iota
    CachedTypeKindIndexType
    CachedTypeKindApparentType
    CachedTypeKindAwaitedType
    // ... 20+ more cache kinds
)
```

**Rust Implementation (Current):**
The Rust port has basic type caching but should expand:
```rust
pub struct TypeArena {
    types: Vec<Type>,
    // Add caches:
    literal_union_cache: HashMap<Vec<TypeId>, TypeId>,
    apparent_type_cache: HashMap<TypeId, TypeId>,
    // ...
}
```

### 3.3 Hash-Based Caching with xxh3

Go uses xxh3 for high-performance hashing:

```go
import "github.com/zeebo/xxh3"

type CacheHashKey xxh3.Uint128

var SignatureKeyErased = CacheHashKey(xxh3.HashString128("-"))
```

**Rust Equivalent:**
```rust
use xxhash_rust::xxh3::Xxh3;

// Or use FxHashMap for even faster hashing
use rustc_hash::FxHashMap;
```

### 3.4 Sync.Once Pattern for One-Time Initialization

**Go Pattern (from compiler/program.go):**
```go
type Program struct {
    commonSourceDirectory     string
    commonSourceDirectoryOnce sync.Once
    
    sourceFilesToEmitOnce sync.Once
    sourceFilesToEmit     []*ast.SourceFile
}

func (p *Program) CommonSourceDirectory() string {
    p.commonSourceDirectoryOnce.Do(func() {
        p.commonSourceDirectory = computeCommonSourceDir(p)
    })
    return p.commonSourceDirectory
}
```

**Rust Equivalent:**
```rust
use once_cell::sync::OnceCell;

pub struct Program {
    common_source_directory: OnceCell<String>,
    source_files_to_emit: OnceCell<Vec<SourceFile>>,
}

impl Program {
    pub fn common_source_directory(&self) -> &str {
        self.common_source_directory.get_or_init(|| {
            compute_common_source_dir(self)
        })
    }
}
```

---

## 4. Memory Management Strategies

### 4.1 Index-Based References (Already Adopted)

Both implementations use index-based references instead of pointers:

**Go (from ast.go):**
```go
type NodeId uint32

func (n *Node) Id() NodeId {
    return NodeId(n.id.Load())
}
```

**Rust (from parser.rs):**
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeIndex(pub u32);

impl NodeIndex {
    pub const NONE: NodeIndex = NodeIndex(u32::MAX);
}
```

### 4.2 Singleton Type Caching

**Go (from checker.go):**
```go
// Types are created once and referenced by ID
type Checker struct {
    anyType             *Type
    unknownType         *Type
    stringType          *Type
    // ... 50+ singleton types
}
```

**Rust (from checker.rs):**
```rust
pub struct TypeArena {
    pub any_type: TypeId,
    pub unknown_type: TypeId,
    pub string_type: TypeId,
    // ... similar pattern
}
```

### 4.3 Flow State Recycling

**Go (from flow.go):**
```go
type FlowState struct {
    reference       *ast.Node
    declaredType    *Type
    // ... other fields
    next            *FlowState  // For linked list recycling
}

func (c *Checker) getFlowState() *FlowState {
    f := c.freeFlowState
    if f == nil {
        f = &FlowState{}
    }
    c.freeFlowState = f.next
    return f
}

func (c *Checker) putFlowState(f *FlowState) {
    *f = FlowState{
        reduceLabels: f.reduceLabels[:0],  // Reuse slice capacity
        next:         c.freeFlowState,
    }
    c.freeFlowState = f
}
```

**Rust Recommendation:**
```rust
pub struct FlowStatePool {
    free_list: Vec<FlowState>,
}

impl FlowStatePool {
    pub fn get(&mut self) -> FlowState {
        self.free_list.pop().unwrap_or_default()
    }
    
    pub fn put(&mut self, mut state: FlowState) {
        state.clear();  // Reset to default values
        self.free_list.push(state);
    }
}
```

---

## 5. Parallelism and Concurrency

### 5.1 WorkGroup Pattern

**Go (from core/workgroup.go):**
```go
type WorkGroup interface {
    Queue(fn func())
    RunAndWait()
}

func NewWorkGroup(singleThreaded bool) WorkGroup {
    if singleThreaded {
        return &singleThreadedWorkGroup{}
    }
    return &parallelWorkGroup{}
}

type parallelWorkGroup struct {
    done atomic.Bool
    wg   sync.WaitGroup
}
```

**Key Insight**: Go provides a **configurable parallelism abstraction** that can be toggled off for debugging.

**Rust Recommendation:**
```rust
use rayon::prelude::*;

pub trait WorkGroup {
    fn queue<F: FnOnce() + Send + 'static>(&self, f: F);
    fn run_and_wait(&self);
}

pub fn new_work_group(single_threaded: bool) -> Box<dyn WorkGroup> {
    if single_threaded {
        Box::new(SingleThreadedWorkGroup::new())
    } else {
        Box::new(ParallelWorkGroup::new())
    }
}
```

### 5.2 Throttled Concurrency

**Go (from core/workgroup.go):**
```go
type ThrottleGroup struct {
    semaphore chan struct{}
    group     *errgroup.Group
}

func (tg *ThrottleGroup) Go(fn func() error) {
    tg.group.Go(func() error {
        tg.semaphore <- struct{}{}  // Acquire
        defer func() {
            <-tg.semaphore  // Release
        }()
        return fn()
    })
}
```

**Rust Equivalent:**
```rust
use tokio::sync::Semaphore;

pub struct ThrottleGroup {
    semaphore: Arc<Semaphore>,
    handles: Vec<JoinHandle<Result<(), Error>>>,
}
```

### 5.3 Checker Pool for Parallel Type Checking

**Go (from compiler/program.go):**
```go
type Program struct {
    checkerPool CheckerPool
    // ...
}

func (p *Program) initCheckerPool() {
    if p.opts.CreateCheckerPool != nil {
        p.checkerPool = p.opts.CreateCheckerPool(p)
    } else {
        p.checkerPool = newCheckerPool(p)
    }
}
```

**Key Insight**: Multiple checker instances can run in parallel on different files.

---

## 6. File Organization Recommendations

Based on the Go structure, the Rust checker should be split:

| Go File | Lines | Rust Equivalent |
|---------|-------|-----------------|
| `checker.go` | 31,145 | `checker/mod.rs` + `checker/core.rs` |
| `relater.go` | 4,926 | `checker/relater.rs` |
| `flow.go` | 2,722 | `checker/flow.rs` |
| `inference.go` | ~2,000 | `checker/inference.rs` |
| `types.go` | 1,271 | `checker/types/mod.rs` (already split) |
| `grammarchecks.go` | ~1,500 | `checker/grammar.rs` |
| `jsx.go` | ~1,200 | `checker/jsx.rs` |
| `jsdoc.go` | ~1,000 | `checker/jsdoc.rs` |
| `nodebuilder.go` | ~800 | `checker/nodebuilder.rs` |
| `services.go` | ~600 | `checker/services.rs` |
| `utilities.go` | ~500 | `checker/utilities.rs` |

### Suggested Rust Module Structure:

```
checker/
├── mod.rs              # Public API, re-exports
├── state.rs            # CheckerState struct
├── core.rs             # Main checking logic
├── relater.rs          # Type relationship checking
├── flow.rs             # Control flow analysis
├── inference.rs        # Type inference
├── narrowing.rs        # Type narrowing (exists)
├── grammar.rs          # Grammar checks
├── jsx.rs              # JSX support
├── jsdoc.rs            # JSDoc support
├── nodebuilder.rs      # Node building utilities
├── services.rs         # Language service features
├── utilities.rs        # Helper functions
├── arena.rs            # Arena allocators (exists)
├── relations.rs        # Type relations (exists)
└── types/
    ├── mod.rs          # Type definitions (exists)
    ├── type_def.rs     # Core Type enum (exists)
    ├── flags.rs        # Type flags (exists)
    ├── diagnostics.rs  # Diagnostic types (exists)
    └── signature.rs    # Signature types
```

---

## 7. Type System Implementation Patterns

### 7.1 Type Representation

**Go (from checker/types.go):**
```go
type Type struct {
    flags      TypeFlags
    objectType *ObjectTypeData  // nil for non-object types
    id         TypeId
    symbol     *ast.Symbol
    alias      AliasData
    // ... more fields
}
```

**Rust Current Approach:**
```rust
pub enum Type {
    Intrinsic(IntrinsicType),
    Literal(LiteralType),
    Object(ObjectType),
    Union(UnionType),
    // ... variants
}
```

**Trade-off Analysis:**
- Go uses a **single struct with optional fields** (more compact, less type-safe)
- Rust uses an **enum with variants** (more type-safe, larger memory footprint)

**Recommendation**: The Rust enum approach is correct for safety, but consider:
1. Using `Box<>` for large variants to reduce enum size
2. Adding a common base struct for shared fields

```rust
// Optimized pattern
pub struct TypeBase {
    pub id: TypeId,
    pub flags: TypeFlags,
    pub symbol: Option<SymbolId>,
}

pub enum TypeData {
    Intrinsic { name: String },
    Literal(Box<LiteralData>),  // Boxed for size
    Object(Box<ObjectData>),
    Union(Box<UnionData>),
    // ...
}

pub struct Type {
    pub base: TypeBase,
    pub data: TypeData,
}
```

### 7.2 Type Relation Caching

**Go (from relater.go):**
```go
type Relation struct {
    results map[CacheHashKey]RelationComparisonResult
}

func (c *Checker) isTypeAssignableTo(source *Type, target *Type) bool {
    return c.isTypeRelatedTo(source, target, c.assignableRelation)
}
```

**Rust Recommendation:**
```rust
pub struct Relation {
    results: FxHashMap<(TypeId, TypeId), RelationResult>,
}

impl Checker {
    pub fn is_type_assignable_to(&self, source: TypeId, target: TypeId) -> bool {
        self.is_type_related_to(source, target, &self.assignable_relation)
    }
}
```

---

## 8. Recommendations for Rust/WASM

### Completed ✅

1. **Split checker.rs** into 11 modules following the Go structure ✅
2. **Split parser.rs** into 12 modules for AST types ✅
3. **Use FxHashMap** for integer-keyed maps (symbol_types, node_types) ✅
4. **Pre-allocate singleton types** in the TypeArena ✅
5. **Use indices everywhere** instead of references ✅

### Future Improvements (When Needed)

6. **Add typed object pools** for frequently allocated structures
7. **Implement flow state recycling** to reduce allocations during type narrowing
8. **Add OnceCell/Lazy** for one-time computed values
9. **Implement a WorkGroup** abstraction for future parallelism

### Performance-Critical Patterns to Adopt

10. **Cache type relations** with a (source_id, target_id) -> result map

### WASM-Specific Considerations

11. **Minimize JS boundary crossings** - batch operations where possible
12. **Consider SharedArrayBuffer** for parallel WASM workers (future)
13. **Profile serialization overhead** - may need custom binary format

---

## 9. Appendix: Code Size Comparison

### Component Line Counts

| Component | TypeScript-Go | Rust/WASM | Ratio |
|-----------|--------------|-----------|-------|
| Scanner | 2,633 | ~1,500 | 1.8x |
| Parser | 6,597 | 2,242 | 2.9x |
| Binder | 2,703 | 1,889 | 1.4x |
| Checker | 45,000+ | 8,996 | 5.0x |
| **Total** | **~60,000** | **~15,000** | **4.0x** |

**Interpretation**: The Rust checker is significantly smaller because it only implements early-stage type infrastructure. The Go checker is feature-complete and represents the target scope.

### Estimated Work Remaining

Based on Go code size:
- Type Relations: ~5,000 lines to port
- Flow Analysis: ~3,000 lines to port
- Type Inference: ~2,000 lines to port
- Grammar Checks: ~2,000 lines to port
- JSX/JSDoc: ~2,500 lines to port
- Node Builder: ~3,000 lines to port
- **Total**: ~20,000-25,000 lines of Rust to write

---

## Conclusion

The TypeScript-Go port provides an excellent blueprint for the Rust/WASM implementation. The key lessons are:

1. **Performance comes from architecture, not micro-optimization**
   - Object pooling reduces GC/allocation overhead
   - Lazy evaluation with caching avoids redundant work
   - Index-based references enable efficient data structures

2. **File organization matters for large codebases**
   - Split the checker into 10-20 focused files
   - Use clear separation of concerns

3. **Parallelism is achievable but optional**
   - Design for single-threaded correctness first
   - Add parallelism with WorkGroup abstraction later

4. **The Rust implementation is on the right track**
   - Arena allocation ✓
   - Index-based references ✓
   - Type singleton caching ✓
   - Modular file structure ✓
   - FxHashMap for hot paths ✓
   - Next: more type caches, object pooling

The Rust/WASM port has completed key architectural improvements (file splitting, FxHashMap) and is ready to continue adding type checking features.
