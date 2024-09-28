use core::fmt;

// Customized Result
pub type Result<T> = std::result::Result<T, Error>;

// Customized Error
#[derive(Debug)]
pub enum Error {
    ParaError(String),
    ConstructorError(String),
    OtherError(String),
}

// Parameter Error
#[derive(Debug)]
pub struct ParaError;

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

impl std::error::Error for Error {}
