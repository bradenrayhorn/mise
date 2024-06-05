use super::{
    requests, responses,
    setup::{self, Harness},
};
use anyhow::Result;
use reqwest::StatusCode;

async fn create_tag(harness: &Harness, name: &str) -> Result<i64> {
    let response = harness
        .post(&format!("/api/v1/tags"))
        .json(&requests::CreateTag { name: name.into() })
        .send()
        .await?;

    assert_eq!(StatusCode::OK, response.status());
    Ok(response.json::<responses::CreateTag>().await?.data)
}

#[tokio::test]
async fn can_createt_and_get_tags() -> Result<()> {
    let harness = setup::with_auth().await?;

    let main_id = create_tag(&harness, "Main Dish").await?;
    let side_id = create_tag(&harness, "Side Dish").await?;

    let response = harness.get("/api/v1/tags").send().await?;
    assert_eq!(StatusCode::OK, response.status());

    let result = response.json::<responses::GetTags>().await?.data;

    assert_eq!(
        vec![
            responses::Tag {
                id: main_id,
                name: "Main Dish".into(),
            },
            responses::Tag {
                id: side_id,
                name: "Side Dish".into(),
            }
        ],
        result
    );

    Ok(())
}
