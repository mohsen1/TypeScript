#!/usr/bin/env node
/**
 * Baseline comparison test for Rust emitter output
 * 
 * Compares .js emit from Rust emitter against TypeScript's baseline .js files.
 * 
 * Usage:
 *   node scripts/baseline-test-rust.mjs [options]
 * 
 * Options:
 *   --limit N        Test first N files (default: 100)
 *   --verbose        Show detailed diff for failures
 *   --pattern GLOB   Filter test files by pattern
 *   --update         Update baselines with Rust output (dangerous!)
 *   --category CAT   Test category: compiler, conformance (default: compiler)
 */

import { readFileSync, readdirSync, existsSync, writeFileSync } from 'fs';
import { join, basename, extname } from 'path';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);

// Parse arguments
const args = process.argv.slice(2);
const getArg = (name, defaultValue) => {
    const idx = args.indexOf(name);
    return idx >= 0 && args[idx + 1] ? args[idx + 1] : defaultValue;
};
const hasFlag = (name) => args.includes(name);

const LIMIT = parseInt(getArg('--limit', '100'), 10);
const VERBOSE = hasFlag('--verbose');
const PATTERN = getArg('--pattern', '');
const UPDATE = hasFlag('--update');
const CATEGORY = getArg('--category', 'compiler');

// Load WASM module
let wasm;
try {
    wasm = require('../wasm/pkg/wasm.js');
} catch (e) {
    console.error('Failed to load WASM module. Run: wasm-pack build wasm --target nodejs');
    console.error(e.message);
    process.exit(1);
}

// Directories
const casesDir = `tests/cases/${CATEGORY}`;
const baselinesDir = 'tests/baselines/reference';

console.log(`\n=== Baseline Test: Rust Emitter vs TypeScript ===\n`);
console.log(`Category: ${CATEGORY}`);
console.log(`Limit: ${LIMIT}`);
console.log(`Pattern: ${PATTERN || '(all)'}\n`);

// Get test files
function getTestFiles() {
    const files = readdirSync(casesDir)
        .filter(f => f.endsWith('.ts') && !f.endsWith('.d.ts'))
        .filter(f => !PATTERN || f.includes(PATTERN))
        .slice(0, LIMIT);
    return files;
}

// Normalize output for comparison
function normalizeOutput(output) {
    return output
        .replace(/\r\n/g, '\n')          // Normalize line endings
        .replace(/[ \t]+$/gm, '')         // Remove trailing whitespace
        .replace(/\n+$/, '\n')            // Single trailing newline
        .trim();
}

// Remove TypeScript-specific syntax for JS comparison
function stripTypeAnnotations(code) {
    // This is a simplified version - the actual emitter handles this properly
    return code
        .replace(/: [a-zA-Z<>\[\]|&\s,]+(?=[,\)\=\{])/g, '')  // Type annotations
        .replace(/\?:/g, ':')                                   // Optional type
        .replace(/<[^>]+>/g, '')                                // Generic type params
        .replace(/interface\s+\w+\s*\{[^}]*\}/gs, '')          // Interfaces
        .replace(/type\s+\w+\s*=[^;]+;/g, '')                  // Type aliases
        .replace(/declare\s+[^;]+;/g, '');                     // Declare statements
}

// Extract JS output from baseline file
// Baseline format: //// [file.ts]\n<source>\n//// [file.js]\n<js output>
function extractJsFromBaseline(baselineContent, testName) {
    // Look for the JS section
    const jsMarker = `//// [${testName}.js]`;
    const jsStartIdx = baselineContent.indexOf(jsMarker);
    
    if (jsStartIdx === -1) {
        return '';
    }
    
    // Get everything after the marker
    let jsContent = baselineContent.slice(jsStartIdx + jsMarker.length);
    
    // Stop at the next marker (if any)
    const nextMarker = jsContent.indexOf('//// [');
    if (nextMarker !== -1) {
        jsContent = jsContent.slice(0, nextMarker);
    }
    
    return jsContent;
}

