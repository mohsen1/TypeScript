/**
 * Diagnostics System - Error Codes
 *
 * This module defines all TypeScript diagnostic codes organized by category.
 * Each code has a unique number, severity, and message template.
 */

/* @internal */
namespace ts.diagnostics {
    /**
     * Diagnostic code ranges by category:
     * - 1000-1999: Syntax errors
     * - 2000-2999: Semantic errors (binding)
     * - 3000-3999: Semantic errors (checking)
     * - 4000-4999: Declaration emit errors
     * - 5000-5999: Compiler options errors
     * - 6000-6999: Command line errors
     * - 7000-7999: Noisy errors (disabled by default)
     * - 8000-8999: Build mode errors
     * - 9000-9999: Reserved
     */

    // ============================================
    // Syntax Errors (1000-1999)
    // ============================================

    export const Diagnostics_Unterminated_string_literal: DiagnosticTemplate = {
        code: 1002,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Unterminated string literal.",
        category: "error",
    };

    export const Diagnostics_Identifier_expected: DiagnosticTemplate = {
        code: 1003,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Identifier expected.",
        category: "error",
    };

    export const Diagnostics_0_expected: DiagnosticTemplate = {
        code: 1005,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "'{0}' expected.",
        category: "error",
    };

    export const Diagnostics_A_file_cannot_have_a_reference_to_itself: DiagnosticTemplate = {
        code: 1006,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "A file cannot have a reference to itself.",
        category: "error",
    };

    export const Diagnostics_The_parser_expected_to_find_a_0_to_match_the_1_token_here: DiagnosticTemplate = {
        code: 1007,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "The parser expected to find a '{0}' to match the '{1}' token here.",
        category: "error",
    };

    export const Diagnostics_Trailing_comma_not_allowed: DiagnosticTemplate = {
        code: 1009,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Trailing comma not allowed.",
        category: "error",
    };

    export const Diagnostics_Asterisk_Slash_expected: DiagnosticTemplate = {
        code: 1010,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "'*/' expected.",
        category: "error",
    };

    export const Diagnostics_An_element_access_expression_should_take_an_argument: DiagnosticTemplate = {
        code: 1011,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "An element access expression should take an argument.",
        category: "error",
    };

    export const Diagnostics_Unexpected_token: DiagnosticTemplate = {
        code: 1012,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Unexpected token.",
        category: "error",
    };

    export const Diagnostics_A_rest_parameter_or_binding_pattern_may_not_have_a_trailing_comma: DiagnosticTemplate = {
        code: 1013,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "A rest parameter or binding pattern may not have a trailing comma.",
        category: "error",
    };

    export const Diagnostics_A_rest_parameter_must_be_last_in_a_parameter_list: DiagnosticTemplate = {
        code: 1014,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "A rest parameter must be last in a parameter list.",
        category: "error",
    };

    export const Diagnostics_Parameter_cannot_have_question_mark_and_initializer: DiagnosticTemplate = {
        code: 1015,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Parameter cannot have question mark and initializer.",
        category: "error",
    };

    export const Diagnostics_A_required_parameter_cannot_follow_an_optional_parameter: DiagnosticTemplate = {
        code: 1016,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "A required parameter cannot follow an optional parameter.",
        category: "error",
    };

    export const Diagnostics_An_index_signature_cannot_have_a_rest_parameter: DiagnosticTemplate = {
        code: 1017,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "An index signature cannot have a rest parameter.",
        category: "error",
    };

    export const Diagnostics_An_index_signature_parameter_cannot_have_an_accessibility_modifier: DiagnosticTemplate = {
        code: 1018,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "An index signature parameter cannot have an accessibility modifier.",
        category: "error",
    };

    export const Diagnostics_An_index_signature_parameter_cannot_have_a_question_mark: DiagnosticTemplate = {
        code: 1019,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "An index signature parameter cannot have a question mark.",
        category: "error",
    };

    export const Diagnostics_An_index_signature_parameter_cannot_have_an_initializer: DiagnosticTemplate = {
        code: 1020,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "An index signature parameter cannot have an initializer.",
        category: "error",
    };

    export const Diagnostics_An_index_signature_must_have_a_type_annotation: DiagnosticTemplate = {
        code: 1021,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "An index signature must have a type annotation.",
        category: "error",
    };

    export const Diagnostics_An_index_signature_parameter_must_have_a_type_annotation: DiagnosticTemplate = {
        code: 1022,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "An index signature parameter must have a type annotation.",
        category: "error",
    };

