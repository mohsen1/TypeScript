# Worker 6 Task List

## Squad: Parser/Scanner - TS1005 Focus

## Current Task - Patterns 4-5 (Import/Export & Conditional Expressions)
**Reference:** `TS1005_REDUCTION_RESULTS.md` - Patterns 4-5 from Worker 1 (TypeScript)

### Pattern 4: Import/Export Specifier Brace Mismatch
**TypeScript Reference:** `src/compiler/parser.ts:4243-4255` - `parsingContextErrors()`

**Problem:** When parsing `import { a from "module"` (missing closing brace), parser encounters `from` and emits "}" expected (TS1005), creating cascading errors from a single missing brace.

**Solution:**
- Check if there's already a recent TS1005 or TS1008 error about an unclosed brace
- Suppress cascading TS1005 errors to avoid duplicate diagnostics
- Use last_error_pos tracking (similar to Worker 7's TS1109 fix)

**Rust Implementation Locations:**
- `wasm/src/thin_parser.rs` - Find import/export parsing
- Look for: `parse_import_declaration`, `parse_export_declaration`, or similar
- Check if `last_error_pos` field exists (used in Worker 7's TS1109 fix)
- Add conditional check before emitting TS1005 for missing braces

### Pattern 5: Conditional Expression Colon Dual Emission
**TypeScript Reference:** `src/compiler/parser.ts:6356-6380` - `parseConditionalExpressionRest()`

**Problem:** When parsing ternary operators with missing colons, `parseExpectedToken` emits TS1005, then code emits another TS1005 when creating the missing node - dual emission for the same error.

**Solution:**
- Remove duplicate TS1005 emission when creating missing node
- `parseExpectedToken` already emitted the error, avoiding dual emissions

**Rust Implementation Locations:**
- `wasm/src/thin_parser.rs` - Find conditional expression parsing
- Look for: `parse_conditional_expression`, `parse_ternary_expression`, or similar
- Find where missing nodes are created after expecting `:`
- Ensure only ONE TS1005 is emitted for missing colon

## Queue
- [ ] Test parser changes on conformance suite to measure reduction
- [ ] Coordinate with Worker 5 to avoid duplicate work
- [ ] Document any Rust-specific patterns discovered

## Completed
- [x] Merge attempt #2 - No commits to merge yet (still at base)

## Context
TS1005 has 42 extra errors in conformance sample. Worker 1 fixed 5 patterns in TypeScript; adapt patterns 4-5 to Rust to reduce cascading and duplicate errors.

### Key Files
- `wasm/src/thin_parser.rs` - main parser implementation
- `src/compiler/parser.ts` - TypeScript reference implementation
- `TS1005_REDUCTION_RESULTS.md` - Detailed pattern analysis

### Search Targets in Rust
```rust
// Import/export parsing:
grep -n "import\|export" wasm/src/thin_parser.rs | grep -i "parse\|declaration"

// Conditional expression parsing:
grep -n "conditional\|ternary\|question" wasm/src/thin_parser.rs

// Last error tracking (from Worker 7's TS1109 fix):
grep -n "last_error_pos" wasm/src/thin_parser.rs
// Line 377: if self.token_pos() != self.last_error_pos
```

### Implementation Strategy
1. **Pattern 4:** Add cascading error check similar to Worker 7's TS1109 fix
2. **Pattern 5:** Find dual emission points and consolidate to single TS1005

### Goal
Adapt TypeScript patterns 4-5 to Rust, reduce TS1005 from current levels toward <50.
