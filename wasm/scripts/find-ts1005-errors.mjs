#!/usr/bin/env node
/**
 * TS1005 Error Finder - Show specific TS1005 errors from test files
 */

import { readFileSync, readdirSync, existsSync } from 'fs';
import { basename, join, extname } from 'path';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);

let wasm;
try {
    wasm = require('../pkg/wasm.js');
} catch (e) {
    console.error('Failed to load WASM module. Run: npm run wasm:build');
    process.exit(1);
}

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

const args = process.argv.slice(2);
const limit = parseInt(args.find(a => /^\d+$/.test(a))) || 200;
const category = args.find(a => ['compiler', 'conformance'].includes(a)) || 'compiler';
const testDir = `../tests/cases/${category}`;
const baselineDir = '../tests/baselines/reference';

const allFiles = getFiles(testDir);
const files = allFiles.slice(0, limit);

const ts1005Errors = [];
let totalTests = 0;

for (const file of files) {
    const source = readSourceFile(file);
    if (source.length > 50000) continue;

    // Skip multi-file tests
    if (source.includes('@filename:')) continue;

    totalTests++;
    const testName = basename(file, extname(file));

    const parser = wasm.createThinParser(basename(file), source);

    try {
        parser.parseSourceFile();
        const diagnosticsJson = parser.getDiagnosticsJson();
        const diagnostics = JSON.parse(diagnosticsJson);

        parser.bindSourceFile();
        const checkJson = parser.checkSourceFile();
        const checkResult = JSON.parse(checkJson);

        const actualCodes = [];
        if (diagnostics) {
            diagnostics.forEach(d => { if (d.code) actualCodes.push(d.code); });
        }
        if (checkResult.diagnostics) {
            checkResult.diagnostics.forEach(d => { if (d.code) actualCodes.push(d.code); });
        }

        const hasTs1005 = actualCodes.includes(1005);
        const expectedErrors = parseExpectedErrors(readFileSync(join(baselineDir, `${testName}.errors.txt`), 'utf-8').toString());
        const expectedHasTs1005 = expectedErrors.includes(1005);

        if (hasTs1005 && !expectedHasTs1005) {
            // Find the actual TS1005 diagnostic message
            const ts1005Diags = (diagnostics || []).concat(checkResult.diagnostics || [])
                .filter(d => d.code === 1005)
                .map(d => ({ message: d.message, start: d.start, length: d.length }));

            ts1005Errors.push({
                file: testName,
                diagnostics: ts1005Diags,
                source: source.split('\n').slice(0, 5).join('\n')
            });
        }

        parser.free();
    } catch (e) {
        try { parser.free(); } catch (e2) {}
    }
}

console.log(`\n=== TS1005 False Positives (${ts1005Errors.length} files) ===\n`);

ts1005Errors.slice(0, 30).forEach((err, i) => {
    console.log(`${i + 1}. ${err.file}`);
    err.diagnostics.forEach(d => {
        console.log(`   - ${d.message}`);
    });
    console.log(`   Source preview:\n   ${err.source.split('\n').join('\n   ')}`);
    console.log('');
});

if (ts1005Errors.length > 30) {
    console.log(`... and ${ts1005Errors.length - 30} more files\n`);
}

process.exit(0);
