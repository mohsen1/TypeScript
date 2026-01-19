describe("unittests:: ThinChecker", () => {
    function createTestProgram(content: string): ts.Program {
        const host = new fakes.CompilerHost(vfs.createFromFileSystem(
            Harness.IO,
            /*ignoreCase*/ true,
            { documents: [new documents.TextDocument("/test.ts", content)], cwd: "/" }
        ));

        return ts.createProgram({
            host,
            rootNames: ["/test.ts"],
            options: { noLib: true, strict: true }
        });
    }

    function createThinCheckerForSource(content: string): { checker: ts.ThinTypeChecker; sourceFile: ts.SourceFile } {
        const program = createTestProgram(content);
        const sourceFile = program.getSourceFile("/test.ts")!;
        const checker = ts.createThinTypeChecker(program);
        return { checker, sourceFile };
    }

    describe("basic type checking", () => {
        it("should check simple variable declarations", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                let x: number = 5;
                let y: string = "hello";
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.equal(diagnostics.length, 0, "No errors expected for valid declarations");
        });

        it("should detect type mismatch in variable initialization", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                let x: number = "hello";
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.isAbove(diagnostics.length, 0, "Expected error for type mismatch");
            assert.include(diagnostics[0].messageText as string, "not assignable");
        });

        it("should check function parameter types", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                function add(a: number, b: number): number {
                    return a + b;
                }
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.equal(diagnostics.length, 0, "No errors expected for valid function");
        });

        it("should detect wrong return type", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                function greet(): number {
                    return "hello";
                }
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.isAbove(diagnostics.length, 0, "Expected error for wrong return type");
        });
    });

    describe("expression evaluation", () => {
        it("should infer number type for arithmetic", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                let x = 5 + 3;
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.equal(diagnostics.length, 0);
        });

        it("should infer string type for concatenation", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                let x = "hello" + "world";
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.equal(diagnostics.length, 0);
        });

        it("should infer boolean type for comparisons", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                let x = 5 > 3;
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.equal(diagnostics.length, 0);
        });
    });

    describe("identifier resolution", () => {
        it("should detect undefined identifier", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                let x = unknownVariable;
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.isAbove(diagnostics.length, 0, "Expected error for undefined identifier");
            assert.include(diagnostics[0].messageText as string, "Cannot find name");
        });
    });

    describe("property access", () => {
        it("should check valid property access", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                interface Person {
                    name: string;
                }
                declare const person: Person;
                let name = person.name;
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.equal(diagnostics.length, 0);
        });

        it("should detect invalid property access", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                interface Person {
                    name: string;
                }
                declare const person: Person;
                let age = person.age;
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.isAbove(diagnostics.length, 0, "Expected error for invalid property");
            assert.include(diagnostics[0].messageText as string, "does not exist on type");
        });
    });

    describe("function calls", () => {
        it("should check valid function call", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                declare function greet(name: string): void;
                greet("world");
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.equal(diagnostics.length, 0);
        });

        it("should detect incorrect argument type", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                declare function greet(name: string): void;
                greet(42);
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.isAbove(diagnostics.length, 0, "Expected error for wrong argument type");
            assert.include(diagnostics[0].messageText as string, "not assignable to parameter");
        });

        it("should detect call on non-callable", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                let x: number = 5;
                x();
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.isAbove(diagnostics.length, 0, "Expected error for calling non-function");
            assert.include(diagnostics[0].messageText as string, "not callable");
        });
    });

    describe("control flow", () => {
        it("should check if statement branches", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                let x: number = 5;
                if (x > 0) {
                    let y = x + 1;
                } else {
                    let z = x - 1;
                }
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.equal(diagnostics.length, 0);
        });

        it("should check for loop", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                for (let i = 0; i < 10; i++) {
                    let x = i * 2;
                }
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.equal(diagnostics.length, 0);
        });

        it("should check while loop", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                let i = 0;
                while (i < 10) {
                    i++;
                }
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.equal(diagnostics.length, 0);
        });
    });

    describe("class checking", () => {
        it("should check class property types", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                class Person {
                    name: string = "";
                    age: number = 0;
                }
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.equal(diagnostics.length, 0);
        });

        it("should detect wrong property initializer type", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                class Person {
                    age: number = "not a number";
                }
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.isAbove(diagnostics.length, 0, "Expected error for wrong property type");
        });
    });

    describe("type assertions", () => {
        it("should handle as expression", () => {
            const { checker, sourceFile } = createThinCheckerForSource(`
                let x: any = "hello";
                let y = x as string;
            `);

            const diagnostics = checker.checkSourceFile(sourceFile);
            assert.equal(diagnostics.length, 0);
        });
    });

    describe("isTypeAssignableTo", () => {
        it("should return true for same types", () => {
            const program = createTestProgram(`let x: number;`);
            const checker = ts.createThinTypeChecker(program);
            const fullChecker = program.getTypeChecker();

            // Use internal methods via casting
            const numType = (fullChecker as any).getNumberType();
            assert.isTrue(checker.isTypeAssignableTo(numType, numType));
        });

        it("should return false for incompatible types", () => {
            const program = createTestProgram(`let x: number;`);
            const checker = ts.createThinTypeChecker(program);
            const fullChecker = program.getTypeChecker();

            // Use internal methods via casting
            const numType = (fullChecker as any).getNumberType();
            const strType = (fullChecker as any).getStringType();
            assert.isFalse(checker.isTypeAssignableTo(numType, strType));
        });
    });
});
