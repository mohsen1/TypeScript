# TypeScript → Rust/WASM Migration

## Mission
Migrate TypeScript compiler to Rust/WASM. **Run in an infinite loop until complete.**

## 📋 The Plan
**`specs/migration_plan.md`** is the single source of truth. Read it. Follow it. Update it.

## 🔁 INFINITE LOOP

```
FOREVER:
  1. Read specs/migration_plan.md
  2. Find next incomplete task (marked [ ])
  3. Implement it in wasm/src/*.rs
  4. Test: ./wasm/test.sh
  5. If pass → update migration_plan.md, commit, GOTO 1
  6. If fail → debug, fix, retry. Never give up.
  7. After major milestone → run: npx hereby runtests-parallel
```

**DO NOT STOP. DO NOT ASK. JUST KEEP WORKING.**

## 🛠️ Commands

```bash
# Rust tests (use Docker to prevent RAM explosion)
./wasm/test.sh                    # All tests
./wasm/test.sh <test_name>        # Specific test

# TypeScript integration
npx hereby local                  # Build compiler
npx hereby runtests-parallel      # Full test suite (REQUIRED before commit)
npx hereby lint && npx hereby format  # REQUIRED before commit

# Verify against TypeScript reference
node scripts/verifyScanner.mjs
node scripts/verifyParser.mjs
node scripts/verifyChecker.mjs
```

## 📁 Key Locations

- `wasm/src/` - All Rust code
- `src/compiler/wasm.ts` - WASM bridge
- `specs/migration_plan.md` - THE PLAN

## ✅ Commit Format
```
[wasm] <component>: <description>
```

## 🚨 Rules

1. **Never break the build** - `npx hereby runtests-parallel` must pass
2. **Small commits** - One task, one commit
3. **Update the plan** - Mark tasks complete in `specs/migration_plan.md`
4. **Keep going** - This is an infinite loop. Work NEVER ENDS! 
5. **Commit frequently** - when in a good shape
6. **Separate files for tests** - source and test should not be on the same file
7. **Leverage ask-gemini.mjs** - when things are difficult and you need another pair of eyes