#!/usr/bin/env node
/**
 * Script to verify the Rust binder produces correct symbols.
 * Run: node scripts/verifyBinder.mjs
 */

import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const require = createRequire(import.meta.url);

// Load the wasm module
const wasmPath = join(__dirname, '..', 'built', 'local', 'wasm', 'wasm.js');
let wasm;
try {
    wasm = require(wasmPath);
} catch (e) {
    console.error('Failed to load wasm module. Make sure to run `npx hereby local` first.');
    console.error(e.message);
    process.exit(1);
}

// Test cases with expected symbols
const testCases = [
    {
        name: 'Variable declaration',
        code: 'const x = 42;',
        expectedSymbols: ['x'],
    },
    {
        name: 'Function declaration',
        code: 'function foo() {}',
        expectedSymbols: ['foo'],
    },
    {
        name: 'Class declaration',
        code: 'class MyClass {}',
        expectedSymbols: ['MyClass'],
    },
    {
        name: 'Interface declaration',
        code: 'interface IFoo { x: number; }',
        expectedSymbols: ['IFoo'],
    },
    {
        name: 'Type alias',
        code: 'type MyType = string;',
        expectedSymbols: ['MyType'],
    },
    {
        name: 'Enum declaration',
        code: 'enum Color { Red, Green, Blue }',
        expectedSymbols: ['Color'],
    },
    {
        name: 'Namespace declaration',
        code: 'namespace NS { export const x = 1; }',
        expectedSymbols: ['NS'],
    },
    {
        name: 'Multiple declarations',
        code: `
            const a = 1;
            function b() {}
            class C {}
            interface D {}
            type E = string;
        `,
        expectedSymbols: ['a', 'b', 'C', 'D', 'E'],
    },
    {
        name: 'Interface merging',
        code: `
            interface Foo { x: number; }
            interface Foo { y: string; }
        `,
        expectedSymbols: ['Foo'],
    },
    {
        name: 'Import declaration',
        code: 'import { foo } from "bar";',
        expectedSymbols: ['foo'],
    },
];

let passed = 0;
let failed = 0;

console.log('Verifying Rust binder...\n');

for (const test of testCases) {
    try {
        const parser = wasm.createParser('test.ts', test.code);
        const rootIdx = parser.parseSourceFile();
        const bindingJson = parser.bindSourceFile(rootIdx);
        const binding = JSON.parse(bindingJson);

        // Get symbol names from the binding result
        const symbols = binding.symbols ? Object.keys(binding.symbols) : [];

        // Check if all expected symbols are present
        let allFound = true;
        for (const expected of test.expectedSymbols) {
            if (!symbols.includes(expected)) {
                allFound = false;
                console.log(`FAIL: ${test.name}`);
                console.log(`  Missing symbol: ${expected}`);
                console.log(`  Found symbols: ${symbols.join(', ')}`);
                failed++;
                break;
            }
        }

        if (allFound) {
            console.log(`PASS: ${test.name}`);
            passed++;
        }
    } catch (e) {
        console.log(`FAIL: ${test.name}`);
        console.log(`  Error: ${e.message}`);
        failed++;
    }
}

console.log(`\n${passed}/${testCases.length} tests passed`);

if (failed > 0) {
    process.exit(1);
}
