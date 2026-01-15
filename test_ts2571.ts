// Test cases for TS2571 vs TS2683
// These should emit TS2683, not TS2571

// Case 1: Regular function using `this`
function regularFunction() {
    this.prop;  // Should be TS2683, not TS2571
}

// Case 2: Function expression using `this`
const fnExpr = function() {
    this.prop;  // Should be TS2683, not TS2571
};

// Case 3: Nested regular function
class MyClass {
    method() {
        function nested() {
            this.prop;  // Should be TS2683, not TS2571
        }
    }
}

// Case 4: Callback with `this`
function doSomething(callback: () => void) {
    callback();
}

doSomething(function() {
    this.prop;  // Should be TS2683, not TS2571
});

// Case 5: Object method with regular function
const obj = {
    method: function() {
        this.prop;  // Should be valid (obj has context)
    }
};

// Case 6: Method returning a function that uses `this`
class Example {
    value = 42;

    getFunction() {
        return function() {
            return this.value;  // Should be TS2683, not TS2571
        };
    }
}
