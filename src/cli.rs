use clap::{Parser, Subcommand};

pub const BINARY_NAME: &str = "krew-wasm";
pub const KREW_WASM_VERBOSE_ENV: &str = "KREW_WASM_VERBOSE";

#[derive(Parser, Debug)]
#[clap(
    name = BINARY_NAME,
    author,
    version,
    about,
    long_about = None,
)]
pub(crate) struct Native {
    #[clap(subcommand)]
    pub command: NativeCommands,

    /// Enable verbose mode
    #[clap(short, long, env = KREW_WASM_VERBOSE_ENV)]
    pub verbose: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum NativeCommands {
    /// Run
    #[clap(arg_required_else_help = true)]
    Run {
        /// Path to the WebAssembly module to execute
        module: String,

        #[clap(last = true)]
        wasm_args: Vec<String>,
    },
    // TODO: add a pull command
    // TODO: add a rm command
}
