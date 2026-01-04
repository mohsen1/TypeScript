#!/usr/bin/env node
/**
 * Run Rust Language Service Tests
 *
 * This script tests the Rust language service implementation against
 * simple test cases to verify core functionality works correctly.
 *
 * Usage:
 *   node scripts/runLanguageServiceTests.mjs                    # Run all tests
 *   node scripts/runLanguageServiceTests.mjs --verbose          # Show details
 *   node scripts/runLanguageServiceTests.mjs --feature goTo     # Test specific feature
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, "..");

// Load TypeScript - check multiple locations
let tsPath = path.join(projectRoot, "built/local/typescript.js");
if (!fs.existsSync(tsPath)) {
    // Try main worktree
    const mainWorktree = path.resolve(projectRoot, "../TypeScript/built/local/typescript.js");
    if (fs.existsSync(mainWorktree)) {
        tsPath = mainWorktree;
        console.log("Using TypeScript from main worktree\n");
    } else {
        console.error("Error: Built TypeScript not found. Run 'hereby local' first.");
        process.exit(1);
    }
}

const ts = await import(tsPath);

// Parse command line args
const args = process.argv.slice(2);
let verbose = false;
let featureFilter = null;

for (let i = 0; i < args.length; i++) {
    if (args[i] === "--verbose") {
        verbose = true;
    } else if (args[i] === "--feature" && args[i + 1]) {
        featureFilter = args[i + 1].toLowerCase();
        i++;
    }
}

console.log(`\n🧪 Rust Language Service Tests\n`);

// =============================================================================
// Test Infrastructure
// =============================================================================

let passed = 0;
let failed = 0;
let skipped = 0;

/**
 * Simple test case structure
 */
class TestCase {
    constructor(name, feature, fn) {
        this.name = name;
        this.feature = feature;
        this.fn = fn;
    }
}

const tests = [];

function test(name, feature, fn) {
    tests.push(new TestCase(name, feature, fn));
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message || "Assertion failed");
    }
}

function assertEqual(actual, expected, message) {
    if (actual !== expected) {
        throw new Error(message || `Expected ${expected}, got ${actual}`);
    }
}

function assertIncludes(arr, item, message) {
    if (!arr.includes(item)) {
        throw new Error(message || `Expected array to include ${item}`);
    }
}

/**
 * Create a minimal language service host for testing
 */
function createTestHost(files) {
    const fileMap = new Map();
    for (const [name, content] of Object.entries(files)) {
        fileMap.set(name, { content, version: "1" });
    }

    return {
        getCompilationSettings: () => ({
            target: ts.ScriptTarget.ES2020,
            module: ts.ModuleKind.ESNext,
            strict: true,
        }),
        getScriptFileNames: () => Array.from(fileMap.keys()),
        getScriptVersion: (fileName) => fileMap.get(fileName)?.version || "0",
        getScriptSnapshot: (fileName) => {
            const file = fileMap.get(fileName);
            return file ? ts.ScriptSnapshot.fromString(file.content) : undefined;
        },
        getCurrentDirectory: () => "/",
        getDefaultLibFileName: () => "lib.d.ts",
        fileExists: (fileName) => fileMap.has(fileName),
        readFile: (fileName) => fileMap.get(fileName)?.content,
        readDirectory: () => [],
        directoryExists: () => true,
        getDirectories: () => [],
    };
}

/**
 * Create a language service for testing
 */
function createTestService(files) {
    const host = createTestHost(files);
    return ts.createLanguageService(host);
}

/**
 * Find position of marker in source (the pattern is slash-star-pipe-star-slash)
 */
function findMarker(source) {
    const marker = "/\u002A|\u002A/";
    const pos = source.indexOf(marker);
    if (pos === -1) return null;
    return {
        pos,
        source: source.replace(marker, ""),
    };
}

// =============================================================================
// Go To Definition Tests
// =============================================================================

test("goToDefinition: local variable", "goTo", () => {
    const source = `
const foo = 42;
console.log(f/*|*/oo);
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const defs = service.getDefinitionAtPosition("/test.ts", pos);
    assert(defs && defs.length > 0, "Should find definition");
    assertEqual(defs[0].fileName, "/test.ts", "Definition should be in same file");
});

test("goToDefinition: function declaration", "goTo", () => {
    const source = `
function greet(name: string) {
    return "Hello " + name;
}
gr/*|*/eet("World");
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const defs = service.getDefinitionAtPosition("/test.ts", pos);
    assert(defs && defs.length > 0, "Should find function definition");
});

test("goToDefinition: class member", "goTo", () => {
    const source = `
class Person {
    name: string;
    constructor(name: string) {
        this.name = name;
    }
}
const p = new Person("Alice");
p.na/*|*/me;
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const defs = service.getDefinitionAtPosition("/test.ts", pos);
    assert(defs && defs.length > 0, "Should find property definition");
});

test("goToDefinition: imported symbol", "goTo", () => {
    const files = {
        "/utils.ts": `export function helper() { return 42; }`,
        "/main.ts": `import { helper } from "./utils";
hel/*|*/per();`,
    };
    const { pos, source: cleanSource } = findMarker(files["/main.ts"]);
    files["/main.ts"] = cleanSource;

    const service = createTestService(files);
    const defs = service.getDefinitionAtPosition("/main.ts", pos);
    assert(defs && defs.length > 0, "Should find imported definition");
});

// =============================================================================
// Completions Tests
// =============================================================================

test("completions: object member", "completions", () => {
    const source = `
const obj = { foo: 1, bar: "hello" };
obj./*|*/
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const completions = service.getCompletionsAtPosition("/test.ts", pos, {});
    assert(completions && completions.entries.length > 0, "Should have completions");

    const names = completions.entries.map(e => e.name);
    assertIncludes(names, "foo", "Should include foo");
    assertIncludes(names, "bar", "Should include bar");
});

