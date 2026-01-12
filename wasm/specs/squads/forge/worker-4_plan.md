# Worker 4 Plan - Squad Forge

## Mission
Fix TS2792: Module Resolution

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement TS2792 Error: Module not found or cannot be resolved**

### Background
TypeScript should emit TS2792 error when an import/export statement references a module that cannot be found or resolved. This involves module resolution logic including:
1. Finding module files based on module specifier
2. Resolving module extensions (.ts, .tsx, .d.ts)
3. Handling node_modules resolution
4. Supporting path mappings (tsconfig paths)
5. Supporting @types package resolution

### Success Criteria
- Emit TS2792 when module cannot be found
- Don't emit for existing modules
- Handle relative imports (./foo, ../bar)
- Handle absolute imports from node_modules
- Handle module resolution with @types packages
- Support baseUrl and paths from tsconfig

### Implementation Steps
1. [ ] Read existing module resolution code in `src/`
2. [ ] Find where imports are resolved
3. [ ] Implement module lookup logic:
   - Check if file exists with .ts, .tsx, .d.ts extensions
   - Check node_modules for package
   - Check @types/ package
4. [ ] Implement TS2792 error emission when module not found
5. [ ] Test with various import patterns

### Key Code Locations
- Module resolution code (likely in `src/` directory)
- Import statement handling in binder/checker
- `src/checker/types/diagnostics.rs` - TS2792 error code

### Test Cases to Implement
```typescript
// Should emit TS2792
import { foo } from './nonexistent'; // Error: Cannot find module './nonexistent'
import { bar } from 'missing-package'; // Error: Cannot find module 'missing-package'

// Should NOT emit (module exists)
import { baz } from './existing'; // OK - ./existing.ts exists
import { qux } from 'typescript'; // OK - in node_modules
import { Any } from '@types/node'; // OK - @types package
```

### Edge Cases
- Module specifier with no extension
- Module specifier in quotes ("lodash" vs 'lodash')
- Triple-slash directives (/// <reference types="node" />)
- Re-exports (export * from 'foo')
- Type-only imports (import type { X } from 'y')

## Task Queue
- [ ] After TS2792: coordinate with other workers on module system

## Completed
- [x] Fix Element Access Literal Keys - Merged to squad/forge

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] binder/module: Implement TS2792 module resolution errors`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
