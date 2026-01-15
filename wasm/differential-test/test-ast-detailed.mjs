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

// Get the raw node data
const nodeData = parser.getNodes();
const nodes = JSON.parse(nodeData);

console.log('=== All Nodes ===');
nodes.forEach((node, i) => {
  console.log(`[${i}] kind=${node.kind} (${node.syntaxKind || ''})`);
  if (node.name) console.log(`    name: ${node.name}`);
  if (node.initializer) console.log(`    initializer: ${node.initializer}`);
  if (node.elements) console.log(`    elements: ${JSON.stringify(node.elements)}`);
});

parser.free();
