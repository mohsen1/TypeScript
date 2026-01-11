#!/usr/bin/env node
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

function getTestFiles(dir) {
  const files = [];
  const walk = (current) => {
    const entries = readdirSync(current);
    for (const entry of entries) {
      const full = join(current, entry);
      if (statSync(full).isDirectory()) walk(full);
      else if (entry.endsWith('.ts')) files.push(full);
    }
  };
  walk(dir);
  return files;
}

async function main() {
  const wasm = await import(join(CONFIG.wasmPkgPath, 'wasm.js'));
  const testFiles = getTestFiles(CONFIG.conformanceDir);

  let falsePositives = [];
  let checked = 0;
  const maxFiles = 3000; // Limit to 3000 files for speed

  console.log(`Scanning for TS7011 (and TS7010) False Positives (max ${maxFiles} files)...`);

  for (const filePath of testFiles) {
    if (checked >= maxFiles) break;

    const code = readFileSync(filePath, 'utf-8');
    // Skip multi-file tests for simplicity
    if (code.includes('@filename')) {
      checked++;
      continue;
    }

    // Run TSC - wrap in try/catch for debug failures
    let tscHas7010 = false;
    let tscHas7011 = false;
    let tscError = false;
    try {
      const sf = ts.createSourceFile(basename(filePath), code, ts.ScriptTarget.ESNext, true);
      const host = ts.createCompilerHost({ noImplicitAny: true, noEmit: true });
      host.getSourceFile = (name) => name === basename(filePath) ? sf : undefined;
      const program = ts.createProgram([basename(filePath)], { noImplicitAny: true, noEmit: true }, host);
      const tscDiags = ts.getPreEmitDiagnostics(program);
      tscHas7010 = tscDiags.some(d => d.code === 7010);
      tscHas7011 = tscDiags.some(d => d.code === 7011);
    } catch (e) {
      // Skip files that cause TSC internal errors
      tscError = true;
    }

    // Skip if TSC had internal error
    if (tscError) {
      checked++;
      continue;
    }

    // Run WASM
    const parser = new wasm.ThinParser(basename(filePath), code);
    parser.parseSourceFile();
    const checkResult = JSON.parse(parser.checkSourceFile());
    const wasmHas7010 = checkResult.diagnostics.some(d => d.code === 7010);
    const wasmHas7011 = checkResult.diagnostics.some(d => d.code === 7011);
    parser.free();

    // Check for false positives: WASM reports error but TSC doesn't
    if ((wasmHas7010 && !tscHas7010) || (wasmHas7011 && !tscHas7011)) {
      falsePositives.push({
        file: filePath.replace(CONFIG.conformanceDir, ''),
        code7010: wasmHas7010 && !tscHas7010,
        code7011: wasmHas7011 && !tscHas7011,
        snippet: code.slice(0, 300),
      });
    }

    checked++;
    if (checked % 500 === 0) {
      console.log(`Checked ${checked} files, found ${falsePositives.length} false positives...`);
    }
  }

  console.log(`\n=== Found ${falsePositives.length} false positives (checked ${checked} files) ===`);
  for (const fp of falsePositives) {
    const codes = [];
    if (fp.code7010) codes.push('TS7010');
    if (fp.code7011) codes.push('TS7011');
    console.log(`\n[${codes.join(', ')}] ${fp.file}`);
    console.log(fp.snippet + '...');
  }
}

main().catch(console.error);
