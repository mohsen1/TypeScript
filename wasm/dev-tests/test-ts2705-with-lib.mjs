import { ThinParser } from './pkg/wasm.js';
import { readFileSync } from 'fs';

const code = `
var foo = async (): Promise<void> => {
};
`;

const parser = new ThinParser('test.ts', code);
parser.parseSourceFile();

// Load lib files
const typescriptLibPath = './orchestrator/node_modules/typescript/lib';
const libEs5 = readFileSync(`${typescriptLibPath}/lib.es5.d.ts`, 'utf-8');
const libEs2015Promise = readFileSync(`${typescriptLibPath}/lib.es2015.promise.d.ts`, 'utf-8');

parser.addLibFile('lib.es5.d.ts', libEs5);
parser.addLibFile('lib.es2015.promise.d.ts', libEs2015Promise);

const result = JSON.parse(parser.checkSourceFile());

console.log('=== Diagnostics with lib files ===');
result.diagnostics.forEach(d => {
  console.log(`Code ${d.code}: ${d.message_text}`);
});
console.log('\nNo TS2705?', result.diagnostics.every(d => d.code !== 2705));
