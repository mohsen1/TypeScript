// Test file for primitive types in class extends clauses
// Should NOT emit TS2304 errors

// Test 1: number
class C1 extends number { }

// Test 2: string
class C2 extends string { }

// Test 3: boolean
class C3 extends boolean { }

// Test 4: void
class C4 extends void { }

// Test 5: null
class C5 extends null { }

// Test 6: undefined
class C6 extends undefined { }

// Test 7: never
class C7 extends never { }

// Test 8: unknown
class C8 extends unknown { }

// Test 9: any
class C9 extends any { }

// Test 10: Valid extends (should still work)
interface Base { }
class Valid extends Base { }
