extern crate bindgen;
extern crate semver;

use bindgen::callbacks::{ParseCallbacks, TypeKind};
use semver::Version;
use std::env;
use std::path::PathBuf;
use std::process::Command;

const WSL_PACKAGE_NAME: &str = "Microsoft.WSL.PluginApi";
const LOCAL_NUGET_FOLDER: &str = "nuget_packages";
const WSL_PLUGIN_API_BINDGEN_OUTPUT_FILE_NAME: &str = "WSLPluginApi.rs";

#[derive(Debug, Default)]
struct BindgenCallback {
    generate_hooks_fields_name: bool,
}

impl BindgenCallback {
    fn new(generate_hooks_fields_names: bool) -> Self {
        Self {
            generate_hooks_fields_name: generate_hooks_fields_names,
        }
    }
}

impl ParseCallbacks for BindgenCallback {
    fn add_derives(&self, info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
        let mut derives = Vec::new();

        if info.kind == TypeKind::Struct {
            if info.name == "WSLVersion" {
                derives.extend(vec![
                    "Eq".to_string(),
                    "PartialEq".to_string(),
                    "Ord".to_string(),
                    "PartialOrd".to_string(),
                    "Hash".to_string(),
                ]);
            } else if info.name.contains("PluginHooks") && self.generate_hooks_fields_name {
                derives.push("FieldNamesAsSlice".to_string());
            }
        }

        derives
    }
}

/// Ensures that the NuGet package is installed in the local folder.
fn ensure_package_installed(
    package_name: &str,
    package_version: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let out_dir: PathBuf = env::var("OUT_DIR")?.into();
    let package_dir = out_dir.join(LOCAL_NUGET_FOLDER);
    let status = Command::new("nuget")
        .args([
            "install",
            package_name,
            "-Version",
            package_version,
            "-OutputDirectory",
            package_dir.to_str().unwrap(),
            "-NonInteractive",
        ])
        .status()?;

    if !status.success() {
        return Err(format!(
            "NuGet install command failed with status: {:?}",
            status.code()
        )
        .into());
    }
    Ok(package_dir.join(format!("{}.{}", package_name, package_version)))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    // Extract version from Cargo package metadata
    let version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    println!("cargo:version={}", version);

    if !version.build.is_empty() {
        println!("cargo:build-metadata={}", version.build);
    }

    let package_version = version.build.to_string();
    let out_path: PathBuf = env::var("OUT_DIR")?.into();

    // Ensure the NuGet package is installed
    let package_path = ensure_package_installed(WSL_PACKAGE_NAME, &package_version)?;

    // Construct paths
    let header_file_path = package_path.join("build/native/include/WslPluginApi.h");

    if !header_file_path.exists() {
        return Err(format!("Header file does not exist: {:?}", header_file_path).into());
    }

    println!("Using header file from: {:?}", header_file_path);

    let hooks_fields_name_feature = env::var("CARGO_FEATURE_HOOKS_FIELD_NAMES").is_ok();
    let mut builder = bindgen::Builder::default()
        .header(header_file_path.to_str().unwrap())
        .raw_line("use windows::core::*;")
        .raw_line("use windows::Win32::Foundation::*;")
        .raw_line("use windows::Win32::Security::*;")
        .raw_line("use windows::Win32::Networking::WinSock::SOCKET;")
        .raw_line("#[allow(clippy::upper_case_acronyms)] type LPCWSTR = PCWSTR;")
        .raw_line("#[allow(clippy::upper_case_acronyms)] type LPCSTR = PCSTR;")
        .raw_line("#[allow(clippy::upper_case_acronyms)] type DWORD = u32;")
        .derive_debug(true)
        .derive_copy(true)
        .allowlist_item("WSL.*")
        .allowlist_item("Wsl.*")
        .clang_arg("-fparse-all-comments")
        .allowlist_recursively(false)
        .parse_callbacks(Box::new(BindgenCallback::new(hooks_fields_name_feature)))
        .generate_comments(true);

    if hooks_fields_name_feature {
        builder = builder.raw_line("use struct_field_names_as_array::FieldNamesAsSlice;");
    }

    // Generate Rust bindings
    let api_header = builder.generate()?;

    // Write bindings to OUT_DIR
    let out_file = out_path.join(WSL_PLUGIN_API_BINDGEN_OUTPUT_FILE_NAME);
    api_header.write_to_file(&out_file)?;

    Ok(())
}
