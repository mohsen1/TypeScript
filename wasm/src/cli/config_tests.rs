use super::config::{load_tsconfig, parse_tsconfig, resolve_compiler_options};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::thin_emitter::{ModuleKind, ScriptTarget};

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
        path.push(format!("stc_cli_test_{}_{}", std::process::id(), nanos));
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn write_file(dir: &Path, name: &str, contents: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, contents).expect("failed to write test file");
    path
}

#[test]
fn parses_jsonc_with_trailing_commas() {
    let input = r#"
    {
      // comment
      "compilerOptions": {
        "target": "es2017", /* inline */
        "module": "commonjs",
      },
      "include": ["src/**/*",],
    }
    "#;

    let config = parse_tsconfig(input).expect("should parse JSONC");
    let options = config.compiler_options.expect("compilerOptions missing");

    assert_eq!(options.target.as_deref(), Some("es2017"));
    assert_eq!(options.module.as_deref(), Some("commonjs"));
    assert_eq!(config.include, Some(vec!["src/**/*".to_string()]));
}

#[test]
fn load_tsconfig_merges_extends() {
    let temp = TempDir::new().expect("temp dir");

    write_file(
        &temp.path,
        "tsconfig.base.json",
        r#"{
          "compilerOptions": {"target": "es2015", "strict": true},
          "include": ["src"],
          "exclude": ["dist"]
        }"#,
    );

    let child_path = write_file(
        &temp.path,
        "tsconfig.json",
        r#"{
          "extends": "./tsconfig.base.json",
          "compilerOptions": {"module": "commonjs", "strict": false},
          "files": ["main.ts"]
        }"#,
    );

    let config = load_tsconfig(&child_path).expect("should load config");
    let options = config.compiler_options.expect("compilerOptions missing");

    assert_eq!(options.target.as_deref(), Some("es2015"));
    assert_eq!(options.module.as_deref(), Some("commonjs"));
    assert_eq!(options.strict, Some(false));
    assert_eq!(config.include, Some(vec!["src".to_string()]));
    assert_eq!(config.exclude, Some(vec!["dist".to_string()]));
    assert_eq!(config.files, Some(vec!["main.ts".to_string()]));
}

#[test]
fn load_tsconfig_detects_extends_cycle() {
    let temp = TempDir::new().expect("temp dir");

    write_file(&temp.path, "a.json", r#"{"extends":"./b.json"}"#);
    write_file(&temp.path, "b.json", r#"{"extends":"./a.json"}"#);

    let err = load_tsconfig(&temp.path.join("a.json")).expect_err("cycle should error");
    let message = err.to_string();
    assert!(message.contains("extends cycle"), "{message}");
}

#[test]
fn resolve_compiler_options_defaults() {
    let resolved = resolve_compiler_options(None).expect("defaults should resolve");

    assert_eq!(resolved.printer.target, ScriptTarget::ESNext);
    assert_eq!(resolved.printer.module, ModuleKind::None);
    assert!(resolved.out_dir.is_none());
    assert!(!resolved.checker.strict);
    assert!(!resolved.no_emit);
}

#[test]
fn resolve_compiler_options_overrides() {
    let config = parse_tsconfig(
        r#"{
          "compilerOptions": {
            "target": "ES2020",
            "module": "common-js",
            "outDir": "dist",
            "strict": true,
            "noEmit": true
          }
        }"#,
    )
    .expect("should parse config");

    let resolved = resolve_compiler_options(config.compiler_options.as_ref())
        .expect("compiler options should resolve");

    assert_eq!(resolved.printer.target, ScriptTarget::ES2020);
    assert_eq!(resolved.printer.module, ModuleKind::CommonJS);
    assert_eq!(resolved.out_dir, Some(PathBuf::from("dist")));
    assert!(resolved.checker.strict);
    assert!(resolved.no_emit);
}

#[test]
fn resolve_compiler_options_rejects_unknown_values() {
    let config = parse_tsconfig(
        r#"{
          "compilerOptions": {
            "target": "es2999",
            "module": "totally-not-a-module"
          }
        }"#,
    )
    .expect("should parse config");

    let err = resolve_compiler_options(config.compiler_options.as_ref())
        .expect_err("unknown compilerOptions should error");
    let message = err.to_string();
    assert!(message.contains("compilerOptions.target"), "{message}");
}
