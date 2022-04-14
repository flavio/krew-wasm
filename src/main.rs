use lazy_static::lazy_static;
use std::collections::HashSet;
use std::env;
use std::path::Path;
use std::process;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

mod cli;
mod errors;
mod store;
mod wasm_host;

mod ls;
mod pull;
mod rm;
mod run;

use clap::Parser;
use cli::{NativeCommands, BINARY_NAME, KREW_WASM_VERBOSE_ENV};

use errors::KrewWapcError;

use store::ALL_MODULES_STORE_ROOT;

lazy_static! {
    // Useful when developing the project: `cargo run` leads to a
    // different argv[0]
    static ref BINARY_NAMES: HashSet<String> = {
        let mut set = HashSet::new();
        set.insert(format!("target/debug/{}", BINARY_NAME));
        set.insert(format!("target/release/{}", BINARY_NAME));
        set.insert(format!("./{}", BINARY_NAME));
        set.insert(BINARY_NAME.to_string());
        set
    };
}

fn setup_logging(verbose: bool) {
    let level_filter = if verbose { "debug" } else { "info" };
    let filter_layer = EnvFilter::new(level_filter)
        .add_directive("cranelift_codegen=off".parse().unwrap()) // this crate generates lots of tracing events we don't care about
        .add_directive("cranelift_wasm=off".parse().unwrap()) // this crate generates lots of tracing events we don't care about
        .add_directive("wasmtime_cranelift=off".parse().unwrap()) // this crate generates lots of tracing events we don't care about
        .add_directive("hyper=off".parse().unwrap()) // this crate generates lots of tracing events we don't care about
        .add_directive("regalloc=off".parse().unwrap()); // this crate generates lots of tracing events we don't care about
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt::layer().with_writer(std::io::stderr))
        .init();
}

#[tokio::main]
async fn main() {
    // setup logging

    store::ensure();

    let args: Vec<String> = env::args().collect();
    if BINARY_NAMES.contains(&args[0]) {
        // native mode
        let cli = cli::Native::parse();
        setup_logging(cli.verbose);
        run_native(cli).await;
    } else {
        // wrapper mode
        let verbose = match std::env::var_os(KREW_WASM_VERBOSE_ENV) {
            Some(v) => v == "1",
            None => false,
        };
        setup_logging(verbose);

        let invocation = Path::new(&args[0]).file_name().unwrap().to_str().unwrap();
        let wasm_module_name = match invocation.strip_prefix("kubectl-") {
            Some(n) => n,
            None => args[0].as_str(),
        };

        let wasm_module_path = ALL_MODULES_STORE_ROOT.join(wasm_module_name);
        if wasm_module_path.exists() {
            let wasi_args = wasm_host::WasiArgs::Inherit;
            match wasm_host::run_plugin(wasm_module_path, &wasi_args) {
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
        } else {
            eprintln!(
                "Cannot find wasm plugin {} at {}. Use `krew-wasm pull` to pull it to the store from an OCI registry",
                wasm_module_name,
                wasm_module_path.to_str().unwrap(),
            );
            process::exit(1);
        }
    }
}

async fn run_native(cli: cli::Native) {
    match cli.command {
        NativeCommands::List => ls::ls(),
        NativeCommands::Pull { uri, force } => {
            let force_pull = if force {
                pull::ForcePull::ForcePull
            } else {
                pull::ForcePull::DoNotForcePull
            };
            pull::pull(&uri, force_pull).await
        }
        NativeCommands::Rm { module } => rm::rm(&module),
        NativeCommands::Run { module, wasm_args } => run::run(module, wasm_args),
    }
}
