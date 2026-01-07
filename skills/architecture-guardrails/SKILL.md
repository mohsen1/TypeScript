---
name: architecture-guardrails
description: Scans for forbidden architecture patterns and cross-checks against WASM_ARCHITECTURE.md.
---

# Architecture Guardrails

Use this when reviewing code changes for architectural violations.

## References
- `wasm/specs/WASM_ARCHITECTURE.md`
- `wasm/specs/SOLVER.md` (for solver boundaries)

## Guardrail checks (example rg commands)
- Legacy AST usage: `rg -n "parser/ast|FatNode" wasm/src`
- Inline transforms in emitter: `rg -n "target_es5|inline transform|TransformContext" wasm/src/thin_emitter`
- Solver bypass from checker: `rg -n "ThinChecker|assignability|type relation" wasm/src`

## Workflow
1. Read `wasm/specs/WASM_ARCHITECTURE.md` and note the non-negotiables.
2. Run targeted scans for forbidden patterns.
3. Inspect any hits and determine if they violate the architecture.
4. Report findings in the worker plan:
   - File path, line, and why it violates the architecture.
   - Suggested fix or follow-up task.
