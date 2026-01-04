#!/usr/bin/env node
/**
 * Test runner for Rust/WASM TypeScript compiler against tests/cases
 *
 * Usage:
 *   node scripts/test-rust-compiler.mjs [test-file]
 *   node scripts/test-rust-compiler.mjs tests/cases/compiler/2dArrays.ts
 *
 * Flags:
 *   --thin    Use ThinParser (high-performance, 16-byte nodes)
 *   --legacy  Use legacy ParserState (208-byte nodes)
 */

import { readFileSync, existsSync } from 'fs';
import { basename, join } from 'path';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);

// Parse flags
const args = process.argv.slice(2);
const useThin = args.includes('--thin');
const useLegacy = args.includes('--legacy');
const testFile = args.find(arg => !arg.startsWith('--'));

// Load WASM module
let wasm;
try {
    wasm = require('../wasm/pkg/wasm.js');
} catch (e) {
    console.error('Failed to load WASM module. Run: wasm-pack build wasm --target nodejs');
    console.error(e.message);
    process.exit(1);
}

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

// Determine which parser to use (default: ThinParser)
const parserType = useLegacy ? 'legacy' : 'thin';
console.log(`=== Rust Compiler Output (${parserType} parser) ===\n`);

const startTime = performance.now();

let parser, rootIdx, diagnostics, binding, checkResult;
let parseTime, bindTime, checkTime;

if (useLegacy) {
    // Legacy parser (208-byte nodes)
    parser = wasm.createParser(testFile, source);
    rootIdx = parser.parseSourceFile();
    parseTime = performance.now() - startTime;

    console.log(`Parse time: ${parseTime.toFixed(2)}ms`);
    console.log(`Node count: ${parser.getNodeCount()}`);

    const diagnosticsJson = parser.getDiagnosticsJson();
    diagnostics = JSON.parse(diagnosticsJson);
    if (diagnostics.length > 0) {
        console.log(`\nParse errors (${diagnostics.length}):`);
        diagnostics.forEach((d, i) => {
            console.log(`  ${i + 1}. ${d.message} (${d.start}-${d.end})`);
        });
    }

    const bindStart = performance.now();
    const bindingJson = parser.bindSourceFile(rootIdx);
    bindTime = performance.now() - bindStart;
    binding = JSON.parse(bindingJson);

    console.log(`\nBind time: ${bindTime.toFixed(2)}ms`);
    console.log(`Symbols: ${Object.keys(binding).length}`);
    if (Object.keys(binding).length > 0) {
        console.log('  ' + Object.keys(binding).slice(0, 10).join(', ') + (Object.keys(binding).length > 10 ? '...' : ''));
    }

    const checkStart = performance.now();
    const checkJson = parser.checkSourceFile();
    checkTime = performance.now() - checkStart;
    checkResult = JSON.parse(checkJson);
} else {
    // ThinParser (16-byte nodes) - High performance path
    if (!wasm.ThinParser && !wasm.createThinParser) {
        console.error('ThinParser not available in WASM module. Rebuild with: wasm-pack build wasm --target nodejs');
        console.error('Falling back to legacy parser...\n');
        // Fall through to legacy
        parser = wasm.createParser(testFile, source);
        rootIdx = parser.parseSourceFile();
        parseTime = performance.now() - startTime;
        console.log(`Parse time: ${parseTime.toFixed(2)}ms`);
        console.log(`Node count: ${parser.getNodeCount()}`);
        diagnostics = JSON.parse(parser.getDiagnosticsJson());
        const bindStart = performance.now();
        binding = JSON.parse(parser.bindSourceFile(rootIdx));
        bindTime = performance.now() - bindStart;
        const checkStart = performance.now();
        checkResult = JSON.parse(parser.checkSourceFile());
        checkTime = performance.now() - checkStart;
    } else {
        parser = wasm.createThinParser ? wasm.createThinParser(testFile, source) : new wasm.ThinParser(testFile, source);
        rootIdx = parser.parseSourceFile();
        parseTime = performance.now() - startTime;

        console.log(`Parse time: ${parseTime.toFixed(2)}ms`);
        console.log(`Node count: ${parser.getNodeCount()}`);

        const diagnosticsJson = parser.getDiagnosticsJson();
        diagnostics = JSON.parse(diagnosticsJson);
        if (diagnostics.length > 0) {
            console.log(`\nParse errors (${diagnostics.length}):`);
            diagnostics.forEach((d, i) => {
                console.log(`  ${i + 1}. ${d.message} (${d.start}-${d.length})`);
            });
        }

        const bindStart = performance.now();
        const bindingJson = parser.bindSourceFile();  // No rootIdx param for ThinParser
        bindTime = performance.now() - bindStart;
        binding = JSON.parse(bindingJson);

        console.log(`\nBind time: ${bindTime.toFixed(2)}ms`);
        console.log(`Symbols: ${binding.symbolCount || 0}`);
        if (binding.symbols) {
            const symbolNames = Object.keys(binding.symbols);
            if (symbolNames.length > 0) {
                console.log('  ' + symbolNames.slice(0, 10).join(', ') + (symbolNames.length > 10 ? '...' : ''));
            }
        }

        const checkStart = performance.now();
        const checkJson = parser.checkSourceFile();
        checkTime = performance.now() - checkStart;
        checkResult = JSON.parse(checkJson);
    }
}

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
console.log(`Parser: ${parserType}`);
console.log(`Total time: ${totalTime.toFixed(2)}ms`);
console.log(`Parse errors: ${diagnostics.length}`);
console.log(`Type errors: ${checkResult.diagnostics?.length || 0}`);

// Cleanup
if (parser.free) parser.free();

// Compare with baselines (TODO)
console.log('\n=== Baseline Comparison (TODO) ===');
console.log('- Compare emitted JS against .js baseline');
console.log('- Compare types against .types baseline');
console.log('- Compare errors against .errors.txt baseline');
