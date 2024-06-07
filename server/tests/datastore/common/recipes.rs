use anyhow::Result;
use mise::{
    datastore::{self, RecipeDocument},
    domain::{self, RegisteringUser, User},
};

#[macro_export]
macro_rules! recipes_tests {
    ($cd:expr) => {
        mod recipes {
            use crate::a_test;
            use crate::datastore::common::{recipes, CreatesDatastore, HoldsDatastore};
            use anyhow::Result;

            a_test!($cd, recipes, can_create_and_get);
            a_test!($cd, recipes, create_creates_initial_revision);
            a_test!($cd, recipes, cannot_create_duplicate);

            a_test!($cd, recipes, cannot_get_non_existent_recipe);

            a_test!($cd, recipes, can_update_recipe);
            a_test!($cd, recipes, cannot_update_with_bad_hash);
            a_test!($cd, recipes, update_creates_new_revision);
            a_test!($cd, recipes, handles_updating_unknown_recipe);

            a_test!($cd, recipes, cannot_get_non_existent_revision);
            a_test!($cd, recipes, cannot_get_revision_for_non_existent_recipe);
            a_test!($cd, recipes, stores_revision_history);

            a_test!($cd, recipes, cannot_get_revisions_for_non_existent_recipe);
        }
    };
}

async fn user(store: &datastore::Pool) -> Result<User> {
    Ok(store
        .upsert_user_by_oauth_id(RegisteringUser {
            potential_id: "user-id".into(),
            oauth_id: "custom|user-1".into(),
            name: "user".into(),
        })
        .await?)
}

async fn tag(store: &datastore::Pool, user_id: &str, name: &str) -> Result<i64> {
    Ok(store
        .create_tag(user_id.to_owned(), name.to_owned())
        .await?)
}

#[derive(Debug, PartialEq, Eq)]
struct ComparableRecipe {
    id: uuid::Uuid,
    title: String,
    ingredients: String,
    instructions: String,
    notes: Option<String>,
    tags: Vec<String>,
}

impl From<domain::Recipe> for ComparableRecipe {
    fn from(value: domain::Recipe) -> Self {
        ComparableRecipe {
            id: value.id,
            title: value.title.into(),
            ingredients: value.ingredients.into(),
            instructions: value.instructions.into(),
            notes: value.notes.map(|n| n.into()),
            tags: value.tags.into_iter().map(|t| t.name.into()).collect(),
        }
    }
}

// create_recipe

pub async fn can_create_and_get(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    let tag_main_id = tag(&store, &user.id, "Main Dish").await?;
    let tag_yummy_id = tag(&store, &user.id, "Yummy").await?;

    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: Some("Don't burn it!".into()),
        tag_ids: vec![tag_main_id, tag_yummy_id],
    };

    let id = uuid::Uuid::new_v4();
    store.create_recipe(id.into(), user.id, recipe).await?;

    let result: ComparableRecipe = store.get_recipe(id.into()).await?.into();

    assert_eq!(id, result.id);
    assert_eq!("Chicken Casserole", result.title);
    assert_eq!("- chicken", result.ingredients);
    assert_eq!("- Cook chicken", result.instructions);
    assert_eq!(Some("Don't burn it!".to_owned()), result.notes);
    assert_eq!(vec!["Main Dish", "Yummy"], result.tags);

    Ok(())
}

pub async fn create_creates_initial_revision(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    let tag_id = tag(&store, &user.id, "Main Dish").await?;

    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: None,
        tag_ids: vec![tag_id],
    };

    let id = uuid::Uuid::new_v4();
    store.create_recipe(id.into(), user.id, recipe).await?;

    // get revisions - there should be one revision: revision 0
    let revisions = store.get_recipe_revisions(id.into()).await?;
    assert_eq!(1, revisions.len());
    assert_eq!(0, revisions[0].revision);

    // try to get revision 0
    let result: ComparableRecipe = store.get_recipe_revision(id.into(), 0).await?.into();

    assert_eq!(id, result.id);
    assert_eq!("Chicken Casserole", result.title);
    assert_eq!("- chicken", result.ingredients);
    assert_eq!("- Cook chicken", result.instructions);
    assert_eq!(None, result.notes);
    assert_eq!(vec!["Main Dish"], result.tags);

    Ok(())
}

