//! Binder Modules
//!
//! Contains symbol binding implementations for TypeScript constructs.

pub mod enums;

pub use enums::{
    EnumBinder, EnumBindingError, EnumMemberSymbol, EnumSymbol, EnumSymbolFlags,
    evaluate_constant_expression, is_valid_enum_member_name,
};
