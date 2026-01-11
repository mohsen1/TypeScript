const wasm = require('./wasm/pkg/wasm.js');

const code = `// @noImplicitAny: true
const f = (a) => a;`;

console.log('Test code:', code);

const parser = new wasm.ThinParser('test.ts', code);
parser.parseSourceFile();
const result = JSON.parse(parser.checkSourceFile());
console.log('Total diagnostics:', result.diagnostics.length);
result.diagnostics.forEach(d => {
  console.log(`  ${d.code}: ${d.message_text}`);
});
