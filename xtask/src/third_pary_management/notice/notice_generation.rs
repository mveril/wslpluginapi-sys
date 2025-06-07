use markdown_builder::Markdown;
use std::{borrow::Cow, fs, path::Path};
pub trait NoticeGeneration {
    fn generate_notice<P: AsRef<Path>>(&self, output_path: P) -> std::io::Result<()> {
        let output_path = if output_path.as_ref().is_absolute() {
            Cow::Borrowed(output_path.as_ref())
        } else {
            Cow::Owned(output_path.as_ref().canonicalize()?)
        };
        let mut md = Markdown::new();
        Self::header_generation(&mut md);
        self.generate_content_in_place(&mut md, &output_path, 1)?;
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(output_path, md.render())?;
        Ok(())
    }

    fn generate_content_in_place<P: AsRef<Path>>(
        &self,
        md: &mut Markdown,
        output_path: P,
        header_level: usize,
    ) -> std::io::Result<()>;

    fn header_generation(md: &mut Markdown) {
        md.header1("Third Party Notices");
    }
}
