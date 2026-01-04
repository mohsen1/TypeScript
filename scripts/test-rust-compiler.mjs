#!/usr/bin/env node
/**
 * Test runner for Rust/WASM TypeScript compiler against tests/cases
 *
 * Usage:
 *   node scripts/test-rust-compiler.mjs [test-file]
 *   node scripts/test-rust-compiler.mjs tests/cases/compiler/2dArrays.ts
 */

import { readFileSync, existsSync } from 'fs';
import { basename, join } from 'path';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);

// Load WASM module
let wasm;
try {
    wasm = require('../wasm/pkg/wasm.js');
} catch (e) {
    console.error('Failed to load WASM module. Run: wasm-pack build wasm --target nodejs');
    console.error(e.message);
    process.exit(1);
}

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
console.log('');

// Run Rust compiler
console.log('=== Rust Compiler Output ===\n');

const startTime = performance.now();

// Parse
const parser = wasm.createParser(testFile, source);
const rootIdx = parser.parseSourceFile();
const parseTime = performance.now() - startTime;

console.log(`Parse time: ${parseTime.toFixed(2)}ms`);
console.log(`Node count: ${parser.getNodeCount()}`);

// Check for parse errors
const diagnosticsJson = parser.getDiagnosticsJson();
const diagnostics = JSON.parse(diagnosticsJson);
if (diagnostics.length > 0) {
    console.log(`\nParse errors (${diagnostics.length}):`);
    diagnostics.forEach((d, i) => {
        console.log(`  ${i + 1}. ${d.message} (${d.start}-${d.end})`);
    });
}

// Bind
const bindStart = performance.now();
const bindingJson = parser.bindSourceFile(rootIdx);
const bindTime = performance.now() - bindStart;
const binding = JSON.parse(bindingJson);

console.log(`\nBind time: ${bindTime.toFixed(2)}ms`);
console.log(`Symbols: ${Object.keys(binding).length}`);
if (Object.keys(binding).length > 0) {
    console.log('  ' + Object.keys(binding).slice(0, 10).join(', ') + (Object.keys(binding).length > 10 ? '...' : ''));
}

// Type check
const checkStart = performance.now();
const checkJson = parser.checkSourceFile();
const checkTime = performance.now() - checkStart;
const checkResult = JSON.parse(checkJson);

console.log(`\nCheck time: ${checkTime.toFixed(2)}ms`);
console.log(`Types created: ${checkResult.typeCount}`);
if (checkResult.diagnostics && checkResult.diagnostics.length > 0) {
    console.log(`Type errors (${checkResult.diagnostics.length}):`);
    checkResult.diagnostics.slice(0, 5).forEach((d, i) => {
        console.log(`  ${i + 1}. ${d.message}`);
    });
    if (checkResult.diagnostics.length > 5) {
        console.log(`  ... and ${checkResult.diagnostics.length - 5} more`);
    }
}

// Summary
const totalTime = performance.now() - startTime;
console.log(`\n=== Summary ===`);
console.log(`Total time: ${totalTime.toFixed(2)}ms`);
console.log(`Parse errors: ${diagnostics.length}`);
console.log(`Type errors: ${checkResult.diagnostics?.length || 0}`);

// Cleanup
parser.free();

// Compare with baselines (TODO)
console.log('\n=== Baseline Comparison (TODO) ===');
console.log('- Compare emitted JS against .js baseline');
console.log('- Compare types against .types baseline');
console.log('- Compare errors against .errors.txt baseline');
