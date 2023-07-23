use anyhow::{Ok, Result};
use cli::Cli;

pub(crate) mod cli;
pub(crate) mod commands;

pub fn run() -> Result<()> {
    let cli = cli::parse();
    match cli {
        Cli::Init(options) => commands::init(options)?,
    };
    Ok(())
}
