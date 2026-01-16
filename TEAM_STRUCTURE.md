# Team 2 Structure and Status

**EM:** Worker 3 (Engineering Manager for Team 2)
**Date:** 2026-01-16
**Branch:** worker-3

---

## Team Composition

| Worker | Squad | Status | Current Task |
|--------|-------|--------|--------------|
| Worker 5 | Syntax Squad | Active | Statement-Level Error Recovery Enhancement |
| Worker 6 | Binder Squad | Active | TS2304 Global Scope / Lib Injection (ongoing) |
| Worker 7 | Semantics Squad | Approved | Module Symbol Resolution (TS7005, TS7008, TS2792) |
| Worker 8 | LSP Squad | Approved | LSP TypeScript Config Integration |

---

## Team 2 Mission

Fix critical issues in the Rust/WASM TypeScript compiler to achieve 95%+ conformance test accuracy. Focus areas:

1. **Parser Accuracy (Tier 1)** - Syntax Squad (Worker 5)
2. **Symbol Resolution (Tier 3)** - Binder Squad (Worker 6)
3. **Type Checker Accuracy (Tier 2)** - Semantics Squad (Worker 7)
4. **LSP Features** - LSP Squad (Worker 8)

---

## Current Conformance Test Status

Based on the latest report (2026-01-14):

| Metric | Result | Target |
|--------|--------|--------|
| **Exact Match** | 32.11% | 95% |
| **Same Error Count** | 42.11% | - |
| **Missing Errors** | 59.47% | <5% |
| **Extra Errors** | 24.74% | <5% |

### Top Priority Error Codes (Extra - False Positives)

| Code | Count | Description | Assigned To |
|------|-------|-------------|-------------|
| TS7006 | 17 | Parameter implicitly has 'any' type | - |
| TS1005 | 10 | 'X' expected | Worker 5 (Syntax Squad) |
| TS7011 | 9 | Function lacks ending return statement | - |

### Top Priority Error Codes (Missing - Under-reporting)

| Code | Count | Description | Assigned To |
|------|-------|-------------|-------------|
| TS2300 | 40 | Duplicate identifier | - |
| TS1109 | 12 | Expression expected | Worker 5 (Syntax Squad) |
| TS2524 | 12 | Property X does not exist | - |

---

## Worker Status Reports

### Worker 5 (Syntax Squad)

**Branch:** worker-5
**Primary Task:** Fix Parser Noise (TS1005 & TS1109)
**Status:** Multiple subtasks completed

#### Completed Work:
- ASI Implementation
- Parser Noise Reduction (Round 2)
- Object Literal Error Recovery
- Array Literal Error Recovery
- TS1005/TS1109 Error Suppression (Merged)

#### Current Work:
- Statement-Level Error Recovery Enhancement
- Improving statement boundary detection for error recovery

#### Success Criteria:
- Reduce TS1005/TS1109 from ~700 to <40 extra errors

---

### Worker 6 (Binder Squad)

**Branch:** worker-6
**Primary Task:** Fix Global Scope / Lib Injection (TS2304)
**Status:** Significant progress, ongoing

#### Completed Work:
- TS2589 Recursion Guards
- TS2454 Fix for lib.d.ts Global Values

#### Current Work:
- Continue TS2304 global scope improvements
- Module symbol merging across files

#### Success Criteria:
- Reduce TS2304 Extra errors from 343 to <10
- `console`, `Promise`, `Array` available in all test cases

---

### Worker 7 (Semantics Squad)

**Branch:** worker-7
**Primary Task:** Module Symbol Resolution (TS7005, TS7008, TS2792)
**Status:** Ready for new task assignment

#### Completed Work:
- Invert Solver Defaults (MERGED)
- WasmProgram lib files fix
- Comprehensive documentation

#### Investigation Complete:
- Module symbol resolution gap identified
- Root cause: No cross-file module resolution during binding

#### Next Priority:
- Fix module export/import resolution
- Target: TS7005 (489 extra) -> <100

---

### Worker 8 (LSP Squad)

**Branch:** worker-8
**Primary Task:** LSP TypeScript Config Integration
**Status:** APPROVED, ready to implement

#### Completed Work:
- TS2564 Verification (all 41 unit tests pass)

#### Current Work:
- LSP TypeScript Config Integration
- Wire tsconfig.json strict setting to LSP features

#### Files to Modify:
- `wasm/src/lsp/project.rs`
- `wasm/src/lsp/hover.rs`
- `wasm/src/lsp/signature_help.rs`
- `wasm/src/lsp/completions.rs`

---

## Priority Order for Team 2

```
1. Parser Noise (Worker 5) - Blocks downstream analysis
2. Global Scope/Lib Injection (Worker 6) - Foundation for symbol resolution
3. Module Symbol Resolution (Worker 7) - High leverage fix (~800 errors)
4. LSP Config Integration (Worker 8) - Quality of life improvement
```

---

## Recent Accomplishments

1. **Invert Solver Defaults** - Solver now returns ERROR instead of ANY for unresolved types
2. **TS2454 Fix** - lib.d.ts globals now work without definite assignment errors
3. **Parser Error Recovery** - Multiple improvements to error recovery and ASI handling
4. **Conformance Improvements** - +2.64% exact match, +5.27% same count, -2.63% extra errors

---

## Key Files Reference

| Component | Location |
|-----------|----------|
| Parser | `wasm/src/thin_parser.rs` |
| Checker | `wasm/src/thin_checker.rs` |
| Binder | `wasm/src/binder/` |
| Solver | `wasm/src/solver/` |
| LSP | `wasm/src/lsp/` |
| Diagnostics | `wasm/src/checker/types/diagnostics.rs` |

---

## Instructions for Workers

1. **Always sync first:** `git fetch origin && git merge origin/rust --no-edit`
2. **Run tests before pushing:** `./wasm/test.sh`
3. **Use Gemini for guidance:** `./scripts/ask-gemini.mjs "I need to implement <task>"`
4. **Commit format:** `[wasm] <component>: <description>`
5. **Signal readiness:** Set "Ready for Merge: Yes" when done

---

## EM-2 Merge Workflow

1. Workers push to their branches (worker-5, worker-6, worker-7, worker-8)
2. EM-2 reviews and validates changes
3. Merge to em-team-2 branch
4. Run conformance tests to validate
5. Escalate to Director when stable

---

*Generated by EM Team 2 (Worker 3) at 2026-01-16*
