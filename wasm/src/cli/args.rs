use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use crate::thin_emitter::{ModuleKind, ScriptTarget};

/// CLI arguments for the stc binary.
#[derive(Parser, Debug)]
#[command(name = "stc", version, about = "Speedy TypeScript Compiler")]
pub struct CliArgs {
    /// ECMAScript target version.
    #[arg(long, value_enum, default_value_t = Target::EsNext)]
    pub target: Target,

    /// Module system for emitted JavaScript.
    #[arg(long, value_enum, default_value_t = Module::None)]
    pub module: Module,

    /// Output directory for emitted files.
    #[arg(long = "outDir", alias = "out-dir")]
    pub out_dir: Option<PathBuf>,

    /// Enable strict type checking.
    #[arg(long)]
    pub strict: bool,

    /// Skip emitting output files.
    #[arg(long = "noEmit", alias = "no-emit")]
    pub no_emit: bool,

    /// Input files to compile.
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Target {
    Es3,
    Es5,
    #[value(alias = "es6")]
    Es2015,
    Es2016,
    Es2017,
    Es2018,
    Es2019,
    Es2020,
    Es2021,
    Es2022,
    #[value(name = "esnext", alias = "es-next")]
    EsNext,
}

impl Target {
    pub fn to_script_target(self) -> ScriptTarget {
        match self {
            Target::Es3 => ScriptTarget::ES3,
            Target::Es5 => ScriptTarget::ES5,
            Target::Es2015 => ScriptTarget::ES2015,
            Target::Es2016 => ScriptTarget::ES2016,
            Target::Es2017 => ScriptTarget::ES2017,
            Target::Es2018 => ScriptTarget::ES2018,
            Target::Es2019 => ScriptTarget::ES2019,
            Target::Es2020 => ScriptTarget::ES2020,
            Target::Es2021 => ScriptTarget::ES2021,
            Target::Es2022 => ScriptTarget::ES2022,
            Target::EsNext => ScriptTarget::ESNext,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Module {
    None,
    #[value(name = "commonjs", alias = "common-js")]
    CommonJs,
    Amd,
    Umd,
    System,
    #[value(alias = "es6")]
    Es2015,
    Es2020,
    Es2022,
    #[value(name = "esnext", alias = "es-next")]
    EsNext,
    #[value(name = "node16", alias = "node-16")]
    Node16,
    #[value(name = "nodenext", alias = "node-next")]
    NodeNext,
}

impl Module {
    pub fn to_module_kind(self) -> ModuleKind {
        match self {
            Module::None => ModuleKind::None,
            Module::CommonJs => ModuleKind::CommonJS,
            Module::Amd => ModuleKind::AMD,
            Module::Umd => ModuleKind::UMD,
            Module::System => ModuleKind::System,
            Module::Es2015 => ModuleKind::ES2015,
            Module::Es2020 => ModuleKind::ES2020,
            Module::Es2022 => ModuleKind::ES2022,
            Module::EsNext => ModuleKind::ESNext,
            Module::Node16 => ModuleKind::Node16,
            Module::NodeNext => ModuleKind::NodeNext,
        }
    }
}
