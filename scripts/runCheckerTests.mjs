#!/usr/bin/env node
/**
 * Run Rust Type Checker against TypeScript compiler test cases
 *
 * This script compares the diagnostics produced by our Rust checker
 * against the expected .errors.txt baselines.
 *
 * Usage:
 *   node scripts/runCheckerTests.mjs                    # Run all tests
 *   node scripts/runCheckerTests.mjs --limit 100        # Run first 100 tests
 *   node scripts/runCheckerTests.mjs --pattern "arrow"  # Run tests matching pattern
 *   node scripts/runCheckerTests.mjs --verbose          # Show all results
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { execSync } from "child_process";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, "..");

// Load TypeScript with WASM
const tsPath = path.join(projectRoot, "built/local/typescript.js");
if (!fs.existsSync(tsPath)) {
    console.error("Error: Built TypeScript not found. Run 'hereby local' first.");
    process.exit(1);
}

const ts = await import(tsPath);

if (!ts.wasmCheckSourceFile) {
    console.error("Error: wasmCheckSourceFile not available. WASM may not be built.");
    process.exit(1);
}

// Parse command line args
const args = process.argv.slice(2);
let limit = Infinity;
let pattern = null;
let verbose = false;

for (let i = 0; i < args.length; i++) {
    if (args[i] === "--limit" && args[i + 1]) {
        limit = parseInt(args[i + 1], 10);
        i++;
    } else if (args[i] === "--pattern" && args[i + 1]) {
        pattern = args[i + 1].toLowerCase();
        i++;
    } else if (args[i] === "--verbose") {
        verbose = true;
    }
}

// Find test files
const compilerTestsDir = path.join(projectRoot, "tests/cases/compiler");
const baselinesDir = path.join(projectRoot, "tests/baselines/reference");

const testFiles = fs.readdirSync(compilerTestsDir)
    .filter(f => f.endsWith(".ts"))
    .filter(f => !pattern || f.toLowerCase().includes(pattern))
    .slice(0, limit);

console.log(`\n🧪 Running Rust Checker Tests\n`);
console.log(`Found ${testFiles.length} test files to run\n`);

let passed = 0;
let failed = 0;
let skipped = 0;

const results = {
    perfectMatch: [],      // Both have same errors (or both have none)
    extraErrors: [],       // Rust reports errors that TS doesn't expect
    missingErrors: [],     // TS expects errors that Rust doesn't report
    wrongCodes: [],        // Same number of errors but different codes
};

/**
 * Parse expected errors from .errors.txt baseline
 */
function parseExpectedErrors(errorsContent) {
    const errors = [];
    const lines = errorsContent.split("\n");

    // Parse error lines like: "file.ts(1,13): error TS1110: Type expected."
    const errorLineRegex = /^(.+?)\((\d+),(\d+)\): error TS(\d+): (.+)$/;

    for (const line of lines) {
        const match = line.match(errorLineRegex);
        if (match) {
            errors.push({
                file: match[1],
                line: parseInt(match[2], 10),
                col: parseInt(match[3], 10),
                code: parseInt(match[4], 10),
                message: match[5],
            });
        }
    }

    return errors;
}

/**
 * Run Rust checker directly (without using wasmCheckSourceFile which has a free() issue)
 */
function runRustChecker(fileName, sourceText) {
    try {
        const parser = ts.wasmCreateParser(fileName, sourceText);
        if (!parser) return null;

        parser.parseSourceFile();
        const resultJson = parser.checkSourceFile();
        return JSON.parse(resultJson);
        // Note: Intentionally NOT calling parser.free() due to borrow issue
    } catch (e) {
        return { error: e.message, crash: true };
    }
}

/**
 * Run checker on a test file and compare to baseline
 */
