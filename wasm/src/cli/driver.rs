use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use crate::binder::SymbolTable;
use crate::checker::types::diagnostics::{Diagnostic, DiagnosticCategory};
use crate::cli::args::CliArgs;
use crate::cli::config::{
    load_tsconfig, resolve_compiler_options, JsxEmit, PathMapping, ResolvedCompilerOptions, TsConfig,
};
use crate::cli::fs::{discover_ts_files, is_ts_file, FileDiscoveryOptions};
use crate::declaration_emitter::DeclarationEmitter;
use crate::parallel::{self, BoundFile, MergedProgram};
use crate::parser::thin_node::{NodeAccess, ThinNodeArena};
use crate::parser::NodeIndex;
use crate::thin_parser::ThinParserState;
use crate::thin_parser::ParseDiagnostic;
use crate::thin_binder::ThinBinderState;
use crate::thin_checker::ThinCheckerState;
use crate::thin_emitter::ThinPrinter;

#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub diagnostics: Vec<Diagnostic>,
    pub emitted_files: Vec<PathBuf>,
}

pub fn compile(args: &CliArgs, cwd: &Path) -> Result<CompilationResult> {
    let cwd = canonicalize_or_owned(cwd);
    let tsconfig_path = resolve_tsconfig_path(&cwd, args.project.as_deref())?;
    let config = load_config(tsconfig_path.as_deref())?;

    let mut resolved = resolve_compiler_options(config.as_ref().and_then(|cfg| cfg.compiler_options.as_ref()))?;
    apply_cli_overrides(&mut resolved, args);

    let base_dir = config_base_dir(&cwd, tsconfig_path.as_deref());
    let base_dir = canonicalize_or_owned(&base_dir);
    let root_dir = normalize_root_dir(&base_dir, resolved.root_dir.clone());
    let out_dir = normalize_output_dir(&base_dir, resolved.out_dir.clone());
    let declaration_dir = normalize_output_dir(&base_dir, resolved.declaration_dir.clone());
    let base_url = normalize_base_url(&base_dir, resolved.base_url.clone());
    resolved.base_url = base_url;

    let discovery = build_discovery_options(
        args,
        &base_dir,
        tsconfig_path.as_deref(),
        config.as_ref(),
        out_dir.as_deref(),
    )?;
    let mut file_paths = discover_ts_files(&discovery)?;
    if !resolved.lib_files.is_empty() {
        let mut merged = std::collections::BTreeSet::new();
        merged.extend(file_paths.into_iter());
        merged.extend(resolved.lib_files.iter().cloned());
        file_paths = merged.into_iter().collect();
    }
    if file_paths.is_empty() {
        bail!("no input files found");
    }

    let sources = read_source_files(&file_paths, &base_dir, &resolved)?;
    let compile_inputs: Vec<(String, String)> = sources
        .into_iter()
        .map(|source| (source.path.to_string_lossy().into_owned(), source.text))
        .collect();

    let program = parallel::compile_files(compile_inputs);
    let mut diagnostics = collect_diagnostics(&program);
    diagnostics.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then(left.start.cmp(&right.start))
            .then(left.code.cmp(&right.code))
    });

    let has_error = diagnostics
        .iter()
        .any(|diag| diag.category == DiagnosticCategory::Error);
    let should_emit = !(resolved.no_emit || (resolved.no_emit_on_error && has_error));

    let emitted_files = if !should_emit {
        Vec::new()
    } else {
        let outputs = emit_outputs(
            &program,
            &resolved,
            &base_dir,
            root_dir.as_deref(),
            out_dir.as_deref(),
            declaration_dir.as_deref(),
        )?;
        write_outputs(&outputs)?
    };

    Ok(CompilationResult {
        diagnostics,
        emitted_files,
    })
}

#[derive(Debug, Clone)]
struct SourceFile {
    path: PathBuf,
    text: String,
}

#[derive(Debug, Clone)]
struct OutputFile {
    path: PathBuf,
    contents: String,
}

