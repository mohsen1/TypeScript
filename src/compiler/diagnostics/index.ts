/**
 * Diagnostics System - Module Index
 *
 * This module provides a comprehensive diagnostics system for TypeScript compilation,
 * including:
 *
 * - All TypeScript diagnostic codes with parameterized templates
 * - Thread-safe diagnostic collector with filtering and statistics
 * - Pretty formatter with ANSI color support
 * - Source code context display
 * - Multiple output formats (pretty, TSC-style, JSON)
 *
 * @example
 * ```typescript
 * // Create a collector
 * const collector = new ts.diagnostics.DiagnosticCollector();
 *
 * // Add diagnostics
 * collector.add(ts.diagnostics.MessageBuilders.cannotFindName("foo", location));
 *
 * // Format output
 * const formatter = ts.diagnostics.createConsoleFormatter();
 * console.log(formatter.formatAll(collector.getSorted()));
 * ```
 */

/* @internal */
namespace ts.diagnostics {
    // Re-export all types
    export type {
        DiagnosticLocation,
        RelatedInformation,
        Diagnostic,
        DiagnosticTemplate,
        FormatterOptions,
        FormattedDiagnostic,
        DiagnosticStats,
        DiagnosticCallback,
        DiagnosticFilter,
        ColorScheme,
    };

    // Export severity enum
    export { DiagnosticSeverity };

    // Export all diagnostic templates
    export {
        // Syntax errors
        Diagnostics_Unterminated_string_literal,
        Diagnostics_Identifier_expected,
        Diagnostics_0_expected,
        Diagnostics_A_file_cannot_have_a_reference_to_itself,
        Diagnostics_The_parser_expected_to_find_a_0_to_match_the_1_token_here,
        Diagnostics_Trailing_comma_not_allowed,
        Diagnostics_Asterisk_Slash_expected,
        Diagnostics_An_element_access_expression_should_take_an_argument,
        Diagnostics_Unexpected_token,
        Diagnostics_Expression_expected,
        Diagnostics_Type_expected,
        Diagnostics_Declaration_expected,

        // Semantic errors - binding
        Diagnostics_Duplicate_identifier_0,
        Diagnostics_Cannot_find_name_0,
        Diagnostics_Module_0_has_no_exported_member_1,
        Diagnostics_Cannot_find_module_0,
        Diagnostics_Module_0_has_no_default_export,
        Diagnostics_Cannot_redeclare_block_scoped_variable_0,
        Diagnostics_Variable_0_is_used_before_being_assigned,
        Diagnostics_Block_scoped_variable_0_used_before_its_declaration,

        // Semantic errors - type checking
        Diagnostics_Type_0_is_not_assignable_to_type_1,
        Diagnostics_Property_0_is_missing_in_type_1_but_required_in_type_2,
        Diagnostics_Property_0_does_not_exist_on_type_1,
        Diagnostics_Property_0_does_not_exist_on_type_1_Did_you_mean_2,
        Diagnostics_Argument_of_type_0_is_not_assignable_to_parameter_of_type_1,
        Diagnostics_Cannot_invoke_an_expression_whose_type_lacks_a_call_signature,
        Diagnostics_Type_0_has_no_call_signatures,
        Diagnostics_This_expression_is_not_constructable,
        Diagnostics_Expected_0_arguments_but_got_1,
        Diagnostics_Expected_at_least_0_arguments_but_got_1,
        Diagnostics_Operator_0_cannot_be_applied_to_types_1_and_2,
        Diagnostics_A_function_whose_declared_type_is_neither_void_nor_any_must_return_a_value,
        Diagnostics_Not_all_code_paths_return_a_value,
        Diagnostics_Object_is_possibly_undefined,
        Diagnostics_Object_is_possibly_null,
        Diagnostics_Object_is_possibly_null_or_undefined,
        Diagnostics_Object_is_of_type_unknown,

        // Class errors
        Diagnostics_Class_0_incorrectly_extends_base_class_1,
        Diagnostics_Class_0_incorrectly_implements_interface_1,
        Diagnostics_Property_0_in_type_1_is_not_assignable_to_the_same_property_in_base_type_2,
        Diagnostics_super_can_only_be_referenced_in_a_derived_class,
        Diagnostics_Super_calls_are_not_permitted_outside_constructors_or_in_nested_functions_inside_constructors,
        Diagnostics_A_super_call_must_be_the_first_statement_in_the_constructor,
        Diagnostics_Constructors_for_derived_classes_must_contain_a_super_call,
        Diagnostics_Property_0_has_no_initializer_and_is_not_definitely_assigned_in_the_constructor,

        // Generic errors
        Diagnostics_Type_0_does_not_satisfy_the_constraint_1,
        Diagnostics_Generic_type_0_requires_1_type_argument_s,
        Diagnostics_Expected_0_type_arguments_but_got_1,
        Diagnostics_Type_parameter_0_has_a_circular_constraint,

        // Async errors
        Diagnostics_await_expressions_are_only_allowed_within_async_functions_and_at_the_top_levels_of_modules,
        Diagnostics_The_return_type_of_an_async_function_or_method_must_be_the_global_Promise_T_type,

        // Strict mode errors
        Diagnostics_Parameter_0_implicitly_has_an_1_type,
        Diagnostics_Variable_0_implicitly_has_type_1_in_some_locations_where_its_type_cannot_be_determined,
        Diagnostics_Function_expression_which_lacks_return_type_annotation_implicitly_has_an_0_return_type,
        Diagnostics_Element_implicitly_has_an_any_type_because_expression_of_type_0_cant_be_used_to_index_type_1,
        Diagnostics_No_index_signature_with_a_parameter_of_type_0_was_found_on_type_1,

        // Declaration emit errors
        Diagnostics_Exported_variable_0_has_or_is_using_name_1_from_external_module_2_but_cannot_be_named,
        Diagnostics_The_inferred_type_of_0_cannot_be_named_without_a_reference_to_1_This_is_likely_not_portable_A_type_annotation_is_necessary,
        Diagnostics_Return_type_of_exported_function_has_or_is_using_private_name_0,

        // Compiler options errors
        Diagnostics_Option_0_cannot_be_specified_with_option_1,
        Diagnostics_Cannot_find_a_tsconfig_json_file_at_the_specified_directory_0,

        // Warnings
        Diagnostics_0_is_declared_but_its_value_is_never_read,
        Diagnostics_0_is_declared_but_never_used,
        Diagnostics_All_imports_in_import_declaration_are_unused,
        Diagnostics_Unreachable_code_detected,
        Diagnostics_Unused_label,
        Diagnostics_Fallthrough_case_in_switch,

        // Lookup map
        diagnosticTemplatesByCode,
    };

    // Export message utilities
    export {
        formatMessage,
        createDiagnosticFromTemplate,
        createDiagnosticWithRelated,
        createLocation,
        createLocationFromSourceFile,
        createRelatedInfo,
        MessageBuilders,
        createDiagnosticChain,
        getSeverityName,
        compareDiagnostics,
        sortDiagnostics,
    };

    // Export collector
    export {
        DiagnosticCollector,
        CommonFilters,
        getGlobalCollector,
        resetGlobalCollector,
        setGlobalCollector,
    };

    // Export formatter
    export {
        Colors,
        DefaultColorScheme,
        NoColorScheme,
        DiagnosticFormatter,
        formatDiagnostic,
        formatDiagnostics,
        formatTscStyle,
        formatAsJson,
        createConsoleFormatter,
        createPlainFormatter,
    };
}
