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

// create_recipe

pub async fn can_create_and_get(store: datastore::Pool) -> Result<()> {
    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: Some("Don't burn it!".into()),
    };

    store.create_recipe("1234".into(), recipe).await?;

    let result = store.get_recipe("1234".into()).await?.document;

    assert_eq!("Chicken Casserole", result.title);
    assert_eq!("- chicken", result.ingredients);
    assert_eq!("- Cook chicken", result.instructions);
    assert_eq!(Some("Don't burn it!".to_owned()), result.notes);

    Ok(())
}

pub async fn create_creates_initial_revision(store: datastore::Pool) -> Result<()> {
    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: None,
    };

    store.create_recipe("1234".into(), recipe).await?;

    // get revisions - there should be one revision: revision 0
    let revisions = store.get_recipe_revisions("1234".into()).await?;
    assert_eq!(1, revisions.len());
    assert_eq!(0, revisions[0].revision);

    // try to get revision 0
    let result = store.get_recipe_revision("1234".into(), 0).await?.document;

    assert_eq!("Chicken Casserole", result.title);
    assert_eq!("- chicken", result.ingredients);
    assert_eq!("- Cook chicken", result.instructions);
    assert_eq!(None, result.notes);

    Ok(())
}

pub async fn cannot_create_duplicate(store: datastore::Pool) -> Result<()> {
    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: None,
    };

    store.create_recipe("1234".into(), recipe.clone()).await?;

    let result = store.create_recipe("1234".into(), recipe).await;

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
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: Some("Don't burn it!".into()),
    };

    store.create_recipe("1234".into(), recipe).await?;

    // get current hash
    let current_hash = store.get_recipe("1234".into()).await?.hash;

    // update document
    store
        .update_recipe(
            "1234".into(),
            RecipeDocument {
                title: "Bean Soup".into(),
                ingredients: "- beans".into(),
                instructions: "- Cook beans".into(),
                notes: None,
            },
            current_hash,
        )
        .await?;

    let result = store.get_recipe("1234".into()).await?.document;

    assert_eq!("Bean Soup", result.title);
    assert_eq!("- beans", result.ingredients);
    assert_eq!("- Cook beans", result.instructions);
    assert_eq!(None, result.notes);

    Ok(())
}

pub async fn cannot_update_with_bad_hash(store: datastore::Pool) -> Result<()> {
    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: Some("Don't burn it!".into()),
    };

    store.create_recipe("1234".into(), recipe).await?;

    // update document
    let result = store
        .update_recipe(
            "1234".into(),
            RecipeDocument {
                title: "Bean Soup".into(),
                ingredients: "- beans".into(),
                instructions: "- Cook beans".into(),
                notes: None,
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
    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: Some("Don't burn it!".into()),
    };

    store.create_recipe("1234".into(), recipe).await?;

    // get current hash
    let current_hash = store.get_recipe("1234".into()).await?.hash;

    // update document
    store
        .update_recipe(
            "1234".into(),
            RecipeDocument {
                title: "Bean Soup".into(),
                ingredients: "- beans".into(),
                instructions: "- Cook beans".into(),
                notes: None,
            },
            current_hash,
        )
        .await?;

    // get revisions - there should be two revision
    let revisions = store.get_recipe_revisions("1234".into()).await?;
    assert_eq!(2, revisions.len());
    assert_eq!(1, revisions[0].revision);
    assert_eq!(0, revisions[1].revision);

    // try to get revision 1
    let result = store.get_recipe_revision("1234".into(), 1).await?.document;

    assert_eq!("Bean Soup", result.title);
    assert_eq!("- beans", result.ingredients);
    assert_eq!("- Cook beans", result.instructions);
    assert_eq!(None, result.notes);

    Ok(())
}

pub async fn handles_updating_unknown_recipe(store: datastore::Pool) -> Result<()> {
    let result = store
        .update_recipe(
            "a-random-id".into(),
            RecipeDocument {
                title: "Bean Soup".into(),
                ingredients: "- chicken".into(),
                instructions: "- Cook chicken".into(),
                notes: None,
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
    // save recipe
    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        ingredients: "- chicken".into(),
        instructions: "- Cook chicken".into(),
        notes: Some("Don't burn it!".into()),
    };

    store.create_recipe("1234".into(), recipe).await?;

    // try to get a non existent revision
    let result = store.get_recipe_revision("bad-id".into(), 99).await;

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
    // create recipe
    let recipe = RecipeDocument {
        title: "one".into(),
        ingredients: "two".into(),
        instructions: "three".into(),
        notes: Some("four".into()),
    };
    store.create_recipe("1234".into(), recipe).await?;

    // update recipe
    let recipe = RecipeDocument {
        title: "five".into(),
        ingredients: "six".into(),
        instructions: "seven".into(),
        notes: Some("eight".into()),
    };
    let hash = store.get_recipe("1234".into()).await?.hash;
    store.update_recipe("1234".into(), recipe, hash).await?;

    // update recipe again
    let recipe = RecipeDocument {
        title: "nine".into(),
        ingredients: "ten".into(),
        instructions: "eleven".into(),
        notes: Some("twelve".into()),
    };
    let hash = store.get_recipe("1234".into()).await?.hash;
    store.update_recipe("1234".into(), recipe, hash).await?;

    // now validate revision history - there should be three revisions
    let revisions = store.get_recipe_revisions("1234".into()).await?;
    assert_eq!(3, revisions.len());
    assert_eq!(2, revisions[0].revision);
    assert_eq!(1, revisions[1].revision);
    assert_eq!(0, revisions[2].revision);

    // check revision 0
    let result = store.get_recipe_revision("1234".into(), 0).await?.document;
    assert_eq!("one", result.title);
    assert_eq!("two", result.ingredients);
    assert_eq!("three", result.instructions);
    assert_eq!(Some("four".into()), result.notes);

    // check revision 1
    let result = store.get_recipe_revision("1234".into(), 1).await?.document;
    assert_eq!("five", result.title);
    assert_eq!("six", result.ingredients);
    assert_eq!("seven", result.instructions);
    assert_eq!(Some("eight".into()), result.notes);

    // check revision 2
    let result = store.get_recipe_revision("1234".into(), 2).await?.document;
    assert_eq!("nine", result.title);
    assert_eq!("ten", result.ingredients);
    assert_eq!("eleven", result.instructions);
    assert_eq!(Some("twelve".into()), result.notes);

    Ok(())
}

// get_revisions

pub async fn cannot_get_revisions_for_non_existent_recipe(store: datastore::Pool) -> Result<()> {
    let result = store.get_recipe_revisions("a-random-id".into()).await?;
    assert_eq!(0, result.len());

    Ok(())
}
