//! Module Transformations
//!
//! This module provides transformations for converting ES modules to:
//! - CommonJS (require/module.exports)
//! - AMD (define)
//! - UMD (Universal Module Definition)
//! - SystemJS

use std::collections::HashMap;
use zang_core::StringInterner;
use zang_parser::{
    ExportDeclaration, ImportDeclaration,
    NamedExportBindings, NamedImportBindings, Statement, SourceFile,
};

/// Target module format for transformation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleFormat {
    /// CommonJS (Node.js style)
    CommonJs,
    /// AMD (Asynchronous Module Definition)
    Amd,
    /// UMD (Universal Module Definition)
    Umd,
    /// SystemJS
    System,
    /// ES Modules (no transformation)
    EsNext,
}

impl Default for ModuleFormat {
    fn default() -> Self {
        Self::CommonJs
    }
}

/// Options for module transformation
#[derive(Debug, Clone)]
pub struct TransformOptions {
    /// Target module format
    pub format: ModuleFormat,
    /// Whether to emit helpers inline or import from tslib
    pub import_helpers: bool,
    /// Whether to preserve const enums
    pub preserve_const_enums: bool,
    /// The module name for AMD/UMD (if specified)
    pub module_name: Option<String>,
    /// Whether to use strict mode
    pub strict: bool,
    /// Whether to emit interop helpers for default imports
    pub es_module_interop: bool,
    /// Whether to allow synthetic default imports
    pub allow_synthetic_default_imports: bool,
}

impl Default for TransformOptions {
    fn default() -> Self {
        Self {
            format: ModuleFormat::CommonJs,
            import_helpers: false,
            preserve_const_enums: false,
            module_name: None,
            strict: true,
            es_module_interop: true,
            allow_synthetic_default_imports: true,
        }
    }
}

/// Result of a module transformation
#[derive(Debug, Clone)]
pub struct TransformResult {
    /// The transformed output as a string
    pub output: String,
    /// Import declarations that need to be converted
    pub imports: Vec<TransformedImport>,
    /// Export declarations that need to be converted
    pub exports: Vec<TransformedExport>,
    /// Helper functions needed
    pub helpers_needed: Vec<HelperFunction>,
}

/// A transformed import
#[derive(Debug, Clone)]
pub struct TransformedImport {
    /// The local variable name
    pub local_name: String,
    /// The module specifier
    pub module_specifier: String,
    /// Whether this is a namespace import
    pub is_namespace: bool,
    /// Whether this is a default import
    pub is_default: bool,
    /// Named imports (local -> imported)
    pub named_imports: HashMap<String, String>,
}

/// A transformed export
#[derive(Debug, Clone)]
pub struct TransformedExport {
    /// The exported name
    pub exported_name: String,
    /// The local name (if different)
    pub local_name: Option<String>,
    /// Whether this is the default export
    pub is_default: bool,
    /// Whether this is a re-export
    pub is_re_export: bool,
    /// Re-export module (if re-export)
    pub re_export_module: Option<String>,
}

/// Helper function needed for interop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelperFunction {
    /// __importDefault for default imports
    ImportDefault,
    /// __importStar for namespace imports
    ImportStar,
    /// __exportStar for re-exporting
    ExportStar,
    /// __createBinding for creating export bindings
    CreateBinding,
}

/// Module transformer trait
pub trait ModuleTransformer {
    /// Transforms a source file to the target module format
    fn transform(&self, source: &SourceFile, options: &TransformOptions) -> TransformResult;

    /// Generates the import statement for the target format
    fn generate_import(&self, import: &TransformedImport) -> String;

    /// Generates the export statement for the target format
    fn generate_export(&self, export: &TransformedExport) -> String;

    /// Generates helper function code if needed
    fn generate_helpers(&self, helpers: &[HelperFunction]) -> String;
}

/// CommonJS transformer
pub struct CommonJsTransformer<'a> {
    interner: &'a StringInterner,
}

impl<'a> CommonJsTransformer<'a> {
    /// Creates a new CommonJS transformer
    pub fn new(interner: &'a StringInterner) -> Self {
        Self { interner }
    }

