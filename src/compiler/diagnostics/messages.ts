/**
 * Diagnostics System - Parameterized Message Templates
 *
 * This module provides utilities for working with parameterized diagnostic messages,
 * including formatting, interpolation, and localization support.
 */

/* @internal */
namespace ts.diagnostics {
    /**
     * Format a message template with the given arguments
     *
     * Replaces {0}, {1}, etc. placeholders with provided arguments.
     * Handles escaping and special characters properly.
     */
    export function formatMessage(template: string, args: (string | number)[]): string {
        return template.replace(/\{(\d+)\}/g, (match, index) => {
            const argIndex = parseInt(index, 10);
            if (argIndex < args.length) {
                return String(args[argIndex]);
            }
            return match; // Leave placeholder if arg not provided
        });
    }

    /**
     * Create a diagnostic from a template with arguments
     */
    export function createDiagnosticFromTemplate(
        template: DiagnosticTemplate,
        args: (string | number)[],
        location?: DiagnosticLocation
    ): Diagnostic {
        return {
            code: template.code,
            severity: template.severity,
            message: formatMessage(template.messageTemplate, args),
            location,
            category: template.category,
            source: "TS",
        };
    }

    /**
     * Create a diagnostic with related information
     */
    export function createDiagnosticWithRelated(
        template: DiagnosticTemplate,
        args: (string | number)[],
        location: DiagnosticLocation | undefined,
        relatedInfo: RelatedInformation[]
    ): Diagnostic {
        const diagnostic = createDiagnosticFromTemplate(template, args, location);
        diagnostic.relatedInformation = relatedInfo;
        return diagnostic;
    }

    /**
     * Create a diagnostic location from a source file and position
     */
    export function createLocation(
        fileName: string,
        start: number,
        length: number,
        sourceText?: string
    ): DiagnosticLocation {
        let line = 1;
        let column = 1;

        if (sourceText) {
            // Calculate line and column from position
            for (let i = 0; i < start && i < sourceText.length; i++) {
                if (sourceText.charCodeAt(i) === 10 /* newline */) {
                    line++;
                    column = 1;
                } else {
                    column++;
                }
            }
        }

        return {
            file: fileName,
            start,
            length,
            line,
            column,
        };
    }

    /**
     * Create a diagnostic location from TypeScript SourceFile and TextSpan
     */
    export function createLocationFromSourceFile(
        sourceFile: SourceFile,
        start: number,
        length: number
    ): DiagnosticLocation {
        const lineAndChar = getLineAndCharacterOfPosition(sourceFile, start);
        return {
            file: sourceFile.fileName,
            start,
            length,
            line: lineAndChar.line + 1, // Convert to 1-based
            column: lineAndChar.character + 1, // Convert to 1-based
        };
    }

    /**
     * Create related information entry
     */
    export function createRelatedInfo(
        location: DiagnosticLocation,
        message: string,
        code?: number
    ): RelatedInformation {
        return {
            location,
            message,
            code,
        };
    }

