extern crate bindgen;
mod header_processing;
use std::env;
use std::path::PathBuf;
const WSL_PLUGIN_API_FILE_BASE_NAME: &str = "WslPluginApi";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    let [wsl_plugin_api_header_file_name, wsl_plugin_api_output_file_name] =
        ["h", "rs"].map(|ext| String::from(WSL_PLUGIN_API_FILE_BASE_NAME) + "." + ext);
    let host = env::var("HOST")?;
    let target = env::var("TARGET")?;
    let out_path: PathBuf = env::var("OUT_DIR")?.into();

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"));
    let header_file_path = manifest_dir
        .join("third_party/Microsoft.WSL.PluginApi/include")
        .join(&wsl_plugin_api_header_file_name);
    println!("cargo:rerun-if-changed={}", header_file_path.display());

    if !header_file_path.exists() {
        return Err(format!("Header file does not exist: {:?}", header_file_path).into());
    }
    let out_file = out_path.join(&wsl_plugin_api_output_file_name);
    let api_header = header_processing::process(header_file_path, host, target)?;
    api_header.write_to_file(&out_file)?;
    println!(
        "cargo:rustc-env=WSL_PLUGIN_API_BINDGEN_OUTPUT_FILE_PATH={}",
        out_file.display()
    );
    Ok(())
}
