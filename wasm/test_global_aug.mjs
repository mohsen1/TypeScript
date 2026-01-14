import { createRequire } from 'module';
const require = createRequire(import.meta.url);
const { ThinParser } = require('./pkg/wasm.js');
import { readFileSync } from 'fs';

const libSource = readFileSync('./tests/lib/lib.d.ts', 'utf8');

// Test case: declare global augmenting Window interface
const testCode = `
declare global {
  interface Window {
    customProperty: string;
    customMethod(): void;
  }
}

// Now try to use the augmented property
const win: Window = window;
win.customProperty = 'test';
win.customMethod();
`;

const parser = new ThinParser('test.ts', testCode);
parser.addLibFile('lib.d.ts', libSource);
parser.parseSourceFile();
parser.bindSourceFile();

// Check if global augmentations were tracked
const bindResult = JSON.parse(parser.bindSourceFile());
console.log('Bind result symbols:', Object.keys(bindResult.symbols || {}).slice(0, 20).join(', '));

const checkResult = JSON.parse(parser.checkSourceFile());
const errors = checkResult.diagnostics || [];
const ts2339 = errors.filter(e => e.code === 2339); // Property does not exist on 'Window'

console.log('\nTotal errors:', errors.length);
console.log('TS2339 errors (property not found):', ts2339.length);
if (ts2339.length > 0) {
  console.log('TS2339 errors:');
  ts2339.forEach(e => console.log('  - ' + e.message_text));
}

parser.free();
