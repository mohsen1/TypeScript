//! Enum Type Checking
//!
//! Handles type checking for TypeScript enums:
//! - Numeric enums with auto-incrementing values
//! - String enums
//! - Const enums with inlining
//! - Enum member access and reverse mapping
//! - Ambient enums (declare enum)

use std::collections::HashMap;

/// Enum member value - can be numeric or string
#[derive(Debug, Clone, PartialEq)]
pub enum EnumMemberValue {
    /// Numeric value (can be computed)
    Number(i64),
    /// String literal value
    String(String),
    /// Computed value (expression that couldn't be evaluated at compile time)
    Computed,
}

/// Enum member declaration
#[derive(Debug, Clone)]
pub struct EnumMember<'a> {
    /// Member name
    pub name: &'a str,
    /// Resolved value
    pub value: EnumMemberValue,
    /// Whether this member has an explicit initializer
    pub has_initializer: bool,
    /// Source location (start, end)
    pub span: (usize, usize),
}

/// Enum declaration info
#[derive(Debug, Clone)]
pub struct EnumDeclaration<'a> {
    /// Enum name
    pub name: &'a str,
    /// Members
    pub members: Vec<EnumMember<'a>>,
    /// Is const enum (values inlined at usage)
    pub is_const: bool,
    /// Is ambient (declare enum)
    pub is_ambient: bool,
    /// Source location
    pub span: (usize, usize),
}

/// Enum type for type checking
#[derive(Debug, Clone)]
pub struct EnumType<'a> {
    /// Enum name
    pub name: &'a str,
    /// Member names to values mapping
    pub members: HashMap<&'a str, EnumMemberValue>,
    /// Is this a const enum
    pub is_const: bool,
    /// Is this a string enum (all members are strings)
    pub is_string_enum: bool,
    /// Is this an ambient enum
    pub is_ambient: bool,
}

/// Error types for enum checking
#[derive(Debug, Clone)]
pub enum EnumError {
    /// Duplicate member name
    DuplicateMember { name: String, span: (usize, usize) },
    /// String enum member without initializer after string member
    MissingInitializer { name: String, span: (usize, usize) },
    /// Computed value in const enum
    ComputedInConstEnum { name: String, span: (usize, usize) },
    /// Invalid member access
    InvalidMemberAccess { enum_name: String, member: String, span: (usize, usize) },
    /// Non-const enum cannot be used in const context
    NonConstInConstContext { name: String, span: (usize, usize) },
    /// Ambient enum member must have initializer
    AmbientMemberRequiresInitializer { name: String, span: (usize, usize) },
    /// Heterogeneous enum (mixed string/number without explicit values)
    HeterogeneousEnum { name: String, span: (usize, usize) },
}

/// Enum checker - handles type checking for enums
pub struct EnumChecker<'a> {
    /// Registered enums by name
    enums: HashMap<&'a str, EnumType<'a>>,
    /// Errors encountered during checking
    errors: Vec<EnumError>,
}

