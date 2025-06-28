use sha2::{digest::Output, Digest, Sha256};
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::Write,
    io::{self, BufRead, BufReader},
};

use std::path::{Path, PathBuf};

use semver::Version;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub version: String,
    pub output_file_path: String,
}

fn main() -> anyhow::Result<()> {
    let root: PathBuf = env::var_os("CARGO_MANIFEST_DIR").unwrap().into();
    let build = root.join("build");
    let hashes: HashMap<PathBuf, Output<Sha256>> = fs::read_dir(&build)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().is_none_or(|ext| ext != "sha256"))
        .filter(|path| !path.starts_with(".git"))
        .filter_map(|path| {
            compute_hash::<Sha256, _>(&path)
                .ok()
                .map(|hash| (path.strip_prefix(&build).unwrap().to_owned(), hash))
        })
        .collect();
    let reader = BufReader::new(File::open(root.join(build.join("checksum.sha256")))?);
    let mut hash_file = HashMap::<Box<str>, Box<str>>::new();
    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let mut splits = line.split_whitespace().take(2);
        if let Some(val) = splits.next() {
            if let Some(key) = splits.next() {
                hash_file.insert(
                    key.to_owned().into_boxed_str(),
                    val.to_owned().into_boxed_str(),
                );
            }
        }
    }
    for (file, hash) in hashes {
        let path_str = file.as_os_str().to_str().expect("Failed to convert path");

        let mem_hash = hash_file
            .get(path_str)
            .unwrap_or_else(|| panic!("No hash in the checksum file for {}", &path_str));

        if hex::encode(hash) != mem_hash.as_ref() {
            panic!("Hash not ok for file {}", &path_str);
        }
    }

    let current_version: Version = env!("CARGO_PKG_VERSION")
        .parse()
        .expect("Failed to parse CARGO_PKG_VERSION");
    let expected_nuget_version = current_version.build.as_str();
    let json_metadata = {
        let metadata_path = root.join("build/metadata.json");
        let file = fs::File::open(metadata_path).expect("Failed to open metadata.json");
        let json: Metadata = serde_json::from_reader(file).unwrap();
        json
    };
    if json_metadata.version != expected_nuget_version {
        panic!(
            "Version mismatch: metadata.json version '{}' does not match package metadata '{}'",
            &json_metadata.version, expected_nuget_version
        );
    }
    let output_full_path = build.join(json_metadata.output_file_path);
    println!(
        "cargo:rustc-env=RUST_BINDING={}",
        output_full_path.to_str().unwrap()
    );
    Ok(())
}

fn compute_hash<D: Digest + Write, P: AsRef<Path>>(path: P) -> io::Result<Output<D>> {
    let mut file = File::open(path.as_ref())?;
    let mut hasher = D::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize())
}
