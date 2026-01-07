use anyhow::Result;
use clap::Parser;

use wasm::cli::args::CliArgs;

fn main() -> Result<()> {
    let _args = CliArgs::parse();
    Ok(())
}
