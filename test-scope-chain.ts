// Test file for BIND-10: Fix symbol lookup order
// This tests that:
// 1. Import symbols are visible in inner scopes (functions, blocks)
// 2. Local variables correctly shadow imports
// 3. Scope chain traversal follows: local -> module -> global

// Test 1: Import symbol visible inside function
import { Array } from "./types";

function testImportVisibleInFunction() {
    // Should resolve to the imported Array symbol
    const arr: Array<string> = [];
    console.log(arr);
}

// Test 2: Local variable shadows import
import { console } from "./types";

function testShadowing() {
    // Local 'console' should shadow the import
    let console = "custom console";
    console.log(console); // Error: console.log is not a function (it's a string)
}

// Test 3: Nested scopes
import { Promise } from "./types";

function testNestedScopes() {
    // Should resolve to imported Promise
    function inner() {
        // Should still resolve to imported Promise
        const p: Promise<void> = Promise.resolve();
    }
}

// Test 4: Block scope shadowing
import { Object } from "./types";

function testBlockShadowing() {
    {
        // Local Object shadows the import in this block
        let Object = "custom object";
        console.log(Object); // Should be the string
    }
    {
        // Should resolve to imported Object here
        const obj: Object = {};
    }
}

// Test 5: Module declarations create scope boundaries
namespace MyModule {
    import { String } from "./types";

    function inNamespace() {
        // Should resolve to imported String inside namespace
        const s: String = "hello";
    }
}

// Test 6: Global symbols (from lib.d.ts) should be visible
function testGlobalSymbols() {
    // These should resolve to global symbols from lib.d.ts
    const num: Number = 42;
    const bool: Boolean = true;
    const undef: Undefined = undefined;
}
