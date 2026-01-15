#!/usr/bin/env node
/**
 * Check what WASM reports for specific test files
 */

import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, join, resolve, basename } from 'path';
import { readFileSync } from 'fs';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const wasmPkgPath = resolve(__dirname, '../pkg');

async function analyzeWithWasm(filePath) {
  const wasm = await import(join(wasmPkgPath, 'wasm.js'));
  const code = readFileSync(filePath, 'utf-8');

  // Extract test directives
  const lines = code.split('\n');
  const testOpts = {};
  let cleanCode = code;
  for (let i = 0; i < Math.min(20, lines.length); i++) {
    const line = lines[i].trim();
    if (line.startsWith('// @')) {
      const match = line.match(/\/\/ @(\w+):\s*(.+)/);
      if (match) testOpts[match[1]] = match[2];
    }
  }

  const fileName = basename(filePath);
  const parser = new wasm.ThinParser(fileName, code);
  parser.parseSourceFile();
  const parseDiags = JSON.parse(parser.getDiagnosticsJson());
  const checkResult = JSON.parse(parser.checkSourceFile());

  const allDiags = [
    ...parseDiags.map(d => ({ code: d.code, message: d.message, line: d.line || d.start?.line })),
    ...(checkResult.diagnostics || []).map(d => ({ code: d.code, message: d.message, line: d.line || d.start?.line })),
  ];

  const ts2322 = allDiags.filter(d => d.code === 2322);

  return {
    has2322: ts2322.length > 0,
    ts2322: ts2322.map(d => ({
      code: d.code,
      message: d.message,
    })),
    allDiagnostics: allDiags.map(d => `TS${d.code}`).join(', '),
  };
}

// Main execution
const filePaths = process.argv.slice(2);

if (filePaths.length === 0) {
  console.error('Usage: node analyze-wasm.mjs <file1.ts> [file2.ts] ...');
  process.exit(1);
}

for (const filePath of filePaths) {
  try {
    console.log(`\n${filePath}:`);
    const result = await analyzeWithWasm(filePath);
    console.log(`  Has TS2322: ${result.has2322}`);
    if (result.ts2322.length > 0) {
      console.log(`  TS2322 count: ${result.ts2322.length}`);
      result.ts2322.forEach(e => {
        console.log(`    - ${e.message}`);
      });
    }
    console.log(`  All errors: ${result.allDiagnostics || '(none)'}`);
  } catch (err) {
    console.error(`  Error:`, err.message);
  }
}
