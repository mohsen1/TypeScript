import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const wasm = require(resolve(__dirname, '../pkg'));
const ts = require('typescript');

// Test 1: await as parameter name in non-async function
const test1 = `function f(await = await) {
}`;

console.log('=== Test 1: await as parameter name ===');
const fileName = 'test.ts';

// TSC
const tsc1 = ts.createSourceFile(fileName, test1, ts.ScriptTarget.Latest, true);
console.log('TSC diagnostics:');
tsc1.parseDiagnostics.forEach(d => console.log(`  ${d.messageText} at ${d.start}`));

// WASM
const parser1 = new wasm.ThinParser(fileName, test1);
parser1.parseSourceFile();
const diags1 = JSON.parse(parser1.getDiagnosticsJson());
console.log('WASM diagnostics:');
diags1.forEach(d => console.log(`  ${d.message} at ${d.span?.start}`));
parser1.free();

// Test 2: await as expression in async function
const test2 = `async function f() {
  await x;
}`;

console.log('\n=== Test 2: await in async function ===');
const tsc2 = ts.createSourceFile(fileName, test2, ts.ScriptTarget.Latest, true);
console.log('TSC diagnostics:');
tsc2.parseDiagnostics.forEach(d => console.log(`  ${d.messageText} at ${d.start}`));

const parser2 = new wasm.ThinParser(fileName, test2);
parser2.parseSourceFile();
const diags2 = JSON.parse(parser2.getDiagnosticsJson());
console.log('WASM diagnostics:');
diags2.forEach(d => console.log(`  ${d.message} at ${d.span?.start}`));
parser2.free();

// Test 3: await as variable reference in non-async function
const test3 = `const await = 1;
function f() {
  return await;
}`;

console.log('\n=== Test 3: await as variable name ===');
const tsc3 = ts.createSourceFile(fileName, test3, ts.ScriptTarget.Latest, true);
console.log('TSC diagnostics:');
tsc3.parseDiagnostics.forEach(d => console.log(`  ${d.messageText} at ${d.start}`));

const parser3 = new wasm.ThinParser(fileName, test3);
parser3.parseSourceFile();
const diags3 = JSON.parse(parser3.getDiagnosticsJson());
console.log('WASM diagnostics:');
diags3.forEach(d => console.log(`  ${d.message} at ${d.span?.start}`));
parser3.free();
