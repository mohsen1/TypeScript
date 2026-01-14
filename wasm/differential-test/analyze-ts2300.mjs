#!/usr/bin/env node
/**
 * TS2300 Deep Dive Analysis Tool
 * Analyzes duplicate identifier errors in detail
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
  DEFAULT_LIB_PATH: resolve(__dirname, '../../tests/lib/lib.d.ts'),
  DEFAULT_LIB_SOURCE: readFileSync(resolve(__dirname, '../../tests/lib/lib.d.ts'), 'utf-8'),
  DEFAULT_LIB_NAME: 'lib.d.ts',
};

const ts = require('typescript');

/**
 * Recursively get all .ts files
 */
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
 * Parse test directives from source code
 */
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
    isMultiFile,
    cleanCode: cleanLines.join('\n'),
    files,
  };
}

/**
 * Run tsc on a single file
 */
async function runTsc(code, fileName = 'test.ts', testOptions = {}) {
  const compilerOptions = {
    strict: testOptions.strict !== false,
    target: ts.ScriptTarget.ES2020,
    module: ts.ModuleKind.ESNext,
    noEmit: true,
    skipLibCheck: true,
  };

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

  const sourceFile = ts.createSourceFile(fileName, code, ts.ScriptTarget.ES2020, true);

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
      line: d.file ? ts.getLineAndCharacterOfPosition(d.file, d.start).line + 1 : 0,
    })),
  };
}

/**
 * Run WASM on a single file
 */
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

/**
 * Analyze a single test file for TS2300
 */
async function analyzeFile(filePath) {
  const rawCode = readFileSync(filePath, 'utf-8');
  const { options, isMultiFile, cleanCode } = parseTestDirectives(rawCode);

  if (isMultiFile) return null; // Skip multi-file for now

  const fileName = basename(filePath);

  const [tscResult, wasmResult] = await Promise.all([
    runTsc(cleanCode, fileName, options),
    runWasm(cleanCode, fileName, options),
  ]);

  const tscTS2300 = tscResult.diagnostics.filter(d => d.code === 2300);
  const wasmTS2300 = wasmResult.diagnostics.filter(d => d.code === 2300);

  return {
    filePath: filePath.replace(CONFIG.conformanceDir + '/', ''),
    fileName,
    tscTS2300Count: tscTS2300.length,
    wasmTS2300Count: wasmTS2300.length,
    tscTS2300,
    wasmTS2300,
    hasDifference: tscTS2300.length !== wasmTS2300.length,
    allTscErrors: tscResult.diagnostics,
    allWasmErrors: wasmResult.diagnostics,
  };
}

/**
 * Main analysis function
 */
async function main() {
  console.log('🔍 TS2300 Deep Dive Analysis\n');

  // Find all test files
  console.log('📂 Scanning for test files...');
  const testFiles = getTestFiles(CONFIG.conformanceDir, 500);
  console.log(`   Found ${testFiles.length} test files\n`);

  // Analyze all files
  console.log('🔬 Analyzing files for TS2300...');
  const results = [];
  let withTS2300 = 0;
  let withDifference = 0;

  for (let i = 0; i < testFiles.length; i++) {
    const result = await analyzeFile(testFiles[i]);
    if (!result) continue;

    if (result.tscTS2300Count > 0 || result.wasmTS2300Count > 0) {
      withTS2300++;
      results.push(result);

      if (result.hasDifference) {
        withDifference++;
      }
    }
  }

  console.log(`   Found ${withTS2300} files with TS2300 errors\n`);
  console.log(`   Found ${withDifference} files with discrepancies\n`);

  // Summary
  console.log('════════════════════════════════════════════════════════════════');
  console.log('  TS2300 ANALYSIS SUMMARY');
  console.log('════════════════════════════════════════════════════════════════\n');

  console.log(`Files with TS2300 (TSC):    ${results.filter(r => r.tscTS2300Count > 0).length}`);
  console.log(`Files with TS2300 (WASM):   ${results.filter(r => r.wasmTS2300Count > 0).length}`);
  console.log(`Files with differences:     ${withDifference}\n`);

  // Show files with differences
  const differences = results.filter(r => r.hasDifference);
  if (differences.length > 0) {
    console.log('════════════════════════════════════════════════════════════════');
    console.log('  FILES WITH DIFFERENCES');
    console.log('════════════════════════════════════════════════════════════════\n');

    for (const result of differences.slice(0, 20)) {
      console.log(`📄 ${result.filePath}`);
      console.log(`   TSC:  ${result.tscTS2300Count} TS2300 errors`);
      console.log(`   WASM: ${result.wasmTS2300Count} TS2300 errors`);
      console.log(`   Difference: ${result.tscTS2300Count - result.wasmTS2300Count} missing by WASM\n`);

      // Show the actual errors
      if (result.tscTS2300.length > 0) {
        console.log(`   TSC TS2300 Errors:`);
        for (const err of result.tscTS2300.slice(0, 5)) {
          console.log(`     Line ${err.line}: ${err.message}`);
        }
      }

      if (result.wasmTS2300.length > 0) {
        console.log(`   WASM TS2300 Errors:`);
        for (const err of result.wasmTS2300.slice(0, 5)) {
          console.log(`     ${err.message}`);
        }
      }

      console.log('');
    }
  }

  // Show files where WASM is missing TS2300
  const missing = results.filter(r => r.tscTS2300Count > r.wasmTS2300Count);
  if (missing.length > 0) {
    console.log('════════════════════════════════════════════════════════════════');
    console.log('  WASM MISSING TS2300 ERRORS');
    console.log('════════════════════════════════════════════════════════════════\n');

    console.log(`Count: ${missing.length} files\n`);

    for (const result of missing.slice(0, 30)) {
      const missingCount = result.tscTS2300Count - result.wasmTS2300Count;
      console.log(`📄 ${result.filePath} (-${missingCount} missing)`);
    }
  }

  // Generate detailed report
  const report = {
    timestamp: new Date().toISOString(),
    summary: {
      totalTestFiles: testFiles.length,
      filesWithTS2300: withTS2300,
      filesWithDifferences: withDifference,
      tscTS2300Total: results.reduce((sum, r) => sum + r.tscTS2300Count, 0),
      wasmTS2300Total: results.reduce((sum, r) => sum + r.wasmTS2300Count, 0),
      missingTS2300: results.reduce((sum, r) => sum + Math.max(0, r.tscTS2300Count - r.wasmTS2300Count), 0),
    },
    filesWithDifferences: differences.map(r => ({
      path: r.filePath,
      tscCount: r.tscTS2300Count,
      wasmCount: r.wasmTS2300Count,
      missingCount: r.tscTS2300Count - r.wasmTS2300Count,
      tscErrors: r.tscTS2300,
      wasmErrors: r.wasmTS2300,
    })),
    filesMissingTS2300: missing.map(r => ({
      path: r.filePath,
      tscCount: r.tscTS2300Count,
      wasmCount: r.wasmTS2300Count,
      missingCount: r.tscTS2300Count - r.wasmTS2300Count,
    })),
  };

  // Save report
  const reportPath = join(__dirname, '../metrics-data/ts2300-analysis.json');
  const fs = require('fs');
  if (!require('fs').existsSync(join(__dirname, '../metrics-data'))) {
    require('fs').mkdirSync(join(__dirname, '../metrics-data'), { recursive: true });
  }
  fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

  console.log(`\n✅ Report saved to: wasm/metrics-data/ts2300-analysis.json`);
}

main().catch(e => {
  console.error('Error:', e);
  process.exit(1);
});
