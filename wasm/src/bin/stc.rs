use anyhow::{Context, Result};
use std::io::IsTerminal;
use clap::Parser;

use wasm::cli::args::CliArgs;
use wasm::cli::{driver, reporter::Reporter};

fn main() -> Result<()> {
    let args = CliArgs::parse();
    let cwd = std::env::current_dir().context("failed to resolve current directory")?;
    let result = driver::compile(&args, &cwd)?;

    if !result.diagnostics.is_empty() {
        let color = std::io::stdout().is_terminal();
        let mut reporter = Reporter::new(color);
        let output = reporter.render(&result.diagnostics);
        if !output.is_empty() {
            eprintln!("{output}");
        }
    }

    let has_errors = result
        .diagnostics
        .iter()
        .any(|diag| diag.category == wasm::checker::types::diagnostics::DiagnosticCategory::Error);

    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}