    /// Transforms an import declaration to CommonJS
    fn transform_import(&self, import: &ImportDeclaration) -> TransformedImport {
        let module_specifier = self
            .interner
            .lookup(import.module_specifier.value)
            .unwrap_or_default();

        let mut result = TransformedImport {
            local_name: String::new(),
            module_specifier,
            is_namespace: false,
            is_default: false,
            named_imports: HashMap::new(),
        };

        if let Some(ref clause) = import.import_clause {
            // Default import
            if let Some(ref name) = clause.name {
                result.local_name = self.interner.lookup(name.name).unwrap_or_else(|| "".to_string()).to_string();
                result.is_default = true;
            }

            // Named bindings
            if let Some(ref bindings) = clause.named_bindings {
                match bindings {
                    NamedImportBindings::Namespace(ns) => {
                        result.local_name =
                            self.interner.lookup(ns.name.name).unwrap_or_else(|| "".to_string()).to_string();
                        result.is_namespace = true;
                    }
                    NamedImportBindings::Named(named) => {
                        for specifier in &named.elements {
                            let local = self
                                .interner
                                .lookup(specifier.name.name)
                                .unwrap_or_default();
                            let imported = specifier
                                .property_name
                                .as_ref()
                                .map(|p| self.interner.lookup(p.name).unwrap_or_else(|| "".to_string()).to_string())
                                .unwrap_or_else(|| local.clone());
                            result.named_imports.insert(local, imported);
                        }
                    }
                }
            }
        }

        result
    }

    /// Transforms an export declaration to CommonJS
    fn transform_export(&self, export: &ExportDeclaration) -> Vec<TransformedExport> {
        let mut results = Vec::new();

        if let Some(ref clause) = export.export_clause {
            match clause {
                NamedExportBindings::Namespace(ns) => {
                    if let Some(ref name) = ns.name {
                        results.push(TransformedExport {
                            exported_name: self
                                .interner
                                .lookup(name.name)
                                .unwrap_or_default(),
                            local_name: None,
                            is_default: false,
                            is_re_export: export.module_specifier.is_some(),
                            re_export_module: export.module_specifier.as_ref().map(|s| {
                                self.interner.lookup(s.value).unwrap_or_default()
                            }),
                        });
                    }
                }
                NamedExportBindings::Named(named) => {
                    for specifier in &named.elements {
                        let exported = self
                            .interner
                            .lookup(specifier.name.name)
                            .unwrap_or_default();
                        let local = specifier
                            .property_name
                            .as_ref()
                            .map(|p| self.interner.lookup(p.name).unwrap_or_default());

                        results.push(TransformedExport {
                            exported_name: exported,
                            local_name: local,
                            is_default: false,
                            is_re_export: export.module_specifier.is_some(),
                            re_export_module: export.module_specifier.as_ref().map(|s| {
                                self.interner.lookup(s.value).unwrap_or_default()
                            }),
                        });
                    }
                }
            }
        }

        results
    }
}

impl<'a> ModuleTransformer for CommonJsTransformer<'a> {
    fn transform(&self, source: &SourceFile, options: &TransformOptions) -> TransformResult {
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut helpers_needed = Vec::new();
        let mut output = String::new();

        // Add "use strict" if needed
        if options.strict {
            output.push_str("\"use strict\";\n");
        }

        // Add __esModule marker
        if options.es_module_interop {
            output.push_str("Object.defineProperty(exports, \"__esModule\", { value: true });\n");
        }

        // Process statements
        for statement in &source.statements {
            match statement {
                Statement::Import(import) => {
                    let transformed = self.transform_import(import);
                    if transformed.is_default && options.es_module_interop {
                        helpers_needed.push(HelperFunction::ImportDefault);
                    }
                    if transformed.is_namespace && options.es_module_interop {
                        helpers_needed.push(HelperFunction::ImportStar);
                    }
                    output.push_str(&self.generate_import(&transformed));
                    output.push('\n');
                    imports.push(transformed);
                }
                Statement::Export(export) => {
                    let transformed = self.transform_export(export);
                    for exp in &transformed {
                        if exp.is_re_export {
                            helpers_needed.push(HelperFunction::ExportStar);
                        }
                        output.push_str(&self.generate_export(exp));
                        output.push('\n');
                    }
                    exports.extend(transformed);
                }
                Statement::ExportAssignment(assignment) => {
                    let exp = TransformedExport {
                        exported_name: "default".to_string(),
                        local_name: None,
                        is_default: !assignment.is_export_equals,
                        is_re_export: false,
                        re_export_module: None,
                    };
                    output.push_str(&self.generate_export(&exp));
                    output.push('\n');
                    exports.push(exp);
                }
                _ => {
                    // Other statements pass through
                }
            }
        }

        // Add helpers at the beginning if needed
        if !helpers_needed.is_empty() && !options.import_helpers {
            let helpers_code = self.generate_helpers(&helpers_needed);
            output = format!("{}\n{}", helpers_code, output);
        }

        TransformResult {
            output,
            imports,
            exports,
            helpers_needed,
        }
    }

