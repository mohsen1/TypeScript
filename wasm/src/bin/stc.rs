use anyhow::{Context, Result};
use clap::Parser;

use wasm::cli::args::CliArgs;
use wasm::cli::driver;

fn main() -> Result<()> {
    let args = CliArgs::parse();
    let cwd = std::env::current_dir().context("failed to resolve current directory")?;
    let result = driver::compile(&args, &cwd)?;

    let has_errors = result
        .diagnostics
        .iter()
        .any(|diag| diag.category == wasm::checker::types::diagnostics::DiagnosticCategory::Error);

    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}
