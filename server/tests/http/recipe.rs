use super::{requests, responses, setup};
use anyhow::Result;
use reqwest::StatusCode;

#[tokio::test]
async fn cannot_get_unknown_recipe() -> Result<()> {
    let harness = setup::with_auth().await?;

    let random_id = uuid::Uuid::new_v4();

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

    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Parm".into(),
            ingredients: "\
                - One chicken\n\
                - Parmesan cheese\
                "
            .into(),
            instructions: "\
                - Broil the chicken\n\
                - Add the parmesan\
                "
            .into(),
            notes: Some("Best served hot!".into()),
            tag_ids: vec![harness.create_tag("Main Dish").await?],
        })
        .send()
        .await?;

    assert_eq!(StatusCode::OK, response.status());
    let id = response.json::<responses::Id>().await?.data;

    // try to get recipe
    let response = harness.get(&format!("/api/v1/recipes/{id}")).send().await?;
    assert_eq!(StatusCode::OK, response.status());

    let result = response.json::<responses::GetRecipe>().await?.data;

    assert_eq!("Chicken Parm", result.title);
    assert_eq!(
        vec![responses::Ingredients {
            title: None,
            ingredients: vec!["One chicken".into(), "Parmesan cheese".into()]
        }],
        result.ingredient_blocks
    );
    assert_eq!(
        vec![responses::Instructions {
            title: None,
            instructions: vec!["Broil the chicken".into(), "Add the parmesan".into()]
        }],
        result.instruction_blocks
    );
    assert_eq!(Some("Best served hot!".into()), result.notes);
    assert_eq!(vec!["Main Dish"], result.tags);

    Ok(())
}

#[tokio::test]
async fn can_create_and_update_recipe() -> Result<()> {
    let harness = setup::with_auth().await?;

    // create the recipe
    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Parm".into(),
            ingredients: "\
                - One chicken\n\
                - Parmesan cheese\
                "
            .into(),
            instructions: "\
                - Broil the chicken\n\
                - Add the parmesan\
                "
            .into(),
            notes: Some("Best served hot!".into()),
            tag_ids: vec![harness.create_tag("Main Dish").await?],
        })
        .send()
        .await?;

    assert_eq!(StatusCode::OK, response.status());
    let id = response.json::<responses::Id>().await?.data;

    // get the recipe to find the hash
    let response = harness.get(&format!("/api/v1/recipes/{id}")).send().await?;
    assert_eq!(StatusCode::OK, response.status());
    let hash = response.json::<responses::GetRecipe>().await?.data.hash;

    // send an update
    let response = harness
        .put(&format!("/api/v1/recipes/{id}"))
        .json(&requests::UpdateRecipe {
            previous_hash: hash,
            title: "One-Step Salad".into(),
            ingredients: "\
                - salad\
                "
            .into(),
            instructions: "\
                - Serve\
                "
            .into(),
            notes: None,
            tag_ids: vec![harness.create_tag("Salads").await?],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());

    // try to get recipe
    let response = harness.get(&format!("/api/v1/recipes/{id}")).send().await?;
    assert_eq!(StatusCode::OK, response.status());

    let result = response.json::<responses::GetRecipe>().await?.data;

    assert_eq!("One-Step Salad", result.title);
    assert_eq!(
        vec![responses::Ingredients {
            title: None,
            ingredients: vec!["salad".into()]
        }],
        result.ingredient_blocks
    );
    assert_eq!(
        vec![responses::Instructions {
            title: None,
            instructions: vec!["Serve".into()]
        }],
        result.instruction_blocks
    );
    assert_eq!(None, result.notes);
    assert_eq!(vec!["Salads"], result.tags);

    Ok(())
}

#[tokio::test]
async fn can_list_recipes() -> Result<()> {
    let harness = setup::with_auth().await?;

    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Alpha".into(),
            ingredients: "- word".into(),
            instructions: "- word".into(),
            notes: None,
            tag_ids: vec![],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());
    let recipe_id_1 = response.json::<responses::Id>().await?.data;

    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Gamma".into(),
            ingredients: "- word".into(),
            instructions: "- word".into(),
            notes: None,
            tag_ids: vec![],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());
    let recipe_id_2 = response.json::<responses::Id>().await?.data;

    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Beta".into(),
            ingredients: "- word".into(),
            instructions: "- word".into(),
            notes: None,
            tag_ids: vec![],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());
    let recipe_id_3 = response.json::<responses::Id>().await?.data;

    // get page one
    let response = harness.get("/api/v1/recipes").send().await?;
    assert_eq!(StatusCode::OK, response.status());

    let page_1_result = response.json::<responses::ListRecipes>().await?;
    assert_eq!(
        vec![
            responses::ListedRecipe {
                id: recipe_id_1.into(),
                title: "Chicken Alpha".into(),
            },
            responses::ListedRecipe {
                id: recipe_id_3.into(),
                title: "Chicken Beta".into(),
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
            ingredients: "- word".into(),
            instructions: "- word".into(),
            notes: None,
            tag_ids: vec![tag_1],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());
    let recipe_id_1 = response.json::<responses::Id>().await?.data;

    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Gamma Alpha".into(),
            ingredients: "- word".into(),
            instructions: "- word".into(),
            notes: None,
            tag_ids: vec![tag_2],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());
    let recipe_id_2 = response.json::<responses::Id>().await?.data;

    let response = harness
        .post("/api/v1/recipes")
        .json(&requests::CreateRecipe {
            title: "Chicken Beta".into(),
            ingredients: "- word".into(),
            instructions: "- word".into(),
            notes: None,
            tag_ids: vec![],
        })
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());

    // get page one
    let response = harness
        .get(&format!(
            "/api/v1/recipes?name=Alpha&tag_ids={tag_1},{tag_2}"
        ))
        .send()
        .await?;
    assert_eq!(StatusCode::OK, response.status());

    let page_1_result = response.json::<responses::ListRecipes>().await?;
    assert_eq!(
        vec![
            responses::ListedRecipe {
                id: recipe_id_1.into(),
                title: "Chicken Alpha".into(),
            },
            responses::ListedRecipe {
                id: recipe_id_2.into(),
                title: "Chicken Gamma Alpha".into(),
            }
        ],
        page_1_result.data
    );

    // get page two
    let response = harness
        .get(&format!(
            "/api/v1/recipes?name=Alpha&tag_ids={tag_1},{tag_2}&next={}",
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
