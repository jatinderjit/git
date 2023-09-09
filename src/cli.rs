use clap::Parser;

use crate::commands::{CatFileCliOptions, HashObjectOptions, InitOptions, LsTreeOptions};

#[derive(Parser, Debug)]
#[command(version, about)]
pub(crate) enum Cli {
    /// Initializes an empty git repository
    Init(InitOptions),

    /// Provides content/type/size information for repository objects
    CatFile(CatFileCliOptions),

    /// Computes content-hash and (optionally) create a blob
    HashObject(HashObjectOptions),

    /// Displays contents of the tree (or a commit's tree) object
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
