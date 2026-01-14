const { ThinParser } = require('./pkg/wasm.js');

const code = `
import { MyPromise } from 'missing';

async function f3(): MyPromise<void> { }
`;

const parser = new ThinParser('test.ts', code);
parser.parseSourceFile();

// Get the return type somehow...
// Let me just check the diagnostics
const result = JSON.parse(parser.checkSourceFile());

console.log('All diagnostics:');
for (const diag of result.diagnostics || []) {
  console.log('  - ' + diag.code + ' at position ' + diag.start + ': ' + diag.message_text);
}

// Let me also try without the import
const code2 = `
declare class MyPromise<T> {}

async function f3(): MyPromise<void> { }
`;

const parser2 = new ThinParser('test2.ts', code2);
parser2.parseSourceFile();
const result2 = JSON.parse(parser2.checkSourceFile());

console.log('\\nTest 2 (with declare class):');
for (const diag of result2.diagnostics || []) {
  console.log('  - ' + diag.code + ': ' + diag.message_text);
}