test("completions: class members", "completions", () => {
    const source = `
class Counter {
    count = 0;
    increment() { this.count++; }
    decrement() { this.count--; }
}
const c = new Counter();
c./*|*/
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const completions = service.getCompletionsAtPosition("/test.ts", pos, {});
    assert(completions && completions.entries.length > 0, "Should have completions");

    const names = completions.entries.map(e => e.name);
    assertIncludes(names, "count", "Should include count");
    assertIncludes(names, "increment", "Should include increment");
});

test("completions: global scope", "completions", () => {
    const source = `
const myVar = 1;
function myFunc() {}
my/*|*/
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const completions = service.getCompletionsAtPosition("/test.ts", pos, {});
    assert(completions && completions.entries.length > 0, "Should have completions");

    const names = completions.entries.map(e => e.name);
    assertIncludes(names, "myVar", "Should include myVar");
    assertIncludes(names, "myFunc", "Should include myFunc");
});

// =============================================================================
// Quick Info (Hover) Tests
// =============================================================================

test("quickInfo: variable type", "quickInfo", () => {
    const source = `
const num/*|*/ber = 42;
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const info = service.getQuickInfoAtPosition("/test.ts", pos);
    assert(info, "Should have quick info");
    assert(info.displayParts && info.displayParts.length > 0, "Should have display parts");
});

test("quickInfo: function signature", "quickInfo", () => {
    const source = `
function add(a: number, b: number): number {
    return a + b;
}
ad/*|*/d(1, 2);
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const info = service.getQuickInfoAtPosition("/test.ts", pos);
    assert(info, "Should have quick info");

    const text = info.displayParts?.map(p => p.text).join("") || "";
    assert(text.includes("number"), "Should show number in signature");
});

// =============================================================================
// Find References Tests
// =============================================================================

test("findReferences: local variable", "references", () => {
    const source = `
const fo/*|*/o = 1;
console.log(foo);
const bar = foo + 2;
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const refs = service.getReferencesAtPosition("/test.ts", pos);
    assert(refs && refs.length >= 3, "Should find at least 3 references (definition + 2 uses)");
});

test("findReferences: function", "references", () => {
    const source = `
function hel/*|*/lo() { return "hi"; }
hello();
const greeting = hello();
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const refs = service.getReferencesAtPosition("/test.ts", pos);
    assert(refs && refs.length >= 3, "Should find at least 3 references");
});

// =============================================================================
// Rename Tests
// =============================================================================

test("rename: local variable", "rename", () => {
    const source = `
const old/*|*/Name = 1;
console.log(oldName);
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const renameInfo = service.getRenameInfo("/test.ts", pos, {});
    assert(renameInfo.canRename, "Should be able to rename");
    assertEqual(renameInfo.displayName, "oldName", "Should have correct display name");
});

test("rename: cannot rename string literal", "rename", () => {
    const source = `
const x = "hel/*|*/lo";
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const renameInfo = service.getRenameInfo("/test.ts", pos, {});
    assert(!renameInfo.canRename, "Should not be able to rename string literal content");
});

// =============================================================================
// Signature Help Tests
// =============================================================================

test("signatureHelp: function call", "signatureHelp", () => {
    const source = `
function greet(name: string, age: number): string {
    return name + age;
}
greet(/*|*/
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const help = service.getSignatureHelpItems("/test.ts", pos, {});
    assert(help && help.items.length > 0, "Should have signature help");
    assert(help.items[0].parameters.length === 2, "Should have 2 parameters");
});

// =============================================================================
// Document Highlights Tests
// =============================================================================

test("documentHighlights: variable occurrences", "highlights", () => {
    const source = `
const va/*|*/lue = 1;
console.log(value);
const double = value * 2;
`;
    const { pos, source: cleanSource } = findMarker(source);
    const service = createTestService({ "/test.ts": cleanSource });

    const highlights = service.getDocumentHighlights("/test.ts", pos, ["/test.ts"]);
    assert(highlights && highlights.length > 0, "Should have highlights");
    assert(highlights[0].highlightSpans.length >= 3, "Should highlight all occurrences");
});

// =============================================================================
// Run Tests
// =============================================================================

console.log(`Running ${tests.length} tests...\n`);

for (const test of tests) {
    if (featureFilter && !test.feature.toLowerCase().includes(featureFilter)) {
        skipped++;
        continue;
    }

    try {
        test.fn();
        passed++;
        if (verbose) {
            console.log(`  ✅ ${test.name}`);
        }
    } catch (e) {
        failed++;
        console.log(`  ❌ ${test.name}`);
        console.log(`     ${e.message}`);
        if (verbose && e.stack) {
            console.log(`     ${e.stack.split("\n").slice(1, 3).join("\n     ")}`);
        }
    }
}

// Print summary
console.log(`\n${"─".repeat(60)}`);
console.log(`📊 Results Summary\n`);
console.log(`  ✅ Passed:  ${passed}`);
console.log(`  ❌ Failed:  ${failed}`);
if (skipped > 0) {
    console.log(`  ⏭️  Skipped: ${skipped}`);
}
console.log(`  📈 Pass Rate: ${((passed / (passed + failed)) * 100).toFixed(1)}%`);

console.log(`\n✅ Test run complete!\n`);

process.exit(failed > 0 ? 1 : 0);
