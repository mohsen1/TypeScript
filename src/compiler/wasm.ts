/**
 * Wasm bridge for loading Rust-compiled WebAssembly modules.
 * The wasm artifacts live in built/local/wasm/ after `hereby local`.
 * This module provides a lazy-load mechanism to avoid bundler issues.
 */

import * as path from "path";
import { Comparison } from "./_namespaces/ts.js";

// =============================================================================
// Wasm Module Interface
// =============================================================================

/** @internal */
export interface WasmModule {
    // POC function
    add(a: number, b: number): number;

    // String comparison utilities (Phase 1.1)
    compareStringsCaseSensitive(a: string | undefined, b: string | undefined): Comparison;
    compareStringsCaseInsensitive(a: string | undefined, b: string | undefined): Comparison;
    compareStringsCaseInsensitiveEslintCompatible(a: string | undefined, b: string | undefined): Comparison;
    equateStringsCaseSensitive(a: string, b: string): boolean;
    equateStringsCaseInsensitive(a: string, b: string): boolean;

    // Path utilities (Phase 1.2)
    isAnyDirectorySeparator(charCode: number): boolean;
    normalizeSlashes(path: string): string;
    hasTrailingDirectorySeparator(path: string): boolean;
    pathIsRelative(path: string): boolean;
    removeTrailingDirectorySeparator(path: string): string;
    ensureTrailingDirectorySeparator(path: string): string;
    hasExtension(fileName: string): boolean;
    getBaseFileName(path: string): string;
    fileExtensionIs(path: string, extension: string): boolean;
}

// =============================================================================
// Module Loading
// =============================================================================

let wasmModule: WasmModule | undefined;
let wasmLoadError: Error | undefined;

/**
 * Lazily loads the wasm module. Returns undefined if loading fails.
 * The wasm shim is located relative to the bundled tsc.js output.
 * @internal
 */
export function getWasm(): WasmModule | undefined {
    if (wasmModule) return wasmModule;
    if (wasmLoadError) return undefined;

    try {
        // At runtime, __dirname points to built/local/ (where tsc.js lives).
        // The wasm shim is at built/local/wasm/wasm.js.
        // We use a dynamic require to prevent esbuild from bundling the shim.
        const wasmPath = path.join(__dirname, "wasm", "wasm.js");
        wasmModule = require(wasmPath) as WasmModule;
        return wasmModule;
    }
    catch (e) {
        wasmLoadError = e instanceof Error ? e : new Error(String(e));
        return undefined;
    }
}

/**
 * Check if wasm module is available.
 * @internal
 */
export function isWasmAvailable(): boolean {
    return getWasm() !== undefined;
}

// =============================================================================
// POC Functions
// =============================================================================

/**
 * Calls the Rust `add` function via wasm. Returns undefined if wasm is unavailable.
 * @internal
 */
export function wasmAdd(a: number, b: number): number | undefined {
    const wasm = getWasm();
    return wasm?.add(a, b);
}

// =============================================================================
// String Comparison Utilities (Phase 1.1)
// =============================================================================

/**
 * Compare two strings using a case-sensitive ordinal comparison (Rust implementation).
 * Falls back to undefined if wasm is unavailable.
 * @internal
 */
export function wasmCompareStringsCaseSensitive(a: string | undefined, b: string | undefined): Comparison | undefined {
    const wasm = getWasm();
    return wasm?.compareStringsCaseSensitive(a, b);
}

/**
 * Compare two strings using a case-insensitive ordinal comparison (Rust implementation).
 * Falls back to undefined if wasm is unavailable.
 * @internal
 */
export function wasmCompareStringsCaseInsensitive(a: string | undefined, b: string | undefined): Comparison | undefined {
    const wasm = getWasm();
    return wasm?.compareStringsCaseInsensitive(a, b);
}

/**
 * Compare two strings using eslint-compatible case-insensitive comparison (Rust implementation).
 * Falls back to undefined if wasm is unavailable.
 * @internal
 */
export function wasmCompareStringsCaseInsensitiveEslintCompatible(a: string | undefined, b: string | undefined): Comparison | undefined {
    const wasm = getWasm();
    return wasm?.compareStringsCaseInsensitiveEslintCompatible(a, b);
}

/**
 * Check if two strings are equal (case-sensitive, Rust implementation).
 * Falls back to undefined if wasm is unavailable.
 * @internal
 */
export function wasmEquateStringsCaseSensitive(a: string, b: string): boolean | undefined {
    const wasm = getWasm();
    return wasm?.equateStringsCaseSensitive(a, b);
}

/**
 * Check if two strings are equal (case-insensitive, Rust implementation).
 * Falls back to undefined if wasm is unavailable.
 * @internal
 */
export function wasmEquateStringsCaseInsensitive(a: string, b: string): boolean | undefined {
    const wasm = getWasm();
    return wasm?.equateStringsCaseInsensitive(a, b);
}

// =============================================================================
// Path Utilities (Phase 1.2)
// =============================================================================

/**
 * Determines whether a charCode corresponds to `/` or `\` (Rust implementation).
 * @internal
 */
export function wasmIsAnyDirectorySeparator(charCode: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.isAnyDirectorySeparator(charCode);
}

/**
 * Normalize path separators, converting `\` into `/` (Rust implementation).
 * @internal
 */
export function wasmNormalizeSlashes(pathStr: string): string | undefined {
    const wasm = getWasm();
    return wasm?.normalizeSlashes(pathStr);
}

/**
 * Determines whether a path has a trailing separator (Rust implementation).
 * @internal
 */
export function wasmHasTrailingDirectorySeparator(pathStr: string): boolean | undefined {
    const wasm = getWasm();
    return wasm?.hasTrailingDirectorySeparator(pathStr);
}

/**
 * Determines whether a path starts with a relative path component (Rust implementation).
 * @internal
 */
export function wasmPathIsRelative(pathStr: string): boolean | undefined {
    const wasm = getWasm();
    return wasm?.pathIsRelative(pathStr);
}

/**
 * Removes a trailing directory separator from a path (Rust implementation).
 * @internal
 */
export function wasmRemoveTrailingDirectorySeparator(pathStr: string): string | undefined {
    const wasm = getWasm();
    return wasm?.removeTrailingDirectorySeparator(pathStr);
}

/**
 * Ensures a path has a trailing directory separator (Rust implementation).
 * @internal
 */
export function wasmEnsureTrailingDirectorySeparator(pathStr: string): string | undefined {
    const wasm = getWasm();
    return wasm?.ensureTrailingDirectorySeparator(pathStr);
}

/**
 * Determines whether a path has an extension (Rust implementation).
 * @internal
 */
export function wasmHasExtension(fileName: string): boolean | undefined {
    const wasm = getWasm();
    return wasm?.hasExtension(fileName);
}

/**
 * Returns the path except for its containing directory name (Rust implementation).
 * @internal
 */
export function wasmGetBaseFileName(pathStr: string): string | undefined {
    const wasm = getWasm();
    return wasm?.getBaseFileName(pathStr);
}

/**
 * Check if path ends with a specific extension (Rust implementation).
 * @internal
 */
export function wasmFileExtensionIs(pathStr: string, extension: string): boolean | undefined {
    const wasm = getWasm();
    return wasm?.fileExtensionIs(pathStr, extension);
}