    export const Diagnostics_An_index_signature_parameter_type_must_be_string_number_symbol_or_a_template_literal_type: DiagnosticTemplate = {
        code: 1023,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "An index signature parameter type must be 'string', 'number', 'symbol', or a template literal type.",
        category: "error",
    };

    export const Diagnostics_readonly_modifier_can_only_appear_on_a_property_declaration_or_index_signature: DiagnosticTemplate = {
        code: 1024,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "'readonly' modifier can only appear on a property declaration or index signature.",
        category: "error",
    };

    export const Diagnostics_An_index_signature_cannot_have_a_trailing_comma: DiagnosticTemplate = {
        code: 1025,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "An index signature cannot have a trailing comma.",
        category: "error",
    };

    export const Diagnostics_Accessibility_modifier_already_seen: DiagnosticTemplate = {
        code: 1028,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Accessibility modifier already seen.",
        category: "error",
    };

    export const Diagnostics_0_modifier_must_precede_1_modifier: DiagnosticTemplate = {
        code: 1029,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "'{0}' modifier must precede '{1}' modifier.",
        category: "error",
    };

    export const Diagnostics_0_modifier_already_seen: DiagnosticTemplate = {
        code: 1030,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "'{0}' modifier already seen.",
        category: "error",
    };

    export const Diagnostics_0_modifier_cannot_appear_on_class_elements_of_this_kind: DiagnosticTemplate = {
        code: 1031,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "'{0}' modifier cannot appear on class elements of this kind.",
        category: "error",
    };

    export const Diagnostics_super_must_be_followed_by_an_argument_list_or_member_access: DiagnosticTemplate = {
        code: 1034,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "'super' must be followed by an argument list or member access.",
        category: "error",
    };

    export const Diagnostics_Declaration_expected: DiagnosticTemplate = {
        code: 1035,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Declaration expected.",
        category: "error",
    };

    export const Diagnostics_Import_declarations_in_a_namespace_cannot_reference_a_module: DiagnosticTemplate = {
        code: 1038,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Import declarations in a namespace cannot reference a module.",
        category: "error",
    };

    export const Diagnostics_Cannot_use_imports_exports_or_module_augmentations_when_module_is_none: DiagnosticTemplate = {
        code: 1039,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Cannot use imports, exports, or module augmentations when '--module' is 'none'.",
        category: "error",
    };

    export const Diagnostics_Expression_expected: DiagnosticTemplate = {
        code: 1109,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Expression expected.",
        category: "error",
    };

    export const Diagnostics_Type_expected: DiagnosticTemplate = {
        code: 1110,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Type expected.",
        category: "error",
    };

    export const Diagnostics_A_0_modifier_cannot_be_used_with_an_import_declaration: DiagnosticTemplate = {
        code: 1079,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "A '{0}' modifier cannot be used with an import declaration.",
        category: "error",
    };

    export const Diagnostics_Invalid_reference_directive_syntax: DiagnosticTemplate = {
        code: 1084,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Invalid reference directive syntax.",
        category: "error",
    };

    // ============================================
    // Semantic Errors - Binding (2000-2999)
    // ============================================

    export const Diagnostics_Duplicate_identifier_0: DiagnosticTemplate = {
        code: 2300,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Duplicate identifier '{0}'.",
        category: "error",
    };

    export const Diagnostics_Cannot_find_name_0: DiagnosticTemplate = {
        code: 2304,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Cannot find name '{0}'.",
        category: "error",
    };

    export const Diagnostics_Module_0_has_no_exported_member_1: DiagnosticTemplate = {
        code: 2305,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Module '{0}' has no exported member '{1}'.",
        category: "error",
    };

    export const Diagnostics_Cannot_find_module_0: DiagnosticTemplate = {
        code: 2307,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Cannot find module '{0}' or its corresponding type declarations.",
        category: "error",
    };

    export const Diagnostics_Module_0_has_no_default_export: DiagnosticTemplate = {
        code: 2613,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Module '{0}' has no default export.",
        category: "error",
    };

    export const Diagnostics_An_export_assignment_cannot_be_used_in_a_module_with_other_exported_elements: DiagnosticTemplate = {
        code: 2309,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "An export assignment cannot be used in a module with other exported elements.",
        category: "error",
    };

    export const Diagnostics_Type_0_recursively_references_itself_as_a_base_type: DiagnosticTemplate = {
        code: 2310,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Type '{0}' recursively references itself as a base type.",
        category: "error",
    };

