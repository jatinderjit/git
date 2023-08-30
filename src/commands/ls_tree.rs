use anyhow::Result;
use clap::Args;

use crate::{
    context::Context,
    objects::{find_hash, object::Contents, ObjectFile},
};

#[derive(Args, Debug)]
pub(crate) struct LsTreeOptions {
    #[arg(long, alias = "")]
    name_only: bool,

    /// The object name (currently only object hash is supported)
    object: String,
}

pub(crate) fn ls_tree(context: Context, options: LsTreeOptions) -> Result<()> {
    let hash = find_hash(&context, &options.object)?;
    let file = ObjectFile::new(&context, &hash);
    let object = file.parse()?;
    match object.contents {
        Contents::Blob(_) => eprintln!("fatal: not a tree object"),
        Contents::Tree(tree) => {
            if options.name_only {
                for line in tree.lines {
                    println!("{}", line.name);
                }
            } else {
                for line in tree.lines {
                    println!("{line}");
                }
            }
        }
        Contents::Commit(commit) => {
            let options = LsTreeOptions {
                name_only: options.name_only,
                object: commit.tree,
            };
            return ls_tree(context, options);
        }
    };
    Ok(())
}
