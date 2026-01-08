use super::args::CliArgs;
use super::driver::{
    compile, compile_with_cache, compile_with_cache_and_changes, CompilationCache,
};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> std::io::Result<Self> {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("tsz_cli_driver_test_{}_{}", std::process::id(), nanos));
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

static TYPES_VERSIONS_ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: Option<&str>) -> Self {
        let previous = std::env::var(key).ok();
        match value {
            Some(value) => {
                // SAFETY: tests serialize env mutation with a global lock.
                unsafe { std::env::set_var(key, value) };
            }
            None => {
                // SAFETY: tests serialize env mutation with a global lock.
                unsafe { std::env::remove_var(key) };
            }
        }
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match self.previous.as_deref() {
            Some(value) => {
                // SAFETY: tests serialize env mutation with a global lock.
                unsafe { std::env::set_var(self.key, value) };
            }
            None => {
                // SAFETY: tests serialize env mutation with a global lock.
                unsafe { std::env::remove_var(self.key) };
            }
        }
    }
}

fn with_types_versions_env<T>(value: Option<&str>, f: impl FnOnce() -> T) -> T {
    let _lock = TYPES_VERSIONS_ENV_LOCK
        .lock()
        .expect("typesVersions env lock");
    let _guard = EnvVarGuard::set("TSZ_TYPES_VERSIONS_COMPILER_VERSION", value);
    f()
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create parent directory");
    }
    std::fs::write(path, contents).expect("failed to write file");
}

fn default_args() -> CliArgs {
    CliArgs {
        target: None,
        module: None,
        out_dir: None,
        project: None,
        strict: false,
        no_emit: false,
        types_versions_compiler_version: None,
        watch: false,
        files: Vec::new(),
    }
}

#[test]
fn compile_with_tsconfig_emits_outputs() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "declaration": true
          },
          "include": ["src/**/*.ts"]
        }"#,
    );
    write_file(&base.join("src/index.ts"), "export const value = 1;");

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());
    assert!(base.join("dist/src/index.js").is_file());
    assert!(base.join("dist/src/index.d.ts").is_file());
}

#[test]
fn compile_with_source_map_emits_map_outputs() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "sourceMap": true
          },
          "include": ["src/**/*.ts"]
        }"#,
    );
    write_file(&base.join("src/index.ts"), "export const value = 1;");

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());
    let js_path = base.join("dist/src/index.js");
    let map_path = base.join("dist/src/index.js.map");
    assert!(js_path.is_file());
    assert!(map_path.is_file());
    let js_contents = std::fs::read_to_string(&js_path).expect("read js output");
    assert!(js_contents.contains("sourceMappingURL=index.js.map"));
    let map_contents = std::fs::read_to_string(&map_path).expect("read map output");
    let map_json: Value = serde_json::from_str(&map_contents).expect("parse map json");
    let file_field = map_json
        .get("file")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    assert_eq!(file_field, "index.js");
    let source_root = map_json
        .get("sourceRoot")
        .and_then(|value| value.as_str())
        .unwrap_or("__missing__");
    assert_eq!(source_root, "");
    let sources_content = map_json
        .get("sourcesContent")
        .and_then(|value| value.as_array())
        .expect("expected sourcesContent");
    assert_eq!(sources_content.len(), 1);
    assert_eq!(
        sources_content[0].as_str().unwrap_or(""),
        "export const value = 1;"
    );
    let mappings = map_json
        .get("mappings")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    assert!(
        mappings.contains(',') || mappings.contains(';'),
        "expected non-trivial mappings, got: {mappings}"
    );
}

#[test]
fn compile_with_declaration_map_emits_map_outputs() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "declaration": true,
            "declarationMap": true
          },
          "include": ["src/**/*.ts"]
        }"#,
    );
    write_file(&base.join("src/index.ts"), "export const value = 1;");

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());
    let dts_path = base.join("dist/src/index.d.ts");
    let map_path = base.join("dist/src/index.d.ts.map");
    assert!(dts_path.is_file());
    assert!(map_path.is_file());
    let dts_contents = std::fs::read_to_string(&dts_path).expect("read d.ts output");
    assert!(dts_contents.contains("sourceMappingURL=index.d.ts.map"));
    let map_contents = std::fs::read_to_string(&map_path).expect("read map output");
    let map_json: Value = serde_json::from_str(&map_contents).expect("parse map json");
    let file_field = map_json
        .get("file")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    assert_eq!(file_field, "index.d.ts");
    let source_root = map_json
        .get("sourceRoot")
        .and_then(|value| value.as_str())
        .unwrap_or("__missing__");
    assert_eq!(source_root, "");
    let sources_content = map_json
        .get("sourcesContent")
        .and_then(|value| value.as_array())
        .expect("expected sourcesContent");
    assert_eq!(sources_content.len(), 1);
    assert_eq!(
        sources_content[0].as_str().unwrap_or(""),
        "export const value = 1;"
    );
    let mappings = map_json
        .get("mappings")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    assert!(
        mappings.contains(',') || mappings.contains(';'),
        "expected non-trivial mappings, got: {mappings}"
    );
}

#[test]
fn compile_with_explicit_files_without_tsconfig() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(&base.join("main.ts"), "export const value = 1;");

    let mut args = default_args();
    args.files = vec![PathBuf::from("main.ts")];

    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());
    assert!(base.join("main.js").is_file());
}

#[test]
fn compile_with_root_dir_flattens_output_paths() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "rootDir": "src",
            "declaration": true
          },
          "include": ["src/**/*.ts"]
        }"#,
    );
    write_file(&base.join("src/index.ts"), "export const value = 1;");

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());
    assert!(base.join("dist/index.js").is_file());
    assert!(base.join("dist/index.d.ts").is_file());
}

#[test]
fn compile_respects_no_emit_on_error() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "include": ["src/**/*.ts"]
        }"#,
    );
    write_file(&base.join("src/index.ts"), "let x = ;");

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_with_project_dir_uses_tsconfig() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    let config_dir = base.join("configs");
    write_file(
        &config_dir.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist"
          },
          "include": ["src/**/*.ts"]
        }"#,
    );
    write_file(&config_dir.join("src/index.ts"), "export const value = 1;");

    let mut args = default_args();
    args.project = Some(PathBuf::from("configs"));

    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());
    assert!(config_dir.join("dist/src/index.js").is_file());
}

