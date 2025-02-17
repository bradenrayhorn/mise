use anyhow::{anyhow, Context};

use crate::{
    core::Error,
    datastore::{self, Pool},
    domain,
    image_processing::ImageProcessor,
    imagestore::{self, ImageStore},
};

pub async fn upload(
    datastore: &Pool,
    image_store: &ImageStore,
    image_processor: &ImageProcessor,
    file: Vec<u8>,
) -> Result<domain::image::Id, Error> {
    let id = domain::image::Id::new();

    image_store
        .upload(
            &original_path(String::from(&id).as_ref()),
            image_processor.process_image(file).await?,
        )
        .await
        .context("Could not upload image.")?;

    datastore
        .create_image(&id)
        .await
        .context("Could not persist image.")?;

    Ok(id)
}

pub async fn get(image_store: &ImageStore, image_id: &str) -> Result<Vec<u8>, Error> {
    image_store
        .get(&original_path(image_id))
        .await
        .map_err(|err| match err {
            imagestore::Error::NotFound(_) => Error::NotFound("Image not found.".into()),
            _ => Error::Other(anyhow!(err).context("Could not get image.")),
        })
}

pub async fn exists(datastore: &Pool, image_id: &str) -> Result<(), Error> {
    let id = domain::image::Id::try_from(image_id)?;
    datastore.get_image(&id).await.map_err(|err| match err {
        datastore::Error::NotFound => Error::NotFound("Image not found.".into()),
        _ => Error::Other(anyhow!(err).context("Could not get image.")),
    })?;

    Ok(())
}

fn original_path(id: &str) -> String {
    format!("{id}-original.jpg")
}
