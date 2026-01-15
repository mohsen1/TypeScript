import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const wasm = require(resolve(__dirname, '../pkg'));
const ts = require('typescript');

// Test: await as parameter name in non-async function
const code = `function f(await = await) {
}`;

console.log('Code:', JSON.stringify(code));
console.log('\nCode positions:');
for (let i = 0; i < code.length; i++) {
  if (code[i] !== ' ') console.log(`  [${i}]: '${code[i]}'`);
}

const fileName = 'test.ts';

// TSC
const tsc = ts.createSourceFile(fileName, code, ts.ScriptTarget.Latest, true);
console.log('\n=== TSC ===');
console.log('parseDiagnostics:', tsc.parseDiagnostics);

// WASM
const parser = new wasm.ThinParser(fileName, code);
parser.parseSourceFile();
const diags = JSON.parse(parser.getDiagnosticsJson());
console.log('\n=== WASM ===');
console.log('All diagnostics:', JSON.stringify(diags, null, 2));
parser.free();