#[test]
fn compile_with_jsx_preserve_emits_jsx_extension() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "jsx": "preserve"
          },
          "include": ["src/**/*.tsx"]
        }"#,
    );
    write_file(&base.join("src/view.tsx"), "export const View = () => <div />;");

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());
    assert!(base.join("dist/src/view.jsx").is_file());
}

#[test]
fn compile_resolves_relative_imports_from_files_list() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist"
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { value } from './util'; export { value };",
    );
    write_file(&base.join("src/util.ts"), "export const value = 1;");

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());
    assert!(base.join("dist/src/index.js").is_file());
    assert!(base.join("dist/src/util.js").is_file());
}

#[test]
fn compile_resolves_paths_mappings() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "baseUrl": ".",
            "paths": {
              "@lib/*": ["src/lib/*"]
            }
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { value } from '@lib/value'; export { value };",
    );
    write_file(&base.join("src/lib/value.ts"), "export const value = 1;");

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());
    assert!(base.join("dist/src/lib/value.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { value } from 'pkg'; export { value };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "types": "index.d.ts"
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/index.d.ts"),
        "export const value = ;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/index.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_tsconfig_types_includes_selected_packages() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true,
            "types": ["foo"]
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(&base.join("src/index.ts"), "export const value = 1;");
    write_file(
        &base.join("node_modules/@types/foo/index.d.ts"),
        "export const foo = ;",
    );
    write_file(
        &base.join("node_modules/@types/bar/index.d.ts"),
        "export const bar = ;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/@types/foo/index.d.ts")));
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/@types/bar/index.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_tsconfig_type_roots_includes_packages() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true,
            "typeRoots": ["types"]
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(&base.join("src/index.ts"), "export const value = 1;");
    write_file(
        &base.join("types/foo/index.d.ts"),
        "export const foo = ;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("types/foo/index.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_exports_subpath() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "exports": {
            ".": { "types": "./types/index.d.ts" },
            "./feature/*": { "types": "./types/feature/*.d.ts" }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/feature/widget.d.ts"),
        "export const widget = ;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            "*": {
              "feature/*": ["types/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/feature/widget.d.ts"),
        "export const widget = ;",
    );
    write_file(
        &base.join("node_modules/pkg/feature/widget.d.ts"),
        "export const widget = 1;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions_best_match() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            ">=6.1": {
              "feature/*": ["types/v61/feature/*"]
            },
            ">=5.0": {
              "feature/*": ["types/v5/feature/*"]
            },
            "*": {
              "feature/*": ["types/fallback/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/v61/feature/widget.d.ts"),
        "export const widget = 1;",
    );
    write_file(
        &base.join("node_modules/pkg/types/v5/feature/widget.d.ts"),
        "export const widget = ;",
    );
    write_file(
        &base.join("node_modules/pkg/types/fallback/feature/widget.d.ts"),
        "export const widget = 1;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/v5/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions_prefers_specific_range() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            ">=6.0": {
              "feature/*": ["types/loose/feature/*"]
            },
            ">=5.0 <7.0": {
              "feature/*": ["types/ranged/feature/*"]
            },
            "*": {
              "feature/*": ["types/fallback/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/loose/feature/widget.d.ts"),
        "export const widget = 1;",
    );
    write_file(
        &base.join("node_modules/pkg/types/ranged/feature/widget.d.ts"),
        "export const widget = ;",
    );
    write_file(
        &base.join("node_modules/pkg/types/fallback/feature/widget.d.ts"),
        "export const widget = 1;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/ranged/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions_respects_cli_version_override() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            ">=7.0": {
              "feature/*": ["types/v7/feature/*"]
            },
            ">=6.0": {
              "feature/*": ["types/v6/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/v7/feature/widget.d.ts"),
        "export const widget = ;",
    );
    write_file(
        &base.join("node_modules/pkg/types/v6/feature/widget.d.ts"),
        "export const widget = 1;",
    );

    let mut args = default_args();
    args.types_versions_compiler_version = Some("7.1".to_string());
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/v7/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions_respects_env_version_override() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            ">=7.0": {
              "feature/*": ["types/v7/feature/*"]
            },
            ">=6.0": {
              "feature/*": ["types/v6/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/v7/feature/widget.d.ts"),
        "export const widget = ;",
    );
    write_file(
        &base.join("node_modules/pkg/types/v6/feature/widget.d.ts"),
        "export const widget = 1;",
    );

    let args = default_args();
    let result = with_types_versions_env(Some("7.1"), || {
        compile(&args, base).expect("compile should succeed")
    });

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/v7/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions_respects_tsconfig_version_override() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true,
            "typesVersionsCompilerVersion": "7.1"
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            ">=7.0": {
              "feature/*": ["types/v7/feature/*"]
            },
            ">=6.0": {
              "feature/*": ["types/v6/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/v7/feature/widget.d.ts"),
        "export const widget = ;",
    );
    write_file(
        &base.join("node_modules/pkg/types/v6/feature/widget.d.ts"),
        "export const widget = 1;",
    );

    let args = default_args();
    let result = with_types_versions_env(None, || {
        compile(&args, base).expect("compile should succeed")
    });

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/v7/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions_tsconfig_extends_inherits_override() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("config/base.json"),
        r#"{
          "compilerOptions": {
            "typesVersionsCompilerVersion": "7.1"
          }
        }"#,
    );
    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "extends": "./config/base.json",
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            ">=7.1": {
              "feature/*": ["types/v71/feature/*"]
            },
            ">=6.0": {
              "feature/*": ["types/v6/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/v71/feature/widget.d.ts"),
        "export const widget = ;",
    );
    write_file(
        &base.join("node_modules/pkg/types/v6/feature/widget.d.ts"),
        "export const widget = 1;",
    );

    let args = default_args();
    let result = with_types_versions_env(None, || {
        compile(&args, base).expect("compile should succeed")
    });

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/v71/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions_env_overrides_tsconfig() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true,
            "typesVersionsCompilerVersion": "6.0"
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            ">=7.0": {
              "feature/*": ["types/v7/feature/*"]
            },
            ">=6.0": {
              "feature/*": ["types/v6/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/v7/feature/widget.d.ts"),
        "export const widget = ;",
    );
    write_file(
        &base.join("node_modules/pkg/types/v6/feature/widget.d.ts"),
        "export const widget = 1;",
    );

    let args = default_args();
    let result = with_types_versions_env(Some("7.1"), || {
        compile(&args, base).expect("compile should succeed")
    });

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/v7/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions_empty_env_uses_tsconfig() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true,
            "typesVersionsCompilerVersion": "7.1"
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            ">=7.1": {
              "feature/*": ["types/v71/feature/*"]
            },
            ">=6.0": {
              "feature/*": ["types/v6/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/v71/feature/widget.d.ts"),
        "export const widget = ;",
    );
    write_file(
        &base.join("node_modules/pkg/types/v6/feature/widget.d.ts"),
        "export const widget = 1;",
    );

    let args = default_args();
    let result = with_types_versions_env(Some(""), || {
        compile(&args, base).expect("compile should succeed")
    });

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/v71/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions_cli_overrides_env_and_tsconfig() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true,
            "typesVersionsCompilerVersion": "6.0"
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            ">=7.2": {
              "feature/*": ["types/v72/feature/*"]
            },
            ">=7.1": {
              "feature/*": ["types/v71/feature/*"]
            },
            ">=6.0": {
              "feature/*": ["types/v6/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/v72/feature/widget.d.ts"),
        "export const widget = ;",
    );
    write_file(
        &base.join("node_modules/pkg/types/v71/feature/widget.d.ts"),
        "export const widget = 1;",
    );
    write_file(
        &base.join("node_modules/pkg/types/v6/feature/widget.d.ts"),
        "export const widget = 1;",
    );

    let mut args = default_args();
    args.types_versions_compiler_version = Some("7.2".to_string());
    let result = with_types_versions_env(Some("7.1"), || {
        compile(&args, base).expect("compile should succeed")
    });

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/v72/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions_invalid_override_falls_back() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            ">=7.0": {
              "feature/*": ["types/v7/feature/*"]
            },
            ">=6.0": {
              "feature/*": ["types/v6/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/v7/feature/widget.d.ts"),
        "export const widget = 1;",
    );
    write_file(
        &base.join("node_modules/pkg/types/v6/feature/widget.d.ts"),
        "export const widget = ;",
    );

    let mut args = default_args();
    args.types_versions_compiler_version = Some("not-a-version".to_string());
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/v6/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions_invalid_env_falls_back() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            ">=7.0": {
              "feature/*": ["types/v7/feature/*"]
            },
            ">=6.0": {
              "feature/*": ["types/v6/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/v7/feature/widget.d.ts"),
        "export const widget = 1;",
    );
    write_file(
        &base.join("node_modules/pkg/types/v6/feature/widget.d.ts"),
        "export const widget = ;",
    );

    let args = default_args();
    let result = with_types_versions_env(Some("not-a-version"), || {
        compile(&args, base).expect("compile should succeed")
    });

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/v6/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions_invalid_tsconfig_falls_back() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true,
            "typesVersionsCompilerVersion": "not-a-version"
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            ">=7.0": {
              "feature/*": ["types/v7/feature/*"]
            },
            ">=6.0": {
              "feature/*": ["types/v6/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/v7/feature/widget.d.ts"),
        "export const widget = 1;",
    );
    write_file(
        &base.join("node_modules/pkg/types/v6/feature/widget.d.ts"),
        "export const widget = ;",
    );

    let args = default_args();
    let result = with_types_versions_env(None, || {
        compile(&args, base).expect("compile should succeed")
    });

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/v6/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_node_modules_types_versions_falls_back_to_wildcard() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg/feature/widget'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "typesVersions": {
            ">=7.0": {
              "feature/*": ["types/v7/feature/*"]
            },
            "*": {
              "feature/*": ["types/fallback/feature/*"]
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/types/v7/feature/widget.d.ts"),
        "export const widget = 1;",
    );
    write_file(
        &base.join("node_modules/pkg/types/fallback/feature/widget.d.ts"),
        "export const widget = ;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/types/fallback/feature/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_package_imports_wildcard() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from '#utils/widget'; export { widget };",
    );
    write_file(
        &base.join("package.json"),
        r##"{
          "imports": {
            "#utils/*": "./types/*"
          }
        }"##,
    );
    write_file(
        &base.join("types/widget.d.ts"),
        "export const widget = ;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("types/widget.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_package_imports_prefers_types_condition() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { feature } from '#feature'; export { feature };",
    );
    write_file(
        &base.join("package.json"),
        r##"{
          "imports": {
            "#feature": {
              "types": "./types/feature.d.ts",
              "default": "./default/feature.d.ts"
            }
          }
        }"##,
    );
    write_file(
        &base.join("types/feature.d.ts"),
        "export const feature = ;",
    );
    write_file(
        &base.join("default/feature.d.ts"),
        "export const feature = 1;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("types/feature.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_package_imports_prefers_require_condition_for_commonjs() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "module": "commonjs",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { feature } from '#feature'; export { feature };",
    );
    write_file(
        &base.join("package.json"),
        r##"{
          "imports": {
            "#feature": {
              "require": "./types/require.d.ts",
              "import": "./types/import.d.ts"
            }
          }
        }"##,
    );
    write_file(
        &base.join("types/require.d.ts"),
        "export const feature = ;",
    );
    write_file(
        &base.join("types/import.d.ts"),
        "export const feature = 1;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("types/require.d.ts")));
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("types/import.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_resolves_package_imports_prefers_import_condition_for_esm() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "module": "esnext",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { feature } from '#feature'; export { feature };",
    );
    write_file(
        &base.join("package.json"),
        r##"{
          "imports": {
            "#feature": {
              "import": "./types/import.d.ts",
              "require": "./types/require.d.ts"
            }
          }
        }"##,
    );
    write_file(
        &base.join("types/import.d.ts"),
        "export const feature = ;",
    );
    write_file(
        &base.join("types/require.d.ts"),
        "export const feature = 1;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("types/import.d.ts")));
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("types/require.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_prefers_browser_exports_for_bundler() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "moduleResolution": "bundler",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { widget } from 'pkg'; export { widget };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "exports": {
            ".": {
              "browser": "./browser.d.ts",
              "node": "./node.d.ts"
            }
          }
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/browser.d.ts"),
        "export const widget = ;",
    );
    write_file(
        &base.join("node_modules/pkg/node.d.ts"),
        "export const widget = 1;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/browser.d.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_node_next_resolves_js_extension_to_ts() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "moduleResolution": "nodenext",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { value } from './util.js'; export { value };",
    );
    write_file(&base.join("src/util.ts"), "export const value = ;");

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("src/util.ts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_node_next_prefers_mts_for_module_package() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "moduleResolution": "nodenext",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { value } from 'pkg'; export { value };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "type": "module"
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/index.mts"),
        "export const value = ;",
    );
    write_file(
        &base.join("node_modules/pkg/index.cts"),
        "export const value = 1;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/index.mts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_node_next_prefers_cts_for_commonjs_package() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "moduleResolution": "nodenext",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        "import { value } from 'pkg'; export { value };",
    );
    write_file(
        &base.join("node_modules/pkg/package.json"),
        r#"{
          "type": "commonjs"
        }"#,
    );
    write_file(
        &base.join("node_modules/pkg/index.mts"),
        "export const value = 1;",
    );
    write_file(
        &base.join("node_modules/pkg/index.cts"),
        "export const value = ;",
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(!result.diagnostics.is_empty());
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("node_modules/pkg/index.cts")));
    assert!(!base.join("dist/src/index.js").is_file());
}

