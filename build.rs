extern crate bindgen;
extern crate semver;

use bindgen::callbacks::{ParseCallbacks, TypeKind};
use cfg_if::cfg_if;
use semver::Version;
use std::env;
use std::path::PathBuf;
cfg_if! {
    if #[cfg(feature = "no-nuget")] {
        use reqwest::blocking::get;
        use std::fs;
        use zip::ZipArchive;
        use tempfile::NamedTempFile;
    } else {
        use std::process::Command;
    }
}

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
    let out_dir: PathBuf = env::var("OUT_DIR")
        .map(PathBuf::from)
        .map_err(|_| "OUT_DIR environment variable is not set")?;
    let package_dir = out_dir.join(LOCAL_NUGET_FOLDER);
    let package_output = package_dir.join(format!("{}.{}", package_name, package_version));

    cfg_if! {
        if #[cfg(feature = "no-nuget")] {
            fs::create_dir_all(&package_dir)?;

            let package_url = format!(
                "https://www.nuget.org/api/v2/package/{}/{}",
                package_name, package_version
            );
            println!("Downloading NuGet package from: {}", package_url);

            let mut response = get(&package_url)?;
            if !response.status().is_success() {
                return Err(format!("Failed to download NuGet package: HTTP {}", response.status()).into());
            }

            // Créer un fichier temporaire pour stocker l'archive
            let mut temp_file = NamedTempFile::new()?;
            response.copy_to(&mut temp_file)?;

            // Ouvrir le fichier temporaire pour extraction
            let temp_path = temp_file.path();
            let zip_file = fs::File::open(temp_path)?;
            let mut archive = ZipArchive::new(zip_file)?;

            println!("Extracting NuGet package to: {:?}", package_output);
            archive.extract(&package_output)?;

        } else {
            println!("Installing NuGet package using NuGet CLI...");

            let status = Command::new("nuget")
                .args([
                    "install",
                    package_name,
                    "-Version",
                    package_version,
                    "-OutputDirectory",
                    package_dir.to_str().ok_or("Invalid package directory path")?,
                    "-NonInteractive",
                ])
                .status()?;

            if !status.success() {
                return Err(format!(
                    "NuGet install command failed with status: {:?}",
                    status.code()
                ).into());
            }
        }
    }

    println!("NuGet package installed successfully: {:?}", package_output);
    Ok(package_output)
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

    if std::env::var("HOST").unwrap() != std::env::var("TARGET").unwrap() {
        builder = builder.clang_arg(format!("--target={}", std::env::var("TARGET").unwrap()));
    }

    // Generate Rust bindings
    let api_header = builder.generate()?;

    // Write bindings to OUT_DIR
    let out_file = out_path.join(WSL_PLUGIN_API_BINDGEN_OUTPUT_FILE_NAME);
    api_header.write_to_file(&out_file)?;

    Ok(())
}
