// Test case for TS2304 symbol resolution
declare const globalVar: string;

function test1() {
    const local = 42;
    return local; // Should NOT emit TS2304
}

function test2() {
    return globalVar; // Should NOT emit TS2304
}

function test3() {
    return unknownVar; // SHOULD emit TS2304
}
