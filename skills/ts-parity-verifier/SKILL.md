---
name: ts-parity-verifier
description: Runs TypeScript parity checks and logs pass rates and key failures.
---

# TS Parity Verifier

Use this to validate behavior against TypeScript reference tests.

## Workflow
1. Run these commands in order:
   - `node scripts/verifyScanner.mjs`
   - `node scripts/verifyParser.mjs`
   - `node scripts/verifyChecker.mjs`
   - `npx hereby runtests-parallel`
2. Capture pass rates, failures, and any notable regressions.
3. Update the active worker plan with:
   - Commands run
   - Pass rates
   - Top failures + links or file paths
4. If results are materially different, update the Executive Summary in `wasm/README.md`.
