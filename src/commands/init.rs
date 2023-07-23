use std::{env, fs, path::PathBuf};

use anyhow::Result;

use crate::cli::Init;

pub(crate) fn init(options: Init) -> Result<()> {
    let repo_root = match options.path {
        Some(path) => PathBuf::from(path),
        None => env::current_dir()?,
    };
    fs::create_dir_all(repo_root.clone())?;

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
