#!/usr/bin/env node
/**
 * Analyze which TS2564/TS2565 errors are missing
 */

import { fileURLToPath } from 'url';
import { resolve, join } from 'path';
import { readFileSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = resolve(__filename, '..');

async function analyzeMissingErrors() {
  const wasm = await import(join(__dirname, 'pkg/wasm.js'));

  const baselineFile = resolve(__dirname, '../../../tests/baselines/reference/strictPropertyInitialization.errors.txt');
  const testFile = resolve(__dirname, '../../../tests/cases/conformance/classes/propertyMemberDeclarations/strictPropertyInitialization.ts');

  const baselineContent = readFileSync(baselineFile, 'utf-8');
  const code = readFileSync(testFile, 'utf-8');

  // Parse baseline to get expected errors
  const expectedErrors = [];
  const lines = baselineContent.split('\n');
  for (const line of lines) {
    if (line.includes('error TS2564:')) {
      const match = line.match(/strictPropertyInitialization\.ts\((\d+),(\d+)\): error TS2564: (.+)/);
      if (match) {
        expectedErrors.push({ line: parseInt(match[1]), col: parseInt(match[2]), code: 2564, message: match[3] });
      }
    } else if (line.includes('error TS2565:')) {
      const match = line.match(/strictPropertyInitialization\.ts\((\d+),(\d+)\): error TS2565: (.+)/);
      if (match) {
        expectedErrors.push({ line: parseInt(match[1]), col: parseInt(match[2]), code: 2565, message: match[3] });
      }
    }
  }

  // Run WASM
  const wasmParser = new wasm.ThinParser(testFile, code);
  wasmParser.parseSourceFile();
  const wasmResult = JSON.parse(wasmParser.checkSourceFile());

  console.log('=== EXPECTED ERRORS (from baseline) ===\n');
  for (const err of expectedErrors) {
    console.log(`Line ${err.line}, Col ${err.col}: TS${err.code} - ${err.message}`);
  }

  console.log('\n=== WASM ERRORS ===\n');
  const wasmErrors = (wasmResult.diagnostics || []);
  for (const err of wasmErrors.filter(e => e.code === 2564 || e.code === 2565)) {
    console.log(`Line ${err.start?.line + 1 || '?'}: TS${err.code} - ${err.message_text}`);
  }

  // Find missing
  console.log('\n=== MISSING ERRORS ===\n');
  const wasm2564Count = wasmErrors.filter(e => e.code === 2564).length;
  const wasm2565Count = wasmErrors.filter(e => e.code === 2565).length;

  console.log(`Expected: ${expectedErrors.filter(e => e.code === 2564).length} TS2564, ${expectedErrors.filter(e => e.code === 2565).length} TS2565`);
  console.log(`Got: ${wasm2564Count} TS2564, ${wasm2565Count} TS2565`);

  if (wasm2564Count < expectedErrors.filter(e => e.code === 2564).length) {
    console.log(`\nMissing ${expectedErrors.filter(e => e.code === 2564).length - wasm2564Count} TS2564 errors`);
  }
  if (wasm2565Count < expectedErrors.filter(e => e.code === 2565).length) {
    console.log(`Missing ${expectedErrors.filter(e => e.code === 2565).length - wasm2565Count} TS2565 errors`);
  }

  return { expectedErrors, wasmErrors };
}

analyzeMissingErrors().catch(err => {
  console.error('Analysis failed:', err);
  process.exit(1);
});