impl<'a> EnumChecker<'a> {
    pub fn new() -> Self {
        EnumChecker {
            enums: HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// Register an enum declaration
    pub fn register_enum(&mut self, decl: EnumDeclaration<'a>) -> Result<&EnumType<'a>, EnumError> {
        // Check for duplicate members
        let mut seen: HashMap<&str, (usize, usize)> = HashMap::new();
        for member in &decl.members {
            if let Some(prev_span) = seen.get(member.name) {
                let err = EnumError::DuplicateMember {
                    name: member.name.to_string(),
                    span: *prev_span,
                };
                self.errors.push(err.clone());
                return Err(err);
            }
            seen.insert(member.name, member.span);
        }

        // Build member map
        let mut members = HashMap::new();
        let mut has_string = false;
        let mut has_number = false;
        let mut has_computed = false;

        for member in &decl.members {
            match &member.value {
                EnumMemberValue::String(_) => {
                    has_string = true;
                }
                EnumMemberValue::Number(_) => {
                    has_number = true;
                }
                EnumMemberValue::Computed => {
                    if decl.is_const {
                        let err = EnumError::ComputedInConstEnum {
                            name: member.name.to_string(),
                            span: member.span,
                        };
                        self.errors.push(err.clone());
                        return Err(err);
                    }
                    has_computed = true;
                }
            }
            members.insert(member.name, member.value.clone());
        }

        // String enums have only string values (no numbers or computed)
        let is_string_enum = has_string && !has_number && !has_computed;

        let enum_type = EnumType {
            name: decl.name,
            members,
            is_const: decl.is_const,
            is_string_enum,
            is_ambient: decl.is_ambient,
        };

        self.enums.insert(decl.name, enum_type);
        Ok(self.enums.get(decl.name).unwrap())
    }

    /// Get an enum by name
    pub fn get_enum(&self, name: &str) -> Option<&EnumType<'a>> {
        self.enums.get(name)
    }

    /// Check enum member access (e.g., MyEnum.Value)
    pub fn check_member_access(
        &mut self,
        enum_name: &str,
        member_name: &str,
        span: (usize, usize),
    ) -> Result<&EnumMemberValue, EnumError> {
        if let Some(enum_type) = self.enums.get(enum_name) {
            if let Some(value) = enum_type.members.get(member_name) {
                Ok(value)
            } else {
                let err = EnumError::InvalidMemberAccess {
                    enum_name: enum_name.to_string(),
                    member: member_name.to_string(),
                    span,
                };
                self.errors.push(err.clone());
                Err(err)
            }
        } else {
            let err = EnumError::InvalidMemberAccess {
                enum_name: enum_name.to_string(),
                member: member_name.to_string(),
                span,
            };
            self.errors.push(err.clone());
            Err(err)
        }
    }

    /// Check reverse mapping access (e.g., MyEnum[0])
    pub fn check_reverse_mapping(
        &self,
        enum_name: &str,
        value: &EnumMemberValue,
    ) -> Option<&'a str> {
        if let Some(enum_type) = self.enums.get(enum_name) {
            // String enums don't have reverse mapping
            if enum_type.is_string_enum {
                return None;
            }
            for (name, member_value) in &enum_type.members {
                if member_value == value {
                    return Some(*name);
                }
            }
        }
        None
    }

    /// Get the inline value for a const enum member
    pub fn get_const_enum_value(
        &self,
        enum_name: &str,
        member_name: &str,
    ) -> Option<&EnumMemberValue> {
        if let Some(enum_type) = self.enums.get(enum_name) {
            if enum_type.is_const {
                return enum_type.members.get(member_name);
            }
        }
        None
    }

    /// Check if an enum is a const enum
    pub fn is_const_enum(&self, name: &str) -> bool {
        self.enums.get(name).map_or(false, |e| e.is_const)
    }

    /// Check if an enum is an ambient enum
    pub fn is_ambient_enum(&self, name: &str) -> bool {
        self.enums.get(name).map_or(false, |e| e.is_ambient)
    }

    /// Get all errors
    pub fn errors(&self) -> &[EnumError] {
        &self.errors
    }

    /// Clear errors
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }
}

impl<'a> Default for EnumChecker<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute numeric enum values with auto-increment
pub fn compute_enum_values<'a>(members: &[(&'a str, Option<i64>, (usize, usize))]) -> Vec<EnumMember<'a>> {
    let mut result = Vec::new();
    let mut next_value: i64 = 0;

    for (name, explicit_value, span) in members {
        let (value, has_initializer) = if let Some(v) = explicit_value {
            next_value = *v + 1;
            (EnumMemberValue::Number(*v), true)
        } else {
            let v = next_value;
            next_value += 1;
            (EnumMemberValue::Number(v), false)
        };

        result.push(EnumMember {
            name,
            value,
            has_initializer,
            span: *span,
        });
    }

    result
}

/// Parse a string enum member value
pub fn parse_string_enum_member<'a>(
    name: &'a str,
    value: &'a str,
    span: (usize, usize),
) -> EnumMember<'a> {
    EnumMember {
        name,
        value: EnumMemberValue::String(value.to_string()),
        has_initializer: true,
        span,
    }
}

