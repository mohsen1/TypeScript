import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const wasm = require(resolve(__dirname, '../pkg'));

const code = `function f(await = await) {
}`;
const fileName = 'test.ts';

const parser = new wasm.ThinParser(fileName, code);
parser.parseSourceFile();
const ast = JSON.parse(parser.getAstJson());

console.log('=== AST ===');
console.log(JSON.stringify(ast, null, 2));
parser.free();
