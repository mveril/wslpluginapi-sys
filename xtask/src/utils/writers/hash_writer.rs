use sha2::digest::Output;
use std::io::Write;

use sha2::Digest;

use super::DoubleWriter;

pub struct HashWriter<W: Write, D: Digest + Write> {
    dw: DoubleWriter<W, D>,
}

impl<W: Write, D: Digest + Write> Write for HashWriter<W, D> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.dw.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.dw.flush()
    }

    fn write_all(&mut self, mut buf: &[u8]) -> std::io::Result<()> {
        self.dw.write_all(buf)
    }
}

impl<W: Write, D: Digest + Write> HashWriter<W, D> {
    pub fn new(writer: W) -> Self {
        Self {
            dw: DoubleWriter::new(writer, D::new()),
        }
    }

    pub fn finalise(self) -> Output<D> {
        let (_, hasher) = self.dw.into_inner();
        hasher.finalize()
    }
}
