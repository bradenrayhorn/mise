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
            use crate::datastore::common::{CreatesDatastore, HoldsDatastore, recipes};
            use anyhow::Result;

            a_test!($cd, recipes, can_create_and_get);
            a_test!($cd, recipes, create_creates_initial_revision);
            a_test!($cd, recipes, cannot_create_duplicate);

            a_test!($cd, recipes, cannot_get_non_existent_recipe);

            a_test!($cd, recipes, can_list_recipes_over_multiple_pages);
            a_test!($cd, recipes, can_list_with_tag_filter);
            a_test!($cd, recipes, can_list_recipe_if_it_has_tag_with_no_filter);
            a_test!(
                $cd,
                recipes,
                filters_properly_if_has_multiple_tags_matching_count_but_wrong_tags
            );

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

async fn tag(store: &datastore::Pool, user_id: &str, name: &str) -> Result<domain::tag::Id> {
    Ok(store
        .create_tag(user_id.to_owned(), name.to_owned())
        .await?)
}

async fn image(store: &datastore::Pool) -> Result<domain::image::Id> {
    let id = domain::image::Id::new();
    store.create_image(&id).await?;
    Ok(id)
}

#[derive(Debug, PartialEq, Eq)]
struct ComparableRecipe {
    id: domain::recipe::Id,
    title: String,
    image_id: Option<domain::image::Id>,
    ingredients: Vec<(Option<String>, Vec<String>)>,
    instructions: Vec<(Option<String>, Vec<String>)>,
    notes: Option<String>,
    tags: Vec<String>,
}

impl From<domain::Recipe> for ComparableRecipe {
    fn from(value: domain::Recipe) -> Self {
        ComparableRecipe {
            id: value.id,
            title: value.title.into(),
            image_id: value.image_id,
            ingredients: value
                .ingredients
                .into_iter()
                .map(|i| {
                    let block = domain::recipe::StringifiedBlock::from(i);
                    (block.title, block.items)
                })
                .collect(),
            instructions: value
                .instructions
                .into_iter()
                .map(|i| {
                    let block = domain::recipe::StringifiedBlock::from(i);
                    (block.title, block.items)
                })
                .collect(),
            notes: value.notes.map(Into::into),
            tags: value.tags.into_iter().map(|t| t.name.into()).collect(),
        }
    }
}

impl ComparableRecipe {
    fn assert_ingredients(&self, expected: &[(Option<&str>, &[&str])]) {
        let converted: Vec<(Option<String>, Vec<String>)> = expected
            .iter()
            .map(|(t, i)| {
                (
                    t.map(|s| s.to_string()),
                    i.iter().map(|s| s.to_string()).collect(),
                )
            })
            .collect();

        assert_eq!(converted, self.ingredients);
    }

    fn assert_instructions(&self, expected: &[(Option<&str>, &[&str])]) {
        let converted: Vec<(Option<String>, Vec<String>)> = expected
            .iter()
            .map(|(t, i)| {
                (
                    t.map(|s| s.to_string()),
                    i.iter().map(|s| s.to_string()).collect(),
                )
            })
            .collect();

        assert_eq!(converted, self.instructions);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ComparableListedRecipe {
    id: domain::recipe::Id,
    title: String,
    image_id: Option<domain::image::Id>,
}

impl From<domain::ListedRecipe> for ComparableListedRecipe {
    fn from(value: domain::ListedRecipe) -> Self {
        ComparableListedRecipe {
            id: value.id,
            title: value.title.into(),
            image_id: value.image_id,
        }
    }
}

// create_recipe

pub async fn can_create_and_get(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    let tag_main_id = tag(&store, &user.id, "Main Dish").await?;
    let tag_yummy_id = tag(&store, &user.id, "Yummy").await?;

    let image_id = image(&store).await?;

    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        image_id: Some(image_id.clone()),
        ingredients: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["Chicken".to_owned()],
        }],
        instructions: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["Cook chicken".to_owned()],
        }],
        notes: Some("Don't burn it!".into()),
        tag_ids: vec![tag_main_id, tag_yummy_id],
    };

    let id = domain::recipe::Id::new();
    store
        .create_recipe(id.clone().into(), user.id, recipe)
        .await?;

    let result: ComparableRecipe = store.get_recipe(id.clone().into()).await?.into();

    assert_eq!(id, result.id);
    assert_eq!("Chicken Casserole", result.title);
    assert_eq!(Some(image_id), result.image_id);
    result.assert_ingredients(&[(None, &["Chicken"])]);
    result.assert_instructions(&[(None, &["Cook chicken"])]);
    assert_eq!(Some("Don't burn it!".to_owned()), result.notes);
    assert_eq!(vec!["Main Dish", "Yummy"], result.tags);

    Ok(())
}

