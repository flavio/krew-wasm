use std::path::Path;
use std::process;

use crate::errors::KrewWapcError;
use crate::wasm_host;

pub(crate) fn run(module: String, wasm_args: Vec<String>) {
    let wasm_module_path = Path::new(module.as_str());
    let wasm_filename = wasm_module_path.file_name().unwrap().to_string_lossy();
    let plugin_name = wasm_filename
        .strip_suffix(".wasm")
        .map(|s| s.to_string())
        .unwrap_or_else(|| wasm_filename.to_string());
    let kubectl_plugin_name = if plugin_name.starts_with("kubectl-") {
        plugin_name
    } else {
        format!("kubectl-{}", plugin_name)
    };

    let mut wasm_args = wasm_args;
    wasm_args.insert(0, kubectl_plugin_name);
    let wasi_args = wasm_host::WasiArgs::UserProvided(wasm_args);

    match wasm_host::run_plugin(wasm_module_path.to_path_buf(), &wasi_args) {
        Err(e) => match e {
            KrewWapcError::PluginExitError { code } => {
                println!();
                process::exit(code)
            }
            _ => {
                eprintln!("{:?}", e);
                process::exit(1)
            }
        },
        Ok(_) => process::exit(0),
    }
}
