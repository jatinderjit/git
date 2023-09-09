use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{bail, Result};

use super::object::Object;
use crate::{
    context::Context,
    objects::{blob::BlobContents, object::Contents},
    utils,
};

pub(crate) struct ObjectFile<'a> {
    context: &'a Context,
    hash: &'a str,
}

impl<'a> ObjectFile<'a> {
    pub(crate) fn new(context: &'a Context, hash: &'a str) -> Self {
        // TODO: better handling of hash. Currently it's assumed to be validated
        // before this function is called.
        Self { context, hash }
    }

    pub fn save(&self, object: &Object) -> Result<()> {
        assert_eq!(self.hash, object.compute_hash());

        let dir_path = self.context.object_dir(self.hash);
        let fp = self.context.object_path(self.hash);

        if !dir_path.exists() {
            fs::create_dir(dir_path)?;
        } else if !dir_path.is_dir() {
            bail!("File already exists instead of directory: {:?}", dir_path);
        }

        if Path::new(&fp).exists() {
            // Remove it since the read only permissions on the object files
            // won't allow us to overwrite it.
            fs::remove_file(&fp)?;
        }

        let body = match &object.contents {
            Contents::Blob(BlobContents(blob)) => blob,
            _ => todo!(),
        };
        let body = utils::zlib_encode(body)?;
        {
            let mut f = File::create(&fp)?;
            f.write_all(&body)?;
        }

        let mut perms = fs::metadata(&fp)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(&fp, perms)?;

        Ok(())
    }

    pub(crate) fn parse(&self) -> Result<Object> {
        let body = fs::read(self.context.object_path(self.hash))?;
        let body = utils::zlib_decode(&body)?;
        Object::parse(&body)
    }
}
