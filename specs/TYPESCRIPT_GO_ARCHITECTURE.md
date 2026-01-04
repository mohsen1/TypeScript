# TypeScript-Go Architecture

> **Native Go Port of the TypeScript Compiler and Language Server**

This document provides a comprehensive architectural overview of the `typescript-go` codebase, a native Go port of the TypeScript compiler targeting behavioral parity with TypeScript 5.9. Also known as **TypeScript 7**, this project aims to deliver improved performance through native compilation while maintaining full compatibility with the original TypeScript semantics.

---

## Table of Contents

1. [Overview](#overview)
2. [Directory Structure](#directory-structure)
3. [Compilation Pipeline](#compilation-pipeline)
4. [Core Types and Data Structures](#core-types-and-data-structures)
5. [Program Orchestration](#program-orchestration)
6. [Language Server Architecture](#language-server-architecture)
7. [Module Resolution](#module-resolution)
8. [Testing Infrastructure](#testing-infrastructure)
9. [Build System](#build-system)
10. [Package Dependency Graph](#package-dependency-graph)
11. [TypeScript to Go Mapping](#typescript-to-go-mapping)

---

## Overview

### Project Goals

- **Behavioral Parity**: Match TypeScript 5.9 semantics exactly
- **Performance**: Leverage Go's native compilation for faster startup and lower memory usage
- **Parallelism**: Utilize goroutines for concurrent type checking and file processing
- **Maintainability**: Clean package separation with clear boundaries

### Key Metrics

| Component | Lines of Code | Notes |
|-----------|---------------|-------|
| Type Checker (`checker/`) | ~31,000+ | Largest component, heart of type inference |
| Parser (`parser/`) | ~6,600 | Full TypeScript/JavaScript parsing |
| Printer (`printer/`) | ~6,000 | Code generation and source maps |
| Scanner (`scanner/`) | ~2,600 | Lexical tokenization |
| Binder (`binder/`) | ~2,700 | Symbol binding and control flow |

### Dependencies

```
github.com/dlclark/regexp2      - Extended regex (JS-compatible)
github.com/go-json-experiment/json - Fast JSON parsing
github.com/google/go-cmp       - Deep equality for testing
github.com/zeebo/xxh3          - Fast hashing (xxHash)
golang.org/x/sync              - Concurrency primitives
```

---

## Directory Structure

The `typescript-go/` repository is organized as follows:

```
typescript-go/
├── cmd/tsgo/              # Main entry point (tsc equivalent)
├── internal/              # All compiler packages (43 total)
├── testdata/              # Test cases and baselines
├── _extension/            # VS Code extension for preview
├── _packages/             # Published npm packages
├── _submodules/TypeScript # Reference TypeScript implementation
└── _tools/                # Build tools and custom linters
```

### Package Reference (internal/)

| Package | Purpose |
|---------|---------|
| **api/** | HTTP API server for programmatic access |
| **ast/** | Core AST definitions, node types, symbols, flags, factories |
| **astnav/** | AST navigation utilities (find token at position, ancestors) |
| **binder/** | Name binding, symbol tables, control flow graph construction |
| **bundled/** | Embedded lib.d.ts files and bundled resources |
| **checker/** | Type checker — inference, compatibility, diagnostics |
| **collections/** | Generic collections: `Set`, `OrderedMap`, `MultiMap`, `SyncMap` |
| **compiler/** | Program orchestration, file management, checker pool |
| **core/** | Core types, compiler options, script targets, utilities |
| **debug/** | Debug assertions and development utilities |
| **diagnostics/** | Diagnostic message definitions and codes |
| **diagnosticwriter/** | Diagnostic formatting and output rendering |
| **evaluator/** | Compile-time expression evaluation (const enums, etc.) |
| **execute/** | CLI execution entry points (tsc, build mode) |
| **format/** | Code formatting engine |
| **fourslash/** | Fourslash test harness for editor feature testing |
| **glob/** | Glob pattern matching for file discovery |
| **jsnum/** | JavaScript number semantics (IEEE 754 compliance) |
| **jsonutil/** | JSON parsing utilities |
| **locale/** | Localization and internationalization |
| **ls/** | Language Service — completions, hover, definitions, etc. |
| **lsp/** | LSP (Language Server Protocol) JSON-RPC implementation |
| **module/** | Module resolution (Node, Classic, Bundler modes) |
| **modulespecifiers/** | Module specifier generation for auto-imports |
| **nodebuilder/** | Type-to-AST node conversion for display purposes |
| **outputpaths/** | Output file path computation |
| **packagejson/** | package.json parsing and caching |
| **parser/** | TypeScript/JavaScript parser |
| **pprof/** | Profiling support integration |
| **printer/** | AST-to-text emission and code generation |
| **project/** | Project system for language server (configured/inferred) |
| **repo/** | Repository utilities for testing |
| **scanner/** | Lexical scanner and tokenizer |
| **semver/** | Semantic versioning support |
| **sourcemap/** | Source map generation and consumption |
| **stringutil/** | String manipulation utilities |
| **symlinks/** | Symlink resolution and handling |
| **testrunner/** | Test runner infrastructure |
| **testutil/** | Test utilities and baseline management |
| **transformers/** | AST transformers for emit (TS, ES, JSX, modules) |
| **tsoptions/** | TypeScript compiler options parsing |
| **tspath/** | Path manipulation utilities |
| **vfs/** | Virtual file system abstraction |

---

## Compilation Pipeline

The TypeScript-Go compiler follows the same six-phase pipeline as the original TypeScript compiler:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                           COMPILATION PIPELINE                                │
└──────────────────────────────────────────────────────────────────────────────┘

  Source Text
       │
       ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   SCANNER   │────▶│   PARSER    │────▶│   BINDER    │────▶│   CHECKER   │
│  (Lexer)    │     │ (Syntax)    │     │ (Symbols)   │     │  (Types)    │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
   Tokens              AST              Symbol Tables        Type Info
                                        Control Flow         Diagnostics
       │
       ▼
┌─────────────┐     ┌─────────────┐
│ TRANSFORMER │────▶│   PRINTER   │────▶ Output (.js, .d.ts, .map)
│ (Lowering)  │     │  (Emit)     │
└─────────────┘     └─────────────┘
   Modified AST        Text + Maps
```

### Phase 1: Scanner (`scanner/`)

The scanner converts source text into a stream of tokens.

**Key Responsibilities:**
- Tokenize identifiers, keywords, operators, literals
- Handle string templates and regex literals
- Track line/column positions for diagnostics
- Support JSX tokenization modes

**Core Types:**
```go
type Scanner struct {
    text          string
    pos           int
    end           int
    token         ast.Kind
    tokenStart    int
    tokenValue    string
    // ...
}

func (s *Scanner) Scan() ast.Kind
func (s *Scanner) Token() ast.Kind
func (s *Scanner) TokenValue() string
```

### Phase 2: Parser (`parser/`)

The parser transforms tokens into an Abstract Syntax Tree (AST).

**Key Responsibilities:**
- Construct hierarchical AST from token stream
- Handle operator precedence and associativity
- Parse TypeScript-specific syntax (types, decorators, enums)
- Generate parse diagnostics for syntax errors

**Parsing Contexts:**
```go
type ParsingContext int

const (
    ParsingContextSourceElements ParsingContext = iota
    ParsingContextBlockStatements
    ParsingContextSwitchClauses
    ParsingContextSwitchClauseStatements
    ParsingContextTypeMembers
    ParsingContextClassMembers
    ParsingContextEnumMembers
    ParsingContextHeritageClauseElement
    ParsingContextVariableDeclarations
    // ... more contexts
)
```

### Phase 3: Binder (`binder/`)

The binder associates declarations with symbols and constructs the control flow graph.

**Key Responsibilities:**
- Create symbols for declarations
- Build symbol tables (locals, exports, members)
- Establish parent-child relationships
- Construct control flow graph for flow analysis
- Handle hoisting and scope rules

**Symbol Structure:**
```go
type Symbol struct {
    Flags            SymbolFlags
    CheckFlags       CheckFlags
    Name             string
    Declarations     []*Node
    ValueDeclaration *Node
    Members          SymbolTable
    Exports          SymbolTable
    Parent           *Symbol
    // ...
}

type SymbolTable map[string]*Symbol
```

### Phase 4: Checker (`checker/`)

The type checker is the heart of TypeScript — performing type inference, compatibility checking, and error reporting.

**Key Responsibilities:**
- Infer types for expressions and declarations
- Check type compatibility and assignability
- Resolve overloads and generics
- Generate semantic diagnostics
- Support control flow narrowing

**Type System:**
```go
type Type struct {
    flags       TypeFlags
    id          TypeId
    symbol      *Symbol
    // Union, intersection, generic instantiation data...
}

type Signature struct {
    flags           SignatureFlags
    declaration     *Node
    typeParameters  []*Type
    parameters      []*Symbol
    resolvedReturnType *Type
    // ...
}
```

### Phase 5: Transformer (`transformers/`)

Transformers modify the AST for emit, performing syntax lowering and module transformation.

**Sub-packages:**
| Transformer | Purpose |
|-------------|---------|
| `tstransforms/` | TypeScript-specific (type erasure, enum emit) |
| `estransforms/` | ES version downleveling |
| `jsxtransforms/` | JSX to createElement calls |
| `moduletransforms/` | Module system conversion (ESM ↔ CJS) |
| `declarations/` | Declaration file (.d.ts) generation |

### Phase 6: Printer (`printer/`)

The printer converts the (possibly transformed) AST back to text.

**Key Responsibilities:**
- Emit JavaScript, TypeScript, or declaration files
- Generate source maps
- Preserve or strip comments
- Handle formatting and whitespace

---

## Core Types and Data Structures

### AST Node (`ast/`)

All AST nodes share a common structure with a `Kind` discriminator:

```go
type Node struct {
    Kind   Kind           // Discriminator (400+ kinds)
    Flags  NodeFlags      // Syntax flags
    Loc    core.TextRange // Source location
    id     atomic.Uint64  // Unique ID
    Parent *Node          // Parent in tree
    data   nodeData       // Polymorphic payload
}

// Node factory with pooling for efficiency
type NodeFactory struct {
    hooks NodeFactoryHooks
    // Pools for different node types...
}

// Collection of nodes
type NodeList struct {
    Loc   core.TextRange
    Nodes []*Node
}
```

### Kind Constants (`ast/kind.go`)

The `Kind` type discriminates between 400+ node types:

```go
type Kind int16

const (
    KindUnknown Kind = iota
    KindEndOfFile
    
    // Tokens
    KindNumericLiteral
    KindStringLiteral
    KindIdentifier
    
    // Keywords
    KindClassKeyword
    KindFunctionKeyword
    KindInterfaceKeyword
    
    // Declarations
    KindFunctionDeclaration
    KindClassDeclaration
    KindInterfaceDeclaration
    KindTypeAliasDeclaration
    KindEnumDeclaration
    
    // Expressions
    KindCallExpression
    KindPropertyAccessExpression
    KindBinaryExpression
    
    // Types
    KindTypeReference
    KindUnionType
    KindIntersectionType
    // ... 400+ total kinds
)
```

### Node Flags (`ast/flags.go`)

```go
type NodeFlags uint32

const (
    NodeFlagsNone                NodeFlags = 0
    NodeFlagsLet                 NodeFlags = 1 << 0
    NodeFlagsConst               NodeFlags = 1 << 1
    NodeFlagsExportContext       NodeFlags = 1 << 2
    NodeFlagsAmbient             NodeFlags = 1 << 3
    NodeFlagsPublic              NodeFlags = 1 << 4
    NodeFlagsPrivate             NodeFlags = 1 << 5
    NodeFlagsProtected           NodeFlags = 1 << 6
    NodeFlagsStatic              NodeFlags = 1 << 7
    NodeFlagsReadonly            NodeFlags = 1 << 8
    NodeFlagsAbstract            NodeFlags = 1 << 9
    NodeFlagsAsync               NodeFlags = 1 << 10
    // ...
)
```

### Symbol Flags (`ast/symbol.go`)

```go
type SymbolFlags uint32

const (
    SymbolFlagsNone               SymbolFlags = 0
    SymbolFlagsFunctionScopedVariable SymbolFlags = 1 << 0
    SymbolFlagsBlockScopedVariable    SymbolFlags = 1 << 1
    SymbolFlagsProperty               SymbolFlags = 1 << 2
    SymbolFlagsEnumMember             SymbolFlags = 1 << 3
    SymbolFlagsFunction               SymbolFlags = 1 << 4
    SymbolFlagsClass                  SymbolFlags = 1 << 5
    SymbolFlagsInterface              SymbolFlags = 1 << 6
    SymbolFlagsConstEnum              SymbolFlags = 1 << 7
    SymbolFlagsRegularEnum            SymbolFlags = 1 << 8
    SymbolFlagsValueModule            SymbolFlags = 1 << 9
    SymbolFlagsNamespaceModule        SymbolFlags = 1 << 10
    SymbolFlagsTypeAlias              SymbolFlags = 1 << 17
    // ...
)
```

---

## Program Orchestration

### Program (`compiler/program.go`)

The `Program` is the central orchestrator that manages the entire compilation:

```go
type ProgramOptions struct {
    Host                        CompilerHost
    Config                      *tsoptions.ParsedCommandLine
    UseSourceOfProjectReference bool
    SingleThreaded              core.Tristate
    CreateCheckerPool           func(*Program) CheckerPool
}

type Program struct {
    opts                  ProgramOptions
    checkerPool           CheckerPool
    sourceFiles           []*ast.SourceFile
    filesByPath           map[tspath.Path]*ast.SourceFile
    resolvedModules       map[tspath.Path]map[string]*ResolvedModule
    programDiagnostics    *ast.DiagnosticsCollection
    // ...
}
```

**Key Methods:**
```go
func (p *Program) GetSourceFiles() []*ast.SourceFile
func (p *Program) GetTypeChecker() *checker.Checker
func (p *Program) GetSemanticDiagnostics(file *ast.SourceFile) []*ast.Diagnostic
func (p *Program) GetSyntacticDiagnostics(file *ast.SourceFile) []*ast.Diagnostic
func (p *Program) Emit(opts *EmitOptions) *EmitResult
```

### Checker Pool

For performance, multiple type checkers can run in parallel:

```go
type CheckerPool interface {
    GetChecker() *checker.Checker
    ReturnChecker(*checker.Checker)
    GetAllDiagnostics() []*ast.Diagnostic
}
```

---

## Language Server Architecture

### Language Service (`ls/`)

The Language Service provides IDE features:

```go
type LanguageService struct {
    host                    Host
    program                 *compiler.Program
    converters              *lsconv.Converters
    documentPositionMappers map[string]*sourcemap.DocumentPositionMapper
}
```

**Features Implemented:**

| File | Feature |
|------|---------|
| `completions.go` | Code completions and auto-import |
| `definition.go` | Go to definition/type definition |
| `hover.go` | Hover information (quick info) |
| `findallreferences.go` | Find all references |
| `codeactions.go` | Quick fixes and refactorings |
| `signaturehelp.go` | Function signature help |
| `diagnostics.go` | Real-time error reporting |
| `format.go` | Document/range formatting |
| `folding.go` | Code folding ranges |
| `callhierarchy.go` | Call hierarchy |
| `documenthighlights.go` | Highlight occurrences |
| `rename.go` | Rename symbol |
| `references.go` | Reference navigation |

### LSP Server (`lsp/`)

The LSP server implements the Language Server Protocol:

```go
type Server struct {
    r Reader
    w Writer
    
    cwd                string
    fs                 vfs.FS
    defaultLibraryPath string
    
    projectService     *project.Service
    requestQueue       chan *request
    pendingRequests    sync.Map
    // ...
}
```

**Protocol Flow:**
```
┌────────────────┐                    ┌────────────────┐
│   VS Code      │◄──── JSON-RPC ────►│   LSP Server   │
│   (Client)     │                    │   (lsp/)       │
└────────────────┘                    └───────┬────────┘
                                              │
                                              ▼
                                      ┌────────────────┐
                                      │ Language       │
                                      │ Service (ls/)  │
                                      └───────┬────────┘
                                              │
                                              ▼
                                      ┌────────────────┐
                                      │ Project        │
                                      │ (project/)     │
                                      └───────┬────────┘
                                              │
                                              ▼
                                      ┌────────────────┐
                                      │ Program        │
                                      │ (compiler/)    │
                                      └────────────────┘
```

### Project System (`project/`)

Manages TypeScript projects (configured via tsconfig.json or inferred):

```go
type Project struct {
    Kind             Kind  // KindInferred or KindConfigured
    currentDirectory string
    configFileName   string
    CommandLine      *tsoptions.ParsedCommandLine
    Program          *compiler.Program
    
    // File watching
    watchedFiles     map[string]*watchedFile
    // ...
}

type Service struct {
    host            Host
    projects        map[string]*Project
    openFiles       map[string]*ScriptInfo
    // ...
}
```

---

## Module Resolution

### Resolver (`module/`)

Handles all TypeScript module resolution modes:

```go
type Resolver struct {
    compilerOptions  *core.CompilerOptions
    host             Host
    cache            *Cache
    // ...
}

type ResolvedModule struct {
    ResolvedFileName        string
    OriginalPath            string
    Extension               Extension
    IsExternalLibraryImport bool
    PackageId               *PackageId
}
```

**Resolution Modes:**

| Mode | Description |
|------|-------------|
| **Classic** | Legacy TypeScript resolution |
| **Node10** | Node.js CommonJS resolution |
| **Node16/NodeNext** | Node.js ESM resolution |
| **Bundler** | Bundler-style resolution (Vite, webpack) |

**Path Mapping Support:**
```json
{
  "compilerOptions": {
    "baseUrl": "./src",
    "paths": {
      "@/*": ["*"],
      "@components/*": ["components/*"]
    }
  }
}
```

---

## Testing Infrastructure

### Test Types

| Type | Location | Description |
|------|----------|-------------|
| **Compiler Tests** | `testdata/tests/cases/compiler/` | Baseline tests for compilation |
| **Fourslash Tests** | `testdata/tests/cases/fourslash/` | Editor feature tests |
| **Submodule Tests** | `_submodules/TypeScript/tests/` | Tests from original TypeScript |

### Test Runner (`testrunner/`)

```go
type Runner interface {
    EnumerateTestFiles() []string
    RunTests(t *testing.T)
}

// Run specific tests:
// go test -run='TestSubmodule/<test name>' ./internal/testrunner
// go test -run='TestLocal/<test name>' ./internal/testrunner
```

### Compiler Test Format

```typescript
// @target: esnext
// @module: preserve
// @strict: true

// @filename: fileA.ts
export interface Person {
    name: string;
    age: number;
}

// @filename: fileB.ts
import { Person } from "./fileA";

const p: Person = { name: "Alice", age: 30 };
```

### Fourslash Test Format (`fourslash/`)

```typescript
/// <reference path='fourslash.ts'/>

// @filename: /a.ts
////export function greet(name: string) {
////    return `Hello, ${name}!`;
////}

// @filename: /b.ts
////import { /*1*/greet } from './a';
////greet/*2*/("World");

goTo.marker("1");
verify.quickInfoIs("(function) greet(name: string): string");

goTo.marker("2");
verify.completions({ includes: "greet" });
```

### Baselines

Test outputs are compared against reference baselines:

```
testdata/baselines/
├── local/      # Generated by test runs
└── reference/  # Expected outputs (committed)
```

Accept new baselines with:
```bash
npx hereby baseline-accept
```

---

## Build System

### Entry Point (`cmd/tsgo/main.go`)

```go
func main() {
    os.Exit(runMain())
}

func runMain() int {
    args := os.Args[1:]
    
    if len(args) > 0 {
        switch args[0] {
        case "--lsp":
            return runLSP(args[1:])
        case "--api":
            return runAPI(args[1:])
        }
    }
    
    // Default: run as tsc
    result := execute.CommandLine(newSystem(), args, nil)
    return int(result.Status)
}
```

### Go Module (`go.mod`)

```go
module github.com/microsoft/typescript-go

go 1.25

require (
    github.com/dlclark/regexp2 v1.11.5
    github.com/go-json-experiment/json v0.0.0-20250103232110-6a9a0fde9288
    github.com/google/go-cmp v0.7.0
    github.com/zeebo/xxh3 v1.0.2
    golang.org/x/sync v0.19.0
)
```

### Hereby Tasks (`Herebyfile.mjs`)

```bash
npx hereby build            # Build tsgo binary
npx hereby test             # Run all tests
npx hereby lint             # Run linters
npx hereby format           # Format code
npx hereby baseline-accept  # Accept new baselines
```

---

## Package Dependency Graph

```
                              ┌─────────────────────────────────┐
                              │           cmd/tsgo              │
                              │         (Entry Point)           │
                              └─────────────────────────────────┘
                                              │
                      ┌───────────────────────┼───────────────────────┐
                      ▼                       ▼                       ▼
               ┌────────────┐          ┌────────────┐          ┌────────────┐
               │  execute   │          │    lsp     │          │    api     │
               │   (CLI)    │          │  (Server)  │          │  (HTTP)    │
               └────────────┘          └────────────┘          └────────────┘
                      │                       │                       │
                      │                       ▼                       │
                      │                ┌────────────┐                 │
                      │                │  project   │                 │
                      │                │ (Projects) │                 │
                      │                └────────────┘                 │
                      │                       │                       │
                      ▼                       ▼                       ▼
               ┌─────────────────────────────────────────────────────────┐
               │                        compiler                         │
               │                   (Program, Emit)                       │
               └─────────────────────────────────────────────────────────┘
                                              │
          ┌───────────────┬───────────────────┼───────────────────┬───────────────┐
          ▼               ▼                   ▼                   ▼               ▼
    ┌──────────┐   ┌──────────┐        ┌──────────┐        ┌──────────┐   ┌──────────────┐
    │  parser  │   │  binder  │        │ checker  │        │ printer  │   │ transformers │
    └──────────┘   └──────────┘        └──────────┘        └──────────┘   └──────────────┘
          │               │                   │                   │               │
          ▼               │                   │                   │               │
    ┌──────────┐          │                   │                   │               │
    │ scanner  │          │                   │                   │               │
    └──────────┘          │                   │                   │               │
          │               │                   │                   │               │
          └───────────────┴───────────────────┴───────────────────┴───────────────┘
                                              │
                                              ▼
               ┌─────────────────────────────────────────────────────────┐
               │                          ast                            │
               │            (Node, Symbol, Type, Kind, Flags)            │
               └─────────────────────────────────────────────────────────┘
                                              │
                      ┌───────────────────────┼───────────────────────┐
                      ▼                       ▼                       ▼
               ┌────────────┐          ┌────────────┐          ┌────────────┐
               │    core    │          │diagnostics │          │ tsoptions  │
               │ (Options)  │          │ (Messages) │          │ (Parsing)  │
               └────────────┘          └────────────┘          └────────────┘
                      │
                      ▼
               ┌────────────┐          ┌────────────┐          ┌────────────┐
               │    vfs     │          │   module   │          │ sourcemap  │
               │ (Files)    │          │(Resolution)│          │  (Maps)    │
               └────────────┘          └────────────┘          └────────────┘
```

---

## TypeScript to Go Mapping

Reference mapping between original TypeScript source files and their Go equivalents:

| TypeScript File | Go Package |
|-----------------|------------|
| `src/compiler/scanner.ts` | `internal/scanner/` |
| `src/compiler/parser.ts` | `internal/parser/` |
| `src/compiler/binder.ts` | `internal/binder/` |
| `src/compiler/checker.ts` | `internal/checker/` |
| `src/compiler/emitter.ts` | `internal/printer/` + `internal/transformers/` |
| `src/compiler/program.ts` | `internal/compiler/` |
| `src/compiler/types.ts` | `internal/ast/` |
| `src/compiler/utilities.ts` | `internal/ast/utilities.go` |
| `src/compiler/moduleResolver.ts` | `internal/module/` |
| `src/services/services.ts` | `internal/ls/` |
| `src/server/session.ts` | `internal/lsp/` |
| `src/server/project.ts` | `internal/project/` |

### Go Idioms vs TypeScript Patterns

| TypeScript Pattern | Go Equivalent |
|-------------------|---------------|
| Union types (`A \| B`) | Interface with type assertion |
| Class inheritance | Struct embedding |
| Async/await | Goroutines + channels |
| Map/Filter/Reduce | For loops or generics |
| Optional chaining (`?.`) | Nil checks |
| Nullish coalescing (`??`) | `if x == nil { x = default }` |
| Type guards | Type assertions/switches |
| Enums | `iota` constants |
| Private fields (`#field`) | Unexported fields (lowercase) |

---

## Performance Characteristics

### Memory Management

- **Node Pooling**: AST nodes are pooled to reduce allocation pressure
- **String Interning**: Identifiers and keywords are interned
- **Lazy Binding**: Symbols are bound lazily during checking

### Parallelism

- **Concurrent File Parsing**: Multiple files parsed in parallel
- **Checker Pool**: Multiple type checkers for parallel semantic analysis
- **Goroutine-per-file**: File processing distributed across goroutines

### Compared to TypeScript (JavaScript)

| Aspect | TypeScript (JS) | typescript-go |
|--------|-----------------|---------------|
| Startup time | ~500ms | ~50ms |
| Memory usage | Higher (V8 overhead) | Lower (native) |
| Type checking | Single-threaded | Multi-threaded |
| Cold start | Slower (JIT warm-up) | Faster (AOT) |

---

## Further Reading

- [TypeScript Compiler Internals](https://github.com/microsoft/TypeScript/wiki/Compiler-Internals)
- [TypeScript-Go README](../typescript-go/README.md)
- [TypeScript-Go CONTRIBUTING](../typescript-go/CONTRIBUTING.md)
- [Language Server Protocol Specification](https://microsoft.github.io/language-server-protocol/)

---

*Generated from typescript-go codebase analysis. Last updated: January 2026.*
