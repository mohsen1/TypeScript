# Worker 3 Task: Fix TS2304 Symbol Resolution (Tier 3 - CRITICAL)

## Assignment
Fix "Cannot find name" errors for valid symbols - globals and imports are not resolving correctly. This is CRITICAL because unresolved symbols default to `Any`, which hides downstream type checking errors.

## Problem Analysis
The compiler is emitting TS2304 ("Cannot find name") for symbols that should be resolved from:
1. Standard library globals (console, Promise, Array, Object, etc.)
2. Imported symbols
3. Global type declarations

**Critical Impact**: When a symbol resolves to ERROR due to TS2304, it defaults to `Any`, which masks downstream TS2322 errors.

## Root Cause Areas

### 1. lib.d.ts Loading and Merging
File: `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/lib_loader.rs`

**Current Implementation**:
- Lines 20-39: `load_default_lib_dts()` function exists
- Lines 73-82: `merge_lib_symbols()` merges lib symbols into target table
- Line 77: Only adds if not already defined (user code can override)

**Investigation Needed**:
1. Is `load_default_lib_dts()` being called during checker initialization?
2. Are lib symbols actually being merged into the global scope?
3. Is the merge happening BEFORE user code type checking?

### 2. Binder Symbol Resolution
Directory: `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/binder/` (search needed)

**Tasks**:
1. Find the binder directory/files (the Glob found no files at that path)
2. Understand how the binder creates and manages symbol tables
3. Verify scope chain walking during symbol lookup
4. Check if lib symbols are in the root scope

**Search Commands**:
```bash
# Find binder files
find /Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src -name "*binder*.rs" -type f

# Find symbol table operations
grep -rn "SymbolTable\|file_locals\|lookup_symbol" wasm/src/
```

### 3. Checker Symbol Lookup
File: `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/thin_checker.rs`

**Key Locations**:
- Line 645: `get_type_of_identifier()` - identifier type resolution
- Line 14087: `error_property_not_exist_at()` - emits errors for unresolved names
- Line 14083: `is_known_global_value_name()` - checks if name is a known global

**Investigation**:
1. How does `get_type_of_identifier()` resolve symbols?
2. Does it check lib.d.ts symbols?
3. Is the scope chain walked correctly?

## Implementation Plan

### Phase 1: Verify lib.d.ts Loading (HIGH PRIORITY)

**Step 1.1: Check if lib loading is integrated**
```bash
# Find where load_default_lib_dts is called
grep -rn "load_default_lib_dts" wasm/src/

# Find ThinChecker initialization
grep -n "ThinChecker::new\|fn new" wasm/src/thin_checker.rs | head -20
```

**Step 1.2: Verify lib.d.ts file exists**
```bash
# Check if lib.d.ts exists
ls -lh /Users/mohsenazimi/code/orchestrator-config/workspace/repo/tests/lib/lib.d.ts

# Check its contents
head -100 /Users/mohsenazimi/code/orchestrator-config/workspace/repo/tests/lib/lib.d.ts
```

**Step 1.3: Add lib loading to checker initialization**
If lib loading is NOT happening, modify thin_checker.rs to:
1. Call `load_default_lib_dts()` during checker creation
2. Merge lib symbols into the binder's global scope
3. Ensure this happens BEFORE checking user code

### Phase 2: Fix Symbol Table Merging

**Step 2.1: Find binder initialization**
The binder must have a root scope where lib symbols are merged.

**Search for**:
- Binder creation/initialization
- File locals table setup
- Global scope setup

**Step 2.2: Ensure proper merge order**
```rust
// Pseudocode for correct order:
fn initialize_checker() {
    // 1. Load lib.d.ts
    let lib_file = load_default_lib_dts();

    // 2. Create binder with lib symbols
    let mut binder = ThinBinderState::new();
    merge_lib_symbols(&mut binder.file_locals, &[lib_file]);

    // 3. Bind user code (user symbols overlay lib symbols)
    binder.bind_source_file(user_arena, source_file_idx);

    // 4. Create checker with merged scope
    let checker = ThinChecker::new(binder, types, arena);
}
```

### Phase 3: Fix Scope Chain Walking

**Step 3.1: Understand current symbol lookup**
Read through the identifier resolution code to understand:
1. How scopes are nested
2. How the lookup walks up the chain
3. Whether it reaches the global scope

