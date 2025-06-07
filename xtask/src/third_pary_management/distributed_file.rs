use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Clone)]
pub struct DistributedFile {
    pub path: PathBuf,
    pub status: Status,
}

impl DistributedFile {
    pub fn new(path: PathBuf, status: Status) -> Self {
        Self { path, status }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Modified,
    Unmodified,
    PackageMetadataGenerated,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Status::Modified => "modified",
            Status::Unmodified => "unmodified",
            Status::PackageMetadataGenerated => "generated from package metadata",
        };
        f.write_str(output)
    }
}
