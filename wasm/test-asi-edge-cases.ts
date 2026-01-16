// ASI Edge Cases Test File
// Tests automatic semicolon insertion in various contexts

// ==================== RESTRICTED PRODUCTIONS ====================
// These should apply ASI immediately after line break

// Test 1: return with line break
function testReturn1() {
    return
    42;
    // Should parse as: return; 42;
    // NOT as: return 42;
}

// Test 2: return with value on same line
function testReturn2() {
    return 42;
    // Should parse as: return 42;
}

// Test 3: throw with line break
function testThrow1() {
    throw
    new Error("test");
    // Should parse as: throw; new Error("test");
    // NOT as: throw new Error("test");
}

// Test 4: throw with value on same line
function testThrow2() {
    throw new Error("test");
    // Should parse as: throw new Error("test");
}

// Test 5: break with line break
function testBreak1() {
    while (true) {
        break
        console.log("unreachable");
        // Should parse as: break; console.log("unreachable");
    }
}

// Test 6: continue with line break
function testContinue1() {
    while (true) {
        continue
        console.log("unreachable");
        // Should parse as: continue; console.log("unreachable");
    }
}

// ==================== POSTFIX OPERATORS ====================
// Postfix ++/-- should NOT apply if there's a line break

// Test 7: postfix ++ with line break (should NOT be postfix)
let x = 5;
x
++
// Should parse as: x; ++ (not x++)
// ASI applies between x and ++

// Test 8: postfix ++ without line break
x++;
// Should parse as: x++

// Test 9: postfix -- with line break
let y = 10;
y
--

// ==================== ARROW FUNCTIONS ====================
// Line breaks prevent arrow function parsing

// Test 10: async arrow function with line break after async
async
() => 42;
// Should NOT parse as async arrow function
// ASI applies after async

// Test 11: arrow function with line break before =>
let z = 5
(z) => z * 2;
// Should NOT parse as arrow function
// ASI applies after z

// Test 12: arrow function without line break
(w) => w * 2;
// Should parse as arrow function

// Test 13: return with object literal
function testReturnObject() {
    return
    {
        x: 1,
        y: 2
    };
    // Should parse as: return; {x: 1, y: 2};
    // NOT as: return {x: 1, y: 2};
}

// Test 14: return with object literal (correct way)
function testReturnObjectCorrect() {
    return {
        x: 1,
        y: 2
    };
}

// ==================== EMPTY STATEMENTS ====================
// ASI should create empty statements where appropriate

// Test 15: if with empty else branch
if (true)
    ; // Empty statement via explicit semicolon
else
    ; // Empty statement via explicit semicolon

// Test 16: while with empty body
while (false)
    ; // Empty statement

// ==================== DO-WHILE ====================
// ASI is required before while in do-while

// Test 17: do-while without semicolon (ASI should apply)
do {
    console.log("test");
}
while (false)
// ASI should apply here

// Test 18: do-while with semicolon
do {
    console.log("test");
};
while (false);

// ==================== YIELD (generators) ====================
// yield is also a restricted production

function* testYield1() {
    yield
    42;
    // Should parse as: yield; 42;
}

function* testYield2() {
    yield 42;
    // Should parse as: yield 42;
}
