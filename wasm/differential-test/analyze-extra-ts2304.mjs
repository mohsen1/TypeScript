#!/usr/bin/env node
/**
 * Analyze TS2304 extra errors (WASM reports but TSC doesn't)
 * TS2304: Cannot find name 'X'.
 */

import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, join, resolve, basename } from 'path';
import { readFileSync, readdirSync, statSync, writeFileSync } from 'fs';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const CONFIG = {
  wasmPkgPath: resolve(__dirname, '../pkg'),
  conformanceDir: resolve(__dirname, '../../tests/cases/conformance'),
};
const DEFAULT_LIB_PATH = resolve(__dirname, '../../tests/lib/lib.d.ts');
const DEFAULT_LIB_SOURCE = readFileSync(DEFAULT_LIB_PATH, 'utf8');
const DEFAULT_LIB_NAME = 'lib.d.ts';

const ts = require('typescript');

function getTestFiles(dir, maxFiles = 5000) {
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
  return { options, isMultiFile, cleanCode: cleanLines.join('\n'), files };
}

function buildCompilerOptions(testOptions = {}) {
  const compilerOptions = {
    strict: testOptions.strict !== false,
    target: ts.ScriptTarget.ES2020,
    module: ts.ModuleKind.ESNext,
    noEmit: true,
    skipLibCheck: true,
  };
  if (testOptions.target) {
    const targetMap = {
      es5: ts.ScriptTarget.ES5, es6: ts.ScriptTarget.ES2015, es2015: ts.ScriptTarget.ES2015,
      es2016: ts.ScriptTarget.ES2016, es2017: ts.ScriptTarget.ES2017, es2018: ts.ScriptTarget.ES2018,
      es2019: ts.ScriptTarget.ES2019, es2020: ts.ScriptTarget.ES2020, es2021: ts.ScriptTarget.ES2021,
      es2022: ts.ScriptTarget.ES2022, esnext: ts.ScriptTarget.ESNext,
    };
    compilerOptions.target = targetMap[testOptions.target.toLowerCase()] || ts.ScriptTarget.ES2020;
  }
  return compilerOptions;
}

function runTscSingle(code, fileName, testOptions) {
  const compilerOptions = buildCompilerOptions(testOptions);
  const sourceFile = ts.createSourceFile(fileName, code, ts.ScriptTarget.ES2020, true);
  const host = ts.createCompilerHost(compilerOptions);
  const originalGetSourceFile = host.getSourceFile;
  host.getSourceFile = (name, languageVersion, onError) => {
    if (name === fileName) return sourceFile;
    return originalGetSourceFile.call(host, name, languageVersion, onError);
  };
  const program = ts.createProgram([fileName], compilerOptions, host);
  const allDiagnostics = [
    ...program.getSyntacticDiagnostics(sourceFile),
    ...program.getSemanticDiagnostics(sourceFile),
  ];
  return allDiagnostics.map(d => ({
    code: d.code,
    message: ts.flattenDiagnosticMessageText(d.messageText, '\n'),
    category: ts.DiagnosticCategory[d.category],
    start: d.start,
    length: d.length,
  }));
}

async function runWasmSingle(code, fileName, wasm, testOptions) {
  const parser = new wasm.ThinParser(fileName, code);
  if (!testOptions.nolib) {
    parser.addLibFile(DEFAULT_LIB_NAME, DEFAULT_LIB_SOURCE);
  }
  parser.parseSourceFile();

  const parseDiags = JSON.parse(parser.getDiagnosticsJson());
  const checkResult = JSON.parse(parser.checkSourceFile());

  const wasmDiags = [
    ...parseDiags.map(d => ({
      code: d.code, message: d.message, category: 'Error', source: 'parser',
    })),
    ...(checkResult.diagnostics || []).map(d => ({
      code: d.code, message: d.message_text, category: d.category, source: 'checker',
    })),
  ];
  parser.free();
  return wasmDiags;
}

function extractSymbolName(message) {
  const match = message.match(/Cannot find name '([^']+)'/);
  return match ? match[1] : null;
}

