use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::binder::{symbol_flags, SymbolId, SymbolTable};
use crate::checker::TypeCache;
use crate::checker::types::diagnostics::{Diagnostic, DiagnosticCategory};
use crate::cli::args::CliArgs;
use crate::cli::config::{
    load_tsconfig, resolve_compiler_options, JsxEmit, ModuleResolutionKind, PathMapping,
    ResolvedCompilerOptions, TsConfig,
};
use crate::cli::fs::{discover_ts_files, is_ts_file, FileDiscoveryOptions};
use crate::declaration_emitter::DeclarationEmitter;
use crate::parallel::{self, BindResult, BoundFile, MergedProgram};
use crate::parser::syntax_kind_ext;
use crate::parser::thin_node::{NodeAccess, ThinNodeArena};
use crate::parser::NodeIndex;
use crate::scanner::SyntaxKind;
use crate::source_map::SourceMapGenerator;
use crate::thin_parser::ThinParserState;
use crate::thin_parser::ParseDiagnostic;
use crate::thin_binder::ThinBinderState;
use crate::thin_checker::ThinCheckerState;
use crate::thin_emitter::{ModuleKind, NewLineKind, ThinPrinter};
use crate::solver::{TypeFormatter, TypeId};
use rustc_hash::FxHasher;

#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub diagnostics: Vec<Diagnostic>,
    pub emitted_files: Vec<PathBuf>,
}

#[derive(Default)]
pub(crate) struct CompilationCache {
    type_caches: HashMap<PathBuf, TypeCache>,
    bind_cache: HashMap<PathBuf, BindCacheEntry>,
    dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
    reverse_dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
    diagnostics: HashMap<PathBuf, Vec<Diagnostic>>,
    export_hashes: HashMap<PathBuf, u64>,
    import_symbol_ids: HashMap<PathBuf, HashMap<PathBuf, Vec<SymbolId>>>,
}

struct BindCacheEntry {
    hash: u64,
    bind_result: BindResult,
}

impl CompilationCache {
    #[cfg(test)]
    pub(crate) fn invalidate_paths_with_dependents<I>(&mut self, paths: I)
    where
        I: IntoIterator<Item = PathBuf>,
    {
        let affected = self.collect_dependents(paths);
        for path in affected {
            self.type_caches.remove(&path);
            self.bind_cache.remove(&path);
            self.diagnostics.remove(&path);
            self.export_hashes.remove(&path);
            self.import_symbol_ids.remove(&path);
        }
    }

    pub(crate) fn invalidate_paths_with_dependents_symbols<I>(&mut self, paths: I)
    where
        I: IntoIterator<Item = PathBuf>,
    {
        let changed: HashSet<PathBuf> = paths.into_iter().collect();
        let affected = self.collect_dependents(changed.iter().cloned());
        for path in affected {
            if changed.contains(&path) {
                self.type_caches.remove(&path);
                self.bind_cache.remove(&path);
                self.diagnostics.remove(&path);
                self.export_hashes.remove(&path);
                self.import_symbol_ids.remove(&path);
                continue;
            }

            self.diagnostics.remove(&path);
            self.export_hashes.remove(&path);

            let mut roots = Vec::new();
            if let Some(dep_map) = self.import_symbol_ids.get(&path) {
                for changed_path in &changed {
                    if let Some(symbols) = dep_map.get(changed_path) {
                        roots.extend(symbols.iter().copied());
                    }
                }
            }

            if roots.is_empty() {
                self.type_caches.remove(&path);
                continue;
            }

            if let Some(cache) = self.type_caches.get_mut(&path) {
                cache.invalidate_symbols(&roots);
            }
        }
    }

    pub(crate) fn invalidate_paths<I>(&mut self, paths: I)
    where
        I: IntoIterator<Item = PathBuf>,
    {
        for path in paths {
            self.type_caches.remove(&path);
            self.bind_cache.remove(&path);
            self.diagnostics.remove(&path);
            self.export_hashes.remove(&path);
            self.import_symbol_ids.remove(&path);
        }
    }

    pub(crate) fn clear(&mut self) {
        self.type_caches.clear();
        self.bind_cache.clear();
        self.dependencies.clear();
        self.reverse_dependencies.clear();
        self.diagnostics.clear();
        self.export_hashes.clear();
        self.import_symbol_ids.clear();
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.type_caches.len()
    }

    #[cfg(test)]
    pub(crate) fn bind_len(&self) -> usize {
        self.bind_cache.len()
    }

    #[cfg(test)]
    pub(crate) fn diagnostics_len(&self) -> usize {
        self.diagnostics.len()
    }

    #[cfg(test)]
    pub(crate) fn export_hash(&self, path: &Path) -> Option<u64> {
        self.export_hashes.get(path).copied()
    }

    #[cfg(test)]
    pub(crate) fn symbol_cache_len(&self, path: &Path) -> Option<usize> {
        self.type_caches.get(path).map(|cache| cache.symbol_types.len())
    }

    #[cfg(test)]
    pub(crate) fn node_cache_len(&self, path: &Path) -> Option<usize> {
        self.type_caches.get(path).map(|cache| cache.node_types.len())
    }

    pub(crate) fn update_dependencies(&mut self, dependencies: HashMap<PathBuf, HashSet<PathBuf>>) {
        let mut reverse = HashMap::new();
        for (source, deps) in &dependencies {
            for dep in deps {
                reverse
                    .entry(dep.clone())
                    .or_insert_with(HashSet::new)
                    .insert(source.clone());
            }
        }
        self.dependencies = dependencies;
        self.reverse_dependencies = reverse;
    }

    fn collect_dependents<I>(&self, paths: I) -> HashSet<PathBuf>
    where
        I: IntoIterator<Item = PathBuf>,
    {
        let mut pending = VecDeque::new();
        let mut affected = HashSet::new();

        for path in paths {
            if affected.insert(path.clone()) {
                pending.push_back(path);
            }
        }

        while let Some(path) = pending.pop_front() {
            let Some(dependents) = self.reverse_dependencies.get(&path) else {
                continue;
            };
            for dependent in dependents {
                if affected.insert(dependent.clone()) {
                    pending.push_back(dependent.clone());
                }
            }
        }

        affected
    }
}

pub fn compile(args: &CliArgs, cwd: &Path) -> Result<CompilationResult> {
    compile_inner(args, cwd, None, None, None)
}

