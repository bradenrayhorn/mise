use anyhow::Result;
use mise::{datastore, domain::RecipeDocument};

#[macro_export]
macro_rules! recipes_tests {
    ($cd:expr) => {
        mod recipes {
            use crate::a_test;
            use crate::datastore::common::{recipes, CreatesDatastore, HoldsDatastore};
            use anyhow::Result;

            a_test!($cd, recipes, can_create_and_get);
            a_test!($cd, recipes, cannot_create_duplicate);
            a_test!($cd, recipes, cannot_get_non_existent_recipe);
            a_test!($cd, recipes, can_update_recipe);
            a_test!($cd, recipes, handles_updating_unknown_recipe);
        }
    };
}

// create_recipe

pub async fn can_create_and_get(store: datastore::Pool) -> Result<()> {
    let recipe = RecipeDocument {
        id: "1234".into(),
        title: "Chicken Casserole".into(),
        document: "# Chicken Casserole".into(),
    };

    store.create_recipe(recipe).await?;

    let result = store.get_recipe("1234".into()).await?;

    assert_eq!("1234", result.id);
    assert_eq!("Chicken Casserole", result.title);
    assert_eq!("# Chicken Casserole", result.document);

    Ok(())
}

pub async fn cannot_create_duplicate(store: datastore::Pool) -> Result<()> {
    let recipe = RecipeDocument {
        id: "1234".into(),
        title: "Chicken Casserole".into(),
        document: "# Chicken Casserole".into(),
    };

    store.create_recipe(recipe.clone()).await?;

    let result = store.create_recipe(recipe).await;

    assert_eq!(true, result.is_err());

    Ok(())
}

// get_recipe

pub async fn cannot_get_non_existent_recipe(store: datastore::Pool) -> Result<()> {
    let r = store.get_recipe("a-random-id".into()).await;

    if let Err(datastore::Error::NotFound) = r {
    } else {
        panic!("get_recipe returned {:?}, expected NotFound", r);
    }

    Ok(())
}

// update_recipe

pub async fn can_update_recipe(store: datastore::Pool) -> Result<()> {
    let recipe = RecipeDocument {
        id: "1234".into(),
        title: "Chicken Casserole".into(),
        document: "# Chicken Casserole".into(),
    };

    store.create_recipe(recipe).await?;

    // update title and document
    store
        .update_recipe(RecipeDocument {
            id: "1234".into(),
            title: "Bean Soup".into(),
            document: "# Bean Soup".into(),
        })
        .await?;

    let result = store.get_recipe("1234".into()).await?;

    assert_eq!("1234", result.id);
    assert_eq!("Bean Soup", result.title);
    assert_eq!("# Bean Soup", result.document);

    Ok(())
}

pub async fn handles_updating_unknown_recipe(store: datastore::Pool) -> Result<()> {
    let result = store
        .update_recipe(RecipeDocument {
            id: "a-random-id".into(),
            title: "Bean Soup".into(),
            document: "# Bean Soup".into(),
        })
        .await;

    if let Ok(_) = result {
    } else {
        panic!("update_recipe was not ok: {:?}", result);
    }

    Ok(())
}
