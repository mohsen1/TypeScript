# Phase 6: Emitter Complete Plan

## Overview

Phase 6 transforms the Rust compiler from a type checker into a full compiler that produces JavaScript output matching TypeScript's baselines.

**Current State:**
- ✅ Phase 6.1: Study & Exploration
- ✅ Phase 6.2: JavaScript Emit (ES6+ output, type stripping)
- ⬜ Phase 6.3: ES5 Transforms (class→IIFE, arrow→function)
- ⬜ Phase 6.4: Output Format (source maps, .d.ts)
- ⬜ Phase 6.5: Baseline Validation

---

## Phase 6.1: Study & Exploration ✅ COMPLETE

### Deliverables
- [x] `wasm/benches/emitter_bench.rs` - Performance benchmarks
- [x] `specs/EMITTER_ANALYSIS.md` - Architecture analysis
- [x] `scripts/baseline-test-rust.mjs` - Baseline comparison tool

### Key Findings
1. TypeScript baselines use ES5 target (classes → IIFEs)
2. Hot paths: string building, indentation, source map tracking
3. Helper functions needed: `__generator`, `__awaiter`, `__extends`

---

## Phase 6.2: JavaScript Emit ✅ COMPLETE

### Deliverables
- [x] Type stripping (parameters, variables, return types)
- [x] Interface/type alias suppression
- [x] Declaration-only function skip
- [x] Modifier handling (strip private/protected/readonly)
- [x] Array type emit fix

### Tests
```bash
cargo test --lib  # 1007 passing
```

---

## Phase 6.3: ES5 Transforms ⬜ IN PLANNING

### Goal
Transform ES6+ output to ES5 to match TypeScript baselines.

### 6.3.1: Class Transform (~3-5 days)

**Input:**
```typescript
class Animal {
    name: string;
    constructor(name: string) {
        this.name = name;
    }
    speak() {
        console.log(this.name);
    }
}
```

**Output (ES5):**
```javascript
var Animal = /** @class */ (function () {
    function Animal(name) {
        this.name = name;
    }
    Animal.prototype.speak = function () {
        console.log(this.name);
    };
    return Animal;
}());
```

**Implementation:**
```
wasm/src/transforms/class_es5.rs
├── ClassTransformer
│   ├── transform_class_declaration()
│   ├── transform_class_expression()
│   ├── emit_constructor_function()
│   ├── emit_prototype_method()
│   ├── emit_static_property()
│   └── emit_class_heritage()  // extends
```

**Tasks:**
| Task | Effort | Priority |
|------|--------|----------|
| Basic class → IIFE | 1 day | P0 |
| Constructor handling | 0.5 day | P0 |
| Prototype methods | 0.5 day | P0 |
| Static members | 0.5 day | P1 |
| Class expressions | 0.5 day | P1 |
| Inheritance (extends) | 1 day | P1 |
| Super calls | 0.5 day | P1 |
| Property initializers | 0.5 day | P2 |

### 6.3.2: Arrow Function Transform (~1 day)

**Input:**
```typescript
const add = (a: number, b: number) => a + b;
const greet = (name: string) => {
    console.log("Hello " + name);
};
```

**Output (ES5):**
```javascript
var add = function (a, b) { return a + b; };
var greet = function (name) {
    console.log("Hello " + name);
};
```

**Implementation:**
```
wasm/src/transforms/arrow_es5.rs
├── ArrowTransformer
│   ├── transform_arrow_function()
│   ├── capture_this()  // for lexical this
│   └── emit_function_expression()
```

**Tasks:**
| Task | Effort | Priority |
|------|--------|----------|
| Basic arrow → function | 0.5 day | P0 |
| Expression body → return | 0.25 day | P0 |
| Lexical `this` capture | 0.5 day | P1 |
| Rest parameters | 0.25 day | P2 |

### 6.3.3: Generator Transform (~3-5 days)

**Input:**
```typescript
function* count() {
    yield 1;
    yield 2;
    yield 3;
}
```

**Output (ES5):**
```javascript
function count() {
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0: return [4 /*yield*/, 1];
            case 1:
                _a.sent();
                return [4 /*yield*/, 2];
            case 2:
                _a.sent();
                return [4 /*yield*/, 3];
            case 3:
                _a.sent();
                return [2 /*return*/];
        }
    });
}
```

**Implementation:**
```
wasm/src/transforms/generators.rs (started)
├── GeneratorTransformer
│   ├── collect_yield_points()
│   ├── build_state_machine()
│   ├── transform_control_flow()  // break/continue/return
│   └── emit_generator_call()
```

