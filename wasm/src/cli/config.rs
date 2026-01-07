use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
    pub out_dir: Option<String>,
    #[serde(default)]
    pub strict: Option<bool>,
    #[serde(default)]
    pub no_emit: Option<bool>,
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
        out_dir: child.out_dir.or(base.out_dir),
        strict: child.strict.or(base.strict),
        no_emit: child.no_emit.or(base.no_emit),
    }
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
