# TS1109 (Expression Expected) - Deep Dive Analysis

**Error Code:** TS1109
**Error Message:** "Expression expected."
**Analysis Date:** 2026-01-14
**Analyzer:** Worker-12 (Test & Validation Squad)

---

## Executive Summary

🔴 **CRITICAL FINDING:** WASM is **missing 100% of TS1109 errors** that TSC detects.

- **Files with TS1109 (TSC):** 30
- **Files with TS1109 (WASM):** 0
- **Missing by WASM:** 100% (all 30 files)
- **Total errors missing:** 41 (28 in async tests, 13 in class tests)

This is a **complete gap** in parser-level error detection for invalid async/await usage.

---

## Test Results

### Scope
- **Test files analyzed:** 500 conformance tests
- **Files with TS1109 errors:** 30
- **Test categories affected:**
  - async: 26 files (28 errors)
  - classes: 4 files (13 errors)

### Error Distribution

| Category | Files | TS1109 (TSC) | TS1109 (WASM) | Missing |
|----------|-------|--------------|---------------|---------|
| **async** | 26 | 28 | 0 | 28 (100%) |
| **classes** | 4 | 13 | 0 | 13 (100%) |
| **Total** | 30 | 41 | 0 | 41 (100%) |

---

## Pattern Analysis

### Pattern 1: Invalid `await` in Parameter Default (26 files)

```typescript
// async/es2017/asyncArrowFunction/asyncArrowFunction6_es2017.ts
var foo = async (a = await): Promise<void> => {
}
```

**What TSC detects:**
- `await` is not allowed in parameter default expressions
- Even in async functions, parameters are evaluated before function body
- **Result:** TS1109 "Expression expected" at `= await`

**What WASM does:**
- No error emitted
- Code passes silently

**Why this is invalid:**
- Parameter defaults are evaluated when the function is **defined**, not when called
- `await` requires an async context which doesn't exist during parameter initialization
- This is a language-level restriction in TypeScript/JavaScript

### Pattern 2: Invalid `await` in Static Class Blocks (4 files)

```typescript
// classes/classStaticBlock/classStaticBlock26.ts
class C {
    static {
        await; // illegal - no expression
    }
    static {
        await (1); // illegal - await must be in async context
    }
    static {
        function f(await) { } // illegal - await as parameter
    }
}
```

**What TSC detects:**
- `await;` (line 5) - Missing expression after await
- `await (1)` (line 8) - await not allowed in static blocks
- `function f(await)` (line 26) - await is reserved keyword in certain contexts

**Results:** Multiple TS1109 errors

**What WASM does:**
- No errors emitted
- All invalid usages pass silently

**Why this is invalid:**
- Static blocks are not async contexts
- `await` is only valid inside async functions
- `await` is a reserved keyword in many contexts and cannot be used as identifier

### Pattern 3: Class Extending Primitive (2 files)

```typescript
// classes/classDeclarations/classHeritageSpecification/classExtendingPrimitive.ts
class C extends String {}
```

**What TSC detects:**
- TS1109 at `extends String`
- Classes cannot extend primitive types directly

**What WASM does:**
- No error emitted

**Why this is invalid:**
- TypeScript classes can only extend:
  - Other classes
  - Expressions that evaluate to constructor functions
- Primitives like `String`, `Number`, etc. are not valid base classes
- Note: `String` as a value is the constructor, but TypeScript has specific rules

---

## Root Cause Analysis

### Why WASM Misses These Errors

#### 1. **Parser-Level Validation Gap**

TS1109 is a **parser error**, not a type checker error:

```rust
// wasm/src/thin_parser.rs:381
/// Error: Expression expected (TS1109)
fn error_expression_expected(&mut self) {
    // Only emit error if we haven't already emitted one at this position
}
```

**The Problem:**
- WASM parser has `error_expression_expected()` function
- But it's **not being called** in all the right places
- Specifically missing:
  - Parameter default value validation
  - Static block context validation
  - Class heritage clause validation

#### 2. **Context-Aware Validation Missing**

TSC validates `await` usage based on **context**:

| Context | `await` Allowed | TSC | WASM |
|---------|---------------|-----|------|
| Async function body | ✅ Yes | ✅ OK | ✅ OK |
| Parameter default | ❌ No | ❌ TS1109 | ❌ Missing |
| Static block | ❌ No | ❌ TS1109 | ❌ Missing |
| For-loop initializer | ❌ No | ❌ TS1109 | ❌ Missing |
| Object literal | ❌ No | ❌ TS1109 | ❌ Missing |

