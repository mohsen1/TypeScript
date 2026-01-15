#!/usr/bin/env node
/**
 * Debug TS2565 - verify tracked properties
 */

import { fileURLToPath } from 'url';
import { resolve, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = resolve(__filename, '..');

async function testTS2565() {
  const wasm = await import(join(__dirname, 'pkg/wasm.js'));

  const code = `
class C {
    a: number;
    b: number;
    constructor() {
        let x = this.a;  // Should error TS2565
        this.b = 0;     // b is assigned, should NOT error TS2564
    }
}
  `;

  console.log('Testing TS2565 with debug info\n');

  const parser = new wasm.ThinParser('test.ts', code.trim());
  parser.parseSourceFile();
  const checkResult = JSON.parse(parser.checkSourceFile());

  console.log('All diagnostics:');
  (checkResult.diagnostics || []).forEach(err => {
    console.log(`  [${err.code}] ${err.message_text}`);
  });

  const ts2564Errors = (checkResult.diagnostics || []).filter(d => d.code === 2564);
  const ts2565Errors = (checkResult.diagnostics || []).filter(d => d.code === 2565);

  console.log(`\nExpected:`);
  console.log(`  - TS2564 for 'a' (no initializer)`);
  console.log(`  - TS2565 for 'a' (used before being assigned)`);
  console.log(`\nGot:`);
  console.log(`  - ${ts2564Errors.length} TS2564 errors`);
  console.log(`  - ${ts2565Errors.length} TS2565 errors`);
}

testTS2565().catch(err => {
  console.error('Test failed:', err);
  process.exit(1);
});
