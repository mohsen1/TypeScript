# Worker 4 Plan - Squad Anvil

## Mission
Fix CLI/Driver Issues and Import Equals

Status: Active
Priority: P1 (High)

## Current Assignment
**Fix CLI/Driver Compilation Issues (3 failing tests)**

### Failing Tests
1. `cli::driver_tests::compile_generic_utility_library_type_utilities`
2. `cli::driver_tests::compile_shorthand_methods`
3. `cli::driver_tests::invalidate_paths_with_dependents_symbols_handles_import_equals`

### Implementation Steps

#### Part 1: Import Equals (`import x = require()`)
1. [ ] Read the failing test `invalidate_paths_with_dependents_symbols_handles_import_equals`
2. [ ] Find import equals handling in `src/thin_emitter.rs`
3. [ ] Ensure CommonJS output: `import x = require('y')` -> `var x = require('y')`
4. [ ] Test: `./wasm/test.sh 2>&1 | grep -E "import_equals"`

#### Part 2: Generic Utility Library
1. [ ] Read the failing test `compile_generic_utility_library_type_utilities`
2. [ ] This likely tests complex generic patterns - check if it's a type inference issue or emission issue
3. [ ] If emission: ensure generic type annotations are stripped correctly
4. [ ] If inference: may need Forge squad help

#### Part 3: Shorthand Methods
1. [ ] Read the failing test `compile_shorthand_methods`
2. [ ] Ensure shorthand method syntax is handled: `{ foo() {} }` vs `{ foo: function() {} }`
3. [ ] Check both parsing and emission

### Key Code Locations
- `src/cli/driver.rs` - compilation driver
- `src/thin_emitter.rs` - import/export emission
- `src/thin_parser.rs` - shorthand method parsing

### Import Equals Transformation
```typescript
// TypeScript
import fs = require('fs');

// CommonJS output
var fs = require('fs');

// ES module output (if module: esnext)
// Keep as-is or convert to: import * as fs from 'fs';
```

## Task Queue
- [ ] After CLI fixes: help with parser error recovery

## Completed
- [x] (Move finished items here)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] cli: Fix import equals emission for CommonJS`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
