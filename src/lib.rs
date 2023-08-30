mod context;
mod utils;

use std::env;

use anyhow::{anyhow, Ok, Result};
use cli::Cli;
use context::Context;
use utils::find_repo_root;

pub(crate) mod cli;
pub(crate) mod commands;
pub(crate) mod objects;

pub fn run() -> Result<()> {
    let cli = cli::parse();
    let context = match cli {
        Cli::Init(_) => Context::new(env::current_dir()?),
        _ => {
            let cwd = env::current_dir()?;
            let root = find_repo_root(cwd).ok_or(anyhow!("not a git repository"))?;
            Context::new(root)
        }
    };
    match cli {
        Cli::Init(options) => commands::init(options)?,
        Cli::CatFile(options) => commands::cat_file(&context, options.into())?,
        Cli::HashObject(options) => {
            let hash = commands::hash_object(&context, options)?;
            println!("{hash}");
        }
        Cli::LsTree(options) => commands::ls_tree(context, options)?,
    };
    Ok(())
}
