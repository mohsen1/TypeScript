//! Exhaustiveness Checking
//!
//! Handles exhaustiveness analysis for switch statements and pattern matching:
//! - Check if all union members are handled
//! - Support for assertNever pattern
//! - Never type for exhaustiveness checking
//! - Missing case detection

use crate::types::intersection::Type;
use crate::checker::union::{UnionChecker, DiscriminatedUnionInfo, simplify_union};
use std::collections::HashSet;

/// Result of exhaustiveness check
#[derive(Debug, Clone)]
pub struct ExhaustivenessResult {
    /// Whether all cases are covered
    pub is_exhaustive: bool,
    /// Types that are not covered
    pub uncovered_types: Vec<Type>,
    /// The narrowed type after all covered cases (should be never if exhaustive)
    pub remaining_type: Type,
    /// Warnings (e.g., unreachable cases)
    pub warnings: Vec<ExhaustivenessWarning>,
}

/// Warning from exhaustiveness check
#[derive(Debug, Clone)]
pub enum ExhaustivenessWarning {
    /// A case is unreachable (already covered by previous case)
    UnreachableCase { case_type: Type },
    /// Duplicate case
    DuplicateCase { case_type: Type },
    /// Default case makes other cases unreachable
    DefaultMakesUnreachable { unreachable_types: Vec<Type> },
}

/// A case in a switch statement
#[derive(Debug, Clone)]
pub struct SwitchCase {
    /// The type being checked (e.g., the literal value)
    pub case_type: Type,
    /// Whether this is the default case
    pub is_default: bool,
}

/// Exhaustiveness checker
pub struct ExhaustivenessChecker {
    union_checker: UnionChecker,
}

impl ExhaustivenessChecker {
    pub fn new() -> Self {
        ExhaustivenessChecker {
            union_checker: UnionChecker::new(),
        }
    }

    /// Check exhaustiveness for a switch statement on a discriminated union
    pub fn check_switch_exhaustiveness(
        &self,
        union_type: &Type,
        discriminant_property: &str,
        cases: &[SwitchCase],
    ) -> ExhaustivenessResult {
        let info = self.union_checker.analyze_discriminated_union(union_type);

        if !info.is_discriminated {
            return self.check_non_discriminated_switch(union_type, cases);
        }

        self.check_discriminated_switch(&info, discriminant_property, cases)
    }

    /// Check exhaustiveness for a discriminated union switch
    fn check_discriminated_switch(
        &self,
        info: &DiscriminatedUnionInfo,
        discriminant_property: &str,
        cases: &[SwitchCase],
    ) -> ExhaustivenessResult {
        let mut warnings = Vec::new();
        let mut covered_indices: HashSet<usize> = HashSet::new();
        let mut has_default = false;
        let mut seen_values: HashSet<String> = HashSet::new();

        // Find the discriminant
        let discriminant = info.discriminants
            .iter()
            .find(|d| d.name == discriminant_property);

        let discriminant = match discriminant {
            Some(d) => d,
            None => {
                // Not a valid discriminant - can't check exhaustiveness
                return ExhaustivenessResult {
                    is_exhaustive: false,
                    uncovered_types: info.members.clone(),
                    remaining_type: simplify_union(info.members.clone()),
                    warnings,
                };
            }
        };

        // Process each case
        for case in cases {
            if case.is_default {
                has_default = true;
                // Check if default makes any remaining cases unreachable
                let remaining: Vec<Type> = info.members
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| !covered_indices.contains(idx))
                    .map(|(_, t)| t.clone())
                    .collect();

                if remaining.is_empty() {
                    warnings.push(ExhaustivenessWarning::DefaultMakesUnreachable {
                        unreachable_types: vec![],
                    });
                }
                // Default covers all remaining cases
                for i in 0..info.members.len() {
                    covered_indices.insert(i);
                }
                continue;
            }

            // Get the literal key for this case
            if let Some(key) = get_literal_key(&case.case_type) {
                // Check for duplicate case
                if seen_values.contains(&key) {
                    warnings.push(ExhaustivenessWarning::DuplicateCase {
                        case_type: case.case_type.clone(),
                    });
                    continue;
                }
                seen_values.insert(key.clone());

                // Find which member this case covers
                if let Some(&member_idx) = discriminant.value_to_member.get(&key) {
                    if covered_indices.contains(&member_idx) {
                        warnings.push(ExhaustivenessWarning::UnreachableCase {
                            case_type: case.case_type.clone(),
                        });
                    } else {
                        covered_indices.insert(member_idx);
                    }
                }
            }
        }

