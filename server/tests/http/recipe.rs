use super::{requests, responses, setup};
use anyhow::Result;
use mise::domain;
use reqwest::StatusCode;

#[tokio::test]
async fn cannot_get_unknown_recipe() -> Result<()> {
    let harness = setup::with_auth().await?;

    let random_id = domain::recipe::Id::new();

    let response = harness
        .get(&format!("/api/v1/recipes/{random_id}"))
        .send()
        .await?;

    assert_eq!(StatusCode::NOT_FOUND, response.status());

    Ok(())
}

#[tokio::test]
async fn can_create_and_get_recipe() -> Result<()> {
    let harness = setup::with_auth().await?;

    let image_id = harness.create_image().await?;
    let tag_id = harness.create_tag("Main Dish").await?;

    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Parm".into(),
            image_id: Some(image_id.clone()),
            ingredients: requests::IngredientBlock::new(&[(
                None,
                &["One chicken", "Parmesan cheese"],
            )]),
            instructions: requests::InstructionBlock::new(&[(
                None,
                &["Broil the chicken", "Add the parmesan"],
            )]),
            notes: Some("Best served hot!".into()),
            tag_ids: vec![tag_id.clone()],
        })
        .send()
        .await?;

    assert_eq!(StatusCode::OK, response.status());
    let id = response.json::<responses::CreateRecipe>().await?.data;

    // try to get recipe
    let response = harness.get(&format!("/api/v1/recipes/{id}")).send().await?;
    assert_eq!(StatusCode::OK, response.status());

    let result = response.json::<responses::GetRecipe>().await?.data;

    assert_eq!("Chicken Parm", result.title);
    assert_eq!(Some(image_id), result.image_id);
    assert_eq!(
        vec![responses::IngredientBlock {
            title: None,
            ingredients: vec!["One chicken".into(), "Parmesan cheese".into()]
        }],
        result.ingredient_blocks
    );
    assert_eq!(
        vec![responses::InstructionBlock {
            title: None,
            instructions: vec!["Broil the chicken".into(), "Add the parmesan".into()]
        }],
        result.instruction_blocks
    );
    assert_eq!(Some("Best served hot!".into()), result.notes);
    assert_eq!(
        vec![responses::TagOnRecipe {
            id: tag_id,
            name: "Main Dish".into()
        }],
        result.tags
    );

    Ok(())
}

#[tokio::test]
async fn can_create_and_update_recipe() -> Result<()> {
    let harness = setup::with_auth().await?;

    let image_id_1 = harness.create_image().await?;
    let image_id_2 = harness.create_image().await?;

    // create the recipe
    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Parm".into(),
            image_id: Some(image_id_1.clone()),
            ingredients: requests::IngredientBlock::new(&[(
                None,
                &["One chicken", "Parmesan cheese"],
            )]),
            instructions: requests::InstructionBlock::new(&[(
                None,
                &["Broil the chicken", "Add the parmesan"],
            )]),
            notes: Some("Best served hot!".into()),
            tag_ids: vec![harness.create_tag("Main Dish").await?],
        })
        .send()
        .await?;

    assert_eq!(StatusCode::OK, response.status());
    let id = response.json::<responses::CreateRecipe>().await?.data;

    // get the recipe to find the hash
    let response = harness.get(&format!("/api/v1/recipes/{id}")).send().await?;
    assert_eq!(StatusCode::OK, response.status());
    let hash = response.json::<responses::GetRecipe>().await?.data.hash;

    // send an update
    let tag_id = harness.create_tag("Salads").await?;

    let response = harness
        .put(&format!("/api/v1/recipes/{id}"))
        .json(&requests::UpdateRecipe {
            previous_hash: hash,
            title: "One-Step Salad".into(),
            image_id: Some(image_id_2.clone()),
            ingredients: requests::IngredientBlock::new(&[(None, &["salad"])]),
            instructions: requests::InstructionBlock::new(&[(None, &["serve"])]),
            notes: None,
            tag_ids: vec![tag_id.clone()],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());

    // try to get recipe
    let response = harness.get(&format!("/api/v1/recipes/{id}")).send().await?;
    assert_eq!(StatusCode::OK, response.status());

    let result = response.json::<responses::GetRecipe>().await?.data;

    assert_eq!("One-Step Salad", result.title);
    assert_eq!(Some(image_id_2), result.image_id);
    assert_eq!(
        vec![responses::IngredientBlock {
            title: None,
            ingredients: vec!["salad".into()]
        }],
        result.ingredient_blocks
    );
    assert_eq!(
        vec![responses::InstructionBlock {
            title: None,
            instructions: vec!["serve".into()]
        }],
        result.instruction_blocks
    );
    assert_eq!(None, result.notes);
    assert_eq!(
        vec![responses::TagOnRecipe {
            id: tag_id,
            name: "Salads".into()
        }],
        result.tags
    );

    Ok(())
}

