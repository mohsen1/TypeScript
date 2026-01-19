// Test Application type expansion - verify no false TS2322 errors from unexpanded Application types

// Case 1: Array<string> should expand properly
const arr1: Array<string> = ["hello", "world"];
const arr2: string[] = arr1; // Should NOT error - Array<string> === string[]

// Case 2: Promise<number> expansion
declare const p1: Promise<number>;
const p2: Promise<number> = p1; // Should NOT error

// Case 3: Nested generic applications
interface Box<T> {
    value: T;
}
const box1: Box<string> = { value: "hello" };
const box2: Box<string> = box1; // Should NOT error

// Case 4: Generic type alias applications
type Container<T> = { item: T };
const container1: Container<number> = { item: 42 };
const container2: Container<number> = container1; // Should NOT error

// Case 5: Application to structural type assignability
interface Named<T> {
    name: string;
    data: T;
}
const named1: Named<boolean> = { name: "test", data: true };
const obj1: { name: string; data: boolean } = named1; // Should NOT error - Application expands to match structural

// Case 6: Nested application types
type Wrapper<T> = { wrapped: T };
type DoubleWrapper<T> = Wrapper<Wrapper<T>>;
const dw1: DoubleWrapper<string> = { wrapped: { wrapped: "hello" } };
const dw2: Wrapper<Wrapper<string>> = dw1; // Should NOT error

// Case 7: Function type with generic application
type Func<T> = (x: T) => T;
const fn1: Func<number> = (x) => x;
const fn2: (x: number) => number = fn1; // Should NOT error

// Case 8: Multiple type parameters
type Pair<A, B> = { first: A; second: B };
const pair1: Pair<string, number> = { first: "hello", second: 42 };
const pair2: { first: string; second: number } = pair1; // Should NOT error

// Case 9: Generic constraint with application
interface Indexable<T> {
    [key: string]: T;
}
const idx1: Indexable<number> = { a: 1, b: 2 };
const idx2: { [key: string]: number } = idx1; // Should NOT error

// Case 10: Array methods return Application types
const strArr: string[] = ["a", "b", "c"];
const mapped: Array<string> = strArr.map(x => x.toUpperCase()); // Should NOT error

// Cases that SHOULD error (to verify we still catch real errors)
const errArr: Array<string> = [1, 2, 3]; // SHOULD emit TS2322
const errBox: Box<string> = { value: 42 }; // SHOULD emit TS2322
const errPair: Pair<string, number> = { first: 42, second: "hello" }; // SHOULD emit TS2322
