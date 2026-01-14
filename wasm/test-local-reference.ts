// Test file for local reference resolution (TS2304 errors)
// Testing basic local variable resolution

// Test 1: Function-scoped var
function test1() {
  var x = 10;
  return x; // Should resolve x
}

// Test 2: Block-scoped let
function test2() {
  if (true) {
    let y = 20;
    return y; // Should resolve y
  }
  return y; // Should error (out of scope)
}

// Test 3: Block-scoped const
function test3() {
  {
    const z = 30;
    console.log(z); // Should resolve z
  }
}

// Test 4: Function parameters
function test4(a: number, b: string) {
  return a + b; // Should resolve a and b
}

// Test 5: Nested functions
function test5() {
  var outer = 10;
  function inner() {
    return outer; // Should resolve outer (closure)
  }
  return inner();
}

// Test 6: Loop variables
function test6() {
  for (let i = 0; i < 10; i++) {
    console.log(i); // Should resolve i
  }
}

// Test 7: Multiple variables in same scope
function test7() {
  let a = 1;
  let b = 2;
  let c = 3;
  return a + b + c; // Should resolve all three
}

// Test 8: Shadowing
function test8() {
  let x = 1;
  {
    let x = 2;
    return x; // Should resolve inner x
  }
}
