//! Type Checker Modules
//!
//! Contains type checking implementations for TypeScript constructs.

pub mod compatibility;
pub mod enums;
pub mod exhaustiveness;
pub mod namespaces;
pub mod union;

pub use compatibility::{
    CompatibilityChecker, CompatibilityError, CompatibilityOptions, CompatibilityResult,
    is_subtype, is_assignable_relaxed, are_equivalent, get_excess_properties, get_missing_properties,
};
pub use enums::{
    EnumChecker, EnumDeclaration, EnumError, EnumMember, EnumMemberValue, EnumType,
    compute_enum_values, parse_string_enum_member, validate_ambient_enum, validate_string_enum,
};
pub use exhaustiveness::{
    ExhaustivenessChecker, ExhaustivenessResult, ExhaustivenessWarning, SwitchCase,
    assert_never_check, narrow_after_case,
};
pub use namespaces::{
    ExportVisibility, NamespaceChecker, NamespaceDeclaration, NamespaceError,
    NamespaceMember, NamespaceMemberKind, ResolvedNamespace, qualify_name, split_qualified_name,
};
pub use union::{
    UnionChecker, DiscriminantProperty, DiscriminatedUnionInfo, NarrowingResult,
    create_union_from_types, simplify_union, get_common_properties,
};
