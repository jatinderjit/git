use std::{env, fs, path::PathBuf};

use anyhow::Result;
use clap::Args;

const DEFAULT_BRANCH: &str = "main";

#[derive(Args, Debug)]
pub(crate) struct InitOptions {
    /// Path where git directory will be created [default: current working directory]
    pub(crate) directory: Option<String>,

    #[arg(short = 'b', long, name = "BRANCH_NAME", default_value = DEFAULT_BRANCH)]
    pub(crate) initial_branch: String,
}

/// Initialize a git directory.
pub(crate) fn init(options: InitOptions) -> Result<()> {
    let repo_root = match options.directory {
        Some(path) => PathBuf::from(path),
        None => env::current_dir()?,
    };
    fs::create_dir_all(repo_root.clone())?;

    // TODO: handle when repo is already initialized.
    let git_root = repo_root.join(".git");
    fs::create_dir_all(git_root.join("objects"))?;
    fs::create_dir_all(git_root.join("refs").join("heads"))?;
    fs::create_dir_all(git_root.join("refs").join("tags"))?;
    fs::write(
        git_root.join("HEAD"),
        format!("ref: refs/heads/{}\n", options.initial_branch),
    )?;
    println!(
        "Initialized empty Git repository in {}",
        git_root.to_string_lossy()
    );
    Ok(())
}