        // Find uncovered types
        let uncovered_types: Vec<Type> = info.members
            .iter()
            .enumerate()
            .filter(|(idx, _)| !covered_indices.contains(idx))
            .map(|(_, t)| t.clone())
            .collect();

        let remaining_type = simplify_union(uncovered_types.clone());
        let is_exhaustive = uncovered_types.is_empty() || has_default;

        ExhaustivenessResult {
            is_exhaustive,
            uncovered_types,
            remaining_type,
            warnings,
        }
    }

    /// Check exhaustiveness for a non-discriminated union (typeof checks, etc.)
    fn check_non_discriminated_switch(
        &self,
        union_type: &Type,
        cases: &[SwitchCase],
    ) -> ExhaustivenessResult {
        let members = match union_type {
            Type::Union(types) => types.clone(),
            _ => vec![union_type.clone()],
        };

        let mut warnings = Vec::new();
        let mut covered_types: HashSet<String> = HashSet::new();
        let mut has_default = false;

        for case in cases {
            if case.is_default {
                has_default = true;
                continue;
            }

            let key = format!("{:?}", case.case_type);
            if covered_types.contains(&key) {
                warnings.push(ExhaustivenessWarning::DuplicateCase {
                    case_type: case.case_type.clone(),
                });
            } else {
                covered_types.insert(key);
            }
        }

        // Find uncovered types
        let uncovered_types: Vec<Type> = members
            .iter()
            .filter(|t| !covered_types.contains(&format!("{:?}", t)))
            .cloned()
            .collect();

        let remaining_type = simplify_union(uncovered_types.clone());
        let is_exhaustive = uncovered_types.is_empty() || has_default;

        ExhaustivenessResult {
            is_exhaustive,
            uncovered_types,
            remaining_type,
            warnings,
        }
    }

    /// Check if a type can be used with assertNever pattern
    /// Returns Ok(()) if the type is `never`, Err with the actual type otherwise
    pub fn check_assert_never(&self, ty: &Type) -> Result<(), Type> {
        if ty.is_never() {
            Ok(())
        } else {
            Err(ty.clone())
        }
    }

    /// Narrow a type after handling specific cases
    /// Returns the remaining type after excluding handled cases
    pub fn get_remaining_type(
        &self,
        union_type: &Type,
        handled_types: &[Type],
    ) -> Type {
        let members = match union_type {
            Type::Union(types) => types.clone(),
            _ => vec![union_type.clone()],
        };

        let handled_keys: HashSet<String> = handled_types
            .iter()
            .map(|t| format!("{:?}", t))
            .collect();

        let remaining: Vec<Type> = members
            .into_iter()
            .filter(|t| !handled_keys.contains(&format!("{:?}", t)))
            .collect();

        simplify_union(remaining)
    }

    /// Check exhaustiveness for if-else chains with type guards
    pub fn check_if_chain_exhaustiveness(
        &self,
        union_type: &Type,
        narrowed_types: &[Type],
    ) -> ExhaustivenessResult {
        let members = match union_type {
            Type::Union(types) => types.clone(),
            _ => vec![union_type.clone()],
        };

        let warnings = Vec::new();
        let mut covered_types: HashSet<String> = HashSet::new();

        for narrowed in narrowed_types {
            // Find which members are covered by this narrowed type
            for member in &members {
                if types_overlap(narrowed, member) {
                    let key = format!("{:?}", member);
                    covered_types.insert(key);
                }
            }
        }

        // Find uncovered types
        let uncovered_types: Vec<Type> = members
            .iter()
            .filter(|t| !covered_types.contains(&format!("{:?}", t)))
            .cloned()
            .collect();

        let remaining_type = simplify_union(uncovered_types.clone());
        let is_exhaustive = uncovered_types.is_empty();

        ExhaustivenessResult {
            is_exhaustive,
            uncovered_types,
            remaining_type,
            warnings,
        }
    }

    /// Get suggestions for missing cases
    pub fn get_missing_case_suggestions(
        &self,
        result: &ExhaustivenessResult,
        discriminant_property: Option<&str>,
    ) -> Vec<String> {
        result.uncovered_types
            .iter()
            .map(|ty| {
                if let (Some(prop), Type::Object(obj)) = (discriminant_property, ty) {
                    if let Some(disc_prop) = obj.properties.get(prop) {
                        format!("case {}: ...", format_type(&disc_prop.prop_type))
                    } else {
                        format!("Handle type: {}", format_type(ty))
                    }
                } else {
                    format!("Handle type: {}", format_type(ty))
                }
            })
            .collect()
    }
}

