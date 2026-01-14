#!/usr/bin/env node
/**
 * Test Command Helper - Show available testing commands for Project Zang
 * 
 * Usage: node wasm/scripts/help.mjs
 */

const commands = {
  "Rust Unit Tests": {
    "Run all tests": "./wasm/test.sh",
    "Run specific test": "./wasm/test.sh test_name", 
    "Run benchmarks": "./wasm/test.sh --bench",
    "Rebuild Docker image": "./wasm/test.sh --rebuild",
    "Clean cache": "./wasm/test.sh --clean"
  },
  
  "TypeScript Conformance": {
    "Quick conformance check": "./wasm/differential-test/run-conformance.sh --max=1000",
    "Full conformance suite": "./wasm/differential-test/run-conformance.sh --all",
    "Test compiler category": "./wasm/differential-test/run-conformance.sh --category=compiler",
    "Test conformance category": "./wasm/differential-test/run-conformance.sh --category=conformance"
  },
  
  "Error Analysis": {
    "Find TS2454 (used before assigned)": "node wasm/differential-test/find-ts2454.mjs",
    "Find TS2322 (not assignable)": "node wasm/differential-test/find-ts2322.mjs", 
    "Find TS2339 (property doesn't exist)": "node wasm/differential-test/find-ts2339.mjs",
    "Find TS2564 (property not initialized)": "node wasm/differential-test/find-ts2564.mjs"
  },
  
  "Individual Testing": {
    "Test single file": "node wasm/scripts/run-single-test.mjs tests/cases/compiler/2dArrays.ts",
    "Test with verbose output": "node wasm/scripts/run-single-test.mjs path/to/test.ts --verbose --thin",
    "Compare baselines": "node wasm/scripts/compare-baselines.mjs 100 compiler",
    "Validate WASM module": "node wasm/scripts/validate-wasm.mjs",
    "Run batch tests": "node wasm/scripts/run-batch-tests.mjs"
  }
};

console.log("🧪 Project Zang - Testing Commands\n");

Object.entries(commands).forEach(([category, cmds]) => {
  console.log(`📁 ${category}`);
  console.log("─".repeat(50));
  
  Object.entries(cmds).forEach(([desc, cmd]) => {
    console.log(`  ${desc.padEnd(35)} ${cmd}`);
  });
  
  console.log("");
});

console.log("📖 For detailed testing guide, see: wasm/TESTING.md");
console.log("📊 For conformance metrics, run: ./wasm/differential-test/run-conformance.sh --max=1000");
console.log("");