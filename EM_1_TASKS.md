# EM_1 Tasks - Binder Squad (CRITICAL)

**Branch:** `em-team-1`
**Priority:** 🔴 HIGHEST
**Assigned Workers:** workers 1-4
**Last Updated:** 2026-01-14

---

## Squad Mission

Fix the **"Any" Poisoning** problem. When the Binder fails to find basic symbols like `console`, `Promise`, or `Array`, the Solver defaults to `Any`, which silences ALL downstream type errors. This is the #1 blocker preventing conformance improvements.

---

## Problem Statement

**Error Code:** TS2304 (Cannot find name 'X')

**Current State:**
- **116 Missing Errors:** We're not catching TS2304 when we should
- **343 Extra Errors:** We're reporting TS2304 when we shouldn't
- **Root Cause:** Global Scope and `lib.d.ts` integration is broken

**Impact:**
When `Promise` fails to resolve, it becomes `Any`. Code like `new Promise((resolve) => resolve(5))` should error if the generic doesn't match, but `Any` silences the error.

---

## Immediate Goals

1. **Debug Global Scope Binding**
   - Verify `lib_loader.rs` correctly merges `lib.d.ts` symbols into root `SymbolTable`
   - Ensure `console.log` resolves in test cases
   - Check `Array`, `Promise`, `Object` basic type resolution

2. **Fix Module Augmentation**
   - Interface merging across files (e.g., `interface Window` in multiple files)
   - Global declaration merging
   - Namespace augmentation

3. **Target Metric:** Reduce TS2304 extra errors to **<50**

---

## Key Files to Investigate

| File | Purpose | Action |
|------|---------|--------|
| `src/lib_loader.rs` | Loads lib.d.ts | Verify symbol merging |
| `src/thin_binder.rs` | Binds symbols to AST | Check `file_locals` population |
| `src/symbol_table.rs` | Symbol storage | Debug global scope lookups |

---

## Worker Assignment Strategy

Assign workers based on expertise:

| Worker | Focus Area |
|--------|-----------|
| worker-1 | `lib_loader.rs` - library file loading and symbol injection |
| worker-2 | `thin_binder.rs` - global scope binding |
| worker-3 | `symbol_table.rs` - symbol lookup and merging |
| worker-4 | Test case triage and regression tracking |

---

## Escalation Triggers

Escalate to Director if:
- TS2304 extra errors drop below 50 (ready for new mission)
- Need architectural changes to SymbolTable (may require EM coordination)
- Team size exceeds 4 (need team split)

---

## Success Criteria

- [ ] TS2304 extra errors < 50
- [ ] `console.log` resolves in >95% of test cases
- [ ] `Promise`, `Array`, `Object` resolve globally
- [ ] No regression in TS2304 missing errors

---

## Notes

- **Do NOT** modify worker task lists directly
- Workers should create their own task breakdown
- Track all blocking issues for team resizing decisions
