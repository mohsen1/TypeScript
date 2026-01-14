const { ThinParser } = require('./pkg/wasm.js');
const { readFileSync } = require('fs');

const code = readFileSync('./test_await_yield.ts', 'utf8');
const parser = new ThinParser('./test_await_yield.ts', code);
parser.parseSourceFile();

const diagnostics = JSON.parse(parser.getDiagnostics());
const ts1109 = diagnostics.filter(d => d.code === 1109);

console.log('=== TS1109 in await/yield test ===');
console.log(`Total TS1109 errors: ${ts1109.length}`);
for (const diag of ts1109) {
  console.log(`[${diag.start}] ${diag.message_text}`);
}
