#!/usr/bin/env node
/**
 * Find TS7011 errors in conformance tests.
 */

import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, join, resolve, basename } from 'path';
import { readFileSync, readdirSync, statSync } from 'fs';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const ts = require('typescript');
const wasm = require(join(__dirname, '../pkg/wasm.js'));

const CONFIG = {
  wasmPkgPath: resolve(__dirname, '../pkg'),
  conformanceDir: resolve(__dirname, '../../tests/cases/conformance'),
};

function getTestFiles(dir, maxFiles = 200) {
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
  return allDiagnostics.map(d => d.code);
}

function runWasm(code, fileName) {
  const parser = new wasm.ThinParser(fileName, code);
  parser.parseSourceFile();
  const checkResult = JSON.parse(parser.checkSourceFile());
  return (checkResult.diagnostics || []).map(d => d.code);
}

async function main() {
  const testFiles = getTestFiles(CONFIG.conformanceDir, 200);
  const extraMatches = [];

  for (const filePath of testFiles) {
    let rawCode;
    try {
      rawCode = readFileSync(filePath, 'utf-8');
    } catch { continue; }

    const { options, cleanCode } = parseTestDirectives(rawCode);
    let tscDiags = [], wasmDiags = [];

    try {
      const fileName = basename(filePath);
      tscDiags = runTsc(cleanCode, fileName);
      wasmDiags = runWasm(cleanCode, fileName, wasm);
    } catch { continue; }

    const tscCodes = new Set(tscDiags);
    const wasmCodes = new Set(wasmDiags);
    const extraCodes = [...wasmCodes].filter(code => !tscCodes.has(code));

    if (extraCodes.includes(7011)) {
      const relPath = filePath.replace(CONFIG.conformanceDir + '/', '');
      extraMatches.push(relPath);
      console.log(`\n=== EXTRA TS7011: ${relPath} ===`);
      console.log('Code snippet (first 50 lines):');
      const lines = cleanCode.split('\n');
      console.log(lines.slice(0, 50).map((l, i) => `${i + 1}: ${l}`).join('\n'));
    }

    if (extraMatches.length >= 8) break;
  }

  console.log('\n\n=== SUMMARY ===');
  console.log(`Total files with extra TS7011: ${extraMatches.length}`);
  for (const m of extraMatches) {
    console.log(`  ${m}`);
  }
}

main().catch(console.error);
