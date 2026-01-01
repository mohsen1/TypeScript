# AST Node Reference

## Node Structure

All AST nodes in the Rust parser use arena allocation:

```rust
pub struct NodeBase {
    pub kind: u16,           // SyntaxKind
    pub flags: u32,          // NodeFlags
    pub modifier_flags: u32, // ModifierFlags
    pub transform_flags: u32, // TransformFlags
    pub pos: u32,            // Start position
    pub end: u32,            // End position
    pub parent: NodeIndex,   // Parent node
    pub id: u32,             // Unique node ID
}
```

## Node Index

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeIndex(pub u32);

impl NodeIndex {
    pub const NULL: NodeIndex = NodeIndex(u32::MAX);
}
```

## Node List

```rust
pub struct NodeList {
    pub items: Vec<NodeIndex>,
    pub pos: u32,
    pub end: u32,
    pub has_trailing_comma: bool,
}
```

## Key Node Types

### Expressions
- `Identifier` - names
- `NumericLiteral`, `StringLiteral`, `BigIntLiteral`
- `BinaryExpression` - `left`, `operator_token`, `right`
- `CallExpression` - `expression`, `type_arguments`, `arguments`
- `PropertyAccessExpression` - `expression`, `name`
- `ArrayLiteralExpression`, `ObjectLiteralExpression`

### Statements
- `VariableStatement` - `declaration_list`
- `ExpressionStatement` - `expression`
- `IfStatement` - `expression`, `then_statement`, `else_statement`
- `Block` - `statements`
- `ReturnStatement` - `expression`

### Declarations
- `FunctionDeclaration` - `name`, `type_parameters`, `parameters`, `type`, `body`
- `ClassDeclaration` - `name`, `type_parameters`, `heritage_clauses`, `members`
- `VariableDeclaration` - `name`, `type`, `initializer`

### Types
- `TypeReference` - `type_name`, `type_arguments`
- `UnionType`, `IntersectionType` - `types`
- `ArrayType` - `element_type`
- `TupleType` - `elements`
- `TypeLiteral` - `members`

## SyntaxKind Ranges

- 0-166: Tokens (from `scanner.rs`)
- 167-309: Node types (from `parser.rs::syntax_kind_ext`)

## Flag Modules

```rust
// Node flags
pub mod node_flags {
    pub const NONE: u32 = 0;
    pub const LET: u32 = 1;
    pub const CONST: u32 = 2;
    // ...
}

// Modifier flags
pub mod modifier_flags {
    pub const NONE: u32 = 0;
    pub const EXPORT: u32 = 1;
    pub const PUBLIC: u32 = 4;
    // ...
}
```
