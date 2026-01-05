# Phase 6: Emitter Analysis

## Executive Summary

Analysis of the ThinEmitter architecture for Phase 6 completion. Focus areas:
1. Performance bottlenecks and optimization opportunities
2. Generator transform implementation strategy
3. TypeScript baseline format matching

---

## 1. Current Architecture

### ThinEmitter (~1,900 LOC)

Location: `wasm/src/thin_emitter.rs`

#### Structure
```rust
pub struct ThinPrinter<'a> {
    arena: &'a ThinNodeArena,    // 16-byte node access
    output: String,              // Output buffer
    indent_level: u32,           // Current indentation
    indent_str: String,          // "    " (4 spaces)
    new_line: String,            // "\n" or "\r\n"
    options: PrinterOptions,
    at_line_start: bool,
    output_line: u32,            // For source maps
    output_column: u32,
}
```

#### Hot Path Analysis

**String Building (Lines 100-145)**
```rust
fn write(&mut self, text: &str) {
    // Check indent needed on every write
    if self.at_line_start && self.indent_level > 0 {
        for _ in 0..self.indent_level {
            self.output.push_str(&self.indent_str);  // Potential allocation
        }
    }
    // Character-by-character iteration for line tracking
    for ch in text.chars() {
        if ch == '\n' { self.output_line += 1; ... }
    }
    self.output.push_str(text);
}
```

**Issues Identified:**
1. Indent string pushed repeatedly (could pre-compute)
2. Character iteration for every write (expensive)
3. `String::push_str` may reallocate

### Performance Recommendations

#### 1. Pre-compute Indentation
```rust
// Current: O(indent_level) per write
for _ in 0..self.indent_level {
    self.output.push_str(&self.indent_str);
}

// Better: O(1) lookup
const INDENT_CACHE: [&str; 16] = [
    "", "    ", "        ", "            ", // ... up to 16 levels
];
fn get_indent(level: u32) -> &'static str {
    INDENT_CACHE.get(level as usize).unwrap_or(&INDENT_CACHE[15])
}
```

#### 2. Batch Line Counting
```rust
// Current: Character iteration on every write
for ch in text.chars() { ... }

// Better: Use memchr for newline counting
fn count_newlines(text: &str) -> u32 {
    memchr::memchr_iter(b'\n', text.as_bytes()).count() as u32
}

// Only update column if needed for source maps
fn write_fast(&mut self, text: &str) {
    if self.at_line_start && self.indent_level > 0 {
        self.output.push_str(get_indent(self.indent_level));
        self.at_line_start = false;
    }
    let newlines = count_newlines(text);
    if newlines > 0 {
        self.output_line += newlines;
        // Only compute column if needed
    }
    self.output.push_str(text);
}
```

#### 3. Pre-allocate Output Buffer
```rust
// Current: 1KB initial capacity
output: String::with_capacity(1024)

// Better: Estimate based on input size
// Typical JS emit is 0.8-1.2x input size
pub fn with_estimated_size(arena: &'a ThinNodeArena, source_len: usize) -> Self {
    ThinPrinter {
        output: String::with_capacity(source_len + source_len / 4),
        ...
    }
}
```

---

## 2. Generator Transforms

### TypeScript's Approach (generators.ts)

TypeScript transforms generators into state machines with these components:

1. **Intermediate Representation (IR)**
   - Labels for jump targets
   - Operations: nop, mark, br, yield, return, throw
   - Code blocks: exception, with, switch, loop, labeled

2. **State Machine Output**
```javascript
// Input
function* gen() {
    yield 1;
    yield 2;
}

// Output
function gen() {
    return __generator(this, function (_a) {
        switch (_a.label) {
            case 0: return [4 /*yield*/, 1];
            case 1:
                _a.sent();
                return [4 /*yield*/, 2];
            case 2:
                _a.sent();
                return [2 /*return*/];
        }
    });
}
```

