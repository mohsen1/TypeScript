# Rust Migration Patterns

## Borrow Checker Patterns

### Problem: Cannot borrow `*self` as mutable more than once

```rust
// ❌ This fails:
self.method_a(self.method_b())

// ✅ Extract to local variable:
let value = self.method_b();
self.method_a(value)
```

### Problem: Borrowing in loops

```rust
// ❌ This fails:
while condition {
    let node = self.arena.get(idx);  // immutable borrow
    self.next_token();                // mutable borrow
}

// ✅ Get values before mutable operations:
while condition {
    let kind = self.arena.get(idx).map(|n| n.base().kind);
    self.next_token();
    // use kind here
}
```

## wasm-bindgen Patterns

### Exposing Structs

```rust
#[wasm_bindgen]
pub struct MyStruct {
    // Simple types work directly
    pub count: u32,

    // Complex types need skip
    #[wasm_bindgen(skip)]
    pub arena: NodeArena,
}
```

### Constructor and Methods

```rust
#[wasm_bindgen]
impl MyStruct {
    #[wasm_bindgen(constructor)]
    pub fn new(text: String) -> MyStruct { ... }

    #[wasm_bindgen(js_name = myMethod)]
    pub fn my_method(&self) -> u32 { ... }

    // Getters for JS
    #[wasm_bindgen(getter)]
    pub fn count(&self) -> u32 { self.count }
}
```

### Separate wasm and internal impls

```rust
// Public wasm-bindgen API
#[wasm_bindgen]
impl ParserState {
    #[wasm_bindgen(constructor)]
    pub fn new(text: String) -> ParserState { ... }
}

// Internal methods (no wasm_bindgen)
impl ParserState {
    fn parse_expression(&mut self) -> NodeIndex { ... }
}
```

## Arena Allocation

### NodeIndex Pattern

```rust
#[derive(Clone, Copy)]
pub struct NodeIndex(pub u32);

impl NodeIndex {
    pub const NULL: NodeIndex = NodeIndex(u32::MAX);

    pub fn is_null(&self) -> bool {
        *self == Self::NULL
    }
}
```

### Adding Nodes

```rust
fn create_identifier(&mut self, text: String) -> NodeIndex {
    let node = Identifier {
        base: NodeBase::new(IDENTIFIER, pos, end),
        text,
    };
    self.arena.add(Node::Identifier(node))
}
```

### Accessing Nodes

```rust
if let Some(node) = self.arena.get(idx) {
    match node {
        Node::Identifier(id) => { /* use id */ }
        _ => { /* other node types */ }
    }
}
```

## Token Parsing Patterns

### Consume and Check

```rust
fn parse_expected(&mut self, kind: SyntaxKind) -> bool {
    if self.token() == kind {
        self.next_token();
        true
    } else {
        self.error(/* ... */);
        false
    }
}

fn parse_optional(&mut self, kind: SyntaxKind) -> bool {
    if self.token() == kind {
        self.next_token();
        true
    } else {
        false
    }
}
```

### Lookahead

```rust
fn is_start_of_type(&self) -> bool {
    matches!(self.token(),
        SyntaxKind::Identifier |
        SyntaxKind::OpenBracketToken |
        SyntaxKind::OpenBraceToken |
        // ...
    )
}
```

## AST Node Creation

### Pattern: finish_node

```rust
fn finish_node<T>(&mut self, node: T) -> NodeIndex
where
    Node: From<T>,
{
    let end = self.get_token_start();
    let id = self.node_count;
    self.node_count += 1;

    let idx = self.arena.add(Node::from(node));

    if let Some(n) = self.arena.get_mut(idx) {
        n.base_mut().end = end;
        n.base_mut().id = id;
    }

    idx
}
```

## Flag Constants Pattern

For wasm-bindgen compatibility, use const modules instead of enums:

```rust
pub mod node_flags {
    pub const NONE: u32 = 0;
    pub const LET: u32 = 1;
    pub const CONST: u32 = 2;
    pub const USING: u32 = 4;
    // Bitwise OR for combinations
}

// Usage:
let flags = node_flags::LET | node_flags::CONST;
```
