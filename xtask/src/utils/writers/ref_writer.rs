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
