const wasm = require('./wasm/pkg/wasm.js');

const code = '// @noImplicitAny: true\nvar foo = async (a = await => await): Promise<void> => {\n}';
console.log('Test code:', code);

try {
  const parser = new wasm.ThinParser('test.ts', code);
  parser.parseSourceFile();
  console.log('Parse successful');
  const result = JSON.parse(parser.checkSourceFile());
  console.log('Check successful');
  console.log('Total diagnostics:', result.diagnostics.length);
  result.diagnostics.forEach(d => {
    console.log(`  ${d.code}: ${d.message_text}`);
  });
} catch (e) {
  console.log('Error:', e.message);
}