    fn generate_import(&self, import: &TransformedImport) -> String {
        if import.is_namespace {
            format!(
                "const {} = __importStar(require(\"{}\"));",
                import.local_name, import.module_specifier
            )
        } else if import.is_default {
            format!(
                "const {} = __importDefault(require(\"{}\")).default;",
                import.local_name, import.module_specifier
            )
        } else if !import.named_imports.is_empty() {
            let destructure: Vec<String> = import
                .named_imports
                .iter()
                .map(|(local, imported)| {
                    if local == imported {
                        local.clone()
                    } else {
                        format!("{}: {}", imported, local)
                    }
                })
                .collect();
            format!(
                "const {{ {} }} = require(\"{}\");",
                destructure.join(", "),
                import.module_specifier
            )
        } else {
            format!("require(\"{}\");", import.module_specifier)
        }
    }

    fn generate_export(&self, export: &TransformedExport) -> String {
        if export.is_default {
            if let Some(ref local) = export.local_name {
                format!("exports.default = {};", local)
            } else {
                "exports.default = void 0;".to_string()
            }
        } else if export.is_re_export {
            if let Some(ref module) = export.re_export_module {
                if let Some(ref local) = export.local_name {
                    format!(
                        "exports.{} = require(\"{}\").{};",
                        export.exported_name, module, local
                    )
                } else {
                    format!(
                        "exports.{} = require(\"{}\").{};",
                        export.exported_name, module, export.exported_name
                    )
                }
            } else {
                String::new()
            }
        } else {
            let local = export.local_name.as_ref().unwrap_or(&export.exported_name);
            format!("exports.{} = {};", export.exported_name, local)
        }
    }

    fn generate_helpers(&self, helpers: &[HelperFunction]) -> String {
        let mut code = String::new();

        for helper in helpers {
            match helper {
                HelperFunction::ImportDefault => {
                    code.push_str(
                        r#"var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
"#,
                    );
                }
                HelperFunction::ImportStar => {
                    code.push_str(
                        r#"var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
"#,
                    );
                }
                HelperFunction::ExportStar => {
                    code.push_str(
                        r#"var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !exports.hasOwnProperty(p)) exports[p] = m[p];
};
"#,
                    );
                }
                HelperFunction::CreateBinding => {
                    code.push_str(
                        r#"var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
"#,
                    );
                }
            }
        }

        code
    }
}

/// AMD transformer
pub struct AmdTransformer<'a> {
    interner: &'a StringInterner,
}

impl<'a> AmdTransformer<'a> {
    /// Creates a new AMD transformer
    pub fn new(interner: &'a StringInterner) -> Self {
        Self { interner }
    }

    /// Gets module dependencies from imports
    fn get_dependencies(&self, source: &SourceFile) -> Vec<String> {
        let mut deps = vec!["require".to_string(), "exports".to_string()];

        for statement in &source.statements {
            if let Statement::Import(import) = statement {
                let specifier = self
                    .interner
                    .lookup(import.module_specifier.value)
                    .unwrap_or_default();
                if !deps.contains(&specifier) {
                    deps.push(specifier);
                }
            }
        }

        deps
    }
}

