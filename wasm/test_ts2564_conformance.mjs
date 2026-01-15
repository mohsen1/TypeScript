#!/usr/bin/env node
/**
 * Test TS2564 against specific conformance test files
 */

import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { resolve, join } from 'path';
import { readFileSync } from 'fs';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = resolve(__filename, '..');

const ts = require('typescript');

async function testConformanceFile() {
  const wasm = await import(join(__dirname, 'pkg/wasm.js'));

  // Test the specific strictPropertyInitialization.ts file
  const testFile = resolve(__dirname, '../../../tests/cases/conformance/classes/propertyMemberDeclarations/strictPropertyInitialization.ts');

  let code;
  try {
    code = readFileSync(testFile, 'utf-8');
  } catch (e) {
    console.error('Could not read test file:', e.message);
    return;
  }

  console.log('Testing strictPropertyInitialization.ts\n');

  // Run TSC
  const tscOptions = {
    strict: true,
    strictPropertyInitialization: true,
    target: ts.ScriptTarget.ES2020,
    module: ts.ModuleKind.ESNext,
    noEmit: true,
  };
  const tscSourceFile = ts.createSourceFile(testFile, code, ts.ScriptTarget.ES2020, true);
  const tscHost = ts.createCompilerHost(tscOptions);
  tscHost.getSourceFile = (name) => {
    if (name === testFile) return tscSourceFile;
    return ts.sys.readFile(name);
  };
  const tscProgram = ts.createProgram([testFile], tscOptions, tscHost);
  const tscDiags = [
    ...tscProgram.getSyntacticDiagnostics(tscSourceFile),
    ...tscProgram.getSemanticDiagnostics(tscSourceFile),
  ];

  // Run WASM
  const wasmParser = new wasm.ThinParser(testFile, code);
  wasmParser.parseSourceFile();
  const wasmResult = JSON.parse(wasmParser.checkSourceFile());

  const tsc2564 = tscDiags.filter(d => d.code === 2564);
  const wasm2564 = (wasmResult.diagnostics || []).filter(d => d.code === 2564);

  console.log(`TSC TS2564 errors: ${tsc2564.length}`);
  console.log(`WASM TS2564 errors: ${wasm2564.length}`);
  console.log();

  if (wasm2564.length === tsc2564.length) {
    console.log('✅ Perfect match! Both have', tsc2564.length, 'TS2564 errors');
  } else {
    const diff = tsc2564.length - wasm2564.length;
    console.log(`❌ Mismatch: Missing ${Math.max(0, diff)} TS2564 errors`);
  }

  return { tsc2564: tsc2564.length, wasm2564: wasm2564.length };
}

testConformanceFile().catch(err => {
  console.error('Test failed:', err);
  process.exit(1);
});
