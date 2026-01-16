// Test readonly array/tuple type assignability
// This should demonstrate that:
// 1. Mutable arrays/tuples CAN be assigned to readonly
// 2. Readonly arrays/tuples CANNOT be assigned to mutable

// Test 1: Readonly arrays
const mutableArray: number[] = [1, 2, 3];
const readonlyArray: readonly number[] = [1, 2, 3];

// This should be OK (mutable can be assigned to readonly)
const test1: readonly number[] = mutableArray;

// This should ERROR (readonly cannot be assigned to mutable)
const test2: number[] = readonlyArray;

// Test 2: Readonly tuples
type ReadonlyTuple = readonly [number, string];
const mutableTuple: [number, string] = [1, "hello"];
const readonlyTuple: ReadonlyTuple = [1, "hello"];

// This should be OK (mutable can be assigned to readonly)
const test3: ReadonlyTuple = mutableTuple;

// This should ERROR (readonly cannot be assigned to mutable)
const test4: [number, string] = readonlyTuple;
