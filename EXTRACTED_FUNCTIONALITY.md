# Extracted TypeScript Functionality for WASM Implementation

This document captures functionality that was improperly implemented in src/ files and should be re-implemented in the wasm/ directory.

## From src/compiler/checker.ts

### Enhanced Error Messages (TS2322)
- `createPropertyErrorMessage()` - Property-aware error messages showing full type paths
- `addTypeOriginInfo()` - Type tracing that shows where types were inferred from
- Enhanced `reportRelationError()` with related diagnostic information

**Target location:** `wasm/src/diagnostics/enhanced_errors.rs`

### Type Tracing Features  
- Support for tracking type origins from return statements, expressions, arguments
- Related information chains for complex type errors
- Property-specific error context

**Target location:** `wasm/src/diagnostics/type_tracing.rs`

## From src/compiler/parser.ts

### TS1005 False Positive Fixes
- Pattern 6: Object literal comma handling with line breaks
- ASI-aware semicolon parsing with `tryParseSemicolon()`
- Line break tolerance in object literals

**Target location:** `wasm/src/parser/asi_handling.rs`

## From src/compiler/diagnosticMessages.json

### New Diagnostic Messages (to be moved to WASM)
- 9512: "The error is in property '{0}'"
- 9513: "Type '{0}' was inferred from expression at this location"  
- 9514: "Type '{0}' was inferred from argument '{1}' at position {2}"
- 9515: "Type '{0}' was inferred from return statement"

**Target location:** `wasm/diagnostics/messages.json`

## Implementation Strategy

1. **Error Enhancement System:** Implement in `wasm/src/diagnostics/` as a post-processing layer
2. **Parser Improvements:** Implement ASI fixes in WASM parser with feature flag
3. **Diagnostic Messages:** Create WASM-specific diagnostic message system
4. **Integration Hooks:** Add minimal (<10 lines) hooks in TypeScript to call WASM enhancements

This approach maintains clean separation while preserving the valuable functionality that was developed.