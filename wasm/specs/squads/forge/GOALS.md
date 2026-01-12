# Squad Forge Goals

Updated: 2026-01-11

## Current Status
- **Unit Tests:** 5024/5101 passing (98.5%)
- **Conformance:** 17.9% exact match, 23.4% same error count
- **Forge-related failures:** 47 thin_checker + 3 solver + 1 control_flow = 51 tests

## Current Milestone
**Milestone 3: Type System Parity** - Fix checker/solver issues blocking conformance

## Objectives (Ranked)

### 1. **Fix Namespace Merging (12 tests)** - HIGH PRIORITY
   - Context: Class/Enum/Function + Namespace declarations must merge properly
   - Success Criteria: All 12 `namespace_merges_with_*` tests pass
   - Key Files:
     - `src/binder.rs` - `can_merge_flags()`, `bind_module_declaration()`, `bind_class_declaration()`
     - `src/thin_checker.rs` - namespace member resolution
   - Estimated Complexity: Medium (2 days)
   - Details:
     1. Update `can_merge_flags` to allow CLASS|FUNCTION|ENUM + MODULE merging
     2. Merge namespace exports into `symbol.exports` table
     3. Merge class statics into same `symbol.exports` table
     4. Correct flag combination: `CLASS | VALUE_MODULE | NAMESPACE_MODULE`

### 2. **Fix Element Access with Literal Keys (5 tests)** - HIGH PRIORITY
   - Context: `obj["prop"]` with string literal should behave like `obj.prop`
   - Success Criteria: All `element_access_literal_key_*` tests pass
   - Key Files:
     - `src/thin_checker.rs` - element access type resolution
   - Estimated Complexity: Low (1 day)

### 3. **Fix Method Bivariance (3 tests)** - QUICK WIN
   - Context: Method parameters should be bivariant, not contravariant
   - Success Criteria: All `method_bivariance_*` tests pass
   - Key Files:
     - `src/solver/logic.rs` - `solve_subtype()` - check `strict_function_types` flag
   - Estimated Complexity: Low (2 hours)

### 4. **Fix New Expression Inference (4 tests)** - MEDIUM
   - Context: `new Class()` should infer proper instance type with inherited properties
   - Success Criteria: All `new_expression_*` tests pass
   - Key Files:
     - `src/thin_checker.rs` - `check_new_expression()`
   - Estimated Complexity: Medium (1-2 days)

### 5. **Fix Solver Generic Inference (3 tests + Redux patterns)** - COMPLEX
   - Context: Generic number index, tuple rest, and advanced patterns
   - Success Criteria: Solver tests pass, redux patterns work
   - Key Files:
     - `src/solver/infer.rs` - generic instantiation
     - `src/solver/operations.rs` - index signature handling
   - Estimated Complexity: High (3+ days)

## Conformance Focus Areas

### Missing Error Codes to Implement
- **TS2564**: Property has no initializer (64 occurrences)
- **TS2454**: Variable used before assignment (43 occurrences)
- **TS7006**: Parameter implicitly has 'any' type (42 occurrences)
- **TS2705**: Async function must return Promise (37 occurrences)
- **TS2322**: Type not assignable (19 occurrences - some missing)

### Extra Error Codes to Fix (False Positives)
- **TS2304**: Cannot find name (129 extra) - likely scope resolution issue
- **TS2355**: Function must return value (82 extra) - control flow analysis
- **TS2339**: Property does not exist (35 extra) - type narrowing issue

## Anti-Priorities
- Do NOT work on emitter/transforms (that's Anvil's domain)
- Do NOT work on LSP features
- Do NOT refactor working code without tests failing

## Cross-Squad Dependencies
- Anvil may need solver changes for transform type queries

## Notes to EM
- Run tests with `./wasm/test.sh --no-fail-fast 2>&1 | grep FAIL`
- Run conformance with `cd wasm/differential-test && ./run-conformance.sh`
- Use `./scripts/ask-gemini.mjs --review` for code review

## Squad Status
- Last EM Report: Session starting
- Workers Active: 0/5
- Branches Pending Merge: None
- Current Focus: Namespace merging + Method bivariance
- Blockers: None
