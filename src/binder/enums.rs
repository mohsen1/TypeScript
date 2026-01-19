//! Enum Binder
//!
//! Binds enum declarations to symbols in the symbol table.
//! Handles:
//! - Enum declaration binding
//! - Enum member value computation
//! - Enum merging detection
//! - Const enum tracking

use std::collections::HashMap;

use crate::checker::enums::{EnumDeclaration, EnumMember, EnumMemberValue};

/// Symbol flags for enums
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumSymbolFlags {
    /// Is const enum
    pub is_const: bool,
    /// Is ambient (declare enum)
    pub is_ambient: bool,
    /// Has been merged with another declaration
    pub is_merged: bool,
}

impl Default for EnumSymbolFlags {
    fn default() -> Self {
        EnumSymbolFlags {
            is_const: false,
            is_ambient: false,
            is_merged: false,
        }
    }
}

/// Enum symbol representing a bound enum
#[derive(Debug, Clone)]
pub struct EnumSymbol<'a> {
    /// Enum name
    pub name: &'a str,
    /// Symbol flags
    pub flags: EnumSymbolFlags,
    /// Member symbols
    pub members: HashMap<&'a str, EnumMemberSymbol<'a>>,
    /// Declaration spans (for merged enums)
    pub declarations: Vec<(usize, usize)>,
}

/// Enum member symbol
#[derive(Debug, Clone)]
pub struct EnumMemberSymbol<'a> {
    /// Member name
    pub name: &'a str,
    /// Parent enum name
    pub parent: &'a str,
    /// Computed value
    pub value: EnumMemberValue,
    /// Declaration span
    pub span: (usize, usize),
}

/// Binding error for enums
#[derive(Debug, Clone)]
pub enum EnumBindingError {
    /// Duplicate enum member within same declaration
    DuplicateMember { name: String, span: (usize, usize) },
    /// Const enum merged with non-const enum
    ConstMergeConflict { name: String, span: (usize, usize) },
    /// Enum member initializer required (in certain contexts)
    InitializerRequired { name: String, span: (usize, usize) },
    /// Invalid initializer expression
    InvalidInitializer { name: String, span: (usize, usize) },
    /// Circular reference in enum initializer
    CircularReference { name: String, span: (usize, usize) },
}

/// Enum binder - binds enum declarations
pub struct EnumBinder<'a> {
    /// Bound enum symbols by name
    enums: HashMap<&'a str, EnumSymbol<'a>>,
    /// Errors encountered during binding
    errors: Vec<EnumBindingError>,
}

