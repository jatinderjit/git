use std::path::{Path, PathBuf};

/// Checks if the path is the path is a git repository root
fn is_repo_root(path: &Path) -> bool {
    let git = path.join(".git");

    // TODO: verify the HEAD file contents (currently assumed to be valid)
    git.is_dir()
        && git.join("HEAD").is_file()
        && git.join("refs").is_dir()
        && git.join("objects").is_dir()
}

/// Return path of the repository root, by traversing the parent folders, until
/// a git repository is found.
pub(crate) fn find_repo_root(path: PathBuf) -> Option<PathBuf> {
    let mut root = Some(path.as_path());
    while let Some(path) = root {
        if is_repo_root(path) {
            return Some(path.to_path_buf());
        }
        root = path.parent();
    }
    None
}
