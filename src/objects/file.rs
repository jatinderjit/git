use std::{
    fs,
    path::{Path, PathBuf},
    str,
};

use super::object::Object;
use anyhow::{anyhow, bail, Result};

pub(crate) struct ObjectFile {
    fp: PathBuf,
}

impl ObjectFile {
    pub(crate) fn from_hash(git_dir: &Path, hash: &str) -> Result<Self> {
        let hash = super::find_hash(git_dir, hash)?;
        let dir = git_dir.join("objects").join(&hash[..2]);
        if !dir.exists() || !dir.is_dir() {
            bail!(
                "No object found for hash {hash} at {}",
                dir.to_string_lossy()
            );
        }
        let prefix = &hash[2..];
        let files =
            fs::read_dir(&dir).map_err(|_| anyhow!("Error reading {}", dir.to_string_lossy()))?;

        let mut fp = None;
        for file in files {
            if let Ok(file) = file {
                let path = file.path();
                let file_name = path.file_name().map(|f| f.to_str());
                if let Some(Some(file_name)) = file_name {
                    if !file_name.starts_with(prefix) {
                        continue;
                    }
                    if fp.is_some() {
                        bail!("Ambiguous hash: {hash}");
                    }
                    fp = Some(path);
                }
            }
        }
        match fp {
            Some(fp) => Ok(ObjectFile { fp }),
            None => bail!("No object found for hash {hash}"),
        }
    }

    pub(crate) fn parse(&self) -> Result<Object> {
        let body = fs::read(&self.fp)?;
        Object::parse(&body)
    }
}
