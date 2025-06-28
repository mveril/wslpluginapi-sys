use std::io::{self, Write};

use super::{DoubleWriter, RefWriter};
use bindgen::Bindings;
use sha2::{Digest, digest::Output};

pub trait ToWriter {
    type Error;
    fn to_writer<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error>;

    fn to_write_and_hash<D: Digest + Write, W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<Output<D>, Self::Error> {
        let mut d_writer = DoubleWriter::new(writer, D::new());
        self.to_writer(&mut d_writer)?;
        let (_, hasher) = d_writer.into_inner();
        Ok(hasher.finalize())
    }
}

impl ToWriter for Bindings {
    type Error = io::Error;
    fn to_writer<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write(Box::new(RefWriter::new(writer)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    pub struct Dummy;
    impl ToWriter for Dummy {
        type Error = io::Error;
        fn to_writer<W: Write>(&self, writer: &mut W) -> io::Result<()> {
            writer.write_all(b"hello world")
        }
    }
    #[test]
    fn test_to_writer_writes_correct_bytes() {
        let dummy = Dummy;
        let mut buf = Vec::new();
        dummy.to_writer(&mut buf).unwrap();
        assert_eq!(&buf, b"hello world");
    }

    #[test]
    fn test_to_write_and_hash_returns_correct_hash() {
        let dummy = Dummy;
        let mut buf = Vec::new();
        let hash = dummy.to_write_and_hash::<Sha256, _>(&mut buf).unwrap();
        let expected = Sha256::digest(b"hello world");
        assert_eq!(&buf, b"hello world");
        assert_eq!(&hash[..], &expected[..]);
    }

    #[test]
    fn test_to_writer_with_refwriter() {
        let dummy = Dummy;
        let mut buf = Vec::new();
        let mut ref_writer = RefWriter::new(&mut buf);
        dummy.to_writer(&mut ref_writer).unwrap();
        assert_eq!(&buf, b"hello world");
    }
}