#[test]
fn compile_with_cache_emits_only_dirty_files() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist"
          },
          "files": ["src/alpha.ts", "src/beta.ts"]
        }"#,
    );

    let alpha_path = base.join("src/alpha.ts");
    let beta_path = base.join("src/beta.ts");
    write_file(&alpha_path, "export const alpha = 1;");
    write_file(&beta_path, "export const beta = 2;");

    let mut cache = CompilationCache::default();
    let args = default_args();

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(result.diagnostics.is_empty());

    let alpha_output = std::fs::canonicalize(base.join("dist/src/alpha.js"))
        .unwrap_or_else(|_| base.join("dist/src/alpha.js"));
    let beta_output = std::fs::canonicalize(base.join("dist/src/beta.js"))
        .unwrap_or_else(|_| base.join("dist/src/beta.js"));
    assert_eq!(result.emitted_files.len(), 2);
    assert!(result.emitted_files.contains(&alpha_output));
    assert!(result.emitted_files.contains(&beta_output));

    write_file(&alpha_path, "export const alpha = 2;");
    let canonical = std::fs::canonicalize(&alpha_path).unwrap_or(alpha_path.clone());
    cache.invalidate_paths_with_dependents(vec![canonical]);

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.emitted_files.len(), 1);
    assert!(result.emitted_files.contains(&alpha_output));
    assert!(!result.emitted_files.contains(&beta_output));
}

