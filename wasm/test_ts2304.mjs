import { createRequire } from 'module';
const require = createRequire(import.meta.url);
const { ThinParser } = require('./pkg/wasm.js');
import { readFileSync } from 'fs';

const libSource = readFileSync('./tests/lib/lib.d.ts', 'utf8');

const testCases = [
  { code: 'console.log("test");', name: 'console usage' },
  { code: 'const arr = new Array();', name: 'Array constructor' },
  { code: 'const obj = new Object();', name: 'Object constructor' },
  { code: 'const x: Promise<string> = Promise.resolve("test");', name: 'Promise type' },
  { code: 'const map = new Map<string, number>();', name: 'Map type' },
  { code: 'const set = new Set<number>();', name: 'Set type' },
  { code: 'const num = parseInt("123");', name: 'parseInt' },
  { code: 'const encoded = encodeURIComponent("test");', name: 'encodeURIComponent' },
  { code: 'declare global { interface Window { custom: string; } }', name: 'declare global' },
  { code: 'function foo<T>(x: T): T { return x; }', name: 'type parameter' },
];

let totalErrors = 0;
let ts2304Errors = 0;

for (const test of testCases) {
  try {
    const parser = new ThinParser('test.ts', test.code);
    parser.addLibFile('lib.d.ts', libSource);
    parser.parseSourceFile();
    parser.bindSourceFile();
    const checkResult = JSON.parse(parser.checkSourceFile());
    const errors = checkResult.diagnostics || [];
    const ts2304 = errors.filter(e => e.code === 2304);
    
    if (errors.length > 0) {
      console.log('\n[' + test.name + ']');
      console.log('  Total errors: ' + errors.length);
      if (ts2304.length > 0) {
        console.log('  TS2304 errors: ' + ts2304.length);
        ts2304.forEach(e => console.log('    - ' + e.message_text));
        ts2304Errors += ts2304.length;
      }
      totalErrors += errors.length;
    }
    
    parser.free();
  } catch (e) {
    console.log('\n[' + test.name + '] ERROR: ' + e.message);
  }
}

console.log('\n\n=== SUMMARY ===');
console.log('Total tests: ' + testCases.length);
console.log('Tests with errors: ' + (totalErrors > 0 ? 'YES' : 'NO'));
console.log('Total TS2304 errors: ' + ts2304Errors);