impl<'a> ModuleTransformer for AmdTransformer<'a> {
    fn transform(&self, source: &SourceFile, options: &TransformOptions) -> TransformResult {
        let deps = self.get_dependencies(source);
        let mut output = String::new();

        // Start AMD define
        if let Some(ref name) = options.module_name {
            output.push_str(&format!("define(\"{}\", [", name));
        } else {
            output.push_str("define([");
        }

        // Dependencies
        let dep_strings: Vec<String> = deps.iter().map(|d| format!("\"{}\"", d)).collect();
        output.push_str(&dep_strings.join(", "));
        output.push_str("], function(require, exports");

        // Module parameters (skip require and exports)
        for dep in deps.iter().skip(2) {
            let param_name = dep
                .split('/')
                .last()
                .unwrap_or(dep)
                .replace('-', "_")
                .replace('.', "_");
            output.push_str(&format!(", {}", param_name));
        }

        output.push_str(") {\n");

        // Add strict mode
        if options.strict {
            output.push_str("    \"use strict\";\n");
        }

        // Add __esModule marker
        if options.es_module_interop {
            output.push_str(
                "    Object.defineProperty(exports, \"__esModule\", { value: true });\n",
            );
        }

        // Process exports (simplified)
        for statement in &source.statements {
            if let Statement::Export(export) = statement {
                if export.declaration.is_some() {
                    // TODO: Emit declaration and export
                }
            }
        }

        output.push_str("});\n");

        TransformResult {
            output,
            imports: Vec::new(),
            exports: Vec::new(),
            helpers_needed: Vec::new(),
        }
    }

    fn generate_import(&self, import: &TransformedImport) -> String {
        // In AMD, imports are parameters to the factory function
        format!("// Import: {} from {}", import.local_name, import.module_specifier)
    }

    fn generate_export(&self, export: &TransformedExport) -> String {
        let local = export.local_name.as_ref().unwrap_or(&export.exported_name);
        format!("    exports.{} = {};", export.exported_name, local)
    }

    fn generate_helpers(&self, _helpers: &[HelperFunction]) -> String {
        // AMD typically doesn't need runtime helpers
        String::new()
    }
}

/// UMD transformer
pub struct UmdTransformer<'a> {
    interner: &'a StringInterner,
    cjs: CommonJsTransformer<'a>,
    amd: AmdTransformer<'a>,
}

impl<'a> UmdTransformer<'a> {
    /// Creates a new UMD transformer
    pub fn new(interner: &'a StringInterner) -> Self {
        Self {
            interner,
            cjs: CommonJsTransformer::new(interner),
            amd: AmdTransformer::new(interner),
        }
    }
}

impl<'a> ModuleTransformer for UmdTransformer<'a> {
    fn transform(&self, source: &SourceFile, options: &TransformOptions) -> TransformResult {
        let mut output = String::new();

        let module_name = options
            .module_name
            .clone()
            .unwrap_or_else(|| "module".to_string());

        // UMD wrapper
        output.push_str(&format!(
            r#"(function (factory) {{
    if (typeof module === "object" && typeof module.exports === "object") {{
        var v = factory(require, exports);
        if (v !== undefined) module.exports = v;
    }}
    else if (typeof define === "function" && define.amd) {{
        define(["require", "exports"], factory);
    }}
    else {{
        var g = typeof globalThis !== "undefined" ? globalThis : typeof self !== "undefined" ? self : this;
        g.{} = {{}};
        factory(function(m) {{ return g[m]; }}, g.{});
    }}
}})(function (require, exports) {{
"#,
            module_name, module_name
        ));

        // Add strict mode
        if options.strict {
            output.push_str("    \"use strict\";\n");
        }

        // Add __esModule marker
        if options.es_module_interop {
            output.push_str(
                "    Object.defineProperty(exports, \"__esModule\", { value: true });\n",
            );
        }

        // Process statements like CommonJS
        for statement in &source.statements {
            match statement {
                Statement::Import(import) => {
                    let transformed = self.cjs.transform_import(import);
                    output.push_str("    ");
                    output.push_str(&self.generate_import(&transformed));
                    output.push('\n');
                }
                Statement::Export(export) => {
                    let transformed = self.cjs.transform_export(export);
                    for exp in transformed {
                        output.push_str("    ");
                        output.push_str(&self.generate_export(&exp));
                        output.push('\n');
                    }
                }
                _ => {}
            }
        }

        output.push_str("});\n");

        TransformResult {
            output,
            imports: Vec::new(),
            exports: Vec::new(),
            helpers_needed: Vec::new(),
        }
    }

    fn generate_import(&self, import: &TransformedImport) -> String {
        self.cjs.generate_import(import)
    }

    fn generate_export(&self, export: &TransformedExport) -> String {
        self.cjs.generate_export(export)
    }

    fn generate_helpers(&self, helpers: &[HelperFunction]) -> String {
        self.cjs.generate_helpers(helpers)
    }
}

