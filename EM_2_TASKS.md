# EM-2 TASK LIST

## Team: em-team-2
## Manager: EM-2
## Base Branch: rust
## Team Branch: em-team-2

---

## Mission Statement

EM-2 is responsible for **semantic accuracy**. Your team ensures that:
1. Module symbol resolution works correctly (TS7005, TS7008, TS2792)
2. Global scope and lib injection work (TS2304)
3. LSP features respect TypeScript configuration
4. Type checker accuracy is maintained

**Why this matters:** Semantic correctness is critical for real-world TypeScript projects. Module resolution, global symbols, and LSP accuracy affect every TypeScript user.

---

## Team Composition

| Worker | Squad | Current Task | Status | Priority |
|--------|-------|--------------|--------|----------|
| Worker 5 | Syntax | TS2348 callable expression over-reporting | 🟢 Assigned | 🟡 HIGH |
| Worker 6 | Binder | TS7006/TS7005 implicit any over-reporting | 🟢 Assigned | 🔴 CRITICAL |
| Worker 7 | Semantics | Complete module import resolution | 🔄 Reassigned | 🔴 CRITICAL |
| Worker 8 | LSP | TS2322 type assignability over-reporting | 🟢 Assigned | 🟡 HIGH |

**EM Branch:** em-team-2

---

## Completed Work

### Worker 6: Global Scope / Lib Injection ✅
**Commits:**
- 5c87adf98 - TS2589 recursion guards
- 6f955bc732a - TS2454 lib.d.ts global values fix
- bef59dfec - Lib symbol injection enhancement

**Problem:** lib.d.ts globals (console, Promise, Array, etc.) weren't available during type checking, causing TS2304 and TS2454 errors.

**Fix:**
- Modified `symbol_is_in_ambient_context` to detect lib symbols
- Merges lib symbols into `current_scope` for immediate availability
- All lib.d.ts globals now work without TS2454

**Impact:** `console.log()`, `Promise.resolve()`, `new Map()`, `new Set()` all work correctly.

### Worker 8: TS2564 Verification ✅
**Commit:** 4ad3a0c4f - TS2564 verification and compilation fix

**Problem:** Task description claimed "413 missing TS2564 errors" and that we weren't running the check.

**Investigation:**
- TS2564 `strictPropertyInitialization` check IS fully implemented
- All 41 unit tests pass
- Check is invoked from `check_class_declaration` and `check_class_expression`
- Implementation includes complete control flow analysis

**Conclusion:** The "413 missing" metric was outdated. Implementation is complete and correct.

---

## Team Priorities (Updated 2026-01-15 13:20)

### Priority 1: Module Symbol Resolution (Tier 3) - 🔴 CRITICAL
**Owner:** Worker 7 (namespace, defaults) + Worker 6 (re-exports, TS2792)

**Goal:** Fix cross-file symbol resolution

**Current State:**
- Worker 7 has partial implementation (named imports work ✅)
- 800+ combined errors (TS7005: 489, TS7008: 336, TS2792: 161)
- **BLOCKER:** Incomplete for 2+ weeks, blocking real-world usage

**Two-Pronged Approach (Effective 2026-01-15 13:50):**

**Worker 7 - Namespace and Default Imports**
- Subtask 7.1: Implement namespace import resolution
- Subtask 7.2: Implement default import resolution
- Subtask 7.3: Fix export table building for namespaces
- Subtask 7.4: Test with conformance suite
- **Commit requirement:** Daily commits with `[wasm] binder: module - <subtask>`

**Worker 6 - Re-exports and TS2792**
- Subtask 6.1: Implement re-export resolution (`export * from 'x'`)
- Subtask 6.2: Implement named re-exports (`export { a } from 'x'`)
- Subtask 6.3: Implement TS2792 `import()` type resolution
- Subtask 6.4: Test with conformance suite
- **Commit requirement:** Daily commits with `[wasm] binder: module - <subtask>`

**Coordination:**
- EM-2 facilitates daily sync between Workers 6-7
- Shared workspace: `wasm/src/binder/mod.rs`, `wasm/src/thin_binder.rs`
- Code reviews required before merging to em-team-2
- Prevent merge conflicts through communication

**Target Metrics:**
| Error Code | Current | Target | Owner |
|------------|---------|--------|-------|
| TS7005 extra | 489 | <100 | Worker 7 |
| TS7008 extra | 336 | <50 | Worker 7 |
| TS2792 missing | 161 | <20 | Worker 6 |
| Re-exports | ??? | 0 extra | Worker 6 |

