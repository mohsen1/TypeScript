# Phase 1 Analysis: TypeId::ANY Usage

## Summary
Found ~150 occurrences of `TypeId::ANY` in the codebase.

## Categories

### ✅ Category A: Intentional ANY (Keep)
1. **Test files** (integration_tests.rs, thin_checker_tests.rs, types_tests.rs)
   - Used for testing type system behavior
   - **Action:** Keep all test uses

2. **Keyword handling** - When user explicitly types `any`
   - Line 786: `k if k == SyntaxKind::AnyKeyword as u16 => TypeId::ANY`
   - Line 982: `"any" => return TypeId::ANY`
   - **Action:** Keep - user's explicit intent

3. **Type guards and comparisons**
   - Lines with `if type == TypeId::ANY`
   - **Action:** Keep - these are checks, not defaults

### ⚠️ Category B: Optimistic Defaults (Change to UNKNOWN)
**Priority locations that hide bugs:**

#### B1. Function Returns (Task 2)
- Line 215: `Some(TypeId::ANY)` - unknown context
- Line 3276, 3307: `(TypeId::ANY, None)` - tuple returns
- Line 3941, 3945, 3989, 3993: `return (TypeId::ANY, None)` - explicit returns
- Line 6278, 6310, 6379, 6407, 6435, 6471, 6499, 6509: Function-related
- **Impact:** Hides missing return type annotations

#### B2. Variable/Expression Defaults (Task 3)
- Line 4254: `.unwrap_or(TypeId::ANY)` - variable type
- Line 4925: `.unwrap_or(TypeId::ANY)` - another unwrap
- Line 5303: `_ if self.is_known_global_value_name(name) => TypeId::ANY`
- Line 6966, 6967: `type_stack.pop().unwrap_or(TypeId::ANY)`
- Line 7033: `type_stack.pop().unwrap_or(TypeId::ANY)`
- **Impact:** Suppresses type errors in expressions

#### B3. Call/Property Access (Task 4)
- Line 7111, 7121: `if callee_type == TypeId::ANY { return TypeId::ANY }`
- Line 7143, 7144: Similar patterns
- Line 7478, 7479: Constructor returns
- Line 8387, 8388: Property access
- Line 8447, 8448: More property access
- Line 8715, 8716, 8722, 8748: Element access
- **Impact:** Invalid operations silently succeed

#### B4. Binary Operations
- Line 6896: `BinaryOpResult::TypeError { .. } => TypeId::ANY`
- Line 6929, 6939: `type_stack.push(TypeId::ANY)`
- Line 7020, 7028: More binary op handling
- **Impact:** Type errors in expressions hidden

#### B5. New Expressions
- Line 7944, 7945: `if constructor_type == TypeId::ANY { return TypeId::ANY }`
- **Impact:** Constructor type errors suppressed

#### B6. Type Resolution Failures
- Line 8130, 9185, 10418, 10496: Various resolution failures
- Line 9764: `(TypeId::ANY, None)` - resolution failure
- Line 11083, 11085: Fallback defaults
- **Impact:** Resolution failures don't emit errors

#### B7. Spread/Rest Operators
- Line 2263 in solver/operations.rs: `let rest_array = self.interner.array(TypeId::ANY)`
- **Impact:** Spread/rest type errors hidden

#### B8. Built-in Method Signatures
- Lines 2414-2568 in solver/operations.rs: Multiple uses for built-in methods
- **Impact:** Built-in methods have permissive types

## Priority Ranking

### P0 (Critical - Do First):
1. **Function return defaults** (B1) - Lines 3276, 3307, 3941, 3945, 3989, 3993
2. **Expression type resolution** (B2) - Lines 4254, 4925, 5303

### P1 (High Impact):
3. **Call expression handling** (B3) - Lines 7111, 7121, 7143, 7144
4. **Binary operation errors** (B4) - Lines 6896, 6929, 6939, 7020, 7028

### P2 (Medium Impact):
5. **Property access** (B3 subset) - Lines 8387, 8388, 8447, 8448, 8715, 8716, 8722
6. **New expressions** (B5) - Lines 7944, 7945

### P3 (Lower Priority):
7. **Built-in method signatures** (B8) - solver/operations.rs lines 2414-2568
8. **Spread operators** (B7) - Line 2263

## Recommended Changes

### Change to `TypeId::UNKNOWN`:
- All "optimistic defaults" where type can't be determined
- This will make type checking stricter

### Keep as `TypeId::ANY`:
- When user explicitly types `any` keyword
- Test files
- Type guards/comparisons

## Next Steps

1. **Start with B1** - Function return defaults
2. **Move to B2** - Variable/Expression defaults  
3. **Then B3/B4** - Call/Binary operations
4. **Test after each change** - Expect "extra errors" to increase (this is good!)

## Expected Impact

- **Missing errors:** Will decrease significantly
- **Extra errors:** Will increase initially (exposes hidden bugs)
- **Conformance:** Exact match may decrease, but correctness increases
