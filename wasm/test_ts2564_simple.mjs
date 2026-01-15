#!/usr/bin/env node
/**
 * Test TS2564 against the official baseline file
 */

import { fileURLToPath } from 'url';
import { resolve, join } from 'path';
import { readFileSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = resolve(__filename, '..');

async function testTS2564Baseline() {
  const wasm = await import(join(__dirname, 'pkg/wasm.js'));

  // Read the baseline errors file
  const baselineFile = resolve(__dirname, '../../../tests/baselines/reference/strictPropertyInitialization.errors.txt');
  const testFile = resolve(__dirname, '../../../tests/cases/conformance/classes/propertyMemberDeclarations/strictPropertyInitialization.ts');

  let baselineContent;
  let code;

  try {
    baselineContent = readFileSync(baselineFile, 'utf-8');
    code = readFileSync(testFile, 'utf-8');
  } catch (e) {
    console.error('Could not read files:', e.message);
    return;
  }

  // Parse baseline to count expected TS2564 errors
  const expected2564 = (baselineContent.match(/error TS2564:/g) || []).length;
  const expected2565 = (baselineContent.match(/error TS2565:/g) || []).length;

  console.log('Testing strictPropertyInitialization.ts\n');
  console.log(`Expected (from baseline):`);
  console.log(`  TS2564 errors: ${expected2564}`);
  console.log(`  TS2565 errors: ${expected2565}`);

  // Run WASM
  const wasmParser = new wasm.ThinParser(testFile, code);
  wasmParser.parseSourceFile();
  const wasmResult = JSON.parse(wasmParser.checkSourceFile());

  const wasm2564 = (wasmResult.diagnostics || []).filter(d => d.code === 2564);
  const wasm2565 = (wasmResult.diagnostics || []).filter(d => d.code === 2565);

  console.log(`\nWASM produced:`);
  console.log(`  TS2564 errors: ${wasm2564.length}`);
  console.log(`  TS2565 errors: ${wasm2565.length}`);

  const match2564 = wasm2564.length === expected2564;
  const match2565 = wasm2565.length === expected2565;

  console.log(`\nResults:`);
  console.log(`  TS2564: ${match2564 ? '✅ PASS' : '❌ FAIL'} (${wasm2564.length}/${expected2564})`);
  console.log(`  TS2565: ${match2565 ? '✅ PASS' : '❌ FAIL'} (${wasm2565.length}/${expected2565})`);

  if (wasm2564.length !== expected2564) {
    console.log(`\n  Missing TS2564 errors: ${expected2564 - wasm2564.length}`);
  }
  if (wasm2565.length !== expected2565) {
    console.log(`  Missing TS2565 errors: ${expected2565 - wasm2565.length}`);
  }

  return {
    match2564,
    match2565,
    expected2564,
    wasm2564: wasm2564.length,
    expected2565,
    wasm2565: wasm2565.length,
  };
}

testTS2564Baseline().catch(err => {
  console.error('Test failed:', err);
  process.exit(1);
});
