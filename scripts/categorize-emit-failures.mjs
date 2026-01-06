#!/usr/bin/env node
import wasm from '../wasm/pkg/wasm.js';
import { readFileSync, readdirSync, existsSync } from 'fs';
import { basename, join } from 'path';

const testDir = 'tests/cases/compiler';
const baselineDir = 'tests/baselines/reference';

function extractJsFromBaseline(baselineContent) {
    const lines = baselineContent.split('\n');
    const jsMarkerRegex = /^\/\/\/\/\s*\[([^\]]+\.js)\]\s*$/;
    let inJsSection = false;
    let jsLines = [];

    for (const line of lines) {
        const markerMatch = line.match(jsMarkerRegex);
        if (markerMatch) {
            inJsSection = true;
            jsLines = [];
            continue;
        }
        if (inJsSection && line.startsWith('//// [')) {
            break;
        }
        if (inJsSection) {
            jsLines.push(line);
        }
    }
    return jsLines.join('\n');
}

// Categorize failures
const categories = {
    commonjs: [],
    ambient: [],
    blankLines: [],
    emptyBlock: [],
    parameterProps: [],
    classExtends: [],
    other: []
};

const files = readdirSync(testDir).filter(f => f.endsWith('.ts')).slice(0, 100);

for (const file of files) {
    const testName = basename(file, '.ts');
    const testFile = join(testDir, file);
    const jsBaseline = join(baselineDir, testName + '.js');

    if (!existsSync(jsBaseline)) continue;

    try {
        const source = readFileSync(testFile, 'utf-8');
        if (source.length > 50000) continue;

        const baselineContent = readFileSync(jsBaseline, 'utf-8');
        const expected = extractJsFromBaseline(baselineContent);

        const p = wasm.createThinParser(testName + '.ts', source);
        p.parseSourceFile();
        p.bindSourceFile();
        const actual = p.emit();
        p.free();

        const normalizeJs = (s) => s.replace(/\r\n/g, '\n').trim();
        const expectedNorm = normalizeJs(expected);
        const actualNorm = normalizeJs(actual);

        if (expectedNorm !== actualNorm) {
            // Categorize
            if (expectedNorm.includes('"use strict"') && !actualNorm.includes('"use strict"')) {
                categories.commonjs.push(testName);
            } else if (actualNorm.includes('var console') || actualNorm.includes('declare ')) {
                categories.ambient.push(testName);
            } else if (expectedNorm.includes('{\n}') && actualNorm.includes('{ }')) {
                categories.emptyBlock.push(testName);
            } else if (expectedNorm.replace(/\n+/g, '\n') === actualNorm.replace(/\n+/g, '\n')) {
                categories.blankLines.push(testName);
            } else if (expectedNorm.includes('this.p') && source.includes('public') && source.includes('constructor')) {
                categories.parameterProps.push(testName);
            } else if (expectedNorm.includes('__extends') || source.includes('extends')) {
                categories.classExtends.push(testName);
            } else {
                categories.other.push(testName);
            }
        }
    } catch (e) {
        categories.other.push(testName + ' (error)');
    }
}

console.log('=== Failure Categories ===');
console.log('CommonJS exports:', categories.commonjs.length, categories.commonjs.slice(0, 5).join(', '));
console.log('Ambient declarations:', categories.ambient.length, categories.ambient.slice(0, 5).join(', '));
console.log('Empty block format:', categories.emptyBlock.length, categories.emptyBlock.slice(0, 5).join(', '));
console.log('Blank lines:', categories.blankLines.length, categories.blankLines.slice(0, 5).join(', '));
console.log('Parameter properties:', categories.parameterProps.length, categories.parameterProps.slice(0, 5).join(', '));
console.log('Class extends:', categories.classExtends.length, categories.classExtends.slice(0, 5).join(', '));
console.log('Other:', categories.other.length, categories.other.slice(0, 10).join(', '));
