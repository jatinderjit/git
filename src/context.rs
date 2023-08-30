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
    use crate::commands::{self, InitOptions};
    use std::cell::RefCell;
    use std::sync::Mutex;
    use std::{env, fs};

    use super::Context;

    static COUNT: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

    pub struct TestContext {
        pub context: Context,
    }

    impl TestContext {
        /// Creates a new temporary directory and sets it as the current working directory.
        /// This directory will be deleted when the value is dropped.
        pub fn no_init() -> Self {
            let guard = COUNT.lock().unwrap();
            *guard.borrow_mut() += 1;
            let value = *guard.borrow();

            // TODO: use an actual random dir
            let dir_name = format!("hpj5vkiu8ftxkrak-{}", value);
            let repo_root = env::temp_dir().join(dir_name);

            assert!(!repo_root.exists(), "Path already exists: {:?}", repo_root);
            fs::create_dir(&repo_root).unwrap();
            let context = Context::new(repo_root);
            Self { context }
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

    impl Drop for TestContext {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.context.repo_root).unwrap();
        }
    }
}
