use anyhow::{anyhow, bail, Result};
use std::fs;

use crate::context::Context;

/// Finds the full object hash for the hash prefix.
pub fn find_hash(context: &Context, hash: &str) -> Result<String> {
    if hash.len() < 4 || hash.len() > 40 {
        bail!("Invalid hash length");
    }

    let dir = context.git_dir.join("objects").join(&hash[..2]);
    if !dir.exists() || !dir.is_dir() {
        bail!("No object found for hash: {hash}");
    }
    if dir.join(&hash[2..]).exists() {
        return Ok(hash.to_owned());
    }

    let mut full_hash = None;
    let prefix = &hash[2..];
    let files = fs::read_dir(&dir).map_err(|_| anyhow!("Error reading {}", dir.display()))?;
    for file in files.flatten() {
        let path = file.path();
        let file_name = path.file_name().map(|f| f.to_str());
        if let Some(Some(file_name)) = file_name {
            if !file_name.starts_with(prefix) {
                continue;
            }
            if full_hash.is_some() {
                bail!("Ambiguous hash: {hash}");
            }
            full_hash = Some(format!("{}{}", &hash[..2], file_name));
        }
    }
    full_hash.ok_or(anyhow!("No object found for hash: {hash}"))
}

/// Return the digest value as a string of hexadecimal digits
pub(crate) fn hex_digest(bytes: &[u8]) -> String {
    let chars = b"0123456789abcdef";
    bytes
        .iter()
        .flat_map(|c| {
            vec![
                chars[(c >> 4) as usize] as char,
                chars[(c & 0x0F) as usize] as char,
            ]
        })
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("")
}
