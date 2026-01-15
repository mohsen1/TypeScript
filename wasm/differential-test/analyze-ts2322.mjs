#!/usr/bin/env node
/**
 * Analyze specific test files for TS2322 errors
 */

import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, join, resolve, basename } from 'path';
import { readFileSync } from 'fs';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const ts = require('typescript');

function analyzeFile(filePath) {
  const code = readFileSync(filePath, 'utf-8');

  // Extract test directives
  const lines = code.split('\n');
  const testOpts = {};
  for (let i = 0; i < Math.min(20, lines.length); i++) {
    const line = lines[i].trim();
    if (line.startsWith('// @')) {
      const match = line.match(/\/\/ @(\w+):\s*(.+)/);
      if (match) testOpts[match[1]] = match[2];
    }
  }

  const compilerOptions = { noEmit: true, skipLibCheck: true };
  if (testOpts.strict) compilerOptions.strict = true;
  if (testOpts.target) compilerOptions.target = ts.ScriptTarget[testOpts.target.toUpperCase()] || ts.ScriptTarget.ES2020;
  if (testOpts.strictPropertyInitialization) compilerOptions.strictPropertyInitialization = testOpts.strictPropertyInitialization === 'true';

  const fileName = basename(filePath);
  const sf = ts.createSourceFile(fileName, code, ts.ScriptTarget.ES2020, true);
  const host = ts.createCompilerHost(compilerOptions);
  const program = ts.createProgram([fileName], compilerOptions, {
    ...host,
    getSourceFile: (name) => name === fileName ? sf : host.getSourceFile(name, ts.ScriptTarget.ES2020),
  });

  const allDiagnostics = [
    ...program.getSyntacticDiagnostics(sf),
    ...program.getSemanticDiagnostics(sf),
  ];

  const ts2322 = allDiagnostics.filter(d => d.code === 2322);
  const allErrors = allDiagnostics.filter(d => d.code !== 6053 && d.code !== 6054); // Remove file not found errors

  return {
    ts2322: ts2322.map(d => ({
      code: d.code,
      message: ts.flattenDiagnosticMessageText(d.messageText, '\n'),
      line: d.start ? sf.getLineAndCharacterOfPosition(d.start).line + 1 : null,
    })),
    allErrors: allErrors.map(d => ({
      code: d.code,
      message: ts.flattenDiagnosticMessageText(d.messageText, '\n'),
      line: d.start ? sf.getLineAndCharacterOfPosition(d.start).line + 1 : null,
    })),
  };
}

// Main execution
const filePaths = process.argv.slice(2);

if (filePaths.length === 0) {
  console.error('Usage: node analyze-ts2322.mjs <file1.ts> [file2.ts] ...');
  process.exit(1);
}

for (const filePath of filePaths) {
  try {
    console.log(`\n${'='.repeat(80)}`);
    console.log(`FILE: ${filePath}`);
    console.log('='.repeat(80));

    const result = analyzeFile(filePath);

    console.log(`\nTS2322 Errors (${result.ts2322.length}):`);
    if (result.ts2322.length === 0) {
      console.log('  (none)');
    } else {
      result.ts2322.forEach(e => {
        console.log(`  Line ${e.line}: TS${e.code}`);
        console.log(`    ${e.message}`);
      });
    }

    console.log(`\nAll Errors (${result.allErrors.length}):`);
    if (result.allErrors.length === 0) {
      console.log('  (none)');
    } else {
      result.allErrors.forEach(e => {
        console.log(`  Line ${e.line}: TS${e.code}: ${e.message.split('\n')[0]}`);
      });
    }
  } catch (err) {
    console.error(`Error analyzing ${filePath}:`, err.message);
  }
}