    export const Diagnostics_Cannot_redeclare_block_scoped_variable_0: DiagnosticTemplate = {
        code: 2451,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Cannot redeclare block-scoped variable '{0}'.",
        category: "error",
    };

    export const Diagnostics_Variable_0_is_used_before_being_assigned: DiagnosticTemplate = {
        code: 2454,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Variable '{0}' is used before being assigned.",
        category: "error",
    };

    export const Diagnostics_Block_scoped_variable_0_used_before_its_declaration: DiagnosticTemplate = {
        code: 2448,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Block-scoped variable '{0}' used before its declaration.",
        category: "error",
    };

    // ============================================
    // Semantic Errors - Type Checking (2300-2999)
    // ============================================

    export const Diagnostics_Type_0_is_not_assignable_to_type_1: DiagnosticTemplate = {
        code: 2322,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Type '{0}' is not assignable to type '{1}'.",
        category: "error",
    };

    export const Diagnostics_Type_0_is_not_assignable_to_type_1_with_exactOptionalPropertyTypes: DiagnosticTemplate = {
        code: 2375,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Type '{0}' is not assignable to type '{1}' with 'exactOptionalPropertyTypes: true'. Consider adding 'undefined' to the type of the target.",
        category: "error",
    };

    export const Diagnostics_Property_0_is_missing_in_type_1_but_required_in_type_2: DiagnosticTemplate = {
        code: 2741,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Property '{0}' is missing in type '{1}' but required in type '{2}'.",
        category: "error",
    };

    export const Diagnostics_Property_0_does_not_exist_on_type_1: DiagnosticTemplate = {
        code: 2339,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Property '{0}' does not exist on type '{1}'.",
        category: "error",
    };

    export const Diagnostics_Property_0_does_not_exist_on_type_1_Did_you_mean_2: DiagnosticTemplate = {
        code: 2551,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Property '{0}' does not exist on type '{1}'. Did you mean '{2}'?",
        category: "error",
    };

    export const Diagnostics_Argument_of_type_0_is_not_assignable_to_parameter_of_type_1: DiagnosticTemplate = {
        code: 2345,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Argument of type '{0}' is not assignable to parameter of type '{1}'.",
        category: "error",
    };

    export const Diagnostics_Cannot_invoke_an_expression_whose_type_lacks_a_call_signature: DiagnosticTemplate = {
        code: 2349,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "This expression is not callable.",
        category: "error",
    };

    export const Diagnostics_Type_0_has_no_call_signatures: DiagnosticTemplate = {
        code: 2757,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Type '{0}' has no call signatures.",
        category: "error",
    };

    export const Diagnostics_This_expression_is_not_constructable: DiagnosticTemplate = {
        code: 2351,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "This expression is not constructable.",
        category: "error",
    };

    export const Diagnostics_Expected_0_arguments_but_got_1: DiagnosticTemplate = {
        code: 2554,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Expected {0} arguments, but got {1}.",
        category: "error",
    };

    export const Diagnostics_Expected_at_least_0_arguments_but_got_1: DiagnosticTemplate = {
        code: 2555,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Expected at least {0} arguments, but got {1}.",
        category: "error",
    };

    export const Diagnostics_Operator_0_cannot_be_applied_to_types_1_and_2: DiagnosticTemplate = {
        code: 2365,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Operator '{0}' cannot be applied to types '{1}' and '{2}'.",
        category: "error",
    };

    export const Diagnostics_A_function_whose_declared_type_is_neither_void_nor_any_must_return_a_value: DiagnosticTemplate = {
        code: 2355,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "A function whose declared type is neither 'void' nor 'any' must return a value.",
        category: "error",
    };

    export const Diagnostics_Not_all_code_paths_return_a_value: DiagnosticTemplate = {
        code: 7030,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Not all code paths return a value.",
        category: "error",
    };

    export const Diagnostics_Type_0_is_not_an_array_type: DiagnosticTemplate = {
        code: 2461,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Type '{0}' is not an array type.",
        category: "error",
    };

    export const Diagnostics_The_left_hand_side_of_a_for_of_statement_cannot_use_a_type_annotation: DiagnosticTemplate = {
        code: 2483,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "The left-hand side of a 'for...of' statement cannot use a type annotation.",
        category: "error",
    };

    export const Diagnostics_Object_is_possibly_undefined: DiagnosticTemplate = {
        code: 2532,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Object is possibly 'undefined'.",
        category: "error",
    };

    export const Diagnostics_Object_is_possibly_null: DiagnosticTemplate = {
        code: 2531,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Object is possibly 'null'.",
        category: "error",
    };