/// Validate string enum - all members after a string member must have initializers
pub fn validate_string_enum(members: &[EnumMember<'_>]) -> Result<(), EnumError> {
    let mut found_string = false;

    for member in members {
        if let EnumMemberValue::String(_) = &member.value {
            found_string = true;
        } else if found_string && !member.has_initializer {
            return Err(EnumError::MissingInitializer {
                name: member.name.to_string(),
                span: member.span,
            });
        }
    }

    Ok(())
}

/// Validate ambient enum - all members must have initializers
pub fn validate_ambient_enum(members: &[EnumMember<'_>]) -> Result<(), EnumError> {
    for member in members {
        if !member.has_initializer {
            return Err(EnumError::AmbientMemberRequiresInitializer {
                name: member.name.to_string(),
                span: member.span,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_enum_auto_increment() {
        let members = compute_enum_values(&[
            ("A", None, (0, 1)),
            ("B", None, (2, 3)),
            ("C", None, (4, 5)),
        ]);

        assert_eq!(members.len(), 3);
        assert_eq!(members[0].value, EnumMemberValue::Number(0));
        assert_eq!(members[1].value, EnumMemberValue::Number(1));
        assert_eq!(members[2].value, EnumMemberValue::Number(2));
    }

    #[test]
    fn test_numeric_enum_explicit_values() {
        let members = compute_enum_values(&[
            ("A", Some(10), (0, 1)),
            ("B", None, (2, 3)),
            ("C", Some(20), (4, 5)),
            ("D", None, (6, 7)),
        ]);

        assert_eq!(members.len(), 4);
        assert_eq!(members[0].value, EnumMemberValue::Number(10));
        assert_eq!(members[1].value, EnumMemberValue::Number(11));
        assert_eq!(members[2].value, EnumMemberValue::Number(20));
        assert_eq!(members[3].value, EnumMemberValue::Number(21));
    }

    #[test]
    fn test_string_enum() {
        let mut checker = EnumChecker::new();

        let decl = EnumDeclaration {
            name: "Direction",
            members: vec![
                EnumMember {
                    name: "Up",
                    value: EnumMemberValue::String("UP".to_string()),
                    has_initializer: true,
                    span: (0, 2),
                },
                EnumMember {
                    name: "Down",
                    value: EnumMemberValue::String("DOWN".to_string()),
                    has_initializer: true,
                    span: (3, 7),
                },
            ],
            is_const: false,
            is_ambient: false,
            span: (0, 10),
        };

        let result = checker.register_enum(decl);
        assert!(result.is_ok());

        let enum_type = result.unwrap();
        assert!(enum_type.is_string_enum);
        assert!(!enum_type.is_const);
    }

    #[test]
    fn test_const_enum() {
        let mut checker = EnumChecker::new();

        let members = compute_enum_values(&[
            ("A", None, (0, 1)),
            ("B", None, (2, 3)),
        ]);

        let decl = EnumDeclaration {
            name: "Color",
            members,
            is_const: true,
            is_ambient: false,
            span: (0, 10),
        };

        let result = checker.register_enum(decl);
        assert!(result.is_ok());

        // Get const enum value for inlining
        let value = checker.get_const_enum_value("Color", "A");
        assert_eq!(value, Some(&EnumMemberValue::Number(0)));
    }

    #[test]
    fn test_duplicate_member_error() {
        let mut checker = EnumChecker::new();

        let decl = EnumDeclaration {
            name: "Test",
            members: vec![
                EnumMember {
                    name: "A",
                    value: EnumMemberValue::Number(0),
                    has_initializer: false,
                    span: (0, 1),
                },
                EnumMember {
                    name: "A",  // Duplicate!
                    value: EnumMemberValue::Number(1),
                    has_initializer: false,
                    span: (2, 3),
                },
            ],
            is_const: false,
            is_ambient: false,
            span: (0, 10),
        };

        let result = checker.register_enum(decl);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EnumError::DuplicateMember { .. }));
    }

    #[test]
    fn test_member_access() {
        let mut checker = EnumChecker::new();

        let members = compute_enum_values(&[
            ("Red", Some(0), (0, 3)),
            ("Green", Some(1), (4, 9)),
            ("Blue", Some(2), (10, 14)),
        ]);

        let decl = EnumDeclaration {
            name: "Color",
            members,
            is_const: false,
            is_ambient: false,
            span: (0, 20),
        };

        checker.register_enum(decl).unwrap();

        // Valid access
        let result = checker.check_member_access("Color", "Red", (0, 9));
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), EnumMemberValue::Number(0));

        // Invalid access
        let result = checker.check_member_access("Color", "Yellow", (0, 12));
        assert!(result.is_err());
    }

    #[test]
    fn test_reverse_mapping() {
        let mut checker = EnumChecker::new();

        let members = compute_enum_values(&[
            ("A", Some(0), (0, 1)),
            ("B", Some(1), (2, 3)),
        ]);

        let decl = EnumDeclaration {
            name: "Test",
            members,
            is_const: false,
            is_ambient: false,
            span: (0, 10),
        };

        checker.register_enum(decl).unwrap();

        // Numeric enum has reverse mapping
        let name = checker.check_reverse_mapping("Test", &EnumMemberValue::Number(0));
        assert_eq!(name, Some("A"));
    }

    #[test]
    fn test_string_enum_no_reverse_mapping() {
        let mut checker = EnumChecker::new();

        let decl = EnumDeclaration {
            name: "Direction",
            members: vec![
                EnumMember {
                    name: "Up",
                    value: EnumMemberValue::String("UP".to_string()),
                    has_initializer: true,
                    span: (0, 2),
                },
            ],
            is_const: false,
            is_ambient: false,
            span: (0, 10),
        };

        checker.register_enum(decl).unwrap();

        // String enum has no reverse mapping
        let name = checker.check_reverse_mapping("Direction", &EnumMemberValue::String("UP".to_string()));
        assert_eq!(name, None);
    }

    #[test]
    fn test_validate_string_enum() {
        // Valid: all have initializers
        let members = vec![
            EnumMember {
                name: "A",
                value: EnumMemberValue::String("a".to_string()),
                has_initializer: true,
                span: (0, 1),
            },
            EnumMember {
                name: "B",
                value: EnumMemberValue::String("b".to_string()),
                has_initializer: true,
                span: (2, 3),
            },
        ];
        assert!(validate_string_enum(&members).is_ok());

        // Invalid: missing initializer after string member
        let members = vec![
            EnumMember {
                name: "A",
                value: EnumMemberValue::String("a".to_string()),
                has_initializer: true,
                span: (0, 1),
            },
            EnumMember {
                name: "B",
                value: EnumMemberValue::Number(1),
                has_initializer: false,  // Error!
                span: (2, 3),
            },
        ];
        assert!(validate_string_enum(&members).is_err());
    }

    #[test]
    fn test_validate_ambient_enum() {
        // Valid: all have initializers
        let members = vec![
            EnumMember {
                name: "A",
                value: EnumMemberValue::Number(0),
                has_initializer: true,
                span: (0, 1),
            },
        ];
        assert!(validate_ambient_enum(&members).is_ok());

        // Invalid: missing initializer
        let members = vec![
            EnumMember {
                name: "A",
                value: EnumMemberValue::Number(0),
                has_initializer: false,
                span: (0, 1),
            },
        ];
        assert!(validate_ambient_enum(&members).is_err());
    }

    #[test]
    fn test_computed_in_const_enum_error() {
        let mut checker = EnumChecker::new();

        let decl = EnumDeclaration {
            name: "Test",
            members: vec![
                EnumMember {
                    name: "A",
                    value: EnumMemberValue::Computed,  // Not allowed in const enum
                    has_initializer: true,
                    span: (0, 1),
                },
            ],
            is_const: true,
            is_ambient: false,
            span: (0, 10),
        };

        let result = checker.register_enum(decl);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EnumError::ComputedInConstEnum { .. }));
    }
}
