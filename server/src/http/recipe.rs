use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    core::{self, Error},
    domain::{CreatingRecipe, UpdatingRecipe},
};

use super::{responses, server::AppState};

#[derive(Deserialize)]
pub struct CreateParams {
    title: String,
    ingredients: String,
    instructions: String,
    notes: Option<String>,
}

#[derive(Serialize)]
pub struct Recipe {
    id: String,
    hash: String,
    title: String,
    ingredient_blocks: Vec<Ingredients>,
    instruction_blocks: Vec<Instructions>,
    notes: Option<String>,
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
    Json(request): Json<CreateParams>,
) -> Result<axum::response::Json<responses::Data<uuid::Uuid>>, Error> {
    let creating_recipe = CreatingRecipe {
        title: request.title.try_into()?,
        ingredients: request.ingredients.try_into()?,
        instructions: request.instructions.try_into()?,
        notes: match request.notes {
            None => None,
            Some(n) => Some(n.try_into()?),
        },
    };

    let id = core::recipe::create(&state.datasource, creating_recipe).await?;

    Ok(axum::response::Json(responses::Data { data: id }))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
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
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
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
    };
    core::recipe::update(&state.datasource, updating_recipe).await?;

    Ok(())
}
