class A {}

abstract class B extends A {}

class C extends B {}

// Test 1: Abstract B to Concrete A - Should error
var AA: typeof A = B;

// Test 2: Concrete A to Abstract B - Should be OK
var BB: typeof B = A;

// Test 3: Abstract B to Concrete C - Should error
var CC: typeof C = B;