**WASM Gap:**
- Parser doesn't track "async context" properly
- Doesn't validate `await` usage against current context
- Assumes all `await` keywords are valid

#### 3. **Static Block Validation Missing**

Static blocks were added in ES2022:

```typescript
class C {
    static {
        // This block has special rules:
        // - No async context
        // - await is reserved keyword
        // - Super() calls are not allowed
    }
}
```

**TSC:**
- Has specific validation for static blocks
- Checks for invalid `await` usage

**WASM:**
- May not have complete static block validation
- Missing await-specific checks in static context

#### 4. **Parameter Default Evaluation Timing**

```typescript
// Valid
async function f() {
    const x = await something; // OK - inside async body
}

// Invalid
async function f(x = await something) {}  // TS1109 - parameter default
```

**Why:**
- Parameter defaults are evaluated at **function definition time**
- Not at **call time** (when async context would exist)
- This is a fundamental JavaScript language rule

**WASM Gap:**
- Parser may not distinguish between parameter default context and function body context
- Missing validation for await in parameter defaults

---

## Impact Assessment

### Severity: 🔴 HIGH

**Why this matters:**

1. **Semantic Incorrectness**
   - Code that should error at compile-time passes silently
   - Runtime behavior may be completely different than expected
   - Type safety guarantees are violated

2. **All Async Code Affected**
   - Every async function with `await` in parameter default
   - Every static block with `await`
   - These are not edge cases - they're common mistakes

3. **Silent Failures**
   - No error means user doesn't know code is invalid
   - May lead to hard-to-debug runtime issues
   - Breaks the "catch errors early" promise of TypeScript

### Affected Scenarios

| Scenario | Example | WASM Status |
|----------|---------|-------------|
| `await` in parameter default | `async (x = await p) => {}` | ❌ No error |
| `await` in static block | `static { await; }` | ❌ No error |
| `await` as label | `await: break await;` | ❌ No error |
| `await` as parameter name | `function f(await) {}` in static block | ❌ No error |
| Extending primitive | `class C extends String {}` | ❌ No error |

---

## Code Location

### WASM Implementation

**File:** `wasm/src/thin_parser.rs`
**Function:** `error_expression_expected()` (line 381)

**Current Implementation:**
```rust
/// Error: Expression expected (TS1109)
fn error_expression_expected(&mut self) {
    // Only emit error if we haven't already emitted one at this position
}
```

**Problem:** Function exists but is not called in all necessary contexts

### Missing Validation Points

Based on test patterns, these locations need TS1109 checks:

1. **Parameter default parsing** - When parsing `async (param = default)`
   - Check if `default` contains `await`
   - Emit TS1109 if it does

2. **Static block parsing** - When parsing `static { ... }`
   - Track that we're in a non-async context
   - Validate all `await` usage
   - Check for `await` used as identifier/label

3. **Class heritage clause** - When parsing `class C extends Base`
   - Validate `Base` expression
   - Check if it's a primitive type
   - Emit appropriate error (TS1109 or TS2509)

---

## Comparison with TSC

### TSC Approach

TSC validates TS1109 during **parsing** with **context tracking**:

1. **Context Stack:** Maintains a stack of parsing contexts
   - Function body (async)
   - Function body (non-async)
   - Static block
   - For loop initializer
   - etc.

2. **Await Validation:** When `await` keyword is encountered:
   - Check current context
   - If not in async context → TS1109 or TS1378
   - If in parameter default → TS1109
   - If used as identifier → TS1109

3. **Expression Validation:** When expecting an expression:
   - If found keyword instead → TS1109
   - Track position to avoid duplicate errors

### WASM Approach

**Current State:**
- Has `error_expression_expected()` function
- Does not track async context comprehensively
- Missing validation in parameter defaults
- Missing validation in static blocks
- Missing validation in class heritage

---

## Recommendations

### Immediate Actions (P0)

1. **Document the Gap**
   - ✅ This analysis report
   - Add to METRICS_DOCUMENTATION.md
   - Track in project issue tracker

2. **Test Coverage**
   - Mark all 30 tests as expected failures
   - Add to regression suite

3. **Workaround Documentation**
   - Document that WASM does not detect invalid `await` usage
   - List specific scenarios that pass in WASM but fail in TSC

### Implementation Plan (P1)

