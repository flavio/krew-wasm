use lazy_static::lazy_static;
use std::collections::HashSet;
use std::env;
use std::path::Path;
use std::process;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

mod cli;
use clap::Parser;
use cli::{NativeCommands, BINARY_NAME, KREW_WASM_VERBOSE_ENV};

mod errors;
use errors::KrewWapcError;

mod wasm_host;

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

fn main() {
    // setup logging

    let args: Vec<String> = env::args().collect();
    if BINARY_NAMES.contains(&args[0]) {
        // native mode
        let cli = cli::Native::parse();
        setup_logging(cli.verbose);
        run_native(cli);
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
        let wasm_module = format!("{}.wasm", wasm_module_name);

        //TODO remove hard coded path
        let wasm_module_path = Path::new("/home/flavio/").join(wasm_module);
        if wasm_module_path.exists() {
            let wasi_args = wasm_host::WasiArgs::Inherit;
            wasm_host::run_plugin(wasm_module_path, &wasi_args).unwrap();
        } else {
            eprintln!(
                "Cannot find wasm plugin {} at {}",
                wasm_module_name,
                wasm_module_path.to_str().unwrap()
            );
            process::exit(1);
        }
    }
}

fn run_native(cli: cli::Native) {
    match cli.command {
        NativeCommands::Run { module, wasm_args } => {
            let wasm_module_path = Path::new(module.as_str());
            let mut wasm_args = wasm_args.clone();
            wasm_args.insert(
                0,
                wasm_module_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            );
            let wasi_args = wasm_host::WasiArgs::UserProvided(wasm_args);

            match wasm_host::run_plugin(wasm_module_path.to_path_buf(), &wasi_args) {
                Err(e) => match e {
                    KrewWapcError::PlugingExitError { code } => {
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
    }
}
