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

| Worker | Squad | Focus Area | Status | Throughput |
|--------|-------|------------|--------|------------|
| Worker 6 | Binder | Global scope / lib injection | ✅ Complete | Medium |
| Worker 7 | Semantics | Module symbol resolution | 🔵 Active | Medium |
| Worker 8 | LSP | TypeScript config integration | 🟢 Approved | TBD |

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

### Priority 1: Module Symbol Resolution (Tier 3)
**Owner:** Worker 7 (Semantics Squad)

**Goal:** Fix cross-file symbol resolution

**Current State:**
- Worker 7 has partial implementation (named imports work)
- 800+ combined errors (TS7005, TS7008, TS2792)
- Incomplete: namespace, default, re-exports, TS2792

**Action Items:**
1. Build export tables for each module
2. Resolve imports against export tables
3. Handle namespace imports
4. Handle default imports
5. Handle re-exports
6. Implement TS2792 `import()` type resolution

**Target Metrics:**
| Error Code | Current | Target |
|------------|---------|--------|
| TS7005 extra | 489 | <100 |
| TS7008 extra | 336 | <50 |
| TS2792 missing | 161 | <20 |

**Key Files:**
- `wasm/src/binder/mod.rs` - module resolution
- `wasm/src/thin_binder.rs` - symbol table management

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

## Merge Readiness Status (2026-01-15 13:20)

| Worker | Status | Notes |
|--------|--------|-------|
| Worker 6 | 🟢 Complete | TS2454, TS2589, lib injection complete |
| Worker 7 | 🔵 Active | Module resolution (partial implementation) |
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
3. 🔄 **Continue Worker 7** on module resolution to completion
4. 📅 **Activate Worker 8** on LSP TypeScript config integration
5. 📅 **Assign new task** to Worker 6 (type checker accuracy)
6. 📋 Track semantic error counts

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
