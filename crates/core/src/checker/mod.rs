//! Type checker implementation for TypeScript.
//!
//! This module provides:
//! - Type definitions (Type enum, ObjectType, etc.)
//! - Type guards (typeof, instanceof, user-defined)
//! - Assertion functions (asserts x is T)
//! - Type predicates and discriminated union narrowing
//! - Control flow analysis for type narrowing
//!
//! # Type Guards
//!
//! Type guards narrow the type of a variable within a conditional block:
//!
//! ```ignore
//! // typeof type guard
//! if (typeof x === "string") {
//!     // x is narrowed to string
//! }
//!
//! // instanceof type guard
//! if (x instanceof Date) {
//!     // x is narrowed to Date
//! }
//!
//! // User-defined type guard
//! function isString(x: unknown): x is string {
//!     return typeof x === "string";
//! }
//! ```
//!
//! # Assertion Functions
//!
//! Assertion functions narrow types after they return:
//!
//! ```ignore
//! function assertString(x: unknown): asserts x is string {
//!     if (typeof x !== "string") throw new Error();
//! }
//!
//! assertString(value);
//! // value is narrowed to string here
//! ```
//!
//! # Discriminated Unions
//!
//! Discriminated unions use a common property to narrow types:
//!
//! ```ignore
//! type Shape =
//!     | { kind: "circle"; radius: number }
//!     | { kind: "square"; side: number };
//!
//! if (shape.kind === "circle") {
//!     // shape is narrowed to { kind: "circle"; radius: number }
//! }
//! ```

pub mod assertions;
pub mod predicates;
pub mod type_defs;
pub mod type_guards;

pub use assertions::*;
pub use predicates::*;
pub use type_defs::*;
pub use type_guards::*;
