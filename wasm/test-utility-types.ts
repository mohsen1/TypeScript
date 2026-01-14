// Test file for built-in utility type resolution
// Testing TS2304 errors for Exclude, ReturnType, Parameters, etc.

// Test 1: Exclude
type Test1 = Exclude<string | number, string>;
// Should resolve to: number

// Test 2: ReturnType
function foo() {
  return { x: 10, y: 20 };
}
type FooReturn = ReturnType<typeof foo>;
// Should resolve to: { x: number; y: number; }

// Test 3: Parameters
type FooParams = Parameters<typeof foo>;
// Should resolve to: []

function bar(a: string, b: number) {
  return a + b;
}
type BarParams = Parameters<typeof bar>;
// Should resolve to: [string, number]

// Test 4: Partial
interface User {
  name: string;
  age: number;
  email: string;
}
type PartialUser = Partial<User>;
// Should resolve to: { name?: string; age?: number; email?: string; }

// Test 5: Required
type RequiredUser = Required<Partial<User>>;
// Should resolve to: { name: string; age: number; email: string; }

// Test 6: Readonly
type ReadonlyUser = Readonly<User>;
// Should resolve to: { readonly name: string; readonly age: number; readonly email: string; }

// Test 7: Record
type RecordType = Record<string, number>;
// Should resolve to: Record<string, number>

// Test 8: Pick
type UserWithName = Pick<User, 'name' | 'email'>;
// Should resolve to: { name: string; email: string; }

// Test 9: Omit
type UserWithoutAge = Omit<User, 'age'>;
// Should resolve to: { name: string; email: string; }

// Test 10: Awaited (TypeScript 4.5+)
async function asyncFunc() {
  return 42;
}
type AwaitedResult = Awaited<ReturnType<typeof asyncFunc>>;
// Should resolve to: number
