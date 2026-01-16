import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const wasm = require(resolve(__dirname, 'pkg'));

function testASI(testName, code) {
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
    console.log('\nAST structure (first 500 chars):');
    console.log(JSON.stringify(astObj, null, 2).substring(0, 500));

    parser.free();
}

// Test 1: return with line break (ASI should apply - should parse as return; 42;)
testASI('Test 1: return with line break', 'function f() {\n  return\n  42;\n}');

// Test 2: return with value on same line (no ASI - should parse as return 42;)
testASI('Test 2: return with value', 'function f() {\n  return 42;\n}');

// Test 3: throw with line break (ASI should apply)
testASI('Test 3: throw with line break', 'function f() {\n  throw\n  new Error("test");\n}');

// Test 4: throw with value on same line
testASI('Test 4: throw with value', 'function f() {\n  throw new Error("test");\n}');

// Test 5: postfix ++ with line break (ASI should apply, NOT postfix)
testASI('Test 5: postfix ++ with line break', 'let x = 5;\nx\n++\n');

// Test 6: yield with line break (ASI should apply)
testASI('Test 6: yield with line break', 'function* f() {\n  yield\n  42;\n}');
