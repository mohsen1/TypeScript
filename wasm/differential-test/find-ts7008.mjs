#!/usr/bin/env node
/**
 * Find TS7008 (implicit any member) differences.
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

async function main() {
  const args = process.argv.slice(2);
  const maxTests = parseInt(args.find(a => a.startsWith('--max='))?.split('=')[1] || '1000', 10);
  const maxSamples = parseInt(args.find(a => a.startsWith('--samples='))?.split('=')[1] || '20', 10);

  const wasm = await import(join(CONFIG.wasmPkgPath, 'wasm.js'));
  const testFiles = getTestFiles(CONFIG.conformanceDir, maxTests);

  let extraCount = 0;
  let missingCount = 0;
  const extraFiles = [];
  const missingFiles = [];

  for (const filePath of testFiles) {
    let code;
    try {
      code = readFileSync(filePath, 'utf-8');
    } catch {
      continue;
    }

    // Skip multi-file tests
    if (code.includes('@filename:')) continue;

    // TSC
    let sourceFile;
    try {
      sourceFile = ts.createSourceFile(filePath, code, ts.ScriptTarget.ES2020, true);
    } catch {
      continue;
    }
    const host = ts.createCompilerHost({ strict: true, noEmit: true, noImplicitAny: true });
    const originalGetSourceFile = host.getSourceFile;
    host.getSourceFile = (name, langVersion, onError) => {
      if (name === filePath) return sourceFile;
      return originalGetSourceFile.call(host, name, langVersion, onError);
    };
    let tscDiags;
    try {
      const program = ts.createProgram([filePath], { strict: true, noEmit: true, noImplicitAny: true }, host);
      tscDiags = [...program.getSemanticDiagnostics(sourceFile)];
    } catch {
      continue;
    }

    // WASM
    let checkResult;
    try {
      const parser = new wasm.ThinParser(filePath, code);
      parser.parseSourceFile();
      checkResult = JSON.parse(parser.checkSourceFile());
    } catch {
      continue;
    }

    const tscCodes = new Set(tscDiags.map(d => d.code));
    const wasmCodes = new Set((checkResult.diagnostics || []).map(d => d.code));

    const extraCodes = [...wasmCodes].filter(c => !tscCodes.has(c));
    const missingCodes = [...tscCodes].filter(c => !wasmCodes.has(c));

    const relPath = filePath.replace(CONFIG.conformanceDir + '/', '');

    if (extraCodes.includes(7008)) {
      extraCount++;
      if (extraFiles.length < maxSamples) {
        extraFiles.push(relPath);
      }
    }
    if (missingCodes.includes(7008)) {
      missingCount++;
      if (missingFiles.length < maxSamples) {
        missingFiles.push(relPath);
      }
    }
  }

  console.log('=== TS7008 Implicit Any Member ===');
  console.log(`Extra (false positives): ${extraCount}`);
  console.log(`Missing (not detected): ${missingCount}`);
  console.log('\nSample extra files:');
  for (const f of extraFiles.slice(0, 10)) {
    console.log(`  ${f}`);
  }
  console.log('\nSample missing files:');
  for (const f of missingFiles.slice(0, 10)) {
    console.log(`  ${f}`);
  }
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
