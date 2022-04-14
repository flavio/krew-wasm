use anyhow::Result;
use directories::{ProjectDirs, UserDirs};
use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    pub(crate) static ref BIN_ROOT: PathBuf = UserDirs::new()
        .expect("cannot find home directory for user")
        .home_dir()
        .join(".krew-wasm")
        .join("bin");
    pub(crate) static ref STORE_ROOT: PathBuf = ProjectDirs::from("io.krew-wasm", "", "krew-wasm")
        .expect("cannot find project dirs")
        .cache_dir()
        .join("krew-wasm-store");
    pub(crate) static ref ALL_MODULES_STORE_ROOT: PathBuf = STORE_ROOT.join("all");
}

// Given a module name, return a tuple with elements that can be
// unlinked directly in the first component of the tuple, and a second
// argument with the full path to the location in the store. In order
// to leave nothing behind in the store, we need to clean up every
// directory until the root of the store after unlinking the module
// from the store.
pub(crate) fn all_module_paths(module_name: &str) -> Result<(Vec<PathBuf>, PathBuf)> {
    let module_bin = BIN_ROOT.join(format!("kubectl-{}", module_name));
    let module_root = ALL_MODULES_STORE_ROOT.join(module_name);
    let module_path = std::fs::read_link(&module_root)?;
    Ok((vec![module_bin, module_root], module_path))
}

pub(crate) fn ensure() {
    // Try to create the kubectl plugin bin path.
    std::fs::create_dir_all(BIN_ROOT.as_path()).unwrap_or_else(|err| {
        panic!(
            "could not create alias binary root at {}: {}",
            BIN_ROOT.display(),
            err
        )
    });
    // Try to create the "all modules" root on the store. Used
    // to look for modules given a name.
    std::fs::create_dir_all(ALL_MODULES_STORE_ROOT.as_path())
        .expect("could not create top level store path for all modules");
}
