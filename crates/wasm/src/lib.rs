//! WebAssembly bindings for the TypeScript Rust port.
//!
//! This crate provides WASM-compatible exports for use in browsers and Node.js.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