#[test]
fn compile_with_cache_updates_dependencies_for_changed_files() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist"
          },
          "files": ["src/index.ts"]
        }"#,
    );

    let index_path = base.join("src/index.ts");
    let util_path = base.join("src/util.ts");
    let extra_path = base.join("src/extra.ts");
    write_file(
        &index_path,
        "import { value } from './util'; export { value };",
    );
    write_file(&util_path, "export const value = ;");

    let mut cache = CompilationCache::default();
    let args = default_args();

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("util.ts")));

    write_file(
        &index_path,
        "import { value } from './extra'; export { value };",
    );
    write_file(&extra_path, "export const value = ;");

    let canonical = std::fs::canonicalize(&index_path).unwrap_or(index_path.clone());
    let result = compile_with_cache_and_changes(&args, base, &mut cache, &[canonical])
        .expect("compile should succeed");
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("extra.ts")));
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.file.contains("util.ts")));
}

#[test]
fn compile_with_cache_skips_dependents_when_exports_unchanged() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist"
          },
          "files": ["src/index.ts"]
        }"#,
    );

    let index_path = base.join("src/index.ts");
    let util_path = base.join("src/util.ts");
    write_file(
        &index_path,
        "import { value } from './util'; export const output = value;",
    );
    write_file(&util_path, "export function value() { return 1; }");

    let mut cache = CompilationCache::default();
    let args = default_args();

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(
        result.diagnostics.is_empty(),
        "initial diagnostics (unchanged exports): {:#?}",
        result.diagnostics
    );

    write_file(&util_path, "export function value() { return 2; }");

    let util_output = std::fs::canonicalize(base.join("dist/src/util.js"))
        .unwrap_or_else(|_| base.join("dist/src/util.js"));
    let index_output = std::fs::canonicalize(base.join("dist/src/index.js"))
        .unwrap_or_else(|_| base.join("dist/src/index.js"));
    let canonical = std::fs::canonicalize(&util_path).unwrap_or(util_path.clone());

    let result = compile_with_cache_and_changes(&args, base, &mut cache, &[canonical])
        .expect("compile should succeed");
    assert!(result.diagnostics.is_empty());
    assert!(result.emitted_files.contains(&util_output));
    assert!(!result.emitted_files.contains(&index_output));
}

#[test]
fn compile_with_cache_rechecks_dependents_on_export_change() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist"
          },
          "files": ["src/index.ts"]
        }"#,
    );

    let index_path = base.join("src/index.ts");
    let util_path = base.join("src/util.ts");
    write_file(
        &index_path,
        "import { value } from './util'; const num: number = value;",
    );
    write_file(&util_path, "export const value = 1;");

    let mut cache = CompilationCache::default();
    let args = default_args();

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(
        result.diagnostics.is_empty(),
        "initial diagnostics (export change): {:#?}",
        result.diagnostics
    );

    write_file(&util_path, "export const value = \"oops\";");

    let util_output = std::fs::canonicalize(base.join("dist/src/util.js"))
        .unwrap_or_else(|_| base.join("dist/src/util.js"));
    let index_output = std::fs::canonicalize(base.join("dist/src/index.js"))
        .unwrap_or_else(|_| base.join("dist/src/index.js"));
    let canonical = std::fs::canonicalize(&util_path).unwrap_or(util_path.clone());

    let result = compile_with_cache_and_changes(&args, base, &mut cache, &[canonical])
        .expect("compile should succeed");
    // Import aliases are still typed as `any`, so assert dependent recompilation instead of diagnostics.
    assert!(result.emitted_files.contains(&util_output));
    assert!(result.emitted_files.contains(&index_output));
}