pub(crate) fn find_tsconfig(cwd: &Path) -> Option<PathBuf> {
    let candidate = cwd.join("tsconfig.json");
    if candidate.is_file() {
        Some(canonicalize_or_owned(&candidate))
    } else {
        None
    }
}

pub(crate) fn resolve_tsconfig_path(cwd: &Path, project: Option<&Path>) -> Result<Option<PathBuf>> {
    let Some(project) = project else {
        return Ok(find_tsconfig(cwd));
    };

    let mut candidate = if project.is_absolute() {
        project.to_path_buf()
    } else {
        cwd.join(project)
    };

    if candidate.is_dir() {
        candidate = candidate.join("tsconfig.json");
    }

    if !candidate.exists() {
        bail!("tsconfig not found at {}", candidate.display());
    }

    if !candidate.is_file() {
        bail!("project path is not a file: {}", candidate.display());
    }

    Ok(Some(canonicalize_or_owned(&candidate)))
}

pub(crate) fn load_config(path: Option<&Path>) -> Result<Option<TsConfig>> {
    let Some(path) = path else {
        return Ok(None);
    };

    let config = load_tsconfig(path)?;
    Ok(Some(config))
}

pub(crate) fn config_base_dir(cwd: &Path, tsconfig_path: Option<&Path>) -> PathBuf {
    tsconfig_path
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| cwd.to_path_buf())
}

fn build_discovery_options(
    args: &CliArgs,
    base_dir: &Path,
    tsconfig_path: Option<&Path>,
    config: Option<&TsConfig>,
    out_dir: Option<&Path>,
) -> Result<FileDiscoveryOptions> {
    if !args.files.is_empty() {
        return Ok(FileDiscoveryOptions {
            base_dir: base_dir.to_path_buf(),
            files: args.files.clone(),
            include: None,
            exclude: None,
            out_dir: out_dir.map(Path::to_path_buf),
        });
    }

    let Some(config) = config else {
        bail!("no input files specified and no tsconfig.json found");
    };
    let Some(tsconfig_path) = tsconfig_path else {
        bail!("no tsconfig.json path available");
    };

    Ok(FileDiscoveryOptions::from_tsconfig(tsconfig_path, config, out_dir))
}

fn read_source_files(
    paths: &[PathBuf],
    base_dir: &Path,
    options: &ResolvedCompilerOptions,
) -> Result<Vec<SourceFile>> {
    let mut sources = HashMap::new();
    let mut seen = HashSet::new();
    let mut pending = VecDeque::new();

    for path in paths {
        let canonical = canonicalize_or_owned(path);
        if seen.insert(canonical.clone()) {
            pending.push_back(canonical);
        }
    }

    while let Some(path) = pending.pop_front() {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let specifiers = collect_module_specifiers_from_text(&path, &text);
        sources.insert(path.clone(), text);

        for specifier in specifiers {
            if let Some(resolved) = resolve_module_specifier(&path, &specifier, options, base_dir) {
                let canonical = canonicalize_or_owned(&resolved);
                if seen.insert(canonical.clone()) {
                    pending.push_back(canonical);
                }
            }
        }
    }

    let mut list: Vec<SourceFile> = sources
        .into_iter()
        .map(|(path, text)| SourceFile { path, text })
        .collect();
    list.sort_by(|left, right| left.path.to_string_lossy().cmp(&right.path.to_string_lossy()));
    Ok(list)
}

fn collect_module_specifiers_from_text(path: &Path, text: &str) -> Vec<String> {
    let file_name = path.to_string_lossy().into_owned();
    let mut parser = ThinParserState::new(file_name, text.to_string());
    let source_file = parser.parse_source_file();
    let (arena, _diagnostics) = parser.into_parts();
    collect_module_specifiers(&arena, source_file)
}

