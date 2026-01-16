# Team Structure and Task Assignments

## Overview

This document tracks the team structure and task assignments for the Project Zang TypeScript-to-Rust/WASM migration.

---

## Team 2 (em-team-2)

**Engineering Manager:** Worker 5
**Team Members:** Workers 6, 7

### Assigned Tasks

Team 2 is assigned **Tier 1: Parser Accuracy**.

| Task | Description | Assigned To | Status |
|------|-------------|-------------|--------|
| TS1109 extra | Parser emits "Expression expected" for valid syntax | Worker 6 | Not Started |
| TS1005 extra | Parser emits "X expected" for valid constructs | Worker 7 | Not Started |

**Key Files:**
- `wasm/src/thin_parser.rs`

### Notes

The parser already has error budgeting mechanisms in place:
- `ts1109_statement_budget`: Limits TS1109 errors per statement (currently set to 3)
- `ts1005_statement_budget`: Limits TS1005 errors per statement (currently set to 2)

These budgets are reset at statement boundaries in `parse_statement()` to prevent error storms.

The issue is that the parser is **over-reporting** these errors - emitting them for valid TypeScript syntax that should not produce errors.

---

## Team 1 (em-team-1)

**Engineering Manager:** Worker 2
**Team Members:** Workers 3, 4

### Assigned Tasks

Based on PROJECT_DIRECTION.md priority tiers, Team 1 is assigned **Tier 0: Quality & Stability Foundations**.

| Task | Description | Assigned To | Status |
|------|-------------|-------------|--------|
| Application type expansion | `TypeKey::Application` is not expanded, leading to incorrect diagnostics/assignability | Worker 3 | Not Started |
| Readonly types | `readonly` arrays/tuples are currently treated as mutable | Worker 4 | Not Started |

**Key Files:**
- `wasm/src/solver/evaluate.rs`
- `wasm/src/solver/intern.rs`
- `wasm/src/solver/instantiate.rs`
- `wasm/src/solver/subtype.rs`

---

## Team 3 (em-team-3)

**Engineering Manager:** Worker 8
**Team Members:** Workers 9, 10

### Assigned Tasks

Team 3 is assigned **Tier 2: Type Checker Accuracy**.

| Task | Description | Assigned To | Status |
|------|-------------|-------------|--------|
| TS2571 extra | "Object is of type 'unknown'" over-reported | Worker 9 | Not Started |
| TS2683 missing | "'this' implicitly has type 'any'" not emitted | Worker 10 | Not Started |

**Key Files:**
- `wasm/src/thin_checker.rs`
- `wasm/src/solver/`

---

## Team 4 (em-team-4)

**Engineering Manager:** Worker 11
**Team Members:** Workers 12, 13, 14

### Assigned Tasks

Team 4 is assigned **Tier 3: Symbol Resolution** and **Tier 4: Implicit Any Checks**.

| Task | Description | Assigned To | Status |
|------|-------------|-------------|--------|
| TS2304 gaps | "Cannot find name" for valid symbols | Worker 12 | Not Started |
| Global merging | Interface/namespace merging across files | Worker 13 | Not Started |
| TS7006 extra | Parameter implicit any over-reported | Worker 14 | Not Started |

**Key Files:**
- `wasm/src/binder/`
- `wasm/src/thin_checker.rs`

---

## Priority Order

According to PROJECT_DIRECTION.md:

```
Quality & Stability (Tier 0) → Parser (Tier 1) → Symbol Resolution (Tier 3) → Type Checker (Tier 2) → Implicit Any (Tier 4) → Async (Tier 5)
```

**Rationale:** Parser errors create broken ASTs that poison downstream analysis. Fix syntax handling before semantic checking.

---

## Workflow for All Teams

1. **Sync first**: `git fetch origin && git merge origin/rust --no-edit`
2. **Ask Gemini**: `./scripts/ask-gemini.mjs "I need to implement <your task>. What files should I modify and what's the approach?"`
3. Write code following Gemini's guidance, add tests, run `./wasm/test.sh`
4. Commit and push to your worker branch
5. Run conformance tests: `./wasm/differential-test/run-conformance.sh --all`
6. Mark "Ready for Merge: Yes" when complete

---

## Rules

- **Stay on your assignment** - Do not self-switch tasks
- **wasm only** - All code changes within `wasm/` directory
- **Docker-only Rust tests**: `./wasm/test.sh`
- **Commit and push** to your worker branch frequently
- **Never modify TypeScript source files** in `src/`

---
*Created by EM Team 2 (Worker 5) on 2026-01-16*