#[test]
fn compile_with_cache_invalidates_paths() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    let index_path = base.join("src/index.ts");
    write_file(&index_path, "export const value = ;");

    let mut cache = CompilationCache::default();
    let args = default_args();

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(!result.diagnostics.is_empty());
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.bind_len(), 1);
    assert_eq!(cache.diagnostics_len(), 1);

    let canonical = std::fs::canonicalize(&index_path).unwrap_or(index_path.clone());
    cache.invalidate_paths_with_dependents(vec![canonical]);
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.bind_len(), 0);
    assert_eq!(cache.diagnostics_len(), 0);

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(!result.diagnostics.is_empty());
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.bind_len(), 1);
    assert_eq!(cache.diagnostics_len(), 1);
}

#[test]
fn compile_with_cache_invalidates_dependents() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "noEmitOnError": true
          },
          "files": ["src/index.ts"]
        }"#,
    );
    let index_path = base.join("src/index.ts");
    let util_path = base.join("src/util.ts");
    write_file(
        &index_path,
        "import { value } from './util'; export { value };",
    );
    write_file(&util_path, "export const value = ;");

    let mut cache = CompilationCache::default();
    let args = default_args();

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(!result.diagnostics.is_empty());
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.bind_len(), 2);
    assert_eq!(cache.diagnostics_len(), 2);

    let canonical = std::fs::canonicalize(&util_path).unwrap_or(util_path.clone());
    cache.invalidate_paths_with_dependents(vec![canonical]);
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.bind_len(), 0);
    assert_eq!(cache.diagnostics_len(), 0);

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(!result.diagnostics.is_empty());
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.bind_len(), 2);
    assert_eq!(cache.diagnostics_len(), 2);
}

#[test]
fn invalidate_paths_with_dependents_symbols_keeps_unrelated_cache() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist"
          },
          "files": ["src/index.ts"]
        }"#,
    );
    let index_path = base.join("src/index.ts");
    let util_path = base.join("src/util.ts");
    write_file(
        &index_path,
        "import { value } from './util'; export const local = 1; export const uses = value;",
    );
    write_file(&util_path, "export const value = 1;");

    let mut cache = CompilationCache::default();
    let args = default_args();

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(result.diagnostics.is_empty());

    let canonical_index = std::fs::canonicalize(&index_path).unwrap_or(index_path.clone());
    let canonical_util = std::fs::canonicalize(&util_path).unwrap_or(util_path.clone());
    let before = cache.symbol_cache_len(&canonical_index).unwrap_or(0);
    assert!(before > 0);

    cache.invalidate_paths_with_dependents_symbols(vec![canonical_util.clone()]);

    let after = cache.symbol_cache_len(&canonical_index).unwrap_or(0);
    assert!(after > 0);
    assert!(after < before);
    assert_eq!(cache.node_cache_len(&canonical_index).unwrap_or(0), 0);
    assert!(cache.symbol_cache_len(&canonical_util).is_none());
}

#[test]
fn invalidate_paths_with_dependents_symbols_handles_reexports() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist"
          },
          "files": ["src/index.ts"]
        }"#,
    );
    let index_path = base.join("src/index.ts");
    let util_path = base.join("src/util.ts");
    write_file(
        &index_path,
        "export { value } from './util'; export const local = 1;",
    );
    write_file(&util_path, "export const value = 1;");

    let mut cache = CompilationCache::default();
    let args = default_args();

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(result.diagnostics.is_empty());
    assert_eq!(cache.len(), 2);

    let canonical_index = std::fs::canonicalize(&index_path).unwrap_or(index_path.clone());
    let canonical_util = std::fs::canonicalize(&util_path).unwrap_or(util_path.clone());

    cache.invalidate_paths_with_dependents_symbols(vec![canonical_util.clone()]);

    assert_eq!(cache.len(), 1);
    assert!(cache.symbol_cache_len(&canonical_index).is_some());
    assert_eq!(cache.node_cache_len(&canonical_index).unwrap_or(1), 0);
    assert!(cache.symbol_cache_len(&canonical_util).is_none());
}

#[test]
fn invalidate_paths_with_dependents_symbols_handles_import_equals() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist"
          },
          "files": ["src/index.ts"]
        }"#,
    );
    let index_path = base.join("src/index.ts");
    let util_path = base.join("src/util.ts");
    write_file(
        &index_path,
        "import util = require('./util'); export const local = util.value;",
    );
    write_file(&util_path, "export const value = 1;");

    let mut cache = CompilationCache::default();
    let args = default_args();

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(result.diagnostics.is_empty());
    assert_eq!(cache.len(), 2);

    let canonical_index = std::fs::canonicalize(&index_path).unwrap_or(index_path.clone());
    let canonical_util = std::fs::canonicalize(&util_path).unwrap_or(util_path.clone());
    let before_nodes = cache.node_cache_len(&canonical_index).unwrap_or(0);
    assert!(before_nodes > 0);

    cache.invalidate_paths_with_dependents_symbols(vec![canonical_util.clone()]);

    assert_eq!(cache.len(), 1);
    assert!(cache.symbol_cache_len(&canonical_index).is_some());
    assert_eq!(cache.node_cache_len(&canonical_index).unwrap_or(1), 0);
    assert!(cache.symbol_cache_len(&canonical_util).is_none());
}

