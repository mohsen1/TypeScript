import * as ts from "./_namespaces/ts.js";

// This file actually uses arguments passed on commandline and executes it

// WASM bridge verification (POC)
const wasmResult = ts.wasmAdd(2, 2);
if (wasmResult !== undefined) {
    ts.sys.write(`[WASM] 2 + 2 = ${wasmResult}${ts.sys.newLine}`);
}

// WASM string comparison verification (Phase 1.1)
const cmpResult = ts.wasmCompareStringsCaseSensitive("abc", "abd");
if (cmpResult !== undefined) {
    const cmpName = cmpResult === ts.Comparison.LessThan ? "LessThan" : cmpResult === ts.Comparison.EqualTo ? "EqualTo" : "GreaterThan";
    ts.sys.write(`[WASM] compareStringsCaseSensitive("abc", "abd") = ${cmpName}${ts.sys.newLine}`);
}

// WASM path utilities verification (Phase 1.2)
const normalizedPath = ts.wasmNormalizeSlashes("path\\to\\file");
if (normalizedPath !== undefined) {
    ts.sys.write(`[WASM] normalizeSlashes("path\\\\to\\\\file") = "${normalizedPath}"${ts.sys.newLine}`);
}

// WASM character classification verification (Phase 1.3)
const isDigitResult = ts.wasmIsDigit("5".charCodeAt(0));
if (isDigitResult !== undefined) {
    ts.sys.write(`[WASM] isDigit('5') = ${isDigitResult}${ts.sys.newLine}`);
}

// WASM scanner types verification (Phase 2)
const isKeywordResult = ts.wasmTokenIsKeyword(ts.SyntaxKind.ConstKeyword);
if (isKeywordResult !== undefined) {
    ts.sys.write(`[WASM] tokenIsKeyword(ConstKeyword) = ${isKeywordResult}${ts.sys.newLine}`);
}
const keywordText = ts.wasmKeywordToText(ts.SyntaxKind.AsyncKeyword);
if (keywordText !== undefined) {
    ts.sys.write(`[WASM] keywordToText(AsyncKeyword) = "${keywordText}"${ts.sys.newLine}`);
}
const asyncKind = ts.wasmTextToKeyword("async");
if (asyncKind !== undefined) {
    ts.sys.write(`[WASM] textToKeyword("async") = ${asyncKind} (AsyncKeyword=${ts.SyntaxKind.AsyncKeyword})${ts.sys.newLine}`);
}

// WASM scanner test (Phase 2)
const rustScanner = ts.wasmCreateScanner("const x = 42;", true);
if (rustScanner) {
    const tokens: number[] = [];
    let tok = rustScanner.scan();
    while (tok !== ts.SyntaxKind.EndOfFileToken) {
        tokens.push(tok);
        tok = rustScanner.scan();
    }
    // Expected: ConstKeyword(87), Identifier(80), EqualsToken(64), NumericLiteral(9), SemicolonToken(27)
    ts.sys.write(`[WASM] scanner("const x = 42;") = [${tokens.join(", ")}]${ts.sys.newLine}`);
    rustScanner.free();
}

// Check for --useRustScanner flag (Phase 2.5)
const useRustScannerIndex = ts.sys.args.indexOf("--useRustScanner");
if (useRustScannerIndex !== -1) {
    ts.sys.useRustScanner = true;
    // Remove the flag from args so it doesn't confuse the compiler
    ts.sys.args.splice(useRustScannerIndex, 1);
    ts.sys.write(`[WASM] Rust scanner enabled${ts.sys.newLine}`);
}

// Check for --useRustParser flag (Phase 3)
const useRustParserIndex = ts.sys.args.indexOf("--useRustParser");
if (useRustParserIndex !== -1) {
    ts.sys.useRustParser = true;
    // Remove the flag from args so it doesn't confuse the compiler
    ts.sys.args.splice(useRustParserIndex, 1);
    ts.sys.write(`[WASM] Rust parser enabled${ts.sys.newLine}`);
}

// enable deprecation logging
ts.Debug.loggingHost = {
    log(_level, s) {
        ts.sys.write(`${s || ""}${ts.sys.newLine}`);
    },
};

if (ts.Debug.isDebugging) {
    ts.Debug.enableDebugInfo();
}

if (ts.sys.tryEnableSourceMapsForHost && /^development$/i.test(ts.sys.getEnvironmentVariable("NODE_ENV"))) {
    ts.sys.tryEnableSourceMapsForHost();
}

if (ts.sys.setBlocking) {
    ts.sys.setBlocking();
}

ts.executeCommandLine(ts.sys, ts.noop, ts.sys.args);
