---
name: typescript-compiler
description: Deep understanding of the TypeScript compiler internals. Use when understanding scanner.ts, parser.ts, checker.ts, or any src/compiler code. Helps with mapping TypeScript patterns to Rust equivalents.
allowed-tools: Read, Grep, Glob
---

# TypeScript Compiler Expert

You understand the internal architecture of the TypeScript compiler.

## When to Use This Skill

- Understanding how TypeScript parses specific syntax
- Finding where a feature is implemented in `src/compiler/`
- Mapping TypeScript patterns to Rust equivalents
- Understanding AST node structure

## Compiler Architecture

```
src/compiler/
├── scanner.ts      # Lexer - converts text to tokens
├── parser.ts       # Parser - converts tokens to AST
├── binder.ts       # Symbol table creation
├── checker.ts      # Type checking (50% of compiler)
├── emitter.ts      # Code generation
├── types.ts        # Type definitions
├── utilities*.ts   # Helper functions
└── factory.ts      # AST node factory
```

## Key Files Reference

For detailed information, see:
- [Parser Reference](../agent_docs/typescript-parser.md) - Parser namespace and functions
- [AST Nodes](../agent_docs/ast-nodes.md) - Node structure reference

## Quick Lookups

### Find how a syntax is parsed
```bash
# Search in parser.ts
grep -n "parseIdentifier\|parseExpression" src/compiler/parser.ts
```

### Find a SyntaxKind usage
```bash
grep -n "SyntaxKind.ClassDeclaration" src/compiler/*.ts
```

### Find node factory usage
```bash
grep -n "factory.create" src/compiler/parser.ts | head -20
```

## Important Patterns

### Scanner Usage in Parser
```typescript
var scanner = createScanner(ScriptTarget.Latest, /*skipTrivia*/ true);
// ...
function token(): SyntaxKind {
    return currentToken;
}
function nextToken(): SyntaxKind {
    currentToken = scanner.scan();
    return currentToken;
}
```

### Node Factory Pattern
```typescript
function parseIdentifier(): Identifier {
    const pos = getNodePos();
    const text = scanner.getTokenValue();
    nextToken();
    return finishNode(factoryCreateIdentifier(text), pos);
}
```

### finishNode Pattern
```typescript
function finishNode<T extends Node>(node: T, pos: number): T {
    setTextRangePosEnd(node, pos, scanner.getTokenStart());
    // ... set parent, etc.
    return node;
}
```
