use axum::async_trait;

use crate::imagestore::Error;

pub struct ImageBackend {
    base_path: std::path::PathBuf,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Other(value.into())
    }
}

impl ImageBackend {
    pub async fn new(path: &str) -> Result<Self, Error> {
        if !tokio::fs::try_exists(path).await? {
            tokio::fs::create_dir(path).await?;
        }

        Ok(ImageBackend {
            base_path: std::path::Path::new(path).to_owned(),
        })
    }
}

#[async_trait]
impl crate::imagestore::ImageBackend for ImageBackend {
    async fn get(&self, path: &str) -> Result<Vec<u8>, Error> {
        tokio::fs::read(self.base_path.join(path))
            .await
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => Error::NotFound(err.into()),
                _ => Error::Other(err.into()),
            })
    }

    async fn upload(&self, path: &str, file: Vec<u8>) -> Result<(), Error> {
        let joined_path = self.base_path.join(path);
        if tokio::fs::try_exists(&joined_path).await? {
            return Err(Error::DuplicatePath(path.to_owned()));
        }

        tokio::fs::write(&joined_path, file).await?;

        Ok(())
    }
}
