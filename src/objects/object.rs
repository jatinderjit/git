use anyhow::{anyhow, bail, Error, Result};
use sha1::{Digest, Sha1};
use std::{
    fmt::{self, Display},
    str::{self, FromStr, Utf8Error},
};
use ObjectKind::*;

use super::hash::hex_digest;

#[derive(Debug)]
pub(crate) enum ObjectKind {
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

pub(crate) struct Object {
    pub raw: Vec<u8>,
    pub size: usize,
    pub contents: Contents,
}

pub(crate) enum Contents {
    Blob(BlobContents),
    Tree(TreeContents),
    Commit(CommitContents),
}

impl Contents {
    fn parse(kind: ObjectKind, body: &[u8]) -> Result<Self> {
        use Contents::*;
        Ok(match kind {
            ObjectKind::Blob => Blob(BlobContents(body.to_owned())),
            ObjectKind::Tree => Tree(TreeContents::parse(body)?),
            ObjectKind::Commit => Commit(CommitContents::parse(body)?),
        })
    }
}

pub struct BlobContents(Vec<u8>);

impl BlobContents {
    pub fn try_string(&self) -> std::result::Result<&str, Utf8Error> {
        str::from_utf8(&self.0)
    }
}

pub(crate) struct TreeContents {
    pub(crate) lines: Vec<TreeRowItem>,
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

pub(crate) struct TreeRowItem {
    pub(crate) perms: String,
    pub(crate) kind: ObjectKind,
    pub(crate) hash: String,
    pub(crate) name: String,
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

// TODO: parse into name, email, timestamp, timezone
pub struct Author {
    pub name: String,
    pub email: String,
    pub timestamp: u32,
    pub timezone: String,
}

impl Author {
    fn parse(line: &str) -> Result<Self> {
        let (name, remaining) = line
            .split_once(" <")
            .ok_or(anyhow!("Invalid author format"))?;
        let (email, remaining) = remaining
            .split_once("> ")
            .ok_or(anyhow!("Invalid author format"))?;
        let (timestamp, timezone) = remaining
            .split_once(" ")
            .ok_or(anyhow!("Invalid author format"))?;
        let timestamp = timestamp
            .parse::<u32>()
            .map_err(|_| anyhow!("Invalid author format (timestamp)"))?;
        let parsed_timezone: i32 = timezone
            .parse()
            .map_err(|_| anyhow!("Invalid author format (timezone)"))?;
        if parsed_timezone >= 2400 || parsed_timezone < -2400 {
            bail!("Invalid author format (timezone)");
        }
        Ok(Self {
            name: name.to_string(),
            email: email.to_string(),
            timestamp,
            timezone: timezone.to_string(),
        })
    }
}
impl Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{} <{}> {} {}",
            self.name, self.email, self.timestamp, self.timezone
        ))
    }
}

pub struct CommitContents {
    pub raw: String,
    pub tree: String,
    pub parents: Vec<String>,
    pub author: Author,
    pub committer: Option<Author>,
    pub gpgsig: Option<String>,
    pub message: String,
}

impl CommitContents {
    fn parse(body: &[u8]) -> Result<Self> {
        let raw = str::from_utf8(body)?;
        let (metadata, message) = raw
            .split_once("\n\n")
            .ok_or(anyhow!("Missing commit message"))?;
        let mut lines = metadata.lines();

        let tree = lines.next().ok_or(anyhow!("Expected tree hash"))?;
        if !tree.starts_with("tree ") {
            bail!("Expected tree hash");
        }
        let tree = tree[5..].to_owned();

        // This will be blank for the initial commits.
        let mut parents = Vec::new();
        let mut line = lines.next().ok_or(anyhow!("Expected parent or author"))?;
        while line.starts_with("parent ") {
            parents.push(line[7..].to_owned());
            line = lines.next().expect("Expected parent or author");
        }

        if !line.starts_with("author ") {
            bail!("Expected author");
        }
        let author = Author::parse(&line[7..])?;

        let committer = match lines
            .next()
            .filter(|line| line.starts_with("committer "))
            .map(|line| Author::parse(&line[10..]))
        {
            Some(Ok(committer)) => Some(committer),
            Some(Err(_)) => bail!("Invalid commit (committer)"),
            None => None,
        };

        let gpgsig = match lines.next() {
            Some(line) => {
                if !line.starts_with("gpgsig") || !line.contains("--BEGIN PGP SIGNATURE--") {
                    bail!("Invalid commit (gpgsig)");
                }
                let mut gpgsig = line[7..].to_string();
                let mut ends = false;
                let err = lines
                    .map(|line| {
                        if !line.starts_with(" ") {
                            return Some("Unexpected lines");
                        }
                        gpgsig.push('\n');
                        gpgsig.push_str(line);
                        let last_line = line.contains("--END PGP SIGNATURE--");
                        ends = ends || last_line;
                        None
                    })
                    .find_map(|e| e);
                if !ends || err.is_some() {
                    bail!("Invalid commit (gpgsig)");
                }
                Some(gpgsig)
            }
            None => None,
        };

        Ok(Self {
            raw: raw.to_owned(),
            tree,
            parents,
            author,
            committer,
            gpgsig,
            message: message.to_owned(),
        })
    }
}

impl Display for CommitContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("tree {}\n", self.tree))?;
        for parent in self.parents.iter() {
            f.write_fmt(format_args!("parent {}\n", parent))?;
        }
        f.write_fmt(format_args!("author {}\n", self.author))?;
        if let Some(committer) = &self.committer {
            f.write_fmt(format_args!("committer {}\n", committer))?;
        }
        if let Some(gpgsig) = &self.gpgsig {
            f.write_fmt(format_args!("gpgsig {}\n", gpgsig))?;
        }
        f.write_fmt(format_args!("\n{}", self.message))
    }
}

impl Object {
    pub(crate) fn new_blob(contents: &[u8]) -> Result<Self> {
        let mut body = Vec::new();
        body.extend(b"blob ");
        body.extend(contents.len().to_string().as_bytes());
        body.push(0);
        body.extend(contents);
        Ok(Self {
            raw: body,
            size: contents.len(),
            contents: Contents::Blob(BlobContents(contents.to_vec())),
        })
    }

    pub(crate) fn parse(body: &[u8]) -> Result<Self> {
        let raw = body.to_vec();

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
            raw,
            size,
            contents: Contents::parse(kind, &body[content_start..])?,
        })
    }

    pub(crate) fn kind(&self) -> ObjectKind {
        use Contents::*;
        match self.contents {
            Blob(_) => ObjectKind::Blob,
            Tree(_) => ObjectKind::Tree,
            Commit(_) => ObjectKind::Commit,
        }
    }

    pub(crate) fn compute_hash(&self) -> String {
        let mut hasher = Sha1::new();
        hasher.update(&self.raw);
        let hash = hasher.finalize().to_vec();
        super::hash::hex_digest(&hash)
    }
}
