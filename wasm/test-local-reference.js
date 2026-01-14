function test1() {
    var x = 10;
    return x;
}
function test2() {
    if (true) {
        let y = 20;
        return y;
    }
    return y;
}
function test3() {
    {
        const z = 30;
        console.log(z);
    }
}
function test4(a, b) {
    return a + b;
}
function test5() {
    var outer = 10;
    function inner() {
        return outer;
    }
    return inner();
}
function test6() {
    for (let i = 0; i < 10; i++) {
        console.log(i);
    }
}
function test7() {
    let a = 1;
    let b = 2;
    let c = 3;
    return a + b + c;
}
function test8() {
    let x = 1;
    {
        let x = 2;
        return x;
    }
}
