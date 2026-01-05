#!/usr/bin/env node
/**
 * Batch test runner for Rust/WASM TypeScript compiler
 * Tests multiple files and reports statistics
 *
 * Usage:
 *   node scripts/batch-test-rust.mjs [limit]
 *   node scripts/batch-test-rust.mjs 100   # Test first 100 files
 */

import { readFileSync, existsSync, readdirSync } from 'fs';
import { basename, join } from 'path';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);

// Load WASM module
let wasm;
try {
    wasm = require('../wasm/pkg/wasm.js');
} catch (e) {
    console.error('Failed to load WASM module. Run: wasm-pack build wasm --target nodejs');
    process.exit(1);
}

const limit = parseInt(process.argv[2]) || 50;
const testDir = process.argv[3] || 'tests/cases/compiler';

// Get test files recursively
function getFiles(dir, files = []) {
    const entries = readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
        const path = join(dir, entry.name);
        if (entry.isDirectory()) {
            getFiles(path, files);
        } else if (entry.name.endsWith('.ts')) {
            files.push(path);
        }
    }
    return files;
}

const allFiles = getFiles(testDir);
const files = allFiles.slice(0, limit);

console.log(`\n=== Batch Testing ${files.length} files ===\n`);

let passed = 0;
let failed = 0;
let parseErrors = 0;
let typeErrors = 0;
let totalParseTime = 0;
let totalBindTime = 0;
let totalCheckTime = 0;

const failures = [];

let skipped = 0;

for (const file of files) {
    // Skip very large files
    const source = readFileSync(file, 'utf-8');
    if (source.length > 50000) {
        skipped++;
        continue;
    }

    // Skip multi-file tests (have @filename: directive)
    const sourceLower = source.toLowerCase();
    if (sourceLower.includes('@filename:') || sourceLower.includes('// @filename')) {
        skipped++;
        continue;
    }

    const testName = basename(file, '.ts');

    try {
        // Use ThinParser if available, otherwise fall back to legacy parser
        const useThinParser = wasm.createThinParser !== undefined;
        const parser = useThinParser
            ? wasm.createThinParser(file, source)
            : wasm.createParser(file, source);

        // Parse
        const parseStart = performance.now();
        const rootIdx = parser.parseSourceFile();
        totalParseTime += performance.now() - parseStart;

        // Check for parse errors
        const diagnosticsJson = parser.getDiagnosticsJson();
        const diagnostics = JSON.parse(diagnosticsJson);
        if (diagnostics.length > 0) {
            parseErrors++;
            failures.push({ file: testName, stage: 'parse', errors: diagnostics.length });
            parser.free();
            failed++;
            continue;
        }

        // Bind
        const bindStart = performance.now();
        if (useThinParser) {
            parser.bindSourceFile();  // ThinParser doesn't need rootIdx
        } else {
            parser.bindSourceFile(rootIdx);
        }
        totalBindTime += performance.now() - bindStart;

        // Type check
        const checkStart = performance.now();
        const checkJson = parser.checkSourceFile();
        totalCheckTime += performance.now() - checkStart;
        const checkResult = JSON.parse(checkJson);

        // Count type errors (some are expected)
        if (checkResult.diagnostics && checkResult.diagnostics.length > 0) {
            typeErrors += checkResult.diagnostics.length;
        }

        parser.free();
        passed++;
    } catch (e) {
        failures.push({ file: testName, stage: 'crash', error: e.message });
        failed++;
    }
}

const tested = passed + failed;
console.log('=== Results ===\n');
console.log(`Tested:      ${tested}/${files.length} (skipped ${skipped} multi-file/large)`);
console.log(`Passed:      ${passed}/${tested} (${(passed/tested*100).toFixed(1)}%)`);
console.log(`Failed:      ${failed}/${tested} (${(failed/tested*100).toFixed(1)}%)`);
console.log(`Parse errors: ${parseErrors} files`);
console.log(`Type errors:  ${typeErrors} total`);
console.log('');
console.log('=== Timing ===\n');
console.log(`Parse total: ${totalParseTime.toFixed(2)}ms (${(totalParseTime/passed).toFixed(2)}ms/file)`);
console.log(`Bind total:  ${totalBindTime.toFixed(2)}ms (${(totalBindTime/passed).toFixed(2)}ms/file)`);
console.log(`Check total: ${totalCheckTime.toFixed(2)}ms (${(totalCheckTime/passed).toFixed(2)}ms/file)`);
console.log(`Total:       ${(totalParseTime+totalBindTime+totalCheckTime).toFixed(2)}ms`);

if (failures.length > 0) {
    console.log(`\n=== Failures (${failures.length}) ===\n`);
    const crashes = failures.filter(f => f.stage === 'crash');
    const parseFailures = failures.filter(f => f.stage === 'parse');
    console.log(`  Crashes: ${crashes.length}`);
    console.log(`  Parse failures: ${parseFailures.length}`);

    if (crashes.length > 0) {
        console.log('\n  Sample crashes (first 5):');
        crashes.slice(0, 5).forEach(f => {
            console.log(`    ${f.file}: ${f.error?.slice(0,100)}`);
        });
    }

    if (parseFailures.length > 0) {
        console.log('\n  Parse failures (first 10):');
        parseFailures.slice(0, 10).forEach(f => {
            console.log(`    ${f.file}: ${f.errors} errors`);
        });
    }
}
