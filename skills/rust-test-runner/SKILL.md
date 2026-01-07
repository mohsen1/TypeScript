---
name: rust-test-runner
description: Runs Rust tests via Docker-only wrapper and records results in worker plans.
---

# Rust Test Runner

Use this when running Rust tests for the wasm codebase.

## Rules
- Always use `./wasm/test.sh` (Docker). Never run `cargo test` directly.
- If Docker is unavailable, stop and report the failure.

## Workflow
1. Identify the scope: full suite or a named test.
2. Run:
   - Full: `./wasm/test.sh`
   - Specific: `./wasm/test.sh <test_name>`
3. Capture the result summary (pass/fail, failing tests, key errors).
4. Update the active worker plan with:
   - Command used
   - Result summary
   - Follow-ups if failures occurred

## Plan update snippet
- Tests: `./wasm/test.sh <test_name>`
- Result: PASS/FAIL + short error summary
