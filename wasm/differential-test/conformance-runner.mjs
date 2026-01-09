#!/usr/bin/env node
/**
 * Conformance Test Runner - Tests WASM compiler against TypeScript conformance tests
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

const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
  dim: '\x1b[2m',
  bold: '\x1b[1m',
};

function log(msg, color = '') {
  console.log(`${color}${msg}${colors.reset}`);
}

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

/**
 * Parse test directives from source code.
 * Returns { options: Object, isMultiFile: boolean, cleanCode: string }
 */
function parseTestDirectives(code) {
  const lines = code.split('\n');
  const options = {};
  let isMultiFile = false;
  const cleanLines = [];

  for (const line of lines) {
    const trimmed = line.trim();

    // Check for @filename directive (multi-file test)
    if (trimmed.startsWith('// @filename:')) {
      isMultiFile = true;
      break;
    }

    // Parse compiler options like // @strict: true
    const match = trimmed.match(/^\/\/\s*@(\w+):\s*(.+)$/);
    if (match) {
      const [, key, value] = match;
      // Parse boolean/number values
      if (value === 'true') options[key.toLowerCase()] = true;
      else if (value === 'false') options[key.toLowerCase()] = false;
      else if (!isNaN(Number(value))) options[key.toLowerCase()] = Number(value);
      else options[key.toLowerCase()] = value;
      continue; // Don't include directive in clean code
    }

    cleanLines.push(line);
  }

  return {
    options,
    isMultiFile,
    cleanCode: cleanLines.join('\n'),
  };
}

async function runTsc(code, fileName = 'test.ts', testOptions = {}) {
  const ts = require('typescript');

  // Build compiler options from test directives
  const compilerOptions = {
    strict: testOptions.strict !== false, // default true
    target: ts.ScriptTarget.ES2020,
    module: ts.ModuleKind.ESNext,
    noEmit: true,
    skipLibCheck: true,
  };

  // Apply test-specific options
  if (testOptions.target) {
    const targetMap = {
      'es5': ts.ScriptTarget.ES5,
      'es6': ts.ScriptTarget.ES2015,
      'es2015': ts.ScriptTarget.ES2015,
      'es2016': ts.ScriptTarget.ES2016,
      'es2017': ts.ScriptTarget.ES2017,
      'es2018': ts.ScriptTarget.ES2018,
      'es2019': ts.ScriptTarget.ES2019,
      'es2020': ts.ScriptTarget.ES2020,
      'es2021': ts.ScriptTarget.ES2021,
      'es2022': ts.ScriptTarget.ES2022,
      'esnext': ts.ScriptTarget.ESNext,
    };
    compilerOptions.target = targetMap[testOptions.target.toLowerCase()] || ts.ScriptTarget.ES2020;
  }

  if (testOptions.noimplicitany !== undefined) {
    compilerOptions.noImplicitAny = testOptions.noimplicitany;
  }
  if (testOptions.strictnullchecks !== undefined) {
    compilerOptions.strictNullChecks = testOptions.strictnullchecks;
  }

  const sourceFile = ts.createSourceFile(
    fileName,
    code,
    ts.ScriptTarget.ES2020,
    true
  );

  const host = ts.createCompilerHost(compilerOptions);
  const originalGetSourceFile = host.getSourceFile;
  host.getSourceFile = (name, languageVersion, onError) => {
    if (name === fileName) {
      return sourceFile;
    }
    return originalGetSourceFile.call(host, name, languageVersion, onError);
  };

  const program = ts.createProgram([fileName], compilerOptions, host);

  const allDiagnostics = [
    ...program.getSyntacticDiagnostics(sourceFile),
    ...program.getSemanticDiagnostics(sourceFile),
  ];

  return {
    diagnostics: allDiagnostics.map(d => ({
      code: d.code,
      message: ts.flattenDiagnosticMessageText(d.messageText, '\n'),
      category: ts.DiagnosticCategory[d.category],
    })),
  };
}

