use anyhow::Result;
use clap::Args;

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

pub(crate) fn cat_file(options: CatFileOptions) -> Result<()> {
    dbg!(options);
    Ok(())
}
