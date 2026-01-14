#!/usr/bin/env node
/**
 * Find all test files that have TS1005 extra errors in WASM
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
  conformanceDir: resolve('/tmp/orchestrator-workspace/worktrees/worker-6/tests/cases/conformance'),
};
const DEFAULT_LIB_PATH = resolve('/tmp/orchestrator-workspace/worktrees/worker-6/tests/lib/lib.d.ts');
const DEFAULT_LIB_SOURCE = readFileSync(DEFAULT_LIB_PATH, 'utf8');
const DEFAULT_LIB_NAME = 'lib.d.ts';

// Recursively get all .ts files
function getTestFiles(dir, maxFiles = 500) {
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
  const files = [];
  let currentFileName = null;
  let currentFileLines = [];

  for (const line of lines) {
    const trimmed = line.trim();

    const filenameMatch = trimmed.match(/^\/\/\s*@filename:\s*(.+)$/);
    if (filenameMatch) {
      isMultiFile = true;
      if (currentFileName) {
        files.push({ name: currentFileName, content: currentFileLines.join('\n') });
      }
      currentFileName = filenameMatch[1].trim();
      currentFileLines = [];
      continue;
    }

    const match = trimmed.match(/^\/\/\s*@(\w+):\s*(.+)$/);
    if (match) {
      let [, key, value] = match;
      value = value === 'true' ? true : value === 'false' ? false : value;
      options[key.toLowerCase()] = value;
      continue;
    }

    (currentFileLines || cleanLines).push(line);
  }

  if (currentFileName) {
    files.push({ name: currentFileName, content: currentFileLines.join('\n') });
  }

  return {
    options,
    isMultiFile,
    cleanCode: isMultiFile ? null : cleanLines.join('\n'),
    files,
  };
}

async function runWasm(code, fileName = 'test.ts', testOptions = {}) {
  try {
    const wasm = await import(join(CONFIG.wasmPkgPath, 'wasm.js'));

    const parser = new wasm.ThinParser(fileName, code);
    if (!testOptions.nolib) {
      parser.addLibFile(DEFAULT_LIB_NAME, DEFAULT_LIB_SOURCE);
    }
    parser.parseSourceFile();

    const parseDiagsJson = parser.getDiagnosticsJson();
    const parseDiags = JSON.parse(parseDiagsJson);

    const checkResultJson = parser.checkSourceFile();
    const checkResult = JSON.parse(checkResultJson);

    const allDiagnostics = [
      ...parseDiags.map(d => ({
        code: d.code,
        message: d.message,
        category: 'Error',
        source: 'parser',
      })),
      ...(checkResult.diagnostics || []).map(d => ({
        code: d.code,
        message: d.message_text,
        category: d.category,
        source: 'checker',
      })),
    ];

    parser.free();

    return {
      diagnostics: allDiagnostics,
      crashed: false,
    };
  } catch (e) {
    return {
      diagnostics: [],
      crashed: true,
      error: e.message,
    };
  }
}

async function main() {
  const testFiles = getTestFiles(CONFIG.conformanceDir, 200);

  console.log(`Analyzing ${testFiles.length} test files for TS1005 errors...\n`);

  const ts1005Files = [];

  for (let i = 0; i < testFiles.length; i++) {
    const filePath = testFiles[i];
    const fileName = basename(filePath);
    const relPath = filePath.replace(CONFIG.conformanceDir + '/', '');
    const cat = relPath.split('/')[0];

    const code = readFileSync(filePath, 'utf8');
    const { options, isMultiFile, files, cleanCode } = parseTestDirectives(code);

    let wasmResult;
    if (isMultiFile) {
      // For multi-file tests, just run the main file
      wasmResult = await runWasm(cleanCode, fileName, options);
    } else {
      wasmResult = await runWasm(cleanCode, fileName, options);
    }

    // Check for TS1005 extra errors
    const ts1005Errors = (wasmResult.diagnostics || []).filter(d => d.code === 1005);

    if (ts1005Errors.length > 0) {
      ts1005Files.push({
        file: relPath,
        category: cat,
        count: ts1005Errors.length,
        errors: ts1005Errors,
      });
    }

    process.stdout.write(`\r  Progress: ${i + 1}/${testFiles.length}`);
  }

  console.log('\n');
  console.log('='.repeat(80));
  console.log(`Found ${ts1005Files.length} files with TS1005 extra errors:\n`);

  // Group by category
  const byCategory = {};
  for (const item of ts1005Files) {
    if (!byCategory[item.category]) {
      byCategory[item.category] = [];
    }
    byCategory[item.category].push(item);
  }

  // Print by category
  for (const [category, items] of Object.entries(byCategory)) {
    console.log(`\n## ${category} (${items.length} files, ${items.reduce((sum, i) => sum + i.count, 0)} errors)\n`);

    for (const item of items) {
      console.log(`  ${item.file} (${item.count} error${item.count > 1 ? 's' : ''})`);
      for (const err of item.errors) {
        console.log(`    - TS${err.code}: ${err.message}`);
      }
    }
  }

  console.log('\n' + '='.repeat(80));
  console.log(`\nTotal: ${ts1005Files.reduce((sum, i) => sum + i.count, 0)} TS1005 errors across ${ts1005Files.length} files\n`);
}

main().catch(console.error);