// Run emit and compare
function testFile(testFile) {
    const testName = basename(testFile, '.ts');
    const sourcePath = join(casesDir, testFile);
    const baselinePath = join(baselinesDir, `${testName}.js`);
    
    // Skip if no baseline exists
    if (!existsSync(baselinePath)) {
        return { name: testName, status: 'skip', reason: 'no baseline' };
    }
    
    const source = readFileSync(sourcePath, 'utf-8');
    const baselineContent = readFileSync(baselinePath, 'utf-8');
    const expectedJs = extractJsFromBaseline(baselineContent, testName);
    
    try {
        // Parse with ThinParser
        let parser;
        if (wasm.createThinParser) {
            parser = wasm.createThinParser(testFile, source);
        } else if (wasm.ThinParser) {
            parser = new wasm.ThinParser(testFile, source);
        } else {
            parser = wasm.createParser(testFile, source);
        }
        
        const root = parser.parseSourceFile();
        
        // Get diagnostics
        const diagnostics = JSON.parse(parser.getDiagnosticsJson());
        if (diagnostics.length > 0) {
            parser.free?.();
            return { 
                name: testName, 
                status: 'skip', 
                reason: `parse errors: ${diagnostics.length}` 
            };
        }
        
        // Emit
        let emittedJs = '';
        if (parser.emit) {
            emittedJs = parser.emit();
        } else if (parser.emitSourceFile) {
            emittedJs = parser.emitSourceFile();
        } else {
            parser.free?.();
            return { name: testName, status: 'skip', reason: 'emit not available' };
        }
        
        parser.free?.();
        
        // Normalize and compare
        const normalizedExpected = normalizeOutput(expectedJs);
        const normalizedActual = normalizeOutput(emittedJs);
        
        if (normalizedActual === normalizedExpected) {
            return { name: testName, status: 'pass' };
        }
        
        // Calculate similarity
        const expectedLines = normalizedExpected.split('\n');
        const actualLines = normalizedActual.split('\n');
        const matchingLines = expectedLines.filter((line, i) => 
            actualLines[i] === line
        ).length;
        const similarity = Math.round(
            (matchingLines / Math.max(expectedLines.length, actualLines.length)) * 100
        );
        
        return {
            name: testName,
            status: 'fail',
            expected: normalizedExpected,
            actual: normalizedActual,
            similarity
        };
    } catch (e) {
        return { 
            name: testName, 
            status: 'error', 
            reason: e.message 
        };
    }
}

// Main
const testFiles = getTestFiles();
console.log(`Found ${testFiles.length} test files\n`);

const results = {
    pass: [],
    fail: [],
    skip: [],
    error: []
};

let processed = 0;
for (const file of testFiles) {
    const result = testFile(file);
    results[result.status].push(result);
    processed++;
    
    // Progress indicator
    if (processed % 10 === 0) {
        process.stdout.write(`\rProcessed ${processed}/${testFiles.length}...`);
    }
}

console.log(`\r                                        \r`);

// Summary
console.log(`\n=== Results ===\n`);
console.log(`✓ Pass:  ${results.pass.length}`);
console.log(`✗ Fail:  ${results.fail.length}`);
console.log(`⊘ Skip:  ${results.skip.length}`);
console.log(`⚠ Error: ${results.error.length}`);

const total = results.pass.length + results.fail.length;
if (total > 0) {
    const passRate = ((results.pass.length / total) * 100).toFixed(1);
    console.log(`\nPass rate: ${passRate}% (${results.pass.length}/${total})`);
}

// Show failures
if (results.fail.length > 0 && VERBOSE) {
    console.log(`\n=== Failures ===\n`);
    for (const fail of results.fail.slice(0, 5)) {
        console.log(`--- ${fail.name} (${fail.similarity}% similar) ---`);
        console.log('\nExpected (first 20 lines):');
        console.log(fail.expected.split('\n').slice(0, 20).join('\n'));
        console.log('\nActual (first 20 lines):');
        console.log(fail.actual.split('\n').slice(0, 20).join('\n'));
        console.log('');
    }
} else if (results.fail.length > 0) {
    console.log(`\n=== Failed Tests ===\n`);
    for (const fail of results.fail.slice(0, 20)) {
        console.log(`  ${fail.name} (${fail.similarity}% similar)`);
    }
    if (results.fail.length > 20) {
        console.log(`  ... and ${results.fail.length - 20} more`);
    }
}

// Show skip reasons
if (results.skip.length > 0) {
    const skipReasons = {};
    for (const skip of results.skip) {
        skipReasons[skip.reason] = (skipReasons[skip.reason] || 0) + 1;
    }
    console.log(`\n=== Skip Reasons ===\n`);
    for (const [reason, count] of Object.entries(skipReasons)) {
        console.log(`  ${reason}: ${count}`);
    }
}

// Show errors
if (results.error.length > 0) {
    console.log(`\n=== Errors ===\n`);
    for (const err of results.error.slice(0, 5)) {
        console.log(`  ${err.name}: ${err.reason}`);
    }
}

// Update baselines if requested
if (UPDATE && results.fail.length > 0) {
    console.log(`\n=== Updating ${results.fail.length} baselines ===\n`);
    for (const fail of results.fail) {
        const baselinePath = join(baselinesDir, `${fail.name}.js`);
        writeFileSync(baselinePath, fail.actual + '\n');
        console.log(`  Updated: ${fail.name}.js`);
    }
}

// Exit code
process.exit(results.error.length > 0 ? 1 : 0);
