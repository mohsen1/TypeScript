//! TypeScript Class Type Checker
//!
//! This module provides type checking for TypeScript classes including:
//!
//! - Class declarations and expressions
//! - Inheritance (extends and implements)
//! - Member accessibility (public, private, protected)
//! - Static vs instance members
//! - Abstract classes and methods
//! - Override modifier checking
//! - Constructor parameter properties
//! - Accessor properties (get/set)
//!
//! # Example
//!
//! ```rust
//! use ts_checker::{ClassChecker, ClassType, ClassMember, MemberKind, ModifierFlags};
//!
//! let mut checker = ClassChecker::new();
//!
//! // Create a class with members
//! let mut class = ClassType::new(0, "Person");
//! class.members.add_member(ClassMember::new(
//!     0, "name", MemberKind::Property, ModifierFlags::PUBLIC
//! ));
//!
//! // Check the class
//! let class_id = checker.check_class(class, false);
//! assert!(checker.get_diagnostics().is_empty());
//! ```

pub mod modifiers;
pub mod members;
pub mod inheritance;
pub mod class;

pub use modifiers::{AccessModifier, ModifierFlags, ModifierContext, ModifierError};
pub use members::{ClassMember, MemberKind, MemberTable, ParameterInfo, TypeId, MemberId, MemberError};
pub use inheritance::{ClassType, InterfaceType, TypeRegistry, InheritanceChecker, TypeParameterInfo};
pub use class::{ClassChecker, ClassCheckOptions, ClassDiagnostic, AccessContext};
