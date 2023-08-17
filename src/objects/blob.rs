use std::str::{self, Utf8Error};

pub struct BlobContents(pub(crate) Vec<u8>);

impl BlobContents {
    pub fn new(contents: &[u8]) -> Self {
        let mut body = Vec::new();
        body.extend(b"blob ");
        body.extend(contents.len().to_string().as_bytes());
        body.push(0);
        body.extend(contents);
        Self(body)
    }

    pub fn try_string(&self) -> std::result::Result<&str, Utf8Error> {
        str::from_utf8(&self.0)
    }
}
