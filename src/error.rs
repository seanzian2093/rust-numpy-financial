use thiserror::Error;

// Customized Result
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the rfinancial crate
#[derive(Debug, Error)]
pub enum Error {
    /// Parameter validation error
    #[error("Parameter error: {0}")]
    ParaError(String),

    /// Constructor error
    #[error("Constructor error: {0}")]
    ConstructorError(String),

    /// Other errors
    #[error("Error: {0}")]
    OtherError(String),
}
