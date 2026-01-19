//! Parser module for TypeScript parsing
//!
//! This module provides:
//! - Main Parser struct with scanner integration
//! - Statement parsing
//! - Expression parsing with precedence climbing
//! - Type annotation parsing
//! - Error recovery
//! - Automatic Semicolon Insertion (ASI)

pub mod error_recovery;
pub mod statements;
pub mod expressions;
pub mod types;
pub mod parser;

pub use error_recovery::{ErrorRecovery, RecoveryStrategy, SyncPoint};
pub use statements::{StatementKind, VarDeclKind, ForVariant, ExportKind, ImportKind, ASIContext};
pub use expressions::{Precedence, ExpressionKind, ObjectMemberKind, ArrayElementKind};
pub use types::{TypeNode, TypeParam, ObjectMember, TypeParameterDecl, Modifier};
pub use parser::{Parser, ParserOptions, ParserState};
