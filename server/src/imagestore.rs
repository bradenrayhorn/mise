use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Image not found.")]
    NotFound(#[source] anyhow::Error),

    #[error("Image already exists {0}.")]
    DuplicatePath(String),

    #[error("Configuration error")]
    Config(#[source] anyhow::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub struct ImageStore {
    backend: Box<dyn ImageBackend + Send + Sync>,
}

#[async_trait]
pub trait ImageBackend {
    async fn get(&self, path: &str) -> Result<Vec<u8>, Error>;
    async fn upload(&self, path: &str, file: Vec<u8>) -> Result<(), Error>;
}

impl ImageStore {
    #[must_use]
    pub fn new(backend: Box<dyn ImageBackend + Send + Sync>) -> Self {
        ImageStore { backend }
    }

    pub async fn upload(&self, path: &str, file: Vec<u8>) -> Result<(), Error> {
        self.backend.upload(path, file).await
    }

    pub async fn get(&self, path: &str) -> Result<Vec<u8>, Error> {
        self.backend.get(path).await
    }
}
