use super::setup::{self};
use anyhow::Result;
use axum::http::HeaderValue;
use reqwest::StatusCode;

#[tokio::test]
async fn can_create_and_get_image() -> Result<()> {
    let harness = setup::with_auth().await?;

    let id = harness.create_image().await?;

    let response = harness.get(&format!("/api/v1/images/{id}")).send().await?;
    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(
        Some(&HeaderValue::from_static("image/jpeg")),
        response.headers().get("Content-Type")
    );
    let image_response = response.bytes().await?;
    assert!(image_response.len() > 1);

    Ok(())
}
