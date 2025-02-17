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

#[tokio::test]
async fn supplies_etag_for_cache() -> Result<()> {
    let harness = setup::with_auth().await?;

    let id = harness.create_image().await?;

    let response = harness.get(&format!("/api/v1/images/{id}")).send().await?;
    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(
        Some(&HeaderValue::from_static("image/jpeg")),
        response.headers().get("Content-Type")
    );

    let etag = response.headers().get("ETag").unwrap().to_str().unwrap();

    // when using the If-None-Match header, no response is supplied
    let second_response = harness
        .get(&format!("/api/v1/images/{id}"))
        .header("If-None-Match", etag)
        .send()
        .await?;
    assert_eq!(StatusCode::NOT_MODIFIED, second_response.status());
    assert_eq!(
        Some(&HeaderValue::from_str(etag).unwrap()),
        second_response.headers().get("ETag")
    );
    assert_eq!(second_response.content_length(), Some(0));

    Ok(())
}