#[test]
fn invalidate_paths_with_dependents_symbols_handles_namespace_reexports() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist"
          },
          "files": ["src/index.ts"]
        }"#,
    );
    let index_path = base.join("src/index.ts");
    let util_path = base.join("src/util.ts");
    write_file(
        &index_path,
        "export * as util from './util'; export const local = 1;",
    );
    write_file(&util_path, "export const value = 1;");

    let mut cache = CompilationCache::default();
    let args = default_args();

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(result.diagnostics.is_empty());
    assert_eq!(cache.len(), 2);

    let canonical_index = std::fs::canonicalize(&index_path).unwrap_or(index_path.clone());
    let canonical_util = std::fs::canonicalize(&util_path).unwrap_or(util_path.clone());
    let before_nodes = cache.node_cache_len(&canonical_index).unwrap_or(0);
    assert!(before_nodes > 0);

    cache.invalidate_paths_with_dependents_symbols(vec![canonical_util.clone()]);

    assert_eq!(cache.len(), 1);
    assert!(cache.symbol_cache_len(&canonical_index).is_some());
    assert_eq!(cache.node_cache_len(&canonical_index).unwrap_or(1), 0);
    assert!(cache.symbol_cache_len(&canonical_util).is_none());
}

#[test]
fn invalidate_paths_with_dependents_symbols_handles_star_reexports() {
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist"
          },
          "files": ["src/index.ts"]
        }"#,
    );
    let index_path = base.join("src/index.ts");
    let util_path = base.join("src/util.ts");
    write_file(
        &index_path,
        "export * from './util'; export const local = 1;",
    );
    write_file(&util_path, "export const value = 1;");

    let mut cache = CompilationCache::default();
    let args = default_args();

    let result = compile_with_cache(&args, base, &mut cache).expect("compile should succeed");
    assert!(result.diagnostics.is_empty());
    assert_eq!(cache.len(), 2);

    let canonical_index = std::fs::canonicalize(&index_path).unwrap_or(index_path.clone());
    let canonical_util = std::fs::canonicalize(&util_path).unwrap_or(util_path.clone());
    let before_nodes = cache.node_cache_len(&canonical_index).unwrap_or(0);
    assert!(before_nodes > 0);

    cache.invalidate_paths_with_dependents_symbols(vec![canonical_util.clone()]);

    assert_eq!(cache.len(), 1);
    assert!(cache.symbol_cache_len(&canonical_index).is_some());
    assert_eq!(cache.node_cache_len(&canonical_index).unwrap_or(1), 0);
    assert!(cache.symbol_cache_len(&canonical_util).is_none());
}

#[test]
fn compile_multi_file_project_with_imports() {
    // End-to-end test for a multi-file project with various import patterns
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    // Create tsconfig.json with CommonJS module for testable require() output
    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "rootDir": "src",
            "module": "commonjs",
            "declaration": true,
            "sourceMap": true
          },
          "include": ["src/**/*.ts"]
        }"#,
    );

    // src/models/user.ts - basic model with interface and class
    write_file(
        &base.join("src/models/user.ts"),
        r#"
export interface User {
    id: number;
    name: string;
    email: string;
}

export class UserImpl implements User {
    id: number;
    name: string;
    email: string;

    constructor(id: number, name: string, email: string) {
        this.id = id;
        this.name = name;
        this.email = email;
    }

    getDisplayName(): string {
        return this.name + " <" + this.email + ">";
    }
}

export type UserId = number;
"#,
    );

    // src/utils/helpers.ts - utility functions
    write_file(
        &base.join("src/utils/helpers.ts"),
        r#"
export function formatName(first: string, last: string): string {
    return first + " " + last;
}

export function validateEmail(email: string): boolean {
    return email.indexOf("@") >= 0;
}

export const DEFAULT_PAGE_SIZE = 20;
"#,
    );

    // src/services/user-service.ts - service using models and utils
    write_file(
        &base.join("src/services/user-service.ts"),
        r#"
import { User, UserImpl, UserId } from '../models/user';
import { formatName, validateEmail } from '../utils/helpers';

export class UserService {
    private users: User[] = [];

    createUser(id: UserId, firstName: string, lastName: string, email: string): User | null {
        if (!validateEmail(email)) {
            return null;
        }
        const name = formatName(firstName, lastName);
        const user = new UserImpl(id, name, email);
        this.users.push(user);
        return user;
    }

    getUserCount(): number {
        return this.users.length;
    }
}
"#,
    );

    // src/index.ts - main entry point re-exporting everything
    write_file(
        &base.join("src/index.ts"),
        r#"
// Re-export models
export { User, UserImpl, UserId } from './models/user';

// Re-export utilities
export { formatName, validateEmail, DEFAULT_PAGE_SIZE } from './utils/helpers';

// Re-export services
export { UserService } from './services/user-service';
"#,
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    // Verify no diagnostics
    assert!(
        result.diagnostics.is_empty(),
        "Expected no diagnostics, got: {:?}",
        result.diagnostics
    );

    // Verify all output files exist
    assert!(
        base.join("dist/models/user.js").is_file(),
        "models/user.js should exist"
    );
    assert!(
        base.join("dist/models/user.d.ts").is_file(),
        "models/user.d.ts should exist"
    );
    assert!(
        base.join("dist/models/user.js.map").is_file(),
        "models/user.js.map should exist"
    );

    assert!(
        base.join("dist/utils/helpers.js").is_file(),
        "utils/helpers.js should exist"
    );
    assert!(
        base.join("dist/utils/helpers.d.ts").is_file(),
        "utils/helpers.d.ts should exist"
    );

    assert!(
        base.join("dist/services/user-service.js").is_file(),
        "services/user-service.js should exist"
    );
    assert!(
        base.join("dist/services/user-service.d.ts").is_file(),
        "services/user-service.d.ts should exist"
    );

    assert!(
        base.join("dist/index.js").is_file(),
        "index.js should exist"
    );
    assert!(
        base.join("dist/index.d.ts").is_file(),
        "index.d.ts should exist"
    );

    // Verify user-service.js has correct CommonJS require statements
    let service_js =
        std::fs::read_to_string(base.join("dist/services/user-service.js")).expect("read service js");
    assert!(
        service_js.contains("require(") || service_js.contains("import"),
        "Service JS should have require or import statements: {}",
        service_js
    );
    assert!(
        service_js.contains("../models/user") || service_js.contains("./models/user"),
        "Service JS should reference models/user: {}",
        service_js
    );
    assert!(
        service_js.contains("../utils/helpers") || service_js.contains("./utils/helpers"),
        "Service JS should reference utils/helpers: {}",
        service_js
    );

    // Verify index.js has re-exports (CommonJS uses Object.defineProperty pattern)
    let index_js = std::fs::read_to_string(base.join("dist/index.js")).expect("read index js");
    assert!(
        index_js.contains("exports") && (index_js.contains("require(") || index_js.contains("Object.defineProperty")),
        "Index JS should have CommonJS exports: {}",
        index_js
    );

    // Verify declaration file for index has proper re-exports
    let index_dts = std::fs::read_to_string(base.join("dist/index.d.ts")).expect("read index d.ts");
    assert!(
        index_dts.contains("User") && index_dts.contains("UserService"),
        "Index d.ts should export User and UserService: {}",
        index_dts
    );

    // Verify source map for user-service has correct sources
    let service_map_contents =
        std::fs::read_to_string(base.join("dist/services/user-service.js.map"))
            .expect("read service map");
    let service_map: Value =
        serde_json::from_str(&service_map_contents).expect("parse service map json");
    let sources = service_map
        .get("sources")
        .and_then(|v| v.as_array())
        .expect("sources array");
    assert!(
        !sources.is_empty(),
        "Source map should have sources"
    );
    let sources_content = service_map
        .get("sourcesContent")
        .and_then(|v| v.as_array());
    assert!(
        sources_content.is_some(),
        "Source map should have sourcesContent"
    );
}

