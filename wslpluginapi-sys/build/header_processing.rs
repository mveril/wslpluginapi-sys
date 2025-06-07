#[cfg(unix)]
use std::path::PathBuf;
use std::{borrow::Cow, collections::HashMap, env, path::Path};

use bindgen::callbacks::{ParseCallbacks, TypeKind};

use cfg_if::cfg_if;
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

fn rust_to_llvm_target() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("x86_64-pc-windows-gnu", "x86_64-w64-mingw32"),
        ("i686-pc-windows-gnu", "i686-w64-mingw32"),
        ("aarch64-pc-windows-gnu", "aarch64-w64-mingw32"),
        ("x86_64-pc-windows-gnullvm", "x86_64-w64-mingw32"),
        ("i686-pc-windows-gnullvm", "i686-w64-mingw32"),
        ("aarch64-pc-windows-gnullvm ", "aarch64-w64-mingw32"),
        ("x86_64-pc-windows-msvc", "x86_64-windows-msvc"),
        ("i686-pc-windows-msvc", "i686-windows-msvc"),
        ("aarch64-pc-windows-msvc", "aarch64-windows-msvc"),
    ])
}

/// If the host is not Windows, replace `Windows.h` with `windows.h` in a temporary file.
#[cfg(unix)]
fn preprocess_header<P: AsRef<Path>>(
    header_path: P,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    use std::{fs, io::Write, path::PathBuf};

    use crate::WSL_PLUGIN_API_HEADER_FILE;

    let content = fs::read_to_string(&header_path)?;
    let modified_content = content.replace("Windows.h", "windows.h");

    let out_dir: PathBuf = env::var("OUT_DIR")?.into();
    let comp_h_file_path = out_dir.join(format!("unix_"{}, WSL_PLUGIN_API_HEADER_FILE_NAME));
    fs::File::create(&comp_h_file_path)?.write_all(modified_content.as_bytes())?;
    println!("Using modified header file at: {:?}", &comp_h_file_path);
    Ok(comp_h_file_path)
}

pub(crate) fn process<P: AsRef<Path>, S: AsRef<str>>(
    header_file_path: P,
    host: S,
    target: S,
) -> Result<bindgen::Bindings, Box<dyn std::error::Error>> {
    let host = host.as_ref();
    let target = target.as_ref();
    // Here we use cow to have the same type and avoiding clowning the PathBuff
    cfg_if! {
        if #[cfg(unix)] {
            let header_file_path: Cow<'_, Path> = Cow::Owned(preprocess_header(header_file_path)?);
        }
        else {
            let header_file_path: Cow<'_, Path> = Cow::Borrowed(header_file_path.as_ref());
        }
    }
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

    if host != target {
        builder = builder.clang_arg(format!("--target={}", rust_to_llvm_target()[target]))
    }
    let binding = builder.generate()?;
    Ok(binding)
}