    export const Diagnostics_Object_is_possibly_null_or_undefined: DiagnosticTemplate = {
        code: 2533,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Object is possibly 'null' or 'undefined'.",
        category: "error",
    };

    export const Diagnostics_Object_is_of_type_unknown: DiagnosticTemplate = {
        code: 2571,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Object is of type 'unknown'.",
        category: "error",
    };

    export const Diagnostics_The_operand_of_a_delete_operator_must_be_a_property_reference: DiagnosticTemplate = {
        code: 2703,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "The operand of a 'delete' operator must be a property reference.",
        category: "error",
    };

    export const Diagnostics_Cannot_use_new_with_an_expression_whose_type_lacks_a_call_or_construct_signature: DiagnosticTemplate = {
        code: 2351,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Cannot use 'new' with an expression whose type lacks a call or construct signature.",
        category: "error",
    };

    export const Diagnostics_Type_0_must_have_a_Symbol_iterator_method_that_returns_an_iterator: DiagnosticTemplate = {
        code: 2488,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Type '{0}' must have a '[Symbol.iterator]()' method that returns an iterator.",
        category: "error",
    };

    export const Diagnostics_An_iterator_must_have_a_next_method: DiagnosticTemplate = {
        code: 2489,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "An iterator must have a 'next()' method.",
        category: "error",
    };

    export const Diagnostics_The_type_returned_by_the_0_method_of_an_iterator_must_have_a_value_property: DiagnosticTemplate = {
        code: 2490,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "The type returned by the '{0}()' method of an iterator must have a 'value' property.",
        category: "error",
    };

    // ============================================
    // Class-related Errors
    // ============================================

    export const Diagnostics_Class_0_incorrectly_extends_base_class_1: DiagnosticTemplate = {
        code: 2415,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Class '{0}' incorrectly extends base class '{1}'.",
        category: "error",
    };

    export const Diagnostics_Class_0_incorrectly_implements_interface_1: DiagnosticTemplate = {
        code: 2420,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Class '{0}' incorrectly implements interface '{1}'.",
        category: "error",
    };

    export const Diagnostics_Property_0_in_type_1_is_not_assignable_to_the_same_property_in_base_type_2: DiagnosticTemplate = {
        code: 2416,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Property '{0}' in type '{1}' is not assignable to the same property in base type '{2}'.",
        category: "error",
    };

    export const Diagnostics_Member_0_implicitly_has_an_1_type: DiagnosticTemplate = {
        code: 7008,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Member '{0}' implicitly has an '{1}' type.",
        category: "error",
    };

    export const Diagnostics_this_implicitly_has_type_any_because_it_does_not_have_a_type_annotation: DiagnosticTemplate = {
        code: 2683,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "'this' implicitly has type 'any' because it does not have a type annotation.",
        category: "error",
    };

    export const Diagnostics_super_can_only_be_referenced_in_a_derived_class: DiagnosticTemplate = {
        code: 2335,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "'super' can only be referenced in a derived class.",
        category: "error",
    };

    export const Diagnostics_Super_calls_are_not_permitted_outside_constructors_or_in_nested_functions_inside_constructors: DiagnosticTemplate = {
        code: 2337,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Super calls are not permitted outside constructors or in nested functions inside constructors.",
        category: "error",
    };

    export const Diagnostics_A_super_call_must_be_the_first_statement_in_the_constructor: DiagnosticTemplate = {
        code: 2376,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "A 'super' call must be the first statement in the constructor when a class contains initialized properties, parameter properties, or private identifiers.",
        category: "error",
    };

    export const Diagnostics_Constructors_for_derived_classes_must_contain_a_super_call: DiagnosticTemplate = {
        code: 2377,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Constructors for derived classes must contain a 'super' call.",
        category: "error",
    };

    export const Diagnostics_Property_0_has_no_initializer_and_is_not_definitely_assigned_in_the_constructor: DiagnosticTemplate = {
        code: 2564,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Property '{0}' has no initializer and is not definitely assigned in the constructor.",
        category: "error",
    };

    // ============================================
    // Generic and Type Parameter Errors
    // ============================================

    export const Diagnostics_Type_0_does_not_satisfy_the_constraint_1: DiagnosticTemplate = {
        code: 2344,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Type '{0}' does not satisfy the constraint '{1}'.",
        category: "error",
    };

    export const Diagnostics_Generic_type_0_requires_1_type_argument_s: DiagnosticTemplate = {
        code: 2314,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Generic type '{0}' requires {1} type argument(s).",
        category: "error",
    };