#[tokio::test]
async fn can_list_recipes() -> Result<()> {
    let harness = setup::with_auth().await?;

    let image_id_1 = harness.create_image().await?;
    let image_id_3 = harness.create_image().await?;

    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Alpha".into(),
            image_id: Some(image_id_1.clone()),
            ingredients: requests::IngredientBlock::new(&[]),
            instructions: requests::InstructionBlock::new(&[]),
            notes: None,
            tag_ids: vec![],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());
    let recipe_id_1 = response.json::<responses::CreateRecipe>().await?.data;

    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Gamma".into(),
            image_id: None,
            ingredients: requests::IngredientBlock::new(&[]),
            instructions: requests::InstructionBlock::new(&[]),
            notes: None,
            tag_ids: vec![],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());
    let recipe_id_2 = response.json::<responses::CreateRecipe>().await?.data;

    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Beta".into(),
            image_id: Some(image_id_3.clone()),
            ingredients: requests::IngredientBlock::new(&[]),
            instructions: requests::InstructionBlock::new(&[]),
            notes: None,
            tag_ids: vec![],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());
    let recipe_id_3 = response.json::<responses::CreateRecipe>().await?.data;

    // get page one
    let response = harness.get("/api/v1/recipes").send().await?;
    assert_eq!(StatusCode::OK, response.status());

    let page_1_result = response.json::<responses::ListRecipes>().await?;
    assert_eq!(
        vec![
            responses::ListedRecipe {
                id: recipe_id_1.into(),
                title: "Chicken Alpha".into(),
                image_id: Some(image_id_1),
            },
            responses::ListedRecipe {
                id: recipe_id_3.into(),
                title: "Chicken Beta".into(),
                image_id: Some(image_id_3),
            }
        ],
        page_1_result.data
    );

    // get page two
    let response = harness
        .get(&format!(
            "/api/v1/recipes?next={}",
            page_1_result.next.unwrap()
        ))
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());

    let page_2_result = response.json::<responses::ListRecipes>().await?;
    assert!(page_2_result.next.is_none());
    assert_eq!(
        vec![responses::ListedRecipe {
            id: recipe_id_2.into(),
            title: "Chicken Gamma".into(),
            image_id: None,
        },],
        page_2_result.data
    );

    Ok(())
}

#[tokio::test]
async fn can_list_recipes_with_filters() -> Result<()> {
    let harness = setup::with_auth().await?;

    let tag_1 = harness.create_tag("Tag1").await?;
    let tag_2 = harness.create_tag("Tag2").await?;

    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Alpha".into(),
            image_id: None,
            ingredients: requests::IngredientBlock::new(&[]),
            instructions: requests::InstructionBlock::new(&[]),
            notes: None,
            tag_ids: vec![tag_1.clone()],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());
    let recipe_id_1 = response.json::<responses::CreateRecipe>().await?.data;

    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Gamma Alpha".into(),
            image_id: None,
            ingredients: requests::IngredientBlock::new(&[]),
            instructions: requests::InstructionBlock::new(&[]),
            notes: None,
            tag_ids: vec![tag_1.clone()],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());
    let recipe_id_2 = response.json::<responses::CreateRecipe>().await?.data;

    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Beta".into(),
            image_id: None,
            ingredients: requests::IngredientBlock::new(&[]),
            instructions: requests::InstructionBlock::new(&[]),
            notes: None,
            tag_ids: vec![tag_2.clone()],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());

    // get page one
    let response = harness
        .get(&format!("/api/v1/recipes?name=Alpha&tag_ids={tag_1}"))
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());

    let page_1_result = response.json::<responses::ListRecipes>().await?;
    assert_eq!(
        vec![
            responses::ListedRecipe {
                id: recipe_id_1.into(),
                title: "Chicken Alpha".into(),
                image_id: None,
            },
            responses::ListedRecipe {
                id: recipe_id_2.into(),
                title: "Chicken Gamma Alpha".into(),
                image_id: None,
            }
        ],
        page_1_result.data
    );

    // get page two
    let response = harness
        .get(&format!(
            "/api/v1/recipes?name=Alpha&tag_ids={tag_1}&next={}",
            page_1_result.next.unwrap(),
        ))
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());

    let page_2_result = response.json::<responses::ListRecipes>().await?;
    assert!(page_2_result.next.is_none());
    assert!(page_2_result.data.is_empty());

    Ok(())
}
