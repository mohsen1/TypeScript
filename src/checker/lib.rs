//! TypeScript Class Type Checker Library
//!
//! This library provides comprehensive type checking for TypeScript classes:
//!
//! - **Class Declarations**: Validates class structure and member declarations
//! - **Inheritance**: Checks extends and implements clauses
//! - **Access Modifiers**: Validates public, private, and protected access
//! - **Static Members**: Distinguishes between static and instance members
//! - **Abstract Classes**: Validates abstract class and method usage
//! - **Override Modifier**: Ensures proper use of the override keyword
//! - **Parameter Properties**: Validates constructor parameter properties
//! - **Accessors**: Checks get/set accessor pairs
//!
//! # Quick Start
//!
//! ```rust
//! use ts_checker::{ClassChecker, ClassType, ClassMember, MemberKind, ModifierFlags};
//!
//! // Create a new class checker
//! let mut checker = ClassChecker::new();
//!
//! // Define a simple class
//! let mut class = ClassType::new(0, "Person");
//! class.members.add_member(ClassMember::new(
//!     0, "name", MemberKind::Property, ModifierFlags::PUBLIC
//! ));
//! class.members.add_member(ClassMember::new(
//!     0, "getName", MemberKind::Method, ModifierFlags::PUBLIC
//! ));
//!
//! // Check the class
//! let class_id = checker.check_class(class, false);
//!
//! // Verify no errors
//! assert!(checker.get_diagnostics().is_empty());
//! ```

pub mod modifiers;
pub mod members;
pub mod inheritance;
pub mod class;

// Re-export all public types
pub use modifiers::{
    AccessModifier,
    ModifierFlags,
    ModifierContext,
    ModifierError,
    validate_modifiers,
};

pub use members::{
    ClassMember,
    MemberKind,
    MemberTable,
    ParameterInfo,
    TypeId,
    MemberId,
    MemberError,
    check_accessor_pair,
    check_duplicate_members,
};

pub use inheritance::{
    ClassType,
    InterfaceType,
    TypeRegistry,
    InheritanceChecker,
    TypeParameterInfo,
    InheritanceError,
};

pub use class::{
    ClassChecker,
    ClassCheckOptions,
    ClassDiagnostic,
    AccessContext,
};