**Step 3.2: Verify global scope access**
Ensure that symbol lookup:
1. Starts at the current scope
2. Walks up through parent scopes
3. Eventually reaches the file_locals (global scope)
4. Checks lib symbols if not found in user code

**Step 3.3: Add debug logging (temporarily)**
```rust
fn get_type_of_identifier(&mut self, idx: NodeIndex) -> TypeId {
    let name = self.get_identifier_name(idx);

    // DEBUG: Log symbol lookup
    eprintln!("Looking up symbol: {}", name);

    if let Some(sym_id) = self.lookup_symbol(name) {
        eprintln!("  Found in scope");
        return self.get_type_of_symbol(sym_id);
    }

    // Check lib globals
    if let Some(lib_sym) = self.lookup_lib_symbol(name) {
        eprintln!("  Found in lib.d.ts");
        return self.get_type_of_symbol(lib_sym);
    }

    eprintln!("  NOT FOUND - emitting TS2304");
    self.error_cannot_find_name(name, idx);
    TypeId::ERROR
}
```

### Phase 4: Fix Import Resolution (if needed)

**Step 4.1: Check import binding**
Verify that imported symbols are properly bound in the binder.

**Step 4.2: Check import type resolution**
Ensure that the checker can resolve types from imported modules.

## Testing Strategy

### Test Cases to Create
```typescript
// Test 1: Standard lib globals
console.log("test"); // Should NOT emit TS2304 for 'console'

// Test 2: Standard lib types
const p: Promise<number> = Promise.resolve(1); // Should NOT emit TS2304

// Test 3: Standard lib constructors
const arr = new Array<number>(); // Should NOT emit TS2304

// Test 4: Standard lib interfaces
const obj: Object = {}; // Should NOT emit TS2304

// Test 5: Global functions
const str = String(123); // Should NOT emit TS2304

// Test 6: DOM types (if lib.dom.d.ts is loaded)
const elem: HTMLElement = document.createElement('div'); // Should NOT emit TS2304
```

### Running Tests
```bash
# Build WASM
cd /Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm
wasm-pack build --target web --out-dir pkg

# Run TS2304-specific test script
cd differential-test
node find-ts2304.mjs

# Run conformance tests
bash run-conformance.sh --max=500 --workers=4

# Check for TS2304 count reduction
# Before fix: note the count
# After fix: verify reduction
```

### Verification Script
```bash
# Create a simple test file
cat > /tmp/test-globals.ts << 'EOF'
// Should resolve from lib.d.ts
console.log("test");
const arr: Array<number> = [];
const p: Promise<string> = Promise.resolve("test");
const obj: Object = {};
EOF

# Test with tsc (baseline)
tsc --noEmit /tmp/test-globals.ts

# Test with our compiler
cd /Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/differential-test
node compare-single.mjs /tmp/test-globals.ts
```

## File Locations Reference

### Files to Read/Modify
1. `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/lib_loader.rs` - lib loading
2. `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/thin_checker.rs` - symbol lookup
3. Binder files (find first): symbol table management
4. `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/tests/lib/lib.d.ts` - standard library

### Key Functions to Find
- `ThinChecker::new()` - checker initialization
- `get_type_of_identifier()` - identifier resolution
- `lookup_symbol()` - symbol table lookup
- `bind_source_file()` - binder entry point
- `merge_lib_symbols()` - lib symbol merging

## Success Criteria
1. Reduction in TS2304 "Missing Errors" for standard lib symbols
2. console, Promise, Array, Object resolve correctly
3. Import resolution works (if applicable)
4. No regressions in other error codes
5. Unblocks proper type checking (no more Any poisoning)

## Expected Impact
This fix will have cascading effects:
- More accurate TS2322 errors (no longer masked by Any from ERROR types)
- More accurate property access errors
- Overall type checking accuracy will improve

## Branch
Create branch: `worker-3-ts2304-symbol-resolution`

## Priority
CRITICAL - This blocks accurate type checking across the board. Fix this FIRST before other type checking improvements.

## Notes
- This is the most impactful fix in this tier
- Coordinate with EM-2 to report progress
- May need to coordinate with Worker 2 (TS2322) once this is fixed
- Document all changes to symbol resolution logic
