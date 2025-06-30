use sha2::{Sha256, digest::Output};
use std::io::{Result, Write};

use sha2::Digest;

pub type Sha256HashWriter<W> = HashWriter<W, Sha256>;

pub struct HashWriter<W: Write, D: Digest> {
    writer: W,
    hash: D,
}

impl<W: Write, D: Digest> Write for HashWriter<W, D> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let value = self.writer.write(buf)?;
        self.hash.update(&buf[..value]);
        return Ok(value);
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write, D: Digest> HashWriter<W, D> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            hash: D::new(),
        }
    }

    pub fn finalise(mut self) -> Result<Output<D>> {
        self.writer.flush()?;
        Ok(self.hash.finalize())
    }
}