    export const Diagnostics_Expected_0_type_arguments_but_got_1: DiagnosticTemplate = {
        code: 2558,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Expected {0} type arguments, but got {1}.",
        category: "error",
    };

    export const Diagnostics_Type_parameter_0_has_a_circular_constraint: DiagnosticTemplate = {
        code: 2313,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Type parameter '{0}' has a circular constraint.",
        category: "error",
    };

    // ============================================
    // Async/Await Errors
    // ============================================

    export const Diagnostics_await_expressions_are_only_allowed_within_async_functions_and_at_the_top_levels_of_modules: DiagnosticTemplate = {
        code: 1308,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "'await' expressions are only allowed within async functions and at the top levels of modules.",
        category: "error",
    };

    export const Diagnostics_The_return_type_of_an_async_function_or_method_must_be_the_global_Promise_T_type: DiagnosticTemplate = {
        code: 1064,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "The return type of an async function or method must be the global Promise<T> type.",
        category: "error",
    };

    export const Diagnostics_An_async_function_or_method_in_ES5_SlashES3_requires_the_Promise_constructor: DiagnosticTemplate = {
        code: 2705,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "An async function or method in ES5/ES3 requires the 'Promise' constructor.",
        category: "error",
    };

    export const Diagnostics_await_expressions_cannot_be_used_in_a_parameter_initializer: DiagnosticTemplate = {
        code: 2524,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "'await' expressions cannot be used in a parameter initializer.",
        category: "error",
    };

    // ============================================
    // Strict Mode Errors (7xxx)
    // ============================================

    export const Diagnostics_Parameter_0_implicitly_has_an_1_type: DiagnosticTemplate = {
        code: 7006,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Parameter '{0}' implicitly has an '{1}' type.",
        category: "error",
    };

    export const Diagnostics_Variable_0_implicitly_has_type_1_in_some_locations_where_its_type_cannot_be_determined: DiagnosticTemplate = {
        code: 7034,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Variable '{0}' implicitly has type '{1}' in some locations where its type cannot be determined.",
        category: "error",
    };

    export const Diagnostics_Function_expression_which_lacks_return_type_annotation_implicitly_has_an_0_return_type: DiagnosticTemplate = {
        code: 7010,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Function expression, which lacks return-type annotation, implicitly has an '{0}' return type.",
        category: "error",
    };

    export const Diagnostics_Element_implicitly_has_an_any_type_because_expression_of_type_0_cant_be_used_to_index_type_1: DiagnosticTemplate = {
        code: 7053,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Element implicitly has an 'any' type because expression of type '{0}' can't be used to index type '{1}'.",
        category: "error",
    };

    export const Diagnostics_No_index_signature_with_a_parameter_of_type_0_was_found_on_type_1: DiagnosticTemplate = {
        code: 7054,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "No index signature with a parameter of type '{0}' was found on type '{1}'.",
        category: "error",
    };

    // ============================================
    // JSX Errors
    // ============================================

    export const Diagnostics_Cannot_find_name_0_Did_you_mean_the_JSX_tag_1: DiagnosticTemplate = {
        code: 2689,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Cannot find name '{0}'. Did you mean the JSX closing tag '{1}'?",
        category: "error",
    };

    export const Diagnostics_JSX_element_0_has_no_corresponding_closing_tag: DiagnosticTemplate = {
        code: 17008,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "JSX element '{0}' has no corresponding closing tag.",
        category: "error",
    };

    export const Diagnostics_Property_0_does_not_exist_on_type_1_JSX_element: DiagnosticTemplate = {
        code: 2339,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Property '{0}' does not exist on type '{1}'.",
        category: "error",
    };

    // ============================================
    // Declaration Emit Errors (4xxx)
    // ============================================

    export const Diagnostics_Exported_variable_0_has_or_is_using_name_1_from_external_module_2_but_cannot_be_named: DiagnosticTemplate = {
        code: 4023,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Exported variable '{0}' has or is using name '{1}' from external module '{2}' but cannot be named.",
        category: "error",
    };

    export const Diagnostics_The_inferred_type_of_0_cannot_be_named_without_a_reference_to_1_This_is_likely_not_portable_A_type_annotation_is_necessary: DiagnosticTemplate = {
        code: 2742,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "The inferred type of '{0}' cannot be named without a reference to '{1}'. This is likely not portable. A type annotation is necessary.",
        category: "error",
    };

