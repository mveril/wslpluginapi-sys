mod header_processing;
mod metadata;
mod nuget;
mod nuspec;
use constcat::concat;

use anyhow::Result;
use cargo_metadata::{Metadata, Package};
use std::{
    fs,
    io::BufReader,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::nuget::{Mode, ensure_package_installed};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::{debug, info, trace, warn};
use zip::ZipArchive;

const WSL_PLUGIN_API_FILE_BASE_NAME: &str = "WslPluginApi";
const WSL_PLUGIN_API_HEADER_FILE_NAME: &str = concat!(WSL_PLUGIN_API_FILE_BASE_NAME, ".h");
const WSL_PLUGIN_API_OUTPUT_FILE_NAME: &str = concat!(WSL_PLUGIN_API_FILE_BASE_NAME, ".rs");

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
    Bindgen {
        #[arg(long, help = "LLVM target triple to use for bindgen generation")]
        target: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    match &cli.command {
        Commands::Bindgen { target } => {
            info!("Running bindgen command...");
            let metadata = fetch_cargo_metadata()?;
            let workspace_root: &Path = metadata.workspace_root.as_ref();

            for package in &metadata.workspace_packages() {
                if package
                    .manifest_path
                    .parent()
                    .map_or(true, |p| p != Path::new(env!("CARGO_MANIFEST_DIR")))
                {
                    process_package(package, workspace_root, target.as_deref())?;
                } else {
                    info!("Skipping package: {}", package.name);
                }
            }
            Ok(())
        }
    }
}

fn fetch_cargo_metadata() -> Result<Metadata> {
    debug!("Fetching cargo metadata...");
    let metadata = cargo_metadata::MetadataCommand::new().exec()?;
    trace!("Full cargo metadata: {:#?}", metadata);
    Ok(metadata)
}

/// `llvm_target` est optionnel (None = autodetect)
fn process_package(
    package: &Package,
    workspace_root: &Path,
    llvm_target: Option<&str>,
) -> anyhow::Result<()> {
    debug!("Processing package: {}", package.name);

    let version = &package.version;
    let nuget_package_version: &str = version.build.as_ref();
    debug!(
        "Package '{}' version: {}",
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
        &nuget_package_version,
        workspace_root,
        Mode::TryNuget,
    )?;
    debug!("NuGet package path: {}", nuget_pkg_path.display());
    let nuspec =
        get_nuspec_from_nupkg(&nuget_pkg_path, nuget_package_name, nuget_package_version)?.unwrap();
    let header_path = get_header_path(&nuget_pkg_path, WSL_PLUGIN_API_HEADER_FILE_NAME)?;
    let bindig = header_processing::process(&header_path, llvm_target)?; // Correction ici : passage d'un Option<&str>
    if let Some(package_path) = package.manifest_path.parent() {
        let build_path = package_path.join("build");
        fs::create_dir_all(&build_path)?;
        let out_file = build_path.join(WSL_PLUGIN_API_OUTPUT_FILE_NAME);
        let metadata_path = build_path.join("metadata.json");
        let metadata = metadata::Metadata::new(
            nuspec.metadata.id,
            nuspec.metadata.version.to_string(),
            &header_path
                .strip_prefix(nuget_pkg_path)?
                .to_string_lossy()
                .into_owned(),
            out_file.strip_prefix(build_path)?.as_str(),
            llvm_target,
        );
        serde_json::to_writer_pretty(fs::File::create(metadata_path)?, &metadata)?;
        bindig.write_to_file(out_file)?;
        Ok(())
    } else {
        warn!(
            "Package manifest path does not have a parent directory: {}",
            std::path::Path::new(package.manifest_path.as_str()).display()
        );
        Ok(())
    }
}

fn get_header_path(nuget_pkg_path: &Path, header_name: &str) -> Result<PathBuf> {
    let mut nuget_native_path = nuget_pkg_path.to_path_buf();
    nuget_native_path.extend(["build", "native", "include", header_name]);
    Ok(nuget_native_path)
}

fn get_nuspec_from_nupkg(
    nuget_pkg_path: &Path,
    nuget_package_name: &str,
    nuget_package_version: &str,
) -> Result<Option<nuspec::Package>> {
    let nuspec_name = format!("{}.nuspec", nuget_package_name);
    debug!("Looking for nuspec file: {}", nuspec_name);

    let nupkg_file = nuget_pkg_path.join(format!(
        "{}.{}.nupkg",
        nuget_package_name, nuget_package_version
    ));
    let zip_file = fs::File::open(&nupkg_file)?;
    let mut archive = ZipArchive::new(zip_file)?;
    trace!("ZIP archive opened with {} files", archive.len());
    match archive.by_name(&nuspec_name) {
        Ok(nuspec_file) => {
            debug!("Found .nuspec file: {}", nuspec_name);
            let nuspec_buffer = BufReader::new(nuspec_file);
            let package_data = nuspec::Package::from_reader(nuspec_buffer)?;
            trace!("Parsed nuspec data: {:#?}", package_data);
            Ok(Some(package_data))
        }
        Err(_) => {
            warn!(
                "Warning: .nuspec file '{}' not found inside {}",
                nuspec_name,
                nuget_pkg_path.display()
            );
            Ok(None)
        }
    }
}
