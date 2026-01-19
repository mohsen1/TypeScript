#!/usr/bin/env node
/**
 * Find files with extra TS1109 errors (WASM has, TSC doesn't)
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
  const testFiles = getTestFiles(CONFIG.conformanceDir, 1000);
  const extraTS1109Files = [];

  console.log(`Analyzing ${testFiles.length} test files for extra TS1109 errors...`);

  for (const filePath of testFiles) {
    const code = readFileSync(filePath, 'utf-8');
    const fileName = basename(filePath);

    try {
      const tscDiags = ts.createSourceFile(fileName, code, ts.ScriptTarget.Latest, true).parseDiagnostics;
      const tscTS1109 = tscDiags.filter(d => d.code === 1109).length;

      const parser = new wasm.ThinParser(fileName, code);
      parser.parseSourceFile();
      const wasmDiags = JSON.parse(parser.getDiagnosticsJson());
      parser.free();
      const wasmTS1109 = wasmDiags.filter(d => d.code === 1109).length;

      if (wasmTS1109 > tscTS1109) {
        extraTS1109Files.push({
          file: fileName.replace(/\.ts$/, ''),
          tsc: tscTS1109,
          wasm: wasmTS1109,
          extra: wasmTS1109 - tscTS1109
        });
      }
    } catch (e) {
      // Skip files that cause TSC to crash
    }
  }

  console.log('\n=== Files with extra TS1109 errors (Top 15) ===');
  extraTS1109Files.sort((a, b) => b.extra - a.extra).slice(0, 15).forEach(f => {
    console.log(`  ${f.file}: +${f.extra} (WASM: ${f.wasm}, TSC: ${f.tsc})`);
  });
  console.log(`\nTotal files with extra TS1109: ${extraTS1109Files.length}`);
  console.log(`Total extra TS1109 errors: ${extraTS1109Files.reduce((sum, f) => sum + f.extra, 0)}`);
}

main().catch(console.error);
