#!/usr/bin/env node
/**
 * Find TS2339 false positives and missing errors in conformance tests.
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
  }));
}

function runTscMulti(files, testOptions) {
  const compilerOptions = buildCompilerOptions(testOptions);
  const sourceFiles = new Map();
  const fileNames = [];

  for (const file of files) {
    const sf = ts.createSourceFile(file.name, file.content, ts.ScriptTarget.ES2020, true);
    sourceFiles.set(file.name, sf);
    fileNames.push(file.name);
  }

  const host = ts.createCompilerHost(compilerOptions);
  const originalGetSourceFile = host.getSourceFile;
  host.getSourceFile = (name, languageVersion, onError) => {
    if (sourceFiles.has(name)) {
      return sourceFiles.get(name);
    }
    return originalGetSourceFile.call(host, name, languageVersion, onError);
  };
  host.fileExists = name => sourceFiles.has(name) || ts.sys.fileExists(name);
  host.readFile = name => {
    const file = files.find(f => f.name === name);
    if (file) return file.content;
    return ts.sys.readFile(name);
  };

  const program = ts.createProgram(fileNames, compilerOptions, host);
  const allDiagnostics = [];
  for (const sf of sourceFiles.values()) {
    allDiagnostics.push(...program.getSyntacticDiagnostics(sf));
    allDiagnostics.push(...program.getSemanticDiagnostics(sf));
  }

  return allDiagnostics.map(d => ({
    code: d.code,
    message: ts.flattenDiagnosticMessageText(d.messageText, '\n'),
    category: ts.DiagnosticCategory[d.category],
  }));
}

function runWasmSingle(code, fileName, wasm) {
  const parser = new wasm.ThinParser(fileName, code);
  parser.parseSourceFile();
  const parseDiags = JSON.parse(parser.getDiagnosticsJson());
  const checkResult = JSON.parse(parser.checkSourceFile());
  let wasmDiags = [
    ...parseDiags.map(d => ({
      code: d.code,
      message: d.message,
      category: 'Error',
    })),
    ...(checkResult.diagnostics || []).map(d => ({
      code: d.code,
      message: d.message_text,
      category: d.category,
    })),
  ];
  return wasmDiags;
}

function runWasmMulti(files, wasm) {
  const program = new wasm.WasmProgram();
  for (const file of files) {
    program.addFile(file.name, file.content);
  }
  const codes = program.getAllDiagnosticCodes();
  return Array.from(codes).map(code => ({ code, message: '', category: 'Error' }));
}

async function main() {
  const args = process.argv.slice(2);
  const maxTests = parseInt(args.find(a => a.startsWith('--max='))?.split('=')[1] || '5000', 10);
  const maxSamples = parseInt(args.find(a => a.startsWith('--samples='))?.split('=')[1] || '50', 10);
  const showMissing = args.includes('--missing');
  const showExtra = args.includes('--extra');
  const showBoth = !showMissing && !showExtra; // Default: show both

  const wasm = await import(join(CONFIG.wasmPkgPath, 'wasm.js'));
  const testFiles = getTestFiles(CONFIG.conformanceDir, maxTests);

  const extraMatches = [];
  const missingMatches = [];
  let processed = 0;

  for (const filePath of testFiles) {
    processed++;
    let rawCode;
    try {
      rawCode = readFileSync(filePath, 'utf-8');
    } catch {
      continue;
    }

    const { options, isMultiFile, cleanCode, files } = parseTestDirectives(rawCode);
    let tscDiags = [];
    let wasmDiags = [];

    try {
      if (isMultiFile && files.length > 0) {
        tscDiags = runTscMulti(files, options);
        wasmDiags = runWasmMulti(files, wasm);
      } else {
        const fileName = basename(filePath);
        tscDiags = runTscSingle(cleanCode, fileName, options);
        wasmDiags = runWasmSingle(cleanCode, fileName, wasm);
      }
    } catch {
      continue;
    }

    const tscCodes = new Set(tscDiags.map(d => d.code));
    const wasmCodes = new Set(wasmDiags.map(d => d.code));
    const relPath = filePath.replace(CONFIG.conformanceDir + '/', '');

    // Extra: WASM reports TS2339 but tsc doesn't
    if ((showExtra || showBoth) && wasmCodes.has(2339) && !tscCodes.has(2339)) {
      const messages = wasmDiags.filter(d => d.code === 2339).map(d => d.message).filter(Boolean);
      if (extraMatches.length < maxSamples) {
        extraMatches.push({ path: relPath, messages, code: rawCode.slice(0, 600) });
      }
    }

    // Missing: tsc reports TS2339 but WASM doesn't
    if ((showMissing || showBoth) && tscCodes.has(2339) && !wasmCodes.has(2339)) {
      const messages = tscDiags.filter(d => d.code === 2339).map(d => d.message).filter(Boolean);
      if (missingMatches.length < maxSamples) {
        missingMatches.push({ path: relPath, messages, code: rawCode.slice(0, 600) });
      }
    }

    // Early exit if we have enough samples
    if (extraMatches.length >= maxSamples && missingMatches.length >= maxSamples) {
      break;
    }
  }

  // Print results
  if (showExtra || showBoth) {
    console.log('\n========================================');
    console.log('EXTRA TS2339 (WASM reports, tsc does not)');
    console.log('========================================');
    for (const match of extraMatches) {
      console.log(`\n--- ${match.path} ---`);
      if (match.messages.length > 0) {
        for (const msg of match.messages.slice(0, 3)) {
          console.log(`  ${msg}`);
        }
      }
    }
    console.log(`\nTotal extra: ${extraMatches.length}`);
  }

  if (showMissing || showBoth) {
    console.log('\n========================================');
    console.log('MISSING TS2339 (tsc reports, WASM does not)');
    console.log('========================================');
    for (const match of missingMatches) {
      console.log(`\n--- ${match.path} ---`);
      if (match.messages.length > 0) {
        for (const msg of match.messages.slice(0, 3)) {
          console.log(`  ${msg}`);
        }
      }
    }
    console.log(`\nTotal missing: ${missingMatches.length}`);
  }

  console.log('\n========================================');
  console.log('SUMMARY');
  console.log('========================================');
  console.log(`Files processed: ${processed}`);
  console.log(`Extra TS2339 files: ${extraMatches.length}`);
  console.log(`Missing TS2339 files: ${missingMatches.length}`);

  // List file paths for easy reference
  if (extraMatches.length > 0) {
    console.log('\nExtra files:');
    for (const m of extraMatches.slice(0, 20)) {
      console.log(`  ${m.path}`);
    }
    if (extraMatches.length > 20) {
      console.log(`  ... and ${extraMatches.length - 20} more`);
    }
  }

  if (missingMatches.length > 0) {
    console.log('\nMissing files:');
    for (const m of missingMatches.slice(0, 20)) {
      console.log(`  ${m.path}`);
    }
    if (missingMatches.length > 20) {
      console.log(`  ... and ${missingMatches.length - 20} more`);
    }
  }

  process.exit(0);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
