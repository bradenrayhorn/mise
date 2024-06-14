use anyhow::{anyhow, Context};
use axum::{
    extract::{Multipart, Path, State},
    response::IntoResponse,
};

use super::{responses, server::AppState};
use crate::core::{self, Error};

pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<axum::response::Json<responses::Data<String>>, Error> {
    if let Some(field) = multipart
        .next_field()
        .await
        .context("Could not read multipart upload.")
        .map_err(Error::Invalid)?
    {
        let bytes = field
            .bytes()
            .await
            .context("Could not read multipart bytes.")
            .map_err(Error::Invalid)?;

        let id = core::image::upload(&state.datasource, &state.image_store, bytes.to_vec()).await?;

        Ok(axum::response::Json(responses::Data { data: id.into() }))
    } else {
        Err(Error::Invalid(anyhow!("No file found.")))
    }
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, Error> {
    let img = core::image::get(&state.image_store, &id).await?;

    Ok((
        [
            ("Content-Type", "image/jpeg"),
            ("Cache-Control", "private, immutable, max-age=31536000"),
        ],
        img,
    ))
}
