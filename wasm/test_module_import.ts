// Test file to verify module import resolution issue
// This should demonstrate TS7005/TS7008 errors

// === file1.ts ===
export const foo = 42;
export function bar() {
  return "hello";
}
export interface Baz {
  value: number;
}

// === file2.ts ===
import { foo, bar, Baz } from './file1';

// This should work - foo is exported from file1
const x = foo;

// This should work - bar is exported from file1
const y = bar();

// This should work - Baz is exported from file1
const z: Baz = { value: 123 };

// Expected behavior: No errors
// Current WASM behavior: TS7005 errors on foo, bar, Baz
// "Symbol 'foo' cannot be referenced from a module"
