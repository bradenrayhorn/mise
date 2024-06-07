use super::{
    responses,
    setup::{self},
};
use anyhow::Result;
use reqwest::StatusCode;

#[tokio::test]
async fn can_createt_and_get_tags() -> Result<()> {
    let harness = setup::with_auth().await?;

    let main_id = harness.create_tag("Main Dish").await?;
    let side_id = harness.create_tag("Side Dish").await?;

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
