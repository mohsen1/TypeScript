#!/usr/bin/env node
/**
 * Test a specific file
 */
import { readFileSync } from 'fs';
import { createRequire } from 'module';
const require = createRequire(import.meta.url);

const wasm = require('../pkg/wasm.js');

const source = readFileSync('../tests/cases/compiler/allowImportClausesToMergeWithTypes.ts', 'utf-8');
console.log('Source:');
console.log(source);
console.log('\n==================\n');

const parser = wasm.createThinParser('test.ts', source);
parser.parseSourceFile();
const diagnosticsJson = parser.getDiagnosticsJson();
const diagnostics = JSON.parse(diagnosticsJson);

console.log('Parser diagnostics:');
if (diagnostics) {
    diagnostics.forEach(d => {
        console.log(`  ${d.code}: ${d.message} at ${d.start}-${d.start + d.length}`);
    });
} else {
    console.log('  (none)');
}

parser.bindSourceFile();
const checkJson = parser.checkSourceFile();
const checkResult = JSON.parse(checkJson);

console.log('\nChecker diagnostics:');
if (checkResult.diagnostics) {
    checkResult.diagnostics.forEach(d => {
        console.log(`  ${d.code}: ${d.message} at ${d.start}-${d.start + d.length}`);
    });
} else {
    console.log('  (none)');
}

parser.free();
