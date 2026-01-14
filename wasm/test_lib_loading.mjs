import { createRequire } from 'module';
const require = createRequire(import.meta.url);
const { ThinParser } = require('./pkg/wasm.js');
import { readFileSync } from 'fs';

const libSource = readFileSync('./tests/lib/lib.d.ts', 'utf8');

// Test 1: Basic lib loading
const parser1 = new ThinParser('test.ts', 'console.log("hello");');
parser1.addLibFile('lib.d.ts', libSource);
parser1.parseSourceFile();
const bindResult = JSON.parse(parser1.bindSourceFile());
console.log('Test 1: Lib symbols in file_locals:', Object.keys(bindResult.symbols || {}).slice(0, 10).join(', '));

// Test 2: Check that lib symbols are actually available
const hasConsole = bindResult.symbols && bindResult.symbols.console !== undefined;
const hasObject = bindResult.symbols && bindResult.symbols.Object !== undefined;
const hasArray = bindResult.symbols && bindResult.symbols.Array !== undefined;
const hasPromise = bindResult.symbols && bindResult.symbols.Promise !== undefined;
console.log('Test 2: Lib symbols available:', { console: hasConsole, Object: hasObject, Array: hasArray, Promise: hasPromise });

// Test 3: Type check to see if TS2304 errors are produced
const checkResult = JSON.parse(parser1.checkSourceFile());
const ts2304Errors = checkResult.diagnostics.filter(d => d.code === 2304);
console.log('Test 3: TS2304 errors ("Cannot find name"):', ts2304Errors.length);
if (ts2304Errors.length > 0) {
  console.log('  TS2304 errors:', ts2304Errors.map(e => e.message_text).join(', '));
}

parser1.free();

// Test 4: Try a more complex example
const parser2 = new ThinParser('test2.ts', 'const x: Promise<string> = new Array();');
parser2.addLibFile('lib.d.ts', libSource);
parser2.parseSourceFile();
const bindResult2 = JSON.parse(parser2.bindSourceFile());
const checkResult2 = JSON.parse(parser2.checkSourceFile());
const ts2304Errors2 = checkResult2.diagnostics.filter(d => d.code === 2304);
console.log('Test 4: Complex example TS2304 errors:', ts2304Errors2.length);
if (ts2304Errors2.length > 0) {
  console.log('  TS2304 errors:', ts2304Errors2.map(e => e.message_text).join(', '));
}
parser2.free();
