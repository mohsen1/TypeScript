const fs = require('fs');
const path = require('path');
const { createThinParser } = require('../wasm/pkg/wasm.js');

const basePath = 'tests/baselines/reference';
const tests = ['ParameterList7', 'abstractClassUnionInstantiation', 'abstractPropertyInConstructor', 'ClassDeclaration26'];

console.log('Starting check...');

for (const testName of tests) {
  const baselineFile = path.join(basePath, testName + '.js');
  console.log('Checking:', baselineFile, fs.existsSync(baselineFile));
  if (!fs.existsSync(baselineFile)) continue;

  const baseline = fs.readFileSync(baselineFile, 'utf8');

  // Split by //// [ markers
  const parts = baseline.split(/\/\/\/\/ \[/);

  // Find the JS and TS parts
  let expected = '';
  let source = '';

  for (const part of parts) {
    // Extract the file name from the start of the part
    const endBracket = part.indexOf(']');
    if (endBracket < 0) continue;
    const fileName = part.substring(0, endBracket);

    if (fileName.endsWith('.js')) {
      // This is the JS output part
      expected = part.substring(endBracket + 2).trim(); // Skip "]\n"
    } else if (fileName.endsWith('.ts') && !fileName.includes('/')) {
      // This is the TS source part (not the full path marker)
      source = part.substring(endBracket + 2).trim(); // Skip "]\n"
    }
  }

  if (!expected || !source) {
    console.log('  Could not parse baseline');
    continue;
  }

  const parser = createThinParser(testName + '.ts', source);
  parser.parseSourceFile();
  let actual = parser.emit().trim();

  // Normalize: strip comments, normalize line endings, collapse whitespace
  const normalize = (s) => s
    .replace(/\r\n/g, '\n')  // Windows -> Unix line endings
    .replace(/\/\/[^\n]*/g, '')  // Strip line comments
    .replace(/\/\*[\s\S]*?\*\//g, '')  // Strip block comments
    .replace(/\s+/g, ' ')
    .trim();
  const stripComments = normalize;

  const expectedNoComments = stripComments(expected);
  const actualNoComments = stripComments(actual);

  const match = expectedNoComments === actualNoComments;
  console.log(testName + ': ' + (match ? 'MATCH (comments only)' : 'MISMATCH'));

  if (!match) {
    console.log('  Expected:', expectedNoComments.substring(0, 200));
    console.log('  Actual:  ', actualNoComments.substring(0, 200));
  }
}
