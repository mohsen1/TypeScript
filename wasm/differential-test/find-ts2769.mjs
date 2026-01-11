#!/usr/bin/env node
/**
 * Find TS2769 extra or missing diagnostics in conformance tests.
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
  let isMultiFile = false;
  const cleanLines = [];

  for (const line of lines) {
    const trimmed = line.trim();
    const filenameMatch = trimmed.match(/^\/\/\s*@filename:\s*(.+)$/);
    if (filenameMatch) {
      isMultiFile = true;
      continue;
    }
    const match = trimmed.match(/^\/\/\s*@(\w+):\s*(.+)$/);
    if (match) {
      const [, key, value] = match;
      if (value === 'true') options[key.toLowerCase()] = true;
      else if (value === 'false') options[key.toLowerCase()] = false;
      else if (!isNaN(Number(value))) options[key.toLowerCase()] = Number(value);
      else options[key.toLowerCase()] = value;
      continue;
    }
    cleanLines.push(line);
  }

  return { options, isMultiFile, cleanCode: cleanLines.join('\n') };
}

function buildCompilerOptions(testOptions = {}) {
  return {
    strict: testOptions.strict !== false,
    target: ts.ScriptTarget.ES2020,
    module: ts.ModuleKind.ESNext,
    noEmit: true,
    skipLibCheck: true,
  };
}

function runTscSingle(code, fileName, testOptions) {
  const compilerOptions = buildCompilerOptions(testOptions);
  const sourceFile = ts.createSourceFile(fileName, code, ts.ScriptTarget.ES2020, true);

  const host = ts.createCompilerHost(compilerOptions);
  const originalGetSourceFile = host.getSourceFile;
  host.getSourceFile = (name, languageVersion, onError) => {
    if (name === fileName) return sourceFile;
    return originalGetSourceFile.call(host, name, languageVersion, onError);
  };

  const program = ts.createProgram([fileName], compilerOptions, host);
  const allDiagnostics = [
    ...program.getSyntacticDiagnostics(sourceFile),
    ...program.getSemanticDiagnostics(sourceFile),
  ];

  return allDiagnostics.map(d => ({ code: d.code }));
}

function runWasmSingle(code, fileName, wasm) {
  const parser = new wasm.ThinParser(fileName, code);
  parser.parseSourceFile();
  const parseDiags = JSON.parse(parser.getDiagnosticsJson());
  const checkResult = JSON.parse(parser.checkSourceFile());
  return [
    ...parseDiags.map(d => ({ code: d.code })),
    ...(checkResult.diagnostics || []).map(d => ({ code: d.code })),
  ];
}

async function main() {
  const args = process.argv.slice(2);
  const maxTests = parseInt(args.find(a => a.startsWith('--max='))?.split('=')[1] || '2000', 10);
  const maxSamples = parseInt(args.find(a => a.startsWith('--samples='))?.split('=')[1] || '20', 10);

  const wasm = await import(join(CONFIG.wasmPkgPath, 'wasm.js'));
  const testFiles = getTestFiles(CONFIG.conformanceDir, maxTests);

  let extraCount = 0;

  for (const filePath of testFiles) {
    let rawCode;
    try {
      rawCode = readFileSync(filePath, 'utf-8');
    } catch {
      continue;
    }

    const { options, isMultiFile, cleanCode } = parseTestDirectives(rawCode);
    if (isMultiFile) continue;

    let tscDiags = [];
    let wasmDiags = [];

    try {
      const fileName = basename(filePath);
      tscDiags = runTscSingle(cleanCode, fileName, options);
      wasmDiags = runWasmSingle(cleanCode, fileName, wasm);
    } catch (e) {
      continue;
    }

    const tscCodes = new Set(tscDiags.map(d => d.code));
    const wasmCodes = new Set(wasmDiags.map(d => d.code));

    if (wasmCodes.has(2769) && !tscCodes.has(2769)) {
      const relPath = filePath.replace(CONFIG.conformanceDir + '/', '');
      extraCount++;
      if (extraCount <= maxSamples) {
        console.log(`Extra TS2769: ${relPath}`);
      }
    }
  }

  console.log(`\nTotal files with extra TS2769: ${extraCount}`);
  process.exit(0);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
