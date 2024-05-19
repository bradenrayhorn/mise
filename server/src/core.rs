use crate::cache;

pub mod session;
pub mod user;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{self}")]
    NotFound(String),

    #[error("Unauthenticated")]
    Unauthenticated(#[source] anyhow::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<cache::Error> for Error {
    fn from(value: cache::Error) -> Self {
        Error::Other(value.into())
    }
}
