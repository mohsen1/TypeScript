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

## Task Queue
- [ ] After emitter edge cases: help with private identifier control flow

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: Strip readonly modifier from JS output`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
