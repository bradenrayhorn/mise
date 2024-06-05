use crate::{domain, session_store};

pub mod recipe;
pub mod session;
pub mod tag;
pub mod user;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{self}")]
    NotFound(String),

    #[error("Unauthenticated")]
    Unauthenticated(#[source] anyhow::Error),

    #[error(transparent)]
    DomainValidation(#[from] domain::ValidationError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<session_store::Error> for Error {
    fn from(value: session_store::Error) -> Self {
        Error::Other(value.into())
    }
}
