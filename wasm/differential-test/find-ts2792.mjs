#!/usr/bin/env node
/**
 * Find TS2792 (Cannot find module) mismatches in conformance tests.
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
const VIRTUAL_ROOT = '/virtual';

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

    const filenameMatch = trimmed.match(/^\/\/\s*@filename:\s*(.+)$/i);
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
  if (testOptions.module) {
    const moduleMap = {
      amd: ts.ModuleKind.AMD,
      commonjs: ts.ModuleKind.CommonJS,
      es6: ts.ModuleKind.ES2015,
      es2015: ts.ModuleKind.ES2015,
      es2020: ts.ModuleKind.ES2020,
      es2022: ts.ModuleKind.ES2022,
      esnext: ts.ModuleKind.ESNext,
      none: ts.ModuleKind.None,
      system: ts.ModuleKind.System,
      umd: ts.ModuleKind.UMD,
      nodenext: ts.ModuleKind.NodeNext,
      node16: ts.ModuleKind.Node16,
    };
    const key = String(testOptions.module).toLowerCase();
    compilerOptions.module = moduleMap[key] ?? ts.ModuleKind.ESNext;
  }
  if (testOptions.moduleresolution) {
    const resolutionMap = {
      classic: ts.ModuleResolutionKind.Classic,
      node: ts.ModuleResolutionKind.NodeJs,
      node10: ts.ModuleResolutionKind.Node10,
      node16: ts.ModuleResolutionKind.Node16,
      nodenext: ts.ModuleResolutionKind.NodeNext,
      bundler: ts.ModuleResolutionKind.Bundler,
    };
    const key = String(testOptions.moduleresolution).toLowerCase();
    compilerOptions.moduleResolution = resolutionMap[key];
  }

  return compilerOptions;
}

function runTscSingle(code, fileName, testOptions) {
  const compilerOptions = buildCompilerOptions(testOptions);
  const virtualName = join(VIRTUAL_ROOT, fileName);
  const sourceFile = ts.createSourceFile(virtualName, code, ts.ScriptTarget.ES2020, true);

  const host = ts.createCompilerHost(compilerOptions);
  const originalGetSourceFile = host.getSourceFile;
  host.getSourceFile = (name, languageVersion, onError) => {
    if (name === virtualName) {
      return sourceFile;
    }
    return originalGetSourceFile.call(host, name, languageVersion, onError);
  };
  host.getCurrentDirectory = () => VIRTUAL_ROOT;
  host.directoryExists = dir => dir === VIRTUAL_ROOT;
  host.realpath = path => path;

  const program = ts.createProgram([virtualName], compilerOptions, host);
  const allDiagnostics = [
    ...program.getSyntacticDiagnostics(sourceFile),
    ...program.getSemanticDiagnostics(sourceFile),
  ];

  return allDiagnostics.map(d => ({
    code: d.code,
    message: ts.flattenDiagnosticMessageText(d.messageText, '\n'),
  }));
}

function runTscMulti(files, testOptions) {
  const compilerOptions = buildCompilerOptions(testOptions);
  const sourceFiles = new Map();
  const fileNames = [];
  const virtualFiles = files.map(file => ({
    name: join(VIRTUAL_ROOT, file.name),
    content: file.content,
  }));

  for (const file of virtualFiles) {
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
  host.getCurrentDirectory = () => VIRTUAL_ROOT;
  host.fileExists = name => sourceFiles.has(name) || ts.sys.fileExists(name);
  host.directoryExists = dir => {
    if (dir === VIRTUAL_ROOT) return true;
    const prefix = dir.endsWith('/') ? dir : `${dir}/`;
    for (const name of sourceFiles.keys()) {
      if (name.startsWith(prefix)) return true;
    }
    return false;
  };
  host.realpath = path => path;
  host.readFile = name => {
    const file = virtualFiles.find(f => f.name === name);
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
  }));
}

function runWasmSingle(code, fileName, wasm) {
  const virtualName = join(VIRTUAL_ROOT, fileName);
  const parser = new wasm.ThinParser(virtualName, code);
  parser.parseSourceFile();
  const parseDiags = JSON.parse(parser.getDiagnosticsJson());
  const checkResult = JSON.parse(parser.checkSourceFile());
  return [
    ...parseDiags.map(d => ({ code: d.code, message: d.message })),
    ...(checkResult.diagnostics || []).map(d => ({ code: d.code, message: d.message_text })),
  ];
}

function runWasmMulti(files, wasm) {
  const program = new wasm.WasmProgram();
  for (const file of files) {
    const virtualName = join(VIRTUAL_ROOT, file.name);
    program.addFile(virtualName, file.content);
  }
  const codes = program.getAllDiagnosticCodes();
  return Array.from(codes).map(code => ({ code, message: '' }));
}

function extractModuleSpecifier(message) {
  if (!message) return null;
  const match = message.match(/'([^']+)'/);
  return match ? match[1] : null;
}

function classifySpecifier(spec) {
  if (!spec) return 'unknown';
  if (spec.startsWith('#')) return '#imports';
  if (spec.startsWith('node:')) return 'node:';
  if (spec.startsWith('@')) return '@scoped';
  if (spec.startsWith('.') || spec.startsWith('/')) return 'relative';
  if (spec.includes('/')) return 'package-subpath';
  return 'package';
}

async function main() {
  const args = process.argv.slice(2);
  const maxTests = parseInt(args.find(a => a.startsWith('--max='))?.split('=')[1] || '3000', 10);
  const maxSamples = parseInt(args.find(a => a.startsWith('--samples='))?.split('=')[1] || '20', 10);

  const wasm = await import(join(CONFIG.wasmPkgPath, 'wasm.js'));
  const testFiles = getTestFiles(CONFIG.conformanceDir, maxTests);

  let extraCount = 0;
  let missingCount = 0;
  let mismatchedCount = 0;
  const missingKinds = new Map();
  const missingSamples = [];
  const extraSamples = [];
  const mismatchedSamples = [];

  for (const filePath of testFiles) {
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
    const tscHas2792 = tscCodes.has(2792);
    const wasmHas2792 = wasmCodes.has(2792);
    const wasmHas2307 = wasmCodes.has(2307);

    if (tscHas2792 && !wasmHas2792) {
      missingCount++;
      const spec = extractModuleSpecifier(tscDiags.find(d => d.code === 2792)?.message);
      const kind = classifySpecifier(spec);
      missingKinds.set(kind, (missingKinds.get(kind) || 0) + 1);
      if (missingSamples.length < maxSamples) {
        missingSamples.push({ path: relPath, spec, kind });
      }
      if (wasmHas2307) {
        mismatchedCount++;
        if (mismatchedSamples.length < maxSamples) {
          mismatchedSamples.push({ path: relPath, spec, kind });
        }
      }
    }

    if (!tscHas2792 && wasmHas2792) {
      extraCount++;
      if (extraSamples.length < maxSamples) {
        extraSamples.push(relPath);
      }
    }
  }

  console.log('=== TS2792 (Cannot find module) ===');
  console.log(`Missing (tsc has 2792, wasm does not): ${missingCount}`);
  console.log(`Extra (wasm has 2792, tsc does not): ${extraCount}`);
  console.log(`Mismatched (tsc 2792, wasm 2307): ${mismatchedCount}`);

  console.log('\nTop missing kinds:');
  const sortedKinds = [...missingKinds.entries()].sort((a, b) => b[1] - a[1]);
  for (const [kind, count] of sortedKinds.slice(0, 10)) {
    console.log(`  ${kind}: ${count}`);
  }

  console.log('\nSample missing files:');
  for (const entry of missingSamples.slice(0, 10)) {
    console.log(`  ${entry.path} (${entry.kind}${entry.spec ? `: ${entry.spec}` : ''})`);
  }

  console.log('\nSample extra files:');
  for (const entry of extraSamples.slice(0, 10)) {
    console.log(`  ${entry}`);
  }

  console.log('\nSample mismatched files (tsc 2792, wasm 2307):');
  for (const entry of mismatchedSamples.slice(0, 10)) {
    console.log(`  ${entry.path} (${entry.kind}${entry.spec ? `: ${entry.spec}` : ''})`);
  }
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
