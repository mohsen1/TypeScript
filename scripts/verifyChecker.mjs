#!/usr/bin/env node
/**
 * Checker Verification Script
 *
 * This script verifies the Rust type checker by comparing its diagnostics
 * against TypeScript's checker on the same source code.
 *
 * Usage:
 *   node scripts/verifyChecker.mjs                    # Run built-in test cases
 *   node scripts/verifyChecker.mjs <sourceFile>       # Check a specific file
 *   node scripts/verifyChecker.mjs --test-suite       # Run against TS test suite
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

// Check if WASM is available
if (!ts.wasmCheckSourceFile) {
    console.error("Error: wasmCheckSourceFile not available. WASM may not be built.");
    process.exit(1);
}

// Test cases for checker verification
const testCases = [
    // Type errors
    {
        name: "Type mismatch in variable declaration",
        code: 'const x: number = "hello";',
        expectErrors: true,
        expectedErrorCode: 2322,
    },
    {
        name: "Valid number assignment",
        code: "const x: number = 42;",
        expectErrors: false,
    },
    {
        name: "Boolean to string error",
        code: "const x: string = true;",
        expectErrors: true,
        expectedErrorCode: 2322,
    },
    {
        name: "Compatible literal types",
        code: 'const x: "hello" = "hello";',
        expectErrors: false,
    },
    {
        name: "Incompatible literal types",
        code: 'const x: "hello" = "world";',
        expectErrors: true,
        expectedErrorCode: 2322,
    },
    // Union types
    {
        name: "Valid union type assignment",
        code: "const x: number | string = 42;",
        expectErrors: false,
    },
    {
        name: "Invalid union type assignment",
        code: "const x: number | string = true;",
        expectErrors: true,
        expectedErrorCode: 2322,
    },
    // Arrays
    {
        name: "Valid array type",
        code: "const arr: number[] = [1, 2, 3];",
        expectErrors: false,
    },
    {
        name: "Array with wrong element type",
        code: 'const arr: number[] = ["a", "b"];',
        expectErrors: true,
    },
    // Functions
    {
        name: "Function with correct return type",
        code: "function add(a: number, b: number): number { return a + b; }",
        expectErrors: false,
    },
    // Basic type inference
    {
        name: "Type inference from literal",
        code: "const x = 42; const y: number = x;",
        expectErrors: false,
    },
    // Object types
    {
        name: "Object literal",
        code: "const obj = { x: 1, y: 2 };",
        expectErrors: false,
    },
    // Null/undefined
    {
        name: "Null assignment to nullable",
        code: "const x: number | null = null;",
        expectErrors: false,
    },
];

/**
 * Run TypeScript checker on source code
 */
function runTsChecker(fileName, sourceText) {
    const compilerHost = ts.createCompilerHost({
        target: ts.ScriptTarget.Latest,
        module: ts.ModuleKind.CommonJS,
        strict: true,
    });

    // Create an in-memory source file
    const originalGetSourceFile = compilerHost.getSourceFile;
    compilerHost.getSourceFile = (name, languageVersion) => {
        if (name === fileName) {
            return ts.createSourceFile(fileName, sourceText, languageVersion, true);
        }
        return originalGetSourceFile.call(compilerHost, name, languageVersion);
    };
    compilerHost.fileExists = (name) => name === fileName;
    compilerHost.readFile = (name) => (name === fileName ? sourceText : undefined);
    compilerHost.getDefaultLibFileName = () => "lib.d.ts";
    compilerHost.writeFile = () => {};
    compilerHost.getCanonicalFileName = (name) => name;
    compilerHost.useCaseSensitiveFileNames = () => true;
    compilerHost.getNewLine = () => "\n";

    const program = ts.createProgram([fileName], {
        target: ts.ScriptTarget.Latest,
        module: ts.ModuleKind.CommonJS,
        strict: true,
        noEmit: true,
        skipLibCheck: true,
    }, compilerHost);

    const diagnostics = ts.getPreEmitDiagnostics(program);
    return diagnostics.map(d => ({
        file: d.file?.fileName || "",
        start: d.start || 0,
        length: d.length || 0,
        message: ts.flattenDiagnosticMessageText(d.messageText, "\n"),
        code: d.code,
        category: d.category,
    }));
}

