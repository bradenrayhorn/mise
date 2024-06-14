use crate::{
    core::Error,
    datastore::{self, Pool, RecipeDocument},
    domain::{self, CreatingRecipe, Recipe, UpdatingRecipe},
};

pub async fn create(
    datastore: &Pool,
    user: domain::user::Authenticated,
    recipe: CreatingRecipe,
) -> Result<domain::recipe::Id, Error> {
    let document = RecipeDocument {
        title: recipe.title.into(),
        image_id: recipe.image_id,
        instructions: recipe.instructions.into(),
        ingredients: recipe.ingredients.into(),
        notes: recipe.notes.map(std::convert::Into::into),
        tag_ids: recipe.tag_ids,
    };

    let id = domain::recipe::Id::new();

    datastore
        .create_recipe(id.clone().into(), user.id, document)
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
        image_id: recipe.image_id,
        instructions: recipe.instructions.into(),
        ingredients: recipe.ingredients.into(),
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
    filter: domain::filter::Recipe,
    cursor: Option<domain::page::cursor::Recipe>,
) -> Result<domain::page::Recipe, Error> {
    datastore
        .list_recipes(filter, cursor)
        .await
        .map_err(|err| Error::Other(err.into()))
}
