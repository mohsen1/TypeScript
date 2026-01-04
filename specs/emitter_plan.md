# Phase 6: Emitter Implementation Plan

## Overview

The Emitter phase converts the AST back to source code (JavaScript, TypeScript, or declaration files). This is one of the most complex phases, requiring ~8,000-12,000 lines of Rust for full feature parity with TypeScript's 6,361-line emitter.ts.

## Current State

The Rust codebase already has a basic `Printer` implementation in `wasm/src/emitter.rs` (~1,600 lines):
- Basic AST node emission for most node types
- Printer options (target, module, newLine, etc.)
- Emit flags
- Indentation and output buffer management
- 30+ roundtrip tests passing

## What's Missing

Based on analysis of TypeScript's emitter.ts, the following features need implementation:

### Priority 1: Core Emission Gaps

| Task | Complexity | Lines Est. |
|------|------------|------------|
| Complete type node emission | Medium | ~400 |
| Heritage clauses (extends, implements) | Low | ~100 |
| Type parameters with constraints | Medium | ~150 |
| Computed property names | Low | ~50 |
| Template literal spans | Medium | ~100 |
| Decorators emission | Medium | ~150 |
| JSX full support (attributes, children) | Medium | ~200 |
| Modifiers (async, static, readonly, etc.) | Low | ~100 |

### Priority 2: Source Map Generation

| Task | Complexity | Lines Est. |
|------|------------|------------|
| VLQ encoding implementation | Medium | ~100 |
| SourceMapGenerator struct | Medium | ~200 |
| Position tracking during emit | Medium | ~150 |
| Source map JSON serialization | Low | ~50 |
| Inline source map support | Low | ~50 |

### Priority 3: Declaration File Emission

| Task | Complexity | Lines Est. |
|------|------------|------------|
| Declaration-only node filtering | High | ~300 |
| Export visibility tracking | Medium | ~150 |
| Type-only imports/exports | Medium | ~100 |
| Triple-slash reference handling | Low | ~50 |
| JSDoc extraction for declarations | Medium | ~150 |

### Priority 4: Comment Preservation

| Task | Complexity | Lines Est. |
|------|------------|------------|
| Leading comment emission | Medium | ~150 |
| Trailing comment emission | Medium | ~100 |
| Detached comment handling | Medium | ~100 |
| JSDoc comment formatting | Low | ~50 |

### Priority 5: JavaScript Downlevel Transforms

| Task | Complexity | Lines Est. |
|------|------------|------------|
| ES2015 class transform | High | ~500 |
| Arrow function transform | Medium | ~150 |
| Destructuring transform | High | ~400 |
| Spread operator transform | Medium | ~200 |
| Template literal transform | Medium | ~150 |
| Generator transform | Very High | ~800 |
| Async/await transform | Very High | ~600 |
| Module system transforms | High | ~500 |

### Priority 6: Advanced Features

| Task | Complexity | Lines Est. |
|------|------------|------------|
| Name generation & collision avoidance | Medium | ~200 |
| Emit helpers (__extends, __awaiter, etc.) | Medium | ~300 |
| Parenthesization rules | Medium | ~200 |
| List formatting (multiline, trailing comma) | Medium | ~150 |
| Build info (.tsbuildinfo) emission | Low | ~100 |

---

## Implementation Phases

### Phase 6.1: Complete Basic Emission (Est. 2-3 days)

**Goal**: Emit valid TypeScript/JavaScript for all node types without transforms.

Tasks:
1. [ ] Add type node emission (TypeReference, UnionType, IntersectionType, etc.)
2. [ ] Add heritage clause emission (extends, implements)
3. [ ] Add type parameter emission with constraints
4. [ ] Add decorator emission
5. [ ] Add modifier emission (public, private, static, readonly, async, etc.)
6. [ ] Add computed property name emission
7. [ ] Add template literal span emission
8. [ ] Complete JSX emission (attributes, spread, fragments)
9. [ ] Add EnumMember emission
10. [ ] Add NamedImports/NamedExports emission

**Verification**: All parser roundtrip tests pass with full fidelity.

### Phase 6.2: Source Map Support (Est. 2 days)

**Goal**: Generate accurate source maps for debugging.

Tasks:
1. [ ] Implement VLQ (Variable Length Quantity) encoding
2. [ ] Create SourceMapGenerator struct
3. [ ] Track source positions during emission
4. [ ] Add source file content inclusion option
5. [ ] Generate inline source maps (data URL)
6. [ ] Generate external .map files
7. [ ] Add source map tests

**Verification**: Generated source maps work correctly in Chrome DevTools.

### Phase 6.3: Comment Preservation (Est. 1-2 days)

**Goal**: Preserve comments in output when not using removeComments.

Tasks:
1. [ ] Parse and store comment positions during scanning
2. [ ] Emit leading comments before nodes
3. [ ] Emit trailing comments after nodes
4. [ ] Handle detached comments between statements
5. [ ] Support removeComments option
6. [ ] JSDoc comment special handling

**Verification**: Roundtrip tests preserve all comments.

### Phase 6.4: Declaration File Emission (Est. 2-3 days)

**Goal**: Generate .d.ts files from TypeScript source.

Tasks:
1. [ ] Create DeclarationEmitter wrapping Printer
2. [ ] Filter to only exported declarations
3. [ ] Strip function/method bodies
4. [ ] Preserve type annotations only
5. [ ] Handle type-only imports/exports
6. [ ] Emit triple-slash references
7. [ ] Generate .d.ts.map source maps

**Verification**: Generated .d.ts files type-check correctly.

