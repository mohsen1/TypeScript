//! TSConfig parsing and compiler options.
//!
//! This module provides:
//! - `CompilerOptions`: All TypeScript compiler options
//! - `RawTsConfig`: Raw tsconfig.json structure
//! - `ResolvedTsConfig`: Fully resolved config with inheritance
//! - `TsConfigParser`: Parser with extends resolution
//!
//! # Example
//!
//! ```ignore
//! use tsrs_core::config::{parse_config_file, find_config_file};
//! use std::path::Path;
//!
//! // Find and parse tsconfig.json
//! if let Some(config_path) = find_config_file(Path::new(".")) {
//!     let config = parse_config_file(&config_path)?;
//!     println!("Target: {:?}", config.compiler_options.target);
//! }
//! ```

pub mod options;
pub mod parser;
pub mod tsconfig;

pub use options::*;
pub use parser::{find_config_file, parse_config_file, TsConfigError, TsConfigParser};
pub use tsconfig::*;
