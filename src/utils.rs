use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

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

pub fn zlib_decode(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut d = ZlibDecoder::new(bytes);
    let mut buffer = Vec::new();
    d.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn zlib_encode(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut e = ZlibEncoder::new(&mut buffer, Compression::best());
    e.write_all(bytes)?;
    e.finish()?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::find_repo_root;
    use super::is_repo_root;
    use crate::commands::hash_object::hash_object;
    use crate::commands::init::init;
    use crate::commands::HashObjectOptions;
    use crate::commands::InitOptions;
    use crate::context::tests::TestContext;
    use crate::objects::find_hash;

    #[test]
    fn test_is_repo_root() {
        let context = TestContext::no_init();
        let context = &context.context;

        assert!(!is_repo_root(&context.repo_root));

        let options = InitOptions {
            directory: Some(context.repo_root.to_str().unwrap().to_string()),
            ..Default::default()
        };
        init(options).unwrap();
        assert!(is_repo_root(&context.repo_root));

        let sub_dir = &context.repo_root.join("sub");
        fs::create_dir(sub_dir).unwrap();
        assert!(!is_repo_root(sub_dir));
    }

    #[test]
    fn test_find_repo_root() {
        let context = TestContext::no_init();
        let context = &context.context;

        assert!(find_repo_root(context.repo_root.clone()).is_none());

        let options = InitOptions {
            directory: Some(context.repo_root.to_str().unwrap().to_string()),
            ..Default::default()
        };
        init(options).unwrap();
        let root = find_repo_root(context.repo_root.clone());
        assert!(root.is_some());
        assert_eq!(root.unwrap(), context.repo_root);

        let sub_dir = context.repo_root.join("sub");
        fs::create_dir(sub_dir.clone()).unwrap();
        let root = find_repo_root(sub_dir);
        assert!(root.is_some());
        assert_eq!(root.unwrap(), context.repo_root);
    }

    #[test]
    fn test_find_hash() {
        let context = TestContext::init();
        let context = &context.context;

        let fp = context.repo_root.join("test.txt");
        fs::write(&fp, "This is a test file.\n").unwrap();

        let options = HashObjectOptions {
            path: fp.to_str().unwrap().to_string(),
            write: true,
        };
        let hash = hash_object(&context, options).unwrap(); // 6de7b8c69d65923eb48b10a560f3d72939df256a

        let found = find_hash(&context, &hash);
        assert!(found.is_ok());
        assert_eq!(found.unwrap(), hash);

        let found = find_hash(&context, &hash[..4]);
        assert!(found.is_ok());
        assert_eq!(found.unwrap(), hash);

        let found = find_hash(context, "aaaaa");
        assert!(found.is_err());

        let found = find_hash(context, &hash[..3]);
        assert!(found.is_err());
    }
}
