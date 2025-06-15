use cargo_lock::Lockfile;
use std::path::PathBuf;

fn main() {
    let mut root: PathBuf = std::env::var_os("CARGO_MANIFEST_DIR").unwrap().into();
    let lockfile = {
        let mut lockfile = None;
        for _ in 0..=1 {
            let lock = root.join("Cargo.lock");
            if lock.exists() {
                lockfile = Some(Lockfile::load(&lock).expect("Failed to load Cargo.lock"));
                break;
            } else {
                root.pop();
            }
        }
        lockfile.expect("Cargo.lock not found")
    };
    
    if let Some(pkg) = lockfile
        .packages
        .iter()
        .find(|pkg| pkg.name.as_str() == "xtask")
    {
        if let Some(dep) = pkg
            .dependencies
            .iter()
            .find(|dep| dep.name.as_str() == "bindgen")
        {
            println!("cargo:rustc-env=BINDGEN_VERSION={}", dep.version);
        } else {
            println!("cargo:warning=No bindgen dependency found in xtask package");
        }
    } else {
        println!("cargo:warning=No xtask package found in Cargo.lock");
    }
}