impl<'a> EnumBinder<'a> {
    pub fn new() -> Self {
        EnumBinder {
            enums: HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// Bind an enum declaration
    pub fn bind_enum(
        &mut self,
        name: &'a str,
        members: &[(&'a str, Option<EnumMemberValue>, (usize, usize))],
        is_const: bool,
        is_ambient: bool,
        span: (usize, usize),
    ) -> Result<&EnumSymbol<'a>, EnumBindingError> {
        // Check for existing enum with same name
        if let Some(existing) = self.enums.get(name) {
            // Check const compatibility
            if existing.flags.is_const != is_const {
                let err = EnumBindingError::ConstMergeConflict {
                    name: name.to_string(),
                    span,
                };
                self.errors.push(err.clone());
                return Err(err);
            }

            // Merge with existing enum
            return self.merge_enum(name, members, span);
        }

        // Check for duplicate members
        let mut seen: HashMap<&str, (usize, usize)> = HashMap::new();
        for (member_name, _, member_span) in members {
            if let Some(prev_span) = seen.get(member_name) {
                let err = EnumBindingError::DuplicateMember {
                    name: (*member_name).to_string(),
                    span: *prev_span,
                };
                self.errors.push(err.clone());
                return Err(err);
            }
            seen.insert(member_name, *member_span);
        }

        // Compute member values with auto-increment
        let bound_members = self.compute_member_values(name, members)?;

        // Create enum symbol
        let symbol = EnumSymbol {
            name,
            flags: EnumSymbolFlags {
                is_const,
                is_ambient,
                is_merged: false,
            },
            members: bound_members,
            declarations: vec![span],
        };

        self.enums.insert(name, symbol);
        Ok(self.enums.get(name).unwrap())
    }

    /// Merge additional members into existing enum
    fn merge_enum(
        &mut self,
        name: &'a str,
        members: &[(&'a str, Option<EnumMemberValue>, (usize, usize))],
        span: (usize, usize),
    ) -> Result<&EnumSymbol<'a>, EnumBindingError> {
        // Get the last value from existing members
        let existing = self.enums.get(name).unwrap();
        let last_value = existing
            .members
            .values()
            .filter_map(|m| match &m.value {
                EnumMemberValue::Number(n) => Some(*n),
                _ => None,
            })
            .max()
            .unwrap_or(-1);

        // Check for duplicate members
        for (member_name, _, member_span) in members {
            if existing.members.contains_key(member_name) {
                let err = EnumBindingError::DuplicateMember {
                    name: (*member_name).to_string(),
                    span: *member_span,
                };
                self.errors.push(err.clone());
                return Err(err);
            }
        }

        // Compute new member values starting after last value
        let new_members = self.compute_member_values_from(name, members, last_value + 1)?;

        // Merge into existing symbol
        let symbol = self.enums.get_mut(name).unwrap();
        symbol.flags.is_merged = true;
        symbol.declarations.push(span);
        symbol.members.extend(new_members);

        Ok(self.enums.get(name).unwrap())
    }

    /// Compute member values with auto-increment starting from 0
    fn compute_member_values(
        &self,
        parent: &'a str,
        members: &[(&'a str, Option<EnumMemberValue>, (usize, usize))],
    ) -> Result<HashMap<&'a str, EnumMemberSymbol<'a>>, EnumBindingError> {
        self.compute_member_values_from(parent, members, 0)
    }

    /// Compute member values with auto-increment starting from given value
    fn compute_member_values_from(
        &self,
        parent: &'a str,
        members: &[(&'a str, Option<EnumMemberValue>, (usize, usize))],
        start: i64,
    ) -> Result<HashMap<&'a str, EnumMemberSymbol<'a>>, EnumBindingError> {
        let mut result = HashMap::new();
        let mut next_value = start;

        for (name, explicit_value, span) in members {
            let value = if let Some(v) = explicit_value {
                if let EnumMemberValue::Number(n) = v {
                    next_value = n + 1;
                }
                v.clone()
            } else {
                let v = EnumMemberValue::Number(next_value);
                next_value += 1;
                v
            };

            let member_symbol = EnumMemberSymbol {
                name,
                parent,
                value,
                span: *span,
            };

            result.insert(*name, member_symbol);
        }

        Ok(result)
    }

    /// Get an enum symbol by name
    pub fn get_enum(&self, name: &str) -> Option<&EnumSymbol<'a>> {
        self.enums.get(name)
    }

    /// Get an enum member symbol
    pub fn get_enum_member(&self, enum_name: &str, member_name: &str) -> Option<&EnumMemberSymbol<'a>> {
        self.enums.get(enum_name)?.members.get(member_name)
    }

    /// Check if an enum exists
    pub fn has_enum(&self, name: &str) -> bool {
        self.enums.contains_key(name)
    }

    /// Convert bound enum to EnumDeclaration for type checking
    pub fn to_declaration(&self, name: &str) -> Option<EnumDeclaration<'a>> {
        let symbol = self.enums.get(name)?;

        let members: Vec<EnumMember<'a>> = symbol
            .members
            .values()
            .map(|m| EnumMember {
                name: m.name,
                value: m.value.clone(),
                has_initializer: !matches!(m.value, EnumMemberValue::Number(_)),
                span: m.span,
            })
            .collect();

        Some(EnumDeclaration {
            name: symbol.name,
            members,
            is_const: symbol.flags.is_const,
            is_ambient: symbol.flags.is_ambient,
            span: symbol.declarations[0],
        })
    }

    /// Get all bound enums
    pub fn all_enums(&self) -> impl Iterator<Item = &EnumSymbol<'a>> {
        self.enums.values()
    }

    /// Get binding errors
    pub fn errors(&self) -> &[EnumBindingError] {
        &self.errors
    }

    /// Clear errors
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }
}

impl<'a> Default for EnumBinder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluate a constant expression for enum member initializer
pub fn evaluate_constant_expression(expr: &str) -> Option<EnumMemberValue> {
    // Simple evaluation for basic cases
    let trimmed = expr.trim();

    // Try parsing as number
    if let Ok(n) = trimmed.parse::<i64>() {
        return Some(EnumMemberValue::Number(n));
    }

    // Try parsing as hex
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        if let Ok(n) = i64::from_str_radix(&trimmed[2..], 16) {
            return Some(EnumMemberValue::Number(n));
        }
    }

    // Try parsing as binary
    if trimmed.starts_with("0b") || trimmed.starts_with("0B") {
        if let Ok(n) = i64::from_str_radix(&trimmed[2..], 2) {
            return Some(EnumMemberValue::Number(n));
        }
    }

    // Try parsing as string literal
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        let s = &trimmed[1..trimmed.len() - 1];
        return Some(EnumMemberValue::String(s.to_string()));
    }

    // Simple unary operations
    if trimmed.starts_with('-') {
        if let Some(EnumMemberValue::Number(n)) = evaluate_constant_expression(&trimmed[1..]) {
            return Some(EnumMemberValue::Number(-n));
        }
    }

    if trimmed.starts_with('~') {
        if let Some(EnumMemberValue::Number(n)) = evaluate_constant_expression(&trimmed[1..]) {
            return Some(EnumMemberValue::Number(!n));
        }
    }

    // Cannot evaluate - mark as computed
    None
}

/// Check if a string represents a valid enum member name
pub fn is_valid_enum_member_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();

