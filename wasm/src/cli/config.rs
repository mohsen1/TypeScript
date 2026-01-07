use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
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
    pub root_dir: Option<PathBuf>,
    pub out_dir: Option<PathBuf>,
    pub declaration_dir: Option<PathBuf>,
    pub emit_declarations: bool,
    pub no_emit: bool,
    pub no_emit_on_error: bool,
}

impl Default for ResolvedCompilerOptions {
    fn default() -> Self {
        ResolvedCompilerOptions {
            printer: PrinterOptions::default(),
            checker: CheckerOptions::default(),
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