    /**
     * Message builders for common diagnostic patterns
     */
    export const MessageBuilders = {
        /**
         * Cannot find name 'x'
         */
        cannotFindName(name: string, location?: DiagnosticLocation): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Cannot_find_name_0,
                [name],
                location
            );
        },

        /**
         * Type 'x' is not assignable to type 'y'
         */
        typeNotAssignable(
            sourceType: string,
            targetType: string,
            location?: DiagnosticLocation
        ): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Type_0_is_not_assignable_to_type_1,
                [sourceType, targetType],
                location
            );
        },

        /**
         * Property 'x' does not exist on type 'y'
         */
        propertyDoesNotExist(
            propertyName: string,
            typeName: string,
            location?: DiagnosticLocation
        ): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Property_0_does_not_exist_on_type_1,
                [propertyName, typeName],
                location
            );
        },

        /**
         * Property 'x' does not exist on type 'y'. Did you mean 'z'?
         */
        propertyDoesNotExistWithSuggestion(
            propertyName: string,
            typeName: string,
            suggestion: string,
            location?: DiagnosticLocation
        ): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Property_0_does_not_exist_on_type_1_Did_you_mean_2,
                [propertyName, typeName, suggestion],
                location
            );
        },

        /**
         * Argument of type 'x' is not assignable to parameter of type 'y'
         */
        argumentNotAssignable(
            argType: string,
            paramType: string,
            location?: DiagnosticLocation
        ): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Argument_of_type_0_is_not_assignable_to_parameter_of_type_1,
                [argType, paramType],
                location
            );
        },

        /**
         * Expected X arguments, but got Y
         */
        wrongArgumentCount(
            expected: number,
            actual: number,
            location?: DiagnosticLocation
        ): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Expected_0_arguments_but_got_1,
                [expected, actual],
                location
            );
        },

        /**
         * Duplicate identifier 'x'
         */
        duplicateIdentifier(name: string, location?: DiagnosticLocation): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Duplicate_identifier_0,
                [name],
                location
            );
        },

        /**
         * Cannot find module 'x'
         */
        cannotFindModule(moduleName: string, location?: DiagnosticLocation): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Cannot_find_module_0,
                [moduleName],
                location
            );
        },

        /**
         * Parameter 'x' implicitly has an 'any' type
         */
        implicitAnyParameter(paramName: string, location?: DiagnosticLocation): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Parameter_0_implicitly_has_an_1_type,
                [paramName, "any"],
                location
            );
        },

        /**
         * 'x' is declared but never used
         */
        unusedDeclaration(name: string, location?: DiagnosticLocation): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_0_is_declared_but_never_used,
                [name],
                location
            );
        },

        /**
         * Generic type 'X' requires Y type argument(s)
         */
        missingTypeArguments(
            typeName: string,
            count: number,
            location?: DiagnosticLocation
        ): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Generic_type_0_requires_1_type_argument_s,
                [typeName, count],
                location
            );
        },

        /**
         * Token expected
         */
        tokenExpected(token: string, location?: DiagnosticLocation): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_0_expected,
                [token],
                location
            );
        },

        /**
         * Object is possibly 'undefined'
         */
        possiblyUndefined(location?: DiagnosticLocation): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Object_is_possibly_undefined,
                [],
                location
            );
        },

        /**
         * Object is possibly 'null'
         */
        possiblyNull(location?: DiagnosticLocation): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Object_is_possibly_null,
                [],
                location
            );
        },

        /**
         * Object is possibly 'null' or 'undefined'
         */
        possiblyNullOrUndefined(location?: DiagnosticLocation): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Object_is_possibly_null_or_undefined,
                [],
                location
            );
        },

        /**
         * Variable 'x' is used before being assigned
         */
        usedBeforeAssigned(name: string, location?: DiagnosticLocation): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Variable_0_is_used_before_being_assigned,
                [name],
                location
            );
        },

        /**
         * Type 'x' does not satisfy the constraint 'y'
         */
        constraintViolation(
            typeName: string,
            constraintName: string,
            location?: DiagnosticLocation
        ): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Type_0_does_not_satisfy_the_constraint_1,
                [typeName, constraintName],
                location
            );
        },

        /**
         * Class 'x' incorrectly extends base class 'y'
         */
        incorrectExtends(
            className: string,
            baseClassName: string,
            location?: DiagnosticLocation
        ): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Class_0_incorrectly_extends_base_class_1,
                [className, baseClassName],
                location
            );
        },

        /**
         * Class 'x' incorrectly implements interface 'y'
         */
        incorrectImplements(
            className: string,
            interfaceName: string,
            location?: DiagnosticLocation
        ): Diagnostic {
            return createDiagnosticFromTemplate(
                Diagnostics_Class_0_incorrectly_implements_interface_1,
                [className, interfaceName],
                location
            );
        },
    };

    /**
     * Diagnostic chain for detailed type errors
     *
     * Creates a chain of diagnostics for complex type mismatches
     */
    export function createDiagnosticChain(
        primary: Diagnostic,
        ...chain: Array<{ template: DiagnosticTemplate; args: (string | number)[] }>
    ): Diagnostic {
        const relatedInfo: RelatedInformation[] = chain.map(item => ({
            location: primary.location!,
            message: formatMessage(item.template.messageTemplate, item.args),
            code: item.template.code,
        }));

        return {
            ...primary,
            relatedInformation: [
                ...(primary.relatedInformation || []),
                ...relatedInfo,
            ],
        };
    }

    /**
     * Get a human-readable severity name
     */
    export function getSeverityName(severity: DiagnosticSeverity): string {
        switch (severity) {
            case DiagnosticSeverity.Error:
                return "error";
            case DiagnosticSeverity.Warning:
                return "warning";
            case DiagnosticSeverity.Suggestion:
                return "suggestion";
            case DiagnosticSeverity.Message:
                return "message";
            default:
                return "unknown";
        }
    }

    /**
     * Compare two diagnostics for sorting
     */
    export function compareDiagnostics(a: Diagnostic, b: Diagnostic): number {
        // First by file name
        if (a.location && b.location) {
            const fileCompare = a.location.file.localeCompare(b.location.file);
            if (fileCompare !== 0) return fileCompare;

            // Then by position
            const posCompare = a.location.start - b.location.start;
            if (posCompare !== 0) return posCompare;
        } else if (a.location) {
            return -1;
        } else if (b.location) {
            return 1;
        }

        // Then by severity (errors first)
        const sevCompare = a.severity - b.severity;
        if (sevCompare !== 0) return sevCompare;

        // Finally by code
        return a.code - b.code;
    }

    /**
     * Sort an array of diagnostics
     */
    export function sortDiagnostics(diagnostics: Diagnostic[]): Diagnostic[] {
        return diagnostics.slice().sort(compareDiagnostics);
    }
}
