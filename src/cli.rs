use clap::{Parser, Subcommand};

pub const BINARY_NAME: &str = "krew-wasm";

/// The `krew-wasm` binary can be invoked in two "modes"
/// * Native: the user invokes this binary by using an executable named `krew-wasm`
/// * Wrapper: the user invokes this binary by using an executable that is not
///   named `krew-wasm`. This is what happens when `kubectl <name of the plugin>`
///   is executed.

/// This is holds the flags that are shared between the "native" and "wrapper"
/// invocation modes. We cannot just use a single struct to handle cli args:
/// the `krew-wasm` binary would then error out when invoked in `wrapper` mode
/// when no params are specified or when the plugin itself has subcommands.
#[derive(Parser, Debug)]
#[clap(
    name = BINARY_NAME,
    author,
    version,
    about,
    long_about = None,
)]
pub(crate) struct Shared {
    /// Enable verbose mode
    #[clap(short, long, env = "KREW_WASM_VERBOSE")]
    pub verbose: bool,
}

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
    pub command: Commands,

    /// Enable verbose mode
    #[clap(short, long, env = "KREW_WASM_VERBOSE")]
    pub verbose: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Run
    #[clap(arg_required_else_help = true)]
    Run {
        /// Path to the WebAssembly module to execute
        module: String,
    },
    // TODO: add a pull command
    // TODO: add a rm command
}
