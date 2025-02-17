use anyhow::{anyhow, Context};
use axum::{
    extract::{Multipart, Path, State},
    http::HeaderMap,
    response::IntoResponse,
};
use reqwest::StatusCode;

use super::{responses, server::AppState};
use crate::core::{self, Error};

const CACHE_CONTROL: &str = "private, immutable, max-age=31536000";

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

        let id = core::image::upload(
            &state.datasource,
            &state.image_store,
            &state.image_processor,
            bytes.to_vec(),
        )
        .await?;

        Ok(axum::response::Json(responses::Data { data: id.into() }))
    } else {
        Err(Error::Invalid(anyhow!("No file found.")))
    }
}

pub async fn get(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::response::Response, Error> {
    // verify image exists first
    core::image::exists(&state.datasource, &id).await?;

    let etag = format!(r#""{id}""#);

    if let Some(if_none_match) = headers.get("if-none-match") {
        if let Ok(if_none_match) = if_none_match.to_str() {
            if if_none_match == etag {
                return Ok((
                    StatusCode::NOT_MODIFIED,
                    [("Cache-Control", CACHE_CONTROL), ("ETag", &etag)],
                )
                    .into_response());
            }
        }
    }

    let img = core::image::get(&state.image_store, &id).await?;

    Ok((
        StatusCode::OK,
        [
            ("Content-Type", "image/jpeg"),
            ("ETag", &etag),
            ("Cache-Control", CACHE_CONTROL),
        ],
        img,
    )
        .into_response())
}
