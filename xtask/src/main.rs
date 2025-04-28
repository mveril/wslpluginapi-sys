mod nuget;
use std::{fs, path::Path};

use crate::nuget::ensure_package_installed;
use clap::{Parser, Subcommand};
use fs_extra::dir::{CopyOptions, copy};
use nuget::Mode;

/// Custom build and development tasks for the project.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Build => {
            println!("Building the project...");
            Ok(())
            // TODO: Add your build logic here
        }
        Commands::Test => {
            println!("Running tests...");
            Ok(())
            // TODO: Add your test logic here
        }
        Commands::Fmt => {
            println!("Formatting the code...");
            Ok(())
            // TODO: Add your fmt logic here
        }
        Commands::Nuget => {
            let metadata = cargo_metadata::MetadataCommand::new().exec()?;
            let workspace_folder: &Path = metadata.workspace_root.as_ref();
            for package in metadata.workspace_default_packages() {
                let version = &package.version;
                let nuget_package_version = &version.build;
                if nuget_package_version.is_empty() {
                    eprintln!("No version found for package: {}", package.name);
                    continue;
                }
                let nuget_package_name = "Microsoft.WSL.PluginApi";
                let nuget_pkg_path = ensure_package_installed(
                    nuget_package_name,
                    nuget_package_version.as_str(),
                    workspace_folder,
                    Mode::TryNuget,
                )?;
                let vendored = package
                    .manifest_path
                    .parent()
                    .unwrap()
                    .join("vendored");
                fs::create_dir_all(&vendored)?;
                // Here we copy all the includes from the nuget package to the vendored includes folder
                copy(
                    nuget_pkg_path.join("build/native/include/").as_path(),
                    vendored.as_path(),
                    &CopyOptions::new().overwrite(true),
                )?;
            }
            Ok(())
        }
    }
}
