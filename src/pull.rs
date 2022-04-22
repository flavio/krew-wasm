use directories::BaseDirs;
use lazy_static::lazy_static;
use policy_fetcher::registry::config::{read_docker_config_json_file, DockerConfig};
use policy_fetcher::{fetch_policy, PullDestination};
use regex::Regex;
use std::{path::Path, process};
use tracing::debug;

use crate::store::{ALL_MODULES_STORE_ROOT, BIN_ROOT, STORE_ROOT};

lazy_static! {
    static ref TAG_REMOVER: Regex = Regex::new(r":[^:]+$").unwrap();
}

#[derive(PartialEq)]
pub(crate) enum ForcePull {
    ForcePull,
    DoNotForcePull,
}

fn default_docker_config() -> Option<DockerConfig> {
    let docker_config_path =
        BaseDirs::new().map(|bd| bd.home_dir().join(".docker").join("config.json"));
    if let Some(dcp) = docker_config_path {
        if dcp.exists() {
            debug!(file = dcp.to_str(), "loading docker config file");
            Some(read_docker_config_json_file(&dcp).expect("Error reading docker config file"))
        } else {
            debug!("docker config file not found");
            None
        }
    } else {
        debug!("cannot infer location of docker config file");
        None
    }
}

pub(crate) async fn pull(uri: &str, force_pull: ForcePull) {
    let docker_config = default_docker_config();

    // Fetch the wasm module
    let module = fetch_policy(
        uri,
        PullDestination::Store(STORE_ROOT.clone()),
        docker_config.as_ref(),
        None,
    )
    .await
    .expect("failed pulling module");

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

    if Path::exists(&ALL_MODULES_STORE_ROOT.join(&module_name)) {
        if force_pull == ForcePull::DoNotForcePull {
            eprintln!("there is already a module with this name ({}). You can pull with the `-f` flag to overwrite the existing module", module_name);
            process::exit(1);
        }
        // When forcing the pull, rm the module name, so all the
        // cleaning logic of the store is triggered. Then, fetch the
        // module again. This is not neat, and the policy fetcher
        // could be improved to provide the path where the module
        // would have been placed to know before pulling if something
        // existed on the path already. Given force pulling does not
        // happen so often, just pull the policy again.
        crate::rm::rm(module_name);
        fetch_policy(uri, PullDestination::Store(STORE_ROOT.clone()), None, None)
            .await
            .expect("failed pulling module");
    }

    // Create the webassembly module symlink in the "all modules" root
    // TODO(ereslibre): figure out Windows behavior
    std::os::unix::fs::symlink(
        &module_store_path,
        ALL_MODULES_STORE_ROOT.join(&module_name),
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
