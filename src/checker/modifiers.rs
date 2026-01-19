//! Modifier Handling for Class Members
//!
//! This module handles access modifiers (public, private, protected),
//! static modifiers, readonly, abstract, and override.

/// Access modifier for class members
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccessModifier {
    #[default]
    Public,
    Private,
    Protected,
}

impl AccessModifier {
    /// Check if this modifier allows access from outside the class
    pub fn is_accessible_from_outside(&self) -> bool {
        matches!(self, AccessModifier::Public)
    }

    /// Check if this modifier allows access from derived classes
    pub fn is_accessible_from_derived(&self) -> bool {
        matches!(self, AccessModifier::Public | AccessModifier::Protected)
    }
}

/// Modifier flags for class members
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ModifierFlags(u32);

impl ModifierFlags {
    pub const NONE: ModifierFlags = ModifierFlags(0);
    pub const EXPORT: ModifierFlags = ModifierFlags(1 << 0);
    pub const AMBIENT: ModifierFlags = ModifierFlags(1 << 1);
    pub const PUBLIC: ModifierFlags = ModifierFlags(1 << 2);
    pub const PRIVATE: ModifierFlags = ModifierFlags(1 << 3);
    pub const PROTECTED: ModifierFlags = ModifierFlags(1 << 4);
    pub const STATIC: ModifierFlags = ModifierFlags(1 << 5);
    pub const READONLY: ModifierFlags = ModifierFlags(1 << 6);
    pub const ABSTRACT: ModifierFlags = ModifierFlags(1 << 7);
    pub const ASYNC: ModifierFlags = ModifierFlags(1 << 8);
    pub const DEFAULT: ModifierFlags = ModifierFlags(1 << 9);
    pub const CONST: ModifierFlags = ModifierFlags(1 << 10);
    pub const OVERRIDE: ModifierFlags = ModifierFlags(1 << 11);
    pub const IN: ModifierFlags = ModifierFlags(1 << 12);
    pub const OUT: ModifierFlags = ModifierFlags(1 << 13);
    pub const DECORATOR: ModifierFlags = ModifierFlags(1 << 14);
    pub const ACCESSOR: ModifierFlags = ModifierFlags(1 << 15);

    // Compound flags
    pub const ACCESSIBILITY: ModifierFlags = ModifierFlags(
        Self::PUBLIC.0 | Self::PRIVATE.0 | Self::PROTECTED.0
    );
    pub const PARAMETER_PROPERTY: ModifierFlags = ModifierFlags(
        Self::ACCESSIBILITY.0 | Self::READONLY.0 | Self::OVERRIDE.0
    );
    pub const NON_PUBLIC_ACCESSIBILITY: ModifierFlags = ModifierFlags(
        Self::PRIVATE.0 | Self::PROTECTED.0
    );
    pub const TYPE_SCRIPT_MODIFIER: ModifierFlags = ModifierFlags(
        Self::AMBIENT.0 | Self::PUBLIC.0 | Self::PRIVATE.0 | Self::PROTECTED.0 |
        Self::READONLY.0 | Self::ABSTRACT.0 | Self::OVERRIDE.0
    );
    pub const EXPORT_DEFAULT: ModifierFlags = ModifierFlags(Self::EXPORT.0 | Self::DEFAULT.0);

    pub fn contains(&self, other: ModifierFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn intersects(&self, other: ModifierFlags) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Get the access modifier
    pub fn access_modifier(&self) -> AccessModifier {
        if self.contains(Self::PRIVATE) {
            AccessModifier::Private
        } else if self.contains(Self::PROTECTED) {
            AccessModifier::Protected
        } else {
            AccessModifier::Public
        }
    }

    pub fn is_static(&self) -> bool {
        self.contains(Self::STATIC)
    }

    pub fn is_readonly(&self) -> bool {
        self.contains(Self::READONLY)
    }

    pub fn is_abstract(&self) -> bool {
        self.contains(Self::ABSTRACT)
    }

    pub fn is_override(&self) -> bool {
        self.contains(Self::OVERRIDE)
    }

    pub fn is_async(&self) -> bool {
        self.contains(Self::ASYNC)
    }

    pub fn is_accessor(&self) -> bool {
        self.contains(Self::ACCESSOR)
    }
}

impl std::ops::BitOr for ModifierFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        ModifierFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ModifierFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for ModifierFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        ModifierFlags(self.0 & rhs.0)
    }
}

/// Diagnostic for modifier errors
#[derive(Debug, Clone)]
pub struct ModifierError {
    pub message: String,
    pub code: u32,
}

/// Diagnostic codes for modifier errors
pub mod error_codes {
    pub const DUPLICATE_MODIFIER: u32 = 1030;
    pub const ACCESSOR_MUST_MATCH: u32 = 1033;
    pub const PRIVATE_PROTECTED_CONFLICT: u32 = 1029;
    pub const ABSTRACT_NOT_IN_CLASS: u32 = 1244;
    pub const ABSTRACT_WITH_BODY: u32 = 1245;
    pub const NON_ABSTRACT_WITHOUT_BODY: u32 = 1246;
    pub const STATIC_ABSTRACT: u32 = 1243;
    pub const OVERRIDE_WITHOUT_EXTENDS: u32 = 4112;
    pub const OVERRIDE_MEMBER_NOT_FOUND: u32 = 4113;
    pub const OVERRIDE_ACCESSOR_MISMATCH: u32 = 4114;
    pub const PRIVATE_OVERRIDE: u32 = 4115;
    pub const READONLY_PARAMETER: u32 = 1090;
    pub const ACCESSOR_READONLY_CONFLICT: u32 = 1091;
}

