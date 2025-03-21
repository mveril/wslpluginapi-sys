extern crate bindgen;
extern crate semver;
mod header_processing;
use constcat::concat;
use semver::Version;
use std::env;
mod nuget;
use nuget::ensure_package_installed;
use std::path::PathBuf;

const WSL_PACKAGE_NAME: &str = "Microsoft.WSL.PluginApi";
const WSL_PLUGIN_API_FILE_NAME: &str = "WslPluginApi";
const WSL_PLUGIN_API_BINDGEN_OUTPUT_FILE_NAME: &str = concat!(WSL_PLUGIN_API_FILE_NAME, ".rs");
const WSL_PLUGIN_API_HEADER_FILE: &str = concat!(WSL_PLUGIN_API_FILE_NAME, ".h");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    let host = env::var("HOST")?;
    let target = env::var("TARGET")?;

    let version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    println!("cargo:version={}", version);

    let package_version = version.build.to_string();
    let out_path: PathBuf = env::var("OUT_DIR")?.into();

    let package_path = ensure_package_installed(WSL_PACKAGE_NAME, &package_version)?;

    let header_file_path = package_path.join(format!(
        "build/native/include/{}",
        WSL_PLUGIN_API_HEADER_FILE
    ));

    if !header_file_path.exists() {
        return Err(format!("Header file does not exist: {:?}", header_file_path).into());
    }
    let out_file = out_path.join(WSL_PLUGIN_API_BINDGEN_OUTPUT_FILE_NAME);
    let api_header = header_processing::process(header_file_path, host, target)?;
    api_header.write_to_file(&out_file)?;
    println!(
        "cargo:rustc-env=WSL_PLUGIN_API_BINDGEN_OUTPUT_FILE_NAME={}",
        out_file.display()
    );
    Ok(())
}
