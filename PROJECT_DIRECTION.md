
### 1. 🔴 CRITICAL: Fix The "Parser Noise" (TS1005 & TS1109)
**Owner:** Syntax Squad
**Data:** 701 combined extra errors (TS1005: 439, TS1109: 262).
**Analysis:** These are false positives. Our `ThinParser` is bailing out or emitting error nodes on syntax that `tsc` accepts. This "noise" makes it impossible to trust downstream semantic errors because a broken AST results in broken symbols.
**Action:**
*   **Implement Error Resynchronization:** When the parser hits an unexpected token, it must not just emit an error; it must advance to the next synchronization point (e.g., next `;` or `}`) and *continue* parsing the rest of the file.
*   **Audit Semicolon Insertion (ASI):** Verify our ASI logic matches TypeScript's exactly. Many TS1005 errors are likely missing semicolons we aren't inferring.

### 2. 🔴 CRITICAL: The "Global Scope" Fix (TS2304)
**Owner:** Binder Squad
**Data:** TS2304 appears in both Extra (343) and Missing (116) lists.
**Analysis:** This is the root of the "Error Poisoning."
*   **Extra TS2304:** We aren't loading `lib.d.ts` correctly in the test runner, so `console`, `Promise`, and `Array` are undefined.
*   **Missing Errors:** When `Promise` is undefined, the Solver treats it as `Any`. This suppresses TS2322 (Type Mismatch) errors downstream.
**Action:**
*   **Fix Lib Injection:** Ensure `lib.d.ts` is correctly merged into the root `SymbolTable` for every test.
*   **Fix Global Merging:** Ensure `interface Window` (and similar globals) merge correctly across files.

### 3. 🟠 STRATEGIC: Invert Solver Defaults (Stop being "Nice")
**Owner:** Semantics Squad
**Data:** 2961 missing errors (60%).
**Analysis:** We are missing 184 `TS2322` (Type Mismatch) and 357 `TS7006` (Implicit Any) errors. This proves our compiler is "optimistic"—when it encounters an unknown type or a resolution failure, it returns `TypeId::ANY`.
**Action:**
*   **Change Default to `UNKNOWN`:** Modify `src/solver/` to return `TypeId::UNKNOWN` or `TypeId::ERROR` instead of `TypeId::ANY` when a symbol cannot be resolved or a type operation fails.
*   **Expect a Regression:** This will cause a massive spike in "Extra Errors." **This is good.** It exposes exactly where our logic is failing rather than hiding it behind `Any`.

### 4. 🟡 TACTICAL: Fix Class Property Initialization (TS2564)
**Owner:** CFA Squad
**Data:** TS2564 is the #1 missing error (413 occurrences).
**Analysis:** "Property 'x' has no initializer..." is missing. This means we are simply *not running* the check that verifies class properties are initialized in the constructor.
**Action:**
*   Implement the `strictPropertyInitialization` check in `thin_checker.rs`. This is a high-ROI task that will knock out the top missing error category.

### 5. 🟢 STABILITY: Recursion Guards
**Data:** 2 Crashes (Stack Overflow).
**Analysis:** `types/typeRelationships/recursiveTypes` caused a panic.
**Action:**
*   Add a `recursion_depth` counter to `solve_subtype` and `check_expression`. Return a "Type instantiation is excessively deep" error (TS2589) when hitting the limit (e.g., 100), rather than crashing the WASM process.

---

### Success Metrics for Next Report
*   **TS1005/TS1109 (Parser):** Reduce from ~700 to <40.
*   **TS2304 (Binder):** Reduce Extra errors from 343 to <10.
*   **TS2564 (CFA):** Reduce Missing errors from 413 to <20.
*   **Exact Match:** Increase from 30.1% to **80%+**.

**Priority Order:** Parser (Noise) -> Binder (Poisoning) -> Solver (Strictness). Do not work on new features until Parser noise is cleared.