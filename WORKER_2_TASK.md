# Worker 2 Task - EM Team 1 Status Report

## Current Task

**EM Team 1: Check for assigned tasks**

You are the Engineering Manager for Team 1.
Check TEAM_STRUCTURE.md for your team's assigned tasks.
Distribute work to your workers and ensure quality delivery.

## EM Status Report

### Team Composition
- **Engineering Manager:** Worker 2
- **Team Members:** Worker 3, Worker 4
- **Priority Tier:** Tier 0 (Quality & Stability Foundations)

### Assigned Tasks Overview

Team 1 is responsible for **Tier 0: Quality & Stability Foundations** issues from PROJECT_DIRECTION.md:

| Task | Owner | Status | Priority |
|------|-------|--------|----------|
| Application type expansion | Worker 3 | Not Started | High |
| Readonly types | Worker 4 | Not Started | High |

---

## Task 1: Application Type Expansion (Worker 3)

### Description
`TypeKey::Application` is not expanded, leading to incorrect diagnostics/assignability.

### Current State
**STATUS:** Implementation exists but needs validation and testing

**Existing Implementation:**
- `thin_checker.rs:11918` - `evaluate_application_type()` - Main expansion logic
- `thin_checker.rs:11948` - `evaluate_application_type_inner()` - Core expansion algorithm
- `solver/evaluate.rs:342` - `evaluate_application()` - Type evaluator expansion
- `solver/subtype.rs:905` - `try_expand_application()` - Subtype checking expansion

**How It Works:**
1. Application types (e.g., `Box<string>`) are represented as `TypeKey::Application(Ref(Box), [string])`
2. When encountered, the evaluator:
   - Resolves the base Ref to get the type body
   - Retrieves type parameters for the base symbol
   - Creates a TypeSubstitution mapping params to args
   - Instantiates the body with the substitution
   - Recursively evaluates the result

**Key Files:**
- `wasm/src/solver/evaluate.rs` - Type evaluation logic
- `wasm/src/solver/intern.rs` - Type interning
- `wasm/src/solver/instantiate.rs` - Type parameter substitution
- `wasm/src/solver/subtype.rs` - Subtype checking with expansion
- `wasm/src/thin_checker.rs` - Type checking integration

**Test Coverage:**
- Tests exist in `wasm/src/solver/evaluate_tests.rs` starting at line 15121
- Tests document expected behavior but are currently written as documentation

**Known Issues:**
- Application types may pass through unchanged in some code paths
- Nested applications need recursive expansion
- Self-referential types need cycle detection (partially implemented)

**Next Steps for Worker 3:**
1. Run existing tests to identify failures
2. Add integration tests for common patterns (type aliases, generics, nested apps)
3. Fix any expansion gaps in evaluate_application_type()
4. Run conformance tests to validate fix

---

## Task 2: Readonly Types (Worker 4)

### Description
`readonly` arrays/tuples are currently treated as mutable.

### Current State
**STATUS:** Placeholder implementation - needs full semantics

**Existing Implementation:**
- `solver/intern.rs:1036` - `readonly_array()` - Returns same as array()
- `solver/intern.rs:1048` - `readonly_tuple()` - Returns same as tuple()
- `thin_checker.rs:1035` - Parser creates `ReadonlyType` wrapper
- TypeKey::ReadonlyType exists in type system

**What's Missing:**
1. **Distinct type representation:** Readonly arrays/tuples need to be distinguishable from mutable ones
2. **Assignability checks:** Mutable should be assignable to readonly, but not vice versa
3. **Write checks:** Properties of readonly objects should reject assignment
4. **Index signatures:** Readonly index signatures should prevent writes

**Required Changes:**

1. **Type Representation** (`solver/intern.rs`):
   ```rust
   // Add readonly flag to Array/Tuple or create separate variants
   pub fn readonly_array(&self, element: TypeId) -> TypeId {
       // Create distinct type with readonly marker
   }

   pub fn readonly_tuple(&self, elements: Vec<TupleElement>) -> TypeId {
       // Create distinct type with readonly marker
   }
   ```

2. **Subtype Checking** (`solver/subtype.rs`):
   - Add rules for readonly assignability
   - `readonly T` is assignable to `T` (covariant)
   - `T` is NOT assignable to `readonly T` (mutability constraint)

3. **Assignment Checking** (`thin_checker.rs`):
   - Check write operations on readonly collections
   - Emit error TS2540 ("Cannot assign to 'X' because it is read-only")

**Key Files:**
- `wasm/src/solver/intern.rs` - Type construction
- `wasm/src/solver/subtype.rs` - Assignability rules
- `wasm/src/thin_checker.rs` - Assignment validation
- `wasm/src/thin_parser.rs` - Parser already creates ReadonlyType nodes

**Test Coverage:**
- Parser tests exist in `thin_parser_tests.rs` (test_thin_parser_readonly_array/tuple)

**Next Steps for Worker 4:**
1. Implement distinct readonly type representation
2. Add assignability rules for readonly types
3. Add write checking to detect mutation of readonly values
4. Add tests for readonly semantics

---

## Branch Status

### Current Changes
- Fixed syntax error in `thin_checker.rs` (removed extra closing brace)
- Code compiles successfully with only warnings (unused imports)

### Validation
- ✅ Code compiles: `cargo check` passes
- ✅ No new errors introduced
- ⚠️  Some unused import warnings (non-blocking)

---

## Requirements

- Complete the task as described

## Files to Modify

- Determine which files need modification based on the task

## Acceptance Criteria

- [x] Task reviewed and team status documented
- [x] Code compiles/builds without errors
- [ ] Tests pass (if applicable)

## Context

- **Branch:** worker-2
- **Base Branch:** rust
- **Mode:** hierarchy
- **Team:** em-team-1
- **Task ID:** 9fb5601c-9c7d-4b38-adf6-7510c9c421b1
- **Priority:** normal

## Instructions

1. Read and understand the task requirements above
2. Make changes incrementally with clear, descriptive commit messages
3. Test your changes before marking the task complete
4. Do not modify files outside your task scope unless necessary
5. When done, commit all changes and push to your branch

Your changes will be automatically merged after review.

---
*Generated by CCO at 2026-01-16T14:02:22.713Z*
