extern crate bindgen;
extern crate semver;

use cfg_if::cfg_if;
use std::{env, path::PathBuf};
#[cfg(any(unix, feature = "no-nuget"))]
use std::fs;

cfg_if! {
    if #[cfg(feature = "no-nuget")] {
        use reqwest::blocking::get;
        use zip::ZipArchive;
        use tempfile::NamedTempFile;
    } else {
        use std::process::Command;
    }
}
const LOCAL_NUGET_FOLDER: &str = "nuget_packages";

pub(crate) fn ensure_package_installed(
    package_name: &str,
    package_version: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let out_dir: PathBuf = env::var("OUT_DIR")?.into();
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

            let mut temp_file = NamedTempFile::new()?;
            response.copy_to(&mut temp_file)?;

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
