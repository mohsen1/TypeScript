# Team Structure - TypeScript Compiler (Rust)

**Phase:** Phase 8 - Conformance, Convergence, and Hardening
**Last Updated:** 2026-01-14
**Director:** claude-code-orchestrator

---

## Current Conformance Status

| Metric | Value | Target |
|--------|-------|--------|
| Exact Match | 30.1% (1488/4939) | **40%** |
| Missing Errors | 60.0% (2961) | **<50%** |
| Extra Errors | 30.9% (1528) | Reduce |

### Critical Issues
- **Error Poisoning:** Solver defaults to `Any` when Binder fails, silencing downstream errors
- **TS2304:** 116 missing + 343 extra (Cannot find name)
- **Parser False Positives:** 701 errors (TS1005: 439, TS1109: 262)

---

## Squad Assignments

### EM_1: Binder Squad (CRITICAL)
**Branch:** `em-team-1`
**Priority:** 🔴 HIGHEST
**Target Error:** TS2304 (Cannot find name)

**Focus:**
- Fix Global Scope binding
- Ensure `lib.d.ts` symbols merge into root `SymbolTable`
- Fix module augmentation resolution
- Debug `console`, `Promise`, `Array` resolution failures
- **Goal:** Reduce TS2304 extra errors to <50

**Key Files:**
- `src/lib_loader.rs`
- `src/thin_binder.rs`
- `src/symbol_table.rs` (if exists)

**Success Metric:** TS2304 extra errors < 50

---

### EM_2: Parser/Scanner Squad (PRIORITY)
**Branch:** `em-team-2`
**Priority:** 🟠 HIGH
**Target Errors:** TS1005, TS1109

**Focus:**
- Fix TS1005 ("expected X") emission - over-triggering on valid syntax
- Fix TS1109 ("expression expected") - false positives on edge cases
- Audit error recovery logic in `thin_parser.rs`
- **Goal:** Reduce parser false positives to <100

**Key Files:**
- `src/thin_parser.rs`
- `src/scanner.rs` (if exists)
- `src/error_recovery.rs` (if exists)

**Success Metric:** Parser false positives < 100

---

### EM_3: Solver Squad (STRATEGIC)
**Branch:** `em-team-3`
**Priority:** 🟠 HIGH
**Target Errors:** TS2322, TS7006

**Focus:**
- Switch default fallback from `Any` to `Unknown` or `Error`
- Harden `solve_subtype` logic
- Implement "Lawyer" layer for TypeScript quirks (function bivariance, void return exceptions)
- Make compiler stricter (meaner) to match `tsc`
- **Goal:** Convert "Missing TS2322" to "Exact Match" or "Extra TS2322"

**Key Files:**
- `src/solver/mod.rs`
- `src/solver/subtype.rs` (if exists)
- `src/solver/inference.rs` (if exists)

**Success Metric:** Switch to `Unknown` fallback completed

---

## Anti-Priorities (DO NOT WORK ON)
- Performance optimization (speed is sufficient at 41.7 tests/sec)
- New Emitter features (downleveling is stable)
- LSP polish (wait for semantic model accuracy)

---

## Resource Allocation

| EM | Squad | Assigned Workers | Total |
|----|-------|------------------|-------|
| EM_1 | Binder | workers 1-4 | 5 |
| EM_2 | Parser | workers 5-6 | 3 |
| EM_3 | Solver | workers 7-9 | 4 |

**Note:** CFA (Control Flow Analysis) work is on hold until TS2304 is under control.

---

## Merging Protocol
1. EMs submit PRs to `rust` branch
2. Director reviews and merges EM branches only
3. After each merge: re-balance teams to maintain size ≤ 4 (including EM)

---

## Escalation Criteria
EMs should escalate to Director when:
- Team size exceeds 4 (need split)
- Blocker requires cross-squad coordination
- Target metric is achieved (ready for reassignment)
- Technical approach is unclear and needs Director input
