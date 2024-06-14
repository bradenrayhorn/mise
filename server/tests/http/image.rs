use super::{
    responses,
    setup::{self},
};
use anyhow::Result;
use axum::http::HeaderValue;
use base64::Engine;
use reqwest::StatusCode;

const JPEG: &str = "/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/wAALCAABAAEBAREA/8QAFAABAAAAAAAAAAAAAAAAAAAACf/EABQQAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQEAAD8AKp//2Q==";

#[tokio::test]
async fn can_create_and_get_image() -> Result<()> {
    let harness = setup::with_auth().await?;

    let base64_engine = base64::engine::general_purpose::STANDARD;

    let body = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(base64_engine.decode(JPEG.as_bytes())?),
    );
    let response = harness
        .post("/api/v1/images")
        .multipart(body)
        .send()
        .await?;

    assert_eq!(StatusCode::OK, response.status());
    let id = response.json::<responses::CreateImage>().await?.data;

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
