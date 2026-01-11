// @strict: true

function f1<T extends string | undefined>(x: T, y: { a: T }, z: [T]): string {
    if (x) {
        x;
        x.length;
        return x;
    }
    if (y.a) {
        y.a.length;
        return y.a;
    }
    if (z[0]) {
        z[0].length;
        return z[0];
    }
    return "hello";
}
