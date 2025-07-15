pub mod writers;

use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use bindgen::Bindings;

pub fn format_with_rustfmt<W: Write>(
    binding: Bindings,
    mut output: W,
    workdir: Option<&Path>,
) -> io::Result<()> {
    let mut cmd = Command::new("rustfmt");
    cmd.arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    if let Some(dir) = workdir {
        cmd.current_dir(dir);
    }
    let mut child = cmd.spawn()?;

    {
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("Failed to open rustfmt stdin"))?;
        binding.write(Box::new(stdin))?;
    }

    if let Some(mut stdout) = child.stdout.take() {
        io::copy(&mut stdout, &mut output)?;
    }

    let status = child.wait()?;
    if !status.success() {
        Err(io::Error::other(format!(
            "rustfmt exited with status {status}"
        )))
    } else {
        Ok(())
    }
}