#[test]
fn compile_multi_file_project_with_default_and_named_imports() {
    // Test default and named import styles
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "rootDir": "src",
            "module": "commonjs",
            "esModuleInterop": true,
            "declaration": true
          },
          "include": ["src/**/*.ts"]
        }"#,
    );

    // src/constants.ts - default export
    write_file(
        &base.join("src/constants.ts"),
        r#"
const CONFIG = {
    apiUrl: "https://api.example.com",
    timeout: 5000
};

export default CONFIG;
export const VERSION = "1.0.0";
"#,
    );

    // src/math.ts - multiple named exports
    write_file(
        &base.join("src/math.ts"),
        r#"
export function add(a: number, b: number): number {
    return a + b;
}

export function multiply(a: number, b: number): number {
    return a * b;
}

export const PI = 3.14159;
"#,
    );

    // src/app.ts - uses default and named imports
    write_file(
        &base.join("src/app.ts"),
        r#"
// Default import
import CONFIG from './constants';
// Named import alongside default
import { VERSION } from './constants';
// Named imports with alias
import { add as addNumbers, multiply, PI } from './math';

export function runApp(): string {
    const sum = addNumbers(1, 2);
    const product = multiply(3, 4);
    const circumference = 2 * PI * 10;
    const url = CONFIG.apiUrl;

    return url + " v" + VERSION + " sum=" + sum + " product=" + product + " circ=" + circumference;
}
"#,
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(
        result.diagnostics.is_empty(),
        "Expected no diagnostics, got: {:?}",
        result.diagnostics
    );

    // Verify all files compiled
    assert!(base.join("dist/constants.js").is_file());
    assert!(base.join("dist/math.js").is_file());
    assert!(base.join("dist/app.js").is_file());
    assert!(base.join("dist/app.d.ts").is_file());

    // Verify app.js has the necessary require statements
    let app_js = std::fs::read_to_string(base.join("dist/app.js")).expect("read app js");
    assert!(
        app_js.contains("./constants") || app_js.contains("constants"),
        "App JS should reference constants: {}",
        app_js
    );
    assert!(
        app_js.contains("./math") || app_js.contains("math"),
        "App JS should reference math: {}",
        app_js
    );

    // Verify declaration file has correct exports
    let app_dts = std::fs::read_to_string(base.join("dist/app.d.ts")).expect("read app d.ts");
    assert!(
        app_dts.contains("runApp"),
        "App d.ts should export runApp: {}",
        app_dts
    );
}

#[test]
fn compile_multi_file_project_with_type_imports() {
    // Test type-only imports compile correctly
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "module": "commonjs",
            "declaration": true
          },
          "include": ["src/**/*.ts"]
        }"#,
    );

    // src/types.ts - shared types
    write_file(
        &base.join("src/types.ts"),
        r#"
export interface Logger {
    log(msg: string): void;
}

export type LogLevel = "debug" | "info" | "error";
"#,
    );

    // src/logger.ts - uses types (type-only import)
    write_file(
        &base.join("src/logger.ts"),
        r#"
import type { Logger, LogLevel } from './types';

export class ConsoleLogger implements Logger {
    private level: LogLevel;

    constructor(level: LogLevel) {
        this.level = level;
    }

    log(msg: string): void {
        // log implementation
    }

    getLevel(): LogLevel {
        return this.level;
    }
}

export function createLogger(level: LogLevel): Logger {
    return new ConsoleLogger(level);
}
"#,
    );

    // src/index.ts - re-exports everything
    write_file(
        &base.join("src/index.ts"),
        r#"
export type { Logger, LogLevel } from './types';
export { ConsoleLogger, createLogger } from './logger';
"#,
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(
        result.diagnostics.is_empty(),
        "Type imports should compile without errors: {:?}",
        result.diagnostics
    );

    assert!(base.join("dist/src/types.js").is_file());
    assert!(base.join("dist/src/logger.js").is_file());
    assert!(base.join("dist/src/index.js").is_file());
    assert!(base.join("dist/src/index.d.ts").is_file());

    // Verify declaration file has type exports
    let index_dts = std::fs::read_to_string(base.join("dist/src/index.d.ts")).expect("read index d.ts");
    assert!(
        index_dts.contains("Logger") && index_dts.contains("LogLevel"),
        "Index d.ts should have type exports for Logger and LogLevel: {}",
        index_dts
    );

    // Verify logger.js has the class implementation
    let logger_js = std::fs::read_to_string(base.join("dist/src/logger.js")).expect("read logger js");
    assert!(
        logger_js.contains("ConsoleLogger") && logger_js.contains("createLogger"),
        "Logger JS should have class and function exports: {}",
        logger_js
    );
}