pub(crate) fn compile_with_cache(
    args: &CliArgs,
    cwd: &Path,
    cache: &mut CompilationCache,
) -> Result<CompilationResult> {
    compile_inner(args, cwd, Some(cache), None, None)
}

pub(crate) fn compile_with_cache_and_changes(
    args: &CliArgs,
    cwd: &Path,
    cache: &mut CompilationCache,
    changed_paths: &[PathBuf],
) -> Result<CompilationResult> {
    let canonical_paths: Vec<PathBuf> = changed_paths
        .iter()
        .map(|path| canonicalize_or_owned(path))
        .collect();
    let mut old_hashes = HashMap::new();
    for path in &canonical_paths {
        if let Some(&hash) = cache.export_hashes.get(path) {
            old_hashes.insert(path.clone(), hash);
        }
    }

    cache.invalidate_paths(canonical_paths.iter().cloned());
    let result = compile_inner(args, cwd, Some(cache), Some(&canonical_paths), None)?;

    let exports_changed = canonical_paths.iter().any(|path| {
        old_hashes.get(path).copied() != cache.export_hashes.get(path).copied()
    });
    if !exports_changed {
        return Ok(result);
    }

    let dependents = cache.collect_dependents(canonical_paths.iter().cloned());
    cache.invalidate_paths_with_dependents_symbols(canonical_paths.into_iter());
    compile_inner(
        args,
        cwd,
        Some(cache),
        Some(changed_paths),
        Some(&dependents),
    )
}

fn compile_inner(
    args: &CliArgs,
    cwd: &Path,
    mut cache: Option<&mut CompilationCache>,
    changed_paths: Option<&[PathBuf]>,
    forced_dirty_paths: Option<&HashSet<PathBuf>>,
) -> Result<CompilationResult> {
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

    let changed_set = changed_paths.map(|paths| {
        paths
            .iter()
            .map(|path| canonicalize_or_owned(path))
            .collect::<HashSet<_>>()
    });
    let SourceReadResult {
        sources,
        dependencies,
    } = {
        let cache_ref = cache.as_deref();
        read_source_files(
            &file_paths,
            &base_dir,
            &resolved,
            cache_ref,
            changed_set.as_ref(),
        )?
    };
    if let Some(cache) = cache.as_deref_mut() {
        cache.update_dependencies(dependencies);
    }

    let (program, dirty_paths) = if let Some(cache) = cache.as_deref_mut() {
        let result = build_program_with_cache(sources, cache);
        (result.program, Some(result.dirty_paths))
    } else {
        let compile_inputs: Vec<(String, String)> = sources
            .into_iter()
            .map(|source| {
                let text = source
                    .text
                    .unwrap_or_else(|| panic!("missing source text for {}", source.path.display()));
                (source.path.to_string_lossy().into_owned(), text)
            })
            .collect();
        (parallel::compile_files(compile_inputs), None)
    };
    if let Some(cache) = cache.as_deref_mut() {
        update_import_symbol_ids(&program, &resolved, &base_dir, cache);
    }

    let mut diagnostics = collect_diagnostics(&program, cache);
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

    let mut dirty_paths = dirty_paths;
    if let Some(forced) = forced_dirty_paths {
        match &mut dirty_paths {
            Some(existing) => {
                existing.extend(forced.iter().cloned());
            }
            None => {
                dirty_paths = Some(forced.iter().cloned().collect());
            }
        }
    }

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
            dirty_paths.as_ref(),
        )?;
        write_outputs(&outputs)?
    };

    Ok(CompilationResult {
        diagnostics,
        emitted_files,
    })
}

struct SourceMeta {
    path: PathBuf,
    file_name: String,
    hash: u64,
    cached_ok: bool,
}

struct BuildProgramResult {
    program: MergedProgram,
    dirty_paths: HashSet<PathBuf>,
}

fn build_program_with_cache(
    sources: Vec<SourceEntry>,
    cache: &mut CompilationCache,
) -> BuildProgramResult {
    let mut meta = Vec::with_capacity(sources.len());
    let mut to_parse = Vec::new();
    let mut dirty_paths = HashSet::new();

    for source in sources {
        let file_name = source.path.to_string_lossy().into_owned();
        let (hash, cached_ok) = match source.text {
            Some(text) => {
                let hash = hash_text(&text);
                let cached_ok = cache
                    .bind_cache
                    .get(&source.path)
                    .map(|entry| entry.hash == hash)
                    .unwrap_or(false);
                if !cached_ok {
                    dirty_paths.insert(source.path.clone());
                    to_parse.push((file_name.clone(), text));
                }
                (hash, cached_ok)
            }
            None => {
                let cached = cache.bind_cache.get(&source.path).unwrap_or_else(|| {
                    panic!("missing cached bind result for {}", source.path.display());
                });
                (cached.hash, true)
            }
        };

        meta.push(SourceMeta {
            path: source.path,
            file_name,
            hash,
            cached_ok,
        });
    }

    let parsed_results = if to_parse.is_empty() {
        Vec::new()
    } else {
        parallel::parse_and_bind_parallel(to_parse)
    };

    let mut parsed_map: HashMap<String, BindResult> = parsed_results
        .into_iter()
        .map(|result| (result.file_name.clone(), result))
        .collect();

    for entry in &meta {
        if entry.cached_ok {
            continue;
        }

        let result = parsed_map
            .remove(&entry.file_name)
            .unwrap_or_else(|| {
                panic!("missing parse result for {}", entry.file_name);
            });
        cache.bind_cache.insert(
            entry.path.clone(),
            BindCacheEntry {
                hash: entry.hash,
                bind_result: result,
            },
        );
    }

    let mut current_paths = HashSet::with_capacity(meta.len());
    for entry in &meta {
        current_paths.insert(entry.path.clone());
    }
    cache
        .bind_cache
        .retain(|path, _| current_paths.contains(path));

    let mut ordered = Vec::with_capacity(meta.len());
    for entry in &meta {
        let Some(cached) = cache.bind_cache.get(&entry.path) else {
            continue;
        };
        ordered.push(&cached.bind_result);
    }

    BuildProgramResult {
        program: parallel::merge_bind_results_ref(&ordered),
        dirty_paths,
    }
}