    export const Diagnostics_Return_type_of_exported_function_has_or_is_using_private_name_0: DiagnosticTemplate = {
        code: 4060,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Return type of exported function has or is using private name '{0}'.",
        category: "error",
    };

    // ============================================
    // Compiler Options Errors (5xxx)
    // ============================================

    export const Diagnostics_Option_0_cannot_be_specified_with_option_1: DiagnosticTemplate = {
        code: 5053,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Option '{0}' cannot be specified with option '{1}'.",
        category: "error",
    };

    export const Diagnostics_Cannot_find_a_tsconfig_json_file_at_the_specified_directory_0: DiagnosticTemplate = {
        code: 5057,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Cannot find a tsconfig.json file at the specified directory: '{0}'.",
        category: "error",
    };

    export const Diagnostics_Unknown_option_excludes_Did_you_mean_exclude: DiagnosticTemplate = {
        code: 5108,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Unknown option 'excludes'. Did you mean 'exclude'?",
        category: "error",
    };

    export const Diagnostics_Option_0_can_only_be_specified_in_tsconfig_json_file_or_set_to_null_on_command_line: DiagnosticTemplate = {
        code: 6230,
        severity: DiagnosticSeverity.Error,
        messageTemplate: "Option '{0}' can only be specified in 'tsconfig.json' file or set to 'null' on command line.",
        category: "error",
    };

    // ============================================
    // Warnings and Suggestions
    // ============================================

    export const Diagnostics_0_is_declared_but_its_value_is_never_read: DiagnosticTemplate = {
        code: 6133,
        severity: DiagnosticSeverity.Warning,
        messageTemplate: "'{0}' is declared but its value is never read.",
        category: "warning",
    };

    export const Diagnostics_0_is_declared_but_never_used: DiagnosticTemplate = {
        code: 6196,
        severity: DiagnosticSeverity.Warning,
        messageTemplate: "'{0}' is declared but never used.",
        category: "warning",
    };

    export const Diagnostics_All_imports_in_import_declaration_are_unused: DiagnosticTemplate = {
        code: 6192,
        severity: DiagnosticSeverity.Warning,
        messageTemplate: "All imports in import declaration are unused.",
        category: "warning",
    };

    export const Diagnostics_Unreachable_code_detected: DiagnosticTemplate = {
        code: 7027,
        severity: DiagnosticSeverity.Warning,
        messageTemplate: "Unreachable code detected.",
        category: "warning",
    };

    export const Diagnostics_Unused_label: DiagnosticTemplate = {
        code: 7028,
        severity: DiagnosticSeverity.Warning,
        messageTemplate: "Unused label.",
        category: "warning",
    };

    export const Diagnostics_Fallthrough_case_in_switch: DiagnosticTemplate = {
        code: 7029,
        severity: DiagnosticSeverity.Warning,
        messageTemplate: "Fallthrough case in switch.",
        category: "warning",
    };

    /**
     * Map of diagnostic code to template for efficient lookup
     */
    export const diagnosticTemplatesByCode: Map<number, DiagnosticTemplate> = new Map([
        [1002, Diagnostics_Unterminated_string_literal],
        [1003, Diagnostics_Identifier_expected],
        [1005, Diagnostics_0_expected],
        [1006, Diagnostics_A_file_cannot_have_a_reference_to_itself],
        [1007, Diagnostics_The_parser_expected_to_find_a_0_to_match_the_1_token_here],
        [1009, Diagnostics_Trailing_comma_not_allowed],
        [1010, Diagnostics_Asterisk_Slash_expected],
        [1011, Diagnostics_An_element_access_expression_should_take_an_argument],
        [1012, Diagnostics_Unexpected_token],
        [2300, Diagnostics_Duplicate_identifier_0],
        [2304, Diagnostics_Cannot_find_name_0],
        [2305, Diagnostics_Module_0_has_no_exported_member_1],
        [2307, Diagnostics_Cannot_find_module_0],
        [2322, Diagnostics_Type_0_is_not_assignable_to_type_1],
        [2339, Diagnostics_Property_0_does_not_exist_on_type_1],
        [2345, Diagnostics_Argument_of_type_0_is_not_assignable_to_parameter_of_type_1],
        [2554, Diagnostics_Expected_0_arguments_but_got_1],
        [6133, Diagnostics_0_is_declared_but_its_value_is_never_read],
        [6196, Diagnostics_0_is_declared_but_never_used],
        [7006, Diagnostics_Parameter_0_implicitly_has_an_1_type],
        [7027, Diagnostics_Unreachable_code_detected],
    ]);
}
