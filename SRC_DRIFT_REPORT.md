# TypeScript src/ Directory Drift Analysis

**Analysis Date:** January 14, 2026  
**Upstream Reference:** microsoft/TypeScript main branch  
**Current Branch:** rust  

## Summary

The local repository has **significant drift** from upstream TypeScript in the `src/` directory, with **37 commits** affecting core TypeScript compiler files. This violates the WASM migration principles that mandate keeping TypeScript source files read-only.

## Drift Statistics

- **Total commits ahead:** 50,308+ (across all branches)
- **Commits affecting src/:** 37 commits
- **Modified files:** 3 core compiler files
  - `src/compiler/checker.ts` (+62 lines, -5 lines)
  - `src/compiler/parser.ts` (+20 lines, -3 lines) 
  - `src/compiler/diagnosticMessages.json` (+16 lines, 0 deletions)

## Detailed Analysis

### 1. src/compiler/checker.ts (Most Problematic)

**Changes:** 62 insertions, 5 deletions  
**Nature:** Extensive modifications to type checking logic

**Problematic commits:**
- `082c81bcdf` - Complete: Task 2 - Enhance TS2322 Error Messages
- `d0d153f202` - Complete: Task 3 - Add Type Tracing to Errors  
- `d71f09b940` - Implement stricter subtype checking for type assignability
- `9abf900d7b` - Replace anyType fallback with unknownType (REVERTED)
- Multiple upstream patches mixed with local changes

**Added functionality:**
- `createPropertyErrorMessage()` helper function
- `addTypeOriginInfo()` for type tracing
- Enhanced error reporting with related information
- Modified property-aware error messages

### 2. src/compiler/parser.ts

**Changes:** 20 insertions, 3 deletions  
**Nature:** TS1005 false positive fixes and WASM integration hooks

**Key changes:**
- Object literal comma handling with line breaks (Pattern 6)
- Modified `tryParseSemicolon()` behavior for ASI compliance
- WASM parser integration remnants

### 3. src/compiler/diagnosticMessages.json

**Changes:** 16 insertions (4 new diagnostic messages)  
**Added messages:**
- 9512: "The error is in property '{0}'"
- 9513: "Type '{0}' was inferred from expression at this location"  
- 9514: "Type '{0}' was inferred from argument '{1}' at position {2}"
- 9515: "Type '{0}' was inferred from return statement"

## Violation Assessment

### 🚨 Critical Violations

1. **Direct TypeScript compiler modifications** - Extensive changes to `checker.ts`
2. **Mixed upstream and local commits** - Makes clean separation impossible
3. **Feature additions** - New diagnostic messages and error handling logic
4. **WASM integration in core files** - Parser has WASM-related changes

### Impact on Migration Goals

- **Blocks clean upstream syncing** - Can't fast-forward merge from microsoft/TypeScript
- **Creates maintenance burden** - Need to maintain parallel implementation 
- **Violates architecture principles** - WASM migration should be isolated to `wasm/`
- **Complicates testing** - Changes affect baseline TypeScript behavior

## Recommended Actions

### 1. Immediate (High Priority)

- **REVERT all src/ changes** to restore clean upstream state
- **Extract functionality** to `wasm/` directory where appropriate  
- **Document any legitimate upstream patches** that need preservation

### 2. Architecture Cleanup

- **Move error enhancements** to WASM-side implementation
- **Use proper integration hooks** (<10 lines total) instead of deep modifications
- **Implement feature flags** in `wasm/` for enhanced diagnostics

### 3. Process Improvement

- **Enforce src/ read-only policy** via pre-commit hooks
- **Separate upstream patches** from migration work
- **Use proper branching strategy** for upstream contributions

## Risk Analysis

**Current state is HIGH RISK:**
- Cannot cleanly merge upstream TypeScript improvements
- May have introduced regressions in core TypeScript functionality  
- Creates confusion about source of truth
- Makes WASM migration harder to review and validate

## Conclusion

The current drift represents a **fundamental violation** of the WASM migration principles. Immediate action is required to restore the `src/` directory to upstream state and relocate all custom functionality to the `wasm/` directory where it belongs.

**RECOMMENDATION: Complete revert of src/ modifications and proper architectural separation.**