/// Validate modifier combinations
pub fn validate_modifiers(flags: ModifierFlags, context: &ModifierContext) -> Vec<ModifierError> {
    let mut errors = Vec::new();

    // Check for conflicting accessibility modifiers
    let access_count = [
        flags.contains(ModifierFlags::PUBLIC),
        flags.contains(ModifierFlags::PRIVATE),
        flags.contains(ModifierFlags::PROTECTED),
    ].iter().filter(|&&x| x).count();

    if access_count > 1 {
        errors.push(ModifierError {
            message: "An accessibility modifier already exists on this declaration".to_string(),
            code: error_codes::PRIVATE_PROTECTED_CONFLICT,
        });
    }

    // Abstract checks
    if flags.is_abstract() {
        if !context.in_abstract_class && !context.is_class_declaration {
            errors.push(ModifierError {
                message: "'abstract' modifier can only appear on a class, method, or property declaration".to_string(),
                code: error_codes::ABSTRACT_NOT_IN_CLASS,
            });
        }

        if flags.is_static() {
            errors.push(ModifierError {
                message: "'static' modifier cannot be used with 'abstract' modifier".to_string(),
                code: error_codes::STATIC_ABSTRACT,
            });
        }
    }

    // Override checks
    if flags.is_override() {
        if flags.contains(ModifierFlags::PRIVATE) {
            errors.push(ModifierError {
                message: "'override' modifier cannot be used with 'private' modifier".to_string(),
                code: error_codes::PRIVATE_OVERRIDE,
            });
        }

        if !context.has_base_class {
            errors.push(ModifierError {
                message: "This member cannot have an 'override' modifier because its containing class does not extend another class".to_string(),
                code: error_codes::OVERRIDE_WITHOUT_EXTENDS,
            });
        }
    }

    errors
}

/// Context for modifier validation
#[derive(Debug, Clone, Default)]
pub struct ModifierContext {
    pub in_abstract_class: bool,
    pub is_class_declaration: bool,
    pub has_base_class: bool,
    pub is_constructor: bool,
    pub is_parameter: bool,
}

impl ModifierContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_class(is_abstract: bool) -> Self {
        ModifierContext {
            in_abstract_class: is_abstract,
            is_class_declaration: true,
            has_base_class: false,
            is_constructor: false,
            is_parameter: false,
        }
    }

    pub fn for_member(in_abstract_class: bool, has_base_class: bool) -> Self {
        ModifierContext {
            in_abstract_class,
            is_class_declaration: false,
            has_base_class,
            is_constructor: false,
            is_parameter: false,
        }
    }

    pub fn for_constructor_param(has_base_class: bool) -> Self {
        ModifierContext {
            in_abstract_class: false,
            is_class_declaration: false,
            has_base_class,
            is_constructor: true,
            is_parameter: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_modifier_default() {
        let flags = ModifierFlags::NONE;
        assert_eq!(flags.access_modifier(), AccessModifier::Public);
    }

    #[test]
    fn test_access_modifier_private() {
        let flags = ModifierFlags::PRIVATE;
        assert_eq!(flags.access_modifier(), AccessModifier::Private);
        assert!(!flags.access_modifier().is_accessible_from_outside());
        assert!(!flags.access_modifier().is_accessible_from_derived());
    }

    #[test]
    fn test_access_modifier_protected() {
        let flags = ModifierFlags::PROTECTED;
        assert_eq!(flags.access_modifier(), AccessModifier::Protected);
        assert!(!flags.access_modifier().is_accessible_from_outside());
        assert!(flags.access_modifier().is_accessible_from_derived());
    }

    #[test]
    fn test_modifier_flags_combinations() {
        let flags = ModifierFlags::PUBLIC | ModifierFlags::STATIC | ModifierFlags::READONLY;
        assert!(flags.contains(ModifierFlags::PUBLIC));
        assert!(flags.is_static());
        assert!(flags.is_readonly());
        assert!(!flags.is_abstract());
    }

    #[test]
    fn test_conflicting_access_modifiers() {
        let flags = ModifierFlags::PUBLIC | ModifierFlags::PRIVATE;
        let context = ModifierContext::for_member(false, false);
        let errors = validate_modifiers(flags, &context);
        assert!(errors.iter().any(|e| e.code == error_codes::PRIVATE_PROTECTED_CONFLICT));
    }

    #[test]
    fn test_abstract_static_conflict() {
        let flags = ModifierFlags::ABSTRACT | ModifierFlags::STATIC;
        let context = ModifierContext::for_member(true, false);
        let errors = validate_modifiers(flags, &context);
        assert!(errors.iter().any(|e| e.code == error_codes::STATIC_ABSTRACT));
    }

    #[test]
    fn test_override_without_extends() {
        let flags = ModifierFlags::OVERRIDE;
        let context = ModifierContext::for_member(false, false);
        let errors = validate_modifiers(flags, &context);
        assert!(errors.iter().any(|e| e.code == error_codes::OVERRIDE_WITHOUT_EXTENDS));
    }

    #[test]
    fn test_override_private_conflict() {
        let flags = ModifierFlags::OVERRIDE | ModifierFlags::PRIVATE;
        let context = ModifierContext::for_member(false, true);
        let errors = validate_modifiers(flags, &context);
        assert!(errors.iter().any(|e| e.code == error_codes::PRIVATE_OVERRIDE));
    }

    #[test]
    fn test_valid_override() {
        let flags = ModifierFlags::OVERRIDE | ModifierFlags::PUBLIC;
        let context = ModifierContext::for_member(false, true);
        let errors = validate_modifiers(flags, &context);
        assert!(errors.is_empty());
    }
}