    // First char must be letter, underscore, or $
    if !first.is_alphabetic() && first != '_' && first != '$' {
        return false;
    }

    // Rest can include digits
    for c in chars {
        if !c.is_alphanumeric() && c != '_' && c != '$' {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_simple_enum() {
        let mut binder = EnumBinder::new();

        let result = binder.bind_enum(
            "Color",
            &[
                ("Red", None, (0, 3)),
                ("Green", None, (5, 10)),
                ("Blue", None, (12, 16)),
            ],
            false,
            false,
            (0, 20),
        );

        assert!(result.is_ok());
        let symbol = result.unwrap();
        assert_eq!(symbol.name, "Color");
        assert_eq!(symbol.members.len(), 3);

        // Check auto-increment values
        assert_eq!(
            symbol.members.get("Red").unwrap().value,
            EnumMemberValue::Number(0)
        );
        assert_eq!(
            symbol.members.get("Green").unwrap().value,
            EnumMemberValue::Number(1)
        );
        assert_eq!(
            symbol.members.get("Blue").unwrap().value,
            EnumMemberValue::Number(2)
        );
    }

    #[test]
    fn test_bind_enum_with_explicit_values() {
        let mut binder = EnumBinder::new();

        let result = binder.bind_enum(
            "Status",
            &[
                ("Pending", Some(EnumMemberValue::Number(10)), (0, 7)),
                ("Active", None, (10, 16)),  // Should be 11
                ("Done", Some(EnumMemberValue::Number(100)), (20, 24)),
                ("Archived", None, (28, 36)),  // Should be 101
            ],
            false,
            false,
            (0, 40),
        );

        assert!(result.is_ok());
        let symbol = result.unwrap();

        assert_eq!(
            symbol.members.get("Pending").unwrap().value,
            EnumMemberValue::Number(10)
        );
        assert_eq!(
            symbol.members.get("Active").unwrap().value,
            EnumMemberValue::Number(11)
        );
        assert_eq!(
            symbol.members.get("Done").unwrap().value,
            EnumMemberValue::Number(100)
        );
        assert_eq!(
            symbol.members.get("Archived").unwrap().value,
            EnumMemberValue::Number(101)
        );
    }

    #[test]
    fn test_bind_const_enum() {
        let mut binder = EnumBinder::new();

        let result = binder.bind_enum(
            "ConstEnum",
            &[("A", None, (0, 1))],
            true,  // is_const
            false,
            (0, 10),
        );

        assert!(result.is_ok());
        let symbol = result.unwrap();
        assert!(symbol.flags.is_const);
    }

    #[test]
    fn test_bind_ambient_enum() {
        let mut binder = EnumBinder::new();

        let result = binder.bind_enum(
            "AmbientEnum",
            &[("X", Some(EnumMemberValue::Number(0)), (0, 1))],
            false,
            true,  // is_ambient
            (0, 15),
        );

        assert!(result.is_ok());
        let symbol = result.unwrap();
        assert!(symbol.flags.is_ambient);
    }

    #[test]
    fn test_enum_merging() {
        let mut binder = EnumBinder::new();

        // First declaration
        binder.bind_enum(
            "Merged",
            &[
                ("A", None, (0, 1)),
                ("B", None, (3, 4)),
            ],
            false,
            false,
            (0, 10),
        ).unwrap();

        // Second declaration (merge)
        let result = binder.bind_enum(
            "Merged",
            &[
                ("C", None, (15, 16)),
                ("D", None, (18, 19)),
            ],
            false,
            false,
            (12, 25),
        );

        assert!(result.is_ok());
        let symbol = result.unwrap();
        assert!(symbol.flags.is_merged);
        assert_eq!(symbol.members.len(), 4);
        assert_eq!(symbol.declarations.len(), 2);

        // C should start after B (value 1)
        assert_eq!(
            symbol.members.get("C").unwrap().value,
            EnumMemberValue::Number(2)
        );
    }

    #[test]
    fn test_const_merge_conflict() {
        let mut binder = EnumBinder::new();

        // Non-const enum
        binder.bind_enum(
            "Test",
            &[("A", None, (0, 1))],
            false,  // non-const
            false,
            (0, 10),
        ).unwrap();

        // Try to merge with const enum - should fail
        let result = binder.bind_enum(
            "Test",
            &[("B", None, (15, 16))],
            true,  // const - conflict!
            false,
            (12, 25),
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EnumBindingError::ConstMergeConflict { .. }
        ));
    }

    #[test]
    fn test_duplicate_member_error() {
        let mut binder = EnumBinder::new();

        let result = binder.bind_enum(
            "Test",
            &[
                ("A", None, (0, 1)),
                ("A", None, (3, 4)),  // Duplicate!
            ],
            false,
            false,
            (0, 10),
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EnumBindingError::DuplicateMember { .. }
        ));
    }

