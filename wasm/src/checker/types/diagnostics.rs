//! Diagnostic codes for the type checker.
//!
//! The Diagnostic struct itself is in state.rs.

/// TypeScript diagnostic error codes.
/// Matches codes from TypeScript's diagnosticMessages.json
pub mod diagnostic_codes {
    // Scanner/Parser errors (1xxx)
    pub const UNTERMINATED_STRING_LITERAL: u32 = 1002;
    pub const IDENTIFIER_EXPECTED: u32 = 1003;
    pub const TOKEN_EXPECTED: u32 = 1005;  // '{0}' expected.
    pub const UNEXPECTED_TOKEN: u32 = 1012;
    pub const EXPRESSION_EXPECTED: u32 = 1109;
    pub const TYPE_EXPECTED: u32 = 1110;

    // Type checking errors (2xxx)
    pub const DUPLICATE_IDENTIFIER: u32 = 2300;
    pub const CANNOT_FIND_NAME: u32 = 2304;
    pub const MODULE_HAS_NO_EXPORTED_MEMBER: u32 = 2305;
    pub const GENERIC_TYPE_REQUIRES_TYPE_ARGUMENTS: u32 = 2314;
    pub const TYPE_IS_NOT_GENERIC: u32 = 2315;
    pub const TYPE_NOT_ASSIGNABLE_TO_TYPE: u32 = 2322;
    pub const PROPERTY_MISSING_IN_TYPE: u32 = 2324;
    pub const TYPES_OF_PROPERTY_INCOMPATIBLE: u32 = 2326;
    pub const PROPERTY_DOES_NOT_EXIST_ON_TYPE: u32 = 2339;
    pub const ARGUMENT_NOT_ASSIGNABLE_TO_PARAMETER: u32 = 2345;
    pub const CANNOT_INVOKE_NON_FUNCTION: u32 = 2349;
    pub const CANNOT_INVOKE_POSSIBLY_UNDEFINED: u32 = 2349;
    pub const EXPECTED_ARGUMENTS: u32 = 2554;  // Expected {0} arguments, but got {1}
    pub const EXPECTED_AT_LEAST_ARGUMENTS: u32 = 2555;
    pub const OBJECT_IS_POSSIBLY_UNDEFINED: u32 = 2532;
    pub const OBJECT_IS_POSSIBLY_NULL: u32 = 2531;
    pub const OBJECT_IS_OF_TYPE_UNKNOWN: u32 = 2571;
    pub const NOT_ALL_CODE_PATHS_RETURN_VALUE: u32 = 2366;
    pub const FUNCTION_LACKS_RETURN_TYPE: u32 = 2355;
    pub const TYPE_HAS_NO_PROPERTY: u32 = 2339;

    // Switch exhaustiveness
    pub const SWITCH_NOT_EXHAUSTIVE: u32 = 2761;  // Not all code paths return a value

    // Object literal errors
    pub const OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES: u32 = 2353;
    pub const EXCESS_PROPERTY_CHECK: u32 = 2353;

    // Index signature errors
    pub const INDEX_SIGNATURE_MISSING: u32 = 2329;
    pub const NO_INDEX_SIGNATURE: u32 = 7053;

    // Function errors
    pub const VOID_NOT_AWAITED: u32 = 2801;

    // Class errors
    pub const SUPER_ONLY_IN_DERIVED_CLASS: u32 = 2335;
    pub const THIS_CANNOT_BE_REFERENCED: u32 = 2332;
}
