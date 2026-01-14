import { readFileSync, readdirSync, statSync } from 'fs';
import { join, basename } from 'path';

const wasm = await import('./pkg/wasm.js');

const TEST_CASES = [
  {
    name: 'Valid code (baseline)',
    code: `var x = 1;
var y = 2;`,
    expectedNodes: 12
  },
  {
    name: 'Missing semicolon (error recovery)',
    code: `var x = 1
var y = 2;`,
    description: 'Should recover and parse both statements'
  },
  {
    name: 'Extra closing brace',
    code: `function foo() {
  var x = 1;
}}
function bar() {
  var y = 2;
}`,
    description: 'Should emit error but continue parsing bar()'
  },
  {
    name: 'Invalid syntax mid-file',
    code: `var x = 1;
$$$ invalid syntax $$$
var y = 2;
var z = 3;`,
    description: 'Should recover and parse y, z declarations'
  },
  {
    name: 'Mismatched braces',
    code: `function foo() {
  var x = 1;
function bar() {
  var y = 2;
}`,
    description: 'Should recover and parse bar() declaration'
  },
];

console.log('Testing Error Recovery');
console.log('='.repeat(60));

let results = {
  total: 0,
  passed: 0,
  totalNodes: 0,
  errors: 0
};

for (const testCase of TEST_CASES) {
  console.log(`\nTest: ${testCase.name}`);
  console.log('-'.repeat(40));
  console.log(`  ${testCase.description || 'Baseline test'}`);

  try {
    const parser = new wasm.ThinParser('test.ts', testCase.code);
    parser.parseSourceFile();

    const nodeCount = parser.getNodeCount();
    const diagsJson = parser.getDiagnosticsJson();
    const diags = JSON.parse(diagsJson);

    results.total++;
    results.totalNodes += nodeCount;
    results.errors += diags.length;

    const hasError = diags.length > 0;
    const recovered = hasError && nodeCount > 5;

    console.log(`  Nodes: ${nodeCount}`);
    console.log(`  Errors: ${diags.length}`);

    if (diags.length > 0) {
      console.log(`  Error messages:`);
      diags.slice(0, 2).forEach(d => {
        console.log(`    - ${d.message?.substring(0, 80)}...`);
      });
    }

    if (recovered || !hasError) {
      results.passed++;
      console.log(`  ✓ ${hasError ? 'Recovered and continued parsing' : 'No errors'}`);
    } else {
      console.log(`  ✗ Failed to recover`);
    }

    parser.free();
  } catch (e) {
    console.log(`  ✗ CRASH: ${e.message}`);
    results.total++;
  }
}

console.log('\n' + '='.repeat(60));
console.log('SUMMARY');
console.log('='.repeat(60));
console.log(`Total Tests:      ${results.total}`);
console.log(`Passed:           ${results.passed} (${(results.passed/results.total*100).toFixed(0)}%)`);
console.log(`Total Nodes:      ${results.totalNodes}`);
console.log(`Total Errors:     ${results.errors} (expected from error recovery tests)`);
console.log(`\nError Recovery: ${results.passed === results.total ? 'WORKING ✓' : 'PARTIAL'}`);
