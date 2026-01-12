import { ThinParser } from './pkg/wasm.js';
import { readFileSync } from 'fs';

const code = readFileSync('./wasm/test_7011_debug.ts', 'utf-8');
const parser = new ThinParser('wasm/test_7011_debug.ts', code);
parser.parseSourceFile();
const result = JSON.parse(parser.checkSourceFile());
console.log('Diagnostics:', JSON.stringify(result.diagnostics, null, 2));