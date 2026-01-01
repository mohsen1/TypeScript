# Autonomous Task Queue

This file contains a prioritized list of tasks for autonomous execution.
Claude should work through these in order, skipping any that are blocked.

## How to Use This File

1. Find the first `[ ]` (unchecked) task
2. Work on it until complete or blocked
3. Mark as `[x]` when done, or `[!]` if blocked
4. Move to next task
5. Commit after each completed task

---

## Phase 3.2 Remaining Tasks

### Parser Features (Priority: High)

- [ ] **Function type parsing** `(x: number) => string`
  - File: `wasm/src/parser_impl.rs`
  - Add `parse_function_type()` method
  - Handle parameter list with types
  - Handle return type after `=>`
  - Test: `type Fn = (x: number) => string`

- [ ] **Constructor type parsing** `new (x: number) => Foo`
  - File: `wasm/src/parser_impl.rs`
  - Similar to function type but starts with `new`
  - Test: `type Ctor = new (x: number) => MyClass`

- [ ] **Conditional type parsing** `T extends U ? X : Y`
  - File: `wasm/src/parser_impl.rs`
  - Add `parse_conditional_type()` after union/intersection
  - Handle `extends` keyword in type context
  - Test: `type Check<T> = T extends string ? 'yes' : 'no'`

- [ ] **Infer type parsing** `infer T`
  - File: `wasm/src/parser_impl.rs`
  - Only valid inside conditional type's extends clause
  - Test: `type Unpacked<T> = T extends Array<infer U> ? U : T`

- [ ] **Type query parsing** `typeof x`
  - File: `wasm/src/parser_impl.rs`
  - Handle `typeof` followed by expression
  - Test: `type T = typeof myVariable`

- [ ] **Keyof type parsing** `keyof T`
  - File: `wasm/src/parser_impl.rs`
  - Type operator, returns union of keys
  - Test: `type Keys = keyof { a: 1, b: 2 }`

- [ ] **Mapped type parsing** `{ [K in keyof T]: T[K] }`
  - File: `wasm/src/parser_impl.rs`
  - Handle `[K in ...]` syntax
  - Handle optional `+`/`-` modifiers
  - Test: `type Readonly<T> = { readonly [K in keyof T]: T[K] }`

### Integration (Priority: High)

- [ ] **Add --useRustParser CLI flag**
  - File: `src/tsc/tsc.ts`
  - Add flag parsing similar to `--useRustScanner`
  - File: `src/compiler/sys.ts`
  - Add `useRustParser?: boolean` to System interface

- [ ] **Create RustParser adapter function**
  - File: `src/compiler/parser.ts`
  - Create `createRustParser()` similar to `createRustScanner()`
  - Return SourceFile from Rust AST
  - Wire into `createSourceFile()` with feature flag

### AST Conversion (Priority: Medium)

- [ ] **Implement basic AST JSON serialization in Rust**
  - File: `wasm/src/parser_impl.rs`
  - Expand `getSourceFileJson()` to serialize full AST
  - Use serde_json or manual JSON building
  - Include all node properties

- [ ] **Parse Rust AST JSON in TypeScript**
  - File: `src/compiler/parser.ts`
  - Add function to convert JSON to TypeScript AST nodes
  - Use node factory to create proper TS nodes
  - Handle all ~120 node types

### Testing (Priority: Medium)

- [ ] **Add parser verification script**
  - File: `scripts/verifyParser.mjs`
  - Compare Rust AST output to TypeScript AST
  - Token-by-token and structure comparison
  - Similar to `verifyScanner.mjs`

- [ ] **Add parser unit tests**
  - File: `wasm/src/parser_impl.rs`
  - Test function declarations
  - Test class declarations
  - Test import/export
  - Test complex types

---

## Phase 4 Preview (Don't Start Yet)

- [ ] Symbol table implementation
- [ ] Scope management
- [ ] Declaration merging
- [ ] Flow analysis setup

---

## Blocked Tasks (Move Here When Stuck)

<!-- Tasks that need human intervention go here -->

---

## Completed Tasks Log

<!-- Move completed tasks here with date -->

[2026-01-01] Phase 3.1 - AST Node Definitions (120 types)
[2026-01-01] Phase 3.2 - Parser Core (statements, expressions)
[2026-01-01] Class, interface, import/export parsing
[2026-01-01] Type parsing (union, intersection, array, tuple, literal)
[2026-01-01] wasm-bindgen exports for parser
[2026-01-01] TypeScript wasm bridge for parser