3. **Instruction Codes**
   - 0: Next
   - 1: Throw
   - 2: Return
   - 3: Break (jump)
   - 4: Yield
   - 5: YieldStar
   - 6: Catch
   - 7: Endfinally

### Rust Implementation Strategy

#### Phase 6.2a: Basic Generator Transform

```rust
// wasm/src/transforms/generators.rs

pub struct GeneratorTransformer {
    /// Current label counter
    label_id: u32,
    /// Yield point locations
    yield_points: Vec<YieldPoint>,
    /// Active blocks (try/catch, loops, etc.)
    block_stack: Vec<CodeBlock>,
}

struct YieldPoint {
    label: u32,
    expression: NodeIndex,
    is_delegating: bool,  // yield*
}

enum CodeBlock {
    Exception { try_label: u32, catch_label: u32, finally_label: u32, end_label: u32 },
    Loop { continue_label: u32, break_label: u32 },
    Labeled { name: String, break_label: u32 },
}

impl GeneratorTransformer {
    pub fn transform_generator_body(
        &mut self,
        body: NodeIndex,
        ctx: &mut TransformContext,
    ) -> NodeIndex {
        // 1. Collect yield points
        self.visit_body(body, ctx);
        
        // 2. Generate state machine
        self.build_state_machine(ctx)
    }
    
    fn build_state_machine(&self, ctx: &mut TransformContext) -> NodeIndex {
        // Build switch statement with cases for each label
        let cases = self.yield_points.iter().map(|yp| {
            // return [4 /*yield*/, expression]
            self.create_yield_return(yp, ctx)
        }).collect();
        
        // __generator(this, function(_a) { switch (_a.label) { cases } })
        self.wrap_in_generator_call(cases, ctx)
    }
}
```

#### Phase 6.2b: Control Flow

Handle break/continue/return within generators:

```rust
fn transform_break_statement(&mut self, label: Option<&str>) -> NodeIndex {
    // Find target block
    let target = self.find_break_target(label);
    
    // return [3 /*break*/, target_label]
    self.create_break_return(target)
}

fn transform_try_statement(&mut self, try_stmt: &TryStatement) -> Vec<NodeIndex> {
    let try_label = self.new_label();
    let catch_label = self.new_label();
    let finally_label = self.new_label();
    let end_label = self.new_label();
    
    self.block_stack.push(CodeBlock::Exception {
        try_label, catch_label, finally_label, end_label
    });
    
    // Transform body
    // ...
    
    self.block_stack.pop();
}
```

---

## 3. Baseline Format Matching

### Key Differences Found

1. **Whitespace**
   - TypeScript: Uses consistent 4-space indentation
   - Current: Matches (4 spaces)
   
2. **Semicolons**
   - TypeScript: Always emits semicolons
   - Current: Has `omit_trailing_semicolon` option (default: emit)
   
3. **Object Literals**
   - TypeScript: Multi-line for >1 property
   - Current: Single line with spaces

4. **Function Bodies**
   - TypeScript: Opening brace on same line
   - Current: Matches

5. **Statement Termination**
   - TypeScript: Newline after each statement
   - Current: Matches

### Required Fixes

```rust
// Object literal formatting
fn emit_object_literal(&mut self, node: &ThinNode) {
    let obj = self.arena.get_literal_expr(node)?;
    
    if obj.elements.nodes.is_empty() {
        self.write("{}");
        return;
    }
    
    // Multi-line for >1 property
    if obj.elements.nodes.len() > 1 {
        self.write("{");
        self.write_line();
        self.increase_indent();
        for (i, &prop) in obj.elements.nodes.iter().enumerate() {
            self.emit(prop);
            if i < obj.elements.nodes.len() - 1 {
                self.write(",");
            }
            self.write_line();
        }
        self.decrease_indent();
        self.write("}");
    } else {
        // Single property: { key: value }
        self.write("{ ");
        self.emit(obj.elements.nodes[0]);
        self.write(" }");
    }
}
```

---