pub async fn cannot_create_duplicate(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: None,
        tag_ids: vec![],
    };

    let id = uuid::Uuid::new_v4();
    store
        .create_recipe(id.into(), user.id.clone(), recipe.clone())
        .await?;

    let result = store.create_recipe(id.into(), user.id, recipe).await;

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
    let user = user(&store).await?;
    let tag_main_id = tag(&store, &user.id, "Main Dish").await?;
    let tag_soup_id = tag(&store, &user.id, "Soup").await?;

    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: Some("Don't burn it!".into()),
        tag_ids: vec![tag_main_id],
    };

    let id = uuid::Uuid::new_v4();
    store
        .create_recipe(id.into(), user.id.clone(), recipe)
        .await?;

    // get current hash
    let current_hash = store.get_recipe(id.into()).await?.hash;

    // update document
    store
        .update_recipe(
            id.into(),
            user.id,
            RecipeDocument {
                title: "Bean Soup".into(),
                ingredients: "- beans".into(),
                instructions: "- Cook beans".into(),
                notes: None,
                tag_ids: vec![tag_soup_id],
            },
            current_hash,
        )
        .await?;

    let result: ComparableRecipe = store.get_recipe(id.into()).await?.into();

    assert_eq!(id, result.id);
    assert_eq!("Bean Soup", result.title);
    assert_eq!("- beans", result.ingredients);
    assert_eq!("- Cook beans", result.instructions);
    assert_eq!(None, result.notes);
    assert_eq!(vec!["Soup"], result.tags);

    Ok(())
}

pub async fn cannot_update_with_bad_hash(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: Some("Don't burn it!".into()),
        tag_ids: vec![],
    };

    store
        .create_recipe("1234".into(), user.id.clone(), recipe)
        .await?;

    // update document
    let result = store
        .update_recipe(
            "1234".into(),
            user.id.clone(),
            RecipeDocument {
                title: "Bean Soup".into(),
                ingredients: "- beans".into(),
                instructions: "- Cook beans".into(),
                notes: None,
                tag_ids: vec![],
            },
            "this is the wrong hash".into(),
        )
        .await;

    if let Err(datastore::Error::Conflict) = result {
    } else {
        panic!("update_recipe returned {:?}, expected Conflict", result);
    }

    Ok(())
}

pub async fn update_creates_new_revision(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: Some("Don't burn it!".into()),
        tag_ids: vec![],
    };

    let id = uuid::Uuid::new_v4();
    store
        .create_recipe(id.into(), user.id.clone(), recipe)
        .await?;

    // get current hash
    let current_hash = store.get_recipe(id.into()).await?.hash;

    // update document
    store
        .update_recipe(
            id.into(),
            user.id.clone(),
            RecipeDocument {
                title: "Bean Soup".into(),
                ingredients: "- beans".into(),
                instructions: "- Cook beans".into(),
                notes: None,
                tag_ids: vec![],
            },
            current_hash,
        )
        .await?;

    // get revisions - there should be two revision
    let revisions = store.get_recipe_revisions(id.into()).await?;
    assert_eq!(2, revisions.len());
    assert_eq!(1, revisions[0].revision);
    assert_eq!(0, revisions[1].revision);

    // try to get revision 1
    let result: ComparableRecipe = store.get_recipe_revision(id.into(), 1).await?.into();

    assert_eq!(id, result.id);
    assert_eq!("Bean Soup", result.title);
    assert_eq!("- beans", result.ingredients);
    assert_eq!("- Cook beans", result.instructions);
    assert_eq!(None, result.notes);
    assert!(result.tags.is_empty());

    Ok(())
}

pub async fn handles_updating_unknown_recipe(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    let id = uuid::Uuid::new_v4();
    let result = store
        .update_recipe(
            id.into(),
            user.id,
            RecipeDocument {
                title: "Bean Soup".into(),
                ingredients: "- chicken".into(),
                instructions: "- Cook chicken".into(),
                notes: None,
                tag_ids: vec![],
            },
            "".into(),
        )
        .await;

    if let Err(datastore::Error::NotFound) = result {
    } else {
        panic!("update_recipe returned {:?}, expected NotFound", result);
    }

    Ok(())
}