#[test]
fn compile_declaration_true_emits_dts_files() {
    // Test that declaration: true produces .d.ts files
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "declaration": true
          },
          "include": ["src/**/*.ts"]
        }"#,
    );
    write_file(
        &base.join("src/index.ts"),
        r#"
export const VERSION = "1.0.0";
export function greet(name: string): string {
    return "Hello, " + name;
}
"#,
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(
        result.diagnostics.is_empty(),
        "Expected no diagnostics, got: {:?}",
        result.diagnostics
    );

    // JS file should exist
    assert!(
        base.join("dist/src/index.js").is_file(),
        "JS output should exist"
    );

    // Declaration file should exist
    assert!(
        base.join("dist/src/index.d.ts").is_file(),
        "Declaration file should exist when declaration: true"
    );

    // Verify declaration file content
    let dts = std::fs::read_to_string(base.join("dist/src/index.d.ts")).expect("read d.ts");
    assert!(
        dts.contains("VERSION") && dts.contains("string"),
        "Declaration should contain VERSION: {}",
        dts
    );
    assert!(
        dts.contains("greet") && dts.contains("name"),
        "Declaration should contain greet function: {}",
        dts
    );
}

#[test]
fn compile_declaration_false_no_dts_files() {
    // Test that declaration: false (or absent) does NOT produce .d.ts files
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "declaration": false
          },
          "include": ["src/**/*.ts"]
        }"#,
    );
    write_file(&base.join("src/index.ts"), "export const value = 42;");

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());

    // JS file should exist
    assert!(
        base.join("dist/src/index.js").is_file(),
        "JS output should exist"
    );

    // Declaration file should NOT exist
    assert!(
        !base.join("dist/src/index.d.ts").is_file(),
        "Declaration file should NOT exist when declaration: false"
    );
}

#[test]
fn compile_declaration_absent_no_dts_files() {
    // Test that missing declaration option does NOT produce .d.ts files
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist"
          },
          "include": ["src/**/*.ts"]
        }"#,
    );
    write_file(&base.join("src/index.ts"), "export const value = 42;");

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());

    // JS file should exist
    assert!(
        base.join("dist/src/index.js").is_file(),
        "JS output should exist"
    );

    // Declaration file should NOT exist (declaration defaults to false)
    assert!(
        !base.join("dist/src/index.d.ts").is_file(),
        "Declaration file should NOT exist when declaration is not specified"
    );
}

#[test]
fn compile_declaration_interface_and_type() {
    // Test declaration output for interfaces and type aliases
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "declaration": true
          },
          "include": ["src/**/*.ts"]
        }"#,
    );
    write_file(
        &base.join("src/types.ts"),
        r#"
export interface User {
    id: number;
    name: string;
    email: string;
}

export type UserId = number;

export type UserRole = "admin" | "user" | "guest";
"#,
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());

    // Declaration file should exist
    let dts_path = base.join("dist/src/types.d.ts");
    assert!(dts_path.is_file(), "Declaration file should exist");

    let dts = std::fs::read_to_string(&dts_path).expect("read d.ts");

    // Interface should be in declaration
    assert!(
        dts.contains("interface User"),
        "Declaration should contain User interface: {}",
        dts
    );
    assert!(
        dts.contains("id") && dts.contains("number"),
        "Declaration should contain id property: {}",
        dts
    );
    assert!(
        dts.contains("name") && dts.contains("string"),
        "Declaration should contain name property: {}",
        dts
    );

    // Type aliases should be in declaration
    assert!(
        dts.contains("UserId"),
        "Declaration should contain UserId type: {}",
        dts
    );
    assert!(
        dts.contains("UserRole"),
        "Declaration should contain UserRole type: {}",
        dts
    );
}

#[test]
fn compile_declaration_class_with_methods() {
    // Test declaration output for classes with methods
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "declaration": true
          },
          "include": ["src/**/*.ts"]
        }"#,
    );
    write_file(
        &base.join("src/calculator.ts"),
        r#"
export class Calculator {
    private value: number;

    constructor(initial: number) {
        this.value = initial;
    }

    add(n: number): Calculator {
        this.value = this.value + n;
        return this;
    }

    subtract(n: number): Calculator {
        this.value = this.value - n;
        return this;
    }

    getResult(): number {
        return this.value;
    }
}
"#,
    );

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());

    // Declaration file should exist
    let dts_path = base.join("dist/src/calculator.d.ts");
    assert!(dts_path.is_file(), "Declaration file should exist");

    let dts = std::fs::read_to_string(&dts_path).expect("read d.ts");

    // Class should be in declaration
    assert!(
        dts.contains("class Calculator"),
        "Declaration should contain Calculator class: {}",
        dts
    );

    // Methods should be in declaration
    assert!(
        dts.contains("add") && dts.contains("Calculator"),
        "Declaration should contain add method with return type: {}",
        dts
    );
    assert!(
        dts.contains("subtract"),
        "Declaration should contain subtract method: {}",
        dts
    );
    assert!(
        dts.contains("getResult") && dts.contains("number"),
        "Declaration should contain getResult method: {}",
        dts
    );

    // Private members should be marked private in declaration
    assert!(
        dts.contains("private") && dts.contains("value"),
        "Declaration should contain private value: {}",
        dts
    );
}

#[test]
fn compile_declaration_with_declaration_dir() {
    // Test that declarationDir puts .d.ts files in separate directory
    let temp = TempDir::new().expect("temp dir");
    let base = &temp.path;

    write_file(
        &base.join("tsconfig.json"),
        r#"{
          "compilerOptions": {
            "outDir": "dist",
            "rootDir": "src",
            "declaration": true,
            "declarationDir": "types"
          },
          "include": ["src/**/*.ts"]
        }"#,
    );
    write_file(&base.join("src/index.ts"), "export const value = 42;");

    let args = default_args();
    let result = compile(&args, base).expect("compile should succeed");

    assert!(result.diagnostics.is_empty());

    // JS file should be in outDir
    assert!(
        base.join("dist/index.js").is_file(),
        "JS output should be in dist/"
    );

    // Declaration file should be in declarationDir, NOT in outDir
    assert!(
        base.join("types/index.d.ts").is_file(),
        "Declaration file should be in types/"
    );
    assert!(
        !base.join("dist/index.d.ts").is_file(),
        "Declaration file should NOT be in dist/ when declarationDir is set"
    );
}
