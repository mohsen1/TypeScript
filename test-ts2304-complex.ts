// From differential report - constrained generic
function f<T extends string>(x: T) {
  const y: T = "hello";  // This should error in TSC but we report Cannot find name 'T'
  return y;
}

// Generic function with interface
interface Animal { name: string }
interface Dog extends Animal { breed: string }

function test<T extends Animal>(x: T): T {
    return x;
}
