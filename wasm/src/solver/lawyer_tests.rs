//! Tests for the Lawyer layer (Any propagation rules).

use super::*;
use crate::TypeInterner;
use crate::interner::Atom;
use crate::solver::types::*;

/// Helper function to create an object type with properties
fn make_test_object(interner: &TypeInterner, props: Vec<(Atom, TypeId)>) -> TypeId {
    let property_infos: Vec<PropertyInfo> = props
        .into_iter()
        .map(|(name, type_id)| PropertyInfo {
            name,
            type_id,
            write_type: type_id,
            optional: false,
            readonly: false,
            is_method: false,
        })
        .collect();

    interner.object(property_infos)
}

#[test]
fn test_any_propagation_rules_default() {
    let interner = TypeInterner::new();
    let rules = AnyPropagationRules::new();

    // Default: allow suppression is true
    assert!(rules.allow_any_suppression);
}

#[test]
fn test_any_propagation_rules_strict() {
    let rules = AnyPropagationRules::strict();

    // Strict: allow suppression is false
    assert!(!rules.allow_any_suppression);
}

#[test]
fn test_any_to_any_allows_suppression() {
    let interner = TypeInterner::new();
    let rules = AnyPropagationRules::new();

    // any to any - always allows suppression
    assert!(rules.is_any_allowed_to_suppress(TypeId::ANY, TypeId::ANY, &interner));
}

#[test]
fn test_any_to_non_any_allows_suppression_by_default() {
    let interner = TypeInterner::new();
    let rules = AnyPropagationRules::new();

    // any to string - allows suppression by default
    assert!(rules.is_any_allowed_to_suppress(TypeId::ANY, TypeId::STRING, &interner));

    // string to any - allows suppression by default
    assert!(rules.is_any_allowed_to_suppress(TypeId::STRING, TypeId::ANY, &interner));
}

#[test]
fn test_strict_mode_disallows_suppression() {
    let interner = TypeInterner::new();
    let rules = AnyPropagationRules::strict();

    // In strict mode, any does not suppress
    assert!(!rules.is_any_allowed_to_suppress(TypeId::ANY, TypeId::STRING, &interner));
    assert!(!rules.is_any_allowed_to_suppress(TypeId::STRING, TypeId::ANY, &interner));
}

#[test]
fn test_non_any_types_return_none() {
    let interner = TypeInterner::new();
    let rules = AnyPropagationRules::new();

    // Neither type is any - should return None (delegate to structural checker)
    assert!(
        rules
            .check_any_propagation(TypeId::STRING, TypeId::NUMBER, &interner)
            .is_none()
    );
}

#[test]
fn test_any_with_object_properties() {
    let interner = TypeInterner::new();
    let rules = AnyPropagationRules::new();

    // Create an object with properties
    let name = interner.intern_string("name");
    let obj_type = make_test_object(&interner, vec![(name, TypeId::STRING)]);

    // any to object with properties - by default allows suppression
    // (this is the current TS behavior)
    assert!(rules.is_any_allowed_to_suppress(TypeId::ANY, obj_type, &interner));
}

#[test]
fn test_any_with_array() {
    let interner = TypeInterner::new();
    let rules = AnyPropagationRules::new();

    // Create an array type
    let array_type = interner.array(TypeId::STRING);

    // any to array - by default allows suppression
    assert!(rules.is_any_allowed_to_suppress(TypeId::ANY, array_type, &interner));
}

#[test]
fn test_check_any_propagation_with_any() {
    let interner = TypeInterner::new();
    let rules = AnyPropagationRules::new();

    // any to any - returns Some(true)
    assert_eq!(
        rules.check_any_propagation(TypeId::ANY, TypeId::ANY, &interner),
        Some(true)
    );

    // string to any - returns Some(true)
    assert_eq!(
        rules.check_any_propagation(TypeId::STRING, TypeId::ANY, &interner),
        Some(true)
    );

    // any to string - returns Some(true)
    assert_eq!(
        rules.check_any_propagation(TypeId::ANY, TypeId::STRING, &interner),
        Some(true)
    );
}

#[test]
fn test_check_any_propagation_without_any() {
    let interner = TypeInterner::new();
    let rules = AnyPropagationRules::new();

    // string to number - neither is any, returns None
    assert_eq!(
        rules.check_any_propagation(TypeId::STRING, TypeId::NUMBER, &interner),
        None
    );
}