## 4. Missing Emit Methods

### Critical (blocking baseline tests)

1. **Return Statement Expression**
   ```rust
   fn emit_return_statement(&mut self, node: &ThinNode) {
       let Some(ret) = self.arena.get_return_statement(node) else { return; };
       self.write("return");
       if !ret.expression.is_none() {
           self.write(" ");
           self.emit(ret.expression);
       }
       self.write_semicolon();
   }
   ```

2. **Template Literals**
   ```rust
   fn emit_template_expression(&mut self, node: &ThinNode) {
       let Some(tpl) = self.arena.get_template_expr(node) else { return; };
       self.write("`");
       self.write(&tpl.head.text);
       for span in &tpl.spans {
           self.write("${");
           self.emit(span.expression);
           self.write("}");
           self.write(&span.literal.text);
       }
       self.write("`");
   }
   ```

3. **Yield Expression**
   ```rust
   fn emit_yield_expression(&mut self, node: &ThinNode) {
       let Some(yield_expr) = self.arena.get_yield_expr(node) else { return; };
       self.write("yield");
       if yield_expr.asterisk_token {
           self.write("*");
       }
       if !yield_expr.expression.is_none() {
           self.write(" ");
           self.emit(yield_expr.expression);
       }
   }
   ```

4. **Await Expression**
   ```rust
   fn emit_await_expression(&mut self, node: &ThinNode) {
       let Some(await_expr) = self.arena.get_await_expr(node) else { return; };
       self.write("await ");
       self.emit(await_expr.expression);
   }
   ```

5. **Spread Element**
   ```rust
   fn emit_spread_element(&mut self, node: &ThinNode) {
       let Some(spread) = self.arena.get_spread_element(node) else { return; };
       self.write("...");
       self.emit(spread.expression);
   }
   ```

6. **Parenthesized Expression**
   ```rust
   fn emit_parenthesized(&mut self, node: &ThinNode) {
       let Some(paren) = self.arena.get_parenthesized(node) else { return; };
       self.write("(");
       self.emit(paren.expression);
       self.write(")");
   }
   ```

---

## 5. Helper Functions

### Required Helpers for ES5 Target

```rust
// wasm/src/transforms/helpers.rs

pub const GENERATOR_HELPER: &str = r#"
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
"#;

pub const AWAITER_HELPER: &str = r#"
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
"#;
```

---

## 6. Action Items

### Phase 6.1 (Study & Exploration) ✅
- [x] Benchmark infrastructure created
- [x] Hot path analysis documented
- [x] TypeScript emitter studied
- [x] This document created

### Phase 6.2 (JavaScript Emit) ✅
- [x] Add missing accessors to ThinNodeArena (parenthesized, template, yield, await, spread)
- [x] Fix array type parsing and emission
- [x] Implement JavaScript emit mode (strip types, interfaces, declarations)
- [x] Skip TypeScript-only modifiers (private, protected, readonly)
- [x] Create `generators.rs` transformer (started)
- [ ] Full state machine generation for ES5 (future)

### Phase 6.3 (ES5 Transforms - Future)
TypeScript baselines use ES5 target. Our emitter produces ES6+.
- [ ] Class → IIFE with prototype methods
- [ ] Arrow function → regular function
- [ ] Generator state machines
- [ ] Template literals → string concatenation

### Phase 6.4 (Output Format Matching)
- [x] Fix object literal multi-line formatting
- [x] Fix return statement expression emission
- [x] Fix template literal emission
- [x] Fix parenthesized expression emission
- [ ] Run baseline-test-rust.mjs for ES6+ comparison
- [ ] Source map generation

---

## Benchmarking Commands

```bash
# Run emitter benchmarks
cd wasm && cargo bench --bench emitter_bench

# Quick throughput test
cargo bench -- emit_throughput

# Compare with parser
cargo bench -- printer_comparison

# Run baseline comparison
node scripts/baseline-test-rust.mjs --limit 100 --verbose
```
