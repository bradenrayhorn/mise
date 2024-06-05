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

    Ok(())
}
