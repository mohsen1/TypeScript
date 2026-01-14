// Test file for type parameter parsing edge cases
// Testing various TS1005 error scenarios

// 1. Basic type parameters (should work)
function basic<T>(x: T): T { return x; }

// 2. Type parameters with constraints (should work)
function constrained<T extends string>(x: T): T { return x; }

// 3. Type parameters with defaults (should work)
function withDefault<T = string>(x: T): T { return x; }

// 4. Type parameters with constraint and default (should work)
function both<T extends string = string>(x: T): T { return x; }

// 5. Variance annotations (TypeScript 4.7)
// These may cause TS1005 errors if not implemented
type Getter<out T> = () => T;
type Setter<in T> = (value: T) => void;

// 6. Nested type parameters (should work)
function nested<T, U>(x: T, y: U): [T, U] { return [x, y]; }

// 7. Complex constraints
interface Animal { name: string; }
interface Dog extends Animal { bark(): void; }
function complex<T extends Animal>(x: T): string { return x.name; }

// 8. Conditional type parameters
type Conditional<T> = T extends string ? number : boolean;

// 9. Type parameters in classes
class GenericClass<T> {
  value: T;
  constructor(value: T) {
    this.value = value;
  }
}

// 10. Type parameters in interfaces
interface GenericInterface<T> {
  getValue(): T;
}

// 11. Type parameters with variance in classes
class Covariant<out T> {
  get(): T { return {} as T; }
}

class Contravariant<in T> {
  set(value: T): void {}
}

// 12. Missing closing > (should cause TS1005)
function missing<T>(x: T): T { return x; }

// 13. Extra commas (might cause TS1005)
function extraComma<T,>(x: T): T { return x; }

// 14. Type parameters with multiple constraints (not valid TS, but interesting)
// function multiple<T extends Animal & Dog>(x: T): string { return x.name; }
