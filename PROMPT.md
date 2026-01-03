# Ralph Development Instructions

You are working on the migration of the TypeScript compiler to Rust (WASM).
Your goal is to complete the tasks listed in `@fix_plan.md`.

## 🐳 ALWAYS USE DOCKER

```bash
# Run tests (REQUIRED before any commit)
docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby runtests-parallel

# Build
docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby local

# Rust tests
docker run --rm -v $(pwd):/workspace typescript-wasm bash -c "cd wasm && cargo test"
```

## Context

- **Rust code:** `wasm/src/`
- **TypeScript code:** `src/compiler/`
- **Task list:** `@fix_plan.md`
- **Full migration plan:** `specs/migration_plan.md`
- **Build instructions:** `@AGENT.md`

## Workflow

1. Read `@fix_plan.md` for current priorities
2. Pick the highest priority incomplete task
3. Read `specs/migration_plan.md` for detailed context
4. Implement in Rust (`wasm/`) and/or TypeScript (`src/`)
5. Run tests: `docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby runtests-parallel`
6. Mark task complete in `@fix_plan.md`
7. Commit: `[wasm] <component>: <description>`

## Guiding Principles

1. **Never break the build** – Every commit must pass tests
2. **Iterate in small slices** – One function, one module at a time
3. **Test before & after** – Run tests frequently
4. **Use Docker** – Ensures consistent environment
5. **Document changes** – Update `@fix_plan.md` and `specs/migration_plan.md`

## Current Focus

Phase 5: Type Checker (~60% complete)
- 215 Rust tests passing
- Next: Fix property access on unions (infinite loop bug)
