#!/usr/bin/env node
/**
 * Find TS2339 extra or missing diagnostics in conformance tests.
 * Compares WASM diagnostics to tsc, supports single-file tests.
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
    isMultiFile,
    cleanCode: cleanLines.join('\n'),
    files,
  };
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
      es5: ts.ScriptTarget.ES5,
      es6: ts.ScriptTarget.ES2015,
      es2015: ts.ScriptTarget.ES2015,
      es2016: ts.ScriptTarget.ES2016,
      es2017: ts.ScriptTarget.ES2017,
      es2018: ts.ScriptTarget.ES2018,
      es2019: ts.ScriptTarget.ES2019,
      es2020: ts.ScriptTarget.ES2020,
      es2021: ts.ScriptTarget.ES2021,
      es2022: ts.ScriptTarget.ES2022,
      esnext: ts.ScriptTarget.ESNext,
    };
    compilerOptions.target = targetMap[testOptions.target.toLowerCase()] || ts.ScriptTarget.ES2020;
  }

  if (testOptions.noimplicitany !== undefined) {
    compilerOptions.noImplicitAny = testOptions.noimplicitany;
  }
  if (testOptions.strictnullchecks !== undefined) {
    compilerOptions.strictNullChecks = testOptions.strictnullchecks;
  }

  return compilerOptions;
}

function runTscSingle(code, fileName, testOptions) {
  const compilerOptions = buildCompilerOptions(testOptions);
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

  return allDiagnostics.map(d => ({
    code: d.code,
    message: ts.flattenDiagnosticMessageText(d.messageText, '\n'),
    category: ts.DiagnosticCategory[d.category],
    line: d.file ? ts.getLineAndCharacterOfPosition(d.file, d.start).line + 1 : 0,
  }));
}

let libPromiseCache = null;

async function loadLibFiles(wasm, target) {
  // Not used in synchronous context, but kept for compatibility
  return loadLibFilesSync(wasm, '');
}

function runWasmSingle(code, fileName, wasm, testOptions) {
  const parser = new wasm.ThinParser(fileName, code);
  parser.parseSourceFile();

  // Load lib files based on @lib directive
  if (testOptions && testOptions.lib) {
    const libFiles = loadLibFilesSync(wasm, testOptions.lib);
    for (const lib of libFiles) {
      parser.addLibFile(lib.name, lib.content);
    }
  }

  const parseDiags = JSON.parse(parser.getDiagnosticsJson());
  const checkResult = JSON.parse(parser.checkSourceFile());
  let wasmDiags = [
    ...parseDiags.map(d => ({
      code: d.code,
      message: d.message,
      category: 'Error',
      line: d.line || 0,
    })),
    ...(checkResult.diagnostics || []).map(d => ({
      code: d.code,
      message: d.message_text,
      category: d.category,
      line: d.line || 0,
    })),
  ];
  return wasmDiags;
}

// Synchronous wrapper for library loading (called during test iteration)
function loadLibFilesSync(wasm, libDirective) {
  // Create cache key based on lib directive
  const cacheKey = `lib_${libDirective}`;
  if (libPromiseCache && libPromiseCache.key === cacheKey) {
    return libPromiseCache.files;
  }

  const { readFileSync } = require('fs');
  const { join } = require('path');
  const typescriptLibPath = join(__dirname, '../../node_modules/typescript/lib');

  const libFiles = [];
  const libs = libDirective.split(',').map(l => l.trim());

  // Always load lib.es5.d.ts (core types)
  libFiles.push({
    name: 'lib.es5.d.ts',
    content: readFileSync(join(typescriptLibPath, 'lib.es5.d.ts'), 'utf-8')
  });

  // Load requested lib files
  for (const lib of libs) {
    const fileName = `lib.${lib}.d.ts`;
    try {
      const content = readFileSync(join(typescriptLibPath, fileName), 'utf-8');
      libFiles.push({ name: fileName, content });
    } catch (e) {
      // Lib file not found, skip
    }
  }

  libPromiseCache = { key: cacheKey, files: libFiles };
  return libFiles;
}

async function main() {
  const args = process.argv.slice(2);
  const maxTests = parseInt(args.find(a => a.startsWith('--max='))?.split('=')[1] || '2000', 10);
  const maxSamples = parseInt(args.find(a => a.startsWith('--samples='))?.split('=')[1] || '20', 10);
  const modeArg = args.find(a => a.startsWith('--mode='))?.split('=')[1];
  const mode = modeArg
    || (args.includes('--missing') ? 'missing' : null)
    || (args.includes('--extra') ? 'extra' : null)
    || 'extra';
  const reportExtra = mode === 'extra' || mode === 'both';
  const reportMissing = mode === 'missing' || mode === 'both';

  const wasm = await import(join(CONFIG.wasmPkgPath, 'wasm.js'));
  const testFiles = getTestFiles(CONFIG.conformanceDir, maxTests);

  const extraMatches = [];
  const missingMatches = [];

  for (const filePath of testFiles) {
    let rawCode;
    try {
      rawCode = readFileSync(filePath, 'utf-8');
    } catch {
      continue;
    }

    const { options, isMultiFile, cleanCode, files } = parseTestDirectives(rawCode);

    // Skip multi-file tests for simplicity
    if (isMultiFile) continue;

    let tscDiags = [];
    let wasmDiags = [];

    try {
      const fileName = basename(filePath);
      tscDiags = runTscSingle(cleanCode, fileName, options);
      wasmDiags = runWasmSingle(cleanCode, fileName, wasm, options);
    } catch (e) {
      continue;
    }

    const tscCodes = new Set(tscDiags.map(d => d.code));
    const wasmCodes = new Set(wasmDiags.map(d => d.code));
    const extraCodes = [...wasmCodes].filter(code => !tscCodes.has(code));
    const missingCodes = [...tscCodes].filter(code => !wasmCodes.has(code));

    if (reportExtra && extraCodes.includes(2339)) {
      const relPath = filePath.replace(CONFIG.conformanceDir + '/', '');
      const wasmMessages = wasmDiags.filter(d => d.code === 2339);
      console.log(`\n=== EXTRA TS2339: ${relPath} ===`);
      console.log('WASM TS2339 errors:');
      for (const m of wasmMessages.slice(0, 5)) {
        console.log(`  Line ${m.line}: ${m.message}`);
      }
      console.log('\nCode snippet:');
      const lines = cleanCode.split('\n');
      console.log(lines.slice(0, 30).map((l, i) => `${i + 1}: ${l}`).join('\n'));
      extraMatches.push({ path: relPath, messages: wasmMessages });
    }

    if (reportMissing && missingCodes.includes(2339)) {
      const relPath = filePath.replace(CONFIG.conformanceDir + '/', '');
      const tscMessages = tscDiags.filter(d => d.code === 2339);
      console.log(`\n=== MISSING TS2339: ${relPath} ===`);
      console.log('TSC TS2339 errors:');
      for (const m of tscMessages.slice(0, 5)) {
        console.log(`  Line ${m.line}: ${m.message}`);
      }
      console.log('\nCode snippet:');
      const lines = cleanCode.split('\n');
      console.log(lines.slice(0, 30).map((l, i) => `${i + 1}: ${l}`).join('\n'));
      missingMatches.push({ path: relPath, messages: tscMessages });
    }

    const totalMatches = extraMatches.length + missingMatches.length;
    if (totalMatches >= maxSamples) {
      break;
    }
  }

  console.log('\n\n=== SUMMARY ===');
  if (reportExtra) {
    console.log(`Total files with extra TS2339: ${extraMatches.length}`);
    for (const m of extraMatches) {
      console.log(`  ${m.path}: ${m.messages.length} extra TS2339 errors`);
    }
  }
  if (reportMissing) {
    console.log(`Total files with missing TS2339: ${missingMatches.length}`);
    for (const m of missingMatches) {
      console.log(`  ${m.path}: ${m.messages.length} missing TS2339 errors`);
    }
  }
  process.exit(0);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
