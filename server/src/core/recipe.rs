use crate::{
    core::Error,
    datastore::{self, Pool, RecipeDocument},
    domain::{self, CreatingRecipe, Recipe, UpdatingRecipe},
};

fn validation_to_other(err: domain::ValidationError) -> Error {
    Error::Other(err.into())
}

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
    let hashed_document = datastore
        .get_recipe(id.to_string())
        .await
        .map_err(|err| match err {
            datastore::Error::NotFound => Error::NotFound(format!("recipe {id} does not exist")),
            _ => Error::Other(err.into()),
        })?;
    let document = hashed_document.document;

    Ok(Recipe {
        id,
        hash: hashed_document.hash,
        title: document.title.try_into().map_err(validation_to_other)?,
        ingredients: document
            .ingredients
            .try_into()
            .map_err(validation_to_other)?,
        instructions: document
            .instructions
            .try_into()
            .map_err(validation_to_other)?,
        notes: match document.notes {
            None => None,
            Some(s) => Some(s.try_into().map_err(validation_to_other)?),
        },
    })
}