fn update_import_symbol_ids(
    program: &MergedProgram,
    options: &ResolvedCompilerOptions,
    base_dir: &Path,
    cache: &mut CompilationCache,
) {
    let mut resolution_cache = ModuleResolutionCache::default();
    let mut import_symbol_ids: HashMap<PathBuf, HashMap<PathBuf, Vec<SymbolId>>> = HashMap::new();

    for (file_idx, file) in program.files.iter().enumerate() {
        let file_path = PathBuf::from(&file.file_name);
        let mut by_dep: HashMap<PathBuf, Vec<SymbolId>> = HashMap::new();
        for (specifier, local_names) in collect_import_bindings(&file.arena, file.source_file) {
            let resolved = resolve_module_specifier(
                Path::new(&file.file_name),
                &specifier,
                options,
                base_dir,
                &mut resolution_cache,
            );
            let Some(resolved) = resolved else {
                continue;
            };
            let canonical = canonicalize_or_owned(&resolved);
            let entry = by_dep.entry(canonical).or_insert_with(Vec::new);
            if let Some(file_locals) = program.file_locals.get(file_idx) {
                for name in local_names {
                    if let Some(sym_id) = file_locals.get(&name) {
                        entry.push(sym_id);
                    }
                }
            }
        }
        for symbols in by_dep.values_mut() {
            symbols.sort_by_key(|sym| sym.0);
            symbols.dedup();
        }
        import_symbol_ids.insert(file_path, by_dep);
    }

    cache.import_symbol_ids = import_symbol_ids;
}

fn hash_text(text: &str) -> u64 {
    let mut hasher = FxHasher::default();
    text.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone)]
struct SourceEntry {
    path: PathBuf,
    text: Option<String>,
}

struct SourceReadResult {
    sources: Vec<SourceEntry>,
    dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
}

#[derive(Debug, Clone)]
struct OutputFile {
    path: PathBuf,
    contents: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageType {
    Module,
    CommonJs,
}

#[derive(Default)]
struct ModuleResolutionCache {
    package_type_by_dir: HashMap<PathBuf, Option<PackageType>>,
}

impl ModuleResolutionCache {
    fn package_type_for_dir(&mut self, dir: &Path, base_dir: &Path) -> Option<PackageType> {
        let mut current = dir;
        let mut visited = Vec::new();

        loop {
            if let Some(value) = self.package_type_by_dir.get(current).copied() {
                for path in visited {
                    self.package_type_by_dir.insert(path, value);
                }
                return value;
            }

            visited.push(current.to_path_buf());

            if let Some(package_json) = read_package_json(&current.join("package.json")) {
                let value = package_type_from_json(Some(&package_json));
                for path in visited {
                    self.package_type_by_dir.insert(path, value);
                }
                return value;
            }

            if current == base_dir {
                for path in visited {
                    self.package_type_by_dir.insert(path, None);
                }
                return None;
            }

            let Some(parent) = current.parent() else {
                for path in visited {
                    self.package_type_by_dir.insert(path, None);
                }
                return None;
            };
            current = parent;
        }
    }
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
    cache: Option<&CompilationCache>,
    changed_paths: Option<&HashSet<PathBuf>>,
) -> Result<SourceReadResult> {
    let mut sources: HashMap<PathBuf, Option<String>> = HashMap::new();
    let mut dependencies: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
    let mut seen = HashSet::new();
    let mut pending = VecDeque::new();
    let mut resolution_cache = ModuleResolutionCache::default();
    let use_cache = cache.is_some() && changed_paths.is_some();

    for path in paths {
        let canonical = canonicalize_or_owned(path);
        if seen.insert(canonical.clone()) {
            pending.push_back(canonical);
        }
    }

    while let Some(path) = pending.pop_front() {
        if use_cache {
            if let (Some(cache), Some(changed_paths)) = (cache, changed_paths) {
                if !changed_paths.contains(&path) {
                    if let (Some(_), Some(cached_deps)) = (
                        cache.bind_cache.get(&path),
                        cache.dependencies.get(&path),
                    ) {
                        dependencies.insert(path.clone(), cached_deps.clone());
                        sources.insert(path.clone(), None);
                        for dep in cached_deps {
                            if seen.insert(dep.clone()) {
                                pending.push_back(dep.clone());
                            }
                        }
                        continue;
                    }
                }
            }
        }

        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let specifiers = collect_module_specifiers_from_text(&path, &text);
        sources.insert(path.clone(), Some(text));
        let entry = dependencies.entry(path.clone()).or_insert_with(HashSet::new);

        for specifier in specifiers {
            if let Some(resolved) =
                resolve_module_specifier(&path, &specifier, options, base_dir, &mut resolution_cache)
            {
                let canonical = canonicalize_or_owned(&resolved);
                entry.insert(canonical.clone());
                if seen.insert(canonical.clone()) {
                    pending.push_back(canonical);
                }
            }
        }
    }

    let mut list: Vec<SourceEntry> = sources
        .into_iter()
        .map(|(path, text)| SourceEntry { path, text })
        .collect();
    list.sort_by(|left, right| left.path.to_string_lossy().cmp(&right.path.to_string_lossy()));
    Ok(SourceReadResult {
        sources: list,
        dependencies,
    })
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

fn collect_import_bindings(
    arena: &ThinNodeArena,
    source_file: NodeIndex,
) -> Vec<(String, Vec<String>)> {
    let mut bindings = Vec::new();
    let Some(node) = arena.get(source_file) else {
        return bindings;
    };
    let Some(source) = arena.get_source_file(node) else {
        return bindings;
    };

    for &stmt_idx in &source.statements.nodes {
        if stmt_idx.is_none() {
            continue;
        }
        let Some(stmt) = arena.get(stmt_idx) else {
            continue;
        };
        let Some(import_decl) = arena.get_import_decl(stmt) else {
            continue;
        };
        let Some(specifier) = arena.get_literal_text(import_decl.module_specifier) else {
            continue;
        };
        let local_names = collect_import_local_names(arena, import_decl);
        if !local_names.is_empty() {
            bindings.push((specifier.to_string(), local_names));
        }
    }

    bindings
}

fn collect_import_local_names(arena: &ThinNodeArena, import_decl: &crate::parser::thin_node::ImportDeclData) -> Vec<String> {
    let mut names = Vec::new();
    if import_decl.import_clause.is_none() {
        return names;
    }

    let clause_idx = import_decl.import_clause;
    if let Some(clause_node) = arena.get(clause_idx) {
        if let Some(clause) = arena.get_import_clause(clause_node) {
            if !clause.name.is_none() {
                if let Some(name) = arena.get_identifier_text(clause.name) {
                    names.push(name.to_string());
                }
            }

            if !clause.named_bindings.is_none() {
                if let Some(bindings_node) = arena.get(clause.named_bindings) {
                    if bindings_node.kind == SyntaxKind::Identifier as u16 {
                        if let Some(name) = arena.get_identifier_text(clause.named_bindings) {
                            names.push(name.to_string());
                        }
                    } else if let Some(named) = arena.get_named_imports(bindings_node) {
                        if !named.name.is_none() {
                            if let Some(name) = arena.get_identifier_text(named.name) {
                                names.push(name.to_string());
                            }
                        }
                        for &spec_idx in &named.elements.nodes {
                            let Some(spec_node) = arena.get(spec_idx) else {
                                continue;
                            };
                            let Some(spec) = arena.get_specifier(spec_node) else {
                                continue;
                            };
                            let local_ident = if !spec.name.is_none() {
                                spec.name
                            } else {
                                spec.property_name
                            };
                            if let Some(name) = arena.get_identifier_text(local_ident) {
                                names.push(name.to_string());
                            }
                        }
                    }
                }
            }
        } else if let Some(name) = arena.get_identifier_text(clause_idx) {
            names.push(name.to_string());
        }
    } else if let Some(name) = arena.get_identifier_text(clause_idx) {
        names.push(name.to_string());
    }

    names
}

fn resolve_module_specifier(
    from_file: &Path,
    module_specifier: &str,
    options: &ResolvedCompilerOptions,
    base_dir: &Path,
    resolution_cache: &mut ModuleResolutionCache,
) -> Option<PathBuf> {
    let specifier = module_specifier.trim();
    if specifier.is_empty() {
        return None;
    }
    let specifier = specifier.replace('\\', "/");
    let mut candidates = Vec::new();

    let resolution = options.effective_module_resolution();
    let from_dir = from_file.parent().unwrap_or(base_dir);
    let package_type = match resolution {
        ModuleResolutionKind::Node16 | ModuleResolutionKind::NodeNext => {
            resolution_cache.package_type_for_dir(from_dir, base_dir)
        }
        _ => None,
    };

    let mut allow_node_modules = false;

    if Path::new(&specifier).is_absolute() {
        candidates.extend(expand_module_path_candidates(
            &PathBuf::from(specifier.as_str()),
            options,
            package_type,
        ));
    } else if specifier.starts_with('.') {
        let joined = from_dir.join(&specifier);
        candidates.extend(expand_module_path_candidates(&joined, options, package_type));
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
                    candidates.extend(expand_module_path_candidates(&path, options, package_type));
                }
            }
        }

