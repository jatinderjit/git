mod utils;

use std::{env, path::PathBuf};

use anyhow::{anyhow, Ok, Result};
use cli::Cli;
use utils::find_repo_root;

pub(crate) mod cli;
pub(crate) mod commands;
pub(crate) mod objects;

pub fn run() -> Result<()> {
    let cli = cli::parse();
    let git_dir = match cli {
        Cli::Init(_) => PathBuf::new(),
        _ => {
            let cwd = env::current_dir()?;
            let root = find_repo_root(cwd).ok_or(anyhow!("not a git repository"))?;
            root.join(".git")
        }
    };
    match cli {
        Cli::Init(options) => commands::init(options)?,
        Cli::CatFile(options) => commands::cat_file(git_dir, options.into())?,
        Cli::HashObject(options) => commands::hash_object(git_dir, options)?,
        Cli::LsTree(options) => commands::ls_tree(git_dir, options)?,
    };
    Ok(())
}
