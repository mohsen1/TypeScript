#!/usr/bin/env node
/**
 * Find files with extra TS1005 errors (WASM has, TSC doesn't)
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
const wasm = require(CONFIG.wasmPkgPath);

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
  const testFiles = getTestFiles(CONFIG.conformanceDir, 200);
  const extraTS1005Files = [];

  console.log(`Analyzing ${testFiles.length} test files for extra TS1005 errors...`);

  for (const filePath of testFiles) {
    const code = readFileSync(filePath, 'utf-8');
    const fileName = basename(filePath);

    try {
      const tscDiags = ts.createSourceFile(fileName, code, ts.ScriptTarget.Latest, true).parseDiagnostics;
      const tscTS1005 = tscDiags.filter(d => d.code === 1005).length;

      const parser = new wasm.ThinParser(fileName, code);
      parser.parseSourceFile();
      const wasmDiags = JSON.parse(parser.getDiagnosticsJson());
      parser.free();
      const wasmTS1005 = wasmDiags.filter(d => d.code === 1005).length;

      if (wasmTS1005 > tscTS1005) {
        extraTS1005Files.push({
          file: fileName.replace(/\.ts$/, ''),
          tsc: tscTS1005,
          wasm: wasmTS1005,
          extra: wasmTS1005 - tscTS1005
        });
      }
    } catch (e) {
      // Skip files that cause TSC to crash
    }
  }

  console.log('\n=== Files with extra TS1005 errors (Top 15) ===');
  extraTS1005Files.sort((a, b) => b.extra - a.extra).slice(0, 15).forEach(f => {
    console.log(`  ${f.file}: +${f.extra} (WASM: ${f.wasm}, TSC: ${f.tsc})`);
  });
  console.log(`\nTotal files with extra TS1005: ${extraTS1005Files.length}`);
  console.log(`Total extra TS1005 errors: ${extraTS1005Files.reduce((sum, f) => sum + f.extra, 0)}`);
}

main().catch(console.error);
