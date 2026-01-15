// Test case for shorthand method with tuple parameter type inference
type FooMethod = {
  method(...args: [type: string, cb: (e: string) => void]): void;
}

let fooM: FooMethod = {
  method(type, cb) {
    // Error: Cannot find name 'type', 'cb'
    console.log(type);
    cb(type);
  }
};

// Test 2: Arrow function with tuple rest parameter
type BarMethod = {
  method(...args: [x: number, y: string]): void;
}

let barM: BarMethod = {
  method: (x, y) => {
    console.log(x, y);
  }
};

// Test 3: Function expression with tuple rest parameter
type BazMethod = {
  method(...args: [a: number, b: string, c: boolean]): void;
}

let bazM: BazMethod = {
  method: function(a, b, c) {
    console.log(a, b, c);
  }
};
