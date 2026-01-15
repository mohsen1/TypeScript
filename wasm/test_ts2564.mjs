#!/usr/bin/env node
/**
 * Quick test for TS2564 - Property initialization check
 */

import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { resolve } from 'path';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = resolve(__filename, '..');

async function testTS2564() {
  const wasm = await import(resolve(__dirname, 'pkg/wasm.js'));

  const testCases = [
    {
      name: 'Property without initializer - should error',
      code: `
class C1 {
    a: number;  // Should error: TS2564
    b: number | undefined;  // Should NOT error (includes undefined)
}
      `,
      expectErrors: 1,
    },
    {
      name: 'Property with initializer - should NOT error',
      code: `
class C2 {
    a = 0;
    b: number = 0;
}
      `,
      expectErrors: 0,
    },
    {
      name: 'Property assigned in constructor - should NOT error',
      code: `
class C3 {
    a: number;
    constructor() {
        this.a = 0;
    }
}
      `,
      expectErrors: 0,
    },
    {
      name: 'Property not definitely assigned - should error',
      code: `
class C4 {
    a: number;  // Should error: TS2564
    constructor(cond: boolean) {
        if (cond) {
            return;
        }
        this.a = 0;
    }
}
      `,
      expectErrors: 1,
    },
  ];

  console.log('Testing TS2564 (strict property initialization)\n');

  for (const test of testCases) {
    console.log(`Test: ${test.name}`);
    try {
      const parser = new wasm.ThinParser('test.ts', test.code.trim());
      parser.parseSourceFile();
      const checkResult = JSON.parse(parser.checkSourceFile());

      const ts2564Errors = (checkResult.diagnostics || []).filter(d => d.code === 2564);
      const pass = ts2564Errors.length === test.expectErrors;

      console.log(`  Expected: ${test.expectErrors} TS2564 errors`);
      console.log(`  Got: ${ts2564Errors.length} TS2564 errors`);
      console.log(`  Result: ${pass ? '✅ PASS' : '❌ FAIL'}`);

      if (ts2564Errors.length > 0) {
        console.log('  Errors:');
        ts2564Errors.forEach(err => {
          console.log(`    - ${err.message_text}`);
        });
      }
    } catch (e) {
      console.log(`  Result: ❌ ERROR: ${e.message}`);
    }
    console.log();
  }
}

testTS2564().catch(err => {
  console.error('Test failed:', err);
  process.exit(1);
});
