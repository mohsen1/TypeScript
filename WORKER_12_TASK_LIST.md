# WORKER-12 Task List

**EM:** EM-4
**Focus:** Parser Accuracy (Tier 1) - Fix TS1109 and TS1005 extra errors

## Mission

Fix parser false positives that cause "Expression expected" (TS1109) and "X expected" (TS1005) errors on valid TypeScript code.

## Current Baseline

From conformance tests (200 files):
- **TS1109 Extra:** 7 occurrences - parser emits "Expression expected" for valid syntax
- **TS1005 Extra:** 5 occurrences - parser emits "X expected" for valid constructs
- **TS1109 Missing:** 7 occurrences - parser should emit but doesn't

This indicates the parser has edge cases in:
- Expression statement detection
- Automatic Semicolon Insertion (ASI)
- Distinguishing declarations from expressions

## Key Files

| File | Purpose |
|------|---------|
| `wasm/src/thin_parser.rs` | Main parser (~8k lines) |
| `wasm/src/scanner.rs` | Tokenization |
| `wasm/src/checker/types/diagnostics.rs` | Error code definitions |

## Steps

### 1. Reproduce the Issue

Find failing test files:
```bash
cd wasm/differential-test
bash run-conformance.sh --max=200 --workers=4 | grep -A2 "TS1109\|TS1005"
```

Look for files like:
- `async/es2017/asyncArrowFunction/asyncArrowFunction3_es2017.ts`
- `async/es2017/functionDeclarations/asyncFunctionDeclaration3_es2017.ts`
- `async/es2017/asyncArrowFunction/asyncArrowFunction7_es2017.ts`

### 2. Create Minimal Repro

For each failing test, extract the minimal pattern that triggers the false positive:

```javascript
// Example repro file in /tmp/test1109.ts
// Run: tsc /tmp/test1109.ts  (should pass)
// Run: node test.mjs with WASM (currently fails)
```

### 3. Compare Parse Trees

Add debug logging to see what the parser is doing vs what TSC does:
- Check expression statement parsing
- Check ASI handling
- Check how async arrows are detected

### 4. Fix the Parser

Common TS1109/TS1005 causes:
1. **ExpressionStatement vs Declaration:** Parser misidentifies `async` as a statement
2. **ASI Edge Cases:** Missing newline detection before certain tokens
3. **Contextual Keywords:** `async`, `await` as identifiers vs keywords

### 5. Validate

```bash
# Build
cd wasm && wasm-pack build --target web --out-dir pkg

# Quick test (should see reduction in TS1109/TS1005 extra errors)
cd differential-test && bash run-conformance.sh --max=200 --workers=4
```

## Success Criteria

- **TS1109 Extra Errors:** Reduced from 7 to ≤2
- **TS1005 Extra Errors:** Reduced from 5 to ≤1
- **No Regressions:** TS1109 Missing errors don't increase significantly

## Resources

- Parser: `wasm/src/thin_parser.rs:1-8500`
- Diagnostics: `wasm/src/checker/types/diagnostics.rs`
- Spec: `wasm/specs/` for architecture context

## Submit Your Work

1. Create branch: `git checkout -b worker-12-parser-fixes`
2. Commit with clear messages
3. Run validation: `bash run-conformance.sh --max=500 --workers=8`
4. Notify EM-4 for review
