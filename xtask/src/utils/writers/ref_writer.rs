use std::io::Write;
pub struct RefWriter<'a, W: Write> {
    innner: &'a mut W,
}

impl<'a, W: Write + 'a> RefWriter<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        Self { innner: writer }
    }
}

impl<'a, T: Write> Write for RefWriter<'a, T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.innner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.innner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_refwriter_with_boxed_writer() {
        // We use a Box<dyn Write> to enable boxing
        let mut buf: Vec<u8> = Vec::new();

        // Wrap the boxed writer in RefWriter
        {
            let writer = RefWriter::new(&mut buf);
            let mut box_writer: Box<dyn Write> = Box::new(writer);
            box_writer.write_all(b"hello").unwrap();
            box_writer.flush().unwrap();
        }
        assert_eq!(buf, b"hello");
    }
}
