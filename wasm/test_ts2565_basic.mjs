#!/usr/bin/env node
/**
 * Basic test for TS2565
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
    constructor() {
        let x = this.a;
    }
}
  `;

  console.log('Testing TS2565: Property accessed before assignment\n');

  const parser = new wasm.ThinParser('test.ts', code.trim());
  parser.parseSourceFile();
  const checkResult = JSON.parse(parser.checkSourceFile());

  console.log('All diagnostics:');
  (checkResult.diagnostics || []).forEach(err => {
    console.log(`  [${err.code}] ${err.message_text}`);
  });

  const ts2564Errors = (checkResult.diagnostics || []).filter(d => d.code === 2564);
  const ts2565Errors = (checkResult.diagnostics || []).filter(d => d.code === 2565);

  console.log(`\nTS2564 errors: ${ts2564Errors.length}`);
  console.log(`TS2565 errors: ${ts2565Errors.length}`);

  if (ts2565Errors.length === 0) {
    console.log('\n⚠️  TS2565 not detected - check implementation');
  }
}

testTS2565().catch(err => {
  console.error('Test failed:', err);
  process.exit(1);
});
