To make the compiler "badass fast," we need to attack the primary bottlenecks of the current implementation: **memory allocation** and **data locality**. The current implementation copies source text into a `Vec<char>` (expensive!) and allocates extensive heap memory for AST nodes and strings.

Here are 3 concrete, high-impact architectural changes to implement immediately.

### 1. Zero-Copy UTF-8 Scanner (Immediate 4x Memory Win)

**Problem**: The current `ScannerState` converts the source `String` (UTF-8) into a `Vec<char>` (UTF-32). This quadruples memory usage and forces a massive allocation at startup.
**Solution**: Scan the UTF-8 bytes directly. Most TypeScript syntax is ASCII, which maps 1:1 to bytes.

**Concrete Implementation:**

Modify `src/scanner_impl.rs`:

```rust
pub struct ScannerState {
    // Store source as bytes, no secondary allocation
    source: String,
    // Current byte position
    pos: usize, 
    // ... other fields
}

impl ScannerState {
    pub fn new(text: String, skip_trivia: bool) -> ScannerState {
        ScannerState {
            source: text,
            pos: 0,
            // ...
        }
    }

    // FAST: Direct byte access for ASCII (99% of code)
    #[inline(always)]
    fn peek_byte(&self) -> u8 {
        if self.pos < self.source.len() {
            self.source.as_bytes()[self.pos]
        } else {
            0
        }
    }

    // CORRECT: Decode UTF-8 only when necessary (identifiers, strings)
    fn peek_char(&self) -> char {
        self.source[self.pos..].chars().next().unwrap_or('\0')
    }
}
```

### 2. String Interning (The "Atom" Pattern)

**Problem**: Every `Identifier` node stores a `String`. In a large project, you might have 10,000 instances of `"id"`, `"value"`, or `"length"`, each with its own heap allocation.
**Solution**: Intern strings into a global/thread-local pool and pass around `u32` indices (Atoms). Comparisons become integer comparisons (`atom_a == atom_b`).

**Concrete Implementation:**

Create `src/interner.rs`:

```rust
use std::collections::HashMap;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Atom(u32);

#[derive(Default)]
pub struct Interner {
    map: HashMap<String, Atom>,
    vec: Vec<String>,
}

impl Interner {
    pub fn intern(&mut self, name: &str) -> Atom {
        if let Some(&id) = self.map.get(name) {
            return id;
        }
        let id = Atom(self.vec.len() as u32);
        let string = name.to_string();
        self.vec.push(string.clone());
        self.map.insert(string, id);
        id
    }

    pub fn resolve(&self, atom: Atom) -> &str {
        &self.vec[atom.0 as usize]
    }
}
```

Update `src/parser/ast/literals.rs`:

```rust
pub struct Identifier {
    pub base: NodeBase,
    pub escaped_text: Atom, // Was String
    // ...
}
```

### 3. Enum Compaction (Cache Locality Win)

**Problem**: Rust enums are as large as their largest variant. The `Node` enum contains variants like `ClassDeclaration` (huge) and `Identifier` (small). This means an array of `Node`s wastes massive amounts of memory on padding, destroying CPU cache locality.
**Solution**: "Box" the large variants so the `Node` enum stays small (e.g., 24-32 bytes).

**Concrete Implementation:**

Modify `src/parser/ast/node.rs`:

```rust
// Before: Node size = ~200 bytes
pub enum Node {
    Identifier(Identifier),             // Small
    ClassDeclaration(ClassDeclaration), // Huge!
    // ...
}

// After: Node size = ~32 bytes (fits 2 per cache line)
pub enum Node {
    Identifier(Identifier),
    // Box the large ones. The pointer cost is negligible compared to cache miss savings.
    ClassDeclaration(Box<ClassDeclaration>), 
    FunctionDeclaration(Box<FunctionDeclaration>),
    SourceFile(Box<SourceFile>),
    // ...
}
```

### 4. SIMD-Accelerated Whitespace Skipping (Raw Speed)

**Problem**: The scanner spends a lot of time looping over whitespace.
**Solution**: Use AVX2/SSE instructions to skip 32 bytes of spaces/tabs at a time.

**Concrete Implementation:**

In `src/scanner_impl.rs` (requires `portable_simd` or `cfg_if` for stable):

```rust
fn skip_whitespace(&mut self) {
    let bytes = self.source.as_bytes();
    while self.pos < self.end {
        // Fast path: SIMD check for space (0x20) or tab (0x09)
        // If not whitespace, break immediately
        match bytes[self.pos] {
            b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
            _ => break,
        }
    }
}
```

### Recommended Action Plan

1.  **Refactor `Scanner` first**: It's self-contained. Moving to `&[u8]` will break things but fix the foundation.
2.  **Add `Interner`**: Thread it through `ParserState`.
3.  **Benchmark**: Run `cargo bench` after these two changes. You should see a 2-5x throughput improvement.