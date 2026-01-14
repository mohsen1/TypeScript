const { ThinParser } = require('./pkg/wasm.js');
const { readFileSync, readdirSync, statSync } = require('fs');
const { join } = require('path');

const CONFORMANCE_DIR = '../tests/cases/conformance';
const MAX_TESTS = 100;

let totalTests = 0;
let totalTS1109 = 0;
let testsWithTS1109 = 0;

function countFiles(dir) {
    let count = 0;
    const files = readdirSync(dir);
    for (const file of files) {
        const fullPath = join(dir, file);
        const stat = statSync(fullPath);
        if (stat.isDirectory() && file !== 'lib') {
            count += countFiles(fullPath);
        } else if (file.endsWith('.ts')) {
            count++;
        }
    }
    return count;
}

function processDirectory(dir, maxFiles = MAX_TESTS) {
    const files = readdirSync(dir);
    let processed = 0;

    for (const file of files) {
        if (processed >= maxFiles) break;

        const fullPath = join(dir, file);
        const stat = statSync(fullPath);

        if (stat.isDirectory() && file !== 'lib') {
            processDirectory(fullPath, maxFiles - processed);
        } else if (file.endsWith('.ts') && processed < maxFiles) {
            try {
                const code = readFileSync(fullPath, 'utf8');
                const parser = new ThinParser(fullPath, code);
                parser.parseSourceFile();
                const result = JSON.parse(parser.checkSourceFile());

                const ts1109 = (result.diagnostics || []).filter(d => d.code === 1109);
                if (ts1109.length > 0) {
                    testsWithTS1109++;
                    totalTS1109 += ts1109.length;
                    console.log(fullPath + ': ' + ts1109.length + ' TS1109 errors');
                }

                totalTests++;
                processed++;
            } catch (e) {
                console.error('Error processing ' + fullPath + ': ' + e.message);
            }
        }
    }
}

console.log('Measuring TS1109 in conformance tests...');
console.log('Max tests: ' + MAX_TESTS);
console.log('');

processDirectory(CONFORMANCE_DIR, MAX_TESTS);

console.log('');
console.log('=== TS1109 Measurement Results ===');
console.log('Total tests processed: ' + totalTests);
console.log('Tests with TS1109 errors: ' + testsWithTS1109);
console.log('Total TS1109 errors: ' + totalTS1109);
console.log('Average TS1109 per test: ' + (totalTS1109 / Math.max(testsWithTS1109, 1)).toFixed(2));