        if candidates.is_empty() {
            candidates.extend(expand_module_path_candidates(
                &base_url.join(&specifier),
                options,
                package_type,
            ));
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
        return resolve_node_module_specifier(from_file, &specifier, base_dir, options);
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

fn expand_module_path_candidates(
    path: &Path,
    options: &ResolvedCompilerOptions,
    package_type: Option<PackageType>,
) -> Vec<PathBuf> {
    let base = normalize_path(path);
    if let Some(extension) = base.extension().and_then(|ext| ext.to_str()) {
        let resolution = options.effective_module_resolution();
        if matches!(
            resolution,
            ModuleResolutionKind::Node16 | ModuleResolutionKind::NodeNext
        ) {
            if let Some(rewritten) = node16_extension_substitution(&base, extension) {
                return rewritten;
            }
        }
        return vec![base];
    }

    let extensions = extension_candidates_for_resolution(options, package_type);
    let mut candidates = Vec::new();
    for ext in extensions {
        candidates.push(base.with_extension(ext));
    }
    for ext in extensions {
        candidates.push(base.join("index").with_extension(ext));
    }
    candidates
}

fn node16_extension_substitution(path: &Path, extension: &str) -> Option<Vec<PathBuf>> {
    let replacements: &[&str] = match extension {
        "js" => &["ts", "tsx", "d.ts"],
        "jsx" => &["tsx", "d.ts"],
        "mjs" => &["mts", "d.mts"],
        "cjs" => &["cts", "d.cts"],
        _ => return None,
    };

    Some(replacements.iter().map(|ext| path.with_extension(ext)).collect())
}

fn extension_candidates_for_resolution(
    options: &ResolvedCompilerOptions,
    package_type: Option<PackageType>,
) -> &'static [&'static str] {
    match options.effective_module_resolution() {
        ModuleResolutionKind::Node16 | ModuleResolutionKind::NodeNext => match package_type {
            Some(PackageType::Module) => &NODE16_MODULE_EXTENSION_CANDIDATES,
            Some(PackageType::CommonJs) => &NODE16_COMMONJS_EXTENSION_CANDIDATES,
            None => &TS_EXTENSION_CANDIDATES,
        },
        _ => &TS_EXTENSION_CANDIDATES,
    }
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
const NODE16_MODULE_EXTENSION_CANDIDATES: [&str; 7] = [
    "mts", "d.mts", "ts", "tsx", "d.ts", "cts", "d.cts",
];
const NODE16_COMMONJS_EXTENSION_CANDIDATES: [&str; 7] = [
    "cts", "d.cts", "ts", "tsx", "d.ts", "mts", "d.mts",
];

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
    #[serde(default, rename = "type")]
    package_type: Option<String>,
    #[serde(default)]
    exports: Option<serde_json::Value>,
}

fn export_conditions(options: &ResolvedCompilerOptions) -> Vec<&'static str> {
    let resolution = options.effective_module_resolution();
    let mut conditions = Vec::new();
    push_condition(&mut conditions, "types");

    match resolution {
        ModuleResolutionKind::Bundler => push_condition(&mut conditions, "browser"),
        ModuleResolutionKind::Node | ModuleResolutionKind::Node16 | ModuleResolutionKind::NodeNext => {
            push_condition(&mut conditions, "node");
        }
    }

    match options.printer.module {
        ModuleKind::CommonJS | ModuleKind::AMD | ModuleKind::UMD | ModuleKind::System => {
            push_condition(&mut conditions, "require");
        }
        ModuleKind::ES2015
        | ModuleKind::ES2020
        | ModuleKind::ES2022
        | ModuleKind::ESNext
        | ModuleKind::Node16
        | ModuleKind::NodeNext => {
            push_condition(&mut conditions, "import");
        }
        _ => {}
    }

    push_condition(&mut conditions, "default");
    match resolution {
        ModuleResolutionKind::Bundler => {
            push_condition(&mut conditions, "import");
            push_condition(&mut conditions, "require");
            push_condition(&mut conditions, "node");
        }
        ModuleResolutionKind::Node | ModuleResolutionKind::Node16 | ModuleResolutionKind::NodeNext => {
            push_condition(&mut conditions, "import");
            push_condition(&mut conditions, "require");
            push_condition(&mut conditions, "browser");
        }
    }