### Phase 6.5: JavaScript Transforms - Basics (Est. 3-4 days)

**Goal**: Downlevel ES2015+ to ES5.

Tasks:
1. [ ] Arrow function → function expression
2. [ ] Template literal → string concatenation
3. [ ] Spread in array literal → Array.concat
4. [ ] Spread in call → Function.apply
5. [ ] Shorthand property → full property
6. [ ] Computed property names → bracket notation
7. [ ] Default parameters → || fallback
8. [ ] Rest parameters → arguments slice

**Verification**: Transformed code runs correctly in ES5 environments.

### Phase 6.6: JavaScript Transforms - Classes (Est. 3-4 days)

**Goal**: Transform ES6 classes to ES5 prototypes.

Tasks:
1. [ ] Class declaration → function + prototype
2. [ ] Constructor → function body
3. [ ] Method → prototype assignment
4. [ ] Static method → function property
5. [ ] Extends → __extends helper
6. [ ] Super calls → parent.call(this)
7. [ ] Property initializers → constructor assignments
8. [ ] Accessor properties → Object.defineProperty

**Verification**: Class semantics preserved in ES5 output.

### Phase 6.7: JavaScript Transforms - Async/Generators (Est. 4-5 days)

**Goal**: Transform async/await and generators to ES5.

Tasks:
1. [ ] Generator function → state machine
2. [ ] Yield expression → state transitions
3. [ ] Async function → __awaiter helper
4. [ ] Await expression → Promise chain
5. [ ] Async generator → combined state machine
6. [ ] For-await-of → async iteration protocol

**Verification**: Async code behaves correctly when transformed.

### Phase 6.8: Module System Transforms (Est. 2-3 days)

**Goal**: Support CommonJS, AMD, UMD, System module output.

Tasks:
1. [ ] ES modules → CommonJS (require/exports)
2. [ ] ES modules → AMD (define)
3. [ ] ES modules → UMD (AMD + CommonJS + global)
4. [ ] ES modules → SystemJS
5. [ ] Import/export elision for type-only
6. [ ] __esModule marker
7. [ ] Interop helpers (__importDefault, __importStar)

**Verification**: Module output works in Node.js and browsers.

### Phase 6.9: Polish & Optimization (Est. 2 days)

**Goal**: Match TypeScript output quality and performance.

Tasks:
1. [ ] Parenthesization rules (operator precedence)
2. [ ] List formatting (trailing commas, multiline)
3. [ ] Name collision avoidance
4. [ ] Emit helpers consolidation
5. [ ] Performance optimization
6. [ ] Build info emission

**Verification**: Byte-for-byte output matching TypeScript in common cases.

---

## Test Strategy

### Unit Tests
- Individual emit function tests for each node type
- Roundtrip tests (parse → emit → parse → emit)
- Source map accuracy tests
- Comment preservation tests

### Integration Tests
- Run against TypeScript's compiler test suite
- Compare output with TypeScript compiler
- Verify transformed code executes correctly

### Verification Script
Create `scripts/verifyEmitter.mjs`:
```javascript
// Compare Rust emitter output with TypeScript emitter
// for a set of representative source files
```

---

## Dependencies

The Emitter depends on:
- **Parser** (complete) - AST nodes to emit
- **Scanner** (complete) - Token text for keywords/operators
- **Binder** (complete) - Symbol info for name generation
- **Checker** (complete) - Type info for declaration emit

---

## Estimated Total Effort

| Phase | Estimated Days |
|-------|----------------|
| 6.1 Complete Basic Emission | 2-3 |
| 6.2 Source Map Support | 2 |
| 6.3 Comment Preservation | 1-2 |
| 6.4 Declaration File Emission | 2-3 |
| 6.5 JavaScript Transforms - Basics | 3-4 |
| 6.6 JavaScript Transforms - Classes | 3-4 |
| 6.7 JavaScript Transforms - Async | 4-5 |
| 6.8 Module System Transforms | 2-3 |
| 6.9 Polish & Optimization | 2 |
| **Total** | **22-28 days** |

---

## Next Steps

1. Start with Phase 6.1 - complete the basic emission gaps
2. Focus on getting all roundtrip tests to full fidelity
3. Add source map support early for debugging
4. Implement transforms incrementally, testing each

---

## Files to Create/Modify

```
wasm/src/
├── emitter.rs          # Extend existing (currently ~1,600 lines)
├── sourcemap.rs        # NEW: Source map generation
├── declaration.rs      # NEW: Declaration file emission
├── transforms/         # NEW: JavaScript transforms
│   ├── mod.rs
│   ├── es2015.rs       # Class, arrow, destructuring
│   ├── es2016.rs       # Exponentiation
│   ├── es2017.rs       # Async/await
│   ├── es2018.rs       # Spread, async iteration
│   ├── es2019.rs       # Object.fromEntries
│   ├── es2020.rs       # Optional chaining, nullish
│   ├── es2021.rs       # Logical assignment
│   ├── generators.rs   # Generator state machine
│   ├── modules.rs      # Module system transforms
│   └── jsx.rs          # JSX transform
└── helpers.rs          # NEW: __extends, __awaiter, etc.
```

## Success Criteria

Phase 6 is complete when:
1. All node types emit correctly (TypeScript and JavaScript)
2. Source maps are accurate and work in debuggers
3. Declaration files (.d.ts) generate correctly
4. Comments are preserved when not stripped
5. JavaScript downlevel transforms produce correct ES5
6. Module systems (CommonJS, ES6, AMD, UMD) work
7. Performance matches or exceeds TypeScript emitter
8. `npx hereby runtests-parallel` passes
