import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const wasm = require(resolve(__dirname, '../pkg'));
const ts = require('typescript');

const code = `// @target: es2017
// @noEmitHelpers: true
function f(await = await) {
}`;

const fileName = 'test.ts';

// TSC diagnostics
const tscResult = ts.createSourceFile(fileName, code, ts.ScriptTarget.Latest, true);
console.log('=== TSC Diagnostics ===');
tscResult.parseDiagnostics.forEach(d => {
  console.log(`  TS${d.code}: ${d.messageText} at ${d.start}`);
});
const tsc1005 = tscResult.parseDiagnostics.filter(d => d.code === 1005).length;
console.log(`Total TSC TS1005: ${tsc1005}`);

// WASM diagnostics
const parser = new wasm.ThinParser(fileName, code);
parser.parseSourceFile();
const wasmDiags = JSON.parse(parser.getDiagnosticsJson());
console.log('\n=== WASM Diagnostics ===');
wasmDiags.forEach(d => {
  console.log(`  TS${d.code}: ${d.message} at ${d.span?.start}`);
});
const wasm1005 = wasmDiags.filter(d => d.code === 1005).length;
console.log(`Total WASM TS1005: ${wasm1005}`);
parser.free();
