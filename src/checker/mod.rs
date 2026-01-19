//! Type Checker Modules
//!
//! Contains type checking implementations for TypeScript constructs.

pub mod enums;
pub mod namespaces;

pub use enums::{
    EnumChecker, EnumDeclaration, EnumError, EnumMember, EnumMemberValue, EnumType,
    compute_enum_values, parse_string_enum_member, validate_ambient_enum, validate_string_enum,
};
pub use namespaces::{
    ExportVisibility, NamespaceChecker, NamespaceDeclaration, NamespaceError,
    NamespaceMember, NamespaceMemberKind, ResolvedNamespace, qualify_name, split_qualified_name,
};