pub async fn create_creates_initial_revision(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    let tag_id = tag(&store, &user.id, "Main Dish").await?;

    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        image_id: None,
        ingredients: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["Chicken".to_owned()],
        }],
        instructions: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["Cook chicken".to_owned()],
        }],
        notes: None,
        tag_ids: vec![tag_id],
    };

    let id = domain::recipe::Id::new();
    store
        .create_recipe(id.clone().into(), user.id, recipe)
        .await?;

    // get revisions - there should be one revision: revision 0
    let revisions = store.get_recipe_revisions(id.clone().into()).await?;
    assert_eq!(1, revisions.len());
    assert_eq!(0, revisions[0].revision);

    // try to get revision 0
    let result: ComparableRecipe = store
        .get_recipe_revision(id.clone().into(), 0)
        .await?
        .into();

    assert_eq!(id, result.id);
    assert_eq!("Chicken Casserole", result.title);
    assert_eq!(None, result.image_id);
    result.assert_ingredients(&[(None, &["Chicken"])]);
    result.assert_instructions(&[(None, &["Cook chicken"])]);
    assert_eq!(None, result.notes);
    assert_eq!(vec!["Main Dish"], result.tags);

    Ok(())
}

pub async fn cannot_create_duplicate(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        image_id: None,
        ingredients: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["Chicken".to_owned()],
        }],
        instructions: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["Cook chicken".to_owned()],
        }],
        notes: None,
        tag_ids: vec![],
    };

    let id = domain::recipe::Id::new();
    store
        .create_recipe(id.clone().into(), user.id.clone(), recipe.clone())
        .await?;

    let result = store
        .create_recipe(id.clone().into(), user.id, recipe)
        .await;

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
        image_id: Some(image(&store).await?),
        ingredients: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["Chicken".to_owned()],
        }],
        instructions: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["Cook chicken".to_owned()],
        }],
        notes: Some("Don't burn it!".into()),
        tag_ids: vec![tag_main_id],
    };

    let id = domain::recipe::Id::new();
    store
        .create_recipe(id.clone().into(), user.id.clone(), recipe)
        .await?;

    // get current hash
    let current_hash = store.get_recipe(id.clone().into()).await?.hash;

    // update document
    let new_image_id = image(&store).await?;
    store
        .update_recipe(
            id.clone().into(),
            user.id,
            RecipeDocument {
                title: "Bean Soup".into(),
                image_id: Some(new_image_id.clone()),
                ingredients: vec![domain::recipe::StringifiedBlock {
                    title: None,
                    items: vec!["Beans".to_owned()],
                }],
                instructions: vec![domain::recipe::StringifiedBlock {
                    title: None,
                    items: vec!["Cook beans".to_owned()],
                }],
                notes: None,
                tag_ids: vec![tag_soup_id],
            },
            current_hash,
        )
        .await?;

    let result: ComparableRecipe = store.get_recipe(id.clone().into()).await?.into();

    assert_eq!(id, result.id);
    assert_eq!("Bean Soup", result.title);
    assert_eq!(Some(new_image_id), result.image_id);
    result.assert_ingredients(&[(None, &["Beans"])]);
    result.assert_instructions(&[(None, &["Cook beans"])]);
    assert_eq!(None, result.notes);
    assert_eq!(vec!["Soup"], result.tags);

    Ok(())
}

