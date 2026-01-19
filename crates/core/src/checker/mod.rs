//! Type checker implementation for TypeScript.
//!
//! This module provides:
//! - Type definitions (Type enum, ObjectType, etc.)
//! - Type guards (typeof, instanceof, user-defined)
//! - Assertion functions (asserts x is T)
//! - Type predicates and discriminated union narrowing
//! - Control flow analysis for type narrowing
//! - Const assertions and literal widening
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
//!
//! # Const Assertions
//!
//! Const assertions preserve literal types and make values readonly:
//!
//! ```ignore
//! // Without as const - type is string[]
//! const arr = ["a", "b", "c"];
//!
//! // With as const - type is readonly ["a", "b", "c"]
//! const arr = ["a", "b", "c"] as const;
//!
//! // Objects become readonly with literal property types
//! const obj = { x: 10 } as const; // { readonly x: 10 }
//! ```
//!
//! # Literal Widening
//!
//! Fresh literal types widen to their base types in certain contexts:
//!
//! ```ignore
//! const x = "hello"; // type is "hello" (literal)
//! let y = "hello";   // type is string (widened)
//!
//! // Literals don't widen in const contexts
//! const obj = { x: 10 } as const; // x stays as 10, not number
//! ```

pub mod assertions;
pub mod const_assertions;
pub mod predicates;
pub mod type_defs;
pub mod type_guards;
pub mod widening;

pub use assertions::*;
pub use const_assertions::*;
pub use predicates::*;
pub use type_defs::*;
pub use type_guards::*;
pub use widening::*;
