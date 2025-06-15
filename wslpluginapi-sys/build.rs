use std::{env, fs, path::PathBuf};

use semver::Version;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub version: String,
}

fn main() {
    let current_version: Version = env!("CARGO_PKG_VERSION")
        .parse()
        .expect("Failed to parse CARGO_PKG_VERSION");
    let expected_nuget_version = current_version.build.as_str();
    let root: PathBuf = env::var_os("CARGO_MANIFEST_DIR").unwrap().into();
    let version = {
        let metadata_path = root.join("build/metadata.json");
        let file = fs::File::open(metadata_path).expect("Failed to open metadata.json");
        let json: Metadata = serde_json::from_reader(file).unwrap();
        json.version
    };
    if version != expected_nuget_version {
        panic!(
            "Version mismatch: metadata.json version '{}' does not match package metadata '{}'",
            version, expected_nuget_version
        );
    }
}
