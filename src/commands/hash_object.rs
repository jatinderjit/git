use std::fs;

use anyhow::Result;
use clap::Args;

use crate::{
    context::Context,
    objects::{object::Object, ObjectFile},
};

#[derive(Args, Debug)]
pub(crate) struct HashObjectOptions {
    /// Actually write the object
    #[arg(short)]
    pub(crate) write: bool,

    /// Compute Object ID (hash) of this file
    pub(crate) path: String,
}

pub(crate) fn hash_object(context: &Context, options: HashObjectOptions) -> Result<String> {
    let contents = fs::read(options.path)?;
    let object = Object::new_blob(&contents);
    let hash = object.compute_hash();
    if options.write {
        let object_file = ObjectFile::new(context, &hash);
        object_file.save(&object)?;
    }
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::{hash_object, HashObjectOptions};
    use crate::context::tests::TestContext;
    use std::fs;

    #[test]
    fn hash_object_create() {
        let context = TestContext::init();
        let context = &context.context;

        let fp = context.repo_root.join("test.txt");
        fs::write(&fp, "This is a test file.\n").unwrap();

        let options = HashObjectOptions {
            path: fp.to_string_lossy().to_string(),
            write: true,
        };
        let hash = hash_object(context, options).unwrap();
        assert_eq!(hash, "6de7b8c69d65923eb48b10a560f3d72939df256a");
        assert!(context.object_path(&hash).exists());
    }

    #[test]
    fn hash_object_no_create() {
        let context = TestContext::init();
        let context = &context.context;

        let fp = context.repo_root.join("test.txt");
        fs::write(&fp, "This is a test file.\n").unwrap();

        let options = HashObjectOptions {
            path: fp.to_string_lossy().to_string(),
            write: false,
        };
        let hash = hash_object(&context, options).unwrap();
        assert_eq!(hash, "6de7b8c69d65923eb48b10a560f3d72939df256a");
        assert!(!context.object_path(&hash).exists());
    }
}
