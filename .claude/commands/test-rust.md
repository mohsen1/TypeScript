---
description: Build and test Rust code, then run TypeScript tests
allowed-tools: Bash
---

# Test Rust and TypeScript

Run the full test suite for both Rust and TypeScript.

## Steps

1. Build and test Rust (in Docker):
```bash
cd /Users/mohsenazimi/code/TypeScript/wasm && \
docker build -t rust-wasm-tests . && \
docker run --rm --memory="1g" --cpus="2.0" rust-wasm-tests
```

2. Build TypeScript:
```bash
cd /Users/mohsenazimi/code/TypeScript && npx hereby local
```

3. Run TypeScript tests (if requested or after significant changes):
```bash
cd /Users/mohsenazimi/code/TypeScript && npx hereby runtests-parallel
```

Report results including:
- Number of Rust tests passed
- Build warnings/errors
- Any test failures
