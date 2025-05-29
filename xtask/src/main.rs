mod licence_definition;
mod nuget;
mod nuspec;
mod third_pary_mangement;

use anyhow::Result;
use nuspec::LicenceContent;
use std::{fs, io::Write, iter::once, path::Path};
use third_pary_mangement::{
    DistributedFile, Status,
    notice::{NoticeGeneration, ThirdPartyNotice, ThirdPartyNoticeItem, ThirdPartyNoticePackage},
};

use crate::nuget::{Mode, ensure_package_installed};
use clap::{Parser, Subcommand, builder::OsStr};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use env_logger;
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
            let mut notice = ThirdPartyNotice::default();
            for package in metadata.workspace_packages() {
                let notice = if package
                    .manifest_path
                    .parent()
                    .map_or(true, |p| p.as_str() != env!("CARGO_MANIFEST_DIR"))
                {
                    notice.push(process_package(package, workspace_root)?);
                } else {
                    info!("Skipping package: {}", package.name);
                    continue;
                };
            }
            notice.generate_notice(&workspace_root.join("THIRD-PARTY-NOTICES.md"))?;
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

fn process_package(
    package: &cargo_metadata::Package,
    workspace_root: &Path,
) -> Result<ThirdPartyNoticePackage> {
    debug!("Processing package: {}", package.name);

    let version = &package.version;
    let nuget_package_version = &version.build;
    debug!(
        "Package '{}' build metadata version: {:?}",
        package.name, nuget_package_version
    );
    if nuget_package_version.is_empty() {
        warn!("No version found for package: {}", package.name);
        return Ok(ThirdPartyNoticePackage::new(package.name.clone()));
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
    let nuspec_data = get_nuspec_from_nupkg(
        &nuget_pkg_path,
        nuget_package_name,
        nuget_package_version.as_str(),
    )?
    .unwrap();
    let licence: Option<LicenceContent> = nuspec_data.metadata.get_licence_content()?;
    let mut notice_item = ThirdPartyNoticeItem::new(
        nuget_package_name.into(),
        nuspec_data.metadata.version.clone(),
        format!(
            "https://www.nuget.org/packages/{}/{}",
            nuspec_data.metadata.id, nuspec_data.metadata.version,
        ),
        nuspec_data.metadata.copyright.clone(),
        licence,
    );
    let headers = copy_native_headers(&nuget_pkg_path, &third_party_wsl_nuget_dir.as_std_path())?;
    notice_item.files_mut().extend(headers);
    let readme: Option<DistributedFile> = handle_readme(
        &nuspec_data,
        nuget_pkg_path.as_ref(),
        third_party_wsl_nuget_dir.as_ref(),
    )?;
    notice_item.files_mut().extend(readme.into_iter());
    let licenses = handle_license(
        &nuspec_data,
        nuget_pkg_path.as_ref(),
        third_party_wsl_nuget_dir.as_ref(),
    )?;
    notice_item.files_mut().extend(licenses.into_iter());
    let mut notice = ThirdPartyNoticePackage::new(package.name.clone());
    notice.push(notice_item);
    notice.generate_notice(
        &package
            .manifest_path
            .parent()
            .unwrap()
            .join("THIRD-PARTY-NOTICES.md"),
    )?;
    Ok(notice)
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

fn copy_native_headers(
    nuget_pkg_path: &Path,
    third_party_wsl_nuget_dir: &Path,
) -> Result<Vec<DistributedFile>> {
    debug!("Copying native headers...");
    let native_include_path = nuget_pkg_path.join("build/native/include/");
    let mut vec = Vec::default();
    for entry in fs::read_dir(&native_include_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            debug!("Copying directory: {}", path.display());
            fs::create_dir_all(third_party_wsl_nuget_dir.join(path.file_name().unwrap()))?;
        } else {
            debug!("Copying file: {}", path.display());
            fs::copy(
                &path,
                third_party_wsl_nuget_dir.join(path.file_name().unwrap()),
            )?;
            let distributed_file = DistributedFile::new(path, Status::Unmodified);
        }
    }
    vec.shrink_to_fit();
    Ok(vec)
}

fn get_nuspec_from_nupkg(
    nuget_pkg_path: &Path,
    nuget_package_name: &str,
    nuget_package_version: &str,
) -> Result<Option<nuspec::Package>> {
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

fn handle_readme(
    package_data: &nuspec::Package,
    nuget_pkg_path: &Path,
    third_party_wsl_nuget_dir: &Path,
) -> Result<Option<DistributedFile>> {
    if let Some(readme_nuget_path) = package_data.metadata.readme.as_deref() {
        let readme_path = third_party_wsl_nuget_dir.join(
            &readme_nuget_path
                .file_name()
                .unwrap_or(&OsStr::from("README")),
        );
        debug!("Copying README file to: {}", readme_path.display());
        fs::copy(nuget_pkg_path.join(readme_nuget_path), &readme_path)?;
        return Ok(Some(DistributedFile::new(readme_path, Status::Unmodified)));
    } else {
        info!("No README file specified in nuspec.");
    }
    Ok(None)
}

fn handle_license(
    package_data: &nuspec::Package,
    nuget_pkg_path: &Path,
    third_party_wsl_nuget_dir: &Path,
) -> Result<Option<DistributedFile>> {
    let some_licence_content = package_data.metadata.get_licence_content()?;
    if let Some(licence_content) = some_licence_content {
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
                        Ok(Some(DistributedFile::new(
                            license_path,
                            Status::PackageMetadataGenerated,
                        )))
                    }
                    nuspec::LicenceBody::File(file) => {
                        debug!("License file found in nuspec.");
                        let license_path = third_party_wsl_nuget_dir.join("LICENSE");
                        debug!("Copy license to: {}", &license_path.display());
                        fs::copy(nuget_pkg_path.join(file), &license_path)?;
                        Ok(Some(DistributedFile::new(license_path, Status::Unmodified)))
                    }
                }
            }
            LicenceContent::URL(url) => {
                debug!("License URL found in nuspec: {}", url);
                Ok(None)
            }
        }
    } else {
        debug!("No license file or expression specified in nuspec.");
        Ok(None)
    }
}
