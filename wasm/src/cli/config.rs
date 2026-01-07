use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use crate::thin_emitter::{ModuleKind, PrinterOptions, ScriptTarget};

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TsConfig {
    #[serde(default)]
    pub extends: Option<String>,
    #[serde(default)]
    pub compiler_options: Option<CompilerOptions>,
    #[serde(default)]
    pub include: Option<Vec<String>>,
    #[serde(default)]
    pub exclude: Option<Vec<String>>,
    #[serde(default)]
    pub files: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompilerOptions {
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub module: Option<String>,
    #[serde(default)]
    pub jsx: Option<String>,
    #[serde(default)]
    pub lib: Option<Vec<String>>,
    #[serde(default)]
    pub root_dir: Option<String>,
    #[serde(default)]
    pub out_dir: Option<String>,
    #[serde(default)]
    pub declaration: Option<bool>,
    #[serde(default)]
    pub declaration_dir: Option<String>,
    #[serde(default)]
    pub strict: Option<bool>,
    #[serde(default)]
    pub no_emit: Option<bool>,
    #[serde(default)]
    pub no_emit_on_error: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct CheckerOptions {
    pub strict: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedCompilerOptions {
    pub printer: PrinterOptions,
    pub checker: CheckerOptions,
    pub jsx: Option<JsxEmit>,
    pub lib_files: Vec<PathBuf>,
    pub root_dir: Option<PathBuf>,
    pub out_dir: Option<PathBuf>,
    pub declaration_dir: Option<PathBuf>,
    pub emit_declarations: bool,
    pub no_emit: bool,
    pub no_emit_on_error: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxEmit {
    Preserve,
    ReactNative,
}

impl Default for ResolvedCompilerOptions {
    fn default() -> Self {
        ResolvedCompilerOptions {
            printer: PrinterOptions::default(),
            checker: CheckerOptions::default(),
            jsx: None,
            lib_files: Vec::new(),
            root_dir: None,
            out_dir: None,
            declaration_dir: None,
            emit_declarations: false,
            no_emit: false,
            no_emit_on_error: false,
        }
    }
}

pub fn resolve_compiler_options(options: Option<&CompilerOptions>) -> Result<ResolvedCompilerOptions> {
    let mut resolved = ResolvedCompilerOptions::default();
    let Some(options) = options else {
        return Ok(resolved);
    };

    if let Some(target) = options.target.as_deref() {
        resolved.printer.target = parse_script_target(target)?;
    }

    if let Some(module) = options.module.as_deref() {
        resolved.printer.module = parse_module_kind(module)?;
    }

    if let Some(jsx) = options.jsx.as_deref() {
        resolved.jsx = Some(parse_jsx_emit(jsx)?);
    }

    if let Some(lib_list) = options.lib.as_ref() {
        resolved.lib_files = resolve_lib_files(lib_list)?;
    }

    if let Some(root_dir) = options.root_dir.as_deref() {
        if !root_dir.is_empty() {
            resolved.root_dir = Some(PathBuf::from(root_dir));
        }
    }

    if let Some(out_dir) = options.out_dir.as_deref() {
        if !out_dir.is_empty() {
            resolved.out_dir = Some(PathBuf::from(out_dir));
        }
    }

    if let Some(declaration_dir) = options.declaration_dir.as_deref() {
        if !declaration_dir.is_empty() {
            resolved.declaration_dir = Some(PathBuf::from(declaration_dir));
        }
    }

    if let Some(declaration) = options.declaration {
        resolved.emit_declarations = declaration;
    }

    if let Some(strict) = options.strict {
        resolved.checker.strict = strict;
    }

    if let Some(no_emit) = options.no_emit {
        resolved.no_emit = no_emit;
    }

    if let Some(no_emit_on_error) = options.no_emit_on_error {
        resolved.no_emit_on_error = no_emit_on_error;
    }

    Ok(resolved)
}

pub fn parse_tsconfig(source: &str) -> Result<TsConfig> {
    let stripped = strip_jsonc(source);
    let normalized = remove_trailing_commas(&stripped);
    let config = serde_json::from_str(&normalized).context("failed to parse tsconfig JSON")?;
    Ok(config)
}

pub fn load_tsconfig(path: &Path) -> Result<TsConfig> {
    let mut visited = HashSet::new();
    load_tsconfig_inner(path, &mut visited)
}

fn load_tsconfig_inner(path: &Path, visited: &mut HashSet<PathBuf>) -> Result<TsConfig> {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if !visited.insert(canonical.clone()) {
        bail!("tsconfig extends cycle detected at {}", canonical.display());
    }

    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read tsconfig: {}", path.display()))?;
    let mut config = parse_tsconfig(&source)
        .with_context(|| format!("failed to parse tsconfig: {}", path.display()))?;

    let extends = config.extends.take();
    if let Some(extends_path) = extends {
        let base_path = resolve_extends_path(path, &extends_path)?;
        let base_config = load_tsconfig_inner(&base_path, visited)?;
        config = merge_configs(base_config, config);
    }

    visited.remove(&canonical);
    Ok(config)
}

fn resolve_extends_path(current_path: &Path, extends: &str) -> Result<PathBuf> {
    let base_dir = current_path
        .parent()
        .ok_or_else(|| anyhow!("tsconfig has no parent directory"))?;
    let mut candidate = PathBuf::from(extends);
    if candidate.extension().is_none() {
        candidate.set_extension("json");
    }

    if candidate.is_absolute() {
        Ok(candidate)
    } else {
        Ok(base_dir.join(candidate))
    }
}

fn merge_configs(base: TsConfig, mut child: TsConfig) -> TsConfig {
    let merged_compiler_options = match (base.compiler_options, child.compiler_options.take()) {
        (Some(base_opts), Some(child_opts)) => Some(merge_compiler_options(base_opts, child_opts)),
        (Some(base_opts), None) => Some(base_opts),
        (None, Some(child_opts)) => Some(child_opts),
        (None, None) => None,
    };

    TsConfig {
        extends: None,
        compiler_options: merged_compiler_options,
        include: child.include.or(base.include),
        exclude: child.exclude.or(base.exclude),
        files: child.files.or(base.files),
    }
}

fn merge_compiler_options(base: CompilerOptions, child: CompilerOptions) -> CompilerOptions {
    CompilerOptions {
        target: child.target.or(base.target),
        module: child.module.or(base.module),
        jsx: child.jsx.or(base.jsx),
        lib: child.lib.or(base.lib),
        root_dir: child.root_dir.or(base.root_dir),
        out_dir: child.out_dir.or(base.out_dir),
        declaration: child.declaration.or(base.declaration),
        declaration_dir: child.declaration_dir.or(base.declaration_dir),
        strict: child.strict.or(base.strict),
        no_emit: child.no_emit.or(base.no_emit),
        no_emit_on_error: child.no_emit_on_error.or(base.no_emit_on_error),
    }
}

fn parse_script_target(value: &str) -> Result<ScriptTarget> {
    let normalized = normalize_option(value);
    let target = match normalized.as_str() {
        "es3" => ScriptTarget::ES3,
        "es5" => ScriptTarget::ES5,
        "es6" | "es2015" => ScriptTarget::ES2015,
        "es2016" => ScriptTarget::ES2016,
        "es2017" => ScriptTarget::ES2017,
        "es2018" => ScriptTarget::ES2018,
        "es2019" => ScriptTarget::ES2019,
        "es2020" => ScriptTarget::ES2020,
        "es2021" => ScriptTarget::ES2021,
        "es2022" => ScriptTarget::ES2022,
        "esnext" => ScriptTarget::ESNext,
        _ => bail!("unsupported compilerOptions.target '{}'", value),
    };

    Ok(target)
}

fn parse_module_kind(value: &str) -> Result<ModuleKind> {
    let normalized = normalize_option(value);
    let module = match normalized.as_str() {
        "none" => ModuleKind::None,
        "commonjs" => ModuleKind::CommonJS,
        "amd" => ModuleKind::AMD,
        "umd" => ModuleKind::UMD,
        "system" => ModuleKind::System,
        "es6" | "es2015" => ModuleKind::ES2015,
        "es2020" => ModuleKind::ES2020,
        "es2022" => ModuleKind::ES2022,
        "esnext" => ModuleKind::ESNext,
        "node16" => ModuleKind::Node16,
        "nodenext" => ModuleKind::NodeNext,
        _ => bail!("unsupported compilerOptions.module '{}'", value),
    };

    Ok(module)
}

fn parse_jsx_emit(value: &str) -> Result<JsxEmit> {
    let normalized = normalize_option(value);
    let jsx = match normalized.as_str() {
        "preserve" => JsxEmit::Preserve,
        "reactnative" => JsxEmit::ReactNative,
        _ => bail!("unsupported compilerOptions.jsx '{}'", value),
    };

    Ok(jsx)
}

fn resolve_lib_files(lib_list: &[String]) -> Result<Vec<PathBuf>> {
    if lib_list.is_empty() {
        return Ok(Vec::new());
    }

    let lib_dir = default_lib_dir()?;
    let lib_map = build_lib_map(&lib_dir)?;
    let mut resolved = Vec::new();
    let mut pending: VecDeque<String> = lib_list
        .iter()
        .map(|value| normalize_lib_name(value))
        .collect();
    let mut visited = HashSet::new();

    while let Some(lib_name) = pending.pop_front() {
        if lib_name.is_empty() || !visited.insert(lib_name.clone()) {
            continue;
        }

        let path = lib_map
            .get(&lib_name)
            .ok_or_else(|| anyhow!("unsupported compilerOptions.lib '{}'", lib_name))?
            .clone();
        resolved.push(path.clone());

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read lib file {}", path.display()))?;
        for reference in extract_lib_references(&contents) {
            pending.push_back(reference);
        }
    }

    Ok(resolved)
}

fn default_lib_dir() -> Result<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let candidate = manifest_dir.join("..").join("src").join("lib");
    if candidate.is_dir() {
        Ok(canonicalize_or_owned(&candidate))
    } else {
        bail!("lib directory not found at {}", candidate.display());
    }
}

fn build_lib_map(lib_dir: &Path) -> Result<HashMap<String, PathBuf>> {
    let mut map = HashMap::new();
    for entry in std::fs::read_dir(lib_dir)
        .with_context(|| format!("failed to read lib directory {}", lib_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.ends_with(".d.ts") {
            continue;
        }

        let stem = file_name.trim_end_matches(".d.ts");
        let stem = stem.strip_suffix(".generated").unwrap_or(stem);
        let key = normalize_lib_name(stem);
        map.insert(key, canonicalize_or_owned(&path));
    }

    Ok(map)
}

fn extract_lib_references(source: &str) -> Vec<String> {
    let mut refs = Vec::new();
    for line in source.lines() {
        let line = line.trim_start();
        if !line.starts_with("///") {
            if line.is_empty() {
                continue;
            }
            break;
        }
        if let Some(value) = parse_reference_lib_value(line) {
            refs.push(normalize_lib_name(value));
        }
    }
    refs
}

fn parse_reference_lib_value(line: &str) -> Option<&str> {
    let mut offset = 0;
    let bytes = line.as_bytes();
    while let Some(idx) = line[offset..].find("lib=") {
        let start = offset + idx;
        if start > 0 {
            let prev = bytes[start - 1];
            if !prev.is_ascii_whitespace() && prev != b'<' {
                offset = start + 4;
                continue;
            }
        }
        let quote = *bytes.get(start + 4)?;
        if quote != b'"' && quote != b'\'' {
            offset = start + 4;
            continue;
        }
        let rest = &line[start + 5..];
        let end = rest.find(quote as char)?;
        return Some(&rest[..end]);
    }
    None
}

fn normalize_lib_name(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn canonicalize_or_owned(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn normalize_option(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch == '-' || ch == '_' || ch.is_whitespace() {
            continue;
        }
        normalized.push(ch.to_ascii_lowercase());
    }
    normalized
}

fn strip_jsonc(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                out.push(ch);
            }
            continue;
        }

        if in_block_comment {
            if ch == '*' {
                if let Some('/') = chars.peek().copied() {
                    chars.next();
                    in_block_comment = false;
                }
            } else if ch == '\n' {
                out.push(ch);
            }
            continue;
        }

        if in_string {
            out.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            out.push(ch);
            continue;
        }

        if ch == '/' {
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    chars.next();
                    in_line_comment = true;
                    continue;
                }
                if next == '*' {
                    chars.next();
                    in_block_comment = true;
                    continue;
                }
            }
        }

        out.push(ch);
    }

    out
}

fn remove_trailing_commas(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape = false;

    while let Some(ch) = chars.next() {
        if in_string {
            out.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            out.push(ch);
            continue;
        }

        if ch == ',' {
            let mut lookahead = chars.clone();
            while let Some(next) = lookahead.peek().copied() {
                if next.is_whitespace() {
                    lookahead.next();
                    continue;
                }
                if next == '}' || next == ']' {
                    break;
                }
                break;
            }

            if let Some(next) = lookahead.peek().copied() {
                if next == '}' || next == ']' {
                    continue;
                }
            }
        }

        out.push(ch);
    }

    out
}