fn collect_module_specifiers(arena: &ThinNodeArena, source_file: NodeIndex) -> Vec<String> {
    let mut specifiers = Vec::new();

    let Some(node) = arena.get(source_file) else {
        return specifiers;
    };
    let Some(source) = arena.get_source_file(node) else {
        return specifiers;
    };

    for &stmt_idx in &source.statements.nodes {
        if stmt_idx.is_none() {
            continue;
        }
        let Some(stmt) = arena.get(stmt_idx) else {
            continue;
        };
        if let Some(import_decl) = arena.get_import_decl(stmt) {
            if let Some(text) = arena.get_literal_text(import_decl.module_specifier) {
                specifiers.push(text.to_string());
            }
        }
        if let Some(export_decl) = arena.get_export_decl(stmt) {
            if let Some(text) = arena.get_literal_text(export_decl.module_specifier) {
                specifiers.push(text.to_string());
            }
        }
    }

    specifiers
}

fn resolve_module_specifier(
    from_file: &Path,
    module_specifier: &str,
    options: &ResolvedCompilerOptions,
    base_dir: &Path,
) -> Option<PathBuf> {
    let specifier = module_specifier.trim();
    if specifier.is_empty() {
        return None;
    }
    let specifier = specifier.replace('\\', "/");
    let mut candidates = Vec::new();

    let mut allow_node_modules = false;

    if Path::new(&specifier).is_absolute() {
        candidates.extend(expand_module_path_candidates(&PathBuf::from(specifier.as_str())));
    } else if specifier.starts_with('.') {
        let from_dir = from_file.parent().unwrap_or(base_dir);
        let joined = from_dir.join(&specifier);
        candidates.extend(expand_module_path_candidates(&joined));
    } else if let Some(base_url) = options.base_url.as_ref() {
        allow_node_modules = true;
        if let Some(paths) = options.paths.as_ref() {
            if let Some((mapping, wildcard)) = select_path_mapping(paths, &specifier) {
                for target in &mapping.targets {
                    let substituted = substitute_path_target(target, &wildcard);
                    let path = if Path::new(&substituted).is_absolute() {
                        PathBuf::from(substituted)
                    } else {
                        base_url.join(substituted)
                    };
                    candidates.extend(expand_module_path_candidates(&path));
                }
            }
        }

        if candidates.is_empty() {
            candidates.extend(expand_module_path_candidates(&base_url.join(&specifier)));
        }
    } else {
        allow_node_modules = true;
    }

    for candidate in candidates {
        if candidate.is_file() && is_ts_file(&candidate) {
            return Some(canonicalize_or_owned(&candidate));
        }
    }

    if allow_node_modules {
        return resolve_node_module_specifier(from_file, &specifier, base_dir);
    }

    None
}

fn select_path_mapping<'a>(
    mappings: &'a [PathMapping],
    specifier: &str,
) -> Option<(&'a PathMapping, String)> {
    let mut best: Option<(&PathMapping, String)> = None;
    let mut best_score = 0usize;
    let mut best_pattern_len = 0usize;

    for mapping in mappings {
        let Some(wildcard) = mapping.match_specifier(specifier) else {
            continue;
        };
        let score = mapping.specificity();
        let pattern_len = mapping.pattern.len();

        let is_better = match &best {
            None => true,
            Some((current, _)) => {
                score > best_score
                    || (score == best_score && pattern_len > best_pattern_len)
                    || (score == best_score
                        && pattern_len == best_pattern_len
                        && mapping.pattern < current.pattern)
            }
        };

        if is_better {
            best_score = score;
            best_pattern_len = pattern_len;
            best = Some((mapping, wildcard));
        }
    }

    best
}

fn substitute_path_target(target: &str, wildcard: &str) -> String {
    if target.contains('*') {
        target.replace('*', wildcard)
    } else {
        target.to_string()
    }
}

fn expand_module_path_candidates(path: &Path) -> Vec<PathBuf> {
    let base = normalize_path(path);
    if base.extension().is_some() {
        return vec![base];
    }

    let mut candidates = Vec::new();
    for ext in TS_EXTENSION_CANDIDATES {
        candidates.push(base.with_extension(ext));
    }
    for ext in TS_EXTENSION_CANDIDATES {
        candidates.push(base.join("index").with_extension(ext));
    }
    candidates
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::RootDir
            | std::path::Component::Normal(_)
            | std::path::Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    normalized
}

