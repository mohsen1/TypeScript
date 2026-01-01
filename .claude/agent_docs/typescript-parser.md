# TypeScript Parser Reference

## Entry Point

The TypeScript parser is in `src/compiler/parser.ts` and exposes:

```typescript
export function createSourceFile(
    fileName: string,
    sourceText: string,
    languageVersionOrOptions: ScriptTarget | CreateSourceFileOptions,
    setParentNodes?: boolean,
    scriptKind?: ScriptKind
): SourceFile
```

## Parser Namespace

The parser implementation is in `namespace Parser`:

- Uses shared scanner: `var scanner = createScanner(ScriptTarget.Latest, true)`
- Uses node factory: `var factory = createNodeFactory(...)`
- State variables: `fileName`, `sourceText`, `currentToken`, `nodeCount`, etc.

## Key Functions

### Statement Parsing
- `parseStatement()` - main dispatcher
- `parseVariableStatement()`
- `parseFunctionDeclaration()`
- `parseClassDeclaration()`
- `parseIfStatement()`, `parseWhileStatement()`, etc.

### Expression Parsing
- `parseExpression()` - entry point
- `parseBinaryExpressionOrHigher()` - handles precedence
- `parseUnaryExpressionOrHigher()`
- `parseLeftHandSideExpressionOrHigher()`
- `parsePrimaryExpression()`

### Type Parsing
- `parseType()` - entry point
- `parseUnionOrIntersectionType()`
- `parseArrayTypeOrHigher()`
- `parseTypeReference()`
- `parseTupleType()`, `parseTypeLiteral()`

## Operator Precedence

From lowest to highest:
1. `||`, `??` - Logical OR, Nullish coalescing
2. `&&` - Logical AND
3. `|` - Bitwise OR
4. `^` - Bitwise XOR
5. `&` - Bitwise AND
6. `==`, `!=`, `===`, `!==` - Equality
7. `<`, `>`, `<=`, `>=`, `instanceof`, `in` - Relational
8. `<<`, `>>`, `>>>` - Shift
9. `+`, `-` - Additive
10. `*`, `/`, `%` - Multiplicative
11. `**` - Exponentiation (right-associative)

## Automatic Semicolon Insertion (ASI)

Parser must handle ASI per ECMAScript spec:
- Before `}` closing a block
- After line terminator when next token is unexpected
- At end of input

## Factory Pattern

TypeScript uses a node factory for creating AST nodes:

```typescript
factoryCreateIdentifier(text: string)
factoryCreateNumericLiteral(value: string)
factoryCreateStringLiteral(text: string)
factoryCreateBinaryExpression(left, operator, right)
// etc.
```
