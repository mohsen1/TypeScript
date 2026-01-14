const { ThinParser } = require('./pkg/wasm.js');
const { readFileSync } = require('fs');

const code = readFileSync('/Users/claude/code/TypeScript-anvil-2-track/tests/cases/conformance/async/es2017/asyncAwaitIsolatedModules_es2017.ts', 'utf8');

const parser = new ThinParser('test.ts', code);
parser.parseSourceFile();
const result = JSON.parse(parser.checkSourceFile());

const ts2355 = (result.diagnostics || []).filter(d => d.code === 2355);
console.log(`Found ${ts2355.length} TS2355 errors:`);
for (const diag of ts2355) {
  console.log(`  - ${diag.message_text}`);
}
