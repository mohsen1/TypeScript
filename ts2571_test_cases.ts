// Test cases for TS2571 vs TS2683 investigation
// Run with: tsc --noEmit ts2571_test_cases.ts

// ============================================================
// CASE 1: Regular function using `this` (should emit TS2683)
// ============================================================

function regularFunction1() {
    return this.bar;  // Expected: TS2683 - "'this' implicitly has type 'any'"
}

function regularFunction2(param: string) {
    console.log(this.x);  // Expected: TS2683 - "'this' implicitly has type 'any'"
    return param;
}

// ============================================================
// CASE 2: Arrow function in object literal (edge case)
// ============================================================

const obj = {
    method: () => {
        return this.bar;  // Expected: TS2683 or type error (no outer `this` context)
    }
};

// ============================================================
// CASE 3: Callback using `this` (should emit TS2683)
// ============================================================

setTimeout(function() {
    console.log(this);  // Expected: TS2683 - "'this' implicitly has type 'any'"
}, 100);

array.forEach(function(item) {
    this.process(item);  // Expected: TS2683 - "'this' implicitly has type 'any'"
});

// ============================================================
// CASE 4: Arrow function should preserve outer `this`
// ============================================================

class MyClass {
    value = 42;

    method() {
        // Arrow function should inherit `this` from method
        const arrow = () => {
            return this.value;  // Expected: OK - this is MyClass
        };
        return arrow();
    }
}

// ============================================================
// CASE 5: Nested function in class method
// ============================================================

class MyClass2 {
    value = 42;

    method() {
        // Regular function creates new `this` context
        function nested() {
            return this.value;  // Expected: TS2683 - "'this' implicitly has type 'any'"
        }
        return nested();
    }

    method2() {
        // Arrow function preserves `this`
        const arrow = () => {
            return this.value;  // Expected: OK - this is MyClass2
        };
        return arrow();
    }
}

// ============================================================
// CASE 6: Object literal methods
// ============================================================

const literal = {
    value: 42,
    method() {
        return this.value;  // Expected: OK - inferred `this` is the literal type
    },

    arrowMethod: () => {
        return this.value;  // Expected: TS2683 or error (arrow captures global scope)
    }
};

// ============================================================
// CASE 7: Function with explicit `this` parameter
// ============================================================

function withExplicitThis(this: { value: number }) {
    return this.value;  // Expected: OK - `this` is explicitly typed
}

// ============================================================
// CASE 8: Bound function
// ============================================================

class MyClass3 {
    value = 42;

    constructor() {
        const bound = function() {
            return this.value;  // Expected: TS2683 (no bind here yet)
        };

        const bound2 = function(this: MyClass3) {
            return this.value;  // Expected: OK - explicit `this` type
        };
    }
}

// ============================================================
// CASE 9: Property access on unknown (TS2571 case)
// ============================================================

function takesUnknown(obj: unknown) {
    return obj.property;  // Expected: TS2571 - "Object is of type 'unknown'"
}

// ============================================================
// CASE 10: Callback in array method
// ============================================================

const arr = [1, 2, 3];
arr.filter(function(item) {
    return this.predicate(item);  // Expected: TS2683 - "'this' implicitly has type 'any'"
});

arr.filter((item) => {
    return this.predicate(item);  // Expected: TS2683 or error (arrow in global scope)
});