#[test]
fn test_set_allow_any_suppression() {
    let mut rules = AnyPropagationRules::new();

    // Default is true
    assert!(rules.allow_any_suppression);

    // Set to false
    rules.set_allow_any_suppression(false);
    assert!(!rules.allow_any_suppression);

    // Set back to true
    rules.set_allow_any_suppression(true);
    assert!(rules.allow_any_suppression);
}

#[test]
fn test_default_trait() {
    let rules = AnyPropagationRules::default();

    // Default should match new()
    assert!(rules.allow_any_suppression);
}

#[test]
fn test_any_with_function() {
    let interner = TypeInterner::new();
    let rules = AnyPropagationRules::new();

    // Create a function type
    let function_shape = FunctionShape {
        type_params: Vec::new(),
        params: vec![ParamInfo {
            name: None,
            type_id: TypeId::STRING,
            optional: false,
            rest: false,
        }],
        this_type: None,
        return_type: TypeId::NUMBER,
        type_predicate: None,
        is_constructor: false,
        is_method: false,
    };
    let function_type = interner.function(function_shape);

    // any to function - has structure, check behavior
    let result = rules.is_any_allowed_to_suppress(TypeId::ANY, function_type, &interner);
    // By default, allows suppression (legacy TS behavior)
    assert!(result);
}

#[test]
fn test_any_with_tuple() {
    let interner = TypeInterner::new();
    let rules = AnyPropagationRules::new();

    // Create a tuple type
    let tuple_elements = vec![
        TupleElement {
            type_id: TypeId::STRING,
            name: None,
            optional: false,
            rest: false,
        },
        TupleElement {
            type_id: TypeId::NUMBER,
            name: None,
            optional: false,
            rest: false,
        },
    ];
    let tuple_type = interner.tuple(tuple_elements);

    // any to tuple - has structure, check behavior
    let result = rules.is_any_allowed_to_suppress(TypeId::ANY, tuple_type, &interner);
    // By default, allows suppression (legacy TS behavior)
    assert!(result);
}

#[test]
fn test_has_structural_mismatch_with_primitives() {
    let interner = TypeInterner::new();
    let rules = AnyPropagationRules::new();

    // Primitives don't have "structure" to mismatch
    // any to primitive - allows suppression
    assert!(rules.is_any_allowed_to_suppress(TypeId::ANY, TypeId::STRING, &interner));
    assert!(rules.is_any_allowed_to_suppress(TypeId::ANY, TypeId::NUMBER, &interner));
    assert!(rules.is_any_allowed_to_suppress(TypeId::ANY, TypeId::BOOLEAN, &interner));
}

#[test]
fn test_strict_mode_with_objects() {
    let interner = TypeInterner::new();
    let rules = AnyPropagationRules::strict();

    // Create an object with properties
    let name = interner.intern_string("name");
    let obj_type = make_test_object(&interner, vec![(name, TypeId::STRING)]);

    // In strict mode, even with objects, any should not suppress
    // This means we should delegate to structural checker
    assert!(!rules.is_any_allowed_to_suppress(TypeId::ANY, obj_type, &interner));
    assert_eq!(
        rules.check_any_propagation(TypeId::ANY, obj_type, &interner),
        None
    );
}

// =============================================================================
// FreshnessTracker Tests
// =============================================================================

#[test]
fn test_freshness_tracker_new() {
    let tracker = FreshnessTracker::new();
    assert!(!tracker.is_fresh(TypeId::STRING));
}

#[test]
fn test_freshness_tracker_mark_fresh() {
    let mut tracker = FreshnessTracker::new();
    let type_id = TypeId(100);
    assert!(!tracker.is_fresh(type_id));
    tracker.mark_fresh(type_id);
    assert!(tracker.is_fresh(type_id));
}

#[test]
fn test_freshness_tracker_remove_freshness() {
    let mut tracker = FreshnessTracker::new();
    let type_id = TypeId(100);
    tracker.mark_fresh(type_id);
    assert!(tracker.is_fresh(type_id));
    tracker.remove_freshness(type_id);
    assert!(!tracker.is_fresh(type_id));
}