**Instruction Codes:**
| Code | Meaning |
|------|---------|
| 0 | Next |
| 1 | Throw |
| 2 | Return |
| 3 | Break (jump) |
| 4 | Yield |
| 5 | YieldStar |
| 6 | Catch |
| 7 | Endfinally |

**Tasks:**
| Task | Effort | Priority |
|------|--------|----------|
| State machine skeleton | 1 day | P0 |
| Basic yield transform | 0.5 day | P0 |
| Label generation | 0.5 day | P0 |
| Control flow (break/continue) | 1 day | P1 |
| try/catch handling | 1 day | P1 |
| yield* delegation | 0.5 day | P2 |
| Return value handling | 0.5 day | P1 |

### 6.3.4: Async/Await Transform (~2-3 days)

**Input:**
```typescript
async function fetchData() {
    const response = await fetch(url);
    return response.json();
}
```

**Output (ES5):**
```javascript
function fetchData() {
    return __awaiter(this, void 0, void 0, function () {
        var response;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, fetch(url)];
                case 1:
                    response = _a.sent();
                    return [2 /*return*/, response.json()];
            }
        });
    });
}
```

**Implementation:**
```
wasm/src/transforms/async_es5.rs
├── AsyncTransformer
│   ├── transform_async_function()
│   ├── wrap_in_awaiter()
│   └── reuse_generator_transform()
```

**Tasks:**
| Task | Effort | Priority |
|------|--------|----------|
| __awaiter wrapper | 0.5 day | P0 |
| await → yield transform | 0.5 day | P0 |
| Variable hoisting | 0.5 day | P1 |
| Error handling | 0.5 day | P1 |
| async methods | 0.5 day | P1 |

### 6.3.5: Template Literal Transform (~0.5 day)

**Input:**
```typescript
const msg = `Hello ${name}!`;
```

**Output (ES5):**
```javascript
var msg = "Hello " + name + "!";
```

**Tasks:**
| Task | Effort | Priority |
|------|--------|----------|
| Basic template → concat | 0.25 day | P0 |
| Tagged templates | 0.5 day | P2 |

### 6.3.6: Helper Functions (~0.5 day)

**Required Helpers:**
```
wasm/src/transforms/helpers.rs
├── __extends    // class inheritance
├── __generator  // generator state machine
├── __awaiter    // async/await
├── __rest       // rest parameters
├── __spread     // spread operator
├── __values     // for-of iteration
└── __read       // destructuring
```

**Tasks:**
| Task | Effort | Priority |
|------|--------|----------|
| Helper injection system | 0.5 day | P0 |
| __extends helper | Already done | - |
| __generator helper | Already done | - |
| __awaiter helper | Already done | - |
| Conditional injection | 0.25 day | P1 |

---

## Phase 6.4: Output Format ⬜

### 6.4.1: Source Maps (~2-3 days)

**Goal:** Generate source maps that map emitted JS back to original TS.

**Implementation:**
```
wasm/src/source_map.rs
├── SourceMapGenerator
│   ├── add_mapping(orig_line, orig_col, gen_line, gen_col)
│   ├── set_source(filename, content)
│   └── generate() -> String (JSON)
```

**Format (Source Map v3):**
```json
{
  "version": 3,
  "file": "output.js",
  "sourceRoot": "",
  "sources": ["input.ts"],
  "names": [],
  "mappings": "AAAA;AACA;..."
}
```

**Tasks:**
| Task | Effort | Priority |
|------|--------|----------|
| SourceMapGenerator struct | 0.5 day | P0 |
| VLQ encoding | 0.5 day | P0 |
| Integrate with ThinPrinter | 0.5 day | P0 |
| Inline source maps | 0.25 day | P1 |
| Source content embedding | 0.25 day | P2 |
| Multi-file support | 0.5 day | P2 |

### 6.4.2: Declaration Files (.d.ts) (~2-3 days)

**Goal:** Generate TypeScript declaration files.

**Input:**
```typescript
export function add(a: number, b: number): number {
    return a + b;
}

export class Calculator {
    private value: number;
    add(n: number): this;
}
```

**Output (.d.ts):**
```typescript
export declare function add(a: number, b: number): number;
export declare class Calculator {
    private value;
    add(n: number): this;
}
```

**Implementation:**
```
wasm/src/declaration_emitter.rs
├── DeclarationEmitter
│   ├── emit_module_declaration()
│   ├── emit_function_declaration()
│   ├── emit_class_declaration()
│   ├── emit_interface_declaration()
│   ├── infer_return_types()  // from implementation
│   └── strip_implementation()
```