**Key Files:**
- `wasm/src/binder/mod.rs` - module resolution
- `wasm/src/thin_binder.rs` - symbol table management
- **Shared workspace** - Workers 6-7 must coordinate to avoid conflicts

**Success Criteria:**
- Week 1 (by 2026-01-22): Substantial progress on namespace/defaults (Worker 7)
- Week 1 (by 2026-01-22): Substantial progress on re-exports (Worker 6)
- Week 2 (by 2026-01-29): TS2792 implementation complete (Worker 6)
- Week 2 (by 2026-01-29): All module errors reduced by 50%

### Priority 2: LSP TypeScript Config Integration
**Owner:** Worker 8 (LSP Squad)

**Goal:** LSP features should respect project's tsconfig.json settings

**Problem:** LSP features hardcode `strict = false` instead of reading tsconfig:
```rust
// wasm/src/lsp/hover.rs:110
let strict = false; // TODO: get from tsconfig

// wasm/src/lsp/project.rs:416
let strict = false; // TODO: get from tsconfig
```

**Infrastructure Already Exists:**
- ✅ `wasm/src/cli/config.rs` has `TsConfig` parsing
- ✅ `load_tsconfig(path: &Path)` function available
- ✅ `resolve_compiler_options()` handles `strict` flag
- ✅ `CheckerOptions` struct has `strict` field

**Action Items:**
1. Add tsconfig discovery to Project
2. Update LSP features to use resolved strict setting:
   - `hover.rs`: Use `project.get_strict()` instead of `false`
   - `project.rs`: Use `project.get_strict()` instead of `false`
   - `signature_help.rs`: Use `project.get_strict()` instead of `false`
   - `completions.rs`: Use `project.get_strict()` instead of `false`
3. Handle tsconfig changes (watch for modifications, reinitialize)

**Success Criteria:**
- LSP respects project's `strict: true` setting
- LSP respects project's `strict: false` setting
- tsconfig.json changes trigger project reinitialization
- No breaking changes to existing behavior

**Estimated Effort:**
- 1-2 hours implementation
- 1 hour testing

**Risk Assessment:** Low risk - localized to LSP module, default behavior preserved

### Priority 3: Type Checker Accuracy (Tier 2)
**Owner:** Worker 6 (after completing current task)

**Goal:** Emit correct semantic errors

**Remaining Issues:**
- TS2571 over-reporting investigation
- TS2683 missing in some contexts
- Other semantic errors

**Action Items:**
1. Investigate TS2571 "Object is of type 'unknown'" over-reporting
2. Investigate TS2683 "'this' implicitly has type 'any'" missing
3. Fix remaining semantic error gaps

---

## Conformance Test Baseline (2026-01-15)

Current baseline from rust branch (commit 74df9fd30d):

**Top Semantic Errors:**
- TS7005: 489 extra (Symbol cannot be referenced from module)
- TS7008: 336 extra (Module has no exported member)
- TS2792: 161 missing (import() type resolution)

**Target:** Fix 800+ module errors, maintain type checker accuracy

---

## Workflow

### For EM-2:
1. **Daily sync**: `git pull origin rust` → merge to em-team-2
2. **Review worker branches**: Check commits, test results
3. **Merge locally**: `git merge worker-X` into em-team-2
4. **Run validation**: `./wasm/differential-test/run-conformance.sh --max=500 --workers=4`
5. **Push to director**: Only when stable and validated

### For Workers:
1. Create branch from em-team-2
2. Work on assigned task ONLY
3. Commit frequently with `[wasm] semantics: <description>`
4. Push to worker-X branch
5. Update task list with status
6. Notify EM-2 when ready for merge

---

## Merge Readiness Status (2026-01-15 13:50)

| Worker | Status | Notes |
|--------|--------|-------|
| Worker 6 | 🔵 Active | Reassigned to help Worker 7 - re-exports, TS2792 |
| Worker 7 | 🔵 Active | Module resolution (partial) - namespace, defaults |
| Worker 8 | 🟢 Approved | LSP config integration approved, ready to start |

---

## Escalation Path

1. Worker commits → worker-X branch
2. EM-2 merges to em-team-2 → validates
3. EM-2 escalates to Director when stable
4. Director reviews → merges to rust

**Do NOT push directly to rust.**

---

## Next Actions for EM-2

