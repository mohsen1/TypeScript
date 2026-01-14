// Test type parameter resolution
function identity<T>(x: T): T {
    return x;
}

// Test interface resolution
interface Person {
    name: string;
}

let p: Person = { name: "Alice" };
