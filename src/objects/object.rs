use anyhow::{anyhow, bail, Result};
use sha1::{Digest, Sha1};
use std::str;

use crate::objects::commit::CommitContents;

use super::{blob::BlobContents, kind::ObjectKind, tree::TreeContents};

pub(crate) struct Object {
    pub(crate) size: usize,
    pub(crate) contents: Contents,
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
            ObjectKind::Blob => Blob(BlobContents::new(body)),
            ObjectKind::Tree => Tree(TreeContents::parse(body)?),
            ObjectKind::Commit => Commit(CommitContents::parse(body)?),
        })
    }
}

impl Object {
    pub fn new_blob(contents: &[u8]) -> Self {
        Self {
            size: contents.len(),
            contents: Contents::Blob(BlobContents::new(contents)),
        }
    }

    pub(crate) fn parse(body: &[u8]) -> Result<Self> {
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
        hasher.update(match &self.contents {
            Contents::Blob(BlobContents(blob)) => blob,
            _ => todo!(),
        });
        let hash = hasher.finalize().to_vec();
        super::hash::hex_digest(&hash)
    }
}
