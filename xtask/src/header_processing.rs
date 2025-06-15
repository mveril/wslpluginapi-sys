use bindgen::callbacks::{ParseCallbacks, TypeKind};
use cfg_if::cfg_if;
use clap::builder;
#[cfg(unix)]
use cow_utils::CowUtils;
use std::{borrow::Cow, path::Path, vec};
#[cfg(unix)]
use std::{fs, io};

#[derive(Debug)]
struct BindgenCallback;

impl BindgenCallback {}

impl ParseCallbacks for BindgenCallback {
    fn add_derives(&self, info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
        if info.kind == TypeKind::Struct && info.name == "WSLVersion" {
            ["Eq", "PartialEq", "Ord", "PartialOrd", "Hash"]
                .into_iter()
                .map(|s| s.into())
                .collect()
        } else {
            Vec::default()
        }
    }

    fn add_attributes(&self, _info: &bindgen::callbacks::AttributeInfo<'_>) -> Vec<String> {
        if _info.kind == TypeKind::Struct && _info.name == "WSLPluginHooksV1" {
            vec!["#[cfg_attr(feature=\"hooks-field-names\", derive(FieldNamesAsSlice))]".into()]
        } else {
            Vec::default()
        }
    }
}

/// If the host is not Windows, replace `Windows.h` with `windows.h` in a temporary file.
#[cfg(unix)]
fn preprocess_header<'a, P: 'a + AsRef<Path>>(
    header_path: &'a P,
    target: Option<&str>,
) -> io::Result<String> {
    let content = fs::read_to_string(&header_path)?;
    let (old, new) = if target.unwrap_or_default().ends_with("-msvc") {
        ("windows.h", "Windows.h")
    } else {
        ("Windows.h", "windows.h")
    };
    let content = match content.cow_replace(old, new) {
        Cow::Borrowed(_) => content,
        Cow::Owned(owned) => owned,
    };
    Ok(content)
}

pub(crate) fn process<P: AsRef<Path>>(
    header_file_path: P,
    target: Option<&str>,
) -> anyhow::Result<bindgen::Bindings> {
    // Here we use a custom struct because the file can be temporary.
    let mut builder = bindgen::Builder::default()
        .raw_line("use windows::core::*;")
        .raw_line("use windows::Win32::Foundation::*;")
        .raw_line("use windows::Win32::Security::*;")
        .raw_line("use windows::Win32::Networking::WinSock::SOCKET;")
        .raw_line("#[allow(clippy::upper_case_acronyms)] type LPCWSTR = PCWSTR;")
        .raw_line("#[allow(clippy::upper_case_acronyms)] type LPCSTR = PCSTR;")
        .raw_line("#[allow(clippy::upper_case_acronyms)] type DWORD = u32;")
        .raw_line(r#"#[cfg(feature = "hooks-field-names")]"#)
        .raw_line("use struct_field_names_as_array::FieldNamesAsSlice;")
        .allowlist_item("WSL.*")
        .allowlist_item("Wsl.*")
        .clang_arg("-fparse-all-comments")
        .allowlist_recursively(false)
        .parse_callbacks(Box::new(BindgenCallback))
        .generate_comments(true);

    if let Some(llvm_target) = target {
        builder = builder.clang_arg(format!("--target={}", llvm_target));
    }
    cfg_if!(
        if #[cfg(unix)] {
            let content = preprocess_header(&header_file_path, target)?;
            builder = builder.header_contents(crate::WSL_PLUGIN_API_HEADER_FILE_NAME, content.as_ref())
        } else {
            builder = builder.header(header_file_path.as_ref().to_str().unwrap());
        }
    );
    let binding = builder.generate()?;
    Ok(binding)
}
