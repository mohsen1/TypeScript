# Worker 3 Task List

## Current Task
- [ ] **CFA-5: Integrate Flow Graph into Checker**
  - Add `check_flow_usage` method in `src/checker/mod.rs`
  - Call Flow Graph builder after binding phase
  - Query analysis results for TS2454 errors
  - Emit "Variable used before assignment" diagnostics

## Queue
- [ ] **CFA-6: Handle class property initialization (TS2564)**
  - Extend Flow Graph to track class properties
  - Check definite assignment in constructors
  - Handle property declarations with initializers
  - Emit "Property not initialized" errors
  - Run conformance tests to verify 90% reduction goal

## Completed
(none yet)
