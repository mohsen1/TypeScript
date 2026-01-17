# Worker 4 Task: Fix TS2507/TS2348 - Constructor & Invocation Errors (Tier 2)

## Assignment
Fix two related error categories:
1. **TS2507**: Non-constructor values used in extends clauses not being fully checked
2. **TS2348**: "Cannot invoke expression" being over-reported for callable types

## Problem 1: TS2507 - Extends Clause Validation

### Current State
From code analysis in `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/thin_checker.rs`:
- Line 16616: Comment mentions checking extends clauses for TS2507
- Line 16636: Comment about checking for literals in extends clauses

### Problem
Classes can extend non-constructor values without proper error reporting:
```typescript
// Should emit TS2507: Type 'number' is not a constructor function type
class A extends 42 {}

// Should emit TS2507
const obj = { x: 1 };
class B extends obj {}

// Should be OK
class C extends BaseClass {}
```

### Investigation Areas

**Search for extends clause checking**:
```bash
# Find class extends checking code
grep -n "check.*heritage\|check.*extends\|ExtendsClause" wasm/src/thin_checker.rs | head -30

# Find TS2507 references
grep -n "TS2507\|NOT_A_CONSTRUCTOR" wasm/src/

# Find heritage clause node handling
grep -n "HeritageClause\|HERITAGE_CLAUSE" wasm/src/thin_checker.rs | head -20
```

**Key locations from earlier grep**:
- Line 16616: Extends clause checking
- Line 16734: Heritage clause checking for unresolved names (TS2304)
- Line 18738: More heritage clause checking

### Implementation for TS2507

**Step 1: Find heritage clause type checking**
Read the code around lines 16616, 16734, 18738 to understand:
1. How heritage clauses are currently checked
2. Where the extends expression type is resolved
3. Whether constructor-ness is verified

**Step 2: Add constructor validation**
When checking an extends clause:
```rust
fn check_heritage_clause(&mut self, heritage_idx: NodeIndex) {
    // Get the extends expression
    let extends_expr = self.get_heritage_expression(heritage_idx);

    // Get its type
    let extends_type = self.get_type_of_node(extends_expr);

    // NEW: Check if it's constructable
    if !self.is_constructable_type(extends_type) {
        self.error_ts2507(extends_expr, extends_type);
        return;
    }

    // Continue with normal heritage checking...
}
```

**Step 3: Implement is_constructable_type()**
Find or create this function:
```rust
fn is_constructable_type(&self, type_id: TypeId) -> bool {
    // Check if the type has construct signatures
    match self.ctx.types.lookup(type_id) {
        Some(TypeKey::Callable(shape_id)) => {
            let shape = self.ctx.types.callable_shape(shape_id);
            !shape.construct_signatures.is_empty()
        }
        Some(TypeKey::Ref(symbol_ref)) => {
            // Resolve the ref and check recursively
            if let Some(resolved) = self.resolve_symbol_type(symbol_ref) {
                self.is_constructable_type(resolved)
            } else {
                false
            }
        }
        // Class types are constructable
        Some(TypeKey::Class(_)) => true,

        // Primitives and literals are NOT constructable
        Some(TypeKey::Literal(_)) => false,
        _ if type_id == TypeId::NUMBER => false,
        _ if type_id == TypeId::STRING => false,
        _ if type_id == TypeId::BOOLEAN => false,

        // Unknown/Any might be constructable (permissive)
        _ if type_id == TypeId::ANY => true,

        _ => false,
    }
}
```

## Problem 2: TS2348 - Call Expression Checking

### Current State
From code analysis:
- Line 7440: Comment about class constructor called without 'new' (TS2348)
- Line 14239: `Report TS2348: "Cannot invoke an expression whose type lacks a call signature"`
- Lines 23186-23263: Test cases for TS2348

### Problem
The error is being over-reported - callable types are not being properly detected:
```typescript
// Should be OK - function is callable
const fn = () => 42;
fn(); // Currently: TS2348, Should: OK

// Should emit TS2348 - class constructor without new
class C {}
C(); // Should: TS2348

// Should be OK - call signature
type Callable = { (): number };
const obj: Callable = () => 42;
obj(); // Should: OK
```

### Investigation Areas

**Find call expression checking**:
```bash
# Find call expression checking
grep -n "check_call_expression\|get_type_of_call" wasm/src/thin_checker.rs | head -20

# Find TS2348 error emission
grep -n "TS2348\|CANNOT_INVOKE\|lacks a call signature" wasm/src/thin_checker.rs | head -20

# Find is_callable checks
grep -n "is_callable" wasm/src/thin_checker.rs | head -20
```

### Implementation for TS2348

