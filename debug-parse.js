const wasm = require('./wasm/pkg/wasm.js');

const code = 'var foo = async (a = await => await): Promise<void> => {\n}';
console.log('Code:', code);

const parser = new wasm.ThinParser('test.ts', code);
const root = parser.parseSourceFile();
console.log('\nParse tree:', JSON.stringify(root, null, 2));
