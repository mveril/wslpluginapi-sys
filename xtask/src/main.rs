mod licence_definition;
mod nuget;
mod nuspec;

use anyhow::Result;
use nuspec::LicenceContent;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::nuget::{Mode, ensure_package_installed};
use clap::{Parser, Subcommand, builder::OsStr};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use env_logger;
use fs_extra::dir::{CopyOptions, copy};
use log::{debug, error, info, trace, warn};
use reqwest::blocking::get;
use zip::ZipArchive;

/// Tâches de build et développement personnalisées pour le projet.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Nuget,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Configure le logger selon le niveau de verbosité
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    match &cli.command {
        Commands::Nuget => {
            info!("Running Nuget command...");
            let metadata = fetch_cargo_metadata()?;

            let workspace_root: &Path = &metadata.workspace_root.as_ref();

            for package in metadata.workspace_packages() {
                if let Some(parent) = package.manifest_path.parent() {
                    if parent.as_str() != env!("CARGO_MANIFEST_DIR") {
                        process_package(package, workspace_root)?;
                    }
                } else {
                    process_package(package, workspace_root)?;
                }
            }
            Ok(())
        }
    }
}

fn fetch_cargo_metadata() -> Result<cargo_metadata::Metadata> {
    debug!("Fetching cargo metadata...");
    let metadata = cargo_metadata::MetadataCommand::new().exec()?;
    trace!("Full cargo metadata: {:#?}", metadata);
    Ok(metadata)
}

fn process_package(package: &cargo_metadata::Package, workspace_root: &Path) -> Result<()> {
    debug!("Processing package: {}", package.name);

    let version = &package.version;
    let nuget_package_version = &version.build;
    debug!(
        "Package '{}' build metadata version: {:?}",
        package.name, nuget_package_version
    );

    if nuget_package_version.is_empty() {
        warn!("No version found for package: {}", package.name);
        return Ok(());
    }

    let nuget_package_name = "Microsoft.WSL.PluginApi";
    debug!(
        "Ensuring NuGet package installed: {} @ {}",
        nuget_package_name, nuget_package_version
    );

    let nuget_pkg_path = ensure_package_installed(
        nuget_package_name,
        nuget_package_version.as_str(),
        workspace_root,
        Mode::TryNuget,
    )?;
    debug!("NuGet package path: {}", nuget_pkg_path.display());

    let third_party_dir = package.manifest_path.parent().unwrap().join("third_party");
    let third_party_wsl_nuget_dir = third_party_dir.join(nuget_package_name);
    prepare_third_party_dirs(
        &third_party_dir.as_std_path(),
        &third_party_wsl_nuget_dir.as_std_path(),
    )?;
    copy_native_headers(&nuget_pkg_path, &third_party_wsl_nuget_dir.as_std_path())?;

    handle_nuspec_and_licenses(
        &nuget_pkg_path,
        nuget_package_name,
        nuget_package_version.as_str(),
        &third_party_wsl_nuget_dir.as_std_path(),
    )?;

    Ok(())
}

fn prepare_third_party_dirs(
    third_party_dir: &Path,
    third_party_wsl_nuget_dir: &Path,
) -> Result<()> {
    debug!(
        "Creating third_party directory at: {}",
        third_party_dir.display()
    );
    fs::create_dir_all(third_party_dir)?;
    debug!(
        "Creating third_party directory for the package at: {}",
        third_party_wsl_nuget_dir.display()
    );
    fs::create_dir_all(third_party_wsl_nuget_dir)?;
    Ok(())
}

fn copy_native_headers(nuget_pkg_path: &Path, third_party_wsl_nuget_dir: &Path) -> Result<()> {
    debug!("Copying native headers...");
    copy(
        nuget_pkg_path.join("build/native/include/"),
        third_party_wsl_nuget_dir,
        &CopyOptions::new().overwrite(true),
    )?;
    Ok(())
}

fn handle_nuspec_and_licenses(
    nuget_pkg_path: &Path,
    nuget_package_name: &str,
    nuget_package_version: &str,
    third_party_wsl_nuget_dir: &Path,
) -> Result<()> {
    let nuspec_name = format!("{}.nuspec", nuget_package_name);
    debug!("Looking for nuspec file: {}", nuspec_name);

    let zip_file = fs::File::open(&nuget_pkg_path.join(format!(
        "{}.{}.nupkg",
        nuget_package_name, nuget_package_version
    )))?;
    let mut archive = ZipArchive::new(zip_file)?;
    trace!("ZIP archive opened with {} files", archive.len());

    match archive.by_name(&nuspec_name) {
        Ok(nuspec_file) => {
            debug!("Found .nuspec file: {}", nuspec_name);
            let package_data: nuspec::Package = serde_xml_rs::from_reader(nuspec_file)?;
            trace!("Parsed nuspec data: {:#?}", package_data);
            handle_readme_and_license(&package_data, nuget_pkg_path, third_party_wsl_nuget_dir)?;
        }
        Err(_) => {
            warn!(
                "Warning: .nuspec file '{}' not found inside {}",
                nuspec_name,
                nuget_pkg_path.display()
            );
        }
    }
    Ok(())
}

fn handle_readme_and_license(
    package_data: &nuspec::Package,
    nuget_pkg_path: &Path,
    third_party_wsl_nuget_dir: &Path,
) -> Result<()> {
    if let Some(readme_nuget_path) = package_data.metadata.readme.as_deref() {
        let readme_path = third_party_wsl_nuget_dir.join(
            &readme_nuget_path
                .file_name()
                .unwrap_or(&OsStr::from("README")),
        );
        debug!("Copying README file to: {}", readme_path.display());
        fs::copy(nuget_pkg_path.join(readme_nuget_path), readme_path)?;
    } else {
        info!("No README file specified in nuspec.");
    }
    if let Some(licence_content) = package_data.metadata.get_licence_content()? {
        match licence_content {
            LicenceContent::Body(body) => {
                debug!("License file or expression found in nuspec.");
                match body {
                    nuspec::LicenceBody::Generator(generator) => {
                        debug!("License generator found in nuspec.");
                        let license_body = generator.generate_body();
                        let license_path = if license_body.len() == 1 {
                            third_party_wsl_nuget_dir.join("LICENSE")
                        } else {
                            third_party_wsl_nuget_dir.join("LICENSES")
                        };
                        debug!("Writing license to: {}", &license_path.display());
                        fs::File::create(&license_path)?
                            .write_all(license_body.join("\n\n").as_bytes())?;
                    }
                    nuspec::LicenceBody::File(file) => {
                        debug!("License file found in nuspec.");
                        let license_path = third_party_wsl_nuget_dir.join("LICENSE");
                        debug!("Copy license to: {}", &license_path.display());
                        fs::copy(nuget_pkg_path.join(file), license_path)?;
                    }
                }
            }
            LicenceContent::URL(url) => {
                debug!("License URL found in nuspec: {}", url);
            }
        }
    }
    Ok(())
}
