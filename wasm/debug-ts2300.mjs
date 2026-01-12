import { ThinParser } from './pkg/wasm.js';
import { readFileSync } from 'fs';
import ts from 'typescript';

const code = readFileSync('/Users/claude/code/TypeScript-forge-4-track/tests/cases/conformance/async/es2017/awaitBinaryExpression/awaitBinaryExpression1_es2017.ts', 'utf-8');

// Run TSC
const sourceFile = ts.createSourceFile('test.ts', code, ts.ScriptTarget.ES2017, true);
const compilerOptions = { strict: true, target: ts.ScriptTarget.ES2017, module: ts.ModuleKind.ESNext, noEmit: true, skipLibCheck: true };
const host = ts.createCompilerHost(compilerOptions);
const origGetSourceFile = host.getSourceFile;
host.getSourceFile = (name) => name === 'test.ts' ? sourceFile : origGetSourceFile.call(host, name);
const program = ts.createProgram(['test.ts'], compilerOptions, host);

const tscDiags = [
  ...program.getSyntacticDiagnostics(sourceFile),
  ...program.getSemanticDiagnostics(sourceFile)
];

console.log('=== TSC Diagnostics ===');
tscDiags.forEach(d => {
  const line = ts.getLineAndCharacterOfPosition(sourceFile, d.start).line + 1;
  console.log(`Line ${line}: TS${d.code}: ${ts.flattenDiagnosticMessageText(d.messageText, '\n')}`);
});

// Run WASM
const parser = new ThinParser('test.ts', code);
parser.parseSourceFile();
const wasmResult = JSON.parse(parser.checkSourceFile());

console.log('\n=== WASM Diagnostics ===');
wasmResult.diagnostics.forEach(d => {
  console.log(`Code ${d.code}: ${d.message_text}`);
});

console.log('\n=== Comparison ===');
const tscCodes = tscDiags.map(d => d.code);
const wasmCodes = wasmResult.diagnostics.map(d => d.code);
const missing = tscCodes.filter(c => !wasmCodes.includes(c));
const extra = wasmCodes.filter(c => !tscCodes.includes(c));
console.log('Missing in WASM:', missing);
console.log('Extra in WASM:', extra);
