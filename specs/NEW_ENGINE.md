# Query-Based Type Checking Architecture

> **Status**: ✅ Implemented in `wasm/src/solver/`

This document summarizes the architectural shift from imperative AST walking to a query-based semantic database.

---

## Core Concepts (All Implemented)

### 1. Types as Integers (Interning) ✅
```rust
pub struct TypeId(pub u32);  // 4 bytes, O(1) equality
```
- Same structure = same TypeId (structural deduplication)
- All types interned in `TypeInterner` (`solver/intern.rs`)

### 2. Coinductive Subtyping ✅
- Recursive types handled via cycle detection
- "Provisional true" on cycle, verify on unwinding
- Implemented in `SubtypeChecker` (`solver/subtype.rs`)

### 3. Constraint-Based Inference ✅
- Union-Find via `ena` crate for inference variables
- Snapshot/rollback for overload resolution
- Implemented in `InferenceContext` (`solver/infer.rs`)

### 4. Lazy Lowering ✅
- AST → TypeId on demand, not eagerly
- Errors isolated per expression
- Implemented in `TypeLowering` (`solver/lower.rs`)

---

## Architecture

```
ThinParser → ThinNodeArena → ThinBinder → ThinChecker
                                              ↓
                                     Solver (Pure Type Logic)
                                              ├─ TypeInterner
                                              ├─ SubtypeChecker  
                                              ├─ InferenceContext
                                              └─ TypeEvaluator

ThinChecker = WHERE (AST traversal, scoping)
Solver = WHAT (type logic, relations)
```

See `specs/SOLVER.md` for the mathematical foundations.
