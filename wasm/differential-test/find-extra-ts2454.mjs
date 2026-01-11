#!/usr/bin/env node
import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, join, resolve } from 'path';
import { readFileSync, readdirSync, statSync } from 'fs';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const CONFIG = {
  wasmPkgPath: resolve(__dirname, '../pkg'),
  conformanceDir: resolve(__dirname, '../../tests/cases/conformance'),
};

async function main() {
  const wasm = await import(join(CONFIG.wasmPkgPath, 'wasm.js'));
  const ts = require('typescript');

  const testFiles = [];
  function walk(dir) {
    try {
      const entries = readdirSync(dir);
      for (const entry of entries) {
        const fullPath = join(dir, entry);
        try {
          const stat = statSync(fullPath);
          if (stat.isDirectory()) {
            walk(fullPath);
          } else if (entry.endsWith('.ts') && !entry.endsWith('.d.ts')) {
            testFiles.push(fullPath);
          }
        } catch (e) {}
      }
    } catch (e) {}
  }

  const controlFlowDir = join(CONFIG.conformanceDir, 'controlFlow');
  walk(controlFlowDir);

  const extraFiles = [];
  
  for (const filePath of testFiles.slice(0, 30)) {
    const source = readFileSync(filePath, 'utf8');
    const relPath = filePath.replace(CONFIG.conformanceDir + '/', '');
    
    const tscResult = ts.compileSourceFile(filePath, source, {
      strict: true,
      noEmit: true,
    });
    
    const tscDiags = tscResult.diagnostics || [];
    const tsc2454 = tscDiags.filter(d => d.code === 2454).length;
    
    const wasmResult = wasm.check(filePath, source);
    const wasmDiags = JSON.parse(wasmResult.diagnostics || '[]');
    const wasm2454 = wasmDiags.filter(d => d.code === 2454).length;
    
    if (wasm2454 > tsc2454) {
      console.log(`\n=== EXTRA TS2454: ${relPath} ===`);
      console.log(`TSC: ${tsc2454}, WASM: ${wasm2454}`);
      
      const extra2454 = wasmDiags.filter(d => d.code === 2454);
      for (const err of extra2454.slice(0, 2)) {
        console.log(`  Line ${err.start_line}: ${err.message}`);
      }
      console.log('Code:');
      console.log(source.slice(0, 300));
      
      extraFiles.push(relPath);
    }
  }
  
  console.log(`\n=== SUMMARY ===`);
  console.log(`Files with extra TS2454: ${extraFiles.length}`);
}

main().catch(err => { console.error(err); process.exit(1); });
