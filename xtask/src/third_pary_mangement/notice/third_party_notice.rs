use std::{borrow::Cow, fs, path::Path};

use super::{ThirdPartyNoticeItem, ThirdPartyNoticePackage, notice_generation::NoticeGeneration};

#[derive(Debug, Clone, Default)]
pub struct ThirdPartyNotice {
    packages: Vec<ThirdPartyNoticePackage>,
}

impl ThirdPartyNotice {
    pub fn add_item(&mut self, item: ThirdPartyNoticePackage) {
        self.packages.push(item);
    }
}

impl NoticeGeneration for ThirdPartyNotice {
    fn generate_content_in_place<P: AsRef<Path>>(
        &self,
        md: &mut markdown_builder::Markdown,
        output_path: P,
        header_level: usize,
    ) -> std::io::Result<()> {
        let add_header = self.len() > 0;
        let package_level = if add_header {
            header_level + 1
        } else {
            header_level
        };
        for package in self {
            if add_header {
                md.header(package.name(), header_level + 1);
            }
            package.generate_content_in_place(md, &output_path, package_level)?;
        }
        Ok(())
    }
}

impl std::ops::Deref for ThirdPartyNotice {
    type Target = Vec<ThirdPartyNoticePackage>;
    fn deref(&self) -> &Self::Target {
        &self.packages
    }
}

impl std::ops::DerefMut for ThirdPartyNotice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.packages
    }
}

impl std::iter::FromIterator<ThirdPartyNoticePackage> for ThirdPartyNotice {
    fn from_iter<I: IntoIterator<Item = ThirdPartyNoticePackage>>(iter: I) -> Self {
        Self {
            packages: Vec::from_iter(iter),
        }
    }
}

impl IntoIterator for ThirdPartyNotice {
    type Item = ThirdPartyNoticePackage;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.packages.into_iter()
    }
}

impl<'a> IntoIterator for &'a ThirdPartyNotice {
    type Item = &'a ThirdPartyNoticePackage;
    type IntoIter = std::slice::Iter<'a, ThirdPartyNoticePackage>;

    fn into_iter(self) -> Self::IntoIter {
        self.packages.iter()
    }
}

impl<'a> IntoIterator for &'a mut ThirdPartyNotice {
    type Item = &'a mut ThirdPartyNoticePackage;
    type IntoIter = std::slice::IterMut<'a, ThirdPartyNoticePackage>;

    fn into_iter(self) -> Self::IntoIter {
        self.packages.iter_mut()
    }
}

impl Extend<ThirdPartyNoticePackage> for ThirdPartyNotice {
    fn extend<T: IntoIterator<Item = ThirdPartyNoticePackage>>(&mut self, iter: T) {
        self.packages.extend(iter);
    }
}
