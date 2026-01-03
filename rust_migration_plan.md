# Task List

## Status

| Phase | Component      | Status | Tests |
|-------|----------------|--------|-------|
| 0     | Infrastructure | ✅ DONE | - |
| 1     | Utilities      | ✅ DONE | 21 Rust |
| 2     | Scanner        | ✅ 95%  | 22 Rust |
| 3     | Parser         | ✅ 98%  | 81 Rust + 19 TS |
| 4     | Binder         | ✅ DONE | 10 TS |
| 5     | Type Checker   | 🟡 67%  | 239 Rust |
| 6     | Emitter        | ⬜ 0%   | - |
| 7     | Language Svc   | ⬜ 0%   | - |
| 8     | Full Rust      | ⬜ 0%   | - |

---

## Phase 5: Type Checker (IN PROGRESS)

### ✅ Completed (5.1 - 5.29)

All of the following are complete:

- **5.1** Type struct, TypeFlags, ObjectFlags, TypeArena, 14 intrinsic types
- **5.2** CheckerState, type caching, assignability, basic inference
- **5.3** Function types, parameters, return types, optional/rest params
- **5.4** Generic types, type parameters, type arguments, instantiation
- **5.5** Object types, property access, type literals, interfaces
- **5.6** Type narrowing (typeof guards, nullable guards)
- **5.7** Generic call inference, explicit type args, inference from args
- **5.8** instanceof type guards
- **5.9** Control flow type guard analysis
- **5.10** Class types, constructor types, new expressions
- **5.11** Index signature parsing
- **5.12** Index signature type checking
- **5.13** Array and tuple types
- **5.14** Array/tuple assignability
- **5.15** Readonly array/tuple types
- **5.16** Rest and optional tuple elements
- **5.17** Spread in array literals
- **5.18** Conditional type evaluation
- **5.19** Template literal types (basic)
- **5.20** Mapped type evaluation
- **5.21** Infer types in conditional types
- **5.22** Keyof type evaluation (basic)
- **5.23** Keyof object types (full), mapped type instantiation, infer pattern matching
- **5.25** Intersection type simplification
- **5.26** Template literal type instantiation
- **5.27** Union type simplification
- **5.28** Distributive conditional types
- **5.29** Index access on arrays/tuples
- **5.30** Property access on unions (get common property type across all members)
- **5.31** Excess property checks (basic detection of extra properties in fresh object literals)
- **5.32** Index signature support in type literals

### 🚧 In Progress / Next Up

- [ ] **Contextual typing**
    - Infer parameter types from context
    - `arr.map(x => x + 1)` infers `x: number`

- [ ] **Discriminated unions**
    - Narrow union by discriminant property
    - `if (obj.kind === "a") { ... }`

- [ ] **Exhaustiveness checking**
    - Ensure all union cases handled in switch
    - Never type for unhandled cases

- [ ] **Diagnostics**
    - Port error message generation
    - Match exact TypeScript error codes
    - Related information spans

---

## Phase 6: Emitter (NOT STARTED)

### 6.1 Printer
- [ ] Port AST → text printing logic
- [ ] Handle formatting and whitespace
- [ ] Source map generation

### 6.2 Transformers
- [ ] Port downlevel transforms (ES2015 → ES5)
- [ ] Module system transforms (ESM ↔ CJS)
- [ ] JSX transform

### 6.3 Declaration Emit
- [ ] Port `.d.ts` generation
- [ ] Handle visibility and export pruning

---

## Phase 7: Language Service (NOT STARTED)

### 7.1 Completions
- [ ] Port completion entry generation
- [ ] Symbol filtering and ranking

### 7.2 Quick Info / Hover
- [ ] Port display parts generation
- [ ] Type-to-string rendering

### 7.3 Navigation
- [ ] Go to definition
- [ ] Find all references
- [ ] Rename support

---

## Phase 8: Full Rust Mode (NOT STARTED)

### 8.1 Standalone Binary
- [ ] Create native `tsc` binary (no Node.js required)
- [ ] CLI argument parsing in Rust
- [ ] File system abstraction

### 8.2 Performance Optimization
- [ ] Profile and optimize hot paths
- [ ] Implement parallel type checking
- [ ] Memory usage optimization

### 8.3 Compatibility Mode
- [ ] Maintain wasm build for Node.js users
- [ ] Ensure identical behavior between native and wasm builds

---

## Blocked

_None currently_

---

## Deferred

- JSX scanning (`scanJsxIdentifier`, `scanJsxAttributeValue`)
- JSDoc scanning (`scanJsDocToken`)
- Remaining rescan methods
- Experimental syntax support
- Collections (`Map`, `Set`, `MultiMap` equivalents)
