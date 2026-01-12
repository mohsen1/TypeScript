# Worker 1 Plan - Squad Anvil

## Mission
Fix Emitter Edge Cases - QUICK WIN tasks

Status: Active
Priority: P0 (Highest)

## Current Assignment
**Fix Emitter Edge Cases (2 failing tests)**

### Background
The emitter has small bugs in handling readonly modifiers and parentheses around throw expressions.

### Failing Tests
1. `test_class_es5_readonly_class_members`
2. `test_two_phase_emission_es5_class_try_throw_parenthesized`

### Implementation Steps for Readonly

1. [ ] Read the failing test `test_class_es5_readonly_class_members` in `src/transforms/class_es5_tests.rs`
2. [ ] Find where class members are emitted in `src/thin_emitter.rs`
3. [ ] Ensure `readonly` keyword is NOT emitted to JavaScript output
4. [ ] The `readonly` modifier only exists in TypeScript - strip it during emit
5. [ ] Test: `./wasm/test.sh 2>&1 | grep -E "readonly_class_members"`

### Implementation Steps for Try-Throw Parentheses

1. [ ] Read the failing test `test_two_phase_emission_es5_class_try_throw_parenthesized`
2. [ ] Find throw statement emission in `src/thin_emitter.rs`
3. [ ] Check if parentheses are being added or removed incorrectly
4. [ ] Ensure parentheses are preserved when the throw argument is a complex expression
5. [ ] Test: `./wasm/test.sh 2>&1 | grep -E "try_throw_parenthesized"`

### Key Code Locations
- `src/thin_emitter.rs` - member emission, statement emission
- `src/scanner.rs` - `SyntaxKind::ReadonlyKeyword`

### Expected Fix Pattern
```rust
// In emit_modifier_list or similar:
fn emit_modifier(&mut self, modifier: SyntaxKind) {
    match modifier {
        SyntaxKind::ReadonlyKeyword => {
            // Skip - TypeScript only, not emitted to JS
        }
        // ... other modifiers
    }
}
```

## Progress

### Fixed Issues
1. **test_two_phase_emission_es5_class_try_throw_parenthesized** - FIXED ✅
   - Issue: Catch clause variable `e` was missing (output showed `catch ()` instead of `catch (e)`)
   - Fix: Updated `ClassES5Emitter::emit_try_statement` to properly extract and emit the variable declaration's name
   - Location: `src/transforms/class_es5.rs:2887-2901`

2. **Type assertion emission** - FIXED ✅
   - Issue: AS_EXPRESSION, TYPE_ASSERTION, and SATISFIES_EXPRESSION were not handled in ClassES5Emitter
   - Fix: Added case in `emit_expression` to strip TypeScript-only type assertions
   - Location: `src/transforms/class_es5.rs:3078-3086`

### Remaining Issue
**test_class_es5_readonly_class_members** - PARTIALLY FIXED ⚠️
   - Issue: Method name `with` is being emitted as `Partial`
   - Root cause: Unknown - appears to be a deeper parser/emitter issue
   - The test expects `ImmutableRecord.prototype.with` but output shows `ImmutableRecord.prototype.Partial`
   - Note: This is NOT related to readonly modifiers as initially suspected
   - The `with` method in source has type parameter `Partial<T>`, and somehow `Partial` is being used as the method name
   - Needs further investigation - possibly in parser or arena storage

## Task Queue
- [ ] Investigate `with` vs `Partial` issue in class method emission
- [ ] After emitter edge cases: help with private identifier control flow

## Completed

### Session 1: Emitter Edge Cases
- [x] Fixed catch clause variable emission (65b99a3305)
- [x] Added AS_EXPRESSION/TYPE_ASSERTION/SATISFIES_EXPRESSION handling (65b99a3305)

### Session 2: LSP Semantic Tokens Enhancement
- [x] Added semantic token support for decorators (adea84beac)
- [x] Added semantic token support for type parameters (adea84beac)
- [x] Added semantic token support for modifiers (adea84beac)
- [x] Updated visit_children() to handle modifiers on all declaration types
- [x] Added TYPE_PARAMETER, PROPERTY_DECLARATION, PARAMETER, GET_ACCESSOR, SET_ACCESSOR, CONSTRUCTOR, TYPE_ALIAS_DECLARATION cases

## Ready for Merge
Yes (adea84beac)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: Strip readonly modifier from JS output`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
