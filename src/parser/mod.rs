//! Parser module for TypeScript parsing
//!
//! This module contains the error recovery logic and supporting utilities
//! for the ThinParser.

pub mod error_recovery;

pub use error_recovery::{ErrorRecovery, RecoveryStrategy, SyncPoint};
