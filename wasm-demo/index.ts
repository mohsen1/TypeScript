/**
 * Minimal TypeScript bridge to the Rust wasm build.
 * Build/run via: npm run wasm:demo
 */
async function main() {
    // Dynamic import keeps this demo isolated from the main TS build.
    // Relative to the compiled output in wasm-demo/dist.
    // @ts-ignore wasm-pack generated module has no TypeScript types
    const wasmModule = await import("../../wasm/pkg/wasm.js");
    const { add } = wasmModule as any;
    const result = add(2, 3);
    console.log(`[wasm] add(2, 3) = ${result}`);
}

main().catch((err) => {
    console.error("[wasm] demo failed", err);
});
