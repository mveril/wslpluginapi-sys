extern crate bindgen;
mod header_processing;
use constcat::concat;
use std::env;
use std::path::PathBuf;
const WSL_PLUGIN_API_FILE_NAME: &str = "WslPluginApi";
const WSL_PLUGIN_API_HEADER_FILE: &str = concat!(WSL_PLUGIN_API_FILE_NAME, ".h");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    let host = env::var("HOST")?;
    let target = env::var("TARGET")?;
    let out_path: PathBuf = env::var("OUT_DIR")?.into();

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"));
    let header_file_path = manifest_dir.join(format!(
        "third_party/Microsoft.WSL.PluginApi/include/{}",
        WSL_PLUGIN_API_HEADER_FILE
    ));

    if !header_file_path.exists() {
        return Err(format!("Header file does not exist: {:?}", header_file_path).into());
    }
    print!("cargo:rerun-if-changed={}", header_file_path.display());
    let out_file = out_path.join("bindings.rs");
    let api_header = header_processing::process(header_file_path, host, target)?;
    api_header.write_to_file(&out_file)?;
    Ok(())
}