/// Creates a transformer for the specified module format
pub fn create_transformer<'a>(
    format: ModuleFormat,
    interner: &'a StringInterner,
) -> Box<dyn ModuleTransformer + 'a> {
    match format {
        ModuleFormat::CommonJs => Box::new(CommonJsTransformer::new(interner)),
        ModuleFormat::Amd => Box::new(AmdTransformer::new(interner)),
        ModuleFormat::Umd => Box::new(UmdTransformer::new(interner)),
        ModuleFormat::System | ModuleFormat::EsNext => {
            // System and ESNext use CommonJS as fallback for now
            Box::new(CommonJsTransformer::new(interner))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_options_default() {
        let options = TransformOptions::default();
        assert_eq!(options.format, ModuleFormat::CommonJs);
        assert!(options.strict);
        assert!(options.es_module_interop);
    }

    #[test]
    fn test_module_format_default() {
        let format = ModuleFormat::default();
        assert_eq!(format, ModuleFormat::CommonJs);
    }

    #[test]
    fn test_commonjs_generate_import_namespace() {
        let interner = StringInterner::new();
        let transformer = CommonJsTransformer::new(&interner);

        let import = TransformedImport {
            local_name: "fs".to_string(),
            module_specifier: "fs".to_string(),
            is_namespace: true,
            is_default: false,
            named_imports: HashMap::new(),
        };

        let output = transformer.generate_import(&import);
        assert!(output.contains("__importStar"));
        assert!(output.contains("require(\"fs\")"));
    }

    #[test]
    fn test_commonjs_generate_import_default() {
        let interner = StringInterner::new();
        let transformer = CommonJsTransformer::new(&interner);

        let import = TransformedImport {
            local_name: "React".to_string(),
            module_specifier: "react".to_string(),
            is_namespace: false,
            is_default: true,
            named_imports: HashMap::new(),
        };

        let output = transformer.generate_import(&import);
        assert!(output.contains("__importDefault"));
        assert!(output.contains("require(\"react\")"));
    }

    #[test]
    fn test_commonjs_generate_import_named() {
        let interner = StringInterner::new();
        let transformer = CommonJsTransformer::new(&interner);

        let mut named = HashMap::new();
        named.insert("useState".to_string(), "useState".to_string());
        named.insert("effect".to_string(), "useEffect".to_string());

        let import = TransformedImport {
            local_name: String::new(),
            module_specifier: "react".to_string(),
            is_namespace: false,
            is_default: false,
            named_imports: named,
        };

        let output = transformer.generate_import(&import);
        assert!(output.contains("require(\"react\")"));
    }

    #[test]
    fn test_commonjs_generate_export() {
        let interner = StringInterner::new();
        let transformer = CommonJsTransformer::new(&interner);

        let export = TransformedExport {
            exported_name: "foo".to_string(),
            local_name: Some("bar".to_string()),
            is_default: false,
            is_re_export: false,
            re_export_module: None,
        };

        let output = transformer.generate_export(&export);
        assert_eq!(output, "exports.foo = bar;");
    }

    #[test]
    fn test_commonjs_generate_default_export() {
        let interner = StringInterner::new();
        let transformer = CommonJsTransformer::new(&interner);

        let export = TransformedExport {
            exported_name: "default".to_string(),
            local_name: Some("MyComponent".to_string()),
            is_default: true,
            is_re_export: false,
            re_export_module: None,
        };

        let output = transformer.generate_export(&export);
        assert_eq!(output, "exports.default = MyComponent;");
    }

    #[test]
    fn test_commonjs_generate_helpers() {
        let interner = StringInterner::new();
        let transformer = CommonJsTransformer::new(&interner);

        let helpers = vec![HelperFunction::ImportDefault, HelperFunction::ImportStar];
        let output = transformer.generate_helpers(&helpers);

        assert!(output.contains("__importDefault"));
        assert!(output.contains("__importStar"));
        assert!(output.contains("__esModule"));
    }

    #[test]
    fn test_create_transformer() {
        let interner = StringInterner::new();

        let cjs = create_transformer(ModuleFormat::CommonJs, &interner);
        let amd = create_transformer(ModuleFormat::Amd, &interner);
        let umd = create_transformer(ModuleFormat::Umd, &interner);

        // Just verify they can be created without panicking
        let _ = cjs;
        let _ = amd;
        let _ = umd;
    }
}
