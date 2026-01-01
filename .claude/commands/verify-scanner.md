---
description: Verify Rust scanner matches TypeScript scanner token-by-token
allowed-tools: Bash, Read
---

# Verify Rust Scanner

Run token-by-token verification between Rust and TypeScript scanners.

## File to verify: $ARGUMENTS

## Steps

1. Run the verification script:
```bash
node scripts/verifyScanner.mjs $ARGUMENTS
```

2. Analyze the output:
   - Look for any mismatches in token kind, value, or position
   - Check for missing or extra tokens
   - Verify edge cases (templates, regex, comments)

3. If mismatches found:
   - Read the relevant section of `wasm/src/scanner_impl.rs`
   - Compare with TypeScript's `src/compiler/scanner.ts`
   - Fix the Rust implementation

4. After fixing, re-run verification and tests:
```bash
node scripts/verifyScanner.mjs $ARGUMENTS
cd wasm && cargo test
```
