# EM_3 Tasks - Solver Squad (STRATEGIC)

**Branch:** `em-team-3`
**Priority:** 🟠 HIGH
**Assigned Workers:** workers 7-9
**Last Updated:** 2026-01-14

---

## Squad Mission

Stop being "nice". The compiler is too permissive—it defaults to `Any` when type checking gets hard. We need to switch to `Unknown`/`Error` fallbacks and harden the subtyping logic to expose real bugs.

---

## Problem Statement

**Error Codes:** TS2322 (Type not assignable), TS7006 (Implicit Any)

**Current State:**
- Solver returns `Any` when it can't resolve a type
- `Any` silences ALL downstream type checking
- We're missing TS2322 and TS7006 errors that `tsc` catches

**Root Cause:**
The "optimistic" compiler approach—when in doubt, return `Any`. This was fine for early development, but now it masks semantic bugs.

**Impact:**
- Missing errors that should be caught
- False sense of correctness
- Impossible to measure real semantic progress

---

## Immediate Goals

1. **Switch Fallback from `Any` to `Unknown`/`Error`**
   - Change `TypeId::ANY` defaults to `TypeId::UNKNOWN` or `TypeId::ERROR`
   - Update `lower_type` to return `Error` on resolution failure
   - **WARNING:** This will temporarily spike error counts—this is GOOD (exposes real bugs)

2. **Harden `solve_subtype` Logic**
   - Implement TypeScript's function bivariance rules
   - Handle void return exceptions
   - Fix generic constraint checking

3. **Implement the "Lawyer" Layer**
   - Reference: `specs/SOLVER.md`
   - TypeScript has intentional quirks—implement them precisely
   - Stop being "helpful" and be "accurate" instead

4. **Target Metric:** Convert "Missing TS2322" to "Exact Match" or "Extra TS2322"

---

## Key Files to Investigate

| File | Purpose | Action |
|------|---------|--------|
| `src/solver/mod.rs` | Main solver entry point | Change default fallback type |
| `src/solver/subtype.rs` | Subtype checking | Harden logic, add TS quirks |
| `src/solver/inference.rs` | Generic inference | Fix constraint handling |
| `specs/SOLVER.md` | TypeScript spec | Implement "Lawyer" layer |

---

## Worker Assignment Strategy

| Worker | Focus Area |
|--------|-----------|
| worker-7 | Change `Any` → `Unknown` fallback across solver |
| worker-8 | Harden `solve_subtype` with function bivariance |
| worker-9 | Implement "Lawyer" layer quirks from specs/SOLVER.md |

---

## Escalation Triggers

Escalate to Director if:
- Fallback switch completed and metrics stabilized
- Need architectural changes to solver design
- Team size exceeds 4 (need team split)

---

## Success Criteria

- [ ] Solver defaults to `Unknown` or `Error` instead of `Any`
- [ ] Function bivariance rules implemented
- [ ] "Lawyer" layer for TS quirks implemented
- [ ] Missing TS2322 errors decrease (or convert to Extra TS2322)

---

## Notes

- **Do NOT** modify worker task lists directly
- Workers should create their own task breakdown
- **Expect temporary metric regression**—this is intentional and good
- Coordinate with EM_1 (Binder) as solver fixes depend on symbol resolution
- This is "strategic" work—it unblocks all other squads by exposing real bugs
