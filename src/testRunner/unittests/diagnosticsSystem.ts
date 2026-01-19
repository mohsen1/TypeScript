describe("unittests:: Diagnostics System", () => {
    describe("DiagnosticSeverity", () => {
        it("should define severity levels", () => {
            assert.equal(ts.diagnostics.DiagnosticSeverity.Error, 0);
            assert.equal(ts.diagnostics.DiagnosticSeverity.Warning, 1);
            assert.equal(ts.diagnostics.DiagnosticSeverity.Suggestion, 2);
            assert.equal(ts.diagnostics.DiagnosticSeverity.Message, 3);
        });
    });

    describe("formatMessage", () => {
        it("should format message with placeholders", () => {
            const result = ts.diagnostics.formatMessage("Cannot find name '{0}'.", ["foo"]);
            assert.equal(result, "Cannot find name 'foo'.");
        });

        it("should handle multiple placeholders", () => {
            const result = ts.diagnostics.formatMessage(
                "Type '{0}' is not assignable to type '{1}'.",
                ["string", "number"]
            );
            assert.equal(result, "Type 'string' is not assignable to type 'number'.");
        });

        it("should handle numeric arguments", () => {
            const result = ts.diagnostics.formatMessage(
                "Expected {0} arguments, but got {1}.",
                [3, 5]
            );
            assert.equal(result, "Expected 3 arguments, but got 5.");
        });

        it("should leave placeholder if arg not provided", () => {
            const result = ts.diagnostics.formatMessage(
                "Missing {0} and {1}.",
                ["first"]
            );
            assert.equal(result, "Missing first and {1}.");
        });
    });

    describe("createDiagnosticFromTemplate", () => {
        it("should create diagnostic from template", () => {
            const diagnostic = ts.diagnostics.createDiagnosticFromTemplate(
                ts.diagnostics.Diagnostics_Cannot_find_name_0,
                ["myVariable"]
            );

            assert.equal(diagnostic.code, 2304);
            assert.equal(diagnostic.severity, ts.diagnostics.DiagnosticSeverity.Error);
            assert.equal(diagnostic.message, "Cannot find name 'myVariable'.");
            assert.equal(diagnostic.category, "error");
            assert.equal(diagnostic.source, "TS");
        });

        it("should include location when provided", () => {
            const location: ts.diagnostics.DiagnosticLocation = {
                file: "/test.ts",
                start: 10,
                length: 5,
                line: 2,
                column: 4,
            };

            const diagnostic = ts.diagnostics.createDiagnosticFromTemplate(
                ts.diagnostics.Diagnostics_Cannot_find_name_0,
                ["x"],
                location
            );

            assert.isDefined(diagnostic.location);
            assert.equal(diagnostic.location!.file, "/test.ts");
            assert.equal(diagnostic.location!.line, 2);
            assert.equal(diagnostic.location!.column, 4);
        });
    });

    describe("MessageBuilders", () => {
        it("cannotFindName should create correct diagnostic", () => {
            const diagnostic = ts.diagnostics.MessageBuilders.cannotFindName("foo");
            assert.equal(diagnostic.code, 2304);
            assert.include(diagnostic.message, "foo");
        });

        it("typeNotAssignable should create correct diagnostic", () => {
            const diagnostic = ts.diagnostics.MessageBuilders.typeNotAssignable(
                "string",
                "number"
            );
            assert.equal(diagnostic.code, 2322);
            assert.include(diagnostic.message, "string");
            assert.include(diagnostic.message, "number");
        });

        it("propertyDoesNotExist should create correct diagnostic", () => {
            const diagnostic = ts.diagnostics.MessageBuilders.propertyDoesNotExist(
                "foo",
                "MyType"
            );
            assert.equal(diagnostic.code, 2339);
            assert.include(diagnostic.message, "foo");
            assert.include(diagnostic.message, "MyType");
        });

        it("wrongArgumentCount should create correct diagnostic", () => {
            const diagnostic = ts.diagnostics.MessageBuilders.wrongArgumentCount(2, 3);
            assert.equal(diagnostic.code, 2554);
            assert.include(diagnostic.message, "2");
            assert.include(diagnostic.message, "3");
        });

        it("possiblyUndefined should create correct diagnostic", () => {
            const diagnostic = ts.diagnostics.MessageBuilders.possiblyUndefined();
            assert.equal(diagnostic.code, 2532);
            assert.include(diagnostic.message, "undefined");
        });
    });

    describe("DiagnosticCollector", () => {
        it("should collect diagnostics", () => {
            const collector = new ts.diagnostics.DiagnosticCollector();

            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("a"));
            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("b"));

            assert.equal(collector.count, 2);
        });

        it("should report errors and warnings separately", () => {
            const collector = new ts.diagnostics.DiagnosticCollector();

            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("a"));
            collector.addFromTemplate(
                ts.diagnostics.Diagnostics_0_is_declared_but_never_used,
                ["unused"]
            );

            assert.equal(collector.errorCount, 1);
            assert.equal(collector.warningCount, 1);
        });

        it("hasErrors should work correctly", () => {
            const collector = new ts.diagnostics.DiagnosticCollector();
            assert.isFalse(collector.hasErrors());

            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("x"));
            assert.isTrue(collector.hasErrors());
        });

        it("should filter diagnostics by file", () => {
            const collector = new ts.diagnostics.DiagnosticCollector();

            const loc1: ts.diagnostics.DiagnosticLocation = {
                file: "/a.ts",
                start: 0,
                length: 1,
                line: 1,
                column: 1,
            };

            const loc2: ts.diagnostics.DiagnosticLocation = {
                file: "/b.ts",
                start: 0,
                length: 1,
                line: 1,
                column: 1,
            };

            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("x", loc1));
            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("y", loc2));

            const aFile = collector.getForFile("/a.ts");
            assert.equal(aFile.length, 1);
            assert.include(aFile[0].message, "x");
        });

        it("should support maxDiagnostics option", () => {
            const collector = new ts.diagnostics.DiagnosticCollector({
                maxDiagnostics: 2,
            });

            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("a"));
            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("b"));
            const added = collector.add(ts.diagnostics.MessageBuilders.cannotFindName("c"));

            assert.isFalse(added);
            assert.equal(collector.count, 2);
        });

        it("should support stopOnFirstError option", () => {
            const collector = new ts.diagnostics.DiagnosticCollector({
                stopOnFirstError: true,
            });

            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("a"));
            assert.isTrue(collector.isStopped());

            const added = collector.add(ts.diagnostics.MessageBuilders.cannotFindName("b"));
            assert.isFalse(added);
        });

        it("should group diagnostics by file", () => {
            const collector = new ts.diagnostics.DiagnosticCollector();

            const loc1: ts.diagnostics.DiagnosticLocation = {
                file: "/a.ts",
                start: 0,
                length: 1,
                line: 1,
                column: 1,
            };

            const loc2: ts.diagnostics.DiagnosticLocation = {
                file: "/b.ts",
                start: 0,
                length: 1,
                line: 1,
                column: 1,
            };

            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("x", loc1));
            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("y", loc1));
            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("z", loc2));

            const groups = collector.groupByFile();
            assert.equal(groups.get("/a.ts")?.length, 2);
            assert.equal(groups.get("/b.ts")?.length, 1);
        });

        it("should calculate statistics", () => {
            const collector = new ts.diagnostics.DiagnosticCollector();

            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("a"));
            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("b"));
            collector.addFromTemplate(
                ts.diagnostics.Diagnostics_0_is_declared_but_never_used,
                ["c"]
            );

            const stats = collector.getStats();
            assert.equal(stats.total, 3);
            assert.equal(stats.errors, 2);
            assert.equal(stats.warnings, 1);
        });

        it("should notify listeners", () => {
            const collector = new ts.diagnostics.DiagnosticCollector();
            const received: ts.diagnostics.Diagnostic[] = [];

            collector.addListener(d => received.push(d));

            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("x"));
            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("y"));

            assert.equal(received.length, 2);
        });

        it("should apply filters", () => {
            const collector = new ts.diagnostics.DiagnosticCollector({
                filters: [ts.diagnostics.CommonFilters.errorsOnly],
            });

            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("a"));
            collector.addFromTemplate(
                ts.diagnostics.Diagnostics_0_is_declared_but_never_used,
                ["b"]
            );

            assert.equal(collector.count, 1);
            assert.isTrue(collector.hasErrors());
            assert.isFalse(collector.hasWarnings());
        });

        it("should create child collectors", () => {
            const parent = new ts.diagnostics.DiagnosticCollector();
            const child = parent.createChild();

            child.add(ts.diagnostics.MessageBuilders.cannotFindName("x"));

            assert.equal(parent.count, 1);
            assert.equal(child.count, 1);
        });

        it("should merge collectors", () => {
            const collector1 = new ts.diagnostics.DiagnosticCollector();
            const collector2 = new ts.diagnostics.DiagnosticCollector();

            collector1.add(ts.diagnostics.MessageBuilders.cannotFindName("a"));
            collector2.add(ts.diagnostics.MessageBuilders.cannotFindName("b"));

            collector1.merge(collector2);
            assert.equal(collector1.count, 2);
        });
    });

    describe("CommonFilters", () => {
        it("errorsOnly should filter non-errors", () => {
            const error = ts.diagnostics.MessageBuilders.cannotFindName("x");
            const warning = ts.diagnostics.createDiagnosticFromTemplate(
                ts.diagnostics.Diagnostics_0_is_declared_but_never_used,
                ["x"]
            );

            assert.isTrue(ts.diagnostics.CommonFilters.errorsOnly(error));
            assert.isFalse(ts.diagnostics.CommonFilters.errorsOnly(warning));
        });

        it("excludeCodes should filter specific codes", () => {
            const filter = ts.diagnostics.CommonFilters.excludeCodes(2304, 2305);
            const d1 = ts.diagnostics.MessageBuilders.cannotFindName("x"); // 2304
            const d2 = ts.diagnostics.MessageBuilders.typeNotAssignable("a", "b"); // 2322

            assert.isFalse(filter(d1));
            assert.isTrue(filter(d2));
        });

        it("combine should AND filters", () => {
            const filter = ts.diagnostics.CommonFilters.combine(
                ts.diagnostics.CommonFilters.errorsOnly,
                ts.diagnostics.CommonFilters.excludeCodes(2304)
            );

            const cannotFind = ts.diagnostics.MessageBuilders.cannotFindName("x"); // 2304, error
            const typeError = ts.diagnostics.MessageBuilders.typeNotAssignable("a", "b"); // 2322, error
            const warning = ts.diagnostics.createDiagnosticFromTemplate(
                ts.diagnostics.Diagnostics_0_is_declared_but_never_used,
                ["x"]
            ); // warning

            assert.isFalse(filter(cannotFind)); // excluded by code
            assert.isTrue(filter(typeError)); // passes both
            assert.isFalse(filter(warning)); // not error
        });
    });

    describe("DiagnosticFormatter", () => {
        it("should format diagnostic without colors", () => {
            const formatter = new ts.diagnostics.DiagnosticFormatter({
                colors: false,
                showContext: false,
                pretty: false,
            });

            const diagnostic = ts.diagnostics.MessageBuilders.cannotFindName("foo");
            const result = formatter.format(diagnostic);

            assert.include(result.text, "TS2304");
            assert.include(result.text, "Cannot find name 'foo'");
            assert.isUndefined(result.coloredText);
        });

        it("should format diagnostic with location", () => {
            const formatter = new ts.diagnostics.DiagnosticFormatter({
                colors: false,
                showContext: false,
            });

            const location: ts.diagnostics.DiagnosticLocation = {
                file: "/test.ts",
                start: 10,
                length: 3,
                line: 2,
                column: 5,
            };

            const diagnostic = ts.diagnostics.MessageBuilders.cannotFindName("foo", location);
            const result = formatter.format(diagnostic);

            assert.include(result.text, "/test.ts");
            assert.include(result.text, "2");
            assert.include(result.text, "5");
        });

        it("should format multiple diagnostics", () => {
            const formatter = new ts.diagnostics.DiagnosticFormatter({
                colors: false,
                showContext: false,
            });

            const diagnostics = [
                ts.diagnostics.MessageBuilders.cannotFindName("a"),
                ts.diagnostics.MessageBuilders.cannotFindName("b"),
            ];

            const result = formatter.formatAll(diagnostics);
            assert.include(result, "Cannot find name 'a'");
            assert.include(result, "Cannot find name 'b'");
        });

        it("should show source context when enabled", () => {
            const sourceCode = `const x = 1;
const y = foo;
const z = 3;`;

            const formatter = new ts.diagnostics.DiagnosticFormatter(
                {
                    colors: false,
                    showContext: true,
                    contextLines: 1,
                },
                fileName => {
                    if (fileName === "/test.ts") return sourceCode;
                    return undefined;
                }
            );

            const location: ts.diagnostics.DiagnosticLocation = {
                file: "/test.ts",
                start: 18,
                length: 3,
                line: 2,
                column: 11,
            };

            const diagnostic = ts.diagnostics.MessageBuilders.cannotFindName("foo", location);
            const result = formatter.format(diagnostic);

            assert.include(result.text, "foo");
        });
    });

    describe("formatTscStyle", () => {
        it("should format in TSC style", () => {
            const location: ts.diagnostics.DiagnosticLocation = {
                file: "/test.ts",
                start: 0,
                length: 3,
                line: 5,
                column: 10,
            };

            const diagnostics = [
                ts.diagnostics.MessageBuilders.cannotFindName("foo", location),
            ];

            const result = ts.diagnostics.formatTscStyle(diagnostics);
            assert.equal(result, "/test.ts(5,10): error TS2304: Cannot find name 'foo'.");
        });
    });

    describe("formatAsJson", () => {
        it("should format as JSON", () => {
            const diagnostics = [
                ts.diagnostics.MessageBuilders.cannotFindName("foo"),
            ];

            const result = ts.diagnostics.formatAsJson(diagnostics);
            const parsed = JSON.parse(result);

            assert.isArray(parsed);
            assert.equal(parsed.length, 1);
            assert.equal(parsed[0].code, 2304);
            assert.equal(parsed[0].severity, "error");
        });
    });

    describe("getSeverityName", () => {
        it("should return correct names", () => {
            assert.equal(
                ts.diagnostics.getSeverityName(ts.diagnostics.DiagnosticSeverity.Error),
                "error"
            );
            assert.equal(
                ts.diagnostics.getSeverityName(ts.diagnostics.DiagnosticSeverity.Warning),
                "warning"
            );
            assert.equal(
                ts.diagnostics.getSeverityName(ts.diagnostics.DiagnosticSeverity.Suggestion),
                "suggestion"
            );
            assert.equal(
                ts.diagnostics.getSeverityName(ts.diagnostics.DiagnosticSeverity.Message),
                "message"
            );
        });
    });

    describe("sortDiagnostics", () => {
        it("should sort by file, then position, then severity", () => {
            const loc1: ts.diagnostics.DiagnosticLocation = {
                file: "/a.ts",
                start: 10,
                length: 1,
                line: 2,
                column: 1,
            };

            const loc2: ts.diagnostics.DiagnosticLocation = {
                file: "/a.ts",
                start: 5,
                length: 1,
                line: 1,
                column: 5,
            };

            const loc3: ts.diagnostics.DiagnosticLocation = {
                file: "/b.ts",
                start: 0,
                length: 1,
                line: 1,
                column: 1,
            };

            const diagnostics = [
                ts.diagnostics.MessageBuilders.cannotFindName("x", loc1),
                ts.diagnostics.MessageBuilders.cannotFindName("y", loc2),
                ts.diagnostics.MessageBuilders.cannotFindName("z", loc3),
            ];

            const sorted = ts.diagnostics.sortDiagnostics(diagnostics);

            // /a.ts with position 5 should come first
            assert.include(sorted[0].message, "y");
            // /a.ts with position 10 should come second
            assert.include(sorted[1].message, "x");
            // /b.ts should come last
            assert.include(sorted[2].message, "z");
        });
    });

    describe("createDiagnosticChain", () => {
        it("should chain diagnostics with related info", () => {
            const location: ts.diagnostics.DiagnosticLocation = {
                file: "/test.ts",
                start: 0,
                length: 1,
                line: 1,
                column: 1,
            };

            const primary = ts.diagnostics.MessageBuilders.typeNotAssignable(
                "string",
                "number",
                location
            );

            const chained = ts.diagnostics.createDiagnosticChain(
                primary,
                {
                    template: ts.diagnostics.Diagnostics_Property_0_is_missing_in_type_1_but_required_in_type_2,
                    args: ["foo", "A", "B"],
                }
            );

            assert.isDefined(chained.relatedInformation);
            assert.isAbove(chained.relatedInformation!.length, 0);
        });
    });

    describe("Diagnostic codes", () => {
        it("should have unique codes", () => {
            const codes = new Set<number>();
            const templates = [
                ts.diagnostics.Diagnostics_Unterminated_string_literal,
                ts.diagnostics.Diagnostics_Identifier_expected,
                ts.diagnostics.Diagnostics_0_expected,
                ts.diagnostics.Diagnostics_Cannot_find_name_0,
                ts.diagnostics.Diagnostics_Type_0_is_not_assignable_to_type_1,
                ts.diagnostics.Diagnostics_Property_0_does_not_exist_on_type_1,
                ts.diagnostics.Diagnostics_Argument_of_type_0_is_not_assignable_to_parameter_of_type_1,
            ];

            for (const template of templates) {
                assert.isFalse(codes.has(template.code), `Duplicate code: ${template.code}`);
                codes.add(template.code);
            }
        });

        it("should have correct categories", () => {
            assert.equal(
                ts.diagnostics.Diagnostics_Cannot_find_name_0.category,
                "error"
            );
            assert.equal(
                ts.diagnostics.Diagnostics_0_is_declared_but_never_used.category,
                "warning"
            );
        });
    });

    describe("Global collector", () => {
        it("should get global collector", () => {
            const collector = ts.diagnostics.getGlobalCollector();
            assert.isDefined(collector);
        });

        it("should reset global collector", () => {
            const collector = ts.diagnostics.getGlobalCollector();
            collector.add(ts.diagnostics.MessageBuilders.cannotFindName("x"));

            ts.diagnostics.resetGlobalCollector();
            assert.equal(collector.count, 0);
        });
    });

    describe("createLocation", () => {
        it("should create location with line/column from source text", () => {
            const source = `line one
line two
line three`;

            // Position at start of "two" (line 2, column 6)
            const location = ts.diagnostics.createLocation("/test.ts", 14, 3, source);

            assert.equal(location.file, "/test.ts");
            assert.equal(location.start, 14);
            assert.equal(location.length, 3);
            assert.equal(location.line, 2);
            assert.equal(location.column, 6);
        });
    });
});
