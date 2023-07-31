use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::objects::{find_hash, object::Contents, ObjectFile};

#[derive(Args, Debug)]
pub(crate) struct CatFileCliOptions {
    #[command(flatten)]
    flag: DisplayFlagGroup,

    /// The object name (currently only object hash is supported)
    object: String,
}

#[derive(Debug)]
pub(crate) struct CatFileOptions {
    flag: DisplayFlag,
    object: String,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub(crate) struct DisplayFlagGroup {
    /// Exit with 0 status code if <OBJECT> exists, and is a valid object
    #[arg(short)]
    exists: bool,

    /// Pretty print the contents of the object
    #[arg(short)]
    pretty: bool,

    /// Show the object size
    #[arg(short)]
    size: bool,

    /// Show the object type
    #[arg(short)]
    type_: bool,
}

#[derive(Debug)]
enum DisplayFlag {
    Exists,
    Pretty,
    Size,
    Type,
}

impl From<DisplayFlagGroup> for DisplayFlag {
    fn from(f: DisplayFlagGroup) -> Self {
        match (f.exists, f.pretty, f.size, f.type_) {
            (true, _, _, _) => DisplayFlag::Exists,
            (_, true, _, _) => DisplayFlag::Pretty,
            (_, _, true, _) => DisplayFlag::Size,
            (_, _, _, _) => DisplayFlag::Type,
        }
    }
}

impl From<CatFileCliOptions> for CatFileOptions {
    fn from(opt: CatFileCliOptions) -> Self {
        Self {
            flag: opt.flag.into(),
            object: opt.object,
        }
    }
}

pub(crate) fn cat_file(git_dir: PathBuf, options: CatFileOptions) -> Result<()> {
    let hash = find_hash(&git_dir, &options.object)?;
    let file = ObjectFile::new(&git_dir, &hash);
    let object = file.parse()?;
    match options.flag {
        DisplayFlag::Exists => {}
        DisplayFlag::Pretty => match object.contents {
            Contents::Blob(blob) => print!("{}", blob.try_string()?),
            Contents::Tree(tree) => println!("{tree}"),
            Contents::Commit(commit) => print!("{commit}"),
        },
        DisplayFlag::Size => println!("{}", object.size),
        DisplayFlag::Type => println!("{}", object.kind()),
    }
    Ok(())
}
