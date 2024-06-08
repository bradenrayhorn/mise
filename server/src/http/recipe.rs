use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::{
    core::{self, Error},
    domain::{self, CreatingRecipe, UpdatingRecipe},
};

use super::{
    responses,
    server::{AppState, AuthenticatedUser},
};

#[derive(Deserialize)]
pub struct CreateParams {
    title: String,
    ingredients: String,
    instructions: String,
    notes: Option<String>,
    tag_ids: Vec<domain::tag::Id>,
}

#[derive(Serialize)]
pub struct Recipe {
    id: String,
    hash: String,
    title: String,
    ingredient_blocks: Vec<Ingredients>,
    instruction_blocks: Vec<Instructions>,
    notes: Option<String>,
    tags: Vec<String>,
}

#[derive(Serialize)]
pub struct Listed {
    id: String,
    title: String,
}

#[derive(Serialize)]
pub struct Page {
    data: Vec<Listed>,
    next: Option<String>,
}

#[derive(Serialize)]
pub struct Instructions {
    title: Option<String>,
    instructions: Vec<String>,
}

#[derive(Serialize)]
pub struct Ingredients {
    title: Option<String>,
    ingredients: Vec<String>,
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateParams>,
) -> Result<axum::response::Json<responses::Data<String>>, Error> {
    let creating_recipe = CreatingRecipe {
        title: request.title.try_into()?,
        ingredients: request.ingredients.try_into()?,
        instructions: request.instructions.try_into()?,
        notes: match request.notes {
            None => None,
            Some(n) => Some(n.try_into()?),
        },
        tag_ids: request.tag_ids,
    };

    let id = core::recipe::create(&state.datasource, user.into(), creating_recipe).await?;

    Ok(axum::response::Json(responses::Data { data: id.into() }))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<domain::recipe::Id>,
) -> Result<axum::response::Json<responses::Data<Recipe>>, Error> {
    let recipe = core::recipe::get(&state.datasource, id).await?;

    Ok(axum::response::Json(responses::Data {
        data: Recipe {
            id: recipe.id.to_string(),
            hash: recipe.hash,
            title: recipe.title.into(),
            ingredient_blocks: recipe
                .ingredients
                .blocks()
                .iter()
                .map(|block| Ingredients {
                    title: block.title().map(ToOwned::to_owned),
                    ingredients: block
                        .ingredients()
                        .to_vec()
                        .iter()
                        .map(ToOwned::to_owned)
                        .collect(),
                })
                .collect(),
            instruction_blocks: recipe
                .instructions
                .blocks()
                .iter()
                .map(|block| Instructions {
                    title: block.title().map(ToOwned::to_owned),
                    instructions: block
                        .instructions()
                        .to_vec()
                        .iter()
                        .map(ToOwned::to_owned)
                        .collect(),
                })
                .collect(),
            notes: recipe.notes.map(Into::into),
            tags: recipe.tags.into_iter().map(|tag| tag.name.into()).collect(),
        },
    }))
}

#[derive(Deserialize)]
pub struct ListParams {
    next: Option<String>,
    title: Option<String>,
    tag_ids: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<axum::response::Json<Page>, Error> {
    let base64_engine = base64::engine::general_purpose::URL_SAFE;

    let filter = domain::filter::Recipe {
        name: params.title,
        tag_ids: match params.tag_ids {
            None => vec![],
            Some(tag_ids) => tag_ids
                .split(',')
                .filter_map(|tag_id| domain::tag::Id::try_from(tag_id).ok())
                .collect(),
        },
    };
    let cursor = match params.next {
        None => None,
        Some(encoded) => {
            let decoded = base64_engine.decode(encoded)?;
            let deserialized: domain::page::cursor::Recipe = postcard::from_bytes(&decoded)?;
            Some(deserialized)
        }
    };

    let page = core::recipe::list(&state.datasource, filter, cursor).await?;

    Ok(axum::response::Json(Page {
        data: page
            .items
            .into_iter()
            .map(|item| Listed {
                id: item.id.to_string(),
                title: item.title.into(),
            })
            .collect(),
        next: match page.next {
            None => None,
            Some(next) => Some(base64_engine.encode(postcard::to_allocvec(&next)?)),
        },
    }))
}

#[derive(Deserialize)]
pub struct UpdateParams {
    previous_hash: String,
    title: String,
    ingredients: String,
    instructions: String,
    notes: Option<String>,
    tag_ids: Vec<domain::tag::Id>,
}

pub async fn update(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<domain::recipe::Id>,
    Json(request): Json<UpdateParams>,
) -> Result<(), Error> {
    let updating_recipe = UpdatingRecipe {
        id,
        previous_hash: request.previous_hash,
        title: request.title.try_into()?,
        ingredients: request.ingredients.try_into()?,
        instructions: request.instructions.try_into()?,
        notes: match request.notes {
            None => None,
            Some(n) => Some(n.try_into()?),
        },
        tag_ids: request.tag_ids,
    };
    core::recipe::update(&state.datasource, user.into(), updating_recipe).await?;

    Ok(())
}
