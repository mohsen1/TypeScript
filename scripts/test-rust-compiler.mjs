#!/usr/bin/env node
/**
 * Test runner for Rust/WASM TypeScript compiler against tests/cases
 *
 * Usage:
 *   node scripts/test-rust-compiler.mjs [test-file]
 *   node scripts/test-rust-compiler.mjs tests/cases/compiler/2dArrays.ts
 */

import { execSync } from 'child_process';
import { readFileSync, existsSync } from 'fs';
import { basename, dirname, join } from 'path';

const testFile = process.argv[2];

if (!testFile) {
    console.log('Usage: node scripts/test-rust-compiler.mjs <test-file>');
    console.log('Example: node scripts/test-rust-compiler.mjs tests/cases/compiler/2dArrays.ts');
    process.exit(1);
}

if (!existsSync(testFile)) {
    console.error(`File not found: ${testFile}`);
    process.exit(1);
}

const source = readFileSync(testFile, 'utf-8');
const testName = basename(testFile, '.ts');

console.log(`\n=== Testing: ${testName} ===\n`);
console.log('Source:');
console.log(source);
console.log('\n---\n');

// Check for baseline files
const baselineDir = 'tests/baselines/reference';
const jsBaseline = join(baselineDir, `${testName}.js`);
const typesBaseline = join(baselineDir, `${testName}.types`);
const errorsBaseline = join(baselineDir, `${testName}.errors.txt`);

console.log('Baseline files:');
console.log(`  .js:     ${existsSync(jsBaseline) ? '✓' : '✗'}`);
console.log(`  .types:  ${existsSync(typesBaseline) ? '✓' : '✗'}`);
console.log(`  .errors: ${existsSync(errorsBaseline) ? '✓' : '✗'}`);

// TODO: Once WASM is built, we can call the Rust compiler here
// For now, just report what would need to be tested

console.log('\n--- Next Steps ---');
console.log('1. Build WASM: wasm-pack build wasm --target nodejs');
console.log('2. Import and call Rust compiler');
console.log('3. Compare output against baselines');