**Tasks:**
| Task | Effort | Priority |
|------|--------|----------|
| Declaration mode flag | 0.25 day | P0 |
| Type preservation | 0.5 day | P0 |
| Export handling | 0.5 day | P0 |
| Class declarations | 0.5 day | P1 |
| Interface/type alias | 0.25 day | P1 |
| Module declarations | 0.5 day | P1 |
| Return type inference | 0.5 day | P2 |

### 6.4.3: Whitespace & Formatting (~1 day)

**Tasks:**
| Task | Effort | Priority |
|------|--------|----------|
| Match TypeScript indentation | 0.25 day | P0 |
| Consistent semicolons | 0.25 day | P0 |
| Newline handling | 0.25 day | P0 |
| Comment preservation | 0.5 day | P2 |

---

## Phase 6.5: Baseline Validation ⬜

### Goal
Achieve high pass rate on TypeScript compiler baselines.

### Metrics
| Baseline | Current | Target |
|----------|---------|--------|
| .js emit | 0% | 80%+ |
| .d.ts emit | 0% | 80%+ |
| .errors.txt | 32.5% | 90%+ |

### Test Commands
```bash
# Run baseline comparison
node scripts/baseline-test-rust.mjs --limit 100 --verbose

# Full compiler test
./wasm/test.sh

# Specific category
node scripts/baseline-test-rust.mjs --pattern "Class"
```

### Validation Approach
1. Start with simple tests (no classes, no generators)
2. Add class transform, validate
3. Add generator/async transform, validate
4. Fix edge cases iteratively

---

## Timeline Estimate

| Phase | Effort | Cumulative |
|-------|--------|------------|
| 6.1 Study (done) | 2 days | 2 days |
| 6.2 JS Emit (done) | 2 days | 4 days |
| 6.3.1 Class ES5 | 4 days | 8 days |
| 6.3.2 Arrow ES5 | 1 day | 9 days |
| 6.3.3 Generators | 4 days | 13 days |
| 6.3.4 Async/Await | 2 days | 15 days |
| 6.3.5 Templates | 0.5 day | 15.5 days |
| 6.3.6 Helpers | 0.5 day | 16 days |
| 6.4.1 Source Maps | 2 days | 18 days |
| 6.4.2 .d.ts | 2 days | 20 days |
| 6.4.3 Formatting | 1 day | 21 days |
| 6.5 Validation | 3 days | 24 days |

**Total Estimate: ~24 working days (5 weeks)**

---

## Priority Order

### P0 (Required for basic baseline matching)
1. Class → IIFE transform
2. Arrow → function transform
3. Basic generator support
4. Helper injection

### P1 (Required for good baseline coverage)
1. Class inheritance (extends)
2. Async/await
3. Source maps
4. .d.ts emit

### P2 (Nice to have)
1. Tagged templates
2. Complex generator control flow
3. Comment preservation
4. Advanced source map features

---

## File Structure

```
wasm/src/
├── thin_emitter.rs          # Main emitter (ES6+ mode)
├── transforms/
│   ├── mod.rs
│   ├── class_es5.rs         # Class → IIFE
│   ├── arrow_es5.rs         # Arrow → function
│   ├── generators.rs        # Generator state machines
│   ├── async_es5.rs         # Async/await
│   ├── helpers.rs           # Runtime helpers
│   └── es2015.rs            # For-of, spread, etc.
├── source_map.rs            # Source map generation
├── declaration_emitter.rs   # .d.ts generation
└── emit_options.rs          # Target, module format, etc.

scripts/
├── baseline-test-rust.mjs   # Baseline comparison
└── emit-benchmark.js        # Performance testing

specs/
├── PHASE_6_PLAN.md          # This document
├── EMITTER_ANALYSIS.md      # Architecture analysis
└── migration_plan.md        # Overall project plan
```

---

## Quick Start Commands

```bash
# Build WASM
cd wasm && wasm-pack build --target nodejs

# Run tests
cargo test --lib

# Run emitter benchmarks
cargo bench --bench emitter_bench

# Test single baseline
node scripts/baseline-test-rust.mjs --pattern "ArrowFunction" --verbose

# Test first 50 baselines
node scripts/baseline-test-rust.mjs --limit 50
```

---

## Success Criteria

Phase 6 is complete when:
1. ✅ JavaScript emit strips all TypeScript syntax
2. ⬜ ES5 target produces IIFE classes and function expressions
3. ⬜ Generators/async functions transform to state machines
4. ⬜ Source maps are generated
5. ⬜ .d.ts files are generated
6. ⬜ Baseline pass rate > 80%
