#!/usr/bin/env node
/**
 * Find EXTRA TS2322 errors in conformance tests.
 * Reports where WASM has TS2322 but TypeScript doesn't.
 */

import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, join, resolve, basename } from 'path';
import { readFileSync, readdirSync, statSync } from 'fs';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const CONFIG = {
  wasmPkgPath: resolve(__dirname, '../pkg'),
  conformanceDir: resolve(__dirname, '../../tests/cases/conformance'),
};

const ts = require('typescript');

function getTestFiles(dir, maxFiles) {
  const files = [];
  function walk(currentDir) {
    if (files.length >= maxFiles) return;
    const entries = readdirSync(currentDir);
    for (const entry of entries) {
      if (files.length >= maxFiles) break;
      const fullPath = join(currentDir, entry);
      const stat = statSync(fullPath);
      if (stat.isDirectory()) {
        walk(fullPath);
      } else if (entry.endsWith('.ts') && !entry.endsWith('.d.ts')) {
        files.push(fullPath);
      }
    }
  }
  walk(dir);
  return files;
}

function parseTestDirectives(code) {
  const lines = code.split('\n');
  const options = {};
  for (let i = 0; i < Math.min(20, lines.length); i++) {
    const line = lines[i].trim();
    if (line.startsWith('// @')) {
      const match = line.match(/\/\/ @(\w+):\s*(.+)/);
      if (match) options[match[1]] = match[2];
    }
  }
  return { options, cleanCode: code };
}

function buildCompilerOptions(testOpts) {
  const opts = { noEmit: true, skipLibCheck: true };
  if (testOpts.strict) opts.strict = true;
  if (testOpts.target) opts.target = ts.ScriptTarget[testOpts.target.toUpperCase()] || ts.ScriptTarget.ES2020;
  if (testOpts.lib) opts.lib = testOpts.lib.split(',').map(l => l.trim());
  return opts;
}

function runTscSingle(code, fileName, testOptions) {
  const compilerOptions = buildCompilerOptions(testOptions);
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
  return allDiagnostics.map(d => ({ code: d.code }));
}

function runWasmSingle(code, fileName, wasm) {
  const parser = new wasm.ThinParser(fileName, code);
  parser.parseSourceFile();
  const parseDiags = JSON.parse(parser.getDiagnosticsJson());
  const checkResult = JSON.parse(parser.checkSourceFile());
  let wasmDiags = [
    ...parseDiags.map(d => ({ code: d.code })),
    ...(checkResult.diagnostics || []).map(d => ({ code: d.code })),
  ];
  return wasmDiags;
}

async function main() {
  const args = process.argv.slice(2);
  const maxTests = parseInt(args.find(a => a.startsWith('--max='))?.split('=')[1] || '500', 10);

  const wasm = await import(join(CONFIG.wasmPkgPath, 'wasm.js'));
  const testFiles = getTestFiles(CONFIG.conformanceDir, maxTests);

  const extra = [];

  for (const filePath of testFiles) {
    let rawCode;
    try {
      rawCode = readFileSync(filePath, 'utf-8');
    } catch {
      continue;
    }

    const { options, cleanCode } = parseTestDirectives(rawCode);
    let tscDiags = [];
    let wasmDiags = [];

    try {
      const fileName = basename(filePath);
      tscDiags = runTscSingle(cleanCode, fileName, options);
      wasmDiags = runWasmSingle(cleanCode, fileName, wasm);
    } catch {
      continue;
    }

    const tscHas2322 = tscDiags.some(d => d.code === 2322);
    const wasmHas2322 = wasmDiags.some(d => d.code === 2322);

    if (!tscHas2322 && wasmHas2322) {
      const relPath = filePath.replace(CONFIG.conformanceDir + '/', '');
      extra.push(relPath);
    }
  }

  console.log(`\n=== EXTRA TS2322 DIAGNOSTICS ===`);
  console.log(`Total files with extra TS2322: ${extra.length}`);
  if (extra.length > 0) {
    console.log('\nFiles:');
    extra.forEach(f => console.log(`  - ${f}`));
  }
  process.exit(0);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
