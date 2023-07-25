use clap::Parser;

use crate::commands::{CatFileCliOptions, InitOptions};

#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) enum Cli {
    Init(InitOptions),
    CatFile(CatFileCliOptions),
}

pub(crate) fn parse() -> Cli {
    Cli::parse()
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
