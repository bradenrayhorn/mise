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
