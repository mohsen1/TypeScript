#!/usr/bin/env node
import { readFileSync, existsSync, readdirSync } from 'fs';
import { basename, join } from 'path';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);
const wasm = require('../wasm/pkg/wasm.js');

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

const files = getFiles('tests/cases/compiler').slice(0, 1000);

let unexpectedFailures = [];

for (const file of files) {
    const source = readFileSync(file, 'utf-8');
    if (source.length > 50000 || source.includes('@filename:') || source.toLowerCase().includes('// @filename')) {
        continue;
    }

    const testName = basename(file, '.ts');
    const errorFile = 'tests/baselines/reference/' + testName + '.errors.txt';
    const expectError = existsSync(errorFile);

    try {
        const parser = wasm.createThinParser(file, source);
        parser.parseSourceFile();
        const diagnostics = JSON.parse(parser.getDiagnosticsJson());
        parser.free();

        if (diagnostics.length > 0 && !expectError) {
            unexpectedFailures.push({ name: testName, errors: diagnostics.length, first: diagnostics[0] });
        }
    } catch (e) {
        unexpectedFailures.push({ name: testName, crash: true });
    }
}

console.log('Unexpected failures:', unexpectedFailures.length);
console.log('');
unexpectedFailures.slice(0, 20).forEach(f => {
    if (f.crash) {
        console.log(f.name + ': CRASH');
    } else {
        console.log(f.name + ':', f.first.message, 'at', f.first.start);
    }
});