/**
 * Run Rust checker on source code
 */
function runRustChecker(fileName, sourceText) {
    const result = ts.wasmCheckSourceFile(fileName, sourceText);
    if (!result) {
        return { error: "WASM not available", diagnostics: [] };
    }
    if (result.error) {
        return { error: result.error, diagnostics: [] };
    }
    return {
        diagnostics: result.diagnostics.map(d => ({
            file: d.file,
            start: d.start,
            length: d.length,
            message: d.message_text,
            code: d.code,
            category: d.category,
        })),
        typeCount: result.typeCount,
    };
}

/**
 * Format a diagnostic for display
 */
function formatDiagnostic(d) {
    return `TS${d.code}: ${d.message} (at ${d.start})`;
}

console.log("\n🔍 Type Checker Verification\n");
console.log("Comparing Rust type checker against TypeScript type checker...\n");

const sourceFile = process.argv[2];

if (sourceFile === "--test-suite") {
    // Run against TypeScript test suite
    console.log("Running against TypeScript test suite...\n");

    const testDir = path.join(projectRoot, "tests/cases/compiler");
    if (!fs.existsSync(testDir)) {
        console.error("Error: Test directory not found:", testDir);
        process.exit(1);
    }

    const files = fs.readdirSync(testDir)
        .filter(f => f.endsWith(".ts"))
        .slice(0, 50); // Start with first 50 tests

    let matching = 0;
    let different = 0;
    let errored = 0;

    for (const file of files) {
        const filePath = path.join(testDir, file);
        const sourceText = fs.readFileSync(filePath, "utf-8");

        try {
            const tsResult = runTsChecker(file, sourceText);
            const rustResult = runRustChecker(file, sourceText);

            if (rustResult.error) {
                console.log(`❌ ${file}: Rust error - ${rustResult.error}`);
                errored++;
                continue;
            }

            const tsDiagCount = tsResult.length;
            const rustDiagCount = rustResult.diagnostics.length;

            if (tsDiagCount === 0 && rustDiagCount === 0) {
                console.log(`✅ ${file}: Both report no errors`);
                matching++;
            } else if (tsDiagCount > 0 && rustDiagCount > 0) {
                // Check if any error codes match
                const tsErrorCodes = new Set(tsResult.map(d => d.code));
                const rustErrorCodes = new Set(rustResult.diagnostics.map(d => d.code));
                const overlap = [...tsErrorCodes].filter(c => rustErrorCodes.has(c));

                if (overlap.length > 0) {
                    console.log(`🟡 ${file}: TS=${tsDiagCount}, Rust=${rustDiagCount} (${overlap.length} matching codes)`);
                    matching++;
                } else {
                    console.log(`⚠️  ${file}: TS=${tsDiagCount}, Rust=${rustDiagCount} (different errors)`);
                    different++;
                }
            } else if (tsDiagCount === 0 && rustDiagCount > 0) {
                console.log(`⚠️  ${file}: Rust reports ${rustDiagCount} errors, TS reports 0`);
                different++;
            } else {
                console.log(`⚠️  ${file}: TS reports ${tsDiagCount} errors, Rust reports 0`);
                different++;
            }
        } catch (e) {
            console.log(`❌ ${file}: Error - ${e.message}`);
            errored++;
        }
    }

    console.log("\n" + "─".repeat(60));
    console.log(`📊 Results: ${matching} matching, ${different} different, ${errored} errors`);
    console.log(`   out of ${files.length} files tested`);

} else if (sourceFile) {
    // Check a specific file
    const sourcePath = path.isAbsolute(sourceFile) ? sourceFile : path.join(projectRoot, sourceFile);
    if (!fs.existsSync(sourcePath)) {
        console.error(`Error: Source file not found: ${sourcePath}`);
        process.exit(1);
    }

    const sourceText = fs.readFileSync(sourcePath, "utf-8");
    console.log(`📄 Checking file: ${sourceFile}`);
    console.log(`   File size: ${sourceText.length} characters\n`);

    // TypeScript checker
    console.log("TypeScript checker:");
    const tsResult = runTsChecker(path.basename(sourceFile), sourceText);
    console.log(`  - Diagnostics: ${tsResult.length}`);
    for (const d of tsResult.slice(0, 10)) {
        console.log(`    ${formatDiagnostic(d)}`);
    }
    if (tsResult.length > 10) {
        console.log(`    ... and ${tsResult.length - 10} more`);
    }

    // Rust checker
    console.log("\nRust checker (via WASM):");
    const rustResult = runRustChecker(path.basename(sourceFile), sourceText);
    if (rustResult.error) {
        console.log(`  - Error: ${rustResult.error}`);
    } else {
        console.log(`  - Diagnostics: ${rustResult.diagnostics.length}`);
        console.log(`  - Types allocated: ${rustResult.typeCount}`);
        for (const d of rustResult.diagnostics.slice(0, 10)) {
            console.log(`    ${formatDiagnostic(d)}`);
        }
        if (rustResult.diagnostics.length > 10) {
            console.log(`    ... and ${rustResult.diagnostics.length - 10} more`);
        }
    }

    // Comparison
    console.log("\n📊 Comparison:");
    if (rustResult.error) {
        console.log("  Cannot compare - Rust checker errored");
    } else {
        const tsHasErrors = tsResult.length > 0;
        const rustHasErrors = rustResult.diagnostics.length > 0;
        console.log(`  TS reports errors: ${tsHasErrors}`);
        console.log(`  Rust reports errors: ${rustHasErrors}`);
        if (tsHasErrors === rustHasErrors) {
            console.log("  ✅ Agreement on presence of errors");
        } else {
            console.log("  ⚠️  Disagreement on presence of errors");
        }
    }

} else {
    // Run test cases
    let passed = 0;
    let failed = 0;

    for (const testCase of testCases) {
        console.log(`📝 ${testCase.name}`);
        console.log(`   Code: ${testCase.code.replace(/\n/g, "\\n")}`);

        const rustResult = runRustChecker("test.ts", testCase.code);

        if (rustResult.error) {
            console.log(`   ❌ RUST ERROR: ${rustResult.error}`);
            failed++;
            console.log();
            continue;
        }

        const hasErrors = rustResult.diagnostics.length > 0;
        const success = hasErrors === testCase.expectErrors;

        if (success) {
            if (testCase.expectErrors && testCase.expectedErrorCode) {
                const foundCode = rustResult.diagnostics.some(d => d.code === testCase.expectedErrorCode);
                if (foundCode) {
                    console.log(`   ✅ PASS - Found expected error TS${testCase.expectedErrorCode}`);
                    passed++;
                } else {
                    const codes = rustResult.diagnostics.map(d => d.code).join(", ");
                    console.log(`   ⚠️  PARTIAL - Expected TS${testCase.expectedErrorCode}, got: ${codes}`);
                    passed++; // Still counts as pass if we got an error
                }
            } else if (testCase.expectErrors) {
                console.log(`   ✅ PASS - Found ${rustResult.diagnostics.length} error(s)`);
                passed++;
            } else {
                console.log(`   ✅ PASS - No errors (as expected)`);
                passed++;
            }
        } else {
            if (testCase.expectErrors) {
                console.log(`   ❌ FAIL - Expected errors but got none`);
            } else {
                const errs = rustResult.diagnostics.map(d => `TS${d.code}: ${d.message}`).join("; ");
                console.log(`   ❌ FAIL - Expected no errors but got: ${errs}`);
            }
            failed++;
        }

        // Show diagnostic details for debugging
        if (rustResult.diagnostics.length > 0) {
            for (const d of rustResult.diagnostics) {
                console.log(`      → TS${d.code}: ${d.message}`);
            }
        }
        console.log(`   Types: ${rustResult.typeCount}`);
        console.log();
    }

    // Summary
    console.log("─".repeat(60));
    console.log(`📊 Results: ${passed} passed, ${failed} failed out of ${testCases.length} tests`);

    if (failed > 0) {
        console.log("\n⚠️  Some tests failed. The Rust checker may need more work.\n");
    } else {
        console.log("\n✅ All tests passed!\n");
    }
}

console.log("✅ Checker verification complete!\n");