async function runWasm(code, fileName = 'test.ts') {
  try {
    const wasm = await import(join(CONFIG.wasmPkgPath, 'wasm.js'));

    const parser = new wasm.ThinParser(fileName, code);
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

function compareDiagnostics(tscResult, wasmResult) {
  const tscCodes = new Set(tscResult.diagnostics.map(d => d.code));
  const wasmCodes = new Set(wasmResult.diagnostics.map(d => d.code));

  const missingInWasm = [...tscCodes].filter(c => !wasmCodes.has(c));
  const extraInWasm = [...wasmCodes].filter(c => !tscCodes.has(c));

  // Perfect match
  const exactMatch = missingInWasm.length === 0 && extraInWasm.length === 0;

  // Same error count (might have different codes)
  const sameCount = tscResult.diagnostics.length === wasmResult.diagnostics.length;

  return {
    exactMatch,
    sameCount,
    tscCount: tscResult.diagnostics.length,
    wasmCount: wasmResult.diagnostics.length,
    missingInWasm,
    extraInWasm,
  };
}

async function main() {
  const args = process.argv.slice(2);
  const maxTests = parseInt(args.find(a => a.startsWith('--max='))?.split('=')[1] || '200', 10);
  const verbose = args.includes('--verbose') || args.includes('-v');
  const category = args.find(a => !a.startsWith('-'))?.toLowerCase();

  log('Conformance Test Runner', colors.bold);
  log('═'.repeat(60), colors.dim);

  let testDir = CONFIG.conformanceDir;
  if (category) {
    testDir = join(CONFIG.conformanceDir, category);
    log(`\nCategory: ${category}`, colors.cyan);
  }

  log(`\nCollecting test files (max ${maxTests})...`, colors.cyan);
  const testFiles = getTestFiles(testDir, maxTests);
  log(`  Found ${testFiles.length} test files`, colors.dim);

  const stats = {
    total: 0,
    skippedMultiFile: 0,
    exactMatch: 0,
    sameCount: 0,
    crashed: 0,
    missingErrors: 0,  // WASM missed errors TSC found
    extraErrors: 0,    // WASM found errors TSC didn't
    byCategory: {},
  };

  const missingCodeCounts = {};
  const extraCodeCounts = {};
  const crashedFiles = [];

  log(`\nRunning tests...`, colors.cyan);

  for (let i = 0; i < testFiles.length; i++) {
    const filePath = testFiles[i];
    const fileName = basename(filePath);
    const relPath = filePath.replace(CONFIG.conformanceDir + '/', '');
    const cat = relPath.split('/')[0];

    if (!verbose) {
      process.stdout.write(`\r  Progress: ${i + 1}/${testFiles.length}`);
    }

    try {
      const rawCode = readFileSync(filePath, 'utf-8');

      // Parse test directives
      const { options, isMultiFile, cleanCode } = parseTestDirectives(rawCode);

      // Skip multi-file tests (need special handling we don't support yet)
      if (isMultiFile) {
        stats.skippedMultiFile++;
        continue;
      }

      const [tscResult, wasmResult] = await Promise.all([
        runTsc(cleanCode, fileName, options),
        runWasm(cleanCode, fileName),
      ]);

      stats.total++;
      stats.byCategory[cat] = stats.byCategory[cat] || { total: 0, exact: 0, same: 0 };
      stats.byCategory[cat].total++;

      if (wasmResult.crashed) {
        stats.crashed++;
        crashedFiles.push({ file: relPath, error: wasmResult.error });
        continue;
      }

      const comparison = compareDiagnostics(tscResult, wasmResult);

      if (comparison.exactMatch) {
        stats.exactMatch++;
        stats.byCategory[cat].exact++;
      }

      if (comparison.sameCount) {
        stats.sameCount++;
        stats.byCategory[cat].same++;
      }

      if (comparison.missingInWasm.length > 0) {
        stats.missingErrors++;
        for (const code of comparison.missingInWasm) {
          missingCodeCounts[code] = (missingCodeCounts[code] || 0) + 1;
        }
      }

      if (comparison.extraInWasm.length > 0) {
        stats.extraErrors++;
        for (const code of comparison.extraInWasm) {
          extraCodeCounts[code] = (extraCodeCounts[code] || 0) + 1;
        }
      }

      if (verbose && !comparison.exactMatch) {
        log(`\n${relPath}:`, colors.yellow);
        log(`  TSC: ${comparison.tscCount} errors, WASM: ${comparison.wasmCount} errors`, colors.dim);
        if (comparison.missingInWasm.length > 0) {
          log(`  Missing: TS${comparison.missingInWasm.join(', TS')}`, colors.red);
        }
        if (comparison.extraInWasm.length > 0) {
          log(`  Extra: TS${comparison.extraInWasm.join(', TS')}`, colors.yellow);
        }
      }

    } catch (e) {
      // Skip files that can't be read
    }
  }

  if (!verbose) {
    console.log('');
  }

  // Print report
  log('\n' + '═'.repeat(60), colors.bold);
  log('  CONFORMANCE TEST REPORT', colors.bold);
  log('═'.repeat(60), colors.bold);

  log(`\n  Summary:`, colors.cyan);
  log(`    Files Found:      ${testFiles.length}`);
  log(`    Multi-File Skipped: ${stats.skippedMultiFile}`, colors.dim);
  log(`    Tests Run:        ${stats.total}`);
  log(`    Exact Match:      ${stats.exactMatch} (${(stats.exactMatch / stats.total * 100).toFixed(1)}%)`, colors.green);
  log(`    Same Error Count: ${stats.sameCount} (${(stats.sameCount / stats.total * 100).toFixed(1)}%)`, colors.blue);
  log(`    WASM Crashed:     ${stats.crashed}`, stats.crashed > 0 ? colors.red : '');

  log(`\n  Parity Issues:`, colors.cyan);
  log(`    Tests with missing errors: ${stats.missingErrors} (${(stats.missingErrors / stats.total * 100).toFixed(1)}%)`, stats.missingErrors > 0 ? colors.red : colors.green);
  log(`    Tests with extra errors:   ${stats.extraErrors} (${(stats.extraErrors / stats.total * 100).toFixed(1)}%)`, stats.extraErrors > 0 ? colors.yellow : colors.green);

  if (Object.keys(missingCodeCounts).length > 0) {
    log(`\n  Most Common Missing Error Codes:`, colors.cyan);
    const sorted = Object.entries(missingCodeCounts).sort((a, b) => b[1] - a[1]).slice(0, 10);
    for (const [code, count] of sorted) {
      log(`    TS${code}: ${count} occurrences`, colors.red);
    }
  }

  if (Object.keys(extraCodeCounts).length > 0) {
    log(`\n  Most Common Extra Error Codes:`, colors.cyan);
    const sorted = Object.entries(extraCodeCounts).sort((a, b) => b[1] - a[1]).slice(0, 10);
    for (const [code, count] of sorted) {
      log(`    TS${code}: ${count} occurrences`, colors.yellow);
    }
  }

  if (Object.keys(stats.byCategory).length > 1) {
    log(`\n  By Category:`, colors.cyan);
    const sorted = Object.entries(stats.byCategory).sort((a, b) => b[1].total - a[1].total);
    for (const [cat, data] of sorted.slice(0, 15)) {
      const pct = (data.exact / data.total * 100).toFixed(0);
      const bar = '█'.repeat(Math.floor(pct / 5)) + '░'.repeat(20 - Math.floor(pct / 5));
      log(`    ${cat.padEnd(20)} ${bar} ${pct}% exact (${data.exact}/${data.total})`);
    }
  }

  if (crashedFiles.length > 0 && crashedFiles.length <= 10) {
    log(`\n  Crashed Files:`, colors.red);
    for (const { file, error } of crashedFiles) {
      log(`    ${file}: ${error?.slice(0, 80) || 'Unknown error'}`, colors.dim);
    }
  }

  log('\n' + '═'.repeat(60) + '\n', colors.bold);
}

main().catch(e => {
  console.error('Fatal error:', e);
  process.exit(1);
});