impl Default for ExhaustivenessChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Get a unique string key for a literal type (internal use)
fn get_literal_key(ty: &Type) -> Option<String> {
    match ty {
        Type::StringLiteral(s) => Some(format!("string:{}", s)),
        Type::NumberLiteral(n) => Some(format!("number:{}", n)),
        Type::BooleanLiteral(b) => Some(format!("boolean:{}", b)),
        Type::Null => Some("null".to_string()),
        Type::Undefined => Some("undefined".to_string()),
        _ => None,
    }
}

/// Check if two types overlap (could both match the same value)
fn types_overlap(a: &Type, b: &Type) -> bool {
    if a == b {
        return true;
    }

    match (a, b) {
        // Literal overlaps with its base type
        (Type::StringLiteral(_), Type::String) => true,
        (Type::String, Type::StringLiteral(_)) => true,
        (Type::NumberLiteral(_), Type::Number) => true,
        (Type::Number, Type::NumberLiteral(_)) => true,
        (Type::BooleanLiteral(_), Type::Boolean) => true,
        (Type::Boolean, Type::BooleanLiteral(_)) => true,

        // Any overlaps with everything
        (Type::Any, _) | (_, Type::Any) => true,

        // Unknown overlaps with everything
        (Type::Unknown, _) | (_, Type::Unknown) => true,

        _ => false,
    }
}

/// Format a type for display
fn format_type(ty: &Type) -> String {
    match ty {
        Type::StringLiteral(s) => format!("\"{}\"", s),
        Type::NumberLiteral(n) => format!("{}", n),
        Type::BooleanLiteral(b) => format!("{}", b),
        Type::Null => "null".to_string(),
        Type::Undefined => "undefined".to_string(),
        Type::Never => "never".to_string(),
        Type::String => "string".to_string(),
        Type::Number => "number".to_string(),
        Type::Boolean => "boolean".to_string(),
        Type::Object(_) => "object".to_string(),
        Type::Array(_) => "array".to_string(),
        Type::Union(types) => {
            types.iter()
                .map(format_type)
                .collect::<Vec<_>>()
                .join(" | ")
        }
        _ => format!("{:?}", ty),
    }
}

/// Helper function to create an assertNever function type check
pub fn assert_never_check(value: &Type) -> Result<(), String> {
    if value.is_never() {
        Ok(())
    } else {
        Err(format!(
            "Argument of type '{}' is not assignable to parameter of type 'never'",
            format_type(value)
        ))
    }
}

