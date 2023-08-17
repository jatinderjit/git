use anyhow::{anyhow, Result};
use std::{
    fmt::{self, Display},
    str,
};

use super::{hash::hex_digest, kind::ObjectKind};

pub(crate) struct TreeContents {
    pub(crate) lines: Vec<TreeRowItem>,
}

impl TreeContents {
    pub(crate) fn parse(body: &[u8]) -> Result<Self> {
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
