const { ThinParser, TypeId } = require('./pkg/wasm.js');

// Test 1: Local Promise interface
const code1 = `
interface Promise<T> {}
async function f1(): Promise<void> { }
`;

// Test 2: Global Promise (no declaration)
const code2 = `
async function f1(): Promise<void> { }
`;

console.log('=== Test 1: Local Promise interface ===');
const parser1 = new ThinParser('test1.ts', code1);
parser1.parseSourceFile();
const result1 = JSON.parse(parser1.checkSourceFile());
console.log('Diagnostics:', result1.diagnostics?.map(d => ({ code: d.code, message: d.message_text })) || []);

console.log('\n=== Test 2: Global Promise ===');
const parser2 = new ThinParser('test2.ts', code2);
parser2.parseSourceFile();
const result2 = JSON.parse(parser2.checkSourceFile());
console.log('Diagnostics:', result2.diagnostics?.map(d => ({ code: d.code, message: d.message_text })) || []);