// get_revision

pub async fn cannot_get_non_existent_revision(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    // save recipe
    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: Some("Don't burn it!".into()),
        tag_ids: vec![],
    };

    let id = uuid::Uuid::new_v4();
    store.create_recipe(id.into(), user.id, recipe).await?;

    // try to get a non existent revision
    let result = store.get_recipe_revision(id.into(), 99).await;

    if let Err(datastore::Error::NotFound) = result {
    } else {
        panic!(
            "get_recipe_revision returned {:?}, expected NotFound",
            result
        );
    }

    Ok(())
}

pub async fn cannot_get_revision_for_non_existent_recipe(store: datastore::Pool) -> Result<()> {
    let result = store.get_recipe_revision("bad-id".into(), 0).await;

    if let Err(datastore::Error::NotFound) = result {
    } else {
        panic!(
            "get_recipe_revision returned {:?}, expected NotFound",
            result
        );
    }

    Ok(())
}

pub async fn stores_revision_history(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    let id = uuid::Uuid::new_v4();
    // create recipe
    let recipe = RecipeDocument {
        title: "one".into(),
        ingredients: "- two".into(),
        instructions: "- three".into(),
        notes: Some("four".into()),
        tag_ids: vec![tag(&store, &user.id, "Tag1").await?],
    };
    store
        .create_recipe(id.into(), user.id.clone(), recipe)
        .await?;

    // update recipe
    let recipe = RecipeDocument {
        title: "five".into(),
        ingredients: "- six".into(),
        instructions: "- seven".into(),
        notes: Some("eight".into()),
        tag_ids: vec![tag(&store, &user.id, "Tag2").await?],
    };
    let hash = store.get_recipe(id.into()).await?.hash;
    store
        .update_recipe(id.into(), user.id.clone(), recipe, hash)
        .await?;

    // update recipe again
    let recipe = RecipeDocument {
        title: "nine".into(),
        ingredients: "- ten".into(),
        instructions: "- eleven".into(),
        notes: Some("twelve".into()),
        tag_ids: vec![tag(&store, &user.id, "Tag3").await?],
    };
    let hash = store.get_recipe(id.into()).await?.hash;
    store
        .update_recipe(id.into(), user.id.clone(), recipe, hash)
        .await?;

    // now validate revision history - there should be three revisions
    let revisions = store.get_recipe_revisions(id.into()).await?;
    assert_eq!(3, revisions.len());
    assert_eq!(2, revisions[0].revision);
    assert_eq!(1, revisions[1].revision);
    assert_eq!(0, revisions[2].revision);

    // check revision 0
    let result: ComparableRecipe = store.get_recipe_revision(id.into(), 0).await?.into();
    assert_eq!("one", result.title);
    assert_eq!("- two", result.ingredients);
    assert_eq!("- three", result.instructions);
    assert_eq!(Some("four".into()), result.notes);
    assert_eq!(vec!["Tag1"], result.tags);

    // check revision 1
    let result: ComparableRecipe = store.get_recipe_revision(id.into(), 1).await?.into();
    assert_eq!("five", result.title);
    assert_eq!("- six", result.ingredients);
    assert_eq!("- seven", result.instructions);
    assert_eq!(Some("eight".into()), result.notes);
    assert_eq!(vec!["Tag2"], result.tags);

    // check revision 2
    let result: ComparableRecipe = store.get_recipe_revision(id.into(), 2).await?.into();
    assert_eq!("nine", result.title);
    assert_eq!("- ten", result.ingredients);
    assert_eq!("- eleven", result.instructions);
    assert_eq!(Some("twelve".into()), result.notes);
    assert_eq!(vec!["Tag3"], result.tags);

    Ok(())
}

// get_revisions

pub async fn cannot_get_revisions_for_non_existent_recipe(store: datastore::Pool) -> Result<()> {
    let result = store.get_recipe_revisions("a-random-id".into()).await?;
    assert_eq!(0, result.len());

    Ok(())
}