#[test]
fn test_freshness_tracker_clear() {
    let mut tracker = FreshnessTracker::new();
    let type_a = TypeId(100);
    let type_b = TypeId(200);
    tracker.mark_fresh(type_a);
    tracker.mark_fresh(type_b);
    assert!(tracker.is_fresh(type_a));
    assert!(tracker.is_fresh(type_b));
    tracker.clear();
    assert!(!tracker.is_fresh(type_a));
    assert!(!tracker.is_fresh(type_b));
}

#[test]
fn test_freshness_tracker_should_check_excess_properties() {
    let mut tracker = FreshnessTracker::new();
    let fresh_type = TypeId(100);
    let non_fresh_type = TypeId(200);
    tracker.mark_fresh(fresh_type);
    assert!(tracker.should_check_excess_properties(fresh_type));
    assert!(!tracker.should_check_excess_properties(non_fresh_type));
}

#[test]
fn test_freshness_tracker_multiple_types() {
    let mut tracker = FreshnessTracker::new();
    let types: Vec<TypeId> = (100..110).map(TypeId).collect();
    for &t in &types {
        tracker.mark_fresh(t);
    }
    for &t in &types {
        assert!(tracker.is_fresh(t));
    }
    tracker.remove_freshness(types[0]);
    tracker.remove_freshness(types[5]);
    assert!(!tracker.is_fresh(types[0]));
    assert!(tracker.is_fresh(types[1]));
    assert!(!tracker.is_fresh(types[5]));
    assert!(tracker.is_fresh(types[9]));
}

#[test]
fn test_freshness_tracker_default() {
    let tracker = FreshnessTracker::default();
    assert!(!tracker.is_fresh(TypeId(100)));
}

// =============================================================================
// TypeScriptQuirks Tests
// =============================================================================

#[test]
fn test_typescript_quirks_list() {
    let quirks = TypeScriptQuirks::QUIRKS;
    assert!(quirks.len() >= 9, "Should have at least 9 documented quirks");
    let quirk_names: Vec<&str> = quirks.iter().map(|(name, _)| *name).collect();
    assert!(quirk_names.contains(&"any-propagation"));
    assert!(quirk_names.contains(&"function-bivariance"));
    assert!(quirk_names.contains(&"method-bivariance"));
    assert!(quirk_names.contains(&"void-return"));
    assert!(quirk_names.contains(&"weak-types"));
    assert!(quirk_names.contains(&"freshness"));
}

// =============================================================================
// FunctionBivarianceConfig Tests
// =============================================================================

#[test]
fn test_function_bivariance_config_new() {
    let config = FunctionBivarianceConfig::new();
    // Default config matches TypeScript's legacy behavior
    assert!(!config.strict_function_types);
    assert!(config.methods_are_bivariant);
    assert!(config.method_callbacks_are_bivariant);
}

#[test]
fn test_function_bivariance_config_default() {
    let config = FunctionBivarianceConfig::default();
    // Default trait should match new()
    assert!(!config.strict_function_types);
    assert!(config.methods_are_bivariant);
    assert!(config.method_callbacks_are_bivariant);
}

#[test]
fn test_function_bivariance_config_strict() {
    let config = FunctionBivarianceConfig::strict();
    // Strict mode: contravariance for functions, bivariance for methods
    assert!(config.strict_function_types);
    assert!(config.methods_are_bivariant);
    assert!(config.method_callbacks_are_bivariant);
}

#[test]
fn test_function_bivariance_config_fully_sound() {
    let config = FunctionBivarianceConfig::fully_sound();
    // Fully sound: contravariance everywhere
    assert!(config.strict_function_types);
    assert!(!config.methods_are_bivariant);
    assert!(!config.method_callbacks_are_bivariant);
}

#[test]
fn test_should_use_bivariance_legacy_mode() {
    let config = FunctionBivarianceConfig::new();
    // In legacy mode, always use bivariance
    assert!(config.should_use_bivariance(false, false)); // regular function
    assert!(config.should_use_bivariance(true, false));  // method
    assert!(config.should_use_bivariance(false, true));  // callback in method
    assert!(config.should_use_bivariance(true, true));   // method callback in method
}

#[test]
fn test_should_use_bivariance_strict_mode() {
    let config = FunctionBivarianceConfig::strict();
    // In strict mode, regular functions use contravariance
    assert!(!config.should_use_bivariance(false, false)); // regular function - contravariant
    assert!(config.should_use_bivariance(true, false));   // method - bivariant
    assert!(config.should_use_bivariance(false, true));   // callback in method - bivariant
    assert!(config.should_use_bivariance(true, true));    // method callback - bivariant
}