    #[test]
    fn test_evaluate_constant_expression() {
        // Decimal
        assert_eq!(
            evaluate_constant_expression("42"),
            Some(EnumMemberValue::Number(42))
        );

        // Negative
        assert_eq!(
            evaluate_constant_expression("-5"),
            Some(EnumMemberValue::Number(-5))
        );

        // Hex
        assert_eq!(
            evaluate_constant_expression("0xFF"),
            Some(EnumMemberValue::Number(255))
        );

        // Binary
        assert_eq!(
            evaluate_constant_expression("0b1010"),
            Some(EnumMemberValue::Number(10))
        );

        // String
        assert_eq!(
            evaluate_constant_expression("\"hello\""),
            Some(EnumMemberValue::String("hello".to_string()))
        );

        // Single quoted string
        assert_eq!(
            evaluate_constant_expression("'world'"),
            Some(EnumMemberValue::String("world".to_string()))
        );

        // Bitwise NOT
        assert_eq!(
            evaluate_constant_expression("~0"),
            Some(EnumMemberValue::Number(-1))
        );

        // Unknown expression
        assert_eq!(evaluate_constant_expression("foo + bar"), None);
    }

    #[test]
    fn test_is_valid_enum_member_name() {
        assert!(is_valid_enum_member_name("Valid"));
        assert!(is_valid_enum_member_name("_private"));
        assert!(is_valid_enum_member_name("$special"));
        assert!(is_valid_enum_member_name("Name123"));

        assert!(!is_valid_enum_member_name(""));
        assert!(!is_valid_enum_member_name("123"));
        assert!(!is_valid_enum_member_name("with-dash"));
        assert!(!is_valid_enum_member_name("with space"));
    }

    #[test]
    fn test_to_declaration() {
        let mut binder = EnumBinder::new();

        binder.bind_enum(
            "Test",
            &[
                ("A", None, (0, 1)),
                ("B", None, (3, 4)),
            ],
            true,
            false,
            (0, 10),
        ).unwrap();

        let decl = binder.to_declaration("Test");
        assert!(decl.is_some());
        let decl = decl.unwrap();
        assert_eq!(decl.name, "Test");
        assert!(decl.is_const);
        assert!(!decl.is_ambient);
    }

    #[test]
    fn test_get_enum_member() {
        let mut binder = EnumBinder::new();

        binder.bind_enum(
            "Color",
            &[
                ("Red", Some(EnumMemberValue::Number(0)), (0, 3)),
            ],
            false,
            false,
            (0, 10),
        ).unwrap();

        let member = binder.get_enum_member("Color", "Red");
        assert!(member.is_some());
        assert_eq!(member.unwrap().name, "Red");
        assert_eq!(member.unwrap().parent, "Color");

        let nonexistent = binder.get_enum_member("Color", "Blue");
        assert!(nonexistent.is_none());
    }
}
