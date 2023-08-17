use anyhow::{bail, Error};
use std::{
    fmt,
    str::{self, FromStr},
};

use ObjectKind::*;

#[derive(Debug)]
pub(crate) enum ObjectKind {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Blob => write!(f, "blob"),
            Tree => write!(f, "tree"),
            Commit => write!(f, "commit"),
        }
    }
}

impl FromStr for ObjectKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ObjectKind::try_from(s.as_bytes())
    }
}

impl<'a> TryFrom<&'a [u8]> for ObjectKind {
    type Error = Error;
    fn try_from(kind: &'a [u8]) -> std::result::Result<Self, Self::Error> {
        match kind {
            b"blob" => Ok(Blob),
            b"tree" => Ok(Tree),
            b"commit" => Ok(Commit),
            _ => match str::from_utf8(kind) {
                Ok(kind) => bail!("Invalid object type: {kind}"),
                Err(_) => bail!("Invalid object type"),
            },
        }
    }
}
