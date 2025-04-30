mod nuget;
mod nuspec;

use anyhow::Result;
use std::{fs, io::Write, path::Path};

use crate::nuget::{Mode, ensure_package_installed};
use clap::{Parser, Subcommand};
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
    /// Build the project
    Build,
    /// Run tests
    Test,
    /// Format the code
    Fmt,
    /// Run a custom command
    Nuget,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Configure the logger based on the verbosity level
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    match &cli.command {
        Commands::Build => {
            info!("Building the project...");
            debug!("Starting build steps...");
            Ok(())
        }
        Commands::Test => {
            info!("Running tests...");
            debug!("Starting test execution...");
            Ok(())
        }
        Commands::Fmt => {
            info!("Formatting the code...");
            debug!("Running code formatter...");
            Ok(())
        }
        Commands::Nuget => {
            info!("Running Nuget command...");
            debug!("Fetching cargo metadata...");
            let metadata = cargo_metadata::MetadataCommand::new().exec()?;
            trace!("Full cargo metadata: {:#?}", metadata);

            let workspace_root: &Path = metadata.workspace_root.as_ref();
            debug!("Workspace root: {}", workspace_root.display());

            let workspace_packages = metadata
                .packages
                .iter()
                .filter(|pkg| metadata.workspace_members.contains(&pkg.id));

            for package in workspace_packages {
                debug!("Processing package: {}", package.name);

                let version = &package.version;
                let nuget_package_version = &version.build;
                debug!(
                    "Package '{}' build metadata version: {:?}",
                    package.name, nuget_package_version
                );

                if nuget_package_version.is_empty() {
                    warn!("No version found for package: {}", package.name);
                    continue;
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

                let vendored_dir = package.manifest_path.parent().unwrap().join("vendored");
                debug!("Creating vendored directory at: {}", vendored_dir);
                fs::create_dir_all(&vendored_dir)?;

                debug!("Copying native headers...");
                copy(
                    nuget_pkg_path.join("build/native/include/"),
                    &vendored_dir,
                    &CopyOptions::new().overwrite(true),
                )?;

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
                        let licences = package_data.metadata.get_licence_texts();
                        if !licences.is_empty() {
                            let license_dir = vendored_dir.join("LICENSES");
                            fs::create_dir_all(&license_dir)?;
                            debug!("Created LICENSES directory at: {}", license_dir);

                            let license_path =
                                license_dir.join(format!("{}.license", nuget_package_name));
                            debug!("Writing license to: {}", license_path);

                            fs::File::create(license_path)?
                                .write_all(licences.join("\n\n").as_bytes())?;
                        } else {
                            debug!("No license URL specified in nuspec.");
                        }
                    }
                    Err(_) => {
                        warn!(
                            "Warning: .nuspec file '{}' not found inside {}",
                            nuspec_name,
                            nuget_pkg_path.display()
                        );
                    }
                }
            }

            Ok(())
        }
    }
}
