# Squad Anvil Goals

Updated: 2026-01-11

## Current Status
- **Unit Tests:** 5024/5101 passing (98.5%)
- **Conformance:** 17.9% exact match, 23.4% same error count
- **Anvil-related failures:** 5 emitter/transforms + 3 parser + 3 CLI = 11 tests

## Current Milestone
**Milestone 3: Emitter Parity** - Fix transform and emitter issues

## Objectives (Ranked)

### 1. **Fix ES5 Private Accessors (3 tests)** - HIGH PRIORITY
   - Context: Private getters/setters need WeakMap-based emission
   - Success Criteria: All `parity_es5_private_accessor_*` tests pass
   - Key Files:
     - `src/transforms/class_es5.rs` - add private accessor handling
     - `src/transforms/private_fields_es5.rs` - accessor transformation
   - Estimated Complexity: Medium (2 days)
   - Implementation:
     1. Generate WeakMap variables: `_ClassName_fieldName_get`, `_ClassName_fieldName_set`
     2. In constructor: `_map.add(this, function() { ...body... })`
     3. Access transforms: `__classPrivateFieldGet(this, _map, "a")` where "a" = accessor

### 2. **Fix Emitter Edge Cases (2 tests)** - QUICK WINS
   - Context: Readonly modifier emission, try-throw parentheses
   - Success Criteria: `readonly_class_members` and `try_throw_parenthesized` tests pass
   - Key Files:
     - `src/thin_emitter.rs` - skip `readonly` keyword in JS output
   - Estimated Complexity: Low (4 hours)

### 3. **Fix Parser Error Recovery (3 tests)** - MEDIUM
   - Context: Parser needs to produce partial AST on errors for LSP
   - Success Criteria: All `thin_parser_*_recovers` tests pass
   - Key Files:
     - `src/thin_parser.rs` - add synchronization points
   - Estimated Complexity: Medium (2 days)
   - Details:
     - `function_keyword_in_class_recovers` - skip to next member
     - `jsx_like_syntax_in_ts_recovers` - handle `<Type>` ambiguity
     - `type_assertion_in_new_expression_reports_ts1109` - proper error code

### 4. **Fix CLI/Driver Issues (3 tests)** - MEDIUM
   - Context: Generic utility library, shorthand methods, import equals
   - Success Criteria: All `cli::driver_tests::*` pass
   - Key Files:
     - `src/cli/driver.rs` - compilation handling
     - `src/thin_emitter.rs` - import equals (`import x = require()`)
   - Estimated Complexity: Medium (1-2 days)

### 5. **Fix Control Flow Private Identifier (1 test)** - QUICK WIN
   - Context: `#privateField in obj` should narrow types
   - Success Criteria: `test_in_operator_private_identifier_narrows` passes
   - Key Files:
     - `src/checker/control_flow.rs` - add `PrivateIdentifier` to `narrow_type_by_condition`
   - Estimated Complexity: Low (2 hours)

## Conformance Focus Areas

### Parser Error Handling
The parser currently produces syntax errors that differ from TypeScript:
- **TS1005** (65 extra): `,` expected - likely overeager parsing
- **TS1109** (25 extra): Expression expected - recovery issues
- **TS1068** (15 extra): Unexpected token

### Emitter Missing Features
Check if these are causing conformance failures:
- CommonJS module emission
- Import/export transformations
- Decorator metadata emission

## Anti-Priorities
- Do NOT work on type checker logic (that's Forge's domain)
- Do NOT implement new TypeScript features
- Do NOT optimize performance yet

## Cross-Squad Dependencies
- May need Forge to fix type resolution for proper emit

## Notes to EM
- Run tests with `./wasm/test.sh --no-fail-fast 2>&1 | grep FAIL`
- For emitter parity: compare output with `npx tsc --target ES5`
- Use `./scripts/ask-gemini.mjs --review` for code review

## Squad Status
- Last EM Report: Session starting
- Workers Active: 0/5
- Branches Pending Merge: None
- Current Focus: ES5 private accessors + emitter edge cases
- Blockers: None