const TS_EXTENSION_CANDIDATES: [&str; 7] = ["ts", "tsx", "d.ts", "mts", "cts", "d.mts", "d.cts"];

#[derive(Debug, Deserialize)]
struct PackageJson {
    #[serde(default)]
    types: Option<String>,
    #[serde(default)]
    typings: Option<String>,
    #[serde(default)]
    main: Option<String>,
    #[serde(default)]
    module: Option<String>,
    #[serde(default)]
    exports: Option<serde_json::Value>,
}

fn resolve_node_module_specifier(
    from_file: &Path,
    module_specifier: &str,
    base_dir: &Path,
) -> Option<PathBuf> {
    let (package_name, subpath) = split_package_specifier(module_specifier)?;
    let mut current = from_file.parent().unwrap_or(base_dir);

    loop {
        let package_root = current.join("node_modules").join(&package_name);
        if package_root.is_dir() {
            let resolved = if let Some(subpath) = subpath.as_deref() {
                resolve_package_entry(&package_root, subpath)
            } else {
                resolve_package_root(&package_root)
            };
            if resolved.is_some() {
                return resolved;
            }
        }

        if current == base_dir {
            break;
        }
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }

    None
}

fn split_package_specifier(specifier: &str) -> Option<(String, Option<String>)> {
    let mut parts = specifier.split('/');
    let first = parts.next()?;

    if first.starts_with('@') {
        let second = parts.next()?;
        let package = format!("{first}/{second}");
        let rest = parts.collect::<Vec<_>>().join("/");
        let subpath = if rest.is_empty() { None } else { Some(rest) };
        return Some((package, subpath));
    }

    let rest = parts.collect::<Vec<_>>().join("/");
    let subpath = if rest.is_empty() { None } else { Some(rest) };
    Some((first.to_string(), subpath))
}

fn resolve_package_root(package_root: &Path) -> Option<PathBuf> {
    let package_json_path = package_root.join("package.json");
    let package_json = read_package_json(&package_json_path);
    let mut candidates = Vec::new();

    if let Some(package_json) = package_json.as_ref() {
        candidates = collect_package_entry_candidates(package_json);
    }

    if !candidates.iter().any(|entry| entry == "index" || entry == "./index") {
        candidates.push("index".to_string());
    }

    for entry in candidates {
        if let Some(resolved) = resolve_package_entry(package_root, &entry) {
            return Some(resolved);
        }
    }

    None
}

fn resolve_package_entry(package_root: &Path, entry: &str) -> Option<PathBuf> {
    let entry = entry.trim();
    if entry.is_empty() {
        return None;
    }
    let entry = entry.trim_start_matches("./");
    let path = if Path::new(entry).is_absolute() {
        PathBuf::from(entry)
    } else {
        package_root.join(entry)
    };

    for candidate in expand_module_path_candidates(&path) {
        if candidate.is_file() && is_ts_file(&candidate) {
            return Some(canonicalize_or_owned(&candidate));
        }
    }

    None
}

fn read_package_json(path: &Path) -> Option<PackageJson> {
    let contents = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

fn collect_package_entry_candidates(package_json: &PackageJson) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();

    for value in [
        package_json.types.as_ref(),
        package_json.typings.as_ref(),
    ] {
        if let Some(value) = value {
            if seen.insert(value.clone()) {
                candidates.push(value.clone());
            }
        }
    }

    if let Some(exports) = package_json.exports.as_ref() {
        if let Some(entry) = extract_exports_path(exports) {
            if seen.insert(entry.clone()) {
                candidates.push(entry);
            }
        }
    }

    for value in [
        package_json.module.as_ref(),
        package_json.main.as_ref(),
    ] {
        if let Some(value) = value {
            if seen.insert(value.clone()) {
                candidates.push(value.clone());
            }
        }
    }

    candidates
}

