#!/usr/bin/env node
import { readFileSync } from 'fs';
import { createRequire } from 'module';
const require = createRequire(import.meta.url);

const wasm = require('../pkg/wasm.js');

const testFiles = [
    'ambientWithStatements',
    'anonymousModules'
];

for (const testName of testFiles) {
    const source = readFileSync(`../tests/cases/compiler/${testName}.ts`, 'utf-8');
    console.log(`\n=== ${testName} ===\n`);

    const parser = wasm.createThinParser(`${testName}.ts`, source);
    parser.parseSourceFile();
    const diagnosticsJson = parser.getDiagnosticsJson();
    const diagnostics = JSON.parse(diagnosticsJson);

    console.log('Parser diagnostics:');
    if (diagnostics && diagnostics.length > 0) {
        diagnostics.forEach(d => {
            const line = source.substring(0, d.start).split('\n').length;
            const col = d.start - source.lastIndexOf('\n', d.start - 1) - 1;
            console.log(`  Line ${line}: ${d.code} - ${d.message}`);
        });
    } else {
        console.log('  (none)');
    }

    parser.free();
}
