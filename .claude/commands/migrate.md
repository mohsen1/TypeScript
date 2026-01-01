---
description: Migrate a TypeScript compiler component to Rust
allowed-tools: Read, Grep, Glob, Bash, Edit, Write
---

# Migrate TypeScript Component to Rust

You are migrating a TypeScript compiler component to Rust/WASM. Follow this structured workflow:

## Task: $ARGUMENTS

## Step 1: Analyze the TypeScript Implementation

1. Read the TypeScript source file(s) for the component
2. Identify all public APIs and their signatures
3. List dependencies on other compiler modules
4. Note any complex patterns (closures, generics, etc.)

## Step 2: Check MIGRATION_PLAN.md

1. Read MIGRATION_PLAN.md to understand current phase
2. Verify this component fits in the current phase
3. Note any dependencies that must be migrated first

## Step 3: Implement in Rust

1. Create or update Rust files in `wasm/src/`
2. Match TypeScript's behavior exactly
3. Add wasm-bindgen exports for public APIs
4. Handle borrow checker issues with local variables

## Step 4: Create TypeScript Bridge

1. Add types to `WasmModule` interface in `src/compiler/wasm.ts`
2. Create wrapper functions
3. Add feature flag if needed

## Step 5: Test

Run in sequence:
```bash
cd wasm && cargo build && cargo test
npx hereby local
npx hereby runtests-parallel
```

## Step 6: Commit

After all tests pass:
```bash
git add wasm/ src/compiler/wasm.ts
git commit -m "feat(wasm): migrate <component> to Rust"
```

## Step 7: Update Documentation

Update MIGRATION_PLAN.md with:
- Checklist items completed
- Progress log entry with date and details
