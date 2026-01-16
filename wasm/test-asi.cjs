#!/usr/bin/env node
/**
 * Test ASI (Automatic Semicolon Insertion) edge cases
 */

const wasm = require('./pkg');

function testCase(testName, code) {
    console.log(`\n=== ${testName} ===`);
    console.log(`Code: ${JSON.stringify(code)}`);

    const fileName = 'test.ts';

    // WASM
    const parser = new wasm.ThinParser(fileName, code);
    parser.parseSourceFile();
    const diags = JSON.parse(parser.getDiagnosticsJson());
    console.log('\nWASM diagnostics:');
    if (diags.length === 0) {
        console.log('  (no parse errors)');
    } else {
        diags.forEach(d => console.log(`  ${d.code}: ${d.message} at ${d.span?.start}`));
    }

    // Get AST for inspection
    const ast = parser.getAstJson();
    const astObj = JSON.parse(ast);
    console.log('\nAST structure (first 800 chars):');
    console.log(JSON.stringify(astObj, null, 2).substring(0, 800));

    parser.free();
}

// Test 1: return with line break (ASI should apply - should parse as return; 42;)
testCase('Test 1: return with line break', 'function f() {\n  return\n  42;\n}');

// Test 2: return with value on same line (no ASI - should parse as return 42;)
testCase('Test 2: return with value', 'function f() {\n  return 42;\n}');

// Test 3: throw with line break (ASI should apply)
testCase('Test 3: throw with line break', 'function f() {\n  throw\n  new Error("test");\n}');

// Test 4: throw with value on same line
testCase('Test 4: throw with value', 'function f() {\n  throw new Error("test");\n}');

// Test 5: postfix ++ with line break (ASI should apply, NOT postfix)
testCase('Test 5: postfix ++ with line break', 'let x = 5;\nx\n++\n');

// Test 6: yield with line break (ASI should apply)
testCase('Test 6: yield with line break', 'function* f() {\n  yield\n  42;\n}');

// Test 7: return with object literal (ASI on line break)
testCase('Test 7: return object with line break', 'function f() {\n  return\n  { x: 1 };\n}');

// Test 8: return with object literal (no ASI)
testCase('Test 8: return object without line break', 'function f() {\n  return { x: 1 };\n}');
