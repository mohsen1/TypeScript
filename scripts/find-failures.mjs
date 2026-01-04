#!/usr/bin/env node
/**
 * Find and categorize test failures
 */

import { readFileSync, readdirSync } from 'fs';
import { basename, join } from 'path';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);
const wasm = require('../wasm/pkg/wasm.js');

const limit = parseInt(process.argv[2]) || 100;
const testDir = process.argv[3] || 'tests/cases/compiler';

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

const files = getFiles(testDir).slice(0, limit);

const parseFailures = [];
const crashes = [];

for (const file of files) {
    const source = readFileSync(file, 'utf-8');
    if (source.length > 50000 || source.includes('@filename:')) continue;

    const testName = basename(file, '.ts');

    try {
        const parser = wasm.createParser(file, source);
        parser.parseSourceFile();
        const diagnostics = JSON.parse(parser.getDiagnosticsJson());
        if (diagnostics.length > 0) {
            // Find what syntax caused the error
            const errPos = diagnostics[0].start || 0;
            const context = source.slice(Math.max(0, errPos - 20), errPos + 30).replace(/\n/g, '\\n');
            parseFailures.push({ file: testName, error: diagnostics[0].message, context });
        }
        parser.free();
    } catch (e) {
        // Find what triggered the crash
        const firstLine = source.split('\n').slice(0, 3).join(' ').slice(0, 80);
        crashes.push({ file: testName, error: e.message?.slice(0, 50), context: firstLine });
    }
}

console.log(`\n=== Parse Failures (${parseFailures.length}) ===\n`);
parseFailures.slice(0, 20).forEach(f => {
    console.log(`${f.file}:`);
    console.log(`  Error: ${f.error}`);
    console.log(`  Context: ...${f.context}...`);
    console.log('');
});

console.log(`\n=== Crashes (${crashes.length}) ===\n`);
crashes.slice(0, 10).forEach(f => {
    console.log(`${f.file}: ${f.context.slice(0, 60)}`);
});