**Step 1: Find call expression type checking**
Read the code around line 7440 and 14239:
1. Understand how call expressions are checked
2. Find where TS2348 is emitted
3. Identify the callable type check

**Step 2: Fix is_callable() logic**
The function likely exists but may be incomplete. Ensure it handles:
```rust
fn is_callable_type(&self, type_id: TypeId) -> bool {
    match self.ctx.types.lookup(type_id) {
        // Function types are callable
        Some(TypeKey::Callable(shape_id)) => {
            let shape = self.ctx.types.callable_shape(shape_id);
            !shape.call_signatures.is_empty()
        }

        // Ref types - resolve and check
        Some(TypeKey::Ref(symbol_ref)) => {
            if let Some(resolved) = self.resolve_symbol_type(symbol_ref) {
                self.is_callable_type(resolved)
            } else {
                false
            }
        }

        // Union: callable if ANY member is callable
        Some(TypeKey::Union(members)) => {
            let members = self.ctx.types.type_list(members);
            members.iter().any(|m| self.is_callable_type(*m))
        }

        // Intersection: callable if ANY member is callable
        Some(TypeKey::Intersection(members)) => {
            let members = self.ctx.types.type_list(members);
            members.iter().any(|m| self.is_callable_type(*m))
        }

        // Any is callable (permissive)
        _ if type_id == TypeId::ANY => true,

        _ => false,
    }
}
```

**Step 3: Update call expression checking**
```rust
fn check_call_expression(&mut self, call_idx: NodeIndex) -> TypeId {
    let call_expr = self.ctx.arena.get_call_expression(call_idx)?;

    // Get the expression being called
    let callee_type = self.get_type_of_node(call_expr.expression);

    // Check if it's callable
    if !self.is_callable_type(callee_type) {
        // SPECIAL CASE: Class constructor without 'new'
        if self.is_constructor_type(callee_type) {
            self.error_ts2348_constructor(call_expr.expression);
        } else {
            self.error_ts2348(call_expr.expression, callee_type);
        }
        return TypeId::ERROR;
    }

    // Continue with normal call checking...
}
```

## Testing Strategy

### Test Cases for TS2507
```typescript
// Test 1: Literal in extends - should emit TS2507
class A extends 42 {}
class B extends "string" {}
class C extends true {}

// Test 2: Object in extends - should emit TS2507
const obj = { x: 1 };
class D extends obj {}

// Test 3: Valid extends - should be OK
class Base {}
class E extends Base {}

// Test 4: Constructor function - should be OK
function F() {}
class G extends F {}
```

### Test Cases for TS2348
```typescript
// Test 1: Function call - should be OK
const fn = () => 42;
fn();

// Test 2: Class without new - should emit TS2348
class C {}
C();

// Test 3: Object with call signature - should be OK
type Callable = { (): number };
const obj: Callable = () => 42;
obj();

// Test 4: Non-callable - should emit TS2348
const num = 42;
num();
```

### Running Tests
```bash
# Build WASM
cd /Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm
wasm-pack build --target web --out-dir pkg

# Run unit tests
cargo test thin_checker_tests | grep -i "ts2507\|ts2348"

# Run conformance tests
cd differential-test
bash run-conformance.sh --max=500 --workers=4

# Check error counts
grep -c "TS2507" conformance_results.txt
grep -c "TS2348" conformance_results.txt
```

## File Locations Reference

### Primary File
- `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/thin_checker.rs`

### Key Areas
1. **Heritage clause checking**: Lines 16616-16750, 18738+
2. **Call expression checking**: Line 7440, 14239
3. **Test cases**: Lines 23186-23263 (TS2348 tests)

### Functions to Find/Modify
- `check_heritage_clause()` or similar
- `check_class_declaration()` - handles extends
- `check_call_expression()` or `get_type_of_call_expression()`
- `is_constructable_type()` / `is_constructor_type()` - line 14095 reference
- `is_callable_type()` / `is_callable()`

## Success Criteria

### TS2507
1. Reduction in "Missing TS2507" errors
2. Literals in extends clauses properly rejected
3. Non-constructor objects in extends rejected
4. Valid extends clauses still work

### TS2348
1. Reduction in "Extra TS2348" errors
2. Function types properly recognized as callable
3. Objects with call signatures recognized as callable
4. Class constructors without 'new' still emit TS2348

## Branch
Create branch: `worker-4-ts2507-ts2348-invocation`

## Notes
- These two issues are related (both about callable/constructable checking)
- Fix TS2507 first (simpler), then TS2348
- Test incrementally after each fix
- Coordinate with Worker 1 (this type checking) if needed
- May need to understand the TypeKey enum structure in solver/types