    conditions
}

fn push_condition(conditions: &mut Vec<&'static str>, condition: &'static str) {
    if !conditions.iter().any(|&value| value == condition) {
        conditions.push(condition);
    }
}

fn resolve_node_module_specifier(
    from_file: &Path,
    module_specifier: &str,
    base_dir: &Path,
    options: &ResolvedCompilerOptions,
) -> Option<PathBuf> {
    let (package_name, subpath) = split_package_specifier(module_specifier)?;
    let conditions = export_conditions(options);
    let mut current = from_file.parent().unwrap_or(base_dir);

    loop {
        let package_root = current.join("node_modules").join(&package_name);
        if package_root.is_dir() {
            let package_json = read_package_json(&package_root.join("package.json"));
            let resolved = resolve_package_specifier(
                &package_root,
                subpath.as_deref(),
                package_json.as_ref(),
                &conditions,
                options,
            );
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

fn resolve_package_specifier(
    package_root: &Path,
    subpath: Option<&str>,
    package_json: Option<&PackageJson>,
    conditions: &[&str],
    options: &ResolvedCompilerOptions,
) -> Option<PathBuf> {
    let package_type = package_type_from_json(package_json);
    if let Some(package_json) = package_json {
        if let Some(exports) = package_json.exports.as_ref() {
            let subpath_key = match subpath {
                Some(value) => format!("./{}", value),
                None => ".".to_string(),
            };
            if let Some(target) = resolve_exports_subpath(exports, &subpath_key, conditions) {
                if let Some(resolved) =
                    resolve_package_entry(package_root, &target, options, package_type)
                {
                    return Some(resolved);
                }
            }
        }
    }

    if let Some(subpath) = subpath {
        return resolve_package_entry(package_root, subpath, options, package_type);
    }

    resolve_package_root(package_root, package_json, options, package_type)
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

fn resolve_package_root(
    package_root: &Path,
    package_json: Option<&PackageJson>,
    options: &ResolvedCompilerOptions,
    package_type: Option<PackageType>,
) -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(package_json) = package_json {
        candidates = collect_package_entry_candidates(package_json);
    }

    if !candidates.iter().any(|entry| entry == "index" || entry == "./index") {
        candidates.push("index".to_string());
    }

    for entry in candidates {
        if let Some(resolved) =
            resolve_package_entry(package_root, &entry, options, package_type)
        {
            return Some(resolved);
        }
    }

    None
}

fn resolve_package_entry(
    package_root: &Path,
    entry: &str,
    options: &ResolvedCompilerOptions,
    package_type: Option<PackageType>,
) -> Option<PathBuf> {
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

    for candidate in expand_module_path_candidates(&path, options, package_type) {
        if candidate.is_file() && is_ts_file(&candidate) {
            return Some(canonicalize_or_owned(&candidate));
        }
    }

    None
}

fn package_type_from_json(package_json: Option<&PackageJson>) -> Option<PackageType> {
    let Some(package_json) = package_json else {
        return None;
    };

    match package_json.package_type.as_deref() {
        Some("module") => Some(PackageType::Module),
        Some("commonjs") => Some(PackageType::CommonJs),
        Some(_) => None,
        None => Some(PackageType::CommonJs),
    }
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

fn resolve_exports_subpath(
    exports: &serde_json::Value,
    subpath_key: &str,
    conditions: &[&str],
) -> Option<String> {
    match exports {
        serde_json::Value::String(value) => {
            if subpath_key == "." {
                Some(value.clone())
            } else {
                None
            }
        }
        serde_json::Value::Array(list) => {
            for entry in list {
                if let Some(resolved) = resolve_exports_subpath(entry, subpath_key, conditions) {
                    return Some(resolved);
                }
            }
            None
        }
        serde_json::Value::Object(map) => {
            let has_subpath_keys = map.keys().any(|key| key.starts_with('.'));
            if has_subpath_keys {
                if let Some(value) = map.get(subpath_key) {
                    if let Some(target) = resolve_exports_target(value, conditions) {
                        return Some(target);
                    }
                }

                let mut best_match: Option<(usize, String, &serde_json::Value)> = None;
                for (key, value) in map {
                    let Some(wildcard) = match_exports_subpath(key, subpath_key) else {
                        continue;
                    };
                    let specificity = key.len();
                    let is_better = match &best_match {
                        None => true,
                        Some((best_len, _, _)) => specificity > *best_len,
                    };
                    if is_better {
                        best_match = Some((specificity, wildcard, value));
                    }
                }

                if let Some((_, wildcard, value)) = best_match {
                    if let Some(target) = resolve_exports_target(value, conditions) {
                        return Some(apply_exports_subpath(&target, &wildcard));
                    }
                }

                None
            } else if subpath_key == "." {
                resolve_exports_target(exports, conditions)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn resolve_exports_target(
    target: &serde_json::Value,
    conditions: &[&str],
) -> Option<String> {
    match target {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Array(list) => {
            for entry in list {
                if let Some(resolved) = resolve_exports_target(entry, conditions) {
                    return Some(resolved);
                }
            }
            None
        }
        serde_json::Value::Object(map) => {
            for condition in conditions {
                if let Some(value) = map.get(*condition) {
                    if let Some(resolved) = resolve_exports_target(value, conditions) {
                        return Some(resolved);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn match_exports_subpath(pattern: &str, subpath_key: &str) -> Option<String> {
    if !pattern.contains('*') {
        return None;
    }
    let pattern = pattern.strip_prefix("./")?;
    let subpath = subpath_key.strip_prefix("./")?;

    let star = pattern.find('*')?;
    let (prefix, suffix) = pattern.split_at(star);
    let suffix = &suffix[1..];

    if !subpath.starts_with(prefix) || !subpath.ends_with(suffix) {
        return None;
    }

    let start = prefix.len();
    let end = subpath.len().saturating_sub(suffix.len());
    if end < start {
        return None;
    }

    Some(subpath[start..end].to_string())
}

fn apply_exports_subpath(target: &str, wildcard: &str) -> String {
    if target.contains('*') {
        target.replace('*', wildcard)
    } else {
        target.to_string()
    }
}

fn collect_diagnostics(
    program: &MergedProgram,
    cache: Option<&mut CompilationCache>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut used_paths = HashSet::new();
    let mut cache = cache;

    for (file_idx, file) in program.files.iter().enumerate() {
        let file_path = PathBuf::from(&file.file_name);
        used_paths.insert(file_path.clone());
        if let Some(cached) = cache
            .as_deref()
            .and_then(|cache| cache.diagnostics.get(&file_path))
        {
            diagnostics.extend(cached.clone());
            continue;
        }

        let binder = create_binder_from_bound_file(file, program, file_idx);
        let cached = cache
            .as_deref_mut()
            .and_then(|cache| cache.type_caches.remove(&file_path));
        let mut checker = if let Some(cached) = cached {
            ThinCheckerState::with_cache(
                &file.arena,
                &binder,
                &program.type_interner,
                file.file_name.clone(),
                cached,
            )
        } else {
            ThinCheckerState::new(
                &file.arena,
                &binder,
                &program.type_interner,
                file.file_name.clone(),
            )
        };
        let mut file_diagnostics = Vec::new();
        for parse_diagnostic in &file.parse_diagnostics {
            file_diagnostics.push(parse_diagnostic_to_checker(
                &file.file_name,
                parse_diagnostic,
            ));
        }
        checker.check_source_file(file.source_file);
        file_diagnostics.extend(std::mem::take(&mut checker.ctx.diagnostics));
        diagnostics.extend(file_diagnostics.clone());
        let export_hash = compute_export_hash(program, file, file_idx, &mut checker);

        if let Some(cache) = cache.as_deref_mut() {
            cache
                .type_caches
                .insert(file_path.clone(), checker.extract_cache());
            cache
                .diagnostics
                .insert(file_path.clone(), file_diagnostics);
            cache.export_hashes.insert(file_path, export_hash);
        }
    }

    if let Some(cache) = cache {
        cache.type_caches.retain(|path, _| used_paths.contains(path));
        cache.diagnostics.retain(|path, _| used_paths.contains(path));
        cache
            .export_hashes
            .retain(|path, _| used_paths.contains(path));
    }

    diagnostics
}

fn compute_export_hash(
    program: &MergedProgram,
    file: &BoundFile,
    file_idx: usize,
    checker: &mut ThinCheckerState,
) -> u64 {
    let mut formatter = TypeFormatter::with_symbols(&program.type_interner, &program.symbols);
    let mut hasher = FxHasher::default();

    if let Some(file_locals) = program.file_locals.get(file_idx) {
        let mut exports: Vec<(&String, SymbolId)> = file_locals
            .iter()
            .filter_map(|(name, &sym_id)| {
                is_exported_symbol(&program.symbols, sym_id).then_some((name, sym_id))
            })
            .collect();
        exports.sort_by(|left, right| left.0.cmp(right.0));

        for (name, sym_id) in exports {
            name.hash(&mut hasher);
            let type_id = checker.get_type_of_symbol(sym_id);
            let type_str = formatter.format(type_id);
            type_str.hash(&mut hasher);
        }
    }

    let mut export_signatures = Vec::new();
    collect_export_signatures(file, checker, &mut formatter, &mut export_signatures);
    export_signatures.sort();
    for signature in export_signatures {
        signature.hash(&mut hasher);
    }

    hasher.finish()
}

fn is_exported_symbol(symbols: &crate::binder::SymbolArena, sym_id: SymbolId) -> bool {
    let Some(symbol) = symbols.get(sym_id) else {
        return false;
    };
    symbol.is_exported || (symbol.flags & symbol_flags::EXPORT_VALUE) != 0
}

fn collect_export_signatures(
    file: &BoundFile,
    checker: &mut ThinCheckerState,
    formatter: &mut TypeFormatter,
    signatures: &mut Vec<String>,
) {
    let arena = &file.arena;
    let Some(node) = arena.get(file.source_file) else {
        return;
    };
    let Some(source) = arena.get_source_file(node) else {
        return;
    };

    for &stmt_idx in &source.statements.nodes {
        let Some(stmt) = arena.get(stmt_idx) else {
            continue;
        };

        if let Some(export_decl) = arena.get_export_decl(stmt) {
            if export_decl.is_default_export {
                if let Some(signature) =
                    export_default_signature(export_decl.export_clause, checker, formatter)
                {
                    signatures.push(signature);
                }
                continue;
            }

            if export_decl.module_specifier.is_none() {
                if !export_decl.export_clause.is_none() {
                    let clause_node = export_decl.export_clause;
                    let clause_node_ref = arena.get(clause_node);
                    if clause_node_ref
                        .and_then(|node| arena.get_named_imports(node))
                        .is_some()
                    {
                        collect_local_named_export_signatures(
                            arena,
                            file.source_file,
                            clause_node,
                            checker,
                            formatter,
                            export_type_prefix(export_decl.is_type_only),
                            signatures,
                        );
                    } else {
                        collect_exported_declaration_signatures(
                            arena,
                            clause_node,
                            checker,
                            formatter,
                            export_type_prefix(export_decl.is_type_only),
                            signatures,
                        );
                    }
                }
                continue;
            }

            let module_spec = arena
                .get_literal_text(export_decl.module_specifier)
                .unwrap_or("")
                .to_string();
            if export_decl.export_clause.is_none() {
                signatures.push(format!(
                    "{}*|{}",
                    export_type_prefix(export_decl.is_type_only),
                    module_spec
                ));
                continue;
            }

            let clause_node = export_decl.export_clause;
            let clause_node_ref = arena.get(clause_node);
            if let Some(named) = clause_node_ref.and_then(|node| arena.get_named_imports(node)) {
                let mut specifiers = Vec::new();
                for &spec_idx in &named.elements.nodes {
                    let Some(spec_node) = arena.get(spec_idx) else {
                        continue;
                    };
                    let Some(spec) = arena.get_specifier(spec_node) else {
                        continue;
                    };
                    let name = arena.get_identifier_text(spec.name).unwrap_or("");
                    if spec.property_name.is_none() {
                        specifiers.push(name.to_string());
                    } else {
                        let property = arena.get_identifier_text(spec.property_name).unwrap_or("");
                        specifiers.push(format!("{} as {}", property, name));
                    }
                }
                specifiers.sort();
                signatures.push(format!(
                    "{}{{{}}}|{}",
                    export_type_prefix(export_decl.is_type_only),
                    specifiers.join(","),
                    module_spec
                ));
            } else if let Some(name) = arena.get_identifier_text(clause_node) {
                signatures.push(format!(
                    "{}* as {}|{}",
                    export_type_prefix(export_decl.is_type_only),
                    name,
                    module_spec
                ));
            }

            continue;
        }

        if let Some(export_assignment) = arena.get_export_assignment(stmt) {
            if !export_assignment.expression.is_none() {
                let type_id = checker.get_type_of_node(export_assignment.expression);
                let type_str = formatter.format(type_id);
                signatures.push(format!("export=:{type_str}"));
            }
        }
    }
}

fn collect_local_named_export_signatures(
    arena: &ThinNodeArena,
    source_file: NodeIndex,
    named_idx: NodeIndex,
    checker: &mut ThinCheckerState,
    formatter: &mut TypeFormatter,
    type_prefix: &str,
    signatures: &mut Vec<String>,
) {
    let Some(named_node) = arena.get(named_idx) else {
        return;
    };
    let Some(named) = arena.get_named_imports(named_node) else {
        return;
    };

    for &spec_idx in &named.elements.nodes {
        let Some(spec_node) = arena.get(spec_idx) else {
            continue;
        };
        let Some(spec) = arena.get_specifier(spec_node) else {
            continue;
        };
        let exported_name = if !spec.name.is_none() {
            arena.get_identifier_text(spec.name).unwrap_or("")
        } else {
            arena.get_identifier_text(spec.property_name).unwrap_or("")
        };
        if exported_name.is_empty() {
            continue;
        }
        let local_name = if !spec.property_name.is_none() {
            arena.get_identifier_text(spec.property_name).unwrap_or("")
        } else {
            exported_name
        };
        let type_id = find_local_declaration(arena, source_file, local_name)
            .map(|decl_idx| checker.get_type_of_node(decl_idx))
            .unwrap_or(TypeId::ANY);
        let type_str = formatter.format(type_id);
        signatures.push(format!("{type_prefix}{exported_name}:{type_str}"));
    }
}

fn collect_exported_declaration_signatures(
    arena: &ThinNodeArena,
    decl_idx: NodeIndex,
    checker: &mut ThinCheckerState,
    formatter: &mut TypeFormatter,
    type_prefix: &str,
    signatures: &mut Vec<String>,
) {
    let Some(node) = arena.get(decl_idx) else {
        return;
    };

    if node.kind == syntax_kind_ext::VARIABLE_STATEMENT {
        if let Some(var_stmt) = arena.get_variable(node) {
            for &list_idx in &var_stmt.declarations.nodes {
                collect_exported_declaration_signatures(
                    arena,
                    list_idx,
                    checker,
                    formatter,
                    type_prefix,
                    signatures,
                );
            }
        }
        return;
    }

    if node.kind == syntax_kind_ext::VARIABLE_DECLARATION_LIST {
        if let Some(list) = arena.get_variable(node) {
            for &decl_idx in &list.declarations.nodes {
                collect_exported_declaration_signatures(
                    arena,
                    decl_idx,
                    checker,
                    formatter,
                    type_prefix,
                    signatures,
                );
            }
        }
        return;
    }

    if let Some(var_decl) = arena.get_variable_declaration(node) {
        if let Some(name) = arena.get_identifier_text(var_decl.name) {
            push_exported_signature(
                name,
                decl_idx,
                checker,
                formatter,
                type_prefix,
                signatures,
            );
        }
        return;
    }

    if let Some(func) = arena.get_function(node) {
        if let Some(name) = arena.get_identifier_text(func.name) {
            push_exported_signature(
                name,
                decl_idx,
                checker,
                formatter,
                type_prefix,
                signatures,
            );
        }
        return;
    }

    if let Some(class) = arena.get_class(node) {
        if let Some(name) = arena.get_identifier_text(class.name) {
            push_exported_signature(
                name,
                decl_idx,
                checker,
                formatter,
                type_prefix,
                signatures,
            );
        }
        return;
    }

    if let Some(interface) = arena.get_interface(node) {
        if let Some(name) = arena.get_identifier_text(interface.name) {
            push_exported_signature(
                name,
                decl_idx,
                checker,
                formatter,
                type_prefix,
                signatures,
            );
        }
        return;
    }

    if let Some(type_alias) = arena.get_type_alias(node) {
        if let Some(name) = arena.get_identifier_text(type_alias.name) {
            push_exported_signature(
                name,
                decl_idx,
                checker,
                formatter,
                type_prefix,
                signatures,
            );
        }
        return;
    }

    if let Some(enum_decl) = arena.get_enum(node) {
        if let Some(name) = arena.get_identifier_text(enum_decl.name) {
            push_exported_signature(
                name,
                decl_idx,
                checker,
                formatter,
                type_prefix,
                signatures,
            );
        }
        return;
    }

    if let Some(module_decl) = arena.get_module(node) {
        let name = arena
            .get_identifier_text(module_decl.name)
            .or_else(|| arena.get_literal_text(module_decl.name));
        if let Some(name) = name {
            push_exported_signature(
                name,
                decl_idx,
                checker,
                formatter,
                type_prefix,
                signatures,
            );
        }
    }
}

fn push_exported_signature(
    name: &str,
    decl_idx: NodeIndex,
    checker: &mut ThinCheckerState,
    formatter: &mut TypeFormatter,
    type_prefix: &str,
    signatures: &mut Vec<String>,
) {
    let type_id = checker.get_type_of_node(decl_idx);
    let type_str = formatter.format(type_id);
    signatures.push(format!("{type_prefix}{name}:{type_str}"));
}

fn find_local_declaration(
    arena: &ThinNodeArena,
    source_file: NodeIndex,
    name: &str,
) -> Option<NodeIndex> {
    let Some(node) = arena.get(source_file) else {
        return None;
    };
    let Some(source) = arena.get_source_file(node) else {
        return None;
    };

    for &stmt_idx in &source.statements.nodes {
        let Some(stmt) = arena.get(stmt_idx) else {
            continue;
        };
        if let Some(export_decl) = arena.get_export_decl(stmt) {
            if export_decl.export_clause.is_none() {
                continue;
            }
            let clause_idx = export_decl.export_clause;
            let Some(clause_node) = arena.get(clause_idx) else {
                continue;
            };
            if arena.get_named_imports(clause_node).is_some() {
                continue;
            }
            if let Some(found) = find_local_declaration_in_node(arena, clause_idx, name) {
                return Some(found);
            }
            continue;
        }

        if let Some(found) = find_local_declaration_in_node(arena, stmt_idx, name) {
            return Some(found);
        }
    }

    None
}

fn find_local_declaration_in_node(
    arena: &ThinNodeArena,
    node_idx: NodeIndex,
    name: &str,
) -> Option<NodeIndex> {
    let Some(node) = arena.get(node_idx) else {
        return None;
    };

    if let Some(var_decl) = arena.get_variable_declaration(node) {
        if let Some(decl_name) = arena.get_identifier_text(var_decl.name) {
            if decl_name == name {
                return Some(node_idx);
            }
        }
        return None;
    }

    if node.kind == syntax_kind_ext::VARIABLE_STATEMENT {
        if let Some(var_stmt) = arena.get_variable(node) {
            for &list_idx in &var_stmt.declarations.nodes {
                if let Some(found) = find_local_declaration_in_node(arena, list_idx, name) {
                    return Some(found);
                }
            }
        }
        return None;
    }

    if node.kind == syntax_kind_ext::VARIABLE_DECLARATION_LIST {
        if let Some(list) = arena.get_variable(node) {
            for &decl_idx in &list.declarations.nodes {
                if let Some(found) = find_local_declaration_in_node(arena, decl_idx, name) {
                    return Some(found);
                }
            }
        }
        return None;
    }

    if let Some(func) = arena.get_function(node) {
        if let Some(decl_name) = arena.get_identifier_text(func.name) {
            if decl_name == name {
                return Some(node_idx);
            }
        }
        return None;
    }

    if let Some(class) = arena.get_class(node) {
        if let Some(decl_name) = arena.get_identifier_text(class.name) {
            if decl_name == name {
                return Some(node_idx);
            }
        }
        return None;
    }

    if let Some(interface) = arena.get_interface(node) {
        if let Some(decl_name) = arena.get_identifier_text(interface.name) {
            if decl_name == name {
                return Some(node_idx);
            }
        }
        return None;
    }

    if let Some(type_alias) = arena.get_type_alias(node) {
        if let Some(decl_name) = arena.get_identifier_text(type_alias.name) {
            if decl_name == name {
                return Some(node_idx);
            }
        }
        return None;
    }

    if let Some(enum_decl) = arena.get_enum(node) {
        if let Some(decl_name) = arena.get_identifier_text(enum_decl.name) {
            if decl_name == name {
                return Some(node_idx);
            }
        }
        return None;
    }

    if let Some(module_decl) = arena.get_module(node) {
        let decl_name = arena
            .get_identifier_text(module_decl.name)
            .or_else(|| arena.get_literal_text(module_decl.name));
        if let Some(decl_name) = decl_name {
            if decl_name == name {
                return Some(node_idx);
            }
        }
    }

    None
}

fn export_default_signature(
    export_clause: NodeIndex,
    checker: &mut ThinCheckerState,
    formatter: &mut TypeFormatter,
) -> Option<String> {
    if export_clause.is_none() {
        return None;
    }
    let type_id = if let Some(sym_id) = checker.ctx.binder.get_node_symbol(export_clause) {
        checker.get_type_of_symbol(sym_id)
    } else {
        checker.get_type_of_node(export_clause)
    };
    let type_str = formatter.format(type_id);
    Some(format!("default:{type_str}"))
}

fn export_type_prefix(is_type_only: bool) -> &'static str {
    if is_type_only {
        "type:"
    } else {
        ""
    }
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
    dirty_paths: Option<&HashSet<PathBuf>>,
) -> Result<Vec<OutputFile>> {
    let mut outputs = Vec::new();
    let new_line = new_line_str(options.printer.new_line);

    for file in &program.files {
        let input_path = PathBuf::from(&file.file_name);
        if let Some(dirty_paths) = dirty_paths {
            if !dirty_paths.contains(&input_path) {
                continue;
            }
        }

        if let Some(js_path) = js_output_path(base_dir, root_dir, out_dir, options.jsx, &input_path) {
            let mut printer = ThinPrinter::with_options(&file.arena, options.printer.clone());
            printer.emit(file.source_file);
            let mut contents = printer.take_output();
            let mut map_output = None;

            if options.source_map {
                if let Some((map_path, map_name, output_name)) = map_output_info(&js_path) {
                    append_source_mapping_url(&mut contents, &map_name, new_line);
                    let map_json = generate_basic_source_map(&output_name, &file.file_name);
                    map_output = Some(OutputFile {
                        path: map_path,
                        contents: map_json,
                    });
                }
            }

            outputs.push(OutputFile { path: js_path, contents });
            if let Some(map_output) = map_output {
                outputs.push(map_output);
            }
        }

        if options.emit_declarations {
            let decl_base = declaration_dir.or(out_dir);
            if let Some(dts_path) = declaration_output_path(base_dir, root_dir, decl_base, &input_path) {
                let mut emitter = DeclarationEmitter::new(&file.arena);
                let mut contents = emitter.emit(file.source_file);
                let mut map_output = None;

                if options.declaration_map {
                    if let Some((map_path, map_name, output_name)) = map_output_info(&dts_path) {
                        append_source_mapping_url(&mut contents, &map_name, new_line);
                        let map_json = generate_basic_source_map(&output_name, &file.file_name);
                        map_output = Some(OutputFile {
                            path: map_path,
                            contents: map_json,
                        });
                    }
                }

                outputs.push(OutputFile { path: dts_path, contents });
                if let Some(map_output) = map_output {
                    outputs.push(map_output);
                }
            }
        }
    }

    Ok(outputs)
}

fn map_output_info(output_path: &Path) -> Option<(PathBuf, String, String)> {
    let output_name = output_path.file_name()?.to_string_lossy().into_owned();
    let map_name = format!("{output_name}.map");
    let map_path = output_path.with_file_name(&map_name);
    Some((map_path, map_name, output_name))
}

fn generate_basic_source_map(output_name: &str, source_name: &str) -> String {
    let mut map = SourceMapGenerator::new(output_name.to_string());
    let source_index = map.add_source(source_name.to_string());
    map.add_simple_mapping(0, 0, source_index, 0, 0);
    map.generate_json()
}

fn append_source_mapping_url(contents: &mut String, map_name: &str, new_line: &str) {
    if !contents.is_empty() && !contents.ends_with(new_line) {
        contents.push_str(new_line);
    }
    contents.push_str("//# sourceMappingURL=");
    contents.push_str(map_name);
}

fn new_line_str(kind: NewLineKind) -> &'static str {
    match kind {
        NewLineKind::LineFeed => "\n",
        NewLineKind::CarriageReturnLineFeed => "\r\n",
    }
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
