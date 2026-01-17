// Simple TS2322 test cases

// Should NOT emit TS2322
const valid1: string = "hello";
const valid2: number = 42;

// Should emit TS2322
const invalid1: string = 42;
const invalid2: number = "hello";

// Subtyping should work
interface Base { x: number; }
interface Extended extends Base { y: number; }

const base: Base = { x: 1 };
const extended: Extended = { x: 1, y: 2 };

// Should NOT emit TS2322 - subtype to supertype
const validSubtype: Base = extended;

// Should emit TS2322 - supertype to subtype (missing property y)
const invalidSubtype: Extended = base;