/// Calculate the narrowed type after a switch case
pub fn narrow_after_case(
    union_type: &Type,
    discriminant_property: &str,
    handled_value: &Type,
) -> Type {
    let checker = UnionChecker::new();
    let result = checker.narrow_by_discriminant(union_type, discriminant_property, handled_value);

    // Return the complement - types that DON'T match the handled value
    simplify_union(result.eliminated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::intersection::{Property, ObjectType};
    use std::collections::HashMap;

    fn make_object(props: Vec<(&str, Type)>) -> Type {
        let mut properties = HashMap::new();
        for (name, ty) in props {
            properties.insert(name.to_string(), Property::new(name, ty));
        }
        Type::Object(ObjectType::with_properties(properties))
    }

    fn make_discriminated_union() -> Type {
        let a = make_object(vec![
            ("kind", Type::StringLiteral("a".to_string())),
            ("value", Type::Number),
        ]);
        let b = make_object(vec![
            ("kind", Type::StringLiteral("b".to_string())),
            ("name", Type::String),
        ]);
        let c = make_object(vec![
            ("kind", Type::StringLiteral("c".to_string())),
            ("flag", Type::Boolean),
        ]);
        Type::Union(vec![a, b, c])
    }

    #[test]
    fn test_exhaustive_switch() {
        let checker = ExhaustivenessChecker::new();
        let union = make_discriminated_union();

        let cases = vec![
            SwitchCase { case_type: Type::StringLiteral("a".to_string()), is_default: false },
            SwitchCase { case_type: Type::StringLiteral("b".to_string()), is_default: false },
            SwitchCase { case_type: Type::StringLiteral("c".to_string()), is_default: false },
        ];

        let result = checker.check_switch_exhaustiveness(&union, "kind", &cases);

        assert!(result.is_exhaustive);
        assert!(result.uncovered_types.is_empty());
        assert!(result.remaining_type.is_never());
    }

    #[test]
    fn test_non_exhaustive_switch() {
        let checker = ExhaustivenessChecker::new();
        let union = make_discriminated_union();

        let cases = vec![
            SwitchCase { case_type: Type::StringLiteral("a".to_string()), is_default: false },
            SwitchCase { case_type: Type::StringLiteral("b".to_string()), is_default: false },
            // Missing case "c"
        ];

        let result = checker.check_switch_exhaustiveness(&union, "kind", &cases);

        assert!(!result.is_exhaustive);
        assert_eq!(result.uncovered_types.len(), 1);
        assert!(!result.remaining_type.is_never());
    }

    #[test]
    fn test_switch_with_default() {
        let checker = ExhaustivenessChecker::new();
        let union = make_discriminated_union();

        let cases = vec![
            SwitchCase { case_type: Type::StringLiteral("a".to_string()), is_default: false },
            SwitchCase { case_type: Type::Never, is_default: true }, // default case
        ];

        let result = checker.check_switch_exhaustiveness(&union, "kind", &cases);

        // With default, it's considered exhaustive
        assert!(result.is_exhaustive);
    }

    #[test]
    fn test_duplicate_case_warning() {
        let checker = ExhaustivenessChecker::new();
        let union = make_discriminated_union();

        let cases = vec![
            SwitchCase { case_type: Type::StringLiteral("a".to_string()), is_default: false },
            SwitchCase { case_type: Type::StringLiteral("a".to_string()), is_default: false }, // duplicate
            SwitchCase { case_type: Type::StringLiteral("b".to_string()), is_default: false },
            SwitchCase { case_type: Type::StringLiteral("c".to_string()), is_default: false },
        ];

        let result = checker.check_switch_exhaustiveness(&union, "kind", &cases);

        assert!(result.warnings.iter().any(|w| matches!(w, ExhaustivenessWarning::DuplicateCase { .. })));
    }

    #[test]
    fn test_assert_never_pass() {
        let checker = ExhaustivenessChecker::new();
        assert!(checker.check_assert_never(&Type::Never).is_ok());
    }

    #[test]
    fn test_assert_never_fail() {
        let checker = ExhaustivenessChecker::new();
        let result = checker.check_assert_never(&Type::String);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_remaining_type() {
        let checker = ExhaustivenessChecker::new();
        let union = Type::Union(vec![Type::String, Type::Number, Type::Boolean]);

        let remaining = checker.get_remaining_type(&union, &[Type::String, Type::Number]);

        assert_eq!(remaining, Type::Boolean);
    }

    #[test]
    fn test_get_remaining_type_all_handled() {
        let checker = ExhaustivenessChecker::new();
        let union = Type::Union(vec![Type::String, Type::Number]);

        let remaining = checker.get_remaining_type(&union, &[Type::String, Type::Number]);

        assert!(remaining.is_never());
    }

    #[test]
    fn test_assert_never_helper() {
        assert!(assert_never_check(&Type::Never).is_ok());
        assert!(assert_never_check(&Type::String).is_err());
    }

    #[test]
    fn test_missing_case_suggestions() {
        let checker = ExhaustivenessChecker::new();
        let union = make_discriminated_union();

        let cases = vec![
            SwitchCase { case_type: Type::StringLiteral("a".to_string()), is_default: false },
        ];

        let result = checker.check_switch_exhaustiveness(&union, "kind", &cases);
        let suggestions = checker.get_missing_case_suggestions(&result, Some("kind"));

        assert_eq!(suggestions.len(), 2);
        assert!(suggestions.iter().any(|s| s.contains("\"b\"")));
        assert!(suggestions.iter().any(|s| s.contains("\"c\"")));
    }

    #[test]
    fn test_if_chain_exhaustiveness() {
        let checker = ExhaustivenessChecker::new();
        let union = Type::Union(vec![Type::String, Type::Number, Type::Boolean]);

        let result = checker.check_if_chain_exhaustiveness(
            &union,
            &[Type::String, Type::Number, Type::Boolean],
        );

        assert!(result.is_exhaustive);
    }

    #[test]
    fn test_narrow_after_case() {
        let union = make_discriminated_union();
        let remaining = narrow_after_case(
            &union,
            "kind",
            &Type::StringLiteral("a".to_string()),
        );

        // Should have "b" and "c" remaining
        if let Type::Union(types) = remaining {
            assert_eq!(types.len(), 2);
        } else {
            panic!("Expected union type");
        }
    }

    #[test]
    fn test_types_overlap() {
        assert!(types_overlap(&Type::String, &Type::String));
        assert!(types_overlap(&Type::StringLiteral("hi".to_string()), &Type::String));
        assert!(!types_overlap(&Type::String, &Type::Number));
    }

    #[test]
    fn test_format_type() {
        assert_eq!(format_type(&Type::String), "string");
        assert_eq!(format_type(&Type::StringLiteral("test".to_string())), "\"test\"");
        assert_eq!(format_type(&Type::NumberLiteral(42.0)), "42");
        assert_eq!(format_type(&Type::Never), "never");
    }
}
