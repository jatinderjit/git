use anyhow::Result;
use std::{fs, path::PathBuf};

use clap::Args;

use crate::objects::{object::Object, ObjectFile};

#[derive(Args, Debug)]
pub(crate) struct HashObjectOptions {
    /// Actually write the object
    #[arg(short)]
    write: bool,

    /// Compute Object ID (hash) of this file
    path: String,
}

pub(crate) fn hash_object(git_dir: PathBuf, options: HashObjectOptions) -> Result<()> {
    let contents = fs::read(options.path)?;
    let object = Object::new_blob(&contents);
    let hash = object.compute_hash();
    if options.write {
        let object_file = ObjectFile::new(&git_dir, &hash);
        object_file.save(&object)?;
    }
    println!("{hash}");
    Ok(())
}
