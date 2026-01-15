#!/usr/bin/env node
/**
 * Compare WASM vs TSC diagnostics for a single file
 */

import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, join, resolve, basename } from 'path';
import { readFileSync } from 'fs';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const ts = require('typescript');

const CONFIG = {
  wasmPkgPath: resolve(__dirname, '../pkg'),
};

function runTscSingle(code, fileName, testOpts) {
  const opts = { noEmit: true, skipLibCheck: true, target: ts.ScriptTarget.ESNext };
  if (testOpts.strict) opts.strict = true;
  if (testOpts.target) opts.target = ts.ScriptTarget[testOpts.target.toUpperCase()] || ts.ScriptTarget.ESNext;

  const sf = ts.createSourceFile(fileName, code, ts.ScriptTarget.ESNext, true);
  const host = ts.createCompilerHost(opts);
  const program = ts.createProgram([fileName], opts, {
    ...host,
    getSourceFile: (name) => name === fileName ? sf : host.getSourceFile(name, ts.ScriptTarget.ESNext),
  });
  const allDiagnostics = [
    ...program.getSyntacticDiagnostics(sf),
    ...program.getSemanticDiagnostics(sf),
  ];
  return allDiagnostics.map(d => ({
    code: d.code,
    message: ts.flattenDiagnosticMessageText(d.messageText, '\n'),
    line: d.start ? sf.getLineAndCharacterOfPosition(d.start).line + 1 : null,
  }));
}

function runWasmSingle(code, fileName, wasm) {
  const parser = new wasm.ThinParser(fileName, code);
  parser.parseSourceFile();
  const parseDiags = JSON.parse(parser.getDiagnosticsJson());
  const checkResult = JSON.parse(parser.checkSourceFile());
  const diags = [
    ...parseDiags.map(d => ({ code: d.code, message: d.message || d.messageText || '', line: d.line })),
    ...(checkResult.diagnostics || []).map(d => ({ code: d.code, message: d.message || d.messageText || '', line: d.line })),
  ];
  // Debug: show full checkResult diagnostics
  if (process.argv.includes('--debug')) {
    console.log('\nRaw WASM diagnostics:');
    console.log(JSON.stringify(checkResult.diagnostics || [], null, 2));
  }
  return diags;
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
  return options;
}

async function main() {
  const wasm = await import(join(CONFIG.wasmPkgPath, 'wasm.js'));

  const filePath = process.argv[2];
  if (!filePath) {
    console.error('Usage: node compare-single.mjs <file.ts>');
    process.exit(1);
  }

  const code = readFileSync(filePath, 'utf-8');
  const fileName = basename(filePath);
  const testOpts = parseTestDirectives(code);

  console.log('=== TSC Diagnostics ===');
  const tscDiags = runTscSingle(code, fileName, testOpts);
  for (const d of tscDiags) {
    console.log('  Line ' + d.line + ': TS' + d.code + ' - ' + (d.message ? d.message.substring(0, 80) : ''));
  }
  console.log('  Total: ' + tscDiags.length);

  console.log('\n=== WASM Diagnostics ===');
  const wasmDiags = runWasmSingle(code, fileName, wasm);
  for (const d of wasmDiags) {
    console.log('  Line ' + d.line + ': TS' + d.code + ' - ' + (d.message ? d.message.substring(0, 80) : ''));
  }
  console.log('  Total: ' + wasmDiags.length);

  const tsc2322 = tscDiags.filter(d => d.code === 2322);
  const wasm2322 = wasmDiags.filter(d => d.code === 2322);
  console.log('\n=== TS2322 Comparison ===');
  console.log('TSC TS2322: ' + tsc2322.length);
  console.log('WASM TS2322: ' + wasm2322.length);

  if (wasm2322.length > tsc2322.length) {
    console.log('\nExtra TS2322 errors from WASM:');
    for (const d of wasm2322) {
      console.log('  Line ' + d.line + ': ' + (d.message || ''));
    }
  }
}

main().catch(console.error);
