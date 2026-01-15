#!/usr/bin/env node
/**
 * Quick test for TS2565 - Property used before being assigned
 */

import { fileURLToPath } from 'url';
import { resolve, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = resolve(__filename, '..');

async function testTS2565() {
  const wasm = await import(join(__dirname, 'pkg/wasm.js'));

  const testCases = [
    {
      name: 'Property accessed before assignment - should error',
      code: `
class C10 {
    a: number;
    b: number;
    #d: number;
    constructor() {
        let x = this.a;  // Error TS2565
        this.a = this.b;  // Error TS2565
        this.b = this.#d //Error TS2565
        this.b = x;
        this.#d = x;
    }
}
      `,
      expect2565: 3,
    },
    {
      name: 'Property accessed after assignment - should NOT error',
      code: `
class C11 {
    a: number;
    constructor() {
        this.a = 0;
        let x = this.a;  // OK - assigned first
    }
}
      `,
      expect2565: 0,
    },
  ];

  console.log('Testing TS2565 (property used before being assigned)\n');

  for (const test of testCases) {
    console.log(`Test: ${test.name}`);
    try {
      const parser = new wasm.ThinParser('test.ts', test.code.trim());
      parser.parseSourceFile();
      const checkResult = JSON.parse(parser.checkSourceFile());

      const ts2565Errors = (checkResult.diagnostics || []).filter(d => d.code === 2565);
      const pass = ts2565Errors.length === test.expect2565;

      console.log(`  Expected: ${test.expect2565} TS2565 errors`);
      console.log(`  Got: ${ts2565Errors.length} TS2565 errors`);
      console.log(`  Result: ${pass ? '✅ PASS' : '❌ FAIL'}`);

      if (ts2565Errors.length > 0) {
        console.log('  Errors:');
        ts2565Errors.forEach(err => {
          console.log(`    - ${err.message_text}`);
        });
      }
    } catch (e) {
      console.log(`  Result: ❌ ERROR: ${e.message}`);
    }
    console.log();
  }
}

testTS2565().catch(err => {
  console.error('Test failed:', err);
  process.exit(1);
});
