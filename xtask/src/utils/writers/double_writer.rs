use std::io::{Result, Write};

pub struct DoubleWriter<W1: Write, W2: Write> {
    w1: W1,
    w2: W2,
}

impl<W1: Write, W2: Write> DoubleWriter<W1, W2> {
    pub fn new(w1: W1, w2: W2) -> Self {
        Self { w1, w2 }
    }

    pub fn into_inner(self) -> (W1, W2) {
        (self.w1, self.w2)
    }
}

impl<W1: Write, W2: Write> Write for DoubleWriter<W1, W2> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let n = self.w1.write(buf)?;
        self.w2.write_all(&buf[..n])?;
        Ok(n)
    }
    fn flush(&mut self) -> Result<()> {
        self.w1.flush()?;
        self.w2.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::io::{Cursor, Result};

    #[test]
    fn write_possessed_cursor() -> Result<()> {
        let buf1 = Cursor::<Vec<u8>>::default();
        let buf2 = Cursor::<Vec<u8>>::default();
        let mut writer = DoubleWriter::new(buf1, buf2);

        writer.write_all(b"abc")?;
        writer.flush()?;

        let (buf1, buf2) = writer.into_inner();
        assert_eq!(buf1.into_inner(), b"abc");
        assert_eq!(buf2.into_inner(), b"abc");
        Ok(())
    }

    #[test]
    fn write_with_sha256() -> Result<()> {
        let buf = Cursor::<Vec<u8>>::default();
        let hasher = Sha256::new();

        let mut writer = DoubleWriter::new(buf, hasher);
        writer.write_all(b"hashme")?;
        writer.flush()?;

        let (buf, hasher) = writer.into_inner();
        assert_eq!(buf.into_inner(), b"hashme");
        let result = hasher.finalize();
        assert_eq!(result, Sha256::digest(b"hashme"));
        Ok(())
    }
}
