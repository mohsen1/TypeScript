// TS2304 Symbol Resolution Test Cases
// These test cases verify that TS2304 "Cannot find name" is emitted correctly

// Test 1: Global symbol from declare (should NOT emit TS2304)
declare const globalConst: string;
function test1() {
    return globalConst;
}

// Test 2: Local variable in same scope (should NOT emit TS2304)
function test2() {
    const local = 42;
    return local;
}

// Test 3: Variable in outer scope (should NOT emit TS2304)
const outerVar = "outer";
function test3() {
    return outerVar;
}

// Test 4: Variable from different function scope (SHOULD emit TS2304)
function test4a() {
    const inner = "hidden";
}
function test4b() {
    return inner; // TS2304: Cannot find name 'inner'
}

// Test 5: Type parameter in function body (should NOT emit TS2304)
function test5<T>(x: T): T {
    return x;
}

// Test 6: Type parameter as type constraint (should NOT emit TS2304)
function test6<T extends string>(x: T): T {
    return x;
}

// Test 7: Function parameter (should NOT emit TS2304)
function test7(param: number) {
    return param;
}

// Test 8: Undeclared variable (SHOULD emit TS2304)
function test8() {
    return undeclaredVar; // TS2304: Cannot find name 'undeclaredVar'
}

// Test 9: Global intrinsic types (should NOT emit TS2304)
function test9() {
    return undefined;
}

// Test 10: Array destructuring (should NOT emit TS2304)
function test10() {
    const [a, b] = [1, 2];
    return a + b;
}

// Test 11: Object destructuring (should NOT emit TS2304)
function test11() {
    const { x, y } = { x: 1, y: 2 };
    return x + y;
}

// Test 12: Block scope variable (should NOT emit TS2304)
function test12() {
    {
        const blockScoped = 123;
    }
    // NOTE: This would emit TDZ error, not TS2304
    // return blockScoped;
}

// Test 13: Variable from ambient module (should NOT emit TS2304)
declare module "ambient" {
    export const ambientVar: number;
}
function test13() {
    // This would need import to work
    // return ambientVar;
}

// Test 14: Class member access (should NOT emit TS2304)
class MyClass {
    member = 42;
    method() {
        return this.member;
    }
}

// Test 15: Static class member access (should NOT emit TS2304)
class MyClass2 {
    static staticMember = "static";
    staticMethod() {
        return this.staticMember;
    }
}

// Test 16: Enum member access (should NOT emit TS2304)
enum MyEnum {
    A = 1,
    B = 2
}
function test16() {
    return MyEnum.A;
}

// Test 17: Import statement (should NOT emit TS2304)
import { useState } from "react"; // Would fail if module not found

// Test 18: Namespace member (should NOT emit TS2304)
namespace MyNamespace {
    export const nsVar = "namespace";
}
function test18() {
    return MyNamespace.nsVar;
}

// Test 19: Variable shadowing (should NOT emit TS2304)
const shadowed = "outer";
function test19() {
    const shadowed = "inner";
    return shadowed;
}

// Test 20: Loop variable (should NOT emit TS2304)
function test20() {
    for (let i = 0; i < 10; i++) {
        console.log(i);
    }
}