#[test]
fn test_should_use_bivariance_fully_sound() {
    let config = FunctionBivarianceConfig::fully_sound();
    // In fully sound mode, everything uses contravariance
    assert!(!config.should_use_bivariance(false, false)); // regular function
    assert!(!config.should_use_bivariance(true, false));  // method - still contravariant
    assert!(!config.should_use_bivariance(false, true));  // callback - contravariant
    assert!(!config.should_use_bivariance(true, true));   // method callback - contravariant
}

#[test]
fn test_are_parameters_compatible_bivariant() {
    let config = FunctionBivarianceConfig::new(); // Legacy bivariant mode

    // In bivariant mode, either direction works
    // source <: target
    assert!(config.are_parameters_compatible(false, true, false));
    // target <: source (contravariant direction)
    assert!(config.are_parameters_compatible(false, false, true));
    // Both directions
    assert!(config.are_parameters_compatible(false, true, true));
    // Neither direction - incompatible
    assert!(!config.are_parameters_compatible(false, false, false));
}

#[test]
fn test_are_parameters_compatible_contravariant() {
    let config = FunctionBivarianceConfig::strict(); // Strict mode for functions

    // For regular functions in strict mode, only contravariant direction works
    // source <: target is NOT enough
    assert!(!config.are_parameters_compatible(false, true, false));
    // target <: source (contravariant direction) IS what we need
    assert!(config.are_parameters_compatible(false, false, true));
    // Both directions - OK
    assert!(config.are_parameters_compatible(false, true, true));
    // Neither direction - incompatible
    assert!(!config.are_parameters_compatible(false, false, false));

    // For methods, still bivariant
    assert!(config.are_parameters_compatible(true, true, false)); // covariant OK for methods
    assert!(config.are_parameters_compatible(true, false, true)); // contravariant OK for methods
}

#[test]
fn test_describe_mode() {
    let legacy = FunctionBivarianceConfig::new();
    assert_eq!(legacy.describe_mode(), "bivariant (legacy TypeScript)");

    let strict = FunctionBivarianceConfig::strict();
    assert_eq!(strict.describe_mode(), "contravariant for functions, bivariant for methods (strictFunctionTypes)");

    let sound = FunctionBivarianceConfig::fully_sound();
    assert_eq!(sound.describe_mode(), "fully contravariant (sound, non-TypeScript-compatible)");
}

#[test]
fn test_function_bivariance_config_clone() {
    let config = FunctionBivarianceConfig::strict();
    let cloned = config.clone();
    assert_eq!(config.strict_function_types, cloned.strict_function_types);
    assert_eq!(config.methods_are_bivariant, cloned.methods_are_bivariant);
    assert_eq!(config.method_callbacks_are_bivariant, cloned.method_callbacks_are_bivariant);
}

#[test]
fn test_function_bivariance_config_custom() {
    // Test custom configuration: strict functions, but no method bivariance
    let custom = FunctionBivarianceConfig {
        strict_function_types: true,
        methods_are_bivariant: false,
        method_callbacks_are_bivariant: true, // but keep callback bivariance
    };

    // Regular functions: contravariant
    assert!(!custom.should_use_bivariance(false, false));
    // Methods: also contravariant (custom)
    assert!(!custom.should_use_bivariance(true, false));
    // Callbacks in methods: still bivariant
    assert!(custom.should_use_bivariance(false, true));
}

#[test]
fn test_function_bivariance_edge_case_both_flags() {
    // Edge case: is_method AND is_callback_in_method are both true
    // Should still use bivariance if any exception applies
    let strict = FunctionBivarianceConfig::strict();
    assert!(strict.should_use_bivariance(true, true));

    // With method bivariance disabled but callback bivariance enabled
    let custom = FunctionBivarianceConfig {
        strict_function_types: true,
        methods_are_bivariant: false,
        method_callbacks_are_bivariant: true,
    };
    // Even though is_method=true wouldn't grant bivariance,
    // is_callback_in_method=true does
    assert!(custom.should_use_bivariance(true, true));

    // With both disabled
    let sound = FunctionBivarianceConfig::fully_sound();
    assert!(!sound.should_use_bivariance(true, true));
}
