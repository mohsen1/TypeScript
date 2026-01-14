#!/usr/bin/env node
/**
 * Find TS1005 false positives in conformance tests.
 * Compares WASM diagnostics to tsc, supports multi-file tests.
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
      const [, key, value] = match;
      if (value === 'true') options[key.toLowerCase()] = true;
      else if (value === 'false') options[key.toLowerCase()] = false;
      else if (!isNaN(Number(value))) options[key.toLowerCase()] = Number(value);
      else options[key.toLowerCase()] = value;
      continue;
    }

    if (isMultiFile && currentFileName) {
      currentFileLines.push(line);
    } else {
      cleanLines.push(line);
    }
  }

  if (isMultiFile && currentFileName) {
    files.push({ name: currentFileName, content: currentFileLines.join('\n') });
  }

  return {
    options,
    cleanCode: isMultiFile ? null : cleanLines.join('\n'),
    files: isMultiFile ? files : [{ name: 'test.ts', content: cleanLines.join('\n') }],
  };
}

function getTscDiagnostics(code, fileName) {
  const sourceFile = ts.createSourceFile(fileName, code, ts.ScriptTarget.Latest, true);
  const diagnostics = [];

  for (const diagnostic of sourceFile.parseDiagnostics) {
    diagnostics.push({
      code: diagnostic.code,
      message: ts.flattenDiagnosticMessageText(diagnostic.messageText, '\n'),
      start: diagnostic.start,
      length: diagnostic.length,
    });
  }

  return diagnostics;
}

function getWasmDiagnostics(code, fileNames) {
  const wasm = require(CONFIG.wasmPkgPath);
  const diagnostics = [];

  for (const fileName of fileNames) {
    const result = wasm.parse(fileName, code);
    if (result.diagnostics) {
      for (const diag of result.diagnostics) {
        diagnostics.push({
          code: diag.code,
          message: diag.message,
          start: diag.span?.start,
          length: diag.span?.length,
        });
      }
    }
  }

  return diagnostics;
}

function analyzeFile(filePath, maxFiles) {
  const code = readFileSync(filePath, 'utf-8');
  const { options, cleanCode, files } = parseTestDirectives(code);

  if (options.skip === true) return null;

  // Get tsc diagnostics
  const tscDiags = new Map();
  for (const file of files) {
    const diags = getTscDiagnostics(file.content, file.name);
    for (const diag of diags) {
      if (diag.code === 1005) {
        const key = `${file.name}:${diag.start}`;
        tscDiags.set(key, diag);
      }
    }
  }

  // Get WASM diagnostics
  const wasmDiags = new Map();
  for (const file of files) {
    const diags = getWasmDiagnostics(file.content, [file.name]);
    for (const diag of diags) {
      if (diag.code === 1005) {
        const key = `${file.name}:${diag.start}`;
        wasmDiags.set(key, diag);
      }
    }
  }

  // Compare
  const falsePositives = [];
  const falseNegatives = [];

  for (const [key, diag] of wasmDiags) {
    if (!tscDiags.has(key)) {
      falsePositives.push(diag);
    }
  }

  for (const [key, diag] of tscDiags) {
    if (!wasmDiags.has(key)) {
      falseNegatives.push(diag);
    }
  }

  return {
    file: basename(filePath),
    falsePositives: falsePositives.length,
    falseNegatives: falseNegatives.length,
    tscCount: tscDiags.size,
    wasmCount: wasmDiags.size,
  };
}

async function main() {
  const maxFiles = 10000;
  const testFiles = getTestFiles(CONFIG.conformanceDir, maxFiles);

  console.log(`Analyzing ${testFiles.length} test files for TS1005...`);

  let totalFalsePositives = 0;
  let totalFalseNegatives = 0;
  let totalTscErrors = 0;
  let totalWasmErrors = 0;

  const worstOffenders = [];

  for (const filePath of testFiles) {
    const result = analyzeFile(filePath, maxFiles);
    if (!result) continue;

    totalFalsePositives += result.falsePositives;
    totalFalseNegatives += result.falseNegatives;
    totalTscErrors += result.tscCount;
    totalWasmErrors += result.wasmCount;

    if (result.falsePositives > 0) {
      worstOffenders.push(result);
    }
  }

  console.log('\n=== TS1005 Analysis Results ===\n');
  console.log(`Test files analyzed: ${testFiles.length}`);
  console.log(`Total tsc TS1005 errors: ${totalTscErrors}`);
  console.log(`Total WASM TS1005 errors: ${totalWasmErrors}`);
  console.log(`False positives (WASM but not tsc): ${totalFalsePositives}`);
  console.log(`False negatives (tsc but not WASM): ${totalFalseNegatives}`);

  console.log('\n=== Top 10 Worst Offenders ===');
  worstOffenders
    .sort((a, b) => b.falsePositives - a.falsePositives)
    .slice(0, 10)
    .forEach((r) => {
      console.log(`${r.file}: +${r.falsePositives} (WASM: ${r.wasmCount}, tsc: ${r.tscCount})`);
    });
}

main().catch(console.error);
