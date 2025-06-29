pub mod writers;

use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

use bindgen::Bindings;

pub fn format_with_rustfmt<W: Write>(binding: Bindings, mut output: W) -> io::Result<()> {
    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    {
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to open rustfmt stdin"))?;
        binding.write(Box::new(stdin))?;
    }

    if let Some(mut stdout) = child.stdout.take() {
        io::copy(&mut stdout, &mut output)?;
    }

    let status = child.wait()?;
    if !status.success() {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("rustfmt exited with status {}", status),
        ))
    } else {
        Ok(())
    }
}
