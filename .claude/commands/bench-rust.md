---
description: Run Rust benchmarks in Docker
allowed-tools: Bash
---

# Run Rust Benchmarks

Run the Rust benchmarks suite inside a Docker container to ensure consistent environment and resource limits.

## Steps

1. Build and run benchmarks (in Docker):
```bash
cd /Users/mohsenazimi/code/TypeScript/wasm && \
docker build -t rust-wasm-tests . && \
docker run --rm --memory="2g" --cpus="4.0" rust-wasm-tests cargo bench
```

Report results including:
- Benchmark names and execution times
- Any performance regressions or improvements if comparing
