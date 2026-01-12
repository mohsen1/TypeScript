#!/usr/bin/env node
/**
 * Find missing TS2322 errors in conformance tests.
 */

import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, join, resolve, basename } from 'path';
import { readFileSync, readdirSync, statSync } from 'fs';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const ts = require('typescript');
const wasm = require(join(__dirname, 'pkg/wasm.js'));

const CONFIG = {
  wasmPkgPath: resolve(__dirname, 'pkg'),
  conformanceDir: resolve(__dirname, '../tests/cases/conformance'),
};

function getTestFiles(dir, maxFiles = 300) {
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
  const cleanLines = [];

  for (const line of lines) {
    const trimmed = line.trim();
    const match = trimmed.match(/^\/\/\s*@\w+:\s*(.+)$/);
    if (match) {
      const [, key, value] = match;
      if (value === 'true') options[key.toLowerCase()] = true;
      else if (value === 'false') options[key.toLowerCase()] = false;
      else if (!isNaN(Number(value))) options[key.toLowerCase()] = Number(value);
      else options[key.toLowerCase()] = value;
      continue;
    }
    if (!trimmed.startsWith('//@filename')) {
      cleanLines.push(line);
    }
  }

  return { options, cleanCode: cleanLines.join('\n') };
}

function runTsc(code, fileName) {
  const compilerOptions = {
    strict: true,
    target: ts.ScriptTarget.ES2020,
    module: ts.ModuleKind.ESNext,
    noEmit: true,
    skipLibCheck: true,
  };

  const sourceFile = ts.createSourceFile(fileName, code, ts.ScriptTarget.ES2020, true);
  const host = ts.createCompilerHost(compilerOptions);
  const originalGetSourceFile = host.getSourceFile;
  host.getSourceFile = (name) => {
    if (name === fileName) return sourceFile;
    return originalGetSourceFile.call(host, name);
  };

  const program = ts.createProgram([fileName], compilerOptions, host);
  const allDiagnostics = [
    ...program.getSyntacticDiagnostics(sourceFile),
    ...program.getSemanticDiagnostics(sourceFile),
  ];

  return allDiagnostics.map(d => ({ code: d.code, start: d.start, length: d.length }));
}

function runWasm(code, fileName) {
  const parser = new wasm.ThinParser(fileName, code);
  parser.parseSourceFile();
  const checkResult = JSON.parse(parser.checkSourceFile());
  return (checkResult.diagnostics || []).map(d => ({ code: d.code, start: d.start, length: d.length }));
}

async function main() {
  const testFiles = getTestFiles(CONFIG.conformanceDir, 300);
  const missing2322 = [];
  const extra2322 = [];

  for (const filePath of testFiles) {
    let rawCode;
    try {
      rawCode = readFileSync(filePath, 'utf-8');
    } catch { continue; }

    const { options, cleanCode } = parseTestDirectives(rawCode);

    let tscDiags = [];
    let wasmDiags = [];

    try {
      const fileName = basename(filePath);
      tscDiags = runTsc(cleanCode, fileName);
      wasmDiags = runWasm(cleanCode, fileName, wasm);
    } catch { continue; }

    const tsc2322 = tscDiags.filter(d => d.code === 2322);
    const wasm2322 = wasmDiags.filter(d => d.code === 2322);

    // Check for missing TS2322 (tsc has it but wasm doesn't)
    if (tsc2322.length > wasm2322.length) {
      const relPath = filePath.replace(CONFIG.conformanceDir + '/', '');
      const count = tsc2322.length - wasm2322.length;
      missing2322.push({ path: relPath, missing: count, tsc: tsc2322.length, wasm: wasm2322.length });

      if (missing2322.length <= 10) {
        console.log(`\n=== MISSING TS2322: ${relPath} ===`);
        console.log(`TSC: ${tsc2322.length}, WASM: ${wasm2322.length}, Missing: ${count}`);
        console.log('Code snippet (first 30 lines):');
        const lines = cleanCode.split('\n');
        console.log(lines.slice(0, 30).map((l, i) => `${i + 1}: ${l}`).join('\n'));
      }
    }

    // Check for extra TS2322 (wasm has it but tsc doesn't)
    if (wasm2322.length > tsc2322.length) {
      const relPath = filePath.replace(CONFIG.conformanceDir + '/', '');
      const count = wasm2322.length - tsc2322.length;
      extra2322.push({ path: relPath, extra: count, tsc: tsc2322.length, wasm: wasm2322.length });
    }
  }

  console.log('\n\n=== SUMMARY ===');
  console.log(`Files with missing TS2322: ${missing2322.length}`);
  console.log(`Files with extra TS2322: ${extra2322.length}`);

  if (missing2322.length > 0) {
    console.log('\n=== TOP 20 MISSING TS2322 FILES ===');
    missing2322.slice(0, 20).forEach(m => {
      console.log(`  ${m.path} (missing ${m.missing}, tsc=${m.tsc}, wasm=${m.wasm})`);
    });
  }
}

main().catch(console.error);
