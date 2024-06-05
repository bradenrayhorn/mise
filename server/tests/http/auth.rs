use super::setup;
use anyhow::Result;
use reqwest::StatusCode;

#[tokio::test]
async fn can_get_me() -> Result<()> {
    let mut harness = setup::harness().await?;

    harness.authenticate("thomas").await?;

    let response = harness.get("/api/v1/auth/me").send().await?;

    assert_eq!(StatusCode::OK, response.status());

    Ok(())
}
