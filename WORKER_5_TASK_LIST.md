# Worker 5 Task List

**Maintained by:** EM-2
**Branch:** worker-5 → rust
**Worktree:** /tmp/orchestrator-workspace/worktrees/worker-5

---

## Current Task (PENDING)

### Task 1: Fix Lib Injection and Global Scope Resolution

**Priority:** CRITICAL (Priority #2 from PROJECT_DIRECTION.md)
**Focus:** lib.d.ts loading and global symbol resolution
**Impact:** TS2304 extra errors (343 occurrences) - root cause of "Any" poisoning
**Location:** `src/lib_loader.rs`, `src/thin_binder.rs`

#### Background
The compiler fails to load and merge TypeScript library definitions (`lib.d.ts`, `lib.dom.d.ts`) into the global symbol table. This causes basic globals like `console`, `Promise`, and `Array` to fail resolution, defaulting to `Any`, which poisons all downstream type checking.

#### Requirements
1. Verify `lib_loader.rs` correctly reads and parses TypeScript library files
2. Ensure library symbols are merged into the root `SymbolTable` (not just stored separately)
3. Fix module augmentation resolution - merging `interface Window` from multiple files
4. Debug why `console.log` and other DOM globals fail to resolve

#### Acceptance Criteria
- [ ] Test: `console.log("hello")` resolves without TS2304 error
- [ ] Test: `Promise.resolve()` resolves without TS2304 error
- [ ] Test: `new Array()` resolves without TS2304 error
- [ ] Running `./wasm/differential-test/run-conformance.sh --max=10000` shows reduction in TS2304 extra errors
- [ ] `cargo test --lib` passes in wasm/ directory

#### Testing Strategy
```bash
# Test basic global resolution
echo 'console.log("test");' > /tmp/test-global.ts && node wasm/distance.js /tmp/test-global.ts

# Run conformance focused on TS2304
./wasm/differential-test/run-conformance.sh --max=10000

# Find specific TS2304 issues
node wasm/differential-test/find-ts2304.mjs

# Run unit tests
cd wasm && cargo test lib_loader
```

#### Implementation Notes
- Look at `src/lib_loader.rs` for library file loading logic
- The root `SymbolTable` should contain DOM globals after library loading
- Module augmentation merges interface declarations across files
- The `thin_binder.rs` `bind_root` function is where library symbols should be injected
- TypeScript's behavior: `lib.d.ts` and `lib.dom.d.ts` globals are always available

---

## Queue (Future Tasks)

### Task 2: Verify Library Context Propagation
**Priority:** HIGH (Supports Task 1)
**Dependencies:** Task 1
**Focus:** Ensure all files can access library-injected symbols

### Task 3: Fix Symbol Table Merging for Multi-File Projects
**Priority:** MEDIUM
**Focus:** Symbol table merging across project files
**Dependencies:** Task 1, Task 2

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
- **Current Focus:** Lib Injection and Global Scope Resolution
