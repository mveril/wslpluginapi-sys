use clap::builder::Str;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Metadata {
    pub id: String,
    pub version: String,
    pub header_file_path: String,
    pub output_file_path: String,
    pub bindgen: BindgenMetadata,
}

impl Metadata {
    pub fn new<Id, Ver, Header, Output, Target>(
        id: Id,
        version: Ver,
        header_file_path: Header,
        output_file_path: Output,
        custom_llvm_target: Option<Target>,
    ) -> Self
    where
        Id: Into<String>,
        Ver: Into<String>,
        Header: Into<String>,
        Output: Into<String>,
        Target: Into<String>,
    {
        let custom_llvm_target = custom_llvm_target.map(|t| t.into());
        Self {
            id: id.into(),
            version: version.into(),
            header_file_path: header_file_path.into(),
            output_file_path: output_file_path.into(),
            bindgen: BindgenMetadata::new(custom_llvm_target),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct BindgenMetadata {
    pub bindgen_version: String,
    pub custom_llvm_target: Option<String>,
    pub build_host: BuildMachine,
}
impl BindgenMetadata {
    fn new(custom_llvm_target: Option<String>) -> Self {
        Self {
            bindgen_version: env!("BINDGEN_VERSION").to_string(),
            custom_llvm_target,
            build_host: BuildMachine::default(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct BuildMachine {
    pub os: String,
    pub arch: String,
}

impl Default for BuildMachine {
    fn default() -> Self {
        BuildMachine {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        }
    }
}
