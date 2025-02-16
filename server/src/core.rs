use crate::{domain, image_processing, session_store};

pub mod image;
pub mod recipe;
pub mod session;
pub mod tag;
pub mod user;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    NotFound(String),

    #[error("Unauthenticated")]
    Unauthenticated(#[source] anyhow::Error),

    #[error("Not a valid request.")]
    Invalid(#[source] anyhow::Error),

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

impl From<postcard::Error> for Error {
    fn from(value: postcard::Error) -> Self {
        Error::Other(value.into())
    }
}

impl From<base64::DecodeError> for Error {
    fn from(value: base64::DecodeError) -> Self {
        Error::Other(value.into())
    }
}

impl From<image_processing::Error> for Error {
    fn from(value: image_processing::Error) -> Self {
        Error::Other(value.into())
    }
}
