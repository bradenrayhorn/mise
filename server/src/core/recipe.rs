use crate::{
    core::Error,
    datastore::{self, Pool, RecipeDocument},
    domain::{self, CreatingRecipe, Recipe, UpdatingRecipe},
};

pub async fn create(
    datastore: &Pool,
    user: domain::user::Authenticated,
    recipe: CreatingRecipe,
) -> Result<uuid::Uuid, Error> {
    let document = RecipeDocument {
        title: recipe.title.into(),
        instructions: recipe.instructions.into(),
        ingredients: recipe.ingredients.into(),
        notes: recipe.notes.map(std::convert::Into::into),
        tag_ids: recipe.tag_ids,
    };

    let id = uuid::Uuid::new_v4();

    datastore
        .create_recipe(id.to_string(), user.id, document)
        .await
        .map_err(|err| Error::Other(err.into()))?;

    Ok(id)
}

pub async fn update(
    datastore: &Pool,
    user: domain::user::Authenticated,
    recipe: UpdatingRecipe,
) -> Result<(), Error> {
    let document = RecipeDocument {
        title: recipe.title.into(),
        instructions: recipe.instructions.into(),
        ingredients: recipe.ingredients.into(),
        notes: recipe.notes.map(std::convert::Into::into),
        tag_ids: recipe.tag_ids,
    };
    let id = recipe.id.to_string();

    datastore
        .update_recipe(id, user.id, document, recipe.previous_hash)
        .await
        .map_err(|err| match err {
            datastore::Error::NotFound => {
                Error::NotFound(format!("recipe {} does not exist", recipe.id))
            }
            _ => Error::Other(err.into()),
        })?;

    Ok(())
}

pub async fn get(datastore: &Pool, id: uuid::Uuid) -> Result<Recipe, Error> {
    datastore
        .get_recipe(id.to_string())
        .await
        .map_err(|err| match err {
            datastore::Error::NotFound => Error::NotFound(format!("recipe {id} does not exist")),
            _ => Error::Other(err.into()),
        })
}