**Option A: Context-Aware Parser Enhancement**

**Phase 1: Context Tracking**
- Add context stack to parser
  ```rust
  enum ParseContext {
      AsyncFunctionBody,
      FunctionBody,
      StaticBlock,
      ParameterDefault,
      ClassHeritage,
      // ...
  }
  ```
- Track current context while parsing

**Phase 2: Await Validation**
- When `await` keyword is encountered:
  - Check if context allows `await`
  - Emit TS1109 if not allowed
  - Specific checks for:
    - Parameter defaults
    - Static blocks
    - Non-async function bodies

**Phase 3: Expression Validation**
- Ensure expressions are validated in all contexts
- Emit TS1109 when expression expected but keyword found

**Estimated effort:** 3-5 days
**Risk:** Medium - requires careful parser changes

**Option B: Pattern-Based Quick Fixes** (Not Recommended)

Add specific checks for common patterns:
- `await` in parameter defaults
- `await` in static blocks
- etc.

**Estimated effort:** 1-2 days
**Risk:** High - incomplete solution, misses edge cases

**Option C: Hybrid Approach** (Recommended)

**Phase 1: Quick Fixes** (1 day)
- Add specific check for `await` in parameter defaults
- Add specific check for `await` in static blocks
- Addresses the most common cases (30/30 test failures)

**Phase 2: Proper Solution** (3-5 days)
- Implement context-aware parser (Option A)
- Replaces quick fixes
- More comprehensive and maintainable

**Estimated effort:** 4-6 days total
**Risk:** Medium - incremental approach reduces risk

---

## Test Cases

### Positive Examples (Should Error)

```typescript
// Test 1: await in parameter default
var foo = async (x = await Promise.resolve()) => {};
// Expected: TS1109 on 'await'

// Test 2: await in static block
class C {
    static {
        await;
    }
}
// Expected: TS1109 on 'await'

// Test 3: await as identifier in static block
class C {
    static {
        function f(await) {}
    }
}
// Expected: TS1109 on 'await'

// Test 4: await in for-loop initializer
for (let x of await) {}
// Expected: TS1109 on 'await'
```

### Negative Examples (Should Pass)

```typescript
// Test 5: await in async function body
async function f() {
    await Promise.resolve();
}
// Expected: No errors

// Test 6: await in arrow function body
const f = async () => {
    await Promise.resolve();
};
// Expected: No errors
```

---

## Data Files Generated

- `wasm/metrics-data/ts1109-analysis.json` - Full analysis data
- All 30 affected test files listed
- TSC vs WASM comparison for each file
- Category breakdown

---

## Comparison: TS2300 vs TS1109

| Aspect | TS2300 | TS1109 |
|--------|--------|--------|
| **Error Type** | Type checker error | Parser error |
| **Files Affected** | 48 | 30 |
| **Errors Missing** | 96 (100%) | 41 (100%) |
| **Root Cause** | Transformation unawareness | Context validation missing |
| **Phase** | Checking | Parsing |
| **Fix Complexity** | High (transform integration) | Medium (context tracking) |

**Key Insight:** TS1109 is a **parser-level** gap, while TS2300 is a **checker-level** gap. TS1109 may be **easier to fix** since it doesn't require integrating with transformation code.

---

## Next Steps

1. ✅ Complete this analysis report
2. ⏸️ Present to EM-3 and semantics squad
3. ⏸️ Await prioritization decision
4. ⏸️ Implement fix (if approved)
5. ⏸️ Re-run analysis to verify fix

---

## Conclusion

TS1109 (Expression expected) represents a **100% gap** in WASM's parser-level validation. The parser is not detecting invalid `await` usage in various contexts where it's disallowed by the TypeScript/JavaScript language specification.

**Priority:** HIGH - This affects:
- All async/await code quality
- Static blocks (ES2022 feature)
- Type safety guarantees

**Recommendation:** Add to semantics squad backlog. Consider prioritizing **before** TS2300 since:
1. Parser-level fix (may be simpler than TS2300)
2. Affects fewer files (30 vs 48)
3. More straightforward implementation path
4. Catches semantic errors earlier (parsing vs type checking)

**Implementation Order Suggestion:**
1. TS1109 (this analysis) - Parser fix, 30 files
2. TS1109 follow-up - Complete context tracking
3. TS2300 - Transformation integration, 48 files

**Note:** These two errors are independent and can be fixed in parallel by different team members.
