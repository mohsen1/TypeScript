# Worker 2 Task: Fix TS2322 Type Assignability (Tier 2)

## Assignment
Fix type assignability false negatives - the solver is currently too permissive, allowing assignments that should fail. We need to make it stricter.

## Problem Analysis
The type assignability checker has false positives - it's NOT emitting TS2322 errors when it should. The solver may be returning `true` for relationships it doesn't understand, or `Any` type poisoning is masking errors.

**Philosophy**: Better to have "Extra TS2322" (too strict) than "Missing TS2322" (unsound type system).

## Key Files to Modify

### Primary Files
1. `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/solver/subtype.rs`
2. `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/solver/compat.rs`
3. `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/thin_checker.rs`

## Current Architecture

### compat.rs (Compatibility Layer)
- Lines 130-178: `is_assignable()` - main entry point
- Lines 139-144: "Lawyer layer" handles `any` propagation
- Lines 155-161: ERROR types explicitly prevented from silently passing
- Lines 172-173: Delegates to `subtype.is_subtype_of()`

### subtype.rs (Structural Subtyping)
- Core logic engine for TypeScript's structural subtyping
- Implements coinductive semantics for recursive types
- TypeResolver trait for lazy symbol resolution

### Key Insight from Code Comments
Line 157 in compat.rs:
```rust
// Error types should NOT silently pass assignability checks.
// This prevents "error poisoning" where a TS2304 (cannot find name) masks
// downstream TS2322 (type not assignable) errors.
```

## Investigation Areas

### Area 1: Default Return Values in Subtype Checking
**Hypothesis**: When the subtype checker encounters types it doesn't understand, it may be defaulting to `true` (permissive) instead of `false` (strict).

**Tasks**:
1. Review `is_subtype_of()` implementation in subtype.rs
2. Find all `return true` statements - verify each is justified
3. Look for fallback cases that default to `true`
4. Add explicit handling for unrecognized type combinations

**Search Commands**:
```bash
# Find all return true in subtype.rs
grep -n "return true\|=> true" wasm/src/solver/subtype.rs | head -50

# Find match arms with catch-all patterns
grep -n "_ =>" wasm/src/solver/subtype.rs | head -30
```

### Area 2: Any Type Propagation
**Hypothesis**: The "lawyer layer" may be too permissive with `any` types, silencing legitimate structural mismatches.

**Tasks**:
1. Review the `AnyPropagationRules` in the codebase
2. Check if `strict_any_propagation` mode is being used
3. Verify that `any` doesn't suppress structural checking when it shouldn't

**Key Code** (compat.rs lines 114-117):
```rust
pub fn set_strict_any_propagation(&mut self, strict: bool) {
    self.lawyer.set_allow_any_suppression(!strict);
    self.cache.clear();
}
```

**Search Commands**:
```bash
# Find AnyPropagationRules definition
grep -rn "AnyPropagationRules" wasm/src/solver/

# Find uses of strict_any_propagation
grep -rn "strict_any_propagation" wasm/src/
```

### Area 3: Structural Type Comparison for Objects
**Hypothesis**: Object type checking may not properly validate all properties, or may be skipping properties.

**Tasks**:
1. Find object shape comparison logic in subtype.rs
2. Verify that all properties are checked (not just a subset)
3. Ensure excess property checking is working
4. Check optional vs required property handling

**Search Commands**:
```bash
# Find object/shape handling in subtype
grep -n "TypeKey::Object\|object_shape\|PropertyInfo" wasm/src/solver/subtype.rs | head -50

# Find property comparison logic
grep -n "has_property\|check_property\|properties" wasm/src/solver/subtype.rs | head -50
```

### Area 4: Union and Intersection Types
**Hypothesis**: Union/intersection assignability may be using OR logic when it should use AND, or vice versa.

**Tasks**:
1. Review union type assignability rules
2. Review intersection type assignability rules
3. Verify correct set-theoretic semantics

**TypeScript Rules**:
- `A | B` is assignable to `C` if BOTH `A <: C` AND `B <: C`
- `C` is assignable to `A | B` if `C <: A` OR `C <: B`
- `A & B` is assignable to `C` if `A <: C` OR `B <: C`
- `C` is assignable to `A & B` if BOTH `C <: A` AND `C <: B`

**Search Commands**:
```bash
# Find union handling
grep -n "TypeKey::Union" wasm/src/solver/subtype.rs | head -30

# Find intersection handling
grep -n "TypeKey::Intersection" wasm/src/solver/subtype.rs | head -30
```

## Implementation Strategy

### Phase 1: Analysis (Day 1)
1. Read through subtype.rs thoroughly (it's large - use offset/limit with Read tool)
2. Document all places where `true` is returned
3. Identify suspicious default behaviors
4. Create a list of potential fixes

### Phase 2: Conservative Fixes (Day 1-2)
Start with the safest fixes:
1. Change catch-all patterns from `=> true` to `=> false`
2. Add TODO comments for complex cases that need more analysis
3. Test after each change

### Phase 3: Structural Improvements (Day 2)
1. Improve object property checking
2. Fix union/intersection logic if needed
3. Add more explicit type pair handling

### Phase 4: Testing (Day 2)
1. Run existing unit tests
2. Run conformance tests
3. Analyze which "Extra TS2322" errors are legitimate
4. Document expected behavior changes

## Testing Strategy

### Build and Test
```bash
# Build WASM
cd /Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm
wasm-pack build --target web --out-dir pkg

# Run solver tests
cargo test solver::subtype
cargo test solver::compat

# Run type checker tests
cargo test thin_checker_tests | grep -i ts2322

# Run conformance tests
cd differential-test
bash run-conformance.sh --max=500 --workers=4
```

### Create Test Cases
```typescript
// Test 1: Object structural mismatch
const a: { x: number } = { x: "string" }; // Should: TS2322

// Test 2: Missing property
const b: { x: number, y: number } = { x: 1 }; // Should: TS2322

// Test 3: Excess property (in object literal)
const c: { x: number } = { x: 1, y: 2 }; // Should: TS2322 in strict mode

// Test 4: Union assignability
const d: string | number = true; // Should: TS2322

// Test 5: Intersection assignability
type AB = { a: number } & { b: string };
const e: AB = { a: 1 }; // Should: TS2322 (missing b)
```

## Expected Changes

### Metrics to Track
1. **Before Fix**:
   - Count of "Missing TS2322" errors in conformance tests
   - Count of "Extra TS2322" errors

2. **After Fix**:
   - Reduced "Missing TS2322" (our goal)
   - May increase "Extra TS2322" (acceptable if legitimate)
   - No regressions in other error codes

### Acceptable Outcomes
- 10-30% reduction in missing TS2322 errors
- Possible increase in extra TS2322 if they're legitimate strict checks
- Document any behavioral changes from TypeScript

## Success Criteria
1. Reduction in TS2322 "Missing Errors" count
2. Solver is provably stricter (fails on unclear relationships)
3. No regressions in existing passing tests
4. Clear documentation of behavioral changes
5. Clean commit message

## Branch
Create branch: `worker-2-ts2322-assignability`

## Notes
- This is a core type system fix - be methodical
- Test incrementally - don't make all changes at once
- Document reasoning for each change
- Coordinate with Worker 3 (TS2304 fix will affect this)
- ERROR types already handled correctly per compat.rs:156-161