fn extract_exports_path(exports: &serde_json::Value) -> Option<String> {
    match exports {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Object(map) => {
            if let Some(entry) = map.get(".") {
                if let Some(path) = extract_exports_path(entry) {
                    return Some(path);
                }
            }

            for key in ["types", "default", "import", "require"] {
                if let Some(value) = map.get(key).and_then(|value| value.as_str()) {
                    return Some(value.to_string());
                }
            }

            None
        }
        _ => None,
    }
}

fn collect_diagnostics(program: &MergedProgram) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (file_idx, file) in program.files.iter().enumerate() {
        for parse_diagnostic in &file.parse_diagnostics {
            diagnostics.push(parse_diagnostic_to_checker(&file.file_name, parse_diagnostic));
        }

        let binder = create_binder_from_bound_file(file, program, file_idx);
        let mut checker = ThinCheckerState::new(
            &file.arena,
            &binder,
            &program.type_interner,
            file.file_name.clone(),
        );
        checker.check_source_file(file.source_file);
        diagnostics.extend(std::mem::take(&mut checker.ctx.diagnostics));
    }

    diagnostics
}

fn parse_diagnostic_to_checker(file_name: &str, diagnostic: &ParseDiagnostic) -> Diagnostic {
    Diagnostic {
        file: file_name.to_string(),
        start: diagnostic.start,
        length: diagnostic.length,
        message_text: diagnostic.message.clone(),
        category: DiagnosticCategory::Error,
        code: diagnostic.code,
        related_information: Vec::new(),
    }
}

fn create_binder_from_bound_file(
    file: &BoundFile,
    program: &MergedProgram,
    file_idx: usize,
) -> ThinBinderState {
    let mut file_locals = SymbolTable::new();

    if file_idx < program.file_locals.len() {
        for (name, &sym_id) in program.file_locals[file_idx].iter() {
            file_locals.set(name.clone(), sym_id);
        }
    }

    for (name, &sym_id) in program.globals.iter() {
        if !file_locals.has(name) {
            file_locals.set(name.clone(), sym_id);
        }
    }

    ThinBinderState::from_bound_state(
        program.symbols.clone(),
        file_locals,
        file.node_symbols.clone(),
    )
}

fn emit_outputs(
    program: &MergedProgram,
    options: &ResolvedCompilerOptions,
    base_dir: &Path,
    root_dir: Option<&Path>,
    out_dir: Option<&Path>,
    declaration_dir: Option<&Path>,
) -> Result<Vec<OutputFile>> {
    let mut outputs = Vec::new();

    for file in &program.files {
        let input_path = PathBuf::from(&file.file_name);

        if let Some(js_path) = js_output_path(base_dir, root_dir, out_dir, options.jsx, &input_path) {
            let mut printer = ThinPrinter::with_options(&file.arena, options.printer.clone());
            printer.emit(file.source_file);
            outputs.push(OutputFile {
                path: js_path,
                contents: printer.take_output(),
            });
        }

        if options.emit_declarations {
            let decl_base = declaration_dir.or(out_dir);
            if let Some(dts_path) = declaration_output_path(base_dir, root_dir, decl_base, &input_path) {
                let mut emitter = DeclarationEmitter::new(&file.arena);
                let contents = emitter.emit(file.source_file);
                outputs.push(OutputFile {
                    path: dts_path,
                    contents,
                });
            }
        }
    }

    Ok(outputs)
}

fn write_outputs(outputs: &[OutputFile]) -> Result<Vec<PathBuf>> {
    outputs.par_iter().try_for_each(|output| -> Result<()> {
        if let Some(parent) = output.path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {}", parent.display()))?;
        }
        std::fs::write(&output.path, &output.contents)
            .with_context(|| format!("failed to write {}", output.path.display()))?;
        Ok(())
    })?;

    Ok(outputs.iter().map(|output| output.path.clone()).collect())
}

