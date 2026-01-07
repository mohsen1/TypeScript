---
name: bench-gatekeeper
description: Runs real_world_bench in Docker, compares throughput to the latest baseline, and logs deltas.
---

# Bench Gatekeeper

Use this when checking performance changes for wasm.

## Rules
- Always use `./wasm/bench.sh` (Docker). Never run `cargo bench` directly.

## Workflow
1. Run: `./wasm/bench.sh real_world_bench`
2. Extract throughput lines (look for `thrpt:`) from the output.
3. Find the latest baseline:
   - Prefer the current worker plan or the last benchmark entry in `wasm/README.md`.
4. Compare each relevant throughput line to the baseline and compute deltas.
5. Update:
   - Worker plan: record the raw results and deltas.
   - `wasm/README.md` Executive Summary only if there is a notable regression or gain.

## Regression rule of thumb
- Flag regressions greater than 5% or consistent multi-run slowdown.
