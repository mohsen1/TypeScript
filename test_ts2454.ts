// Test case for TS2454 - Variable used before assignment

// Test 1: Basic case - should report TS2454
function test1() {
    let x: string;
    console.log(x);  // Should report TS2454
}

// Test 2: Conditional assignment - should report TS2454
function test2() {
    let x: string;
    if (Math.random() > 0.5) {
        x = "hello";
    }
    console.log(x);  // Should report TS2454 (not all paths assign)
}

// Test 3: All paths assign - should NOT report TS2454
function test3() {
    let x: string;
    if (Math.random() > 0.5) {
        x = "hello";
    } else {
        x = "world";
    }
    console.log(x);  // Should NOT report TS2454
}

// Test 4: Has initializer - should NOT report TS2454
function test4() {
    let x: string = "hello";
    console.log(x);  // Should NOT report TS2454
}
