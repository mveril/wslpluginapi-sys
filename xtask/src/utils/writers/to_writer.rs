use std::io::{self, Write};

use super::{DoubleWriter, RefWriter};
use bindgen::Bindings;
use sha2::{Digest, digest::Output};

pub trait ToWriter {
    type Errror;
    fn to_writer<W: Write>(&self, writer: &mut W) -> Result<(), Self::Errror>;

    fn to_write_and_hash<D: Digest + Write, W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<Output<D>, Self::Errror> {
        let mut d_writer = DoubleWriter::new(writer, D::new());
        self.to_writer(&mut d_writer)?;
        let (_, hasher) = d_writer.into_inner();
        Ok(hasher.finalize())
    }
}

impl ToWriter for Bindings {
    type Errror = io::Error;
    fn to_writer<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.write(Box::new(RefWriter::new(writer)))
    }
}
