#!/usr/bin/env node
/**
 * Scanner Verification Script
 * 
 * This script verifies that the Rust scanner produces the exact same token stream
 * as the TypeScript scanner. It reads a source file and compares token-by-token.
 * 
 * Usage: node scripts/verifyScanner.mjs [sourceFile]
 * Default: src/compiler/core.ts
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.resolve(__dirname, "..");

// Load the built TypeScript module
const tsPath = path.join(projectRoot, "built/local/typescript.js");
if (!fs.existsSync(tsPath)) {
    console.error("Error: Built TypeScript not found. Run 'hereby local' first.");
    process.exit(1);
}

const ts = await import(tsPath);

// Get the source file to verify
const sourceFile = process.argv[2] || "src/compiler/core.ts";
const sourcePath = path.isAbsolute(sourceFile) ? sourceFile : path.join(projectRoot, sourceFile);

if (!fs.existsSync(sourcePath)) {
    console.error(`Error: Source file not found: ${sourcePath}`);
    process.exit(1);
}

const sourceText = fs.readFileSync(sourcePath, "utf-8");
console.log(`\n🔍 Verifying scanner on: ${sourceFile}`);
console.log(`   File size: ${sourceText.length} characters`);

// Create both scanners
const tsScanner = ts.createScanner(ts.ScriptTarget.Latest, /* skipTrivia */ true);
tsScanner.setText(sourceText);

const wasmScanner = ts.wasmCreateScanner(sourceText, /* skipTrivia */ true);
if (!wasmScanner) {
    console.error("Error: WASM scanner not available. Check wasm build.");
    process.exit(1);
}

// Token name lookup helper (SyntaxKind is a const enum, so we need manual mapping)
const syntaxKindNames = {
    0: "Unknown",
    1: "EndOfFileToken",
    2: "SingleLineCommentTrivia",
    3: "MultiLineCommentTrivia",
    4: "NewLineTrivia",
    5: "WhitespaceTrivia",
    9: "NumericLiteral",
    10: "BigIntLiteral",
    11: "StringLiteral",
    15: "NoSubstitutionTemplateLiteral",
    16: "TemplateHead",
    17: "TemplateMiddle",
    18: "TemplateTail",
    19: "OpenBraceToken",
    20: "CloseBraceToken",
    21: "OpenParenToken",
    22: "CloseParenToken",
    23: "OpenBracketToken",
    24: "CloseBracketToken",
    25: "DotToken",
    26: "DotDotDotToken",
    27: "SemicolonToken",
    28: "CommaToken",
    29: "QuestionDotToken",
    30: "LessThanToken",
    32: "GreaterThanToken",
    39: "EqualsGreaterThanToken",
    40: "PlusToken",
    41: "MinusToken",
    42: "AsteriskToken",
    44: "SlashToken",
    54: "ExclamationToken",
    58: "QuestionToken",
    59: "ColonToken",
    64: "EqualsToken",
    80: "Identifier",
    81: "PrivateIdentifier",
    87: "ConstKeyword",
    100: "FunctionKeyword",
    102: "ImportKeyword",
    107: "ReturnKeyword",
    121: "LetKeyword",
    134: "AsyncKeyword",
    156: "TypeKeyword",
};

function getTokenName(kind) {
    return syntaxKindNames[kind] || `Token(${kind})`;
}

// Verify token by token
let tokenCount = 0;
let mismatches = 0;
const maxMismatches = 10; // Stop after this many mismatches

console.log(`\n📊 Scanning and comparing tokens...\n`);

while (true) {
    const tsToken = tsScanner.scan();
    const wasmToken = wasmScanner.scan();
    
    tokenCount++;
    
    // Compare tokens
    const tsPos = tsScanner.getTokenStart();
    const wasmPos = wasmScanner.getTokenStart();
    const tsEnd = tsScanner.getTokenEnd();
    const wasmEnd = wasmScanner.getTokenEnd();
    const tsText = tsScanner.getTokenText();
    const wasmText = wasmScanner.getTokenText();
    const tsValue = tsScanner.getTokenValue();
    const wasmValue = wasmScanner.getTokenValue();
    
    let mismatch = false;
    let details = [];
    
    if (tsToken !== wasmToken) {
        details.push(`token: TS=${getTokenName(tsToken)} vs WASM=${getTokenName(wasmToken)}`);
        mismatch = true;
    }
    if (tsPos !== wasmPos) {
        details.push(`start: TS=${tsPos} vs WASM=${wasmPos}`);
        mismatch = true;
    }
    if (tsEnd !== wasmEnd) {
        details.push(`end: TS=${tsEnd} vs WASM=${wasmEnd}`);
        mismatch = true;
    }
    if (tsText !== wasmText) {
        details.push(`text: TS="${tsText.substring(0, 20)}" vs WASM="${wasmText.substring(0, 20)}"`);
        mismatch = true;
    }
    
    if (mismatch) {
        mismatches++;
        console.log(`❌ Mismatch at token #${tokenCount}:`);
        for (const d of details) {
            console.log(`   ${d}`);
        }
        console.log(`   Context: "${sourceText.substring(Math.max(0, tsPos - 10), tsPos + 30).replace(/\n/g, "\\n")}"`);
        console.log();
        
        if (mismatches >= maxMismatches) {
            console.log(`\n⚠️  Stopping after ${maxMismatches} mismatches.`);
            break;
        }
    }
    
    // Check for end of file
    if (tsToken === ts.SyntaxKind.EndOfFileToken || wasmToken === ts.SyntaxKind.EndOfFileToken) {
        if (tsToken !== wasmToken) {
            console.log(`❌ One scanner reached EOF before the other!`);
            console.log(`   TS token: ${getTokenName(tsToken)}, WASM token: ${getTokenName(wasmToken)}`);
            mismatches++;
        }
        break;
    }
}

// Clean up
wasmScanner.free();

// Report results
console.log(`\n${"=".repeat(60)}`);
console.log(`📈 VERIFICATION RESULTS`);
console.log(`${"=".repeat(60)}`);
console.log(`   Total tokens scanned: ${tokenCount}`);
console.log(`   Mismatches found: ${mismatches}`);

if (mismatches === 0) {
    console.log(`\n✅ SUCCESS: Rust scanner matches TypeScript scanner exactly!`);
    process.exit(0);
} else {
    console.log(`\n❌ FAILURE: ${mismatches} mismatches found.`);
    console.log(`   Fix the discrepancies in wasm/src/scanner_impl.rs`);
    process.exit(1);
}
