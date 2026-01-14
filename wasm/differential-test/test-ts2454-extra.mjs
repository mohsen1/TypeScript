#!/usr/bin/env node
import { createRequire } from 'module';
import { join, basename } from 'path';
import { readFileSync, readdirSync, statSync } from 'fs';

const require = createRequire(import.meta.url);
const ts = require('typescript');
const wasm = require('../pkg/wasm.js');

const dir = './tests/cases/conformance';

function getFiles(dir, files = [], max = 3000) {
  if (files.length >= max) return files;
  try {
    for (const e of readdirSync(dir)) {
      if (files.length >= max) break;
      const p = join(dir, e);
      try {
        if (statSync(p).isDirectory()) getFiles(p, files, max);
        else if (e.endsWith('.ts') && !e.endsWith('.d.ts')) files.push(p);
      } catch {}
    }
  } catch {}
  return files;
}

function runTsc(code, fileName) {
  const sf = ts.createSourceFile(fileName, code, ts.ScriptTarget.ES2020, true);
  const host = ts.createCompilerHost({ strict: true, noEmit: true, skipLibCheck: true });
  const origGet = host.getSourceFile;
  host.getSourceFile = (name) => name === fileName ? sf : origGet.call(host, name);
  const prog = ts.createProgram([fileName], { strict: true, noEmit: true, skipLibCheck: true }, host);
  return [...prog.getSyntacticDiagnostics(sf), ...prog.getSemanticDiagnostics(sf)];
}

function runWasm(code, fileName) {
  const parser = new wasm.ThinParser(fileName, code);
  parser.parseSourceFile();
  const parseDiags = JSON.parse(parser.getDiagnosticsJson());
  const checkResult = JSON.parse(parser.checkSourceFile());
  return [...parseDiags, ...(checkResult.diagnostics || [])];
}

const files = getFiles(dir);
console.log('Found', files.length, 'test files');
let total = 0, extraTotal = 0;
const extras = [];

for (const f of files) {
  let code;
  try { code = readFileSync(f, 'utf-8'); } catch { continue; }
  if (code.includes('@filename:')) continue;

  const name = basename(f);
  let tscD, wasmD;
  try {
    tscD = runTsc(code, name);
    wasmD = runWasm(code, name);
  } catch (e) { continue; }

  const tsc2454 = tscD.filter(d => d.code === 2454).length;
  const wasm2454 = wasmD.filter(d => d.code === 2454).length;

  if (wasm2454 > tsc2454) {
    const extra = wasm2454 - tsc2454;
    extraTotal += extra;
    extras.push({ path: f, tsc: tsc2454, wasm: wasm2454, extra });
  }
  total++;
}

console.log('\n=== SUMMARY ===');
console.log('Files checked:', total);
console.log('Files with extra 2454:', extras.length);
console.log('Total extra 2454 errors:', extraTotal);
extras.sort((a, b) => b.extra - a.extra);
console.log('\nTop 30 files with extra TS2454:');
for (const e of extras.slice(0, 30)) {
  console.log(' ', e.extra, 'extra:', e.path.replace('./tests/cases/conformance/', ''));
}
