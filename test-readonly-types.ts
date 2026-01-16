// Test readonly array/tuple type assignability
// Source: https://www.typescriptlang.org/docs/handbook/release-notes/typescript-3.4.html

// Readonly arrays
const mutableArray: number[] = [1, 2, 3];
const readonlyArray: readonly number[] = [1, 2, 3];

// This should be OK (mutable can be assigned to readonly)
const test1: readonly number[] = mutableArray;

// This should ERROR (readonly cannot be assigned to mutable)
const test2: number[] = readonlyArray;

// Readonly tuples
type ReadonlyTuple = readonly [number, string];
const mutableTuple: [number, string] = [1, "hello"];
const readonlyTuple: ReadonlyTuple = [1, "hello"];

// This should be OK (mutable can be assigned to readonly)
const test3: ReadonlyTuple = mutableTuple;

// This should ERROR (readonly cannot be assigned to mutable)
const test4: [number, string] = readonlyTuple;

// Readonly arrays should reject mutations
readonlyArray.push(4); // Should error
readonlyArray[0] = 5; // Should error