1. ✅ Create em-team-2 branch
2. ✅ Create EM_2_TASKS.md
3. ✅ **Worker 6 reassigned** to help Worker 7 (2026-01-15 13:50)
4. ✅ **All workers assigned new tasks** (2026-01-15 14:05)
5. 🔄 **Monitor worker progress** on current tasks
6. 🔄 **Merge worker branches** as they complete tasks
7. 📋 Track error reduction metrics (conformance tests)

---

## Current Task Assignments (2026-01-15 14:05)

### Worker 5: Fix TS2348 "Cannot Invoke Expression" Over-Reporting
- **Priority:** 🟡 HIGH (Tier 2 - Type Checker Accuracy)
- **Squad:** Syntax Squad
- **Problem:** Type checker emits TS2348 errors for callable expressions
- **Files:** `wasm/src/thin_checker.rs`
- **Target:** Reduce TS2348 over-reporting by 50%+

### Worker 6: Fix TS7006/TS7005 Implicit Any Over-Reporting
- **Priority:** 🔴 CRITICAL (Tier 4 - Implicit Any Checks)
- **Squad:** Binder Squad
- **Problem:** ~200 extra TS7006 and ~150 extra TS7005 errors
- **Root Cause:** Doesn't check if type can be inferred from default values/initializers
- **Files:** `wasm/src/thin_checker.rs`
- **Target:** Reduce TS7006 from ~200 to <100, TS7005 from ~150 to <75
- **Coordination:** Worker-3 (EM-1) also working on TS7006

### Worker 7: Complete Module Import Resolution
- **Priority:** 🔴 CRITICAL (Tier 3 - Module System)
- **Squad:** Semantics Squad
- **Status:** 🔄 Reassigned to complete partial implementation
- **Completed:** Basic named imports
- **Missing:** Namespace imports, default exports, re-exports, dynamic imports
- **Files:** `wasm/src/binder/mod.rs`, `wasm/src/thin_binder.rs`
- **Target:** TS7005 <100, TS7008 <50, TS2792 <20

### Worker 8: Fix TS2322 Type Assignability Over-Reporting
- **Priority:** 🟡 HIGH (Tier 1 - Type Assignability)
- **Squad:** LSP Squad
- **Problem:** ~6 extra TS2322 errors in conformance tests
- **Root Cause:** May not recognize structural type compatibility, type narrowing
- **Files:** `wasm/src/thin_checker.rs`, `wasm/src/checker/types/subtype.rs`
- **Target:** Reduce TS2322 from ~6 to <3
- **Coordination:** Worker-2 (EM-1) also working on type assignability

---

## Completed Work Summary

### Worker 5 (Syntax Squad) ✅
- ASI (Automatic Semicolon Insertion) implementation
- Statement-level error recovery
- Object/array literal error recovery
- Control statement error recovery
- ESLint ignore for WASM test library
- **Impact:** TS1109 reduced from 700+ to 7

### Worker 6 (Binder Squad) ✅
- TS2589 recursion guards
- TS2454 lib.d.ts global values fix
- Lib symbol injection enhancement
- **Impact:** TS2304 reduced to 7 missing

### Worker 7 (Semantics Squad) ⚠️
- Module import resolution (partial - basic named imports only)
- **Status:** Salvaged and merged, needs completion

### Worker 8 (LSP Squad) ✅
- TS2564 verification (implementation already existed)
- LSP TypeScript Config Integration
- **Impact:** LSP now respects tsconfig.json strict setting

---

## Notes

- All work must stay in `wasm/` directory
- TypeScript source files (`src/compiler/`) are READ-ONLY
- Run `./wasm/test.sh` for Rust tests (Docker-only)
- Use `./scripts/ask-gemini.mjs` before coding (if applicable)
- Target: 95%+ exact match before production
- **Focus:** Semantic accuracy (module resolution, LSP, type checker)

---

## Team Size and Capacity

**Current Workers:** 3 (Worker 6, Worker 7, Worker 8)
**Capacity:** At maximum (limit is 4)

**Future Considerations:**
- Can accept 1 more worker if needed
- Focus on semantic tasks (module resolution, LSP, type checker)
- Worker 7's module resolution work is highest priority (800+ errors)

---

## Success Metrics

| Metric | Current | Target (EM-2) |
|--------|---------|---------------|
| TS7005 extra errors | 489 | <100 |
| TS7008 extra errors | 336 | <50 |
| TS2792 missing errors | 161 | <20 |
| Exact Match Rate | ~30% | 40%+ |

**Overall EM-2 Goal:** Fix 800+ module errors, improve exact match rate by 10%
