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
