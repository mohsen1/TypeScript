// Test union type assignability - these should NOT emit TS2322

// Case 1: Union to wider union (string | number assignable to string | number | boolean)
type T1 = string | number;
type T2 = string | number | boolean;
let x1: T1 = "hello";
let y1: T2 = x1; // Should NOT error - T1 is subset of T2

// Case 2: Base type to union (string assignable to string | number)
let x2: string = "hello";
let y2: string | number = x2; // Should NOT error - string is in union

// Case 3: Union to union with same types
type T3 = string | number;
type T4 = number | string; // Different order
let x3: T3 = "hello";
let y3: T4 = x3; // Should NOT error - same types

// Case 4: Narrow type to wider union
type T5 = "a" | "b";
type T6 = "a" | "b" | "c";
let x4: T5 = "a";
let y4: T6 = x4; // Should NOT error - T5 is subset of T6

// Case 5: Generic with union constraint
function foo<T extends string>(x: T): T | number {
    return x; // Should NOT error - T is assignable to T | number
}

// Case 6: Function parameter union
function bar(x: string | number): void {
    const y: string | number | boolean = x; // Should NOT error
}

// Cases that SHOULD error (to verify we still catch real errors)
let err1: string = 5; // SHOULD emit TS2322
let err2: number = "hello"; // SHOULD emit TS2322
