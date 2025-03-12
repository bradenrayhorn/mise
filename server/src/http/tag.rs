use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};

use crate::{
    core::{self, Error},
    domain,
};

use super::{
    responses,
    server::{AppState, AuthenticatedUser},
};

#[derive(Serialize)]
pub struct Tag {
    id: domain::tag::Id,
    name: String,
}

#[derive(Serialize)]
pub struct TagWithStats {
    id: domain::tag::Id,
    name: String,
    recipe_count: u64,
}

#[derive(Deserialize)]
pub struct CreateParams {
    name: String,
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateParams>,
) -> Result<axum::response::Json<responses::Data<domain::tag::Id>>, Error> {
    let creating = domain::tag::Creating {
        name: request.name.try_into()?,
    };

    let id = core::tag::create(&state.datasource, user.into(), creating).await?;

    Ok(axum::response::Json(responses::Data { data: id }))
}

pub async fn get_all(
    State(state): State<AppState>,
) -> Result<axum::response::Json<responses::Data<Vec<Tag>>>, Error> {
    let result = core::tag::get_all(&state.datasource).await?;

    Ok(axum::response::Json(responses::Data {
        data: result
            .into_iter()
            .map(|tag| Tag {
                id: tag.id,
                name: tag.name.into(),
            })
            .collect(),
    }))
}

pub async fn get_all_with_stats(
    State(state): State<AppState>,
) -> Result<axum::response::Json<responses::Data<Vec<TagWithStats>>>, Error> {
    let result = core::tag::get_all_with_stats(&state.datasource).await?;

    Ok(axum::response::Json(responses::Data {
        data: result
            .into_iter()
            .map(|tag| TagWithStats {
                id: tag.id,
                name: tag.name.into(),
                recipe_count: tag.recipe_count,
            })
            .collect(),
    }))
}
