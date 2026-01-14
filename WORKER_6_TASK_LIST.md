# Worker 6 Task List

**Maintained by:** EM-2
**Branch:** worker-6 → rust
**Worktree:** /tmp/orchestrator-workspace/worktrees/worker-6

---

## Current Task (PENDING)

### Task 1: Fix File Locals and Scope Chain Resolution

**Priority:** CRITICAL (Priority #2 from PROJECT_DIRECTION.md)
**Focus:** `file_locals` population and symbol accessibility
**Impact:** TS2304 missing errors (116 occurrences) - symbols not found when they should be
**Location:** `src/thin_binder.rs`, `src/symbol_table.rs`

#### Background
The compiler's `file_locals` table is not correctly populated with symbols from the library context and global scope. This causes valid identifiers to fail resolution because they're not in the accessible symbol chain, even though they're defined somewhere in the project.

#### Requirements
1. Fix `file_locals` population from library context (globals should be in each file's locals)
2. Ensure scope chain correctly walks: locals → globals → library symbols
3. Debug namespace/import resolution edge cases
4. Verify symbol table merging logic for multi-file projects

#### Acceptance Criteria
- [ ] Imported symbols resolve without TS2304 error
- [ ] Namespace-qualified symbols resolve correctly
- [ ] Multi-file projects share symbols correctly
- [ ] Running `./wasm/differential-test/run-conformance.sh --max=10000` shows reduction in TS2304 missing errors
- [ ] `cargo test --lib` passes in wasm/ directory

#### Testing Strategy
```bash
# Test import resolution
cat > /tmp/test-import.ts << 'EOF'
import { Promise } from 'typescript';
Promise.resolve();
EOF
node wasm/distance.js /tmp/test-import.ts

# Run conformance focused on TS2304
./wasm/differential-test/run-conformance.sh --max=10000

# Find specific TS2304 issues
node wasm/differential-test/find-ts2304.mjs

# Run unit tests
cd wasm && cargo test symbol_table
cd wasm && cargo test thin_binder
```

#### Implementation Notes
- Look at `src/thin_binder.rs` for `file_locals` population logic
- The scope chain must correctly fall through from file locals to global/library symbols
- Import statements should add symbols to `file_locals`
- Namespace declarations should create a nested scope context
- The `SymbolTable::resolve` function needs to walk the full scope chain

#### Key Concepts
- **file_locals**: Symbols defined in or imported into the current file
- **scope chain**: locals → parent scopes → globals → library symbols
- **symbol merging**: How symbols from multiple files are combined
- **import resolution**: How `import { x } from './y'` adds `x` to file locals

---

## Queue (Future Tasks)

### Task 2: Verify Symbol Table Merging for Multi-File Projects
**Priority:** HIGH
**Focus:** Symbol table merging across project files
**Dependencies:** Task 1

### Task 3: Fix Namespace Resolution Edge Cases
**Priority:** MEDIUM
**Focus:** Nested namespaces and namespace merging
**Dependencies:** Task 1

---

## Anti-Priorities (DO NOT WORK ON)
- New emitter transforms
- LSP features
- CLI argument parsing
- Performance micro-optimizations

---

## Status Updates
- **Created:** 2026-01-14
- **Last Updated:** 2026-01-14
- **Current Focus:** File Locals and Scope Chain Resolution