fn js_output_path(
    base_dir: &Path,
    root_dir: Option<&Path>,
    out_dir: Option<&Path>,
    jsx: Option<JsxEmit>,
    input_path: &Path,
) -> Option<PathBuf> {
    if is_declaration_file(input_path) {
        return None;
    }

    let extension = js_extension_for(input_path, jsx)?;
    let relative = output_relative_path(base_dir, root_dir, input_path);
    let mut output = match out_dir {
        Some(out_dir) => out_dir.join(relative),
        None => input_path.to_path_buf(),
    };
    output.set_extension(extension);
    Some(output)
}

fn declaration_output_path(
    base_dir: &Path,
    root_dir: Option<&Path>,
    out_dir: Option<&Path>,
    input_path: &Path,
) -> Option<PathBuf> {
    if is_declaration_file(input_path) {
        return None;
    }

    let relative = output_relative_path(base_dir, root_dir, input_path);
    let file_name = relative.file_name()?.to_str()?;
    let new_name = declaration_file_name(file_name)?;

    let mut output = match out_dir {
        Some(out_dir) => out_dir.join(relative),
        None => input_path.to_path_buf(),
    };
    output.set_file_name(new_name);
    Some(output)
}

fn output_relative_path(base_dir: &Path, root_dir: Option<&Path>, input_path: &Path) -> PathBuf {
    if let Some(root_dir) = root_dir {
        if let Ok(relative) = input_path.strip_prefix(root_dir) {
            return relative.to_path_buf();
        }
    }

    input_path
        .strip_prefix(base_dir)
        .unwrap_or(input_path)
        .to_path_buf()
}

fn declaration_file_name(file_name: &str) -> Option<String> {
    if file_name.ends_with(".mts") {
        return Some(file_name.trim_end_matches(".mts").to_string() + ".d.mts");
    }
    if file_name.ends_with(".cts") {
        return Some(file_name.trim_end_matches(".cts").to_string() + ".d.cts");
    }
    if file_name.ends_with(".tsx") {
        return Some(file_name.trim_end_matches(".tsx").to_string() + ".d.ts");
    }
    if file_name.ends_with(".ts") {
        return Some(file_name.trim_end_matches(".ts").to_string() + ".d.ts");
    }

    None
}

fn is_declaration_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
}

fn js_extension_for(path: &Path, jsx: Option<JsxEmit>) -> Option<&'static str> {
    let name = path.file_name().and_then(|name| name.to_str())?;
    if name.ends_with(".mts") {
        return Some("mjs");
    }
    if name.ends_with(".cts") {
        return Some("cjs");
    }

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("ts") => Some("js"),
        Some("tsx") => match jsx {
            Some(JsxEmit::Preserve) => Some("jsx"),
            Some(JsxEmit::ReactNative) | None => Some("js"),
        },
        _ => None,
    }
}

pub(crate) fn normalize_base_url(base_dir: &Path, dir: Option<PathBuf>) -> Option<PathBuf> {
    dir.map(|dir| {
        let resolved = if dir.is_absolute() {
            dir
        } else {
            base_dir.join(dir)
        };
        canonicalize_or_owned(&resolved)
    })
}

pub(crate) fn normalize_output_dir(base_dir: &Path, dir: Option<PathBuf>) -> Option<PathBuf> {
    dir.map(|dir| {
        if dir.is_absolute() {
            dir
        } else {
            base_dir.join(dir)
        }
    })
}

pub(crate) fn normalize_root_dir(base_dir: &Path, dir: Option<PathBuf>) -> Option<PathBuf> {
    dir.map(|dir| {
        let resolved = if dir.is_absolute() {
            dir
        } else {
            base_dir.join(dir)
        };
        canonicalize_or_owned(&resolved)
    })
}

fn canonicalize_or_owned(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

pub(crate) fn apply_cli_overrides(options: &mut ResolvedCompilerOptions, args: &CliArgs) {
    if let Some(target) = args.target {
        options.printer.target = target.to_script_target();
    }
    if let Some(module) = args.module {
        options.printer.module = module.to_module_kind();
    }
    if let Some(out_dir) = args.out_dir.as_ref() {
        options.out_dir = Some(out_dir.clone());
    }
    if args.strict {
        options.checker.strict = true;
    }
    if args.no_emit {
        options.no_emit = true;
    }
}
