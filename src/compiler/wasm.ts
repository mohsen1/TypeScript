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

    // Character classification (Phase 1.3 - Scanner prep)
    isLineBreak(ch: number): boolean;
    isWhiteSpaceSingleLine(ch: number): boolean;
    isWhiteSpaceLike(ch: number): boolean;
    isDigit(ch: number): boolean;
    isOctalDigit(ch: number): boolean;
    isHexDigit(ch: number): boolean;
    isASCIILetter(ch: number): boolean;
    isWordCharacter(ch: number): boolean;

    // Scanner types (Phase 2)
    tokenIsKeyword(token: number): boolean;
    tokenIsIdentifierOrKeyword(token: number): boolean;
    tokenIsReservedWord(token: number): boolean;
    tokenIsStrictModeReservedWord(token: number): boolean;
    tokenIsLiteral(token: number): boolean;
    tokenIsTemplateLiteral(token: number): boolean;
    tokenIsPunctuation(token: number): boolean;
    tokenIsAssignmentOperator(token: number): boolean;
    tokenIsTrivia(token: number): boolean;
    keywordToText(token: number): string | undefined;
    punctuationToText(token: number): string | undefined;
    textToKeyword(text: string): number | undefined;
    stringToToken(text: string): number;

    // Scanner class (Phase 2)
    ScannerState: WasmScannerStateClass;
    createScanner(text: string, skipTrivia: boolean): WasmScannerStateInstance;
}

/** @internal */
interface WasmScannerStateClass {
    new(text: string, skipTrivia: boolean): WasmScannerStateInstance;
}

/** @internal */
interface WasmScannerStateInstance {
    scan(): number;
    getPos(): number;
    getTokenFullStart(): number;
    getTokenStart(): number;
    getTokenEnd(): number;
    getToken(): number;
    getTokenValue(): string;
    getTokenText(): string;
    getTokenFlags(): number;
    hasPrecedingLineBreak(): boolean;
    isUnterminated(): boolean;
    isIdentifier(): boolean;
    isReservedWord(): boolean;
    setText(text: string, start?: number, length?: number): void;
    resetTokenState(pos: number): void;
    getText(): string;
    free(): void;
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

// =============================================================================
// Character Classification (Phase 1.3 - Scanner Prep)
// =============================================================================

/**
 * Check if character is a line break (Rust implementation).
 * @internal
 */
export function wasmIsLineBreak(ch: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.isLineBreak(ch);
}

/**
 * Check if character is single-line whitespace (Rust implementation).
 * @internal
 */
export function wasmIsWhiteSpaceSingleLine(ch: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.isWhiteSpaceSingleLine(ch);
}

/**
 * Check if character is any whitespace including line breaks (Rust implementation).
 * @internal
 */
export function wasmIsWhiteSpaceLike(ch: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.isWhiteSpaceLike(ch);
}

/**
 * Check if character is a decimal digit (Rust implementation).
 * @internal
 */
export function wasmIsDigit(ch: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.isDigit(ch);
}

/**
 * Check if character is an octal digit (Rust implementation).
 * @internal
 */
export function wasmIsOctalDigit(ch: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.isOctalDigit(ch);
}

/**
 * Check if character is a hex digit (Rust implementation).
 * @internal
 */
export function wasmIsHexDigit(ch: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.isHexDigit(ch);
}

/**
 * Check if character is an ASCII letter (Rust implementation).
 * @internal
 */
export function wasmIsASCIILetter(ch: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.isASCIILetter(ch);
}

/**
 * Check if character is a word character (Rust implementation).
 * @internal
 */
export function wasmIsWordCharacter(ch: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.isWordCharacter(ch);
}

// =============================================================================
// Scanner Types (Phase 2)
// =============================================================================

/**
 * Check if a token is a keyword (Rust implementation).
 * @internal
 */
export function wasmTokenIsKeyword(token: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.tokenIsKeyword(token);
}

/**
 * Check if a token is an identifier or keyword (Rust implementation).
 * @internal
 */
export function wasmTokenIsIdentifierOrKeyword(token: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.tokenIsIdentifierOrKeyword(token);
}

/**
 * Check if a token is a reserved word (Rust implementation).
 * @internal
 */
export function wasmTokenIsReservedWord(token: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.tokenIsReservedWord(token);
}

/**
 * Check if a token is a strict mode reserved word (Rust implementation).
 * @internal
 */
export function wasmTokenIsStrictModeReservedWord(token: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.tokenIsStrictModeReservedWord(token);
}

/**
 * Check if a token is a literal (Rust implementation).
 * @internal
 */
export function wasmTokenIsLiteral(token: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.tokenIsLiteral(token);
}

/**
 * Check if a token is a template literal (Rust implementation).
 * @internal
 */
export function wasmTokenIsTemplateLiteral(token: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.tokenIsTemplateLiteral(token);
}

/**
 * Check if a token is punctuation (Rust implementation).
 * @internal
 */
export function wasmTokenIsPunctuation(token: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.tokenIsPunctuation(token);
}

/**
 * Check if a token is an assignment operator (Rust implementation).
 * @internal
 */
export function wasmTokenIsAssignmentOperator(token: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.tokenIsAssignmentOperator(token);
}

/**
 * Check if a token is trivia (Rust implementation).
 * @internal
 */
export function wasmTokenIsTrivia(token: number): boolean | undefined {
    const wasm = getWasm();
    return wasm?.tokenIsTrivia(token);
}

/**
 * Get the text representation of a keyword token (Rust implementation).
 * @internal
 */
export function wasmKeywordToText(token: number): string | undefined {
    const wasm = getWasm();
    return wasm?.keywordToText(token);
}

/**
 * Get the text representation of a punctuation token (Rust implementation).
 * @internal
 */
export function wasmPunctuationToText(token: number): string | undefined {
    const wasm = getWasm();
    return wasm?.punctuationToText(token);
}

/**
 * Convert a string to its keyword SyntaxKind, if it's a keyword (Rust implementation).
 * Returns undefined if the text is not a keyword or wasm is unavailable.
 * @internal
 */
export function wasmTextToKeyword(text: string): number | undefined {
    const wasm = getWasm();
    return wasm?.textToKeyword(text);
}

/**
 * Get the token kind for a given text, including identifiers and keywords (Rust implementation).
 * Returns Identifier if the text is not a keyword.
 * Returns undefined if wasm is unavailable.
 * @internal
 */
export function wasmStringToToken(text: string): number | undefined {
    const wasm = getWasm();
    return wasm?.stringToToken(text);
}

// =============================================================================
// Scanner (Phase 2)
// =============================================================================

/**
 * Create a new Rust scanner instance.
 * Returns undefined if wasm is unavailable.
 * @internal
 */
export function wasmCreateScanner(text: string, skipTrivia: boolean): WasmScannerStateInstance | undefined {
    const wasm = getWasm();
    if (!wasm) return undefined;
    return wasm.createScanner(text, skipTrivia);
}

/**
 * Type alias for the Rust scanner instance.
 * @internal
 */
export type WasmScanner = WasmScannerStateInstance;
