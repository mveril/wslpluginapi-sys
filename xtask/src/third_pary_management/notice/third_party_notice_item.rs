use log::debug;
use markdown_builder::{BlockQuote, Bold, Inline, Link, List};

use crate::{nuspec::LicenceContent, third_pary_management::distributed_file::DistributedFile};
use std::{borrow::Cow, path::Path};

use super::notice_generation::NoticeGeneration;

#[derive(Debug, Clone)]
pub struct ThirdPartyNoticeItem {
    name: String,
    version: String,
    link: String,
    copyright: Option<String>,
    license: Option<LicenceContent>,
    files: Vec<DistributedFile>,
}

impl NoticeGeneration for ThirdPartyNoticeItem {
    fn generate_content_in_place<P: AsRef<Path>>(
        &self,
        md: &mut markdown_builder::Markdown,
        output_path: P,
        header_level: usize,
    ) -> std::io::Result<()> {
        md.header(self.name(), header_level);
        md.paragraph(&format!(
            "{} {}",
            "Version :".to_bold(),
            self.version().to_bold()
        ));
        let licence_txt = if let Some(license) = self.license() {
            match license {
                crate::nuspec::LicenceContent::Body(licence_body) => match licence_body {
                    crate::nuspec::LicenceBody::Generator(license_definition) => {
                        Cow::Borrowed(license_definition.license.as_ref())
                    }
                    crate::nuspec::LicenceBody::File(_) => {
                        Cow::Borrowed("See the LICENSE file for details.")
                    }
                },
                crate::nuspec::LicenceContent::URL(url) => Cow::Owned(format!(
                    "See the {} for details.",
                    Link::builder().text("license").url(url).inlined().build()
                )),
            }
        } else {
            Cow::Borrowed("Not specified")
        };
        md.paragraph(format!("{} {}", "License :".to_bold(), licence_txt));
        md.paragraph(format!(
            "{} {}",
            "Source:".to_bold(),
            Link::builder()
                .text(self.link())
                .url(self.name())
                .footer(false)
                .inlined()
                .build()
        ));

        if !self.files().is_empty() {
            md.header("Included files from the package.", header_level + 1);
        }
        let md_files_list = self
            .files()
            .iter()
            .fold(List::builder(), |builder, file| {
                debug!("Adding file to list: {:?}", &file.path);
                builder.append(format!(
                    "{} {}",
                    file.path
                        .strip_prefix(output_path.as_ref().parent().unwrap())
                        .unwrap()
                        .to_string_lossy()
                        .replace("\\", "/")
                        .to_inline(),
                    file.status
                ))
            })
            .unordered();
        md.list(md_files_list);
        if let Some(copyright) = self.copyright() {
            md.paragraph("copyright:".to_bold())
                .paragraph(copyright.to_block_quote());
        }
        Ok(())
    }
}

impl ThirdPartyNoticeItem {
    pub fn new(
        name: String,
        version: String,
        link: String,
        copyright: Option<String>,
        license: Option<LicenceContent>,
    ) -> Self {
        Self {
            name,
            version,
            link,
            copyright,
            license,
            files: Vec::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn link(&self) -> &str {
        &self.link
    }

    pub fn copyright(&self) -> Option<&str> {
        self.copyright.as_deref()
    }

    pub fn license(&self) -> Option<&LicenceContent> {
        self.license.as_ref()
    }

    pub fn files(&self) -> &Vec<DistributedFile> {
        &self.files
    }

    pub fn files_mut(&mut self) -> &mut Vec<DistributedFile> {
        &mut self.files
    }

    pub fn add_file(&mut self, file: DistributedFile) {
        self.files.push(file);
    }

    // Méthodes utiles de collection
    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, DistributedFile> {
        self.files.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, DistributedFile> {
        self.files.iter_mut()
    }

    pub fn clear(&mut self) {
        self.files.clear();
    }

    pub fn remove(&mut self, index: usize) -> DistributedFile {
        self.files.remove(index)
    }

    pub fn get(&self, index: usize) -> Option<&DistributedFile> {
        self.files.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut DistributedFile> {
        self.files.get_mut(index)
    }
}
