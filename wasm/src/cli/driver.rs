use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

use crate::binder::SymbolTable;
use crate::checker::types::diagnostics::{Diagnostic, DiagnosticCategory};
use crate::cli::args::CliArgs;
use crate::cli::config::{load_tsconfig, resolve_compiler_options, ResolvedCompilerOptions, TsConfig};
use crate::cli::fs::{discover_ts_files, FileDiscoveryOptions};
use crate::declaration_emitter::DeclarationEmitter;
use crate::parallel::{self, BoundFile, MergedProgram};
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
    let tsconfig_path = find_tsconfig(&cwd);
    let config = load_config(tsconfig_path.as_deref())?;

    let mut resolved = resolve_compiler_options(config.as_ref().and_then(|cfg| cfg.compiler_options.as_ref()))?;
    apply_cli_overrides(&mut resolved, args);

    let base_dir = config_base_dir(&cwd, tsconfig_path.as_deref());
    let base_dir = canonicalize_or_owned(&base_dir);
    let out_dir = normalize_output_dir(&base_dir, resolved.out_dir.clone());
    let declaration_dir = normalize_output_dir(&base_dir, resolved.declaration_dir.clone());

    let discovery = build_discovery_options(
        args,
        &base_dir,
        tsconfig_path.as_deref(),
        config.as_ref(),
        out_dir.as_deref(),
    )?;
    let file_paths = discover_ts_files(&discovery)?;
    if file_paths.is_empty() {
        bail!("no input files found");
    }

    let sources = read_source_files(&file_paths)?;
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

    let emitted_files = if resolved.no_emit {
        Vec::new()
    } else {
        let outputs = emit_outputs(
            &program,
            &resolved,
            &base_dir,
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
        Some(candidate)
    } else {
        None
    }
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

fn read_source_files(paths: &[PathBuf]) -> Result<Vec<SourceFile>> {
    let mut sources = Vec::with_capacity(paths.len());
    for path in paths {
        let canonical = canonicalize_or_owned(path);
        let text = std::fs::read_to_string(&canonical)
            .with_context(|| format!("failed to read {}", canonical.display()))?;
        sources.push(SourceFile {
            path: canonical,
            text,
        });
    }
    Ok(sources)
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
    out_dir: Option<&Path>,
    declaration_dir: Option<&Path>,
) -> Result<Vec<OutputFile>> {
    let mut outputs = Vec::new();

    for file in &program.files {
        let input_path = PathBuf::from(&file.file_name);

        if let Some(js_path) = js_output_path(base_dir, out_dir, &input_path) {
            let mut printer = ThinPrinter::with_options(&file.arena, options.printer.clone());
            printer.emit(file.source_file);
            outputs.push(OutputFile {
                path: js_path,
                contents: printer.take_output(),
            });
        }

        if options.emit_declarations {
            let decl_base = declaration_dir.or(out_dir);
            if let Some(dts_path) = declaration_output_path(base_dir, decl_base, &input_path) {
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

fn js_output_path(base_dir: &Path, out_dir: Option<&Path>, input_path: &Path) -> Option<PathBuf> {
    if is_declaration_file(input_path) {
        return None;
    }

    let extension = js_extension_for(input_path)?;
    let relative = input_path.strip_prefix(base_dir).unwrap_or(input_path);
    let mut output = match out_dir {
        Some(out_dir) => out_dir.join(relative),
        None => input_path.to_path_buf(),
    };
    output.set_extension(extension);
    Some(output)
}

fn declaration_output_path(
    base_dir: &Path,
    out_dir: Option<&Path>,
    input_path: &Path,
) -> Option<PathBuf> {
    if is_declaration_file(input_path) {
        return None;
    }

    let relative = input_path.strip_prefix(base_dir).unwrap_or(input_path);
    let file_name = relative.file_name()?.to_str()?;
    let new_name = declaration_file_name(file_name)?;

    let mut output = match out_dir {
        Some(out_dir) => out_dir.join(relative),
        None => input_path.to_path_buf(),
    };
    output.set_file_name(new_name);
    Some(output)
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

fn js_extension_for(path: &Path) -> Option<&'static str> {
    let name = path.file_name().and_then(|name| name.to_str())?;
    if name.ends_with(".mts") {
        return Some("mjs");
    }
    if name.ends_with(".cts") {
        return Some("cjs");
    }

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("ts") | Some("tsx") => Some("js"),
        _ => None,
    }
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