function categorizeSymbol(name, code) {
  const builtinTypes = ['Array', 'Object', 'String', 'Number', 'Boolean', 'Function', 'Symbol', 'BigInt',
    'Promise', 'Map', 'Set', 'WeakMap', 'WeakSet', 'RegExp', 'Date', 'Error', 'Partial', 'Required',
    'Readonly', 'Record', 'Pick', 'Omit', 'Exclude', 'Extract', 'NonNullable', 'ReturnType', 'Parameters',
    'ConstructorParameters', 'InstanceType', 'Uppercase', 'Lowercase', 'Capitalize', 'Uncapitalize',
    'Awaited', 'NoInfer', 'PropertyKey', 'ThisType', 'ThisParameterType', 'OmitThisParameter', 'Iterable', 'Iterator',
    'IterableIterator', 'ArrayLike', 'PromiseLike', 'AsyncIterator', 'AsyncIterable', 'AsyncIterableIterator'];
  const globalObjects = ['console', 'window', 'document', 'globalThis', 'self', 'global', 'process',
    'Buffer', 'JSON', 'Math', 'Reflect', 'Proxy', 'Intl', 'WebAssembly', 'Atomics', 'SharedArrayBuffer',
    'DataView', 'ArrayBuffer', 'Int8Array', 'Uint8Array', 'Int16Array', 'Uint16Array', 'Int32Array',
    'Uint32Array', 'Float32Array', 'Float64Array', 'BigInt64Array', 'BigUint64Array', 'URL', 'URLSearchParams',
    'TextEncoder', 'TextDecoder', 'AbortController', 'AbortSignal', 'fetch', 'Request', 'Response',
    'Headers', 'FormData', 'Blob', 'File', 'FileReader', 'XMLHttpRequest', 'WebSocket', 'Worker',
    'MessageChannel', 'MessagePort', 'BroadcastChannel', 'crypto', 'Crypto', 'CryptoKey', 'SubtleCrypto'];
  const globalFunctions = ['eval', 'parseInt', 'parseFloat', 'isNaN', 'isFinite', 'encodeURI',
    'encodeURIComponent', 'decodeURI', 'decodeURIComponent', 'escape', 'unescape', 'setTimeout',
    'setInterval', 'clearTimeout', 'clearInterval', 'setImmediate', 'clearImmediate', 'queueMicrotask',
    'atob', 'btoa', 'alert', 'confirm', 'prompt', 'print', 'requestAnimationFrame', 'cancelAnimationFrame',
    'require', 'module', 'exports', '__dirname', '__filename'];
  const globalConstants = ['undefined', 'NaN', 'Infinity', 'arguments', 'true', 'false', 'null'];

  if (/^[A-Z]$/.test(name) || /^T[A-Z][a-z]/.test(name)) return 'type_parameter';
  if (builtinTypes.includes(name)) return 'builtin_type';
  if (globalObjects.includes(name)) return 'global_object';
  if (globalFunctions.includes(name)) return 'global_function';
  if (globalConstants.includes(name)) return 'global_constant';
  if (/^[A-Z]/.test(name)) return 'user_defined_type';
  if (/^[a-z_$]/.test(name)) return 'local_reference';
  return 'unknown';
}

async function main() {
  const args = process.argv.slice(2);
  const maxTests = parseInt(args.find(a => a.startsWith('--max='))?.split('=')[1] || '5000', 10);

  console.log('Analyzing TS2304 Extra Errors...');
  console.log('═'.repeat(60));

  const wasm = await import(join(CONFIG.wasmPkgPath, 'wasm.js'));
  const testFiles = getTestFiles(CONFIG.conformanceDir, maxTests);
  console.log(`Found ${testFiles.length} test files`);

  const extraErrors = [];
  const categoryCount = {};
  const symbolCount = {};
  let processed = 0;

  for (const filePath of testFiles) {
    processed++;
    if (processed % 100 === 0) {
      process.stdout.write(`\rProcessed: ${processed}/${testFiles.length}`);
    }

    let rawCode;
    try {
      rawCode = readFileSync(filePath, 'utf-8');
    } catch { continue; }

    const { options, isMultiFile, cleanCode, files } = parseTestDirectives(rawCode);
    if (isMultiFile) continue;

    let tscDiags, wasmDiags;
    try {
      const fileName = basename(filePath);
      tscDiags = runTscSingle(cleanCode, fileName, options);
      wasmDiags = await runWasmSingle(cleanCode, fileName, wasm, options);
    } catch { continue; }

    const tsc2304 = tscDiags.filter(d => d.code === 2304);
    const tsc2304Messages = new Set(tsc2304.map(d => d.message));
    const wasm2304 = wasmDiags.filter(d => d.code === 2304);
    const extra2304 = wasm2304.filter(d => !tsc2304Messages.has(d.message));

    for (const err of extra2304) {
      const symbolName = extractSymbolName(err.message);
      if (!symbolName) continue;
      const category = categorizeSymbol(symbolName, cleanCode);
      categoryCount[category] = (categoryCount[category] || 0) + 1;
      symbolCount[symbolName] = (symbolCount[symbolName] || 0) + 1;
      const relPath = filePath.replace(CONFIG.conformanceDir + '/', '');
      extraErrors.push({
        file: relPath, symbol: symbolName, category, message: err.message,
        codeSnippet: cleanCode.slice(0, 300),
      });
    }
  }

  console.log('\n\n' + '═'.repeat(60));
  console.log('  TS2304 EXTRA ERROR ANALYSIS');
  console.log('═'.repeat(60));
  console.log('\n  Total Extra TS2304 Errors:', extraErrors.length);
  console.log('\n  By Category:');
  const sortedCategories = Object.entries(categoryCount).sort((a, b) => b[1] - a[1]);
  for (const [cat, count] of sortedCategories) {
    const pct = ((count / extraErrors.length) * 100).toFixed(1);
    console.log(`    ${cat.padEnd(25)} ${count} (${pct}%)`);
  }
  console.log('\n  Top 30 Symbols Not Found:');
  const sortedSymbols = Object.entries(symbolCount).sort((a, b) => b[1] - a[1]).slice(0, 30);
  for (const [sym, count] of sortedSymbols) {
    const category = categorizeSymbol(sym, '');
    console.log(`    ${sym.padEnd(25)} ${count} occurrences (${category})`);
  }

  const report = {
    summary: { totalExtraErrors: extraErrors.length, processedFiles: processed },
    byCategory: categoryCount,
    topSymbols: sortedSymbols.map(([sym, count]) => ({ symbol: sym, count, category: categorizeSymbol(sym, '') })),
    sampleErrors: extraErrors.slice(0, 50),
  };
  writeFileSync(join(__dirname, 'output/ts2304-extra-analysis.json'), JSON.stringify(report, null, 2));
  console.log('\n  Detailed report saved to output/ts2304-extra-analysis.json');
  console.log('\n' + '═'.repeat(60));
}

main().catch(err => { console.error(err); process.exit(1); });