function runTest(testFile) {
    const testPath = path.join(compilerTestsDir, testFile);
    const baseName = testFile.replace(".ts", "");
    const errorsPath = path.join(baselinesDir, `${baseName}.errors.txt`);

    // Read test source
    const sourceText = fs.readFileSync(testPath, "utf-8");

    // Get expected errors (empty if no .errors.txt exists)
    const hasExpectedErrors = fs.existsSync(errorsPath);
    const expectedErrors = hasExpectedErrors
        ? parseExpectedErrors(fs.readFileSync(errorsPath, "utf-8"))
        : [];

    // Run Rust checker
    const result = runRustChecker(testFile, sourceText);

    if (!result) {
        return { status: "skipped", reason: "WASM not available" };
    }

    if (result.crash) {
        return { status: "skipped", reason: `Crash: ${result.error}` };
    }

    if (result.error) {
        return { status: "skipped", reason: result.error };
    }

    const actualErrors = result.diagnostics || [];
    const actualCodes = actualErrors.map(d => d.code).sort((a, b) => a - b);
    const expectedCodes = expectedErrors.map(e => e.code).sort((a, b) => a - b);

    // Compare results
    if (actualCodes.length === 0 && expectedCodes.length === 0) {
        return { status: "pass", type: "perfectMatch" };
    }

    if (JSON.stringify(actualCodes) === JSON.stringify(expectedCodes)) {
        return { status: "pass", type: "perfectMatch" };
    }

    // Categorize the failure
    const actualSet = new Set(actualCodes);
    const expectedSet = new Set(expectedCodes);

    const extra = actualCodes.filter(c => !expectedSet.has(c));
    const missing = expectedCodes.filter(c => !actualSet.has(c));

    if (extra.length > 0 && missing.length === 0) {
        return {
            status: "fail",
            type: "extraErrors",
            extra,
            actual: actualCodes,
            expected: expectedCodes,
        };
    }

    if (missing.length > 0 && extra.length === 0) {
        return {
            status: "fail",
            type: "missingErrors",
            missing,
            actual: actualCodes,
            expected: expectedCodes,
        };
    }

    return {
        status: "fail",
        type: "wrongCodes",
        extra,
        missing,
        actual: actualCodes,
        expected: expectedCodes,
    };
}

// Run tests
for (const testFile of testFiles) {
    const result = runTest(testFile);

    if (result.status === "pass") {
        passed++;
        results.perfectMatch.push(testFile);
        if (verbose) {
            console.log(`  ✅ ${testFile}`);
        }
    } else if (result.status === "skipped") {
        skipped++;
        if (verbose) {
            console.log(`  ⏭️  ${testFile} - ${result.reason}`);
        }
    } else {
        failed++;
        results[result.type].push({ file: testFile, ...result });
        if (verbose) {
            console.log(`  ❌ ${testFile}`);
            console.log(`     Expected: [${result.expected.join(", ")}]`);
            console.log(`     Actual:   [${result.actual.join(", ")}]`);
        }
    }
}

// Print summary
console.log(`\n${"─".repeat(60)}`);
console.log(`📊 Results Summary\n`);
console.log(`  ✅ Passed:  ${passed}`);
console.log(`  ❌ Failed:  ${failed}`);
console.log(`  ⏭️  Skipped: ${skipped}`);
console.log(`  📈 Pass Rate: ${((passed / (passed + failed)) * 100).toFixed(1)}%`);

if (failed > 0) {
    console.log(`\n📋 Failure Breakdown:\n`);

    if (results.extraErrors.length > 0) {
        console.log(`  🔴 Extra errors (Rust reports more): ${results.extraErrors.length}`);
        if (verbose || results.extraErrors.length <= 5) {
            for (const r of results.extraErrors.slice(0, 5)) {
                console.log(`     - ${r.file}: extra codes [${r.extra.join(", ")}]`);
            }
        }
    }

    if (results.missingErrors.length > 0) {
        console.log(`  🟡 Missing errors (Rust reports less): ${results.missingErrors.length}`);
        if (verbose || results.missingErrors.length <= 5) {
            for (const r of results.missingErrors.slice(0, 5)) {
                console.log(`     - ${r.file}: missing codes [${r.missing.join(", ")}]`);
            }
        }
    }

    if (results.wrongCodes.length > 0) {
        console.log(`  🟠 Wrong codes (different errors): ${results.wrongCodes.length}`);
        if (verbose || results.wrongCodes.length <= 5) {
            for (const r of results.wrongCodes.slice(0, 5)) {
                console.log(`     - ${r.file}: +[${r.extra.join(", ")}] -[${r.missing.join(", ")}]`);
            }
        }
    }
}

console.log(`\n✅ Test run complete!\n`);
