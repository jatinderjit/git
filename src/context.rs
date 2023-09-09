use std::path::PathBuf;

pub struct Context {
    pub repo_root: PathBuf,
    // Though this git_dir doesn't have to be inside the repo root, here we assume git_dir to
    // always be "<repo_root>/.git"
    pub git_dir: PathBuf,
}

impl Context {
    pub(crate) fn new(repo_root: PathBuf) -> Self {
        let git_dir = repo_root.join(".git");
        Self { repo_root, git_dir }
    }
    pub(crate) fn object_dir(&self, hash: &str) -> PathBuf {
        self.git_dir.join("objects").join(&hash[..2])
    }

    pub(crate) fn object_path(&self, hash: &str) -> PathBuf {
        self.object_dir(hash).join(&hash[2..])
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use tempfile::TempDir;

    use super::Context;
    use crate::commands::{self, InitOptions};

    pub struct TestContext {
        _temp_dir: TempDir,
        pub context: Context,
    }

    impl TestContext {
        /// Creates a new temporary directory and sets it as the current working directory.
        /// This directory will be deleted when the value is dropped.
        pub fn no_init() -> Self {
            let temp_dir = TempDir::new().unwrap();
            let context = Context::new(temp_dir.path().to_path_buf());
            Self {
                _temp_dir: temp_dir,
                context,
            }
        }

        pub fn init() -> Self {
            let context = Self::no_init();
            let options = InitOptions {
                directory: Some(context.context.repo_root.to_str().unwrap().to_string()),
                ..Default::default()
            };
            commands::init::init(options).unwrap();
            context
        }
    }
}
