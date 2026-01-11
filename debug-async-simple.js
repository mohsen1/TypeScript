const wasm = require('./wasm/pkg/wasm.js');

const tests = [
  `// @noImplicitAny: true
const f = async (a) => a;`,
  `// @noImplicitAny: true
async function f(a) { }`,
  `// @noImplicitAny: true
function f(a = () => {}) { }`,
];

tests.forEach((code, i) => {
  console.log(`\nTest ${i+1}:`, code.split('\n')[1]);
  const parser = new wasm.ThinParser('test.ts', code);
  parser.parseSourceFile();
  const result = JSON.parse(parser.checkSourceFile());
  const ts7006 = result.diagnostics.filter(d => d.code === 7006);
  console.log('  TS7006 count:', ts7006.length);
  ts7006.forEach(d => console.log(`    ${d.message_text}`));
});
