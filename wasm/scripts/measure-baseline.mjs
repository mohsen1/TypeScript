#!/usr/bin/env node
/**
 * Baseline Error Counter - Count TS1005 and TS1109 extra errors
 *
 * This script runs TypeScript test cases through the WASM compiler and counts
 * how many "extra" TS1005 (';' expected) and TS1109 (Expression expected) errors
 * we emit compared to the baseline.
 */

import { readFileSync, readdirSync, existsSync } from 'fs';
import { basename, join, extname, dirname } from 'path';
import { fileURLToPath } from 'url';
import { createRequire } from 'module';

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

// Load WASM module
let wasm;
async function loadWasm() {
    try {
        // Try dynamic import first (for ESM WASM)
        const wasmModule = await import('../pkg/wasm.js');
        return wasmModule;
    } catch (e) {
        try {
            // Fallback to require (for CJS WASM)
            return require('../pkg/wasm.js');
        } catch (e2) {
            console.error('Failed to load WASM module. Run: npm run wasm:build');
            console.error('Error:', e.message || e2.message);
            process.exit(1);
        }
    }
}

async function main() {
    wasm = await loadWasm();

    function readSourceFile(path) {
    const buffer = readFileSync(path);
    if (buffer.length >= 2 && buffer[0] === 0xFE && buffer[1] === 0xFF) {
        const swapped = Buffer.alloc(buffer.length - 2);
        for (let i = 2; i < buffer.length; i += 2) {
            if (i + 1 < buffer.length) {
                swapped[i - 2] = buffer[i + 1];
                swapped[i - 1] = buffer[i];
            }
        }
        return swapped.toString('utf16le');
    }
    if (buffer.length >= 2 && buffer[0] === 0xFF && buffer[1] === 0xFE) {
        return buffer.slice(2).toString('utf16le');
    }
    if (buffer.length >= 3 && buffer[0] === 0xEF && buffer[1] === 0xBB && buffer[2] === 0xBF) {
        return buffer.slice(3).toString('utf-8');
    }
    return buffer.toString('utf-8');
}

function parseExpectedErrors(content) {
    const errors = [];
    const regex = /: error TS(\d+):/g;
    let match;
    while ((match = regex.exec(content)) !== null) {
        errors.push(parseInt(match[1], 10));
    }
    return errors;
}

function getFiles(dir, files = []) {
    const entries = readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
        const path = join(dir, entry.name);
        if (entry.isDirectory()) {
            getFiles(path, files);
        } else if (entry.name.endsWith('.ts') || entry.name.endsWith('.tsx')) {
            files.push(path);
        }
    }
    return files;
}

// Parse args
const args = process.argv.slice(2);
const limit = parseInt(args.find(a => /^\d+$/.test(a))) || 500;
const category = args.find(a => ['compiler', 'conformance'].includes(a)) || 'compiler';
const testDir = `../tests/cases/${category}`;
const baselineDir = '../tests/baselines/reference';

const allFiles = getFiles(testDir);
const files = allFiles.slice(0, limit);

// Stats
let totalTests = 0;
let ts1005Extra = 0;
let ts1109Extra = 0;
let otherExtra = 0;
let missing = 0;
let skipped = 0;

console.log(`\n=== Baseline Error Counter: ${category} (${files.length} files) ===\n`);

for (const file of files) {
    const source = readSourceFile(file);

    if (source.length > 50000) {
        skipped++;
        continue;
    }

    totalTests++;
    const testName = basename(file, extname(file));

    // Skip multi-file tests (they have @filename: directives)
    // These should be handled separately
    if (source.includes('@filename:')) {
        skipped++;
        continue;
    }

    const parser = wasm.createThinParser(basename(file), source);

    try {
        parser.parseSourceFile();
        const diagnosticsJson = parser.getDiagnosticsJson();
        const diagnostics = JSON.parse(diagnosticsJson);

        parser.bindSourceFile();
        const checkJson = parser.checkSourceFile();
        const checkResult = JSON.parse(checkJson);

        // Count actual errors by code
        const actualErrors = {};
        if (diagnostics) {
            diagnostics.forEach(d => {
                if (d.code) {
                    actualErrors[d.code] = (actualErrors[d.code] || 0) + 1;
                }
            });
        }
        if (checkResult.diagnostics) {
            checkResult.diagnostics.forEach(d => {
                if (d.code) {
                    actualErrors[d.code] = (actualErrors[d.code] || 0) + 1;
                }
            });
        }

        // Get expected errors
        const errorsBaseline = join(baselineDir, `${testName}.errors.txt`);
        let expectedErrors = [];
        if (existsSync(errorsBaseline)) {
            const baselineContent = readFileSync(errorsBaseline, 'utf-8');
            expectedErrors = parseExpectedErrors(baselineContent);
        }

        // Count extra TS1005 and TS1109
        const expectedSet = new Set(expectedErrors);
        const actualTs1005 = actualErrors[1005] || 0;
        const expectedTs1005 = expectedSet.has(1005) ? 1 : 0;
        const actualTs1109 = actualErrors[1109] || 0;
        const expectedTs1109 = expectedSet.has(1109) ? 1 : 0;

        if (actualTs1005 > expectedTs1005) {
            ts1005Extra += (actualTs1005 - expectedTs1005);
        }
        if (actualTs1109 > expectedTs1109) {
            ts1109Extra += (actualTs1109 - expectedTs1109);
        }

        // Count missing errors
        for (const code of expectedSet) {
            if (!actualErrors[code]) {
                missing++;
            }
        }

        parser.free();
    } catch (e) {
        try { parser.free(); } catch (e2) {}
        console.error(`Error processing ${testName}: ${e.message}`);
    }
}

console.log('=== Results ===\n');
console.log(`Total tests: ${totalTests}`);
console.log(`Skipped:    ${skipped} (large files)`);
console.log(`\nExtra Errors (False Positives):`);
console.log(`  TS1005 ("';' expected" or similar): ${ts1005Extra}`);
console.log(`  TS1109 ("Expression expected"):     ${ts1109Extra}`);
console.log(`  Total extra:                        ${ts1005Extra + ts1109Extra}`);
console.log(`\nMissing Errors (False Negatives): ${missing}`);
console.log(`\nTarget: <40 extra errors`);
console.log(`Status: ${ts1005Extra + ts1109Extra < 40 ? '✅ PASS' : '❌ FAIL'}`);

process.exit(ts1005Extra + ts1109Extra < 40 ? 0 : 1);
