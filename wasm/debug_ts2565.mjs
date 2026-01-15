#!/usr/bin/env node
/**
 * Debug TS2565 check
 */

import { fileURLToPath } from 'url';
import { resolve, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = resolve(__filename, '..');

async function testTS2565() {
  const wasm = await import(join(__dirname, 'pkg/wasm.js'));

  const code = `
// @strict: true
// @strictPropertyInitialization: true

class C10 {
    a: number;
    b: number;
    #d: number;
    constructor() {
        let x = this.a;  // Error TS2565 - should error
        this.a = this.b;  // Error TS2565 - should error
        this.b = this.#d //Error TS2565 - should error
        this.b = x;
        this.#d = x;
    }
}
  `;

  console.log('Testing TS2565 with strict mode enabled\n');

  try {
    const parser = new wasm.ThinParser('test.ts', code.trim());
    parser.parseSourceFile();
    const checkResult = JSON.parse(parser.checkSourceFile());

    console.log('All diagnostics:');
    (checkResult.diagnostics || []).forEach(err => {
      console.log(`  [${err.code}] ${err.message_text}`);
    });

    const ts2565Errors = (checkResult.diagnostics || []).filter(d => d.code === 2565);
    const ts2564Errors = (checkResult.diagnostics || []).filter(d => d.code === 2564);

    console.log(`\nTS2564 errors: ${ts2564Errors.length}`);
    console.log(`TS2565 errors: ${ts2565Errors.length}`);
  } catch (e) {
    console.error('Error:', e.message);
  }
}

testTS2565().catch(err => {
  console.error('Test failed:', err);
  process.exit(1);
});