pub async fn cannot_update_with_bad_hash(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        image_id: None,
        ingredients: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["Chicken".to_owned()],
        }],
        instructions: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["Cook chicken".to_owned()],
        }],
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
                image_id: None,
                ingredients: vec![domain::recipe::StringifiedBlock {
                    title: None,
                    items: vec!["Beans".to_owned()],
                }],
                instructions: vec![domain::recipe::StringifiedBlock {
                    title: None,
                    items: vec!["Cook beans".to_owned()],
                }],
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
        image_id: None,
        ingredients: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["Chicken".to_owned()],
        }],
        instructions: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["Cook chicken".to_owned()],
        }],
        notes: Some("Don't burn it!".into()),
        tag_ids: vec![],
    };

    let id = domain::recipe::Id::new();
    store
        .create_recipe(id.clone().into(), user.id.clone(), recipe)
        .await?;

    // get current hash
    let current_hash = store.get_recipe(id.clone().into()).await?.hash;

    // update document
    store
        .update_recipe(
            id.clone().into(),
            user.id.clone(),
            RecipeDocument {
                title: "Bean Soup".into(),
                image_id: None,
                ingredients: vec![domain::recipe::StringifiedBlock {
                    title: None,
                    items: vec!["Beans".to_owned()],
                }],
                instructions: vec![domain::recipe::StringifiedBlock {
                    title: None,
                    items: vec!["Cook beans".to_owned()],
                }],
                notes: None,
                tag_ids: vec![],
            },
            current_hash,
        )
        .await?;

    // get revisions - there should be two revision
    let revisions = store.get_recipe_revisions(id.clone().into()).await?;
    assert_eq!(2, revisions.len());
    assert_eq!(1, revisions[0].revision);
    assert_eq!(0, revisions[1].revision);

    // try to get revision 1
    let result: ComparableRecipe = store
        .get_recipe_revision(id.clone().into(), 1)
        .await?
        .into();

    assert_eq!(id, result.id);
    assert_eq!("Bean Soup", result.title);
    assert_eq!(None, result.image_id);
    result.assert_ingredients(&[(None, &["Beans"])]);
    result.assert_instructions(&[(None, &["Cook beans"])]);
    assert_eq!(None, result.notes);
    assert!(result.tags.is_empty());

    Ok(())
}

