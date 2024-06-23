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
    image_id: Option<String>,
    ingredients: Vec<Ingredients>,
    instructions: Vec<Instructions>,
    notes: Option<String>,
    tag_ids: Vec<domain::tag::Id>,
}

#[derive(Serialize)]
pub struct Recipe {
    id: String,
    hash: String,
    title: String,
    image_id: Option<String>,
    ingredient_blocks: Vec<Ingredients>,
    instruction_blocks: Vec<Instructions>,
    notes: Option<String>,
    tags: Vec<AttachedTag>,
}

#[derive(Serialize)]
pub struct AttachedTag {
    id: String,
    name: String,
}

#[derive(Serialize)]
pub struct Listed {
    id: String,
    title: String,
    image_id: Option<String>,
}

#[derive(Serialize)]
pub struct Page {
    data: Vec<Listed>,
    next: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Instructions {
    title: Option<String>,
    instructions: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Ingredients {
    title: Option<String>,
    ingredients: Vec<String>,
}

fn parse_instructions(block: Instructions) -> Result<domain::recipe::InstructionBlock, Error> {
    Ok(domain::recipe::InstructionBlock {
        title: match block.title {
            None => None,
            Some(n) => Some(n.try_into()?),
        },
        instructions: block
            .instructions
            .into_iter()
            .map(domain::recipe::Instruction::try_from)
            .collect::<Result<Vec<domain::recipe::Instruction>, domain::ValidationError>>()?,
    })
}

fn parse_ingredients(block: Ingredients) -> Result<domain::recipe::IngredientBlock, Error> {
    Ok(domain::recipe::IngredientBlock {
        title: match block.title {
            None => None,
            Some(n) => Some(n.try_into()?),
        },
        ingredients: block
            .ingredients
            .into_iter()
            .map(domain::recipe::Ingredient::try_from)
            .collect::<Result<Vec<domain::recipe::Ingredient>, domain::ValidationError>>()?,
    })
}

pub async fn create(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateParams>,
) -> Result<axum::response::Json<responses::Data<String>>, Error> {
    let creating_recipe = CreatingRecipe {
        title: request.title.try_into()?,
        image_id: match request.image_id {
            None => None,
            Some(n) => Some(n.as_str().try_into()?),
        },
        ingredients: request
            .ingredients
            .into_iter()
            .map(parse_ingredients)
            .collect::<Result<Vec<domain::recipe::IngredientBlock>, Error>>()?,
        instructions: request
            .instructions
            .into_iter()
            .map(parse_instructions)
            .collect::<Result<Vec<domain::recipe::InstructionBlock>, Error>>()?,
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
            image_id: recipe.image_id.map(Into::into),
            ingredient_blocks: recipe
                .ingredients
                .into_iter()
                .map(|block| Ingredients {
                    title: block.title.map(String::from),
                    ingredients: block.ingredients.into_iter().map(String::from).collect(),
                })
                .collect(),
            instruction_blocks: recipe
                .instructions
                .into_iter()
                .map(|block| Instructions {
                    title: block.title.map(String::from),
                    instructions: block.instructions.into_iter().map(String::from).collect(),
                })
                .collect(),
            notes: recipe.notes.map(Into::into),
            tags: recipe
                .tags
                .into_iter()
                .map(|tag| AttachedTag {
                    id: tag.id.into(),
                    name: tag.name.into(),
                })
                .collect(),
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
                image_id: item.image_id.map(Into::into),
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
    image_id: Option<String>,
    ingredients: Vec<Ingredients>,
    instructions: Vec<Instructions>,
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
        image_id: match request.image_id {
            None => None,
            Some(n) => Some(n.as_str().try_into()?),
        },
        ingredients: request
            .ingredients
            .into_iter()
            .map(parse_ingredients)
            .collect::<Result<Vec<domain::recipe::IngredientBlock>, Error>>()?,
        instructions: request
            .instructions
            .into_iter()
            .map(parse_instructions)
            .collect::<Result<Vec<domain::recipe::InstructionBlock>, Error>>()?,
        notes: match request.notes {
            None => None,
            Some(n) => Some(n.try_into()?),
        },
        tag_ids: request.tag_ids,
    };
    core::recipe::update(&state.datasource, user.into(), updating_recipe).await?;

    Ok(())
}
