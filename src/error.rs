use thiserror::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[allow(non_camel_case_types)]
    #[error("{0}")]
    new(String),
    #[error("{0}")]
    Other(String),
}
