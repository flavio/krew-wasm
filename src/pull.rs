use directories::{ProjectDirs, UserDirs};
use lazy_static::lazy_static;
use policy_fetcher::{fetch_policy, PullDestination};
use regex::Regex;
use std::{
    path::{Path, PathBuf},
    process,
};

use crate::cli::BINARY_NAME;

lazy_static! {
    static ref BIN_ROOT: PathBuf = UserDirs::new()
        .expect("cannot find home directory for user")
        .home_dir()
        .join(".krew-wasm")
        .join("bin");
    static ref TAG_REMOVER: Regex = Regex::new(r":[^:]+$").unwrap();
    static ref STORE_ROOT: PathBuf = ProjectDirs::from("io.krew-wasm", "", "krew-wasm")
        .expect("cannot find project dirs")
        .cache_dir()
        .join("krew-wasm-store");
    pub static ref ALL_MODULES_STORE_ROOT: PathBuf = STORE_ROOT.join("all");
}

pub(crate) async fn pull(uri: String) {
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
    let all_modules_store_root = &ALL_MODULES_STORE_ROOT;
    std::fs::create_dir_all(all_modules_store_root.as_path())
        .expect("could not create top level store path for all modules");

    // Fetch the wasm module
    let module = fetch_policy(&uri, PullDestination::Store(STORE_ROOT.clone()), None, None)
        .await
        .expect("failed pulling module");

    // Create the webassembly module symlink in the "all modules" root
    let module_store_path = module.local_path;
    let module_name = TAG_REMOVER
        .replace(
            module_store_path
                .file_name()
                .expect("missing filename")
                .to_str()
                .expect("bad filename"),
            "",
        )
        .to_string();
    let module_name = module_name.strip_suffix(".wasm").unwrap_or(&module_name);

    if Path::exists(&all_modules_store_root.join(&module_name)) {
        eprintln!("there is already a module with this name. Run `{} rm {}` to remove it, and run this command again", BINARY_NAME, module_name);
        process::exit(1);
    }

    // TODO(ereslibre): figure out Windows behavior
    std::os::unix::fs::symlink(
        &module_store_path,
        all_modules_store_root.join(&module_name),
    )
    .expect("error symlinking top level module");

    // Create the kubectl plugin symlink pointing to ourselves
    let kubectl_plugin_name = format!("kubectl-{}", &module_name,);
    // TODO(ereslibre): figure out Windows behavior
    std::os::unix::fs::symlink(
        std::env::current_exe().expect("cannot find current executable"),
        &BIN_ROOT.join(&kubectl_plugin_name),
    )
    .expect("error symlinking kubectl plugin");

    println!("module was pulled successfully. Make sure to add {} to your $PATH so that `kubectl` can find the {} plugin", BIN_ROOT.display(), kubectl_plugin_name);
}
