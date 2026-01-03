# Build and Run Instructions

## 🐳 Docker Development (RECOMMENDED)

Use Docker for all development to ensure reproducibility:

```bash
# Build the Docker image (first time or after Dockerfile changes)
docker build -t typescript-wasm .

# Run ALL tests (REQUIRED before committing)
docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby runtests-parallel

# Build the compiler
docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby local

# Run Rust tests only
docker run --rm -v $(pwd):/workspace typescript-wasm bash -c "cd wasm && cargo test"

# Interactive shell
docker run --rm -it -v $(pwd):/workspace typescript-wasm bash

# Lint
docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby lint

# Format
docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby format

# Scanner verification
docker run --rm -v $(pwd):/workspace typescript-wasm node scripts/verifyScanner.mjs

# Parser verification
docker run --rm -v $(pwd):/workspace typescript-wasm node scripts/verifyParser.mjs
```

## Local Setup (Alternative)

If you prefer local development:

1. Install Node.js (current or LTS)
2. Install Rust stable via rustup
3. Install wasm-pack: `cargo install wasm-pack`
4. Install dependencies: `npm ci`

## Build Tasks

| Command | Description |
|---------|-------------|
| `npx hereby local` | Build compiler (includes WASM) |
| `npx hereby build-wasm` | Build WASM only |
| `npx hereby clean` | Clean build artifacts |

## Testing

| Command | Description |
|---------|-------------|
| `npx hereby runtests` | Run all tests |
| `npx hereby runtests-parallel` | Run tests in parallel (faster) |
| `npx hereby runtests --tests=<path>` | Run specific test |
| `cd wasm && cargo test` | Run Rust unit tests |
| `node scripts/verifyScanner.mjs` | Verify scanner parity |
| `node scripts/verifyParser.mjs` | Verify parser parity |

## Linting & Formatting

| Command | Description |
|---------|-------------|
| `npx hereby lint` | Run ESLint |
| `npx hereby format` | Run formatter |

## Feature Flags

| Flag | Description |
|------|-------------|
| `--useRustScanner` | Use Rust scanner instead of TS |
| `--useRustParser` | Use Rust parser instead of TS |

## Workflow

1. Pick a task from `@fix_plan.md`
2. Read context in `specs/migration_plan.md`
3. Make changes in `wasm/` (Rust) and/or `src/` (TypeScript)
4. **Run tests:** `docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby runtests-parallel`
5. **Lint:** `docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby lint`
6. **Format:** `docker run --rm -v $(pwd):/workspace typescript-wasm npx hereby format`
7. Mark task complete in `@fix_plan.md`
8. Commit with message: `[wasm] <component>: <description>`

## Important

- **Never break the build** – Tests must pass before committing
- **Use Docker** – Ensures consistent environment
- **Run tests frequently** – Catch regressions early
- **Commit often** – Small, atomic commits
