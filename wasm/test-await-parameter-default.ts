// Test await handling in parameter defaults
// Parameter defaults are evaluated in the parent scope, not the async function body

// Test 1: await used as identifier reference in parameter default (should work - TS2304 at most)
async function test1(a = await) {
    return a;
}

// Test 2: await used as expression in parameter default (should emit TS1109)
async function test2(a = await 42) {
    return a;
}

// Test 3: await used in function body (should work correctly)
async function test3() {
    const x = await Promise.resolve(42);
    return x;
}

// Test 4: await in nested parameter default
async function test4(a = (b = await 42) => b) {
    return a;
}

console.log("All tests compiled");
