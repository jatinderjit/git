use clap::Parser;

use crate::commands::{CatFileCliOptions, HashObjectOptions, InitOptions, LsTreeOptions};

#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) enum Cli {
    Init(InitOptions),
    CatFile(CatFileCliOptions),
    HashObject(HashObjectOptions),
    LsTree(LsTreeOptions),
}

pub(crate) fn parse() -> Cli {
    Cli::parse()
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
