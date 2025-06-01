use anyhow::Result;
use reqwest::blocking::get;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use tempfile::NamedTempFile;
use zip::ZipArchive;

#[allow(dead_code)]
#[derive(Default)]
pub(crate) enum Mode {
    #[default]
    TryNuget,
    NoNuget,
    Nuget,
}

const LOCAL_NUGET_FOLDER: &str = "nuget_packages";

/// Ensures that a NuGet package is installed into OUT_DIR/nuget_packages.
///
/// # Arguments
/// * `package_name` - The identifier of the NuGet package.
/// * `package_version` - The version string of the package.
/// * `out_dir` - The output directory for the installation.
/// * `mode` - Controls whether to use the NuGet CLI or manual download.
///
/// # Returns
/// The path to the extracted or installed package folder.
pub(crate) fn ensure_package_installed<P: AsRef<Path>, S: AsRef<str>>(
    package_name: S,
    package_version: S,
    out_dir: P,
    mode: Mode,
) -> Result<PathBuf> {
    let out_dir = out_dir.as_ref();
    let package_name = package_name.as_ref();
    let package_version = package_version.as_ref();
    let package_dir = out_dir.join(LOCAL_NUGET_FOLDER);
    let package_output = package_dir.join(format!("{}.{}", package_name, package_version));

    fs::create_dir_all(&package_dir)?;

    match mode {
        Mode::Nuget => {
            install_with_nuget_cli(package_name, package_version, &package_dir)?;
        }
        Mode::NoNuget => {
            download_and_extract(package_name, package_version, &package_output)?;
        }
        Mode::TryNuget => {
            if let Err(e) = install_with_nuget_cli(package_name, package_version, &package_dir) {
                println!(
                    "NuGet CLI failed: {}. Falling back to manual download...",
                    e
                );
                download_and_extract(package_name, package_version, &package_output)?;
            }
        }
    }

    println!("NuGet package installed successfully: {:?}", package_output);
    Ok(package_output)
}

fn install_with_nuget_cli(
    package_name: &str,
    package_version: &str,
    package_dir: &Path,
) -> Result<ExitStatus> {
    println!("Installing NuGet package using NuGet CLI...");
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
    Ok(status)
}

fn download_and_extract(
    package_name: &str,
    package_version: &str,
    package_output: &Path,
) -> Result<()> {
    let package_url = format!(
        "https://www.nuget.org/api/v2/package/{}/{}",
        package_name, package_version
    );
    println!("Downloading NuGet package from: {}", package_url);

    let mut response = get(&package_url)?.error_for_status()?;

    let mut temp_file = NamedTempFile::new()?;
    response.copy_to(&mut temp_file)?;

    let zip_file = fs::File::open(temp_file.path())?;
    let mut archive = ZipArchive::new(zip_file)?;
    println!("Extracting NuGet package to: {:?}", package_output);
    archive.extract(package_output)?;

    Ok(())
}
