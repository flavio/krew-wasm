use lazy_static::lazy_static;
use std::collections::HashSet;
use std::path::Path;
use std::process;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

mod cli;
use clap::Parser;
use cli::{Commands, BINARY_NAME};

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

fn main() {
    let shared_cli = cli::Shared::parse();

    // setup logging
    let level_filter = if shared_cli.verbose { "debug" } else { "info" };
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

    let args: Vec<String> = std::env::args().collect();
    if BINARY_NAMES.contains(&args[0]) {
        let cli = cli::Native::parse();
        run_native(cli);
    } else {
        let invocation = Path::new(&args[0]).file_name().unwrap().to_str().unwrap();
        let wasm_module_name = match invocation.strip_prefix("kubectl-") {
            Some(n) => n,
            None => args[0].as_str(),
        };
        let wasm_module = format!("{}.wasm", wasm_module_name);

        //TODO remove hard coded path
        let wasm_module_path = Path::new("/home/flavio/").join(wasm_module);
        if wasm_module_path.exists() {
            wasm_host::run_plugin(wasm_module_path).unwrap();
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
        Commands::Run { module } => {
            let wasm_module_pah = Path::new(module.as_str());
            match wasm_host::run_plugin(wasm_module_pah.to_path_buf()) {
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
