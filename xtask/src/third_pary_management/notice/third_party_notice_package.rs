use std::path::Path;

use super::{ThirdPartyNoticeItem, notice_generation::NoticeGeneration};

#[derive(Debug, Clone)]
pub struct ThirdPartyNoticePackage {
    name: String,
    items: Vec<ThirdPartyNoticeItem>,
}

impl ThirdPartyNoticePackage {
    pub fn new(name: String) -> Self {
        Self {
            name,
            items: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl NoticeGeneration for ThirdPartyNoticePackage {
    fn generate_content_in_place<P: AsRef<Path>>(
        &self,
        md: &mut markdown_builder::Markdown,
        output_path: P,
        header_level: usize,
    ) -> std::io::Result<()> {
        for item in self {
            item.generate_content_in_place(md, &output_path, header_level + 1)?;
        }
        Ok(())
    }
}

impl std::ops::Deref for ThirdPartyNoticePackage {
    type Target = Vec<ThirdPartyNoticeItem>;
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl std::ops::DerefMut for ThirdPartyNoticePackage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl IntoIterator for ThirdPartyNoticePackage {
    type Item = ThirdPartyNoticeItem;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a ThirdPartyNoticePackage {
    type Item = &'a ThirdPartyNoticeItem;
    type IntoIter = std::slice::Iter<'a, ThirdPartyNoticeItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<'a> IntoIterator for &'a mut ThirdPartyNoticePackage {
    type Item = &'a mut ThirdPartyNoticeItem;
    type IntoIter = std::slice::IterMut<'a, ThirdPartyNoticeItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter_mut()
    }
}

impl Extend<ThirdPartyNoticeItem> for ThirdPartyNoticePackage {
    fn extend<T: IntoIterator<Item = ThirdPartyNoticeItem>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}
