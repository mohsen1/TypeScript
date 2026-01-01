#!/usr/bin/env node
/**
 * Parser Verification Script
 *
 * This script verifies that the Rust parser produces an AST that can be
 * consumed by TypeScript's compiler infrastructure.
 *
 * Usage: node scripts/verifyParser.mjs [sourceFile]
 * Default: A simple test file
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, "..");

// Load the built TypeScript module
const tsPath = path.join(projectRoot, "built/local/typescript.js");
if (!fs.existsSync(tsPath)) {
    console.error("Error: Built TypeScript not found. Run 'hereby local' first.");
    process.exit(1);
}

const ts = await import(tsPath);

// Test cases for parser verification
const testCases = [
    // Basic declarations
    {
        name: "Simple variable declaration",
        code: "const x = 42;",
        expectedStatements: 1,
    },
    {
        name: "Variable with type annotation",
        code: "let y: number = 10;",
        expectedStatements: 1,
    },
    {
        name: "Multiple variable declarations",
        code: "const x = 1;\nconst y = 2;\nconst z = x + y;",
        expectedStatements: 3,
    },

    // Functions
    {
        name: "Function declaration",
        code: "function add(a: number, b: number): number { return a + b; }",
        expectedStatements: 1,
    },
    {
        name: "Function with no return type",
        code: "function greet(name: string) { console.log(name); }",
        expectedStatements: 1,
    },

    // Control flow
    {
        name: "If statement",
        code: "if (true) { console.log('yes'); }",
        expectedStatements: 1,
    },
    {
        name: "If-else statement",
        code: "if (x > 0) { console.log('positive'); } else { console.log('negative'); }",
        expectedStatements: 1,
    },
    {
        name: "Return statement",
        code: "function test() { return 42; }",
        expectedStatements: 1,
    },

    // Expressions
    {
        name: "Binary expression",
        code: "const sum = 1 + 2 * 3;",
        expectedStatements: 1,
    },
    {
        name: "Call expression",
        code: "console.log('hello');",
        expectedStatements: 1,
    },
    {
        name: "Property access",
        code: "const len = arr.length;",
        expectedStatements: 1,
    },

    // Loops
    {
        name: "While loop",
        code: "while (true) { break; }",
        expectedStatements: 1,
    },
    {
        name: "For loop",
        code: "for (let i = 0; i < 10; i++) { console.log(i); }",
        expectedStatements: 1,
    },

    // Classes
    {
        name: "Class declaration",
        code: "class Foo { constructor() {} }",
        expectedStatements: 1,
    },
    {
        name: "Class with method",
        code: "class Bar { greet() { return 'hello'; } }",
        expectedStatements: 1,
    },

    // Imports/Exports
    {
        name: "Import declaration",
        code: "import { foo } from 'bar';",
        expectedStatements: 1,
    },
    {
        name: "Export declaration",
        code: "export const x = 42;",
        expectedStatements: 1,
    },

    // Types
    {
        name: "Interface declaration",
        code: "interface IFoo { x: number; }",
        expectedStatements: 1,
    },
    {
        name: "Type alias",
        code: "type Point = { x: number; y: number };",
        expectedStatements: 1,
    },
];

console.log("\n🔍 Parser Verification\n");
console.log("Testing Rust parser integration with TypeScript AST consumers...\n");

// Get source file if provided
const sourceFile = process.argv[2];
if (sourceFile) {
    const sourcePath = path.isAbsolute(sourceFile) ? sourceFile : path.join(projectRoot, sourceFile);
    if (!fs.existsSync(sourcePath)) {
        console.error(`Error: Source file not found: ${sourcePath}`);
        process.exit(1);
    }

    const sourceText = fs.readFileSync(sourcePath, "utf-8");
    console.log(`📄 Parsing file: ${sourceFile}`);
    console.log(`   File size: ${sourceText.length} characters\n`);

    // Parse with TypeScript parser
    console.log("TypeScript parser:");
    const tsResult = ts.createSourceFile(sourceFile, sourceText, ts.ScriptTarget.Latest, true);
    console.log(`  - Statements: ${tsResult.statements.length}`);
    console.log(`  - Node count: ${tsResult.nodeCount}`);
    console.log(`  - Identifier count: ${tsResult.identifierCount}`);
    console.log(`  - Errors: ${tsResult.parseDiagnostics.length}`);

    // Enable Rust parser
    if (ts.sys) {
        ts.sys.useRustParser = true;
    }

    // Parse with Rust parser
    console.log("\nRust parser (via WASM):");
    try {
        const rustResult = ts.createSourceFile(sourceFile, sourceText, ts.ScriptTarget.Latest, true);
        console.log(`  - Statements: ${rustResult.statements.length}`);
        console.log(`  - Node count: ${rustResult.nodeCount}`);
        console.log(`  - Identifier count: ${rustResult.identifierCount}`);
        console.log(`  - Errors: ${rustResult.parseDiagnostics.length}`);

        // Compare results
        console.log("\n📊 Comparison:");
        const stmtMatch = tsResult.statements.length === rustResult.statements.length;
        console.log(`  Statement count: ${stmtMatch ? "✅ MATCH" : "❌ MISMATCH"}`);
        console.log(`    TS: ${tsResult.statements.length}, Rust: ${rustResult.statements.length}`);
    } catch (e) {
        console.log(`  - Error: ${e.message}`);
    }

    // Disable Rust parser
    if (ts.sys) {
        ts.sys.useRustParser = false;
    }
} else {
    // Run test cases
    let passed = 0;
    let failed = 0;

    for (const testCase of testCases) {
        console.log(`📝 ${testCase.name}`);
        console.log(`   Code: ${testCase.code.replace(/\n/g, "\\n")}`);

        // Parse with TypeScript parser first (for comparison)
        const tsResult = ts.createSourceFile("test.ts", testCase.code, ts.ScriptTarget.Latest, true);
        console.log(`   TS statements: ${tsResult.statements.length}`);

        // Enable Rust parser
        if (ts.sys) {
            ts.sys.useRustParser = true;
        }

        try {
            const rustResult = ts.createSourceFile("test.ts", testCase.code, ts.ScriptTarget.Latest, true);
            const success = rustResult.statements.length === testCase.expectedStatements;

            if (success) {
                console.log(`   ✅ PASS - Rust statements: ${rustResult.statements.length}`);
                passed++;
            } else {
                console.log(`   ❌ FAIL - Rust statements: ${rustResult.statements.length}, expected: ${testCase.expectedStatements}`);
                failed++;
            }
        } catch (e) {
            console.log(`   ❌ ERROR - ${e.message}`);
            failed++;
        }

        // Disable Rust parser
        if (ts.sys) {
            ts.sys.useRustParser = false;
        }

        console.log();
    }

    // Summary
    console.log("─".repeat(50));
    console.log(`📊 Results: ${passed} passed, ${failed} failed out of ${testCases.length} tests`);

    if (failed > 0) {
        process.exit(1);
    }
}

console.log("\n✅ Parser verification complete!\n");
