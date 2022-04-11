use thiserror::Error;

pub type Result<T> = std::result::Result<T, KrewWapcError>;

#[derive(Error, Debug)]
pub enum KrewWapcError {
    #[error("Plugin exited with code {code:?}")]
    PlugingExitError { code: i32 },

    #[error("wasm evaluation error: {0}")]
    GenericWasmEvalError(String),

    #[error("{0}")]
    GenericError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
