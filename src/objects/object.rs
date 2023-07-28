use anyhow::{anyhow, bail, Error, Result};
use std::{
    fmt::{self, Display},
    io::Read,
    str::{self, FromStr, Utf8Error},
};
use ObjectKind::*;

use super::hash::hex_digest;

#[derive(Debug)]
pub enum ObjectKind {
    Blob,
    Tree,
    Commit,
}

impl Display for ObjectKind {
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

pub struct Object {
    pub size: usize,
    pub contents: Contents,
}

pub enum Contents {
    Blob(BlobContents),
    Tree(TreeContents),
    Commit(String),
}

impl Contents {
    fn parse(kind: ObjectKind, body: &[u8]) -> Result<Self> {
        use Contents::*;
        Ok(match kind {
            ObjectKind::Blob => Blob(BlobContents(body.to_owned())),
            ObjectKind::Tree => Tree(TreeContents::parse(body)?),
            ObjectKind::Commit => Commit(str::from_utf8(body)?.to_owned()),
        })
    }
}

pub struct BlobContents(Vec<u8>);

impl BlobContents {
    pub fn try_string(&self) -> std::result::Result<&str, Utf8Error> {
        str::from_utf8(&self.0)
    }
}

pub struct TreeContents {
    lines: Vec<TreeRowItem>,
}

impl TreeContents {
    fn parse(body: &[u8]) -> Result<Self> {
        let mut lines = Vec::new();
        let mut i = 0;
        while i < body.len() {
            let space = i + body[i..]
                .iter()
                .position(|c| *c == b' ')
                .ok_or(anyhow!("Corrupt file"))?;
            let perms = str::from_utf8(&body[i..space])?;
            i = space + 1;

            let null = i + body[i..]
                .iter()
                .position(|c| *c == 0)
                .ok_or(anyhow!("Corrupt file"))?;
            let name = str::from_utf8(&body[i..null])?;
            i = null + 1;

            let hash = hex_digest(&body[i..i + 20]);
            i += 20;

            lines.push(TreeRowItem::new(perms, hash, name));
        }
        Ok(Self { lines })
    }
}

impl Display for TreeContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            &self
                .lines
                .iter()
                .map(|row| row.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

pub struct TreeRowItem {
    perms: String,
    kind: ObjectKind,
    hash: String,
    name: String,
}

impl TreeRowItem {
    fn new(perms: &str, hash: String, name: &str) -> Self {
        let perms = format!("{:0>6}", perms);
        Self {
            kind: if perms.starts_with("1") {
                ObjectKind::Blob
            } else {
                ObjectKind::Tree
            },
            perms,
            hash,
            name: name.to_string(),
        }
    }
}

impl Display for TreeRowItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}\t{}",
            self.perms, self.kind, self.hash, self.name
        )
    }
}

impl Object {
    fn decode(bytes: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::ZlibDecoder;
        let mut z = ZlibDecoder::new(bytes);
        let mut buffer = Vec::new();
        z.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    pub fn parse(body: &[u8]) -> Result<Self> {
        let body = Self::decode(body)?;

        let space = body
            .iter()
            .position(|c| *c == b' ')
            .ok_or(anyhow!("Corrupt object"))?;
        let kind = ObjectKind::try_from(&body[..space])?;
        let size = str::from_utf8(
            body[space + 1..]
                .split(|b| *b == 0)
                .next()
                .ok_or(anyhow!("Corrupt object"))?,
        )
        .map_err(|_| anyhow!("Corrupt object"))?;
        let content_start = space + 1 + size.len() + 1;
        let size = size
            .parse::<usize>()
            .map_err(|_| anyhow!("Corrupt object"))?;
        if body.len() - content_start != size {
            bail!("Corrupt hash (Invalid size: {})", size);
        }
        Ok(Object {
            size,
            contents: Contents::parse(kind, &body[content_start..])?,
        })
    }

    pub fn kind(&self) -> ObjectKind {
        use Contents::*;
        match self.contents {
            Blob(_) => ObjectKind::Blob,
            Tree(_) => ObjectKind::Tree,
            Commit(_) => ObjectKind::Commit,
        }
    }
}