pub async fn handles_updating_unknown_recipe(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    let id = domain::recipe::Id::new();
    let result = store
        .update_recipe(
            id.into(),
            user.id,
            RecipeDocument {
                title: "Bean Soup".into(),
                image_id: None,
                ingredients: vec![domain::recipe::StringifiedBlock {
                    title: None,
                    items: vec!["Chicken".to_owned()],
                }],
                instructions: vec![domain::recipe::StringifiedBlock {
                    title: None,
                    items: vec!["Cook chicken".to_owned()],
                }],
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

// list_recipes

pub async fn can_list_recipes_over_multiple_pages(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;

    let recipe_id_1 = domain::recipe::Id::new();
    let image_id_1 = image(&store).await?;
    store
        .create_recipe(
            recipe_id_1.clone().into(),
            user.id.clone(),
            RecipeDocument {
                title: "Recipe 1".into(),
                image_id: Some(image_id_1.clone()),
                ingredients: vec![],
                instructions: vec![],
                notes: None,
                tag_ids: vec![],
            },
        )
        .await?;

    let recipe_id_2 = domain::recipe::Id::new();
    let image_id_2 = image(&store).await?;
    store
        .create_recipe(
            recipe_id_2.clone().into(),
            user.id.clone(),
            RecipeDocument {
                title: "Recipe 2".into(),
                image_id: Some(image_id_2.clone()),
                ingredients: vec![],
                instructions: vec![],
                notes: None,
                tag_ids: vec![],
            },
        )
        .await?;

    let recipe_id_3 = domain::recipe::Id::new();
    let image_id_3 = image(&store).await?;
    store
        .create_recipe(
            recipe_id_3.clone().into(),
            user.id.clone(),
            RecipeDocument {
                title: "Recipe 3".into(),
                image_id: Some(image_id_3.clone()),
                ingredients: vec![],
                instructions: vec![],
                notes: None,
                tag_ids: vec![],
            },
        )
        .await?;

    // fetch recipes

    let first_page = store
        .list_recipes(domain::filter::Recipe { tag_ids: vec![] }, None)
        .await?;

    assert_eq!(2, first_page.items.len());
    let recipe: ComparableListedRecipe = first_page.items[0].clone().into();
    assert_eq!(recipe_id_1, recipe.id);
    assert_eq!("Recipe 1", recipe.title);
    assert_eq!(Some(image_id_1), recipe.image_id);

    let recipe: ComparableListedRecipe = first_page.items[1].clone().into();
    assert_eq!(recipe_id_2, recipe.id);
    assert_eq!("Recipe 2", recipe.title);
    assert_eq!(Some(image_id_2), recipe.image_id);

    let second_page = store
        .list_recipes(domain::filter::Recipe { tag_ids: vec![] }, first_page.next)
        .await?;

    assert!(second_page.next.is_none());
    assert_eq!(1, second_page.items.len());
    let recipe: ComparableListedRecipe = second_page.items[0].clone().into();
    assert_eq!(recipe_id_3, recipe.id);
    assert_eq!("Recipe 3", recipe.title);
    assert_eq!(Some(image_id_3), recipe.image_id);

    Ok(())
}

pub async fn can_list_with_tag_filter(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;

    let tag1 = tag(&store, &user.id, "Tag1").await?;
    let tag2 = tag(&store, &user.id, "Tag2").await?;
    let tag3 = tag(&store, &user.id, "Tag3").await?;
    let tag4 = tag(&store, &user.id, "Tag4").await?;

    let recipe_id_1 = domain::recipe::Id::new();
    store
        .create_recipe(
            recipe_id_1.clone().into(),
            user.id.clone(),
            RecipeDocument {
                title: "Good Chicken".into(),
                image_id: None,
                ingredients: vec![],
                instructions: vec![],
                notes: None,
                tag_ids: vec![tag1.clone(), tag2.clone()],
            },
        )
        .await?;

    let recipe_id_2 = domain::recipe::Id::new();
    store
        .create_recipe(
            recipe_id_2.clone().into(),
            user.id.clone(),
            RecipeDocument {
                title: "Rabbit?".into(),
                image_id: None,
                ingredients: vec![],
                instructions: vec![],
                notes: None,
                tag_ids: vec![tag3],
            },
        )
        .await?;

    let recipe_id_3 = domain::recipe::Id::new();
    store
        .create_recipe(
            recipe_id_3.clone().into(),
            user.id.clone(),
            RecipeDocument {
                title: "Baked beans".into(),
                image_id: None,
                ingredients: vec![],
                instructions: vec![],
                notes: None,
                tag_ids: vec![tag2.clone(), tag4.clone()],
            },
        )
        .await?;

    // fetch recipes

    let filter = domain::filter::Recipe {
        tag_ids: vec![tag1, tag2],
    };

    let result = store.list_recipes(filter.clone(), None).await?;

    assert_eq!(1, result.items.len());
    assert!(result.next.is_none());

    let recipe: ComparableListedRecipe = result.items[0].clone().into();
    assert_eq!(recipe_id_1, recipe.id);
    assert_eq!("Good Chicken", recipe.title);
    assert_eq!(None, recipe.image_id);

    Ok(())
}

pub async fn can_list_recipe_if_it_has_tag_with_no_filter(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;

    let tag1 = tag(&store, &user.id, "Tag1").await?;
    let tag2 = tag(&store, &user.id, "Tag2").await?;

    let recipe_id_1 = domain::recipe::Id::new();
    store
        .create_recipe(
            recipe_id_1.clone().into(),
            user.id.clone(),
            RecipeDocument {
                title: "Good Chicken".into(),
                image_id: None,
                ingredients: vec![],
                instructions: vec![],
                notes: None,
                tag_ids: vec![tag1.clone(), tag2.clone()],
            },
        )
        .await?;

    // fetch recipes

    let filter = domain::filter::Recipe { tag_ids: vec![] };

    let result = store.list_recipes(filter.clone(), None).await?;

    assert_eq!(1, result.items.len());
    assert!(result.next.is_none());

    assert_eq!(recipe_id_1, result.items[0].id);

    Ok(())
}

pub async fn filters_properly_if_has_multiple_tags_matching_count_but_wrong_tags(
    store: datastore::Pool,
) -> Result<()> {
    let user = user(&store).await?;

    let tag1 = tag(&store, &user.id, "Tag1").await?;
    let tag2 = tag(&store, &user.id, "Tag2").await?;
    let tag3 = tag(&store, &user.id, "Tag3").await?;
    let tag4 = tag(&store, &user.id, "Tag4").await?;

    let recipe_id_1 = domain::recipe::Id::new();
    store
        .create_recipe(
            recipe_id_1.clone().into(),
            user.id.clone(),
            RecipeDocument {
                title: "Good Chicken".into(),
                image_id: None,
                ingredients: vec![],
                instructions: vec![],
                notes: None,
                tag_ids: vec![tag1.clone(), tag2.clone()],
            },
        )
        .await?;

    // fetch recipes

    let filter = domain::filter::Recipe {
        tag_ids: vec![tag3, tag4],
    };

    let result = store.list_recipes(filter.clone(), None).await?;

    assert_eq!(0, result.items.len());
    assert!(result.next.is_none());

    Ok(())
}

// get_revision

pub async fn cannot_get_non_existent_revision(store: datastore::Pool) -> Result<()> {
    let user = user(&store).await?;
    // save recipe
    let recipe = RecipeDocument {
        title: "Chicken Casserole".into(),
        image_id: None,
        ingredients: vec![],
        instructions: vec![],
        notes: Some("Don't burn it!".into()),
        tag_ids: vec![],
    };

    let id = domain::recipe::Id::new();
    store
        .create_recipe(id.clone().into(), user.id, recipe)
        .await?;

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
    let id = domain::recipe::Id::new();

    let image_id_1 = image(&store).await?;
    let image_id_2 = image(&store).await?;
    let image_id_3 = image(&store).await?;

    // create recipe
    let recipe = RecipeDocument {
        title: "one".into(),
        image_id: Some(image_id_1.clone()),
        ingredients: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["two".to_owned()],
        }],
        instructions: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["three".to_owned()],
        }],
        notes: Some("four".into()),
        tag_ids: vec![tag(&store, &user.id, "Tag1").await?],
    };
    store
        .create_recipe(id.clone().into(), user.id.clone(), recipe)
        .await?;

    // update recipe
    let recipe = RecipeDocument {
        title: "five".into(),
        image_id: Some(image_id_2.clone()),
        ingredients: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["six".to_owned()],
        }],
        instructions: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["seven".to_owned()],
        }],
        notes: Some("eight".into()),
        tag_ids: vec![tag(&store, &user.id, "Tag2").await?],
    };
    let hash = store.get_recipe(id.clone().into()).await?.hash;
    store
        .update_recipe(id.clone().into(), user.id.clone(), recipe, hash)
        .await?;

    // update recipe again
    let recipe = RecipeDocument {
        title: "nine".into(),
        image_id: Some(image_id_3.clone()),
        ingredients: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["ten".to_owned()],
        }],
        instructions: vec![domain::recipe::StringifiedBlock {
            title: None,
            items: vec!["eleven".to_owned()],
        }],
        notes: Some("twelve".into()),
        tag_ids: vec![tag(&store, &user.id, "Tag3").await?],
    };
    let hash = store.get_recipe(id.clone().into()).await?.hash;
    store
        .update_recipe(id.clone().into(), user.id.clone(), recipe, hash)
        .await?;

    // now validate revision history - there should be three revisions
    let revisions = store.get_recipe_revisions(id.clone().into()).await?;
    assert_eq!(3, revisions.len());
    assert_eq!(2, revisions[0].revision);
    assert_eq!(1, revisions[1].revision);
    assert_eq!(0, revisions[2].revision);

    // check revision 0
    let result: ComparableRecipe = store
        .get_recipe_revision(id.clone().into(), 0)
        .await?
        .into();
    assert_eq!("one", result.title);
    assert_eq!(Some(image_id_1), result.image_id);
    result.assert_ingredients(&[(None, &["two"])]);
    result.assert_instructions(&[(None, &["three"])]);
    assert_eq!(Some("four".into()), result.notes);
    assert_eq!(vec!["Tag1"], result.tags);

    // check revision 1
    let result: ComparableRecipe = store
        .get_recipe_revision(id.clone().into(), 1)
        .await?
        .into();
    assert_eq!("five", result.title);
    assert_eq!(Some(image_id_2), result.image_id);
    result.assert_ingredients(&[(None, &["six"])]);
    result.assert_instructions(&[(None, &["seven"])]);
    assert_eq!(Some("eight".into()), result.notes);
    assert_eq!(vec!["Tag2"], result.tags);

    // check revision 2
    let result: ComparableRecipe = store
        .get_recipe_revision(id.clone().into(), 2)
        .await?
        .into();
    assert_eq!("nine", result.title);
    assert_eq!(Some(image_id_3), result.image_id);
    result.assert_ingredients(&[(None, &["ten"])]);
    result.assert_instructions(&[(None, &["eleven"])]);
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
