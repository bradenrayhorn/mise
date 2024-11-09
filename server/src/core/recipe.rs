use anyhow::Context;

use crate::{
    core::Error,
    datastore::{self, Pool, RecipeDocument},
    domain::{
        self, recipe::StringifiedBlock, CreatingRecipe, ListedRecipe, Recipe, UpdatingRecipe,
    },
    search::Backend,
};

pub async fn create(
    datastore: &Pool,
    search_backend: &Backend,
    user: domain::user::Authenticated,
    recipe: CreatingRecipe,
) -> Result<domain::recipe::Id, Error> {
    let document = RecipeDocument {
        title: recipe.title.into(),
        image_id: recipe.image_id,
        instructions: recipe
            .instructions
            .into_iter()
            .map(StringifiedBlock::from)
            .collect(),
        ingredients: recipe
            .ingredients
            .into_iter()
            .map(StringifiedBlock::from)
            .collect(),
        notes: recipe.notes.map(std::convert::Into::into),
        tag_ids: recipe.tag_ids,
    };

    let id = domain::recipe::Id::new();

    datastore
        .create_recipe(id.clone().into(), user.id, document)
        .await
        .map_err(|err| Error::Other(err.into()))?;

    search_backend
        .index_recipes()
        .await
        .map_err(|err| Error::Other(err.into()))?;

    Ok(id)
}

pub async fn update(
    datastore: &Pool,
    search_backend: &Backend,
    user: domain::user::Authenticated,
    recipe: UpdatingRecipe,
) -> Result<(), Error> {
    let document = RecipeDocument {
        title: recipe.title.into(),
        image_id: recipe.image_id,
        instructions: recipe
            .instructions
            .into_iter()
            .map(StringifiedBlock::from)
            .collect(),
        ingredients: recipe
            .ingredients
            .into_iter()
            .map(StringifiedBlock::from)
            .collect(),
        notes: recipe.notes.map(std::convert::Into::into),
        tag_ids: recipe.tag_ids,
    };

    datastore
        .update_recipe(
            recipe.id.clone().into(),
            user.id,
            document,
            recipe.previous_hash,
        )
        .await
        .map_err(|err| match err {
            datastore::Error::NotFound => {
                Error::NotFound(format!("recipe {} does not exist", recipe.id))
            }
            _ => Error::Other(err.into()),
        })?;

    search_backend
        .index_recipes()
        .await
        .map_err(|err| Error::Other(err.into()))?;

    Ok(())
}

pub async fn get(datastore: &Pool, id: domain::recipe::Id) -> Result<Recipe, Error> {
    datastore
        .get_recipe(id.clone().into())
        .await
        .map_err(|err| match err {
            datastore::Error::NotFound => Error::NotFound(format!("recipe {id} does not exist")),
            _ => Error::Other(err.into()),
        })
}

pub async fn list(
    datastore: &Pool,
    search_backend: &Backend,
    query: Option<String>,
    filter: domain::filter::Recipe,
    cursor: Option<domain::page::cursor::Recipe>,
) -> Result<domain::page::Recipe, Error> {
    if let Some(search) = query {
        let recipe_ids = search_backend
            .search(search, filter)
            .await
            .map_err(|err| Error::Other(err.into()))?;

        let recipes = futures::future::try_join_all(recipe_ids.into_iter().map(|id| async move {
            let recipe = datastore
                .get_recipe(id.clone().into())
                .await
                .context(format!("get recipe: {id}"))?;

            Ok::<ListedRecipe, Error>(ListedRecipe {
                id: recipe.id,
                title: recipe.title,
                image_id: recipe.image_id,
            })
        }))
        .await
        .context("recipes/list: get searched recipes")?;

        Ok(domain::page::Recipe {
            items: recipes,
            next: None,
        })
    } else {
        datastore
            .list_recipes(filter, cursor)
            .await
            .map_err(|err| Error::Other(err.into()))
    }
}
