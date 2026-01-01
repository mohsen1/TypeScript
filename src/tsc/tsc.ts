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